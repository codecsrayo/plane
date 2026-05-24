// src/routes/projects.rs
//! Project endpoints — Phase 2b.
//!
//! Equivalent to `plane/app/views/project/base.py` and `member.py` in Django.
//! Authentication: session cookie **or** API key (via `AnyAuth`).
//! Authorization: active workspace membership; Admin role for mutations.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        permissions::{ROLE_ADMIN, ROLE_GUEST, ROLE_MEMBER, ROLE_VIEWER},
    },
    entities::{
        intake_issues, intakes, issue_sequences, project_deploy_boards, project_identifiers,
        project_member_invites, project_members, project_user_properties, projects, states,
        user_favorites, users, workspace_members, workspaces,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ─── Default states when creating a project (mirrors Django DEFAULT_STATES) ────

struct DefaultState {
    name: &'static str,
    color: &'static str,
    sequence: f64,
    group: &'static str,
    is_default: bool,
    is_triage: bool,
}

const DEFAULT_STATES: &[DefaultState] = &[
    DefaultState {
        name: "Backlog",
        color: "#60646C",
        sequence: 15000.0,
        group: "backlog",
        is_default: true,
        is_triage: false,
    },
    DefaultState {
        name: "Todo",
        color: "#60646C",
        sequence: 25000.0,
        group: "unstarted",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "In Progress",
        color: "#F59E0B",
        sequence: 35000.0,
        group: "started",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "Done",
        color: "#46A758",
        sequence: 45000.0,
        group: "completed",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "Cancelled",
        color: "#9AA4BC",
        sequence: 55000.0,
        group: "cancelled",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "Triage",
        color: "#4E5355",
        sequence: 65000.0,
        group: "triage",
        is_default: false,
        is_triage: true,
    },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Gets active project by id within the workspace.
async fn project_by_id(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
) -> Result<projects::Model, AppError> {
    projects::Entity::find_by_id(project_id)
        .active()
        .filter(projects::Column::WorkspaceId.eq(workspace_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

/// Gets active project membership (or Forbidden).
async fn project_member_for_user(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<project_members::Model>, AppError> {
    project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(user_id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)
}

/// Verifies project Admin OR workspace Admin.
fn require_project_admin(
    pm: &Option<project_members::Model>,
    wm: &workspace_members::Model,
) -> Result<(), AppError> {
    let project_role = pm.as_ref().map(|m| m.role).unwrap_or(0);
    if project_role >= ROLE_ADMIN || wm.role >= ROLE_ADMIN {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn validate_role(role: i16) -> Result<(), AppError> {
    if [ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&role) {
        Ok(())
    } else {
        Err(AppError::BadRequest("Invalid role value".into()))
    }
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub identifier: String,
    pub description: String,
    pub network: i16,
    /// FK to workspace. In Django the serializer exposes the FK as `workspace`
    /// (not `workspace_id`); the frontend filters projects by `project.workspace`.
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    pub emoji: Option<String>,
    pub icon_prop: Option<serde_json::Value>,
    pub logo_props: serde_json::Value,
    pub cover_image: Option<String>,
    /// FKs exposed under Django's serializer naming convention (no `_id`
    /// suffix); the frontend reads `default_assignee`, `project_lead`,
    /// `default_state`, `estimate` directly.
    #[serde(rename = "default_assignee")]
    pub default_assignee_id: Option<Uuid>,
    #[serde(rename = "project_lead")]
    pub project_lead_id: Option<Uuid>,
    #[serde(rename = "default_state")]
    pub default_state_id: Option<Uuid>,
    #[serde(rename = "estimate")]
    pub estimate_id: Option<Uuid>,
    pub cycle_view: bool,
    pub module_view: bool,
    pub issue_views_view: bool,
    pub page_view: bool,
    pub intake_view: bool,
    pub is_time_tracking_enabled: bool,
    pub is_issue_type_enabled: bool,
    pub guest_view_all_features: bool,
    pub timezone: String,
    pub archive_in: i32,
    pub close_in: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Number of active members in the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_members: Option<i64>,
    /// User role in this project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_role: Option<i16>,
}

impl ProjectResponse {
    pub(crate) fn from_model(
        p: &projects::Model,
        total_members: Option<i64>,
        member_role: Option<i16>,
    ) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            identifier: p.identifier.clone(),
            description: p.description.clone(),
            network: p.network,
            workspace_id: p.workspace_id,
            emoji: p.emoji.clone(),
            icon_prop: p.icon_prop.clone(),
            logo_props: p.logo_props.clone(),
            cover_image: p.cover_image.clone(),
            default_assignee_id: p.default_assignee_id,
            project_lead_id: p.project_lead_id,
            default_state_id: p.default_state_id,
            estimate_id: p.estimate_id,
            cycle_view: p.cycle_view,
            module_view: p.module_view,
            issue_views_view: p.issue_views_view,
            page_view: p.page_view,
            intake_view: p.intake_view,
            is_time_tracking_enabled: p.is_time_tracking_enabled,
            is_issue_type_enabled: p.is_issue_type_enabled,
            guest_view_all_features: p.guest_view_all_features,
            timezone: p.timezone.clone(),
            archive_in: p.archive_in,
            close_in: p.close_in,
            archived_at: p.archived_at.map(Into::into),
            created_at: p.created_at.into(),
            updated_at: p.updated_at.into(),
            total_members,
            member_role,
        }
    }
}

/// Flat response for `GET /workspaces/{slug}/projects/` — exactly mirrors
/// the 22 fields Django exposes via `.values(...)` in
/// `ProjectViewSet.list` (`plane/app/views/project/base.py`).
///
/// Important keys:
/// - `workspace` (not `workspace_id`): frontend filters with `project.workspace`.
/// - `project_lead` (not `project_lead_id`): DRF convention for FK.
/// - `inbox_view`: alias for `intake_view`.
/// - `sort_order`: comes from the user's `project_user_properties`.
/// - `intake_count`: count of `intake_issues` with status=-2 (PENDING).
///
/// DOES NOT include `description`, `emoji`, `cover_image`, `timezone`, etc. — Django
/// doesn't send them either in this endpoint; for those use `/projects/{id}/` and
/// `/projects/details/`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectListResponse {
    pub id: Uuid,
    pub name: String,
    pub identifier: String,
    pub sort_order: Option<f64>,
    pub logo_props: serde_json::Value,
    pub member_role: Option<i16>,
    pub intake_count: i64,
    pub archived_at: Option<DateTime<Utc>>,
    pub workspace: Uuid,
    pub cycle_view: bool,
    pub issue_views_view: bool,
    pub module_view: bool,
    pub page_view: bool,
    pub inbox_view: bool,
    pub guest_view_all_features: bool,
    pub project_lead: Option<Uuid>,
    pub network: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub network: Option<i16>,
    pub emoji: Option<String>,
    pub project_lead_id: Option<Uuid>,
    pub default_assignee_id: Option<Uuid>,
    pub timezone: Option<String>,
    // Django parity: ProjectSerializer uses `fields = "__all__"` with
    // `read_only_fields = ["workspace", "deleted_at"]`, so it accepts
    // logo_props / cover_image / cover_image_asset in the POST body.
    // Reference: apps/api/plane/app/serializers/project.py:30-37.
    pub logo_props: Option<serde_json::Value>,
    pub cover_image: Option<String>,
    pub cover_image_asset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub network: Option<i16>,
    pub emoji: Option<String>,
    pub project_lead_id: Option<Uuid>,
    pub default_assignee_id: Option<Uuid>,
    pub timezone: Option<String>,
    pub cycle_view: Option<bool>,
    pub module_view: Option<bool>,
    pub issue_views_view: Option<bool>,
    pub page_view: Option<bool>,
    pub intake_view: Option<bool>,
    pub is_time_tracking_enabled: Option<bool>,
    pub cover_image: Option<String>,
    // Django parity: ProjectSerializer (partial_update with partial=True)
    // accepts logo_props and cover_image_asset in PATCH — fields = "__all__"
    // covers both and neither is in read_only_fields.
    // Reference: apps/api/plane/app/views/project/base.py:344-349.
    pub logo_props: Option<serde_json::Value>,
    pub cover_image_asset_id: Option<Uuid>,
    pub archive_in: Option<i32>,
    pub close_in: Option<i32>,
    pub guest_view_all_features: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectMemberResponse {
    pub id: Uuid,
    pub member_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub role: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&project_members::Model> for ProjectMemberResponse {
    fn from(m: &project_members::Model) -> Self {
        Self {
            id: m.id,
            member_id: m.member_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            role: m.role,
            is_active: m.is_active,
            created_at: m.created_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectMemberRequest {
    pub role: i16,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectInvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub role: i16,
    pub accepted: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<&project_member_invites::Model> for ProjectInvitationResponse {
    fn from(i: &project_member_invites::Model) -> Self {
        Self {
            id: i.id,
            email: i.email.clone(),
            role: i.role,
            accepted: i.accepted,
            project_id: i.project_id,
            workspace_id: i.workspace_id,
            created_at: i.created_at.into(),
        }
    }
}

/// Expected body: `{"emails": [{"email": "...", "role": 15}, ...]}`.
///
/// Exact mirror of Django contract (`ProjectInvitationsViewset.create`),
/// which reads `request.data.get("emails", [])`. Maintaining a single format avoids
/// polymorphic extractors that hide bugs and simplifies the client.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProjectInvitationRequest {
    pub emails: Vec<ProjectInviteEmail>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProjectInviteEmail {
    pub email: String,
    pub role: i16,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/`
///
/// Lists projects visible to the user in the workspace. Mirrors Django's
/// `ProjectViewSet.list`:
/// - ADMIN: all workspace projects.
/// - MEMBER: projects where user is active member **or** `network == 2` (public).
/// - GUEST: only projects where user is active member.
///
/// Returns [`ProjectListResponse`] with the same shape as DRF's `.values(...)`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Project list", body = Vec<ProjectListResponse>),
        (status = 403, description = "Not a workspace member"),
    )
)]
pub async fn list_projects(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<ProjectListResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;

    // ── 1. Role filtering (mirrors Django's `def list`) ────────────────────
    let projects_list = if wm.role >= ROLE_ADMIN {
        projects::Entity::find()
            .active()
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        let member_project_ids: Vec<Uuid> = project_members::Entity::find()
            .active()
            .filter(project_members::Column::WorkspaceId.eq(ws.id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|pm| pm.project_id)
            .collect();

        if wm.role == ROLE_GUEST {
            // GUEST: strictly their projects
            if member_project_ids.is_empty() {
                return Ok(Json(vec![]));
            }
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(projects::Column::Id.is_in(member_project_ids))
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        } else {
            // MEMBER (or VIEWER): their projects + public projects (network=2)
            use sea_orm::Condition;
            let condition = if member_project_ids.is_empty() {
                Condition::all().add(projects::Column::Network.eq(2i16))
            } else {
                Condition::any()
                    .add(projects::Column::Id.is_in(member_project_ids))
                    .add(projects::Column::Network.eq(2i16))
            };
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(condition)
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        }
    };

    if projects_list.is_empty() {
        return Ok(Json(vec![]));
    }

    let project_ids: Vec<Uuid> = projects_list.iter().map(|p| p.id).collect();

    // ── 2. User's `member_role` per project (only active memberships) ──────
    let pm_map: std::collections::HashMap<Uuid, i16> = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::ProjectId.is_in(project_ids.clone()))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pm| (pm.project_id, pm.role))
        .collect();

    // ── 3. `sort_order` per project for the current user ───────────────────
    let sort_orders: std::collections::HashMap<Uuid, f64> =
        project_user_properties::Entity::find()
            .filter(project_user_properties::Column::UserId.eq(user.id))
            .filter(project_user_properties::Column::WorkspaceId.eq(ws.id))
            .filter(project_user_properties::Column::ProjectId.is_in(project_ids.clone()))
            .filter(project_user_properties::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.project_id, p.sort_order))
            .collect();

    // ── 4. `intake_count` per project (status=-2 PENDING, not soft-deleted) ─
    //
    // Single grouped query instead of N+1 (more efficient than the loop in
    // `list_projects_detail`). Mirrors Django's `Count(filter=Q(status=-2, ...))`.
    let mut intake_counts: std::collections::HashMap<Uuid, i64> =
        std::collections::HashMap::new();
    let intake_rows: Vec<(Uuid, i64)> = intake_issues::Entity::find()
        .select_only()
        .column(intake_issues::Column::ProjectId)
        .column_as(
            sea_orm::sea_query::Expr::col(intake_issues::Column::Id).count(),
            "count",
        )
        .filter(intake_issues::Column::ProjectId.is_in(project_ids.clone()))
        .filter(intake_issues::Column::Status.eq(-2i32))
        .filter(intake_issues::Column::DeletedAt.is_null())
        .group_by(intake_issues::Column::ProjectId)
        .into_tuple()
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;
    for (pid, c) in intake_rows {
        intake_counts.insert(pid, c);
    }

    // ── 5. Assemble + order by (sort_order NULLS LAST, name) like Django ────
    let mut responses: Vec<ProjectListResponse> = projects_list
        .iter()
        .map(|p| ProjectListResponse {
            id: p.id,
            name: p.name.clone(),
            identifier: p.identifier.clone(),
            sort_order: sort_orders.get(&p.id).copied(),
            logo_props: p.logo_props.clone(),
            member_role: pm_map.get(&p.id).copied(),
            intake_count: *intake_counts.get(&p.id).unwrap_or(&0),
            archived_at: p.archived_at.map(Into::into),
            workspace: p.workspace_id,
            cycle_view: p.cycle_view,
            issue_views_view: p.issue_views_view,
            module_view: p.module_view,
            page_view: p.page_view,
            inbox_view: p.intake_view,
            guest_view_all_features: p.guest_view_all_features,
            project_lead: p.project_lead_id,
            network: p.network,
            created_at: p.created_at.into(),
            updated_at: p.updated_at.into(),
            created_by: p.created_by_id,
            updated_by: p.updated_by_id,
        })
        .collect();

    responses.sort_by(|a, b| {
        // NULLS LAST on sort_order, then by name (case-sensitive as PG default).
        match (a.sort_order, b.sort_order) {
            (Some(x), Some(y)) => x
                .partial_cmp(&y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });

    Ok(Json(responses))
}

// ─── Extended DTO for /details ────────────────────────────────────────────────

/// Extended project response — mirrors Django's `ProjectListSerializer`.
/// Includes calculated fields: `is_favorite`, `sort_order`, `members`, `anchor`,
/// `inbox_view`, `intake_count`, `next_work_item_sequence`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectDetailResponse {
    #[serde(flatten)]
    pub base: ProjectResponse,
    pub is_favorite: bool,
    pub sort_order: Option<f64>,
    /// UUIDs of active project members.
    pub members: Vec<Uuid>,
    /// Anchor of the public deploy-board, if it exists.
    pub anchor: Option<String>,
    /// Alias of intake_view for frontend compatibility.
    pub inbox_view: bool,
    /// Number of pending intake-issues (status = -2).
    pub intake_count: i64,
    /// Next available sequence_id for a new issue.
    pub next_work_item_sequence: i64,
}

/// `GET /api/workspaces/{slug}/projects/details/`
///
/// Complete list of projects with all calculated fields that the
/// frontend needs to render the sidebar and projects home.
/// Mirrors Django's `ProjectViewSet.list_detail`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/details/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Project detail list", body = Vec<ProjectDetailResponse>),
        (status = 403, description = "Not a workspace member"),
    )
)]
pub async fn list_projects_detail(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<ProjectDetailResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;

    // ── 1. Visible projects by role ─────────────────────────────────────────
    let projects_list = if wm.role >= ROLE_ADMIN {
        projects::Entity::find()
            .active()
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .order_by_asc(projects::Column::Name)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        let member_project_ids: Vec<Uuid> = project_members::Entity::find()
            .active()
            .filter(project_members::Column::WorkspaceId.eq(ws.id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|pm| pm.project_id)
            .collect();

        if wm.role == ROLE_GUEST {
            if member_project_ids.is_empty() {
                return Ok(Json(vec![]));
            }
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(projects::Column::Id.is_in(member_project_ids))
                .order_by_asc(projects::Column::Name)
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        } else {
            // MEMBER: own + public projects (network=2)
            use sea_orm::Condition;
            let condition = if member_project_ids.is_empty() {
                Condition::all().add(projects::Column::Network.eq(2i16))
            } else {
                Condition::any()
                    .add(projects::Column::Id.is_in(member_project_ids))
                    .add(projects::Column::Network.eq(2i16))
            };
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(condition)
                .order_by_asc(projects::Column::Name)
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        }
    };

    if projects_list.is_empty() {
        return Ok(Json(vec![]));
    }

    let project_ids: Vec<Uuid> = projects_list.iter().map(|p| p.id).collect();

    // ── 2. Memberships: user role and full member list ──────────────────────
    let all_members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::ProjectId.is_in(project_ids.clone()))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let user_role_map: std::collections::HashMap<Uuid, i16> = all_members
        .iter()
        .filter(|m| m.member_id == Some(user.id))
        .map(|m| (m.project_id, m.role))
        .collect();

    let mut members_map: std::collections::HashMap<Uuid, Vec<Uuid>> =
        std::collections::HashMap::new();
    for m in &all_members {
        if let Some(mid) = m.member_id {
            members_map.entry(m.project_id).or_default().push(mid);
        }
    }

    // ── 3. User favorites ───────────────────────────────────────────────────
    let favorites: std::collections::HashSet<Uuid> = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.is_in(project_ids.clone()))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    // ── 4. User sort_order per project ──────────────────────────────────────
    let sort_orders: std::collections::HashMap<Uuid, f64> =
        project_user_properties::Entity::find()
            .filter(project_user_properties::Column::UserId.eq(user.id))
            .filter(project_user_properties::Column::WorkspaceId.eq(ws.id))
            .filter(project_user_properties::Column::ProjectId.is_in(project_ids.clone()))
            .filter(project_user_properties::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.project_id, p.sort_order))
            .collect();

    // ── 5. Deploy-board anchor ──────────────────────────────────────────────
    let anchors: std::collections::HashMap<Uuid, String> =
        project_deploy_boards::Entity::find()
            .filter(project_deploy_boards::Column::WorkspaceId.eq(ws.id))
            .filter(project_deploy_boards::Column::ProjectId.is_in(project_ids.clone()))
            .filter(project_deploy_boards::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|d| (d.project_id, d.anchor))
            .collect();

    // ── 6. intake_count (pending = -2) per project ───────────────────────────
    let mut intake_counts: std::collections::HashMap<Uuid, i64> =
        std::collections::HashMap::new();
    for &pid in &project_ids {
        let count = intake_issues::Entity::find()
            .filter(intake_issues::Column::ProjectId.eq(pid))
            .filter(intake_issues::Column::Status.eq(-2i32))
            .filter(intake_issues::Column::DeletedAt.is_null())
            .count(&state.db)
            .await
            .map_err(AppError::Database)? as i64;
        intake_counts.insert(pid, count);
    }

    // ── 7. next_work_item_sequence per project ──────────────────────────────
    let mut next_sequences: std::collections::HashMap<Uuid, i64> =
        std::collections::HashMap::new();
    for &pid in &project_ids {
        let max_seq = issue_sequences::Entity::find()
            .select_only()
            .column(issue_sequences::Column::Sequence)
            .filter(issue_sequences::Column::ProjectId.eq(pid))
            .filter(issue_sequences::Column::DeletedAt.is_null())
            .order_by_desc(issue_sequences::Column::Sequence)
            .limit(1)
            .into_tuple::<i64>()
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;
        next_sequences.insert(pid, max_seq.map(|s| s + 1).unwrap_or(1));
    }

    // ── 8. Assemble responses ───────────────────────────────────────────────
    let responses = projects_list
        .iter()
        .map(|p| {
            let role = user_role_map.get(&p.id).copied();
            let base = ProjectResponse::from_model(p, None, role);
            ProjectDetailResponse {
                is_favorite: favorites.contains(&p.id),
                sort_order: sort_orders.get(&p.id).copied(),
                members: members_map.get(&p.id).cloned().unwrap_or_default(),
                anchor: anchors.get(&p.id).cloned(),
                inbox_view: p.intake_view,
                intake_count: *intake_counts.get(&p.id).unwrap_or(&0),
                next_work_item_sequence: *next_sequences.get(&p.id).unwrap_or(&1),
                base,
            }
        })
        .collect();

    Ok(Json(responses))
}

/// `POST /api/workspaces/{slug}/projects/`
///
/// Creates a project, initializes it with default states and adds the
/// user as project Admin.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = ProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    // ── Shape validations (forbidden chars, length) ────────────────────────
    // Mirrors plane.db.models.project.Project.FORBIDDEN_IDENTIFIER_CHARS_PATTERN
    // and Django's ProjectSerializer.validate_identifier.
    if body.name.is_empty() || body.name.len() > 255 {
        return Err(AppError::Validation(serde_json::json!({
            "name": ["PROJECT_NAME_INVALID_LENGTH"]
        })));
    }
    if body.identifier.is_empty() || body.identifier.len() > 12 {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_INVALID_LENGTH"]
        })));
    }
    let identifier = body.identifier.trim().to_uppercase();
    // Django rejects: & + , : ; $ ^ } { * = ? @ # | ' < > . ( ) % ! -
    const FORBIDDEN_CHARS: &[char] = &[
        '&', '+', ',', ':', ';', '$', '^', '}', '{', '*', '=', '?', '@', '#', '|', '\'', '<',
        '>', '.', '(', ')', '%', '!', '-',
    ];
    if identifier.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_CANNOT_CONTAIN_SPECIAL_CHARACTERS"]
        })));
    }

    let ws = workspace_by_slug(&state.db, &slug).await?;
    // Any active member can create projects
    require_workspace_member(&state.db, ws.id, user.id).await?;

    // ── Identifier uniqueness (only non soft-deleted projects) ─────────────
    let dup_identifier = projects::Entity::find()
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .filter(projects::Column::Identifier.eq(&identifier))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;
    if dup_identifier > 0 {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_ALREADY_EXIST"]
        })));
    }

    // ── Name uniqueness (mirrors Django's ProjectSerializer.validate_name) ──
    let dup_name = projects::Entity::find()
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .filter(projects::Column::Name.eq(&body.name))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;
    if dup_name > 0 {
        return Err(AppError::Validation(serde_json::json!({
            "name": ["PROJECT_NAME_ALREADY_EXIST"]
        })));
    }

    let project_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();
    let network = body.network.unwrap_or(0);

    if ![0i16, 2].contains(&network) {
        return Err(AppError::BadRequest(
            "network must be 0 (secret) or 2 (public)".into(),
        ));
    }

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    // Create project
    let new_project = projects::ActiveModel {
        id: Set(project_id),
        name: Set(body.name.clone()),
        identifier: Set(identifier),
        description: Set(body.description.unwrap_or_default()),
        description_text: Set(None),
        description_html: Set(None),
        network: Set(network),
        workspace_id: Set(ws.id),
        emoji: Set(body.emoji),
        icon_prop: Set(None),
        // Django Parity: optional fields in POST, default server-side
        // when the client doesn't send them (JSONField default=dict / null).
        logo_props: Set(body.logo_props.clone().unwrap_or_else(|| serde_json::json!({}))),
        cover_image: Set(body.cover_image.clone()),
        cover_image_asset_id: Set(body.cover_image_asset_id),
        default_assignee_id: Set(body.default_assignee_id),
        project_lead_id: Set(body.project_lead_id),
        default_state_id: Set(None),
        estimate_id: Set(None),
        cycle_view: Set(true),
        module_view: Set(true),
        issue_views_view: Set(true),
        page_view: Set(true),
        intake_view: Set(true),
        is_time_tracking_enabled: Set(false),
        is_issue_type_enabled: Set(false),
        guest_view_all_features: Set(false),
        timezone: Set(body.timezone.unwrap_or_else(|| ws.timezone.clone())),
        archive_in: Set(0),
        close_in: Set(0),
        archived_at: Set(None),
        deleted_at: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let project = new_project.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert project");
        AppError::Database(e)
    })?;

    // Create Admin membership for the creator
    let creator_member = project_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        project_id: Set(project_id),
        workspace_id: Set(ws.id),
        member_id: Set(Some(user.id)),
        role: Set(ROLE_ADMIN),
        is_active: Set(true),
        comment: Set(None),
        view_props: Set(serde_json::json!({})),
        default_props: Set(serde_json::json!({})),
        preferences: Set(serde_json::json!({})),
        sort_order: Set(65535.0),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };
    creator_member.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert project member");
        AppError::Database(e)
    })?;

    // If project_lead is different from the creator, add them as Admin as well
    if let Some(lead_id) = body.project_lead_id {
        if lead_id != user.id {
            let lead_member = project_members::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                member_id: Set(Some(lead_id)),
                role: Set(ROLE_ADMIN),
                is_active: Set(true),
                comment: Set(None),
                view_props: Set(serde_json::json!({})),
                default_props: Set(serde_json::json!({})),
                preferences: Set(serde_json::json!({})),
                sort_order: Set(65535.0),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            };
            lead_member.insert(&txn).await.map_err(|e| {
                tracing::error!(error = %e, "Failed to insert project lead member");
                AppError::Database(e)
            })?;
        }
    }

    // Create default states (mirrors Django's DEFAULT_STATES)
    let mut default_state_id: Option<Uuid> = None;
    for ds in DEFAULT_STATES {
        let state_id = Uuid::new_v4();
        let slug_state = ds.name.to_lowercase().replace(' ', "-");
        let new_state = states::ActiveModel {
            id: Set(state_id),
            name: Set(ds.name.to_string()),
            description: Set(String::new()),
            color: Set(ds.color.to_string()),
            slug: Set(slug_state),
            sequence: Set(ds.sequence),
            group: Set(ds.group.to_string()),
            default: Set(ds.is_default),
            is_triage: Set(ds.is_triage),
            project_id: Set(project_id),
            workspace_id: Set(ws.id),
            external_id: Set(None),
            external_source: Set(None),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        new_state.insert(&txn).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to insert default state");
            AppError::Database(e)
        })?;
        if ds.is_default {
            default_state_id = Some(state_id);
        }
    }

    // Set default_state_id on the project
    if let Some(ds_id) = default_state_id {
        let mut active: projects::ActiveModel = project.clone().into();
        active.default_state_id = Set(Some(ds_id));
        active.update(&txn).await.map_err(AppError::Database)?;
    }

    txn.commit().await.map_err(AppError::Database)?;

    let resp = ProjectResponse::from_model(&project, Some(1), Some(ROLE_ADMIN));
    Ok((StatusCode::CREATED, Json(resp)))
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/`
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Project detail", body = ProjectResponse),
        (status = 403, description = "Not a member"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    // Verify access: workspace admin or project member.
    // Non-members receive 404 (not 403) to avoid leaking project existence
    // to unauthorized users — standard security practice.
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    if pm.is_none() && wm.role < ROLE_ADMIN {
        return Err(AppError::NotFound);
    }

    let total = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    let role = pm.as_ref().map(|m| m.role);
    Ok(Json(ProjectResponse::from_model(
        &project,
        Some(total as i64),
        role,
    )))
}

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Updated project", body = ProjectResponse),
        (status = 403, description = "Requires project Admin or workspace Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    if let Some(ref name) = body.name {
        if name.is_empty() || name.len() > 255 {
            return Err(AppError::BadRequest(
                "Project name must be between 1 and 255 characters".into(),
            ));
        }
    }
    if let Some(net) = body.network {
        if ![0i16, 2].contains(&net) {
            return Err(AppError::BadRequest(
                "network must be 0 (secret) or 2 (public)".into(),
            ));
        }
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: projects::ActiveModel = project.into();

    // Capture if client is enabling intake_view to apply idempotent
    // creation of the Intake row after update — Django parity
    // (apps/api/plane/app/views/project/base.py:353-360).
    let enabled_intake = body.intake_view == Some(true);

    if let Some(v) = body.name { active.name = Set(v); }
    if let Some(v) = body.description { active.description = Set(v); }
    if let Some(v) = body.network { active.network = Set(v); }
    if let Some(v) = body.emoji { active.emoji = Set(Some(v)); }
    if let Some(v) = body.project_lead_id { active.project_lead_id = Set(Some(v)); }
    if let Some(v) = body.default_assignee_id { active.default_assignee_id = Set(Some(v)); }
    if let Some(v) = body.timezone { active.timezone = Set(v); }
    if let Some(v) = body.cycle_view { active.cycle_view = Set(v); }
    if let Some(v) = body.module_view { active.module_view = Set(v); }
    if let Some(v) = body.issue_views_view { active.issue_views_view = Set(v); }
    if let Some(v) = body.page_view { active.page_view = Set(v); }
    if let Some(v) = body.intake_view { active.intake_view = Set(v); }
    if let Some(v) = body.is_time_tracking_enabled { active.is_time_tracking_enabled = Set(v); }
    if let Some(v) = body.cover_image { active.cover_image = Set(Some(v)); }
    if let Some(v) = body.cover_image_asset_id { active.cover_image_asset_id = Set(Some(v)); }
    if let Some(v) = body.logo_props { active.logo_props = Set(v); }
    if let Some(v) = body.archive_in { active.archive_in = Set(v); }
    if let Some(v) = body.close_in { active.close_in = Set(v); }
    if let Some(v) = body.guest_view_all_features { active.guest_view_all_features = Set(v); }

    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;

    // Django parity: if intake_view just got enabled, perform get-or-create
    // of the Intake row for the project. Without this, `projects.intake_view=true`
    // but the `intakes` table empty → frontend shows intake UI but
    // POST /intake-issues/ handler fails (see routes/intake.rs).
    //
    // NOTE: Django doesn't use transaction.atomic() here either, so if this
    // INSERT fails, client will see intake_view=true but the intake won't
    // exist. The create_intake_issue handler has defensive get-or-create
    // as a safety net.
    if enabled_intake {
        let existing = intakes::Entity::find()
            .active()
            .filter(intakes::Column::ProjectId.eq(updated.id))
            .filter(intakes::Column::IsDefault.eq(true))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if existing.is_none() {
            intakes::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(format!("{} Intake", updated.name)),
                description: Set(String::new()),
                is_default: Set(true),
                view_props: Set(serde_json::json!({})),
                logo_props: Set(serde_json::json!({})),
                project_id: Set(updated.id),
                workspace_id: Set(updated.workspace_id),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?;
        }
    }

    let role = pm.as_ref().map(|m| m.role);
    Ok(Json(ProjectResponse::from_model(&updated, None, role)))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/`
///
/// Soft-delete. Requires project Admin or workspace Admin.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 403, description = "Requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: projects::ActiveModel = project.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Project Members ──────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/members/`
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Member list", body = Vec<ProjectMemberResponse>),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_project_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<ProjectMemberResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    if pm.is_none() && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .order_by_asc(project_members::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(members.iter().map(ProjectMemberResponse::from).collect()))
}

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/members/{pk}/`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("pk"         = Uuid,   Path, description = "Member record UUID"),
    ),
    request_body = UpdateProjectMemberRequest,
    responses(
        (status = 200, description = "Updated member", body = ProjectMemberResponse),
        (status = 403, description = "Requires Admin"),
    )
)]
pub async fn update_project_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateProjectMemberRequest>,
) -> Result<Json<ProjectMemberResponse>, AppError> {
    validate_role(body.role)?;

    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let target = project_members::Entity::find_by_id(pk)
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: project_members::ActiveModel = target.into();
    active.role = Set(body.role);
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ProjectMemberResponse::from(&updated)))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/members/{pk}/`
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("pk"         = Uuid,   Path, description = "Member record UUID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 403, description = "Requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn remove_project_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let target = project_members::Entity::find_by_id(pk)
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: project_members::ActiveModel = target.into();
    active.is_active = Set(false);
    active.deleted_at = Set(Some(now));
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Project Invitations ──────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/invitations/`
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Invitation list", body = Vec<ProjectInvitationResponse>),
        (status = 403, description = "Requires Admin"),
    )
)]
pub async fn list_project_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<ProjectInvitationResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let invites = project_member_invites::Entity::find()
        .active()
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .filter(project_member_invites::Column::Accepted.eq(false))
        .order_by_desc(project_member_invites::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(invites.iter().map(ProjectInvitationResponse::from).collect()))
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/invitations/`
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    request_body = CreateProjectInvitationRequest,
    responses(
        (status = 201, description = "Invitations created", body = Vec<ProjectInvitationResponse>),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Requires Admin"),
    )
)]
pub async fn create_project_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateProjectInvitationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.emails.is_empty() {
        return Err(AppError::BadRequest("emails list is required".into()));
    }

    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut created = Vec::with_capacity(body.emails.len());

    for invite_req in &body.emails {
        validate_role(invite_req.role)?;

        let existing = project_member_invites::Entity::find()
            .active()
            .filter(project_member_invites::Column::ProjectId.eq(project_id))
            .filter(project_member_invites::Column::Email.eq(&invite_req.email))
            .filter(project_member_invites::Column::Accepted.eq(false))
            .count(&state.db)
            .await
            .map_err(AppError::Database)?;

        if existing > 0 {
            tracing::warn!(
                email = %invite_req.email,
                project = %project_id,
                "Skipping duplicate pending project invitation"
            );
            continue;
        }

        let new_invite = project_member_invites::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            workspace_id: Set(ws.id),
            email: Set(invite_req.email.clone()),
            role: Set(invite_req.role),
            accepted: Set(false),
            token: Set(Uuid::new_v4().to_string()),
            message: Set(None),
            responded_at: Set(None),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        let saved = new_invite
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?;
        created.push(saved);
    }

    let responses: Vec<ProjectInvitationResponse> =
        created.iter().map(ProjectInvitationResponse::from).collect();
    Ok((StatusCode::CREATED, Json(responses)))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/invitations/{pk}/`
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("pk"         = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 204, description = "Invitation deleted"),
        (status = 403, description = "Requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_project_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let invite = project_member_invites::Entity::find_by_id(pk)
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .filter(project_member_invites::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: project_member_invites::ActiveModel = invite.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── GET /workspaces/{slug}/projects/{project_id}/project-members/me ──────────
//
// Mirror of `plane/app/views/project/member.py::ProjectMemberUserEndpoint`:
// returns the authenticated user's ProjectMember serialized with
// `ProjectMemberSerializer` (workspace/project/member nested as "lite").
//
// Note: Django uses `ProjectMember.objects.get(...)` on the manager which already
// filters `deleted_at__isnull=True`; the absence of result raises 404 via
// DoesNotExist. Here we translate it to `AppError::NotFound` with the same
// filters (is_active + deleted_at IS NULL via `.active()`).

/// Subset of `WorkspaceLiteSerializer`
/// (`apps/api/plane/app/serializers/workspace.py:78-82`):
/// `["name", "slug", "id", "logo_url"]`. DOES NOT include raw `logo`, same as Django.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceLiteDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    /// Mirror of `Workspace.logo_url` (`apps/api/plane/db/models/workspace.py:146-154`):
    /// `logo_asset.asset_url` → `logo` crudo → `None`.
    pub logo_url: Option<String>,
}

impl From<&workspaces::Model> for WorkspaceLiteDto {
    fn from(w: &workspaces::Model) -> Self {
        // logo_asset.asset_url for WORKSPACE_LOGO is `/api/assets/v2/static/{id}/`
        // (see `apps/api/plane/db/models/asset.py:79-87`).
        let logo_url = if let Some(asset_id) = w.logo_asset_id {
            Some(format!("/api/assets/v2/static/{}/", asset_id))
        } else {
            w.logo.clone()
        };
        Self {
            id: w.id,
            name: w.name.clone(),
            slug: w.slug.clone(),
            logo_url,
        }
    }
}

/// Subset of `ProjectLiteSerializer`
/// (`apps/api/plane/app/serializers/project.py:97-108`):
/// `["id","identifier","name","cover_image","cover_image_url","logo_props","description"]`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectLiteDto {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub cover_image: Option<String>,
    /// Mirror of `Project.cover_image_url`
    /// (`apps/api/plane/db/models/project.py:127-137`).
    pub cover_image_url: Option<String>,
    pub logo_props: serde_json::Value,
    pub description: String,
}

impl From<&projects::Model> for ProjectLiteDto {
    fn from(p: &projects::Model) -> Self {
        // PROJECT_COVER.asset_url → `/api/assets/v2/static/{id}/`
        let cover_image_url = if let Some(asset_id) = p.cover_image_asset_id {
            Some(format!("/api/assets/v2/static/{}/", asset_id))
        } else {
            p.cover_image.clone()
        };
        Self {
            id: p.id,
            identifier: p.identifier.clone(),
            name: p.name.clone(),
            cover_image: p.cover_image.clone(),
            cover_image_url,
            logo_props: p.logo_props.clone(),
            description: p.description.clone(),
        }
    }
}

/// Mirror of `ProjectMemberSerializer` (fields="__all__") with
/// nested `workspace`, `project`, `member` (Lite). Fields consumed by the
/// frontend store are explicitly listed to avoid over-filtering; the
/// returned fields are those persisted by the `ProjectMember` model
/// (`apps/api/plane/db/models/project.py::ProjectMember`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectMemberMeResponse {
    pub id: Uuid,
    pub role: i16,
    pub is_active: bool,
    pub comment: Option<String>,
    pub view_props: serde_json::Value,
    pub default_props: serde_json::Value,
    pub preferences: serde_json::Value,
    pub sort_order: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub member: Option<crate::routes::workspaces::UserLiteDto>,
    pub project: ProjectLiteDto,
    pub workspace: WorkspaceLiteDto,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub member_id: Option<Uuid>,
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/project-members/me/`
///
/// Returns the authenticated user's ProjectMember. 404 if no active
/// membership exists (Django parity with `ProjectMember.objects.get(...)`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-members/me/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Current user's project member", body = ProjectMemberMeResponse),
        (status = 404, description = "User is not a member of this project"),
    )
)]
pub async fn get_project_member_me(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectMemberMeResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    // Django DOES NOT require workspace-member for this endpoint, but the query
    // by (workspace_slug, project_id, member=request.user, is_active=true)
    // returns 404 if user doesn't belong. We replicate the same semantics:
    // search directly for the row and return 404 if it doesn't exist.

    let member = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Load active project and user for nested objects.
    // `project_by_id` filters `deleted_at IS NULL` — if the project is deleted
    // we return 404, same as Django (`ProjectMember.project` with
    // `active_objects` manager filters deleted_at).
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    let user_row = users::Entity::find_by_id(user.id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Admin-visibility for email/last_login_medium: same criteria as other
    // serializadores (`is_admin = role >= ROLE_ADMIN` en el proyecto).
    let is_admin = member.role >= ROLE_ADMIN;
    let member_dto = user_row
        .as_ref()
        .map(|u| crate::routes::workspaces::user_to_lite(u, is_admin));

    Ok(Json(ProjectMemberMeResponse {
        id: member.id,
        role: member.role,
        is_active: member.is_active,
        comment: member.comment.clone(),
        view_props: member.view_props.clone(),
        default_props: member.default_props.clone(),
        preferences: member.preferences.clone(),
        sort_order: member.sort_order,
        created_at: member.created_at.into(),
        updated_at: member.updated_at.into(),
        created_by: member.created_by_id,
        updated_by: member.updated_by_id,
        member: member_dto,
        project: ProjectLiteDto::from(&project),
        workspace: WorkspaceLiteDto::from(&ws),
        project_id: member.project_id,
        workspace_id: member.workspace_id,
        member_id: member.member_id,
    }))
}


// ─── GET /workspaces/{slug}/projects/{project_id}/members/{pk}/ ──────────────

/// Returns a specific project member.
///
/// Mirror of `ProjectMemberViewSet.retrieve`
/// (`apps/api/plane/app/views/project/member.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn get_project_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let caller_pm = project_member_for_user(&state.db, project_id, user.id).await?;
    let is_admin = caller_pm.as_ref().map(|m| m.role).unwrap_or(0) > ROLE_GUEST
        || wm.role > ROLE_GUEST;

    let member = project_members::Entity::find_by_id(pk)
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let user_model = users::Entity::find_by_id(member.member_id.ok_or(AppError::NotFound)?)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({
        "id": member.id,
        "member_id": member.member_id,
        "role": member.role,
        "is_active": member.is_active,
        "member": {
            "id": user_model.id,
            "display_name": user_model.display_name,
            "first_name": user_model.first_name,
            "last_name": user_model.last_name,
            "avatar": user_model.avatar,
            "email": if is_admin { user_model.email.clone() } else { None },
        },
    })))
}

// ─── POST /workspaces/{slug}/projects/{project_id}/members/ ──────────────────

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AddProjectMembersRequest {
    pub members: Vec<ProjectMemberEntry>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ProjectMemberEntry {
    pub member_id: Uuid,
    pub role: i16,
}

/// Adds members to the project in bulk.
///
/// Mirror of `ProjectMemberViewSet.create`
/// (`apps/api/plane/app/views/project/member.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn create_project_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<AddProjectMembersRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let caller_pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&caller_pm, &wm)?;

    if body.members.is_empty() {
        return Err(AppError::BadRequest("At least one member is required".into()));
    }

    for entry in &body.members {
        validate_role(entry.role)?;
    }

    let _project = project_by_id(&state.db, ws.id, project_id).await?;
    let now = chrono::Utc::now().fixed_offset();

    // Validate workspace roles — batch fetch instead of N queries.
    let member_ids: Vec<Uuid> = body.members.iter().map(|m| m.member_id).collect();
    let ws_members: std::collections::HashMap<Uuid, i16> = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.is_in(member_ids.clone()))
        .filter(workspace_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|wm| (wm.member_id, wm.role))
        .collect();

    for entry in &body.members {
        let ws_role = *ws_members.get(&entry.member_id).unwrap_or(&0);
        // Workspace admin cannot have a low role in the project.
        if ws_role >= ROLE_ADMIN && entry.role <= ROLE_MEMBER {
            return Err(AppError::BadRequest(
                "Cannot assign a role lower than workspace admin role".into(),
            ));
        }
        // Workspace guest cannot have a high role in the project.
        if ws_role <= ROLE_GUEST && entry.role >= ROLE_MEMBER {
            return Err(AppError::BadRequest(
                "Cannot assign a role higher than workspace guest role".into(),
            ));
        }
    }

    // Upsert: reactivate if already exists, or insert new.
    let existing: std::collections::HashMap<Uuid, project_members::Model> =
        project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::MemberId.is_in(member_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .filter_map(|m| m.member_id.map(|id| (id, m)))
            .collect();

    let _role_map: std::collections::HashMap<Uuid, i16> =
        body.members.iter().map(|m| (m.member_id, m.role)).collect();

    for entry in &body.members {
        if let Some(pm) = existing.get(&entry.member_id) {
            // Reactivate + update role
            let mut am: project_members::ActiveModel = pm.clone().into();
            am.role = Set(entry.role);
            am.is_active = Set(true);
            am.updated_at = Set(now);
            am.update(&state.db).await.map_err(AppError::Database)?;
        } else {
            // Create new. NOTE: project_members has several jsonb/double
            // NOT NULL columns without DEFAULT in the DB (baseline.sql
            // project_members): view_props, default_props, preferences,
            // sort_order. Django populates them via model defaults (Python),
            // SeaORM doesn't replicate that → we must set them explicitly
            // or the INSERT fails with 23502 → 500.
            project_members::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                member_id: Set(Some(entry.member_id)),
                role: Set(entry.role),
                is_active: Set(true),
                comment: Set(None),
                view_props: Set(crate::utils::django_defaults::default_props()),
                default_props: Set(crate::utils::django_defaults::default_props()),
                preferences: Set(crate::utils::django_defaults::default_preferences()),
                sort_order: Set(65535.0),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&state.db)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, project_id = %project_id, member_id = %entry.member_id, "create_project_members: insert project_members failed");
                AppError::Database(e)
            })?;
        }

        // project_user_properties — ON CONFLICT DO NOTHING
        // project_user_properties: same NOT NULL columns without DEFAULT in
        // DB (display_properties, display_filters, filters, rich_filters,
        // preferences, sort_order). Django defaults in
        // apps/api/plane/db/models/project.py:ProjectUserProperty.
        //
        // Idempotency: the previous version used
        //   ON CONFLICT (project_id, user_id) DO NOTHING
        // but the DB DOES NOT have a unique constraint on those 2 columns;
        // only:
        //   1) UNIQUE (user_id, project_id, deleted_at)  — 3 columns
        //   2) partial unique (user_id, project_id) WHERE deleted_at IS NULL
        // Postgres rejects ON CONFLICT with
        //   "there is no unique or exclusion constraint matching the
        //    ON CONFLICT specification"
        // which the handler converted to AppError::Database → 500.
        // Fix: pre-check (same pattern as project_members INSERT
        // above), without ON CONFLICT.
        let existing_pup = project_user_properties::Entity::find()
            .filter(project_user_properties::Column::ProjectId.eq(project_id))
            .filter(project_user_properties::Column::UserId.eq(entry.member_id))
            .filter(project_user_properties::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if existing_pup.is_none() {
            project_user_properties::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                user_id: Set(entry.member_id),
                display_properties: Set(crate::utils::django_defaults::default_display_properties()),
                display_filters: Set(crate::utils::django_defaults::default_display_filters()),
                filters: Set(crate::utils::django_defaults::default_filters()),
                rich_filters: Set(serde_json::json!({})),
                preferences: Set(crate::utils::django_defaults::default_preferences()),
                sort_order: Set(65535.0),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&state.db)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, project_id = %project_id, user_id = %entry.member_id, "create_project_members: insert project_user_properties failed");
                AppError::Database(e)
            })?;
        }
    }

    // Return updated members
    let updated_members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.is_in(
            body.members.iter().map(|m| m.member_id).collect::<Vec<_>>()
        ))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<serde_json::Value> = updated_members.iter().map(|m| serde_json::json!({
        "id": m.id,
        "member_id": m.member_id,
        "role": m.role,
        "project_id": m.project_id,
    })).collect();

    Ok((StatusCode::CREATED, Json(resp)))
}

