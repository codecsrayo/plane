// src/routes/intake.rs
//! Intake endpoints (issue inbox).
//!
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intakes/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/intakes/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intake-issues/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/intake-issues/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{
        cycle_issues, intake_issues, intakes, issue_assignees, issue_attachments, issue_labels,
        issue_links, issues, module_issues, states,
    },
    error::AppError,
    routes::issues::{sync_assignees, sync_labels},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Aggregate helpers ─────────────────────────────────────────────────────────

#[derive(Default)]
struct IssueAggregates {
    label_ids: Vec<Uuid>,
    assignee_ids: Vec<Uuid>,
    module_ids: Vec<Uuid>,
    cycle_id: Option<Uuid>,
    sub_issues_count: i64,
    attachment_count: i64,
    link_count: i64,
}

/// Batch-loads aggregate data for a slice of issue IDs to avoid N+1.
/// Parity with Django `IssueDetailSerializer` annotated fields.
async fn fetch_issue_aggregates(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, IssueAggregates>, AppError> {
    if issue_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let mut map: std::collections::HashMap<Uuid, IssueAggregates> =
        issue_ids.iter().map(|&id| (id, IssueAggregates::default())).collect();

    for row in issue_labels::Entity::find()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(e) = map.get_mut(&row.issue_id) {
            e.label_ids.push(row.label_id);
        }
    }

    for row in issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(e) = map.get_mut(&row.issue_id) {
            e.assignee_ids.push(row.assignee_id);
        }
    }

    for row in module_issues::Entity::find()
        .filter(module_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(module_issues::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(e) = map.get_mut(&row.issue_id) {
            e.module_ids.push(row.module_id);
        }
    }

    for row in cycle_issues::Entity::find()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(e) = map.get_mut(&row.issue_id) {
            if e.cycle_id.is_none() {
                e.cycle_id = Some(row.cycle_id);
            }
        }
    }

    for row in issues::Entity::find()
        .filter(issues::Column::ParentId.is_in(issue_ids.to_vec()))
        .filter(issues::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(parent_id) = row.parent_id {
            if let Some(e) = map.get_mut(&parent_id) {
                e.sub_issues_count += 1;
            }
        }
    }

    for row in issue_attachments::Entity::find()
        .filter(issue_attachments::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_attachments::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(e) = map.get_mut(&row.issue_id) {
            e.attachment_count += 1;
        }
    }

    for row in issue_links::Entity::find()
        .filter(issue_links::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_links::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        if let Some(e) = map.get_mut(&row.issue_id) {
            e.link_count += 1;
        }
    }

    Ok(map)
}

// ── Intake status constants (matches Django IntakeIssue.STATUS_CHOICES) ───────
pub const STATUS_PENDING: i32 = -2;
pub const STATUS_REJECTED: i32 = -1;
pub const STATUS_SNOOZED: i32 = 0;
pub const STATUS_ACCEPTED: i32 = 1;
pub const STATUS_DUPLICATE: i32 = 2;

// ── Source constants (matches Django SourceType.TextChoices) ──────────────────
// See: apps/api/plane/db/models/intake.py → class SourceType.
// Django only defines IN_APP for now; the create handler hard-codes this
// value and ignores the `source` provided by the client (base.py:270).
pub const SOURCE_IN_APP: &str = "IN_APP";

