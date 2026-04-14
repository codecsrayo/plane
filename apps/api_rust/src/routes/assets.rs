// src/routes/assets.rs
//! Endpoints de gestión de assets (imágenes de perfil, logos, covers, adjuntos).
//!
//! Equivalente a `plane/app/views/asset/v2.py` en Django.
//! Usa presigned URLs de S3/MinIO para upload directo desde el cliente.
//!
//! Rutas implementadas:
//!   POST   /api/assets/v2/user-assets/
//!   PATCH  /api/assets/v2/user-assets/{asset_id}/
//!   DELETE /api/assets/v2/user-assets/{asset_id}/
//!
//!   POST   /api/assets/v2/workspaces/{slug}/
//!   GET    /api/assets/v2/workspaces/{slug}/{asset_id}/
//!   PATCH  /api/assets/v2/workspaces/{slug}/{asset_id}/
//!   DELETE /api/assets/v2/workspaces/{slug}/{asset_id}/
//!
//!   GET    /api/assets/v2/static/{asset_id}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    auth::extractors::WorkspaceMemberGuard,
    entities::{file_assets, projects, users, workspaces},
    error::AppError,
    utils::s3::{build_s3_client, presigned_put_url},
    AppState,
};

// ── Constantes ────────────────────────────────────────────────────────────────

/// Tipos MIME permitidos para imágenes.
const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/jpg",
    "image/gif",
];

/// Tamaño máximo por defecto: 5 MB.
const DEFAULT_FILE_SIZE_LIMIT: i64 = 5 * 1024 * 1024;

/// TTL de presigned URL de upload: 1 hora.
const UPLOAD_URL_TTL_SECS: u64 = 3600;

// ── Entity types (espejo de Django FileAsset.EntityTypeContext) ───────────────

const ENTITY_USER_AVATAR: &str = "USER_AVATAR";
const ENTITY_USER_COVER: &str = "USER_COVER";
const ENTITY_WORKSPACE_LOGO: &str = "WORKSPACE_LOGO";
const ENTITY_PROJECT_COVER: &str = "PROJECT_COVER";
const ENTITY_ISSUE_ATTACHMENT: &str = "ISSUE_ATTACHMENT";
const ENTITY_ISSUE_DESCRIPTION: &str = "ISSUE_DESCRIPTION";
const ENTITY_PAGE_DESCRIPTION: &str = "PAGE_DESCRIPTION";
const ENTITY_COMMENT_DESCRIPTION: &str = "COMMENT_DESCRIPTION";

/// Todos los entity types válidos.
const VALID_ENTITY_TYPES: &[&str] = &[
    ENTITY_USER_AVATAR,
    ENTITY_USER_COVER,
    ENTITY_WORKSPACE_LOGO,
    ENTITY_PROJECT_COVER,
    ENTITY_ISSUE_ATTACHMENT,
    ENTITY_ISSUE_DESCRIPTION,
    ENTITY_PAGE_DESCRIPTION,
    ENTITY_COMMENT_DESCRIPTION,
];