// ─── POST /workspaces/{slug}/projects/{project_id}/members/leave/ ────────────

/// The authenticated user leaves the project.
///
/// Mirror of `ProjectMemberViewSet.leave`
/// (`apps/api/plane/app/views/project/member.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/leave/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn leave_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let pm = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verify they are not the only project Admin.
    if pm.role >= ROLE_ADMIN {
        let admin_count = project_members::Entity::find()
            .active()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::Role.gte(ROLE_ADMIN))
            .filter(project_members::Column::IsActive.eq(true))
            .count(&state.db)
            .await
            .map_err(AppError::Database)?;

        if admin_count <= 1 {
            return Err(AppError::BadRequest(
                "You cannot leave the project as you are the only admin.                  Please delete the project or promote another user to admin.".into(),
            ));
        }
    }

    let mut am: project_members::ActiveModel = pm.into();
    am.is_active = Set(false);
    am.updated_at = Set(chrono::Utc::now().fixed_offset());
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── POST /workspaces/{slug}/projects/{project_id}/project-views/ ────────────

/// Persists member's view_props, default_props, preferences, and sort_order.
///
/// Mirror of `ProjectUserViewsEndpoint.post`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-views/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn update_project_views(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let _ws = workspace_by_slug(&state.db, &slug).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id)
        .await?
        .ok_or(AppError::Forbidden)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut am: project_members::ActiveModel = pm.into();

    if let Some(v) = body.get("view_props") {
        am.view_props = Set(v.clone());
    }
    if let Some(v) = body.get("default_props") {
        am.default_props = Set(v.clone());
    }
    if let Some(v) = body.get("sort_order") {
        if let Some(n) = v.as_f64() {
            am.sort_order = Set(n);
        }
    }
    am.updated_at = Set(now);
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/project-views/`
///
/// Returns `view_props`, `default_props`, `preferences`, and `sort_order` of the
/// authenticated project member. Symmetric counterpart to
/// `update_project_views`: reads what `POST` persists.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-views/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Member's view props"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "Not a project member"),
    )
)]
pub async fn get_project_user_views(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ws = workspace_by_slug(&state.db, &slug).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id)
        .await?
        .ok_or(AppError::Forbidden)?;

    Ok(Json(serde_json::json!({
        "view_props":    pm.view_props,
        "default_props": pm.default_props,
        "preferences":   pm.preferences,
        "sort_order":    pm.sort_order,
    })))
}

// ── GET /workspaces/{slug}/projects/{project_id}/summary ─────────────────────

#[derive(Debug, Deserialize)]
pub struct ProjectSummaryQuery {
    pub fields: Option<String>,
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/summary`
///
/// Summary variant exposed to the internal frontend (does not require admin).
/// Reuses `v1_router::compute_project_summary` to avoid duplicating counting
/// logic, but returns flat counts at root —
/// `{ id, name, identifier, members, states, ... }` — because that's how
/// contract tests and frontend hooks consume them.
///
/// The public `/api/v1/.../summary` endpoint maintains the nested shape
/// (`{counts: {...}}`) and requires workspace Admin.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/summary",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("fields"     = Option<String>, Query, description = "CSV of fields to include"),
    ),
    responses(
        (status = 200, description = "Project counts"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "Not a workspace member"),
        (status = 404, description = "Project or workspace does not exist"),
    )
)]
pub async fn get_project_summary(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Query(query): Query<ProjectSummaryQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (project, counts) = crate::routes::v1_router::compute_project_summary(
        &state,
        &user,
        &slug,
        project_id,
        query.fields.as_deref(),
        false, // active member is enough — we don't require admin
    )
    .await?;

    let mut out = serde_json::Map::new();
    out.insert("id".into(), serde_json::json!(project.id));
    out.insert("name".into(), serde_json::json!(project.name));
    out.insert("identifier".into(), serde_json::json!(project.identifier));
    for (k, v) in counts {
        out.insert(k, v);
    }
    Ok(Json(serde_json::Value::Object(out)))
}

