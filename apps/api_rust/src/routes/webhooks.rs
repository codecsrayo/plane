// src/routes/webhooks.rs
//! Endpoints de Webhooks de workspace.
//!
//!   GET    /api/workspaces/{slug}/webhooks/
//!   POST   /api/workspaces/{slug}/webhooks/
//!   GET    /api/workspaces/{slug}/webhooks/{pk}/
//!   PATCH  /api/workspaces/{slug}/webhooks/{pk}/
//!   DELETE /api/workspaces/{slug}/webhooks/{pk}/
//!   POST   /api/workspaces/{slug}/webhooks/{pk}/regenerate/
//!   GET    /api/workspaces/{slug}/webhook-logs/{webhook_id}/

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
    auth::{extractors::WorkspaceMemberGuard, permissions::require_workspace_admin},
    entities::{webhook_logs, webhooks},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub is_active: bool,
    pub secret_key: String,
    pub project: bool,
    pub issue: bool,
    pub module: bool,
    pub cycle: bool,
    pub issue_comment: bool,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl WebhookResponse {
    fn from_model(m: webhooks::Model) -> Self {
        Self {
            id: m.id,
            url: m.url,
            is_active: m.is_active,
            secret_key: m.secret_key,
            project: m.project,
            issue: m.issue,
            module: m.module,
            cycle: m.cycle,
            issue_comment: m.issue_comment,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WebhookLogResponse {
    pub id: Uuid,
    pub event_type: Option<String>,
    pub response_status: Option<String>,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub retry_count: i16,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub is_active: Option<bool>,
    pub project: Option<bool>,
    pub issue: Option<bool>,
    pub module: Option<bool>,
    pub cycle: Option<bool>,
    pub issue_comment: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWebhookRequest {
    pub url: Option<String>,
    pub is_active: Option<bool>,
    pub project: Option<bool>,
    pub issue: Option<bool>,
    pub module: Option<bool>,
    pub cycle: Option<bool>,
    pub issue_comment: Option<bool>,
}

/// Genera un secret key a partir de dos UUIDs v4 concatenados (sin guiones).
/// Produce 64 caracteres hexadecimales con 128 bits de entropía del OS.
fn generate_secret() -> String {
    format!(
        "{}{}",
        Uuid::new_v4().as_simple(),
        Uuid::new_v4().as_simple()
    )
}

// ── GET /webhooks/ ────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/webhooks/",
    tag = "Webhooks",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses((status = 200, description = "Lista de webhooks")),
    security(("TokenAuth" = []))
)]
pub async fn list_webhooks(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<Vec<WebhookResponse>>, AppError> {
    require_workspace_admin(&guard.member)?;

    let rows = webhooks::Entity::find()
        .active()
        .filter(webhooks::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(webhooks::Column::IsInternal.eq(false))
        .order_by_asc(webhooks::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(WebhookResponse::from_model).collect()))
}

// ── POST /webhooks/ ───────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/webhooks/",
    tag = "Webhooks",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Webhook creado"),
        (status = 400, description = "URL inválida"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_webhook(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

    if body.url.trim().is_empty() {
        return Err(AppError::BadRequest("url es requerida".into()));
    }

    // created_at/updated_at explícitos (NOT NULL sin DEFAULT).
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let webhook = webhooks::ActiveModel {
        id: Set(Uuid::new_v4()),
        url: Set(body.url),
        is_active: Set(body.is_active.unwrap_or(true)),
        secret_key: Set(generate_secret()),
        project: Set(body.project.unwrap_or(false)),
        issue: Set(body.issue.unwrap_or(true)),
        module: Set(body.module.unwrap_or(false)),
        cycle: Set(body.cycle.unwrap_or(false)),
        issue_comment: Set(body.issue_comment.unwrap_or(false)),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        is_internal: Set(false),
        version: Set("v1".to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(WebhookResponse::from_model(webhook))))
}

// ── GET /webhooks/{pk}/ ───────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/webhooks/{pk}/",
    tag = "Webhooks",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Webhook ID"),
    ),
    responses(
        (status = 200, description = "Detalle del webhook"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_webhook(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<WebhookResponse>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wh = webhooks::Entity::find_by_id(pk)
        .active()
        .filter(webhooks::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(WebhookResponse::from_model(wh)))
}

// ── PATCH /webhooks/{pk}/ ─────────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/webhooks/{pk}/",
    tag = "Webhooks",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Webhook ID"),
    ),
    responses(
        (status = 200, description = "Webhook actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_webhook(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wh = webhooks::Entity::find_by_id(pk)
        .active()
        .filter(webhooks::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: webhooks::ActiveModel = wh.into();
    if let Some(url) = body.url {
        am.url = Set(url);
    }
    if let Some(active) = body.is_active {
        am.is_active = Set(active);
    }
    if let Some(v) = body.project { am.project = Set(v); }
    if let Some(v) = body.issue { am.issue = Set(v); }
    if let Some(v) = body.module { am.module = Set(v); }
    if let Some(v) = body.cycle { am.cycle = Set(v); }
    if let Some(v) = body.issue_comment { am.issue_comment = Set(v); }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(WebhookResponse::from_model(updated)))
}

// ── DELETE /webhooks/{pk}/ ────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/webhooks/{pk}/",
    tag = "Webhooks",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Webhook ID"),
    ),
    responses((status = 204, description = "Eliminado")),
    security(("TokenAuth" = []))
)]
pub async fn delete_webhook(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let wh = webhooks::Entity::find_by_id(pk)
        .active()
        .filter(webhooks::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: webhooks::ActiveModel = wh.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── POST /webhooks/{pk}/regenerate/ ──────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/webhooks/{pk}/regenerate/",
    tag = "Webhooks",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Webhook ID"),
    ),
    responses((status = 200, description = "Secret regenerado")),
    security(("TokenAuth" = []))
)]
pub async fn regenerate_secret(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<WebhookResponse>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wh = webhooks::Entity::find_by_id(pk)
        .active()
        .filter(webhooks::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: webhooks::ActiveModel = wh.into();
    am.secret_key = Set(generate_secret());
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(WebhookResponse::from_model(updated)))
}

// ── GET /webhook-logs/{webhook_id}/ ──────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/webhook-logs/{webhook_id}/",
    tag = "Webhooks",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
    ),
    responses((status = 200, description = "Logs del webhook")),
    security(("TokenAuth" = []))
)]
pub async fn list_webhook_logs(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, webhook_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<WebhookLogResponse>>, AppError> {
    require_workspace_admin(&guard.member)?;

    // Verify webhook belongs to workspace
    let _ = webhooks::Entity::find_by_id(webhook_id)
        .active()
        .filter(webhooks::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let logs = webhook_logs::Entity::find()
        .filter(webhook_logs::Column::Webhook.eq(webhook_id))
        .order_by_desc(webhook_logs::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        logs.into_iter()
            .map(|l| WebhookLogResponse {
                id: l.id,
                event_type: l.event_type,
                response_status: l.response_status,
                request_headers: l.request_headers,
                request_body: l.request_body,
                response_headers: l.response_headers,
                response_body: l.response_body,
                retry_count: l.retry_count,
                created_at: l.created_at,
            })
            .collect(),
    ))
}
