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
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    auth::extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
    auth::permissions::{require_role, ROLE_GUEST},
    entities::{file_assets, projects, users, workspaces},
    error::AppError,
    utils::s3_presigned_post::{generate_presigned_post, public_s3_endpoint, PresignedPost},
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

/// Tipos MIME permitidos para issue attachments (V2).
///
/// Mirror exacto de `ATTACHMENT_MIME_TYPES` en Django
/// (`apps/api/plane/settings/common.py:371-459`).
/// Se valida en el POST del upload para rechazar tipos no permitidos con
/// HTTP 400 — paridad con `IssueAttachmentV2Endpoint.post`.
const ATTACHMENT_MIME_TYPES: &[&str] = &[
    // Images
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/svg+xml",
    "image/webp",
    "image/tiff",
    "image/bmp",
    // Documents
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "text/plain",
    "text/markdown",
    "application/rtf",
    "application/vnd.oasis.opendocument.spreadsheet",
    "application/vnd.oasis.opendocument.text",
    "application/vnd.oasis.opendocument.presentation",
    "application/vnd.oasis.opendocument.graphics",
    // Microsoft Visio
    "application/vnd.visio",
    // Netpbm format
    "image/x-portable-graymap",
    "image/x-portable-bitmap",
    "image/x-portable-pixmap",
    // Open Office Base
    "application/vnd.oasis.opendocument.database",
    // Audio
    "audio/mpeg",
    "audio/wav",
    "audio/ogg",
    "audio/midi",
    "audio/x-midi",
    "audio/aac",
    "audio/flac",
    "audio/x-m4a",
    // Video
    "video/mp4",
    "video/mpeg",
    "video/ogg",
    "video/webm",
    "video/quicktime",
    "video/x-msvideo",
    "video/x-ms-wmv",
    // Archives
    "application/zip",
    "application/x-rar",
    "application/x-rar-compressed",
    "application/x-tar",
    "application/gzip",
    "application/x-zip",
    "application/x-zip-compressed",
    "application/x-7z-compressed",
    "application/x-compressed",
    "application/x-compressed-tar",
    "application/x-compressed-tar-gz",
    "application/x-compressed-tar-bz2",
    "application/x-compressed-tar-zip",
    "application/x-compressed-tar-7z",
    "application/x-compressed-tar-rar",
    // 3D Models
    "model/gltf-binary",
    "model/gltf+json",
    "application/octet-stream",
    // Fonts
    "font/ttf",
    "font/otf",
    "font/woff",
    "font/woff2",
    // Other
    "text/css",
    "text/javascript",
    "application/json",
    "text/xml",
    "text/csv",
    "application/xml",
    // SQL
    "application/x-sql",
    // Gzip
    "application/x-gzip",
];

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CompleteUploadRequest {
    /// Metadatos adicionales a guardar en `attributes`.
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UploadResponse {
    /// Datos para el POST multipart/form-data al bucket.
    /// El cliente envía los `fields` como campos del form + el `file`.
    /// Espejo de `S3Storage.generate_presigned_post` en Django (boto3).
    pub upload_data: PresignedPost,
    pub asset_id: Uuid,
    pub asset_url: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
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
        deleted_at: Set(None),
    };

    let asset = new_asset.insert(&state.db).await.map_err(AppError::Database)?;

    let upload_data = generate_presigned_post(
        &state.config.aws_s3_bucket,
        &state.config.aws_endpoint,
        &state.config.aws_region,
        &state.config.aws_access_key_id,
        &state.config.aws_secret_access_key,
        &asset_key,
        &content_type,
        size_limit as i64,
        UPLOAD_URL_TTL_SECS as i64,
    )?;

    let asset_url = asset_url_from_key(&asset_key, &state.config.aws_endpoint, &state.config.aws_s3_bucket);

    Ok(Json(UploadResponse {
        upload_data,
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
        deleted_at: Set(None),
    };

    let asset = new_asset.insert(&state.db).await.map_err(AppError::Database)?;

    let upload_data = generate_presigned_post(
        &state.config.aws_s3_bucket,
        &state.config.aws_endpoint,
        &state.config.aws_region,
        &state.config.aws_access_key_id,
        &state.config.aws_secret_access_key,
        &asset_key,
        &content_type,
        size_limit as i64,
        UPLOAD_URL_TTL_SECS as i64,
    )?;

    let asset_url = asset_url_from_key(&asset_key, &state.config.aws_endpoint, &state.config.aws_s3_bucket);

    Ok(Json(UploadResponse {
        upload_data,
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

// ── Issue Attachments (V2) ────────────────────────────────────────────────────
//
// Mirror Django: `IssueAttachmentV2Endpoint` en
// `apps/api/plane/app/views/issue/attachment.py`.
// URL Django: `apps/api/plane/app/urls/issue.py:137-146`.

/// Response shape del `IssueAttachmentSerializer` Django (`fields = "__all__"` + `asset_url`).
///
/// Espejo de `plane/app/serializers/issue.py:IssueAttachmentSerializer`.
/// Campos en orden idéntico al modelo `FileAsset` para paridad con consumidores
/// que iteren por `Object.keys()` en el frontend.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueAttachmentV2Response {
    pub id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub attributes: serde_json::Value,
    pub asset: String,
    pub entity_type: Option<String>,
    pub entity_identifier: Option<String>,
    pub is_deleted: bool,
    pub is_archived: bool,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub size: f64,
    pub is_uploaded: bool,
    pub storage_metadata: Option<serde_json::Value>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub user: Option<Uuid>,
    pub workspace: Option<Uuid>,
    pub project: Option<Uuid>,
    pub issue: Option<Uuid>,
    pub comment: Option<Uuid>,
    pub page: Option<Uuid>,
    pub draft_issue: Option<Uuid>,
    /// Ruta relativa al endpoint de descarga del attachment.
    /// Mirror: `FileAsset.asset_url` property (Django) cuando
    /// `entity_type == ISSUE_ATTACHMENT`:
    /// `/api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{id}/`
    pub asset_url: String,
}

/// Construye el `asset_url` relativo para un `ISSUE_ATTACHMENT`.
///
/// Mirror exacto de `FileAsset.asset_url` property para ISSUE_ATTACHMENT:
/// `apps/api/plane/db/models/asset.py:89-90`.
fn issue_attachment_asset_url(
    slug: &str,
    project_id: Uuid,
    issue_id: Uuid,
    asset_id: Uuid,
) -> String {
    format!(
        "/api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{asset_id}/"
    )
}

fn issue_attachment_to_response(
    a: &file_assets::Model,
    slug: &str,
) -> IssueAttachmentV2Response {
    // Los campos FK con nombre original del modelo (`workspace`, `project`, `issue`, ...)
    // se rellenan con los UUID crudos — mismo shape que DRF default con
    // `fields = "__all__"` sobre un `ForeignKey`.
    let asset_url = match (a.project_id, a.issue_id) {
        (Some(pid), Some(iid)) => issue_attachment_asset_url(slug, pid, iid, a.id),
        // Fallback seguro: si falta algún id (no debería para ISSUE_ATTACHMENT),
        // emitimos la ruta relativa al asset estático para no romper el cliente.
        _ => format!("/api/assets/v2/static/{}/", a.id),
    };

    IssueAttachmentV2Response {
        id: a.id,
        created_at: a.created_at,
        updated_at: a.updated_at,
        attributes: a.attributes.clone(),
        asset: a.asset.clone(),
        entity_type: a.entity_type.clone(),
        entity_identifier: a.entity_identifier.clone(),
        is_deleted: a.is_deleted,
        is_archived: a.is_archived,
        external_id: a.external_id.clone(),
        external_source: a.external_source.clone(),
        size: a.size,
        is_uploaded: a.is_uploaded,
        storage_metadata: a.storage_metadata.clone(),
        deleted_at: a.deleted_at,
        created_by: a.created_by_id,
        updated_by: a.updated_by_id,
        user: a.user_id,
        workspace: a.workspace_id,
        project: a.project_id,
        issue: a.issue_id,
        comment: a.comment_id,
        page: a.page_id,
        draft_issue: a.draft_issue_id,
        asset_url,
    }
}

/// GET /api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/
///
/// Lista los attachments subidos (`is_uploaded = true`) de un issue.
///
/// Mirror exacto de `IssueAttachmentV2Endpoint.get` sin `pk`
/// (`apps/api/plane/app/views/issue/attachment.py:169-200`):
/// - Permisos: ADMIN / MEMBER / GUEST (validado por `ProjectMemberGuard` +
///   `require_role(ROLE_GUEST)`).
/// - Filtro: `issue_id`, `entity_type = 'ISSUE_ATTACHMENT'`, workspace, project,
///   `is_uploaded = true`.
#[utoipa::path(
    get,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 200, description = "Lista de attachments del issue",
            body = Vec<IssueAttachmentV2Response>),
        (status = 403, description = "Sin permisos"),
    )
)]
pub async fn list_issue_attachments_v2(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<Vec<IssueAttachmentV2Response>>, AppError> {
    // `allow_permission([ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST])` en Django.
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    // Importante: filtramos por `project.id` del guard (ya resuelto y validado
    // contra el workspace). No confiamos en el `project_id` del path extractor
    // crudo — `guard.project` garantiza que existe y pertenece al workspace.
    let attachments = file_assets::Entity::find()
        .filter(file_assets::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(file_assets::Column::ProjectId.eq(guard.project.id))
        .filter(file_assets::Column::IssueId.eq(issue_id))
        .filter(file_assets::Column::EntityType.eq(ENTITY_ISSUE_ATTACHMENT))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let response: Vec<IssueAttachmentV2Response> = attachments
        .iter()
        .map(|a| issue_attachment_to_response(a, &slug))
        .collect();

    Ok(Json(response))
}

// ── POST /assets/v2/.../issues/{id}/attachments/ (initiate upload) ────────────

/// Body del POST V2. Paridad con `request.data` de `IssueAttachmentV2Endpoint.post`:
/// `name`, `type`, `size`. Ninguno es opcional en el path exitoso Django
/// (faltar `type` → 400).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InitiateIssueAttachmentV2Request {
    pub name: String,
    /// MIME type. Debe estar en `ATTACHMENT_MIME_TYPES`.
    #[serde(rename = "type")]
    pub content_type: String,
    /// Tamaño en bytes. Se clampa a `DEFAULT_FILE_SIZE_LIMIT`.
    pub size: Option<i64>,
}

/// Response del POST V2. Paridad exacta con Django
/// (`apps/api/plane/app/views/issue/attachment.py:138-145`):
/// `{upload_data, asset_id, attachment, asset_url}`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InitiateIssueAttachmentV2Response {
    pub upload_data: PresignedPost,
    pub asset_id: Uuid,
    pub attachment: IssueAttachmentV2Response,
    pub asset_url: String,
}

/// POST /api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/
///
/// Inicia el flujo de upload de un issue attachment:
/// 1. Crea `FileAsset` con `is_uploaded = false`.
/// 2. Devuelve presigned POST para que el cliente haga el upload directo al bucket.
/// 3. El cliente confirma con PATCH al endpoint de complete (ya existente).
///
/// Mirror de `IssueAttachmentV2Endpoint.post`
/// (`apps/api/plane/app/views/issue/attachment.py:98-146`).
///
/// Permisos: ADMIN / MEMBER / GUEST — mismos que el GET.
#[utoipa::path(
    post,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    request_body = InitiateIssueAttachmentV2Request,
    responses(
        (status = 200, description = "Upload iniciado",
            body = InitiateIssueAttachmentV2Response),
        (status = 400, description = "MIME type no permitido"),
        (status = 403, description = "Sin permisos"),
    )
)]
pub async fn initiate_issue_attachment_upload_v2(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    headers: HeaderMap,
    Json(body): Json<InitiateIssueAttachmentV2Request>,
) -> Result<Json<InitiateIssueAttachmentV2Response>, AppError> {
    // Mirror: `allow_permission([ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST])` Django.
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    // Mirror: `if not type or type not in settings.ATTACHMENT_MIME_TYPES` → 400.
    if body.content_type.is_empty()
        || !ATTACHMENT_MIME_TYPES.contains(&body.content_type.as_str())
    {
        return Err(AppError::BadRequest("Invalid file type.".into()));
    }

    // Mirror Django: `size = int(request.data.get("size", FILE_SIZE_LIMIT))` y luego
    // `size_limit = min(size, FILE_SIZE_LIMIT)`. Aquí rechazamos valores negativos
    // explícitamente — Django no lo hace, pero un size negativo es patología pura
    // y evita que se propague un `i64` negativo a `as f64` más abajo.
    let requested = body.size.unwrap_or(DEFAULT_FILE_SIZE_LIMIT);
    if requested < 0 {
        return Err(AppError::BadRequest("Invalid size.".into()));
    }
    let size_limit = requested.min(DEFAULT_FILE_SIZE_LIMIT);

    // `asset_key = f"{workspace.id}/{uuid4().hex}-{name}"` — paridad byte-a-byte
    // con `attachment.py:114`.
    let ws_id = guard.workspace.id;
    let asset_key = format!("{}/{}-{}", ws_id, Uuid::new_v4().simple(), body.name);

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_asset = file_assets::ActiveModel {
        id: Set(Uuid::new_v4()),
        asset: Set(asset_key.clone()),
        size: Set(size_limit as f64),
        is_uploaded: Set(false),
        is_deleted: Set(false),
        is_archived: Set(false),
        entity_type: Set(Some(ENTITY_ISSUE_ATTACHMENT.to_string())),
        // Importante: usar guard.workspace.id y guard.project.id (ya validados
        // por el guard contra el workspace). No confiar en el path raw para
        // prevenir IDOR cross-workspace.
        workspace_id: Set(Some(ws_id)),
        project_id: Set(Some(guard.project.id)),
        issue_id: Set(Some(issue_id)),
        page_id: Set(None),
        comment_id: Set(None),
        draft_issue_id: Set(None),
        user_id: Set(None),
        entity_identifier: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        attributes: Set(serde_json::json!({
            "name": body.name,
            "type": body.content_type.clone(),
            "size": size_limit,
        })),
        storage_metadata: Set(None),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let asset = new_asset
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Mirror Django S3Storage.__init__ (storage.py:40-58): cuando USE_MINIO=1
    // el presigned debe firmarse contra el dominio público del request, no
    // contra el hostname interno de Docker, para evitar Mixed-Content
    // blocking y fallos de DNS en el navegador.
    let signing_endpoint = public_s3_endpoint(&state.config, &headers);

    let upload_data = generate_presigned_post(
        &state.config.aws_s3_bucket,
        &signing_endpoint,
        &state.config.aws_region,
        &state.config.aws_access_key_id,
        &state.config.aws_secret_access_key,
        &asset_key,
        &body.content_type,
        size_limit,
        UPLOAD_URL_TTL_SECS as i64,
    )?;

    let attachment = issue_attachment_to_response(&asset, &slug);
    let asset_url = attachment.asset_url.clone();

    Ok(Json(InitiateIssueAttachmentV2Response {
        upload_data,
        asset_id: asset.id,
        attachment,
        asset_url,
    }))
}