// ─── GET + POST + DELETE /workspaces/{slug}/user-favorite-projects/ ───────────

/// Lists user's favorite projects in the workspace.
///
/// Mirror of `ProjectFavoritesViewSet.list`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-favorite-projects/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn list_project_favorites(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let favs = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .order_by_asc(user_favorites::Column::Sequence)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = favs.iter().map(|f| serde_json::json!({
        "id": f.id,
        "entity_identifier": f.entity_identifier,
        "entity_type": f.entity_type,
        "project_id": f.project_id,
    })).collect();

    Ok(Json(result))
}

/// Adds a project to favorites.
///
/// Mirror of `ProjectFavoritesViewSet.create`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/user-favorite-projects/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn create_project_favorite(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let project_id: Uuid = body.get("project")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("project is required".into()))?;

    let now = chrono::Utc::now().fixed_offset();

    // Idempotent: do not duplicate if already exists.
    let existing = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_none() {
        user_favorites::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set("project".into()),
            entity_identifier: Set(Some(project_id)),
            project_id: Set(Some(project_id)),
            workspace_id: Set(ws.id),
            user_id: Set(user.id),
            is_folder: Set(false),
            sequence: Set(65535.0),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Removes a project from favorites.
///
/// Mirror of `ProjectFavoritesViewSet.destroy`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/user-favorite-projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn delete_project_favorite(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Hard delete (Django: soft=False)
    user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── POST + DELETE /workspaces/{slug}/projects/{project_id}/archive/ ─────────

/// Archives a project.
///
/// Mirror of `ProjectArchiveUnarchiveEndpoint.post`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/archive/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn archive_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;

    // ADMIN or MEMBER can archive
    let role = pm.as_ref().map(|m| m.role).unwrap_or(0);
    if role < ROLE_MEMBER && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let project = project_by_id(&state.db, ws.id, project_id).await?;
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let mut am: projects::ActiveModel = project.into();
    am.archived_at = Set(Some(now));
    am.updated_at = Set(now);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    // Remove project favorites (Django behavior)
    user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "archived_at": updated.archived_at })))
}

