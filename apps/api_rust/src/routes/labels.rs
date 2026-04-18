// src/routes/labels.rs
//! Endpoints de Labels (etiquetas de proyecto).
//!
//!   GET    /api/workspaces/{slug}/projects/{project_id}/labels/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/labels/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/labels/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/labels/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/labels/{pk}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::labels,
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LabelResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub color: String,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub sort_order: f64,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl LabelResponse {
    fn from_model(m: labels::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            color: m.color,
            parent_id: m.parent_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            sort_order: m.sort_order,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateLabelRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<f64>,
}

// ── GET /labels/ ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/labels/",
    tag = "Labels",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Lista de labels")),
    security(("TokenAuth" = []))
)]
pub async fn list_labels(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<LabelResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = labels::Entity::find()
        .active()
        .filter(labels::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(labels::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(LabelResponse::from_model).collect()))
}

// ── POST /labels/ ─────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/labels/",
    tag = "Labels",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Label creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_label(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateLabelRequest>,
) -> Result<(StatusCode, Json<LabelResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    // NOTA: created_at / updated_at se setean explícitamente porque
    // labels::ActiveModelBehavior está vacío (sin hook before_save) y las
    // columnas son NOT NULL. Dejarlas con Default::default() hacía que SeaORM
    // enviara NULL y la BD rechazara con 23502 ("violates not-null constraint").
    // Mismo patrón que issue_extras.rs / pages.rs / workspace_extras.rs.
    let now = chrono::Utc::now().into();

    let label = labels::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        color: Set(body.color.unwrap_or_else(|| "#6b7280".to_owned())),
        description: Set(body.description.unwrap_or_default()),
        parent_id: Set(body.parent_id),
        project_id: Set(Some(guard.project.id)),
        workspace_id: Set(guard.workspace.id),
        sort_order: Set(65535.0),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        external_id: Set(None),
        external_source: Set(None),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(LabelResponse::from_model(label))))
}

// ── GET /labels/{pk}/ ─────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/labels/{pk}/",
    tag = "Labels",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Label ID"),
    ),
    responses(
        (status = 200, description = "Detalle del label"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_label(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<LabelResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let label = labels::Entity::find_by_id(pk)
        .active()
        .filter(labels::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(LabelResponse::from_model(label)))
}

// ── PATCH /labels/{pk}/ ───────────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/labels/{pk}/",
    tag = "Labels",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Label ID"),
    ),
    responses(
        (status = 200, description = "Label actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_label(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateLabelRequest>,
) -> Result<Json<LabelResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let label = labels::Entity::find_by_id(pk)
        .active()
        .filter(labels::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: labels::ActiveModel = label.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(color) = body.color {
        am.color = Set(color);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    if body.parent_id.is_some() {
        am.parent_id = Set(body.parent_id);
    }
    if let Some(order) = body.sort_order {
        am.sort_order = Set(order);
    }
    am.updated_by_id = Set(Some(guard.user.id));
    // Django usa auto_now=True en updated_at (TimeAuditModel). En SeaORM hay
    // que setearlo explícitamente, de lo contrario ActiveModel lo deja
    // Unchanged y el UPDATE no lo toca.
    am.updated_at = Set(chrono::Utc::now().into());

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(LabelResponse::from_model(updated)))
}

// ── DELETE /labels/{pk}/ ──────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/labels/{pk}/",
    tag = "Labels",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Label ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_label(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let label = labels::Entity::find_by_id(pk)
        .active()
        .filter(labels::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: labels::ActiveModel = label.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