// ── Priority constants (matches Django IssueCreateSerializer validation) ──────
pub const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "urgent", "none"];

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakeResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl IntakeResponse {
    fn from_model(m: intakes::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            is_default: m.is_default,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
/// Issue detail nested within IntakeIssueResponse.
///
/// Parity with Django `IssueDetailSerializer` (serializers/issue.py:924-935)
/// used by `IntakeIssueDetailSerializer` (serializers/intake.py:93-107).
/// The frontend reads `inboxIssue.issue.created_by` (root.tsx:77,80),
/// `issue.name`, `issue.description_html`, `issue.priority`, `issue.sequence_id`,
/// `issue.label_ids`, `issue.assignee_ids` — all mandatory.
pub struct IntakeIssueNestedIssue {
    pub id: Uuid,
    pub name: String,
    pub description_html: String,
    pub state_id: Option<Uuid>,
    pub priority: String,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub sort_order: f64,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub archived_at: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    /// Django serializes `created_by` FK as UUID without `_id`.
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub is_draft: bool,
    /// Django `is_intake` is always false for issues in the intake queue.
    /// It becomes true only after the issue is accepted and moved out of intake.
    pub is_intake: bool,
    pub estimate_point: Option<Uuid>,
    pub cycle_id: Option<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub sub_issues_count: i64,
    pub attachment_count: i64,
    pub link_count: i64,
}

impl IntakeIssueNestedIssue {
    fn from_model(m: &issues::Model) -> Self {
        Self::from_model_with_aggregates(m, None)
    }

    fn from_model_with_aggregates(m: &issues::Model, agg: Option<&IssueAggregates>) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            description_html: m.description_html.clone(),
            state_id: m.state_id,
            priority: m.priority.clone(),
            sequence_id: m.sequence_id,
            project_id: m.project_id,
            parent_id: m.parent_id,
            sort_order: m.sort_order,
            start_date: m.start_date,
            target_date: m.target_date,
            completed_at: m.completed_at,
            archived_at: m.archived_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            is_draft: m.is_draft,
            is_intake: false,
            estimate_point: m.estimate_point_id,
            cycle_id: agg.and_then(|a| a.cycle_id),
            label_ids: agg.map(|a| a.label_ids.clone()).unwrap_or_default(),
            assignee_ids: agg.map(|a| a.assignee_ids.clone()).unwrap_or_default(),
            module_ids: agg.map(|a| a.module_ids.clone()).unwrap_or_default(),
            sub_issues_count: agg.map(|a| a.sub_issues_count).unwrap_or(0),
            attachment_count: agg.map(|a| a.attachment_count).unwrap_or(0),
            link_count: agg.map(|a| a.link_count).unwrap_or(0),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakeIssueResponse {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub intake_id: Uuid,
    pub status: i32,
    pub source: Option<String>,
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duplicate_to_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    /// Associated issue details — required by frontend. See
    /// IntakeIssueNestedIssue.
    pub issue: IntakeIssueNestedIssue,
}

impl IntakeIssueResponse {
    fn from_joined(ii: intake_issues::Model, issue: &issues::Model) -> Self {
        Self::from_joined_with_aggregates(ii, issue, None)
    }

    fn from_joined_with_aggregates(
        ii: intake_issues::Model,
        issue: &issues::Model,
        agg: Option<&IssueAggregates>,
    ) -> Self {
        Self {
            id: ii.id,
            issue_id: ii.issue_id,
            intake_id: ii.intake_id,
            status: ii.status,
            source: ii.source,
            snoozed_till: ii.snoozed_till,
            duplicate_to_id: ii.duplicate_to_id,
            project_id: ii.project_id,
            workspace_id: ii.workspace_id,
            created_by_id: ii.created_by_id,
            created_at: ii.created_at,
            issue: IntakeIssueNestedIssue::from_model_with_aggregates(issue, agg),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIntakeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Body of POST /intake-issues/ — parity with Django IntakeIssueViewSet.create
/// (`apps/api/plane/app/views/intake/base.py:222-284`).
///
/// The frontend sends the nested payload `{source, issue: {...}}`. Django **ignores**
/// any `intake_id` provided in the body and auto-resolves the project's
/// intake (`base.py:264`). The client's `source` is also discarded and
/// hard-coded to `IN_APP` (`base.py:270`). We keep `source` in the DTO for
/// logging/telemetry, but it is never used for persistence.
///
/// Additional fields the frontend may send in `issue` (parent_id,
/// start_date, target_date, estimate_point, type_id, assignee_ids, label_ids)
/// are processed by Django via `IssueCreateSerializer`. Here they are silently
/// discarded by serde — see TODO in handler.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeIssueRequest {
    /// Ignored for Django parity (hard-code `IN_APP`). Accepted to not
    /// break clients that send it.
    #[serde(default)]
    pub source: Option<String>,
    pub issue: CreateIntakeIssueBody,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeIssueBody {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    // Django IssueCreateSerializer also accepts these optional fields.
    pub parent_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub estimate_point: Option<Uuid>,
    pub type_id: Option<Uuid>,
    #[serde(default)]
    pub assignee_ids: Vec<Uuid>,
    #[serde(default)]
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIntakeIssueRequest {
    pub status: Option<i32>,
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duplicate_to_id: Option<Uuid>,
    pub source: Option<String>,
}

// ── GET /intakes/ ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Intakes list")),
    security(("TokenAuth" = []))
)]
pub async fn list_intakes(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<IntakeResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = intakes::Entity::find()
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(intakes::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(IntakeResponse::from_model).collect()))
}

// ── POST /intakes/ ────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Intake created"),
        (status = 400, description = "Validation error"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateIntakeRequest>,
) -> Result<(StatusCode, Json<IntakeResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    // Explicit created_at/updated_at (NOT NULL without DEFAULT).
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let intake = intakes::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or_default()),
        is_default: Set(false),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        view_props: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(IntakeResponse::from_model(intake))))
}

// ── GET /intakes/{pk}/ ────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Intake ID"),
    ),
    responses(
        (status = 200, description = "Intake detail"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<IntakeResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let intake = intakes::Entity::find_by_id(pk)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(IntakeResponse::from_model(intake)))
}