/// Unarchives a project.
///
/// Mirror of `ProjectArchiveUnarchiveEndpoint.delete`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/archive/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn unarchive_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;

    let role = pm.as_ref().map(|m| m.role).unwrap_or(0);
    if role < ROLE_MEMBER && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let project = projects::Entity::find_by_id(project_id)
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let mut am: projects::ActiveModel = project.into();
    am.archived_at = Set(None);
    am.updated_at = Set(now);
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── GET + DELETE /workspaces/{slug}/project-identifiers/ ────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct IdentifierQuery {
    pub name: Option<String>,
}

/// Verifies if a project identifier already exists in the workspace.
///
/// Mirror of `ProjectIdentifierEndpoint.get`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/project-identifiers/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn check_project_identifier(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Query(q): Query<IdentifierQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Single mode: list workspace identifiers, with optional `?name=` filter.
    // The handler name is maintained for compatibility with historical
    // use (existence check ⇔ exists>0 in response).
    let mut q_select = project_identifiers::Entity::find()
        .filter(project_identifiers::Column::WorkspaceId.eq(ws.id));

    if let Some(name) = q.name.map(|n| n.trim().to_uppercase()).filter(|n| !n.is_empty()) {
        q_select = q_select.filter(project_identifiers::Column::Name.eq(name));
    }

    let identifiers = q_select
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let identifiers_data: Vec<serde_json::Value> = identifiers
        .iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "name": i.name,
                "project": i.project_id,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "exists": identifiers_data.len(),
        "identifiers": identifiers_data,
    })))
}

