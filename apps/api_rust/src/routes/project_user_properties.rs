// src/routes/project_user_properties.rs
//! Endpoint for display/filter properties by user-project.
//!
//! Mirror of `plane/app/views/issue/base.py::ProjectUserDisplayPropertyEndpoint`
//! (lines 730-757) and URL `apps/api/plane/app/urls/issue.py:216-219`:
//!
//!   GET   /api/workspaces/{slug}/projects/{project_id}/user-properties/
//!   PATCH /api/workspaces/{slug}/projects/{project_id}/user-properties/
//!
//! Key semantics: both verbs perform `get_or_create` — GET never returns
//! 404 due to row absence; it creates it with defaults. This is what resolves
//! the 404 seen in the frontend project panel.

use axum::{
    extract::State,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::extractors::ProjectMemberGuard,
    entities::project_user_properties,
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ─── Defaults — mirror of `plane/db/models/issue.py` ──────────────────────────
//
// Django defines `get_default_filters`, `get_default_display_filters` and
// `get_default_display_properties` as callables that return these dicts.
// We replicate them here with the EXACT shape so that the initial response
// (when the row does not yet exist and we just created it) is identical.

fn default_filters() -> serde_json::Value {
    serde_json::json!({
        "priority": null,
        "state": null,
        "state_group": null,
        "assignees": null,
        "created_by": null,
        "labels": null,
        "start_date": null,
        "target_date": null,
        "subscriber": null,
    })
}

fn default_display_filters() -> serde_json::Value {
    serde_json::json!({
        "group_by": null,
        "order_by": "-created_at",
        "type": null,
        "sub_issue": true,
        "show_empty_groups": true,
        "layout": "list",
        "calendar_date_range": "",
    })
}

fn default_display_properties() -> serde_json::Value {
    serde_json::json!({
        "assignee": true,
        "attachment_count": true,
        "created_on": true,
        "due_date": true,
        "estimate": true,
        "key": true,
        "labels": true,
        "link": true,
        "priority": true,
        "start_date": true,
        "state": true,
        "sub_issue_count": true,
        "updated_on": true,
    })
}

/// Mirror de `get_default_preferences` en
/// `apps/api/plane/db/models/project.py:64-65`.
fn default_preferences() -> serde_json::Value {
    serde_json::json!({
        "pages": {
            "block_display": true
        },
        "navigation": {
            "default_tab": "work_items",
            "hide_in_more_menu": []
        }
    })
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

/// Mirror of `ProjectUserPropertySerializer` (fields="__all__").
/// All model fields of `ProjectUserProperty` as persisted by
/// `apps/api/plane/db/models/project.py:342-373`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectUserPropertyResponse {
    pub id: Uuid,
    pub user: Uuid,
    pub project: Uuid,
    pub workspace: Uuid,
    pub filters: serde_json::Value,
    pub display_filters: serde_json::Value,
    pub display_properties: serde_json::Value,
    pub rich_filters: serde_json::Value,
    pub preferences: serde_json::Value,
    pub sort_order: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl From<&project_user_properties::Model> for ProjectUserPropertyResponse {
    fn from(m: &project_user_properties::Model) -> Self {
        Self {
            id: m.id,
            user: m.user_id,
            project: m.project_id,
            workspace: m.workspace_id,
            filters: m.filters.clone(),
            display_filters: m.display_filters.clone(),
            display_properties: m.display_properties.clone(),
            rich_filters: m.rich_filters.clone(),
            preferences: m.preferences.clone(),
            sort_order: m.sort_order,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
        }
    }
}

/// Body allowed in PATCH. All fields are optional — `partial=True` semantics
/// from the Django serializer.
///
/// `user`, `workspace`, `project` are read-only in Django and are ignored here
/// even if they come in the payload (we don't include them in the struct).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectUserPropertyRequest {
    pub filters: Option<serde_json::Value>,
    pub display_filters: Option<serde_json::Value>,
    pub display_properties: Option<serde_json::Value>,
    pub rich_filters: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
    pub sort_order: Option<f64>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Finds or creates the `project_user_properties` row for the user and project.
/// Mirror of `ProjectUserProperty.objects.get_or_create(user=..., project_id=...)`.
///
/// Important note: the unique index in Django is `(user, project, deleted_at)`
/// with the `deleted_at IS NULL` constraint. Here we filter by `deleted_at IS NULL`
/// via `.active()`. In case of a race condition, the concurrent INSERT would fail
/// with a unique constraint violation; we don't need to handle that here because
/// the Django backend doesn't either — if it happens, the client would retry and
/// the subsequent GET would find the row.
async fn get_or_create(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<project_user_properties::Model, AppError> {
    if let Some(existing) = project_user_properties::Entity::find()
        .active()
        .filter(project_user_properties::Column::UserId.eq(user_id))
        .filter(project_user_properties::Column::ProjectId.eq(project_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
    {
        return Ok(existing);
    }

    let now = chrono::Utc::now().fixed_offset();
    let created = project_user_properties::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        project_id: Set(project_id),
        workspace_id: Set(workspace_id),
        filters: Set(default_filters()),
        display_filters: Set(default_display_filters()),
        display_properties: Set(default_display_properties()),
        rich_filters: Set(serde_json::json!({})),
        preferences: Set(default_preferences()),
        sort_order: Set(65535.0),
        created_at: Set(now),
        updated_at: Set(now),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .map_err(AppError::Database)?;

    Ok(created)
}

// ─── GET ──────────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/user-properties/`
///
/// Parity with Django (`ProjectUserDisplayPropertyEndpoint.get`,
/// issue/base.py:754-757) + decorator `@allow_permission([ADMIN, MEMBER, GUEST])`:
///
/// - `ProjectMemberGuard` validates: workspace by slug + live project in that
///   workspace + active user membership to the project. If any condition
///   fails: 404 (workspace/project does not exist) or 403 (no membership),
///   replicating Django's semantics exactly.
/// - `get_or_create` ensures we NEVER return 404 due to property row absence
///   — if it doesn't exist, it's created with defaults. This resolves the
///   original 404 on the project panel in the frontend.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-properties/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "User properties", body = ProjectUserPropertyResponse),
        (status = 403, description = "User is not a member of the project"),
        (status = 404, description = "Workspace or project not found"),
    )
)]
pub async fn get_project_user_properties(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<ProjectUserPropertyResponse>, AppError> {
    let row = get_or_create(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        guard.user.id,
    )
    .await?;
    Ok(Json(ProjectUserPropertyResponse::from(&row)))
}

// ─── PATCH ────────────────────────────────────────────────────────────────────

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/user-properties/`
///
/// Partially updates the fields. If the row does not exist, it creates it before
/// applying the patch (same semantics as Django,
/// `ProjectUserDisplayPropertyEndpoint.patch`, issue/base.py:732-751).
///
/// Permission validation identical to Django: requires active membership to
/// the project (`ProjectMemberGuard`). No membership → 403.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-properties/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    request_body = UpdateProjectUserPropertyRequest,
    responses(
        (status = 200, description = "Updated user properties", body = ProjectUserPropertyResponse),
        (status = 403, description = "User is not a member of the project"),
        (status = 404, description = "Workspace or project not found"),
    )
)]
pub async fn update_project_user_properties(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<UpdateProjectUserPropertyRequest>,
) -> Result<Json<ProjectUserPropertyResponse>, AppError> {
    let row = get_or_create(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        guard.user.id,
    )
    .await?;

    let mut am: project_user_properties::ActiveModel = row.into();
    if let Some(v) = body.filters {
        am.filters = Set(v);
    }
    if let Some(v) = body.display_filters {
        am.display_filters = Set(v);
    }
    if let Some(v) = body.display_properties {
        am.display_properties = Set(v);
    }
    if let Some(v) = body.rich_filters {
        am.rich_filters = Set(v);
    }
    if let Some(v) = body.preferences {
        am.preferences = Set(v);
    }
    if let Some(v) = body.sort_order {
        am.sort_order = Set(v);
    }
    am.updated_at = Set(chrono::Utc::now().fixed_offset());
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ProjectUserPropertyResponse::from(&updated)))
}