/// Entity types válidos para user-assets.
const USER_ENTITY_TYPES: &[&str] = &[ENTITY_USER_AVATAR, ENTITY_USER_COVER];

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct InitiateUploadRequest {
    /// Nombre del archivo original.
    pub name: String,
    /// MIME type. Default: `image/jpeg`.
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    /// Tamaño en bytes. Se limita al máximo del servidor.
    pub size: Option<i64>,
    /// Tipo de entidad (USER_AVATAR, WORKSPACE_LOGO, etc.).
    pub entity_type: String,
    /// ID de la entidad asociada (issue_id, page_id, etc.) cuando aplica.
    pub entity_identifier: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteUploadRequest {
    /// Metadatos adicionales a guardar en `attributes`.
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// URL de subida directa a S3 (presigned PUT).
    pub upload_url: String,
    pub asset_id: Uuid,
    pub asset_url: String,
}

#[derive(Debug, Serialize)]
pub struct AssetResponse {
    pub id: Uuid,
    pub asset: String,
    pub entity_type: Option<String>,
    pub is_uploaded: bool,
    pub is_deleted: bool,
    pub size: f64,
    pub attributes: serde_json::Value,
    pub workspace_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub issue_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn asset_url_from_key(asset_key: &str, endpoint: &str, bucket: &str) -> String {
    // URL pública si el bucket es público; presigned si es privado.
    // En el flujo de Plane la URL se genera on-demand al hacer GET.
    format!("{endpoint}/{bucket}/{asset_key}")
}

fn asset_to_response(a: &file_assets::Model) -> AssetResponse {
    AssetResponse {
        id: a.id,
        asset: a.asset.clone(),
        entity_type: a.entity_type.clone(),
        is_uploaded: a.is_uploaded,
        is_deleted: a.is_deleted,
        size: a.size,
        attributes: a.attributes.clone(),
        workspace_id: a.workspace_id,
        user_id: a.user_id,
        project_id: a.project_id,
        issue_id: a.issue_id,
        created_at: a.created_at,
    }
}

/// Marca un asset como eliminado (soft-delete) sin borrar del bucket.
async fn soft_delete_asset(
    db: &sea_orm::DatabaseConnection,
    asset: file_assets::Model,
) -> Result<(), AppError> {
    let mut am: file_assets::ActiveModel = asset.into();
    am.is_deleted = Set(true);
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(db).await.map_err(AppError::Database)?;
    Ok(())
}

/// Propaga el `asset_id` a la entidad correspondiente tras confirmar el upload.
async fn propagate_asset_to_entity(
    db: &sea_orm::DatabaseConnection,
    asset: &file_assets::Model,
) -> Result<(), AppError> {
    let entity_type = asset.entity_type.as_deref().unwrap_or("");

    match entity_type {
        ENTITY_WORKSPACE_LOGO => {
            if let Some(ws_id) = asset.workspace_id {
                if let Some(ws) = workspaces::Entity::find_by_id(ws_id)
                    .one(db)
                    .await
                    .map_err(AppError::Database)?
                {
                    let mut am: workspaces::ActiveModel = ws.into();
                    am.logo_asset_id = Set(Some(asset.id));
                    am.logo = Set(None);
                    am.update(db).await.map_err(AppError::Database)?;
                }
            }
        }
        ENTITY_PROJECT_COVER => {
            if let Some(proj_id) = asset.project_id {
                if let Some(proj) = projects::Entity::find_by_id(proj_id)
                    .one(db)
                    .await
                    .map_err(AppError::Database)?
                {
                    let mut am: projects::ActiveModel = proj.into();
                    am.cover_image_asset_id = Set(Some(asset.id));
                    am.cover_image = Set(None);
                    am.update(db).await.map_err(AppError::Database)?;
                }
            }
        }
        ENTITY_USER_AVATAR => {
            if let Some(user_id) = asset.user_id {
                if let Some(user) = users::Entity::find_by_id(user_id)
                    .one(db)
                    .await
                    .map_err(AppError::Database)?
                {
                    let mut am: users::ActiveModel = user.into();
                    am.avatar_asset_id = Set(Some(asset.id));
                    am.avatar = Set(String::new());
                    am.update(db).await.map_err(AppError::Database)?;
                }
            }
        }
        ENTITY_USER_COVER => {
            if let Some(user_id) = asset.user_id {
                if let Some(user) = users::Entity::find_by_id(user_id)
                    .one(db)
                    .await
                    .map_err(AppError::Database)?
                {
                    let mut am: users::ActiveModel = user.into();
                    am.cover_image_asset_id = Set(Some(asset.id));
                    am.cover_image = Set(None);
                    am.update(db).await.map_err(AppError::Database)?;
                }
            }
        }
        // ISSUE_ATTACHMENT, ISSUE_DESCRIPTION, PAGE_DESCRIPTION, COMMENT_DESCRIPTION
        // se resuelven por issue_id / page_id / comment_id ya almacenados en el asset.
        _ => {}
    }

    Ok(())
}

// ── User Asset Endpoints ──────────────────────────────────────────────────────

/// POST /api/assets/v2/user-assets/
///
/// Inicia el upload de un asset de usuario (avatar o cover).
/// Devuelve una presigned URL de PUT para subir directamente a S3.
#[utoipa::path(
    post,
    path = "/assets/v2/user-assets/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Upload initiated — presigned URL returned"),
        (status = 400, description = "Invalid entity type or file type"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn initiate_user_asset_upload(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<InitiateUploadRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !USER_ENTITY_TYPES.contains(&body.entity_type.as_str()) {
        return Err(AppError::BadRequest(
            "entity_type must be USER_AVATAR or USER_COVER".into(),
        ));
    }

    let content_type = body.content_type.unwrap_or_else(|| "image/jpeg".to_string());
    if !ALLOWED_IMAGE_TYPES.contains(&content_type.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid file type. Only JPEG, PNG, WebP, JPG and GIF files are allowed.".into(),
        ));
    }

    let size = body.size.unwrap_or(DEFAULT_FILE_SIZE_LIMIT);
    let size_limit = size.min(DEFAULT_FILE_SIZE_LIMIT) as f64;
    let asset_key = format!("{}/{}-{}", user.id, Uuid::new_v4().simple(), body.name);

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_asset = file_assets::ActiveModel {
        id: Set(Uuid::new_v4()),
        asset: Set(asset_key.clone()),
        size: Set(size_limit),
        is_uploaded: Set(false),
        is_deleted: Set(false),
        is_archived: Set(false),
        entity_type: Set(Some(body.entity_type)),
        user_id: Set(Some(user.id)),
        workspace_id: Set(None),
        project_id: Set(None),
        issue_id: Set(None),
        page_id: Set(None),
        comment_id: Set(None),
        draft_issue_id: Set(None),
        entity_identifier: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        attributes: Set(serde_json::json!({
            "name": body.name,
            "type": content_type.clone(),
            "size": size_limit,
        })),
        storage_metadata: Set(None),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let asset = new_asset.insert(&state.db).await.map_err(AppError::Database)?;

    let s3 = build_s3_client(&state.config).await;
    let upload_url = presigned_put_url(
        &s3,
        &state.config.aws_s3_bucket,
        &asset_key,
        &content_type,
        UPLOAD_URL_TTL_SECS,
    )
    .await?;

    let asset_url = asset_url_from_key(&asset_key, &state.config.aws_endpoint, &state.config.aws_s3_bucket);

    Ok(Json(UploadResponse {
        upload_url,
        asset_id: asset.id,
        asset_url,
    }))
}

/// PATCH /api/assets/v2/user-assets/{asset_id}/
///
/// Confirma que el upload a S3 fue exitoso y propaga el asset_id a la entidad.
#[utoipa::path(
    patch,
    path = "/assets/v2/user-assets/{asset_id}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(("asset_id" = Uuid, Path, description = "Asset ID")),
    responses(
        (status = 204, description = "Upload confirmed"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn complete_user_asset_upload(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<CompleteUploadRequest>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find_by_id(asset_id)
        .filter(file_assets::Column::UserId.eq(user.id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: file_assets::ActiveModel = asset.clone().into();
    am.is_uploaded = Set(true);
    am.updated_at = Set(Utc::now().into());
    if let Some(attrs) = body.attributes {
        am.attributes = Set(attrs);
    }
    am.update(&state.db).await.map_err(AppError::Database)?;

    propagate_asset_to_entity(&state.db, &asset).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/assets/v2/user-assets/{asset_id}/
#[utoipa::path(
    delete,
    path = "/assets/v2/user-assets/{asset_id}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(("asset_id" = Uuid, Path, description = "Asset ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_user_asset(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find_by_id(asset_id)
        .filter(file_assets::Column::UserId.eq(user.id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    soft_delete_asset(&state.db, asset).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Workspace Asset Endpoints ─────────────────────────────────────────────────

/// POST /api/assets/v2/workspaces/{slug}/
///
/// Inicia el upload de un asset de workspace (logo, project cover, issue attachment, etc.).
#[utoipa::path(
    post,
    path = "/assets/v2/workspaces/{slug}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Upload initiated"),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn initiate_workspace_asset_upload(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<InitiateUploadRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !VALID_ENTITY_TYPES.contains(&body.entity_type.as_str()) {
        return Err(AppError::BadRequest("Invalid entity_type.".into()));
    }

    let content_type = body.content_type.unwrap_or_else(|| "image/jpeg".to_string());
    if !ALLOWED_IMAGE_TYPES.contains(&content_type.as_str()) {
        return Err(AppError::BadRequest(
            "Invalid file type. Only JPEG, PNG, WebP, JPG and GIF files are allowed.".into(),
        ));
    }

    let size = body.size.unwrap_or(DEFAULT_FILE_SIZE_LIMIT);
    let size_limit = size.min(DEFAULT_FILE_SIZE_LIMIT) as f64;
    let ws_id = guard.workspace.id;
    let asset_key = format!("{}/{}-{}", ws_id, Uuid::new_v4().simple(), body.name);

    // Determinar los campos de entidad según entity_type
    let (issue_id, project_id, page_id, comment_id, entity_identifier) =
        match body.entity_type.as_str() {
            ENTITY_ISSUE_ATTACHMENT | ENTITY_ISSUE_DESCRIPTION => (
                body.entity_identifier,
                None,
                None,
                None,
                body.entity_identifier.map(|u| u.to_string()),
            ),
            ENTITY_PROJECT_COVER => (
                None,
                body.entity_identifier,
                None,
                None,
                body.entity_identifier.map(|u| u.to_string()),
            ),
            ENTITY_PAGE_DESCRIPTION => (
                None,
                None,
                body.entity_identifier,
                None,
                body.entity_identifier.map(|u| u.to_string()),
            ),
            ENTITY_COMMENT_DESCRIPTION => (
                None,
                None,
                None,
                body.entity_identifier,
                body.entity_identifier.map(|u| u.to_string()),
            ),
            _ => (None, None, None, None, None),
        };

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_asset = file_assets::ActiveModel {
        id: Set(Uuid::new_v4()),
        asset: Set(asset_key.clone()),
        size: Set(size_limit),
        is_uploaded: Set(false),
        is_deleted: Set(false),
        is_archived: Set(false),
        entity_type: Set(Some(body.entity_type)),
        workspace_id: Set(Some(ws_id)),
        user_id: Set(None),
        project_id: Set(project_id),
        issue_id: Set(issue_id),
        page_id: Set(page_id),
        comment_id: Set(comment_id),
        draft_issue_id: Set(None),
        entity_identifier: Set(entity_identifier),
        external_id: Set(None),
        external_source: Set(None),
        attributes: Set(serde_json::json!({
            "name": body.name,
            "type": content_type.clone(),
            "size": size_limit,
        })),
        storage_metadata: Set(None),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let asset = new_asset.insert(&state.db).await.map_err(AppError::Database)?;

    let s3 = build_s3_client(&state.config).await;
    let upload_url = presigned_put_url(
        &s3,
        &state.config.aws_s3_bucket,
        &asset_key,
        &content_type,
        UPLOAD_URL_TTL_SECS,
    )
    .await?;

    let asset_url = asset_url_from_key(&asset_key, &state.config.aws_endpoint, &state.config.aws_s3_bucket);

    Ok(Json(UploadResponse {
        upload_url,
        asset_id: asset.id,
        asset_url,
    }))
}

/// PATCH /api/assets/v2/workspaces/{slug}/{asset_id}/
///
/// Confirma el upload y propaga el asset_id a la entidad correspondiente.
#[utoipa::path(
    patch,
    path = "/assets/v2/workspaces/{slug}/{asset_id}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 204, description = "Confirmed"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn complete_workspace_asset_upload(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, asset_id)): Path<(String, Uuid)>,
    Json(body): Json<CompleteUploadRequest>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find_by_id(asset_id)
        .filter(file_assets::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: file_assets::ActiveModel = asset.clone().into();
    am.is_uploaded = Set(true);
    am.updated_at = Set(Utc::now().into());
    if let Some(attrs) = body.attributes {
        am.attributes = Set(attrs);
    }
    am.update(&state.db).await.map_err(AppError::Database)?;

    propagate_asset_to_entity(&state.db, &asset).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/assets/v2/workspaces/{slug}/{asset_id}/
#[utoipa::path(
    delete,
    path = "/assets/v2/workspaces/{slug}/{asset_id}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_workspace_asset(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, asset_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find_by_id(asset_id)
        .filter(file_assets::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    soft_delete_asset(&state.db, asset).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/assets/v2/workspaces/{slug}/{asset_id}/
///
/// Devuelve metadata del asset. La URL de descarga se genera on-demand en el cliente.
#[utoipa::path(
    get,
    path = "/assets/v2/workspaces/{slug}/{asset_id}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 200, description = "Asset metadata"),
        (status = 404, description = "Not found or not uploaded"),
    )
)]
pub async fn get_workspace_asset(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, asset_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find_by_id(asset_id)
        .filter(file_assets::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if !asset.is_uploaded {
        return Err(AppError::NotFound);
    }

    Ok(Json(asset_to_response(&asset)))
}

// ── Static Asset Endpoint ─────────────────────────────────────────────────────

/// GET /api/assets/v2/static/{asset_id}/
///
/// Devuelve metadata de un asset estático (sin requerir pertenencia a workspace).
/// El cliente usa la `asset` key para construir la URL de descarga.
#[utoipa::path(
    get,
    path = "/assets/v2/static/{asset_id}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(("asset_id" = Uuid, Path, description = "Asset ID")),
    responses(
        (status = 200, description = "Asset metadata"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_static_asset(
    AnyAuth(_user): AnyAuth,
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find_by_id(asset_id)
        .filter(file_assets::Column::IsDeleted.eq(false))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(asset_to_response(&asset)))
}