// ── PATCH /intakes/{pk}/ ──────────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Intake ID"),
    ),
    responses(
        (status = 200, description = "Intake updated"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIntakeRequest>,
) -> Result<Json<IntakeResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let intake = intakes::Entity::find_by_id(pk)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: intakes::ActiveModel = intake.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(IntakeResponse::from_model(updated)))
}

// ── DELETE /intakes/{pk}/ ─────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Intake ID"),
    ),
    responses((status = 204, description = "Deleted")),
    security(("TokenAuth" = []))
)]
pub async fn delete_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let intake = intakes::Entity::find_by_id(pk)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: intakes::ActiveModel = intake.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /intake-issues/ ───────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Intake issues list")),
    security(("TokenAuth" = []))
)]
pub async fn list_intake_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<IntakeIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .order_by_desc(intake_issues::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batch fetch associated issues. Django uses
    // `.select_related("issue")` in base.py (inline prefetch); we do a
    // second query with `IN` to avoid N+1 and avoid depending on
    // `find_also_related` (which has ambiguity with Issues1/Issues2 in
    // intake_issues entity — there are two FKs to issues: issue_id and
    // duplicate_to_id).
    //
    // Soft-deleted issues: if an issue_id points to a deleted issue, that
    // intake_issue is omitted from response (Django also excludes them due to
    // default manager filtering deleted_at=null).
    if rows.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let issue_ids: Vec<Uuid> = rows.iter().map(|r| r.issue_id).collect();
    let issues_by_id: std::collections::HashMap<Uuid, issues::Model> = issues::Entity::find()
        .active()
        .filter(issues::Column::Id.is_in(issue_ids.clone()))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|i| (i.id, i))
        .collect();

    let aggregates = fetch_issue_aggregates(&state.db, &issue_ids).await?;

    let responses: Vec<IntakeIssueResponse> = rows
        .into_iter()
        .filter_map(|ii| {
            issues_by_id.get(&ii.issue_id).map(|issue| {
                IntakeIssueResponse::from_joined_with_aggregates(
                    ii,
                    issue,
                    aggregates.get(&issue.id),
                )
            })
        })
        .collect();

    Ok(Json(responses))
}

