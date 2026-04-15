// src/routes/api_tokens.rs
//! Endpoints de gestión de API Tokens.
//!
//! Equivalente Django: `plane/app/views/api.py` → `ApiTokenEndpoint`.
//!
//! Rutas implementadas:
//!   GET    /api/api-tokens/          → lista todos los tokens del usuario
//!   POST   /api/api-tokens/          → crea un token
//!   GET    /api/api-tokens/{pk}/     → detalle de un token
//!   PATCH  /api/api-tokens/{pk}/     → actualiza label/description/expired_at
//!   DELETE /api/api-tokens/{pk}/     → elimina un token

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::api_tokens,
    error::AppError,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Respuesta completa del token (incluye el valor del token — solo en creación).
#[derive(Debug, Serialize)]
pub struct ApiTokenFullResponse {
    pub id: Uuid,
    pub token: String,
    pub label: String,
    pub description: String,
    pub is_active: bool,
    pub is_service: bool,
    pub user_type: i16,
    pub expired_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub workspace_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Respuesta de lectura del token (oculta el valor del token).
#[derive(Debug, Serialize)]
pub struct ApiTokenReadResponse {
    pub id: Uuid,
    pub label: String,
    pub description: String,
    pub is_active: bool,
    pub is_service: bool,
    pub user_type: i16,
    pub expired_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub workspace_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<api_tokens::Model> for ApiTokenFullResponse {
    fn from(m: api_tokens::Model) -> Self {
        Self {
            id: m.id,
            token: m.token,
            label: m.label,
            description: m.description,
            is_active: m.is_active,
            is_service: m.is_service,
            user_type: m.user_type,
            expired_at: m.expired_at.map(Into::into),
            last_used: m.last_used.map(Into::into),
            workspace_id: m.workspace_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

impl From<api_tokens::Model> for ApiTokenReadResponse {
    fn from(m: api_tokens::Model) -> Self {
        Self {
            id: m.id,
            label: m.label,
            description: m.description,
            is_active: m.is_active,
            is_service: m.is_service,
            user_type: m.user_type,
            expired_at: m.expired_at.map(Into::into),
            last_used: m.last_used.map(Into::into),
            workspace_id: m.workspace_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateApiTokenRequest {
    /// Etiqueta descriptiva. Si se omite, se genera un UUID hex.
    pub label: Option<String>,
    pub description: Option<String>,
    pub expired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApiTokenRequest {
    pub label: Option<String>,
    pub description: Option<String>,
    pub expired_at: Option<DateTime<Utc>>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api-tokens/
///
/// Lista todos los tokens no-service del usuario autenticado.
#[utoipa::path(
    get,
    path = "/api/api-tokens/",
    tag = "API Tokens",
    responses(
        (status = 200, description = "Lista de API tokens"),
        (status = 401, description = "No autenticado"),
    )
)]
pub async fn list_api_tokens(
    State(state): State<AppState>,
    AnyAuth(user_id): AnyAuth,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;

    let tokens = api_tokens::Entity::find()
        .filter(api_tokens::Column::UserId.eq(user_id))
        .filter(api_tokens::Column::IsService.eq(false))
        .filter(api_tokens::Column::DeletedAt.is_null())
        .order_by_desc(api_tokens::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<ApiTokenReadResponse> = tokens.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// POST /api-tokens/
///
/// Crea un nuevo API token. El valor del token sólo es visible en esta respuesta.
#[utoipa::path(
    post,
    path = "/api/api-tokens/",
    tag = "API Tokens",
    responses(
        (status = 201, description = "Token creado (incluye el valor del token)"),
        (status = 401, description = "No autenticado"),
    )
)]
pub async fn create_api_token(
    State(state): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Json(body): Json<CreateApiTokenRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;

    // Generar token aleatorio seguro (32 bytes hex = 64 caracteres)
    let raw_token = {
        use std::fmt::Write;
        let bytes: [u8; 32] = rand_token();
        let mut s = String::with_capacity(64);
        for b in bytes {
            write!(s, "{:02x}", b).unwrap();
        }
        s
    };

    let label = body
        .label
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string());

    let new = api_tokens::ActiveModel {
        id: Set(Uuid::new_v4()),
        token: Set(raw_token),
        label: Set(label),
        description: Set(body.description.unwrap_or_default()),
        is_active: Set(true),
        is_service: Set(false),
        user_type: Set(0), // 0 = usuario normal, 1 = bot
        expired_at: Set(body.expired_at.map(Into::into)),
        last_used: Set(None),
        workspace_id: Set(None),
        user_id: Set(user_id),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
        allowed_rate_limit: Set("standard".to_string()),
    };

    let saved = new.insert(db).await.map_err(AppError::Database)?;
    let resp: ApiTokenFullResponse = saved.into();
    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /api-tokens/{pk}/
///
/// Detalle de un token (sin el valor).
#[utoipa::path(
    get,
    path = "/api/api-tokens/{pk}/",
    tag = "API Tokens",
    params(("pk" = Uuid, Path, description = "Token ID")),
    responses(
        (status = 200, description = "Token encontrado"),
        (status = 404, description = "No encontrado"),
    )
)]
pub async fn get_api_token(
    State(state): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(pk): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;

    let token = api_tokens::Entity::find_by_id(pk)
        .filter(api_tokens::Column::UserId.eq(user_id))
        .filter(api_tokens::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let resp: ApiTokenReadResponse = token.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// PATCH /api-tokens/{pk}/
///
/// Actualiza label, description o expired_at de un token.
#[utoipa::path(
    patch,
    path = "/api/api-tokens/{pk}/",
    tag = "API Tokens",
    params(("pk" = Uuid, Path, description = "Token ID")),
    responses(
        (status = 200, description = "Token actualizado"),
        (status = 404, description = "No encontrado"),
    )
)]
pub async fn update_api_token(
    State(state): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(pk): Path<Uuid>,
    Json(body): Json<UpdateApiTokenRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;

    let token = api_tokens::Entity::find_by_id(pk)
        .filter(api_tokens::Column::UserId.eq(user_id))
        .filter(api_tokens::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: api_tokens::ActiveModel = token.into();
    if let Some(label) = body.label {
        if label.is_empty() {
            return Err(AppError::BadRequest("Label cannot be empty".into()));
        }
        active.label = Set(label);
    }
    if let Some(desc) = body.description {
        active.description = Set(desc);
    }
    if let Some(exp) = body.expired_at {
        active.expired_at = Set(Some(exp.into()));
    }
    active.updated_at = Set(chrono::Utc::now().into());
    active.updated_by_id = Set(Some(user_id));

    let saved = active.update(db).await.map_err(AppError::Database)?;
    let resp: ApiTokenReadResponse = saved.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// DELETE /api-tokens/{pk}/
///
/// Elimina (soft-delete) un token no-service del usuario.
#[utoipa::path(
    delete,
    path = "/api/api-tokens/{pk}/",
    tag = "API Tokens",
    params(("pk" = Uuid, Path, description = "Token ID")),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 403, description = "No puedes eliminar tokens de servicio"),
        (status = 404, description = "No encontrado"),
    )
)]
pub async fn delete_api_token(
    State(state): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(pk): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let db = &state.db;

    let token = api_tokens::Entity::find_by_id(pk)
        .filter(api_tokens::Column::UserId.eq(user_id))
        .filter(api_tokens::Column::IsService.eq(false))
        .filter(api_tokens::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: api_tokens::ActiveModel = token.into();
    active.deleted_at = Set(Some(chrono::Utc::now().into()));
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Utilidad interna ──────────────────────────────────────────────────────────

/// Genera 32 bytes aleatorios usando el OS PRNG.
/// No requiere dependencias adicionales — usa `getrandom` vía `uuid`.
fn rand_token() -> [u8; 32] {
    let mut buf = [0u8; 32];
    // Reutilizamos el generador de Uuid que ya existe en el binario
    // (uuid::Uuid::new_v4 usa getrandom internamente).
    for chunk in buf.chunks_mut(16) {
        let bytes = *Uuid::new_v4().as_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
    buf
}