/// Deletes a project identifier without an associated project.
///
/// Mirror of `ProjectIdentifierEndpoint.delete`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/project-identifiers/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn delete_project_identifier(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let name = body.get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("name is required".into()))?;

    // Do not delete if there is an active project with that identifier
    let project_exists = projects::Entity::find()
        .active()
        .filter(projects::Column::Identifier.eq(&name))
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    if project_exists > 0 {
        return Err(AppError::BadRequest(
            "Cannot delete an identifier of an existing project".into(),
        ));
    }

    project_identifiers::Entity::delete_many()
        .filter(project_identifiers::Column::Name.eq(&name))
        .filter(project_identifiers::Column::WorkspaceId.eq(ws.id))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── GET /workspaces/{slug}/projects/{project_id}/invitations/{pk}/ ──────────

/// Returns specific project invitation detail.
///
/// Mirror of `ProjectInvitationsViewset.retrieve`
/// (`apps/api/plane/app/urls/project.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn get_project_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<ProjectInvitationResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let invite = project_member_invites::Entity::find_by_id(pk)
        .active()
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ProjectInvitationResponse::from(&invite)))
}

// ── Project Join & User Invitations ─────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProjectJoinRequest {
    /// Email of the user accepting the invitation.
    pub email: String,
    /// Whether they accept or decline the invitation.
    #[serde(default)]
    pub accepted: bool,
}

