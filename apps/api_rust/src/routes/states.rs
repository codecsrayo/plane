// src/routes/states.rs
//! State Endpoints — Phase 3.
//!
//! Equivalent to `plane/app/views/state/base.py` in Django.
//! Authentication: session cookie **or** API key (via `AnyAuth`).
//! Authorization: active project membership.
//! - Read: any active member (Guest, Viewer, Member, Admin).
//! - Write / delete / mark-default: only Project Admin or Workspace Admin.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{any_auth::AnyAuth, permissions::ROLE_ADMIN},
    entities::{project_members, projects, states, workspace_members},
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

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

/// The user needs to be a Project Admin or Workspace Admin.
fn require_admin(
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

/// The user needs to be an active project member (any role).
fn require_project_member(pm: &Option<project_members::Model>) -> Result<(), AppError> {
    if pm.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub color: String,
    pub slug: String,
    pub group: String,
    /// Normalized position within the group (calculated at response time).
    pub sequence: f64,
    /// sequence alias — required by frontend's IState (used in StateGroupIcon as percentage).
    pub order: f64,
    pub default: bool,
    pub is_triage: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&states::Model> for StateResponse {
    fn from(s: &states::Model) -> Self {
        Self {
            id: s.id,
            name: s.name.clone(),
            description: s.description.clone(),
            color: s.color.clone(),
            slug: s.slug.clone(),
            group: s.group.clone(),
            sequence: s.sequence,
            order: s.sequence,
            default: s.default,
            is_triage: s.is_triage,
            project_id: s.project_id,
            workspace_id: s.workspace_id,
            created_by_id: s.created_by_id,
            updated_by_id: s.updated_by_id,
            external_id: s.external_id.clone(),
            external_source: s.external_source.clone(),
            created_at: s.created_at.with_timezone(&Utc),
            updated_at: s.updated_at.with_timezone(&Utc),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateStateRequest {
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub description: String,
    pub group: String,
    pub sequence: Option<f64>,
    pub default: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStateRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub group: Option<String>,
    pub sequence: Option<f64>,
    pub default: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct GroupedQuery {
    #[serde(default)]
    pub grouped: Option<String>,
}

// ─── Path params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WorkspaceProjectPath {
    pub slug: String,
    pub project_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct WorkspaceProjectStatePath {
    pub slug: String,
    pub project_id: Uuid,
    pub pk: Uuid,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// Lists all active states (non-triage) of a project.
///
/// Accepts `?grouped=true` to group by state group.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/states/",
    tag = "States",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project UUID"),
        GroupedQuery,
    ),
    responses(
        (status = 200, description = "List of states"),
        (status = 403, description = "Forbidden"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_states(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectPath>,
    Query(q): Query<GroupedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_project_member(&pm)?;

    let raw_states = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .order_by_asc(states::Column::Sequence)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Normalizes `sequence` within each group (mirrors Django logic)
    let mut responses: Vec<StateResponse> = raw_states.iter().map(StateResponse::from).collect();
    normalize_sequence_by_group(&mut responses);

    if q.grouped.as_deref() == Some("true") {
        // Returns map { group: [states] }
        let mut map: std::collections::HashMap<String, Vec<StateResponse>> =
            std::collections::HashMap::new();
        for s in responses {
            map.entry(s.group.clone()).or_default().push(s);
        }
        let value = serde_json::to_value(map)
            .map_err(|e| AppError::Internal(e.into()))?;
        return Ok((StatusCode::OK, Json(value)).into_response());
    }

    Ok((StatusCode::OK, Json(responses)).into_response())
}

/// Gets a specific state by ID.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path, description = "State UUID"),
    ),
    responses(
        (status = 200, description = "State found", body = StateResponse),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_state(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_project_member(&pm)?;

    let state = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::OK, Json(StateResponse::from(&state))).into_response())
}

/// Creates a new state in the project. Admin only.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/states/",
    tag = "States",
    request_body = CreateStateRequest,
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
    ),
    responses(
        (status = 201, description = "State created", body = StateResponse),
        (status = 400, description = "Duplicate name or invalid data"),
        (status = 403, description = "Forbidden"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_state(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectPath>,
    Json(body): Json<CreateStateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_admin(&pm, &wm)?;

    validate_group(&body.group)?;

    // Unique name per project (excluding soft-deleted)
    let duplicate = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::Name.eq(&body.name))
        .one(db)
        .await
        .map_err(AppError::Database)?;
    if duplicate.is_some() {
        return Err(AppError::BadRequest("The state name is already taken".into()));
    }

    // Calculates maximum sequence
    let max_seq = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .iter()
        .map(|s| s.sequence as i64)
        .max()
        .unwrap_or(0);

    let sequence = body.sequence.unwrap_or((max_seq + 15000) as f64);
    let slug = slugify(&body.name);
    let now = chrono::Utc::now().fixed_offset();

    let new_state = states::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: Set(body.name.clone()),
        description: Set(body.description.clone()),
        color: Set(body.color.clone()),
        slug: Set(slug),
        group: Set(body.group.clone()),
        sequence: Set(sequence),
        default: Set(body.default.unwrap_or(false)),
        is_triage: Set(false),
        project_id: Set(p.project_id),
        workspace_id: Set(ws.id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        external_id: Set(None),
        external_source: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let inserted = new_state
        .insert(db)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") {
                AppError::BadRequest("The state name is already taken".into())
            } else {
                AppError::Database(e)
            }
        })?;

    Ok((StatusCode::CREATED, Json(StateResponse::from(&inserted))).into_response())
}

/// Partially updates a state. Admin only.
#[utoipa::path(
    patch,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/",
    tag = "States",
    request_body = UpdateStateRequest,
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path),
    ),
    responses(
        (status = 200, description = "State updated", body = StateResponse),
        (status = 400, description = "Duplicate name"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_state(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
    Json(body): Json<UpdateStateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_admin(&pm, &wm)?;

    let existing = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Validate unique name if it is being changed
    if let Some(ref new_name) = body.name {
        let duplicate = states::Entity::find()
            .active()
            .filter(states::Column::ProjectId.eq(p.project_id))
            .filter(states::Column::Name.eq(new_name))
            .filter(states::Column::Id.ne(p.pk))
            .one(db)
            .await
            .map_err(AppError::Database)?;
        if duplicate.is_some() {
            return Err(AppError::BadRequest("The state name is already taken".into()));
        }
    }

    if let Some(ref g) = body.group {
        validate_group(g)?;
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: states::ActiveModel = existing.into();

    if let Some(name) = body.name {
        let slug = slugify(&name);
        active.slug = Set(slug);
        active.name = Set(name);
    }
    if let Some(color) = body.color {
        active.color = Set(color);
    }
    if let Some(desc) = body.description {
        active.description = Set(desc);
    }
    if let Some(group) = body.group {
        active.group = Set(group);
    }
    if let Some(seq) = body.sequence {
        active.sequence = Set(seq);
    }
    if let Some(def) = body.default {
        active.default = Set(def);
    }
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(db).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("already exists") || msg.contains("duplicate") {
            AppError::BadRequest("The state name is already taken".into())
        } else {
            AppError::Database(e)
        }
    })?;

    Ok((StatusCode::OK, Json(StateResponse::from(&updated))).into_response())
}

/// Deletes a state (only if it's empty and not default). Admin only.
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path),
    ),
    responses(
        (status = 204, description = "State deleted"),
        (status = 400, description = "Default state or with issues"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_state(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_admin(&pm, &wm)?;

    let state = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if state.default {
        return Err(AppError::BadRequest(
            "Default state cannot be deleted".into(),
        ));
    }

    // Verify that there are no issues in this state
    use crate::entities::issues;
    let issue_exists = issues::Entity::find()
        .active()
        .filter(issues::Column::StateId.eq(p.pk))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if issue_exists {
        return Err(AppError::BadRequest(
            "The state is not empty, only empty states can be deleted".into(),
        ));
    }

    // Soft-delete: set deleted_at
    let now = chrono::Utc::now().fixed_offset();
    let mut active: states::ActiveModel = state.into();
    active.deleted_at = Set(Some(now));
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Gets the triage state of the project (intake state).
///
/// If the project never had a triage state (e.g. created before automatic
/// seeding was implemented, or migrated from Django without it), it creates it
/// idempotently — mirroring Django logic in
/// `plane/app/views/intake/base.py` (lines 239-249).
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/intake-state/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
    ),
    responses(
        (status = 200, description = "Triage state", body = StateResponse),
        (status = 404, description = "Project or workspace not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn intake_state(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectPath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_project_member(&pm)?;

    // Verify that the project actually belongs to the workspace (and is not deleted)
    let _project = projects::Entity::find_by_id(p.project_id)
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Search for the existing triage state
    if let Some(state) = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)?
    {
        return Ok((StatusCode::OK, Json(StateResponse::from(&state))).into_response());
    }

    // Does not exist -> create idempotently (get-or-create), same as Django in intake POST.
    // Values are the same as used by Django: name="Triage", color="#4E5355",
    // sequence=65000, group="triage", default=false, is_triage=true.
    let now = chrono::Utc::now().fixed_offset();
    let new_state = states::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: Set("Triage".to_string()),
        description: Set(String::new()),
        color: Set("#4E5355".to_string()),
        slug: Set("triage".to_string()),
        group: Set("triage".to_string()),
        sequence: Set(65000.0),
        default: Set(false),
        is_triage: Set(true),
        project_id: Set(p.project_id),
        workspace_id: Set(ws.id),
        // Triage state is a system state — it has no explicit owner.
        created_by_id: Set(None),
        updated_by_id: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let inserted = new_state
        .insert(db)
        .await
        .map_err(AppError::Database)?;

    Ok((StatusCode::OK, Json(StateResponse::from(&inserted))).into_response())
}

/// Marks a state as the project default (deactivates the previous one). Admin only.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path),
    ),
    responses(
        (status = 204, description = "Marked as default"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn mark_default(
    State(app): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user.id).await?;
    let pm = project_member_for_user(db, p.project_id, user.id).await?;
    require_admin(&pm, &wm)?;

    // Verify that the state exists
    let _target = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let txn = db.begin().await.map_err(AppError::Database)?;
    let now = chrono::Utc::now().fixed_offset();

    // Remove default from all project states
    let all_defaults = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::Default.eq(true))
        .all(&txn)
        .await
        .map_err(AppError::Database)?;

    for s in all_defaults {
        let mut a: states::ActiveModel = s.into();
        a.default = Set(false);
        a.updated_at = Set(now);
        a.update(&txn).await.map_err(AppError::Database)?;
    }

    // Mark the selected one
    let target = states::Entity::find_by_id(p.pk)
        .one(&txn)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;
    let mut a: states::ActiveModel = target.into();
    a.default = Set(true);
    a.updated_at = Set(now);
    a.update(&txn).await.map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

// ─── Internal Utilities ──────────────────────────────────────────────────────

/// Normalizes the sequence within each group to a value between 0 and 1
/// (relative position), mirroring Django logic.
fn normalize_sequence_by_group(states: &mut [StateResponse]) {
    use std::collections::HashMap;
    let mut group_counts: HashMap<String, usize> = HashMap::new();
    let mut group_indices: HashMap<String, usize> = HashMap::new();

    for s in states.iter() {
        *group_counts.entry(s.group.clone()).or_insert(0) += 1;
    }
    for s in states.iter_mut() {
        let count = *group_counts.get(&s.group).unwrap_or(&1);
        let idx = group_indices.entry(s.group.clone()).or_insert(0);
        *idx += 1;
        s.sequence = *idx as f64 / count as f64;
    }
}

/// Validates that the group is one of the accepted values.
fn validate_group(group: &str) -> Result<(), AppError> {
    const VALID: &[&str] = &[
        "backlog", "unstarted", "started", "completed", "cancelled", "triage",
    ];
    if VALID.contains(&group) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Invalid group '{}'. Valid values: backlog, unstarted, started, completed, cancelled, triage",
            group
        )))
    }
}

/// Generates a simple slug from a name (lowercase, replace spaces with hyphens).
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}
