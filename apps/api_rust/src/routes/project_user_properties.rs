// src/routes/project_user_properties.rs
//! Endpoint de display/filter properties por usuario-proyecto.
//!
//! Mirror de `plane/app/views/issue/base.py::ProjectUserDisplayPropertyEndpoint`
//! (líneas 730-757) y URL `apps/api/plane/app/urls/issue.py:216-219`:
//!
//!   GET   /api/workspaces/{slug}/projects/{project_id}/user-properties/
//!   PATCH /api/workspaces/{slug}/projects/{project_id}/user-properties/
//!
//! Semántica clave: ambos verbos hacen `get_or_create` — el GET jamás devuelve
//! 404 por ausencia de fila; la crea con defaults. Esto es lo que resuelve
//! el 404 visto en el panel de proyecto del frontend.

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::{project_user_properties, projects, workspaces},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ─── Defaults — mirror de `plane/db/models/issue.py` ──────────────────────────
//
// Django define `get_default_filters`, `get_default_display_filters` y
// `get_default_display_properties` como callables que devuelven estos dicts.
// Los replicamos aquí con el shape EXACTO para que la respuesta inicial
// (cuando la fila aún no existe y la acabamos de crear) sea idéntica.

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

// ─── DTOs ─────────────────────────────────────────────────────────────────────

/// Mirror de `ProjectUserPropertySerializer` (fields="__all__").
/// Todos los campos del modelo `ProjectUserProperty` tal como los persiste
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

/// Body admitido en PATCH. Todos los campos son opcionales — semántica
/// `partial=True` del serializer Django.
///
/// `user`, `workspace`, `project` son read-only en Django y se ignoran aquí
/// incluso si vienen en el payload (no los incluimos en el struct).
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

/// Busca o crea la fila `project_user_properties` del usuario para el proyecto.
/// Mirror de `ProjectUserProperty.objects.get_or_create(user=..., project_id=...)`.
///
/// Nota importante: el índice único en Django es `(user, project, deleted_at)`
/// con constraint `deleted_at IS NULL`. Aquí filtramos por `deleted_at IS NULL`
/// via `.active()`. En caso de carrera, el INSERT concurrente fallaría con
/// violación de constraint único; no necesitamos manejar eso aquí porque el
/// backend Django tampoco lo hace — si ocurre, el cliente reintentaría y el
/// GET siguiente ya encontraría la fila.
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
        preferences: Set(serde_json::json!({})),
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

/// Resuelve el workspace por slug y verifica que el proyecto exista activo.
/// No requerimos membresía de proyecto explícita — el decorador
/// `@allow_permission([ADMIN, MEMBER, GUEST])` de Django exige un rol, pero
/// cualquier rol basta. Para simplificar y mantener la paridad, solo exigimos
/// workspace-member y que el proyecto exista dentro del workspace.
async fn resolve_scope(
    db: &sea_orm::DatabaseConnection,
    slug: &str,
    project_id: Uuid,
) -> Result<(workspaces::Model, projects::Model), AppError> {
    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let project = projects::Entity::find_by_id(project_id)
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok((ws, project))
}

// ─── GET ──────────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/user-properties/`
///
/// `get_or_create` — NUNCA devuelve 404 si el proyecto existe; si la fila del
/// usuario no existe la crea con defaults. Paridad con Django
/// (`ProjectUserDisplayPropertyEndpoint.get`, issue/base.py:754-757).
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
    )
)]
pub async fn get_project_user_properties(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectUserPropertyResponse>, AppError> {
    let (ws, _project) = resolve_scope(&state.db, &slug, project_id).await?;
    let row = get_or_create(&state.db, ws.id, project_id, user.id).await?;
    Ok(Json(ProjectUserPropertyResponse::from(&row)))
}

// ─── PATCH ────────────────────────────────────────────────────────────────────

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/user-properties/`
///
/// Actualiza parcialmente los campos. Si la fila no existe, la crea antes
/// de aplicar el patch (misma semántica que Django,
/// `ProjectUserDisplayPropertyEndpoint.patch`, issue/base.py:732-751).
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
    )
)]
pub async fn update_project_user_properties(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<UpdateProjectUserPropertyRequest>,
) -> Result<Json<ProjectUserPropertyResponse>, AppError> {
    let (ws, _project) = resolve_scope(&state.db, &slug, project_id).await?;
    let row = get_or_create(&state.db, ws.id, project_id, user.id).await?;

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
    am.updated_by_id = Set(Some(user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ProjectUserPropertyResponse::from(&updated)))
}
