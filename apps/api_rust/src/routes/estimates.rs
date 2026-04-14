// src/routes/estimates.rs
//! Endpoints de Estimates.
//!
//!   GET    /api/workspaces/{slug}/projects/{project_id}/estimates/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/estimates/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}/

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
    entities::{estimate_points, estimates},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct EstimatePointResponse {
    pub id: Uuid,
    pub key: i32,
    pub value: String,
    pub description: String,
    pub estimate_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
}

impl EstimatePointResponse {
    fn from_model(m: estimate_points::Model) -> Self {
        Self {
            id: m.id,
            key: m.key,
            value: m.value,
            description: m.description,
            estimate_id: m.estimate_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EstimateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub last_used: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub points: Vec<EstimatePointResponse>,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct EstimatePointInput {
    pub key: i32,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEstimateRequest {
    pub name: String,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub points: Vec<EstimatePointInput>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEstimateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub last_used: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEstimatePointRequest {
    pub key: i32,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEstimatePointRequest {
    pub key: Option<i32>,
    pub value: Option<String>,
    pub description: Option<String>,
}

// ── Helper ────────────────────────────────────────────────────────────────────

async fn enrich_estimate(
    db: &sea_orm::DatabaseConnection,
    est: estimates::Model,
) -> Result<EstimateResponse, AppError> {
    let points = estimate_points::Entity::find()
        .active()
        .filter(estimate_points::Column::EstimateId.eq(est.id))
        .order_by_asc(estimate_points::Column::Key)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(EstimateResponse {
        id: est.id,
        name: est.name,
        description: est.description,
        r#type: est.r#type,
        last_used: est.last_used,
        project_id: est.project_id,
        workspace_id: est.workspace_id,
        created_by_id: est.created_by_id,
        created_at: est.created_at,
        updated_at: est.updated_at,
        points: points.into_iter().map(EstimatePointResponse::from_model).collect(),
    })
}

// ── GET /estimates/ ───────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Lista de estimates con sus puntos")),
    security(("TokenAuth" = []))
)]
pub async fn list_estimates(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<EstimateResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = estimates::Entity::find()
        .active()
        .filter(estimates::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(estimates::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let mut result = Vec::with_capacity(rows.len());
    for est in rows {
        result.push(enrich_estimate(&state.db, est).await?);
    }
    Ok(Json(result))
}

// ── POST /estimates/ ──────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Estimate creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_estimate(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateEstimateRequest>,
) -> Result<(StatusCode, Json<EstimateResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let points = body.points.clone();

    let estimate = state
        .db
        .transaction::<_, estimates::Model, AppError>(|txn| {
            let name = body.name.clone();
            let description = body.description.clone().unwrap_or_default();
            let est_type = body.r#type.clone().unwrap_or_else(|| "category".to_owned());
            Box::pin(async move {
                let est = estimates::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    name: Set(name),
                    description: Set(description),
                    r#type: Set(est_type),
                    last_used: Set(false),
                    project_id: Set(project_id),
                    workspace_id: Set(workspace_id),
                    created_by_id: Set(Some(user_id)),
                    updated_by_id: Set(Some(user_id)),
                    ..Default::default()
                }
                .insert(txn)
                .await
                .map_err(AppError::Database)?;

                for p in &points {
                    estimate_points::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        key: Set(p.key),
                        value: Set(p.value.clone()),
                        description: Set(p.description.clone().unwrap_or_default()),
                        estimate_id: Set(est.id),
                        project_id: Set(project_id),
                        workspace_id: Set(workspace_id),
                        created_by_id: Set(Some(user_id)),
                        updated_by_id: Set(Some(user_id)),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await
                    .map_err(AppError::Database)?;
                }

                Ok(est)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    let response = enrich_estimate(&state.db, estimate).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// ── GET /estimates/{estimate_id}/ ─────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("estimate_id" = Uuid, Path, description = "Estimate ID"),
    ),
    responses(
        (status = 200, description = "Detalle del estimate"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_estimate(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, estimate_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<EstimateResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let est = estimates::Entity::find_by_id(estimate_id)
        .active()
        .filter(estimates::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(enrich_estimate(&state.db, est).await?))
}

// ── PATCH /estimates/{estimate_id}/ ───────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("estimate_id" = Uuid, Path, description = "Estimate ID"),
    ),
    responses(
        (status = 200, description = "Estimate actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_estimate(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, estimate_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateEstimateRequest>,
) -> Result<Json<EstimateResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let est = estimates::Entity::find_by_id(estimate_id)
        .active()
        .filter(estimates::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: estimates::ActiveModel = est.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    if let Some(t) = body.r#type {
        am.r#type = Set(t);
    }
    if let Some(lu) = body.last_used {
        am.last_used = Set(lu);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(enrich_estimate(&state.db, updated).await?))
}

// ── DELETE /estimates/{estimate_id}/ ──────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("estimate_id" = Uuid, Path, description = "Estimate ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_estimate(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, estimate_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let est = estimates::Entity::find_by_id(estimate_id)
        .active()
        .filter(estimates::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Soft-delete points too
    let points = estimate_points::Entity::find()
        .active()
        .filter(estimate_points::Column::EstimateId.eq(estimate_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for p in points {
        let mut am: estimate_points::ActiveModel = p.into();
        am.deleted_at = Set(Some(now));
        am.update(&state.db).await.map_err(AppError::Database)?;
    }

    let mut am: estimates::ActiveModel = est.into();
    am.deleted_at = Set(Some(now));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── POST /estimates/{estimate_id}/estimate-points/ ────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("estimate_id" = Uuid, Path, description = "Estimate ID"),
    ),
    responses(
        (status = 201, description = "Punto creado"),
        (status = 404, description = "Estimate no encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_estimate_point(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, estimate_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateEstimatePointRequest>,
) -> Result<(StatusCode, Json<EstimatePointResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let _ = estimates::Entity::find_by_id(estimate_id)
        .active()
        .filter(estimates::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let point = estimate_points::ActiveModel {
        id: Set(Uuid::new_v4()),
        key: Set(body.key),
        value: Set(body.value),
        description: Set(body.description.unwrap_or_default()),
        estimate_id: Set(estimate_id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(EstimatePointResponse::from_model(point))))
}

// ── PATCH /estimate-points/{pk}/ ─────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("estimate_id" = Uuid, Path, description = "Estimate ID"),
        ("pk" = Uuid, Path, description = "EstimatePoint ID"),
    ),
    responses(
        (status = 200, description = "Punto actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_estimate_point(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, estimate_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
    Json(body): Json<UpdateEstimatePointRequest>,
) -> Result<Json<EstimatePointResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let point = estimate_points::Entity::find_by_id(pk)
        .active()
        .filter(estimate_points::Column::EstimateId.eq(estimate_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: estimate_points::ActiveModel = point.into();
    if let Some(key) = body.key {
        am.key = Set(key);
    }
    if let Some(val) = body.value {
        am.value = Set(val);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(EstimatePointResponse::from_model(updated)))
}

// ── DELETE /estimate-points/{pk}/ ────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}/",
    tag = "Estimates",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("estimate_id" = Uuid, Path, description = "Estimate ID"),
        ("pk" = Uuid, Path, description = "EstimatePoint ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_estimate_point(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, estimate_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let point = estimate_points::Entity::find_by_id(pk)
        .active()
        .filter(estimate_points::Column::EstimateId.eq(estimate_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: estimate_points::ActiveModel = point.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