// ── POST /intake-issues/ ──────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Intake issue created"),
        (status = 400, description = "Validation error"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateIntakeIssueRequest>,
) -> Result<(StatusCode, Json<IntakeIssueResponse>), AppError> {
    // Django uses @allow_permission([ADMIN, MEMBER, GUEST]) — intake accepts
    // tickets from guests (base.py:221). Downgraded to ROLE_GUEST.
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // ── Validations (Django parity base.py:223-234) ─────────────────────────
    if body.issue.name.trim().is_empty() {
        return Err(AppError::BadRequest("Name is required".into()));
    }

    if let Some(ref p) = body.issue.priority {
        if !VALID_PRIORITIES.contains(&p.as_str()) {
            return Err(AppError::BadRequest("Invalid priority".into()));
        }
    }

    // ── Auto-resolve or create project intake ────────────────────────────
    //
    // Django (base.py:264) does `Intake.objects.filter(...).first()` and assumes
    // it exists — if not, it would crash with AttributeError 500 at
    // `intake_id.id`. We are more defensive: get-or-create
    // inline to repair projects where `projects.intake_view=true` but the
    // intake row was never created (historical update_project bug in Rust API,
    // separate fix — see projects.rs).
    //
    // Still, if project has `intake_view=false`, we return 400 not to
    // create intakes in projects that explicitly do not have them enabled.
    //
    // NOTE: we ignore any `intake_id` coming in the body. The DTO no longer
    // accepts it (strict Django parity — it also discards it).
    if !guard.project.intake_view {
        return Err(AppError::BadRequest(
            "Intake is not enabled in this project".into(),
        ));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let intake = if let Some(existing) = intakes::Entity::find()
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .filter(intakes::Column::WorkspaceId.eq(guard.workspace.id))
        .order_by_asc(intakes::Column::CreatedAt)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    {
        existing
    } else {
        // Same values as Django (project/base.py:356-360):
        // name=f"{project.name} Intake", is_default=true.
        // `view_props` and `logo_props` are JSON NOT NULL with default {} in DB.
        intakes::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(format!("{} Intake", guard.project.name)),
            description: Set(String::new()),
            is_default: Set(true),
            view_props: Set(serde_json::json!({})),
            logo_props: Set(serde_json::json!({})),
            project_id: Set(guard.project.id),
            workspace_id: Set(guard.workspace.id),
            created_by_id: Set(Some(guard.user.id)),
            updated_by_id: Set(Some(guard.user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?
    };

    // ── Get-or-create triage state (Django parity base.py:239-249) ───────────
    //
    // TODO(refactor): this logic is identical to routes::states::intake_state.
    // Extract to `pub(crate) fn ensure_triage_state(db, workspace_id, project_id)`
    // in a shared helper.

    let triage_state = if let Some(existing) = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(guard.project.id))
        .filter(states::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(states::Column::IsTriage.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    {
        existing
    } else {
        // Same values as Django (base.py:241-249):
        // name="Triage", color="#4E5355", sequence=65000, group="triage",
        // default=false, is_triage=true. triage state is system-owned
        // (no explicit owner), same as in routes::states::intake_state.
        states::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Triage".to_string()),
            description: Set(String::new()),
            color: Set("#4E5355".to_string()),
            slug: Set("triage".to_string()),
            group: Set("triage".to_string()),
            sequence: Set(65000.0),
            default: Set(false),
            is_triage: Set(true),
            project_id: Set(guard.project.id),
            workspace_id: Set(guard.workspace.id),
            created_by_id: Set(None),
            updated_by_id: Set(None),
            external_id: Set(None),
            external_source: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?
    };

    // ── Calculate sequence_id ──────────────────────────────────────────────────
    //
    // NOTE 1: MAX() without GROUP BY always returns a row (even if table
    // is empty, with NULL value). We decode to Option<i32> and flatten
    // over the Option<Option<i32>> from .one().
    //
    // NOTE 2: target is i32 (NOT i64). In Postgres MAX(INT4) → INT4;
    // not promoted to BIGINT like in MySQL. Using i64 produces
    // "mismatched types; Rust type Option<i64> (as SQL type INT8) is not
    // compatible with SQL type INT4".
    use sea_orm::QuerySelect;
    let max_seq: Option<i32> = issues::Entity::find()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .select_only()
        .column_as(
            sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
            "max_seq",
        )
        .into_tuple::<Option<i32>>()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .flatten();
    let sequence_id = max_seq.unwrap_or(0) + 1;

    // ── Create issue + assignees + labels in a transaction ────────────────
    //
    // Django parity: IssueCreateSerializer.save handles parent_id, start_date,
    // target_date, estimate_point, type_id, assignee_ids, label_ids.
    // is_draft=false: IssueCreateSerializer does not set is_draft.
    let issue_id = Uuid::new_v4();
    let assignee_ids = body.issue.assignee_ids.clone();
    let label_ids = body.issue.label_ids.clone();
    let txn = state.db.begin().await.map_err(AppError::Database)?;
    let issue = issues::ActiveModel {
        id: Set(issue_id),
        name: Set(body.issue.name),
        description_html: Set(body.issue.description_html.unwrap_or_default()),
        description_json: Set(serde_json::json!({})),
        priority: Set(body.issue.priority.unwrap_or_else(|| "none".to_owned())),
        sequence_id: Set(sequence_id),
        sort_order: Set(65535.0),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        state_id: Set(Some(triage_state.id)), // Django base.py:250
        parent_id: Set(body.issue.parent_id),
        start_date: Set(body.issue.start_date),
        target_date: Set(body.issue.target_date),
        estimate_point_id: Set(body.issue.estimate_point),
        type_id: Set(body.issue.type_id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        is_draft: Set(false),
        description_stripped: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    if !assignee_ids.is_empty() {
        sync_assignees(
            &txn,
            issue.id,
            guard.project.id,
            guard.workspace.id,
            guard.user.id,
            &assignee_ids,
        )
        .await?;
    }
    if !label_ids.is_empty() {
        sync_labels(
            &txn,
            issue.id,
            guard.project.id,
            guard.workspace.id,
            guard.user.id,
            &label_ids,
        )
        .await?;
    }

    // ── Create intake_issue (Django parity base.py:266-271) ────────────────
    //
    // `source` hard-coded to IN_APP — Django ignores client value
    // (base.py:270). `body.source` is only accepted not to break old clients;
    // it is discarded here.
    let _ = body.source; // silence unused-field lint; see DTO doc
    let intake_issue = intake_issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        issue_id: Set(issue.id),
        intake_id: Set(intake.id),
        status: Set(STATUS_PENDING),
        source: Set(Some(SOURCE_IN_APP.to_string())),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        extra: Set(serde_json::json!({})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    let mut aggregates = fetch_issue_aggregates(&state.db, &[issue.id]).await?;
    let agg = aggregates.remove(&issue.id);

    Ok((
        StatusCode::CREATED,
        Json(IntakeIssueResponse::from_joined_with_aggregates(
            intake_issue,
            &issue,
            agg.as_ref(),
        )),
    ))
}

// ── GET /intake-issues/{pk}/ ──────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "IntakeIssue ID"),
    ),
    responses(
        (status = 200, description = "Intake issue detail"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<IntakeIssueResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Django interprets `pk` in /inbox-issues/{pk}/ URL as
    // issue_id, NOT as intake_issue id
    // (base.py:499,525 retrieve → `issue_id=pk`). Frontend also sends
    // `response.issue.id` as path param (project-inbox.store.ts:467, 430).
    let ii = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::IssueId.eq(pk))
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Fetch associated issue (Django parity select_related("issue")).
    // If issue is soft-deleted, return 404 — intake_issue without
    // valid issue is inconsistent state.
    let issue = issues::Entity::find_by_id(ii.issue_id)
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut aggregates = fetch_issue_aggregates(&state.db, &[issue.id]).await?;
    let agg = aggregates.remove(&issue.id);

    Ok(Json(IntakeIssueResponse::from_joined_with_aggregates(
        ii,
        &issue,
        agg.as_ref(),
    )))
}

// ── PATCH /intake-issues/{pk}/ ────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "IntakeIssue ID"),
    ),
    responses(
        (status = 200, description = "IntakeIssue updated"),
        (status = 400, description = "Invalid status"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIntakeIssueRequest>,
) -> Result<Json<IntakeIssueResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // pk is issue_id (Django parity base.py:334 partial_update).
    let ii = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::IssueId.eq(pk))
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Validate status if provided
    if let Some(status) = body.status {
        let valid = [STATUS_PENDING, STATUS_REJECTED, STATUS_SNOOZED, STATUS_ACCEPTED, STATUS_DUPLICATE];
        if !valid.contains(&status) {
            return Err(AppError::BadRequest(format!(
                "invalid status: {status}. Allowed values: -2, -1, 0, 1, 2"
            )));
        }
    }

    let issue_id = ii.issue_id;
    let mut am: intake_issues::ActiveModel = ii.into();

    if let Some(status) = body.status {
        am.status = Set(status);
        // Upon acceptance, promote issue from draft to active
        if status == STATUS_ACCEPTED {
            if let Some(issue) = issues::Entity::find_by_id(issue_id)
                .one(&state.db)
                .await
                .map_err(AppError::Database)?
            {
                let mut iam: issues::ActiveModel = issue.into();
                iam.is_draft = Set(false);
                iam.update(&state.db).await.map_err(AppError::Database)?;
            }
        }
    }
    if body.snoozed_till.is_some() {
        am.snoozed_till = Set(body.snoozed_till);
    }
    if body.duplicate_to_id.is_some() {
        am.duplicate_to_id = Set(body.duplicate_to_id);
    }
    if let Some(src) = body.source {
        am.source = Set(Some(src));
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    // Fetch issue after update. If status=ACCEPTED, is_draft was promoted
    // in block above; we need post-update version to reflect
    // that change in response.
    let issue = issues::Entity::find_by_id(updated.issue_id)
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut aggregates = fetch_issue_aggregates(&state.db, &[issue.id]).await?;
    let agg = aggregates.remove(&issue.id);

    Ok(Json(IntakeIssueResponse::from_joined_with_aggregates(
        updated,
        &issue,
        agg.as_ref(),
    )))
}

// ── DELETE /intake-issues/{pk}/ ───────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "IntakeIssue ID"),
    ),
    responses((status = 204, description = "Deleted")),
    security(("TokenAuth" = []))
)]
pub async fn delete_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // pk is issue_id (Django parity base.py:549 destroy).
    let ii = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::IssueId.eq(pk))
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    // Cascade to issue: Django also deletes Issue if intake_issue
    // is in pending/rejected/snoozed/duplicate status (base.py:556-559).
    // STATUS_ACCEPTED (1) keeps issue because it was already promoted to
    // project and exists as regular work item.
    let issue_id = ii.issue_id;
    let status_val = ii.status;
    let cascade_statuses = [STATUS_PENDING, STATUS_REJECTED, STATUS_SNOOZED, STATUS_DUPLICATE];

    let mut am: intake_issues::ActiveModel = ii.into();
    am.deleted_at = Set(Some(now));
    am.update(&state.db).await.map_err(AppError::Database)?;

    if cascade_statuses.contains(&status_val) {
        if let Some(issue) = issues::Entity::find_by_id(issue_id)
            .active()
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
        {
            let mut iam: issues::ActiveModel = issue.into();
            iam.deleted_at = Set(Some(now));
            iam.update(&state.db).await.map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