/// Accepts or declines a project invitation (public endpoint).
///
/// `POST /workspaces/{slug}/projects/{project_id}/join/{pk}`
///
/// Mirror of `ProjectJoinEndpoint.post` in Django.
/// Does not require authentication (AllowAny), only the correct email.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/join/{pk}",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Invitation ID"),
    ),
    request_body = ProjectJoinRequest,
    responses(
        (status = 200, description = "Invitation accepted or declined"),
        (status = 400, description = "Already responded"),
        (status = 403, description = "Email mismatch"),
        (status = 404, description = "Invitation not found"),
    )
)]
pub async fn join_project_invitation(
    State(state): State<AppState>,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<ProjectJoinRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = body.email.trim().to_lowercase();

    let invite = project_member_invites::Entity::find_by_id(pk)
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .filter(project_member_invites::Column::WorkspaceId.is_not_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verify email matches
    if email.is_empty() || invite.email.to_lowercase() != email {
        return Err(AppError::Forbidden);
    }

    // Already responded
    if invite.responded_at.is_some() {
        return Err(AppError::BadRequest(
            "You have already responded to the invitation request".into(),
        ));
    }

    // Register response
    let mut active: project_member_invites::ActiveModel = invite.clone().into();
    active.accepted = Set(body.accepted);
    active.responded_at = Set(Some(chrono::Utc::now().into()));
    active.update(&state.db).await.map_err(AppError::Database)?;

    if !body.accepted {
        return Ok(Json(serde_json::json!({
            "message": "Project Invitation was not accepted"
        })));
    }

    // Accepted — incorporate into workspace and project
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&email))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if let Some(user) = user {
        // Ensure workspace membership
        let ws_member = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(invite.workspace_id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if ws_member.is_none() {
            let role = if invite.role >= 15 { 15i16 } else { invite.role };
            let new_wm = workspace_members::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                workspace_id: Set(invite.workspace_id),
                member_id: Set(user.id),
                role: Set(role),
                is_active: Set(true),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let _ = new_wm.insert(&state.db).await; // ignore conflict
        } else if let Some(wm) = ws_member {
            let mut wm_active: workspace_members::ActiveModel = wm.into();
            wm_active.is_active = Set(true);
            let _ = wm_active.update(&state.db).await;
        }

        // Ensure project membership
        let pm = project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if pm.is_none() {
            let new_pm = project_members::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                project_id: Set(project_id),
                member_id: Set(Some(user.id)),
                role: Set(invite.role),
                workspace_id: Set(invite.workspace_id),
                is_active: Set(true),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let _ = new_pm.insert(&state.db).await; // ignore conflict
        } else if let Some(pm_model) = pm {
            let mut pm_active: project_members::ActiveModel = pm_model.into();
            pm_active.is_active = Set(true);
            let _ = pm_active.update(&state.db).await;
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Project Invitation Accepted"
    })))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProjectInvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub accepted: bool,
    pub role: i16,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
}

