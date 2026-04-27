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
    response::{IntoResponse, Redirect},
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    auth::extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
    auth::permissions::{require_role, ROLE_ADMIN, ROLE_GUEST},
    entities::{file_assets, projects, users, workspaces},
    error::AppError,
    utils::{
        s3::{build_s3_client, build_s3_presign_client, copy_object, presigned_get_url},
        s3_presigned_post::{generate_presigned_post, public_s3_endpoint, PresignedPost},
    },
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
///
/// **Paridad Django (`StaticFileAssetEndpoint`, `asset/v2.py:432`)**: este
/// endpoint es público (`permission_classes = [AllowAny]`) porque lo consumen
/// páginas sin sesión (ej. avatares en sign-in, logos de workspace en landing).
/// Para evitar filtrar metadata de assets privados vía ID conocido, sólo se
/// sirven entity_types que ya son públicos por naturaleza: USER_AVATAR,
/// USER_COVER, WORKSPACE_LOGO, PROJECT_COVER (ver Django línea 449-459).
#[utoipa::path(
    get,
    path = "/assets/v2/static/{asset_id}/",
    tag = "Assets",
    params(("asset_id" = Uuid, Path, description = "Asset ID")),
    responses(
        (status = 200, description = "Asset metadata"),
        (status = 400, description = "Invalid entity type"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_static_asset(
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

    // Whitelist: sólo permitir entity_types públicos. Evita que un ID filtrado
    // de un ISSUE_ATTACHMENT/PAGE_DESCRIPTION exponga metadata vía endpoint
    // público. Paridad exacta con Django `asset/v2.py:449-459`.
    const PUBLIC_ENTITY_TYPES: &[&str] = &[
        ENTITY_USER_AVATAR,
        ENTITY_USER_COVER,
        ENTITY_WORKSPACE_LOGO,
        ENTITY_PROJECT_COVER,
    ];
    let is_public = asset
        .entity_type
        .as_deref()
        .map(|et| PUBLIC_ENTITY_TYPES.contains(&et))
        .unwrap_or(false);
    if !is_public {
        return Err(AppError::BadRequest("Invalid entity type.".into()));
    }

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

// ── PATCH /assets/v2/.../issues/{id}/attachments/{pk}/ (complete upload) ──────

/// PATCH /api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}/
///
/// Confirma que el upload al bucket completó. Marca `is_uploaded = true` y fija
/// `created_by` al usuario autenticado la primera vez que se confirma.
///
/// Mirror exacto de `IssueAttachmentV2Endpoint.patch`
/// (`apps/api/plane/app/views/issue/attachment.py:202-229`):
/// - Idempotente: si ya estaba `is_uploaded = true`, no vuelve a disparar la
///   activity (que aquí se omite por ser una task async Django; se puede
///   migrar después sin cambiar el contrato).
/// - Retorna 204.
#[utoipa::path(
    patch,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
        ("pk" = Uuid, Path, description = "FileAsset ID"),
    ),
    responses(
        (status = 204, description = "Upload confirmado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "Asset no encontrado"),
    )
)]
pub async fn complete_issue_attachment_upload_v2(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Mirror: `allow_permission([ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST])`.
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    // Fetch con filtros de seguridad: workspace + project + issue del guard.
    // Django solo filtra por slug + project_id — aquí además confirmamos que
    // el asset pertenece al issue del path para prevenir confirmar uploads
    // de otros issues con un pk cualquiera.
    let asset = file_assets::Entity::find_by_id(pk)
        .filter(file_assets::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(file_assets::Column::ProjectId.eq(guard.project.id))
        .filter(file_assets::Column::IssueId.eq(issue_id))
        .filter(file_assets::Column::EntityType.eq(ENTITY_ISSUE_ATTACHMENT))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Mirror Django: solo si no estaba uploaded antes, marca is_uploaded=true
    // y setea created_by al usuario actual (primer confirmador = "dueño"
    // del attachment para efectos de permisos de borrado posterior).
    if !asset.is_uploaded {
        let mut am: file_assets::ActiveModel = asset.into();
        am.is_uploaded = Set(true);
        am.created_by_id = Set(Some(guard.user.id));
        am.updated_by_id = Set(Some(guard.user.id));
        am.updated_at = Set(Utc::now().into());
        am.update(&state.db).await.map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── DELETE /assets/v2/.../issues/{id}/attachments/{pk}/ ───────────────────────

/// DELETE /api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}/
///
/// Soft-delete de un attachment: `is_deleted = true` + `deleted_at = now()`.
/// No borra del bucket (mirror Django V2 que también hace soft-delete).
///
/// Permisos: `@allow_permission([ROLE.ADMIN], creator=True, model=FileAsset)`
/// (`apps/api/plane/app/views/issue/attachment.py:148`). En Rust esto se
/// traduce a: ADMIN **o** creador del asset.
#[utoipa::path(
    delete,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}/",
    tag = "Assets",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
        ("pk" = Uuid, Path, description = "FileAsset ID"),
    ),
    responses(
        (status = 204, description = "Attachment eliminado"),
        (status = 403, description = "Solo ADMIN o creador del asset"),
        (status = 404, description = "Asset no encontrado"),
    )
)]
pub async fn delete_issue_attachment_v2(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let asset = file_assets::Entity::find_by_id(pk)
        .filter(file_assets::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(file_assets::Column::ProjectId.eq(guard.project.id))
        .filter(file_assets::Column::IssueId.eq(issue_id))
        .filter(file_assets::Column::EntityType.eq(ENTITY_ISSUE_ATTACHMENT))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Mirror `@allow_permission([ROLE.ADMIN], creator=True, model=FileAsset)`:
    // ADMIN de project/workspace O creator del asset.
    let is_admin = guard.project_member.role >= ROLE_ADMIN
        || guard.workspace_member.role >= ROLE_ADMIN;
    let is_creator = asset.created_by_id == Some(guard.user.id);
    if !is_admin && !is_creator {
        return Err(AppError::Forbidden);
    }

    soft_delete_asset(&state.db, asset).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── DTOs adicionales ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DuplicateAssetRequest {
    pub project_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
    pub entity_type: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DuplicateAssetResponse {
    pub asset_id: Uuid,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AssetCheckResponse {
    pub exists: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkAssetRequest {
    pub asset_ids: Vec<Uuid>,
}

// ── AssetRestoreEndpoint ──────────────────────────────────────────────────────

/// Restaura un asset previamente eliminado (soft-delete).
///
/// `POST /assets/v2/workspaces/{slug}/restore/{asset_id}`
///
/// Mirror de `AssetRestoreEndpoint.post` en Django.
#[utoipa::path(
    post,
    path = "/assets/v2/workspaces/{slug}/restore/{asset_id}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Asset ID to restore"),
    ),
    responses(
        (status = 204, description = "Asset restored"),
        (status = 404, description = "Asset not found"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn restore_workspace_asset(
    guard: WorkspaceMemberGuard,
    State(state): State<AppState>,
    Path((slug, asset_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let _ = (guard, &slug);

    // Buscar incluyendo soft-deleted (all_objects en Django)
    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(asset_id))
        .filter(file_assets::Column::WorkspaceId.is_not_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: file_assets::ActiveModel = asset.into();
    active.is_deleted = Set(false);
    active.deleted_at = Set(None);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── ProjectAssetEndpoint ──────────────────────────────────────────────────────

/// Inicia un upload de asset a nivel proyecto (cover, issue description, etc.)
///
/// `POST /assets/v2/workspaces/{slug}/projects/{project_id}`
///
/// Mirror de `ProjectAssetEndpoint.post` en Django.
#[utoipa::path(
    post,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    request_body = InitiateUploadRequest,
    responses(
        (status = 200, description = "Upload URL generated", body = UploadResponse),
        (status = 400, description = "Invalid entity type or file type"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn initiate_project_asset_upload(
    guard: ProjectMemberGuard,
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
    headers: HeaderMap,
    Json(body): Json<InitiateUploadRequest>,
) -> Result<Json<UploadResponse>, AppError> {
    let _ = &slug;

    if !VALID_ENTITY_TYPES.contains(&body.entity_type.as_str()) {
        return Err(AppError::BadRequest("Invalid entity type".into()));
    }

    let content_type = body.content_type.as_deref().unwrap_or("image/jpeg");
    if !ALLOWED_IMAGE_TYPES.contains(&content_type) {
        return Err(AppError::BadRequest(
            "Invalid file type. Only JPEG, PNG, WebP, JPG and GIF files are allowed.".into(),
        ));
    }

    let size = body.size.unwrap_or(DEFAULT_FILE_SIZE_LIMIT);
    let size_limit = size.min(DEFAULT_FILE_SIZE_LIMIT);

    // Obtener workspace
    let workspace = workspaces::Entity::find()
        .filter(workspaces::Column::Slug.eq(&slug))
        .filter(workspaces::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let asset_key = format!("{}/{}-{}", workspace.id, uuid::Uuid::new_v4().simple(), body.name);

    // Campos de FK según entity_type + entity_identifier
    let entity_id = body.entity_identifier;
    // NOT NULL en BD: created_at, updated_at, is_archived. SeaORM no infiere
    // defaults a partir del schema cuando la columna no tiene `DEFAULT`, por
    // lo que `..Default::default()` los deja como `NotSet` y la INSERT falla
    // con "null value in column \"created_at\" ... violates not-null
    // constraint". Paridad con `initiate_workspace_asset_upload`.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let mut new_asset = file_assets::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        asset: Set(asset_key.clone()),
        size: Set(size_limit as f64),
        attributes: Set(serde_json::json!({
            "name": body.name,
            "type": content_type,
            "size": size_limit,
        })),
        entity_type: Set(Some(body.entity_type.clone())),
        workspace_id: Set(Some(workspace.id)),
        project_id: Set(Some(project_id)),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        is_uploaded: Set(false),
        is_deleted: Set(false),
        is_archived: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    // Asignar FK de entidad según tipo
    match body.entity_type.as_str() {
        ENTITY_ISSUE_DESCRIPTION | ENTITY_ISSUE_ATTACHMENT => {
            new_asset.issue_id = Set(entity_id);
        }
        ENTITY_PAGE_DESCRIPTION => {
            new_asset.page_id = Set(entity_id);
        }
        ENTITY_COMMENT_DESCRIPTION => {
            new_asset.comment_id = Set(entity_id);
        }
        _ => {}
    }

    let asset = new_asset.insert(&state.db).await.map_err(AppError::Database)?;

    let _endpoint = public_s3_endpoint(&state.config, &headers);
    let presigned = generate_presigned_post(
        &state.config.aws_s3_bucket,
        &state.config.aws_endpoint,
        &state.config.aws_region,
        &state.config.aws_access_key_id,
        &state.config.aws_secret_access_key,
        &asset_key,
        content_type,
        size_limit,
        UPLOAD_URL_TTL_SECS as i64,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("presigned post error: {e}")))?;

    let asset_url = asset_url_from_key(&asset_key, &state.config.aws_endpoint, &state.config.aws_s3_bucket);

    Ok(Json(UploadResponse {
        upload_data: presigned,
        asset_id: asset.id,
        asset_url,
    }))
}

/// Marca un asset de proyecto como subido (complete upload).
///
/// `PATCH /assets/v2/workspaces/{slug}/projects/{project_id}/{pk}`
///
/// Mirror de `ProjectAssetEndpoint.patch` en Django.
#[utoipa::path(
    patch,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Asset ID"),
    ),
    request_body = CompleteUploadRequest,
    responses(
        (status = 204, description = "Asset marked as uploaded"),
        (status = 404, description = "Asset not found"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn complete_project_asset_upload(
    _guard: ProjectMemberGuard,
    State(state): State<AppState>,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CompleteUploadRequest>,
) -> Result<StatusCode, AppError> {
    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(pk))
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: file_assets::ActiveModel = asset.into();
    active.is_uploaded = Set(true);
    if let Some(attrs) = body.attributes {
        active.attributes = Set(attrs);
    }
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Elimina (soft-delete) un asset de proyecto.
///
/// `DELETE /assets/v2/workspaces/{slug}/projects/{project_id}/{pk}`
///
/// Mirror de `ProjectAssetEndpoint.delete` en Django.
#[utoipa::path(
    delete,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 204, description = "Asset deleted"),
        (status = 404, description = "Asset not found"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn delete_project_asset(
    _guard: ProjectMemberGuard,
    State(state): State<AppState>,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(pk))
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    soft_delete_asset(&state.db, asset).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Obtiene un asset de proyecto y redirige a la URL firmada.
///
/// `GET /assets/v2/workspaces/{slug}/projects/{project_id}/{pk}`
///
/// Mirror de `ProjectAssetEndpoint.get` en Django.
#[utoipa::path(
    get,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 302, description = "Redirect to signed URL"),
        (status = 404, description = "Asset not found or not uploaded"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn get_project_asset(
    _guard: ProjectMemberGuard,
    State(state): State<AppState>,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(pk))
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let s3 = build_s3_presign_client(&state.config);
    let url = presigned_get_url(&s3, &state.config.aws_s3_bucket, &asset.asset, 3600).await?;
    Ok(Redirect::temporary(&url))
}

// ── ProjectBulkAssetEndpoint ──────────────────────────────────────────────────

/// Asigna en bulk assets a una entidad (issue, page, comment, draft, project cover).
///
/// `POST /assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk`
///
/// Mirror de `ProjectBulkAssetEndpoint.post` en Django.
#[utoipa::path(
    post,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("entity_id" = Uuid, Path, description = "Entity ID (issue, page, comment, etc.)"),
    ),
    request_body = BulkAssetRequest,
    responses(
        (status = 204, description = "Assets updated"),
        (status = 400, description = "No asset IDs provided"),
        (status = 404, description = "No assets found"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn bulk_project_assets(
    _guard: ProjectMemberGuard,
    State(state): State<AppState>,
    Path((_slug, project_id, entity_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<BulkAssetRequest>,
) -> Result<StatusCode, AppError> {
    if body.asset_ids.is_empty() {
        return Err(AppError::BadRequest("No asset ids provided.".into()));
    }

    // Obtener el primer asset para inferir entity_type
    let first_asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let entity_type = first_asset.entity_type.as_deref().unwrap_or("");

    // Actualizar todos los assets del batch
    use sea_orm::sea_query::Expr;
    match entity_type {
        ENTITY_PROJECT_COVER => {
            // Asignar project_id y actualizar cover del proyecto
            file_assets::Entity::update_many()
                .col_expr(file_assets::Column::ProjectId, Expr::value(project_id))
                .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
                .exec(&state.db)
                .await
                .map_err(AppError::Database)?;

            // Actualizar cover del proyecto
            if let Some(project) = projects::Entity::find_by_id(project_id)
                .one(&state.db)
                .await
                .map_err(AppError::Database)?
            {
                let mut p: projects::ActiveModel = project.into();
                p.cover_image_asset_id = Set(body.asset_ids.first().copied());
                p.update(&state.db).await.map_err(AppError::Database)?;
            }
        }
        ENTITY_ISSUE_DESCRIPTION => {
            let _ = file_assets::Entity::update_many()
                .col_expr(file_assets::Column::IssueId, Expr::value(entity_id))
                .col_expr(file_assets::Column::ProjectId, Expr::value(project_id))
                .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
                .exec(&state.db)
                .await; // ignore IntegrityError (issue deleted)
        }
        ENTITY_COMMENT_DESCRIPTION => {
            let _ = file_assets::Entity::update_many()
                .col_expr(file_assets::Column::CommentId, Expr::value(entity_id))
                .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
                .exec(&state.db)
                .await; // ignore IntegrityError
        }
        ENTITY_PAGE_DESCRIPTION => {
            file_assets::Entity::update_many()
                .col_expr(file_assets::Column::PageId, Expr::value(entity_id))
                .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
                .exec(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
        _ => {}
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── AssetCheckEndpoint ────────────────────────────────────────────────────────

/// Verifica si un asset existe en el workspace (incluyendo eliminados no expirados).
///
/// `GET /assets/v2/workspaces/{slug}/check/{asset_id}`
///
/// Mirror de `AssetCheckEndpoint.get` en Django.
#[utoipa::path(
    get,
    path = "/assets/v2/workspaces/{slug}/check/{asset_id}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Asset ID to check"),
    ),
    responses(
        (status = 200, description = "Check result", body = AssetCheckResponse),
    ),
    security(("sessionAuth" = []))
)]
pub async fn check_workspace_asset(
    guard: WorkspaceMemberGuard,
    State(state): State<AppState>,
    Path((_slug, asset_id)): Path<(String, Uuid)>,
) -> Result<Json<AssetCheckResponse>, AppError> {
    let _ = guard;
    let exists = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(asset_id))
        .filter(file_assets::Column::WorkspaceId.is_not_null())
        .filter(file_assets::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    Ok(Json(AssetCheckResponse { exists }))
}

// ── DuplicateAssetEndpoint ────────────────────────────────────────────────────

/// Duplica un asset existente (copia en S3 + nuevo registro en DB).
///
/// `POST /assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}`
///
/// Mirror de `DuplicateAssetEndpoint.post` en Django.
#[utoipa::path(
    post,
    path = "/assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Source asset ID to duplicate"),
    ),
    request_body = DuplicateAssetRequest,
    responses(
        (status = 200, description = "Asset duplicated", body = DuplicateAssetResponse),
        (status = 400, description = "Invalid entity type"),
        (status = 404, description = "Asset or project not found"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn duplicate_workspace_asset(
    guard: WorkspaceMemberGuard,
    State(state): State<AppState>,
    Path((slug, asset_id)): Path<(String, Uuid)>,
    Json(body): Json<DuplicateAssetRequest>,
) -> Result<Json<DuplicateAssetResponse>, AppError> {
    if !VALID_ENTITY_TYPES.contains(&body.entity_type.as_str()) {
        return Err(AppError::BadRequest("Invalid entity type or entity id".into()));
    }

    let workspace = workspaces::Entity::find()
        .filter(workspaces::Column::Slug.eq(&slug))
        .filter(workspaces::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verificar que el proyecto existe si se especificó
    if let Some(pid) = body.project_id {
        let project_exists = projects::Entity::find()
            .filter(projects::Column::Id.eq(pid))
            .filter(projects::Column::WorkspaceId.eq(workspace.id))
            .filter(projects::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .is_some();
        if !project_exists {
            return Err(AppError::NotFound);
        }
    }

    // Obtener el asset original (solo uploaded)
    let original = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(asset_id))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let orig_name = original.attributes
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("file");

    let dest_key = format!(
        "{}/{}-{}",
        workspace.id,
        uuid::Uuid::new_v4().simple(),
        orig_name,
    );

    // Copiar en S3
    let s3 = build_s3_client(&state.config);
    copy_object(&s3, &state.config.aws_s3_bucket, &original.asset, &dest_key).await?;

    // Crear nuevo registro en DB.
    // NOT NULL en BD: created_at, updated_at, is_archived. `..Default::default()`
    // los deja como `NotSet` -> INSERT falla con NOT NULL constraint.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let mut new_asset = file_assets::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        asset: Set(dest_key.clone()),
        size: Set(original.size),
        attributes: Set(original.attributes.clone()),
        entity_type: Set(Some(body.entity_type.clone())),
        workspace_id: Set(Some(workspace.id)),
        project_id: Set(body.project_id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        is_uploaded: Set(true),
        is_deleted: Set(false),
        is_archived: Set(false),
        storage_metadata: Set(original.storage_metadata.clone()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    // Asignar FK según entity_type
    let entity_id = body.entity_id;
    match body.entity_type.as_str() {
        ENTITY_WORKSPACE_LOGO => {
            new_asset.workspace_id = Set(entity_id.or(Some(workspace.id)));
        }
        ENTITY_PROJECT_COVER => {
            new_asset.project_id = Set(entity_id);
        }
        ENTITY_USER_AVATAR | ENTITY_USER_COVER => {
            new_asset.user_id = Set(entity_id);
        }
        ENTITY_ISSUE_ATTACHMENT | ENTITY_ISSUE_DESCRIPTION => {
            new_asset.issue_id = Set(entity_id);
        }
        ENTITY_PAGE_DESCRIPTION => {
            new_asset.page_id = Set(entity_id);
        }
        ENTITY_COMMENT_DESCRIPTION => {
            new_asset.comment_id = Set(entity_id);
        }
        _ => {}
    }

    let created = new_asset.insert(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(DuplicateAssetResponse { asset_id: created.id }))
}

// ── Download endpoints ────────────────────────────────────────────────────────

/// Genera URL de descarga (attachment) para un asset de workspace y redirige.
///
/// `GET /assets/v2/workspaces/{slug}/download/{asset_id}`
///
/// Mirror de `WorkspaceAssetDownloadEndpoint.get` en Django.
#[utoipa::path(
    get,
    path = "/assets/v2/workspaces/{slug}/download/{asset_id}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 302, description = "Redirect to download URL"),
        (status = 404, description = "Asset not found or not uploaded"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn download_workspace_asset(
    guard: WorkspaceMemberGuard,
    State(state): State<AppState>,
    Path((_slug, asset_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = guard;
    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(asset_id))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let filename = asset
        .attributes
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("download");

    let s3 = build_s3_presign_client(&state.config);
    // Presigned GET con content-disposition attachment
    let url = presigned_get_download_url(&s3, &state.config.aws_s3_bucket, &asset.asset, filename, 3600).await?;
    Ok(Redirect::temporary(&url))
}

/// Genera URL de descarga (attachment) para un asset de proyecto y redirige.
///
/// `GET /assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}`
///
/// Mirror de `ProjectAssetDownloadEndpoint.get` en Django.
#[utoipa::path(
    get,
    path = "/assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}",
    tag = "Assets",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("asset_id" = Uuid, Path, description = "Asset ID"),
    ),
    responses(
        (status = 302, description = "Redirect to download URL"),
        (status = 404, description = "Asset not found or not uploaded"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn download_project_asset(
    _guard: ProjectMemberGuard,
    State(state): State<AppState>,
    Path((_slug, project_id, asset_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.eq(asset_id))
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let filename = asset
        .attributes
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("download");

    let s3 = build_s3_presign_client(&state.config);
    let url = presigned_get_download_url(&s3, &state.config.aws_s3_bucket, &asset.asset, filename, 3600).await?;
    Ok(Redirect::temporary(&url))
}

/// Helper: genera presigned GET URL con Content-Disposition: attachment.
async fn presigned_get_download_url(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    filename: &str,
    ttl_secs: u64,
) -> Result<String, AppError> {
    use aws_sdk_s3::presigning::PresigningConfig;
    use std::time::Duration;

    let config = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("presigning config: {e}")))?;

    let safe_filename = filename.replace('"', "\\\"");
    let req = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .response_content_disposition(format!("attachment; filename=\"{safe_filename}\""))
        .presigned(config)
        .await
        .map_err(|e| {
            tracing::error!("presigned_get_download_url error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to generate download URL"))
        })?;

    Ok(req.uri().to_string())
}

// ── Workspace-level bulk asset update ─────────────────────────────────────────
/// POST /assets/v2/workspaces/{slug}/{entity_id}/bulk
///
/// Equivalente workspace del bulk_project_assets — usado cuando el frontend
/// llama desde contexto de workspace (sin project_id, e.g. workspace cover).
pub async fn bulk_workspace_assets(
    guard: WorkspaceMemberGuard,
    State(state): State<AppState>,
    Path((_slug, entity_id)): Path<(String, Uuid)>,
    Json(body): Json<BulkAssetRequest>,
) -> Result<StatusCode, AppError> {
    let _ = guard;
    if body.asset_ids.is_empty() {
        return Err(AppError::BadRequest("No asset ids provided.".into()));
    }

    let first_asset = file_assets::Entity::find()
        .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let entity_type = first_asset.entity_type.as_deref().unwrap_or("");

    use sea_orm::sea_query::Expr;
    match entity_type {
        ENTITY_PAGE_DESCRIPTION => {
            file_assets::Entity::update_many()
                .col_expr(file_assets::Column::PageId, Expr::value(entity_id))
                .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
                .exec(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
        // Workspace cover or other workspace-scoped types — just mark is_uploaded
        _ => {
            file_assets::Entity::update_many()
                .col_expr(file_assets::Column::IsUploaded, Expr::value(true))
                .filter(file_assets::Column::Id.is_in(body.asset_ids.clone()))
                .exec(&state.db)
                .await
                .map_err(AppError::Database)?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Legacy V1 file-assets endpoints ───────────────────────────────────────────
//
// These are the pre-v2 asset paths still called by FileService in the frontend:
//   DELETE /api/workspaces/file-assets/{workspace_id}/{asset_key}/
//   POST   /api/workspaces/file-assets/{workspace_id}/{asset_key}/restore/
//   DELETE /api/users/file-assets/{asset_key}/
//
// Django implementation: plane/app/views/asset/base.py
// FileAssetEndpoint.delete  → set is_deleted = True
// FileAssetViewSet.restore  → set is_deleted = False
// UserAssetsEndpoint.delete → set is_deleted = True (scoped to created_by)

/// DELETE /api/workspaces/file-assets/{workspace_id}/{asset_key}/
///
/// Soft-deletes a legacy workspace file asset by constructing the asset key
/// as `"{workspace_id}/{asset_key}"` — mirrors Django FileAssetEndpoint.delete.
#[utoipa::path(
    delete,
    path = "/api/workspaces/file-assets/{workspace_id}/{asset_key}",
    tag = "Assets",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace UUID"),
        ("asset_key"    = String, Path, description = "Asset key (filename)"),
    ),
    responses(
        (status = 204, description = "Asset soft-deleted"),
        (status = 404, description = "Asset not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_legacy_workspace_file_asset(
    State(state): State<AppState>,
    _: AnyAuth,
    Path((workspace_id, asset_key)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let full_key = format!("{}/{}", workspace_id, asset_key);

    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Asset.eq(&full_key))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Asset not found".into()))?;

    let mut am: file_assets::ActiveModel = asset.into();
    am.is_deleted = Set(true);
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/workspaces/file-assets/{workspace_id}/{asset_key}/restore/
///
/// Restores a soft-deleted legacy workspace file asset.
/// Mirrors Django FileAssetViewSet.restore.
#[utoipa::path(
    post,
    path = "/api/workspaces/file-assets/{workspace_id}/{asset_key}/restore",
    tag = "Assets",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace UUID"),
        ("asset_key"    = String, Path, description = "Asset key (filename)"),
    ),
    responses(
        (status = 204, description = "Asset restored"),
        (status = 404, description = "Asset not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn restore_legacy_workspace_file_asset(
    State(state): State<AppState>,
    _: AnyAuth,
    Path((workspace_id, asset_key)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let full_key = format!("{}/{}", workspace_id, asset_key);

    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Asset.eq(&full_key))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Asset not found".into()))?;

    let mut am: file_assets::ActiveModel = asset.into();
    am.is_deleted = Set(false);
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/users/file-assets/{asset_key}/
///
/// Soft-deletes a legacy user file asset scoped to the authenticated user.
/// Mirrors Django UserAssetsEndpoint.delete.
#[utoipa::path(
    delete,
    path = "/api/users/file-assets/{asset_key}",
    tag = "Assets",
    params(
        ("asset_key" = String, Path, description = "Asset key"),
    ),
    responses(
        (status = 204, description = "Asset soft-deleted"),
        (status = 404, description = "Asset not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_legacy_user_file_asset(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(asset_key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;

    let asset = file_assets::Entity::find()
        .filter(file_assets::Column::Asset.eq(&asset_key))
        .filter(file_assets::Column::CreatedById.eq(auth_user.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Asset not found".into()))?;

    let mut am: file_assets::ActiveModel = asset.into();
    am.is_deleted = Set(true);
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