/// Lists the authenticated user's project invitations.
///
/// `GET /users/me/workspaces/{slug}/projects/invitations`
///
/// Mirror of `UserProjectInvitationsViewset.list` in Django.
#[utoipa::path(
    get,
    path = "/users/me/workspaces/{slug}/projects/invitations",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
    ),
    responses(
        (status = 200, description = "List of project invitations"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn list_user_project_invitations(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<UserProjectInvitationResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let invites = project_member_invites::Entity::find()
        .filter(project_member_invites::Column::Email.eq(
            user.email.as_deref().unwrap_or(""),
        ))
        .filter(project_member_invites::Column::WorkspaceId.eq(ws.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = invites
        .iter()
        .map(|i| UserProjectInvitationResponse {
            id: i.id,
            email: i.email.clone(),
            accepted: i.accepted,
            role: i.role,
            project_id: i.project_id,
            workspace_id: i.workspace_id,
        })
        .collect();

    Ok(Json(result))
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECT DEPLOY BOARDS
// ═══════════════════════════════════════════════════════════════════════════

use crate::entities::deploy_boards;

#[derive(Debug, Serialize)]
pub struct DeployBoardResponse {
    pub id: Uuid,
    pub anchor: String,
    pub is_comments_enabled: bool,
    pub is_reactions_enabled: bool,
    pub is_votes_enabled: bool,
    pub view_props: serde_json::Value,
    pub intake_id: Option<Uuid>,
    // Frontend TPublishSettings uses "project" and "workspace" (not project_id/workspace_id)
    #[serde(rename = "project")]
    pub project_id: Option<Uuid>,
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    pub entity_name: Option<String>,
    pub entity_identifier: Option<Uuid>,
    pub is_activity_enabled: bool,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    // Required by TPublishSettings — null in CE
    pub inbox: Option<serde_json::Value>,
    pub project_details: Option<serde_json::Value>,
    pub workspace_detail: Option<serde_json::Value>,
}

impl From<deploy_boards::Model> for DeployBoardResponse {
    fn from(m: deploy_boards::Model) -> Self {
        Self {
            id: m.id,
            anchor: m.anchor,
            is_comments_enabled: m.is_comments_enabled,
            is_reactions_enabled: m.is_reactions_enabled,
            is_votes_enabled: m.is_votes_enabled,
            view_props: m.view_props,
            intake_id: m.intake_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            entity_name: m.entity_name,
            entity_identifier: m.entity_identifier,
            is_activity_enabled: m.is_activity_enabled,
            is_disabled: m.is_disabled,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            inbox: None,
            project_details: None,
            workspace_detail: None,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Deploy board or null"),
        (status = 403, description = "Forbidden"),
    )
)]
/// GET /workspaces/{slug}/projects/{project_id}/project-deploy-boards
/// Returns the project deploy board (or null if it doesn't exist).
pub async fn get_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Option<DeployBoardResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    let board = deploy_boards::Entity::find()
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::EntityIdentifier.eq(project_id))
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(board.map(DeployBoardResponse::from)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertDeployBoardRequest {
    pub is_comments_enabled: Option<bool>,
    pub is_reactions_enabled: Option<bool>,
    pub is_votes_enabled: Option<bool>,
    pub view_props: Option<serde_json::Value>,
    pub intake_id: Option<Uuid>,
}

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Deploy board created/updated"),
        (status = 403, description = "Forbidden"),
    )
)]
/// POST /workspaces/{slug}/projects/{project_id}/project-deploy-boards
/// Creates or updates the project deploy board (upsert like Django).
pub async fn upsert_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<UpsertDeployBoardRequest>,
) -> Result<Json<DeployBoardResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    // Search for existing deploy board
    let existing = deploy_boards::Entity::find()
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::EntityIdentifier.eq(project_id))
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let default_view_props = serde_json::json!({
        "list": true, "kanban": true, "calendar": true, "gantt": true, "spreadsheet": true
    });

    let board = if let Some(existing) = existing {
        let mut am: deploy_boards::ActiveModel = existing.into();
        if let Some(v) = body.is_comments_enabled {
            am.is_comments_enabled = Set(v);
        }
        if let Some(v) = body.is_reactions_enabled {
            am.is_reactions_enabled = Set(v);
        }
        if let Some(v) = body.is_votes_enabled {
            am.is_votes_enabled = Set(v);
        }
        if let Some(v) = body.view_props {
            am.view_props = Set(v);
        }
        am.intake_id = Set(body.intake_id);
        am.updated_by_id = Set(Some(user.id));
        am.update(&state.db).await.map_err(AppError::Database)?
    } else {
        let anchor = uuid::Uuid::new_v4().to_string().replace('-', "");
        let am = deploy_boards::ActiveModel {
            id: Set(Uuid::new_v4()),
            anchor: Set(anchor),
            entity_name: Set(Some("project".to_string())),
            entity_identifier: Set(Some(project_id)),
            project_id: Set(Some(project_id)),
            workspace_id: Set(ws.id),
            is_comments_enabled: Set(body.is_comments_enabled.unwrap_or(false)),
            is_reactions_enabled: Set(body.is_reactions_enabled.unwrap_or(false)),
            is_votes_enabled: Set(body.is_votes_enabled.unwrap_or(false)),
            view_props: Set(body.view_props.unwrap_or(default_view_props)),
            intake_id: Set(body.intake_id),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            is_activity_enabled: Set(true),
            is_disabled: Set(false),
            deleted_at: Set(None),
            ..Default::default()
        };
        am.insert(&state.db).await.map_err(AppError::Database)?
    };

    Ok(Json(DeployBoardResponse::from(board)))
}

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Deploy board ID"),
    ),
    responses(
        (status = 200, description = "Updated deploy board"),
        (status = 404, description = "Not found"),
    )
)]
/// PATCH /workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}
pub async fn update_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpsertDeployBoardRequest>,
) -> Result<Json<DeployBoardResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    let existing = deploy_boards::Entity::find_by_id(pk)
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: deploy_boards::ActiveModel = existing.into();
    if let Some(v) = body.is_comments_enabled {
        am.is_comments_enabled = Set(v);
    }
    if let Some(v) = body.is_reactions_enabled {
        am.is_reactions_enabled = Set(v);
    }
    if let Some(v) = body.is_votes_enabled {
        am.is_votes_enabled = Set(v);
    }
    if let Some(v) = body.view_props {
        am.view_props = Set(v);
    }
    am.intake_id = Set(body.intake_id);
    am.updated_by_id = Set(Some(user.id));
    let board = am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(DeployBoardResponse::from(board)))
}

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Deploy board ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Forbidden"),
    )
)]
/// DELETE /workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}
pub async fn delete_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    let board = deploy_boards::Entity::find_by_id(pk)
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: deploy_boards::ActiveModel = board.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECT MEMBER PREFERENCES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct MemberPreferencesResponse {
    pub preferences: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("member_id" = Uuid, Path, description = "Member user ID"),
    ),
    responses(
        (status = 200, description = "Member preferences JSON"),
        (status = 404, description = "Member not found"),
    )
)]
/// GET /workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}
pub async fn get_project_member_preferences(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, member_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<MemberPreferencesResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let pm = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(member_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(MemberPreferencesResponse {
        preferences: pm.preferences,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("member_id" = Uuid, Path, description = "Member user ID"),
    ),
    responses(
        (status = 200, description = "Updated preferences JSON"),
        (status = 404, description = "Member not found"),
    )
)]
/// PATCH /workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}
pub async fn update_project_member_preferences(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, member_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<MemberPreferencesResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let pm = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(member_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: project_members::ActiveModel = pm.into();
    am.preferences = Set(body);
    am.updated_by_id = Set(Some(user.id));
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(MemberPreferencesResponse {
        preferences: updated.preferences,
    }))
}

// ─── POST /api/users/me/workspaces/{slug}/projects/invitations ────────────────
//
// Bulk-join: accepts a list of project_ids and adds the user as a member
// of each project if they have a pending invitation or the project is public.
// Mirror of `UserProjectInvitationsViewSet.create` in Django.

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkJoinProjectsRequest {
    pub project_ids: Vec<Uuid>,
}

#[utoipa::path(
    post,
    path = "/users/me/workspaces/{slug}/projects/invitations",
    tag = "Projects",
    request_body = inline(BulkJoinProjectsRequest),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Joined projects"),
        (status = 404, description = "Workspace not found"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn join_user_project_invitations(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<BulkJoinProjectsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let now = chrono::Utc::now().fixed_offset();
    let mut joined: Vec<serde_json::Value> = Vec::new();

    for project_id in &body.project_ids {
        // Verify project exists and belongs to the workspace
        let project = projects::Entity::find_by_id(*project_id)
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .active()
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let Some(project) = project else { continue };

        // Verify if already a member
        let already_member = project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(project.id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .active()
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .is_some();

        if already_member {
            joined.push(serde_json::json!({"project_id": project_id, "status": "already_member"}));
            continue;
        }

        // Search for pending invitation
        let invite = project_member_invites::Entity::find()
            .filter(project_member_invites::Column::ProjectId.eq(project.id))
            .filter(project_member_invites::Column::Email.eq(
                user.email.as_deref().unwrap_or(""),
            ))
            .active()
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let role = invite.as_ref().map(|i| i.role).unwrap_or(10); // default: member

        // Create membership
        project_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project.id),
            workspace_id: Set(ws.id),
            member_id: Set(Some(user.id)),
            role: Set(role),
            is_active: Set(true),
            comment: Set(None),
            view_props: Set(serde_json::json!({})),
            default_props: Set(serde_json::json!({})),
            preferences: Set(serde_json::json!({})),
            sort_order: Set(65535.0),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;

        // Mark invitation as accepted if it existed
        if let Some(inv) = invite {
            let mut am: project_member_invites::ActiveModel = inv.into();
            am.accepted = Set(true);
            am.updated_at = Set(now);
            let _ = am.update(&state.db).await;
        }

        joined.push(serde_json::json!({"project_id": project_id, "status": "joined"}));
    }

    Ok(Json(joined))
}
