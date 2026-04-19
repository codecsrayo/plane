// src/routes/issue_extras2.rs
//! Endpoints adicionales de Issues.
//!
//! Cubre los equivalentes Django de:
//!   issue/attachment.py  → adjuntos de issue (FileAsset v2)
//!   issue/archive.py     → archivar / desarchivar issues
//!   issue/version.py     → versiones de issue
//!   issue/base.py        → bulk update de fechas de issue (IssueBulkUpdateDateEndpoint)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_MEMBER},
    },
    entities::{
        cycle_issues, file_assets, issue_assignees, issue_comments,
        issue_labels, issue_versions, issues, module_issues, project_members,
        projects, states, workspaces,
    },
    error::AppError,
    utils::{s3_presigned_post::{generate_presigned_post, PresignedPost}, soft_delete::SoftDeleteExt},
    AppState,
};

const ROLE_ADMIN: i16 = 20;
const ROLE_GUEST: i16 = 5;
const ENTITY_TYPE_ISSUE_ATTACHMENT: &str = "issue_attachment";

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE ATTACHMENTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct IssueAttachmentResponse {
    pub id: Uuid,
    pub attributes: JsonValue,
    pub asset: String,
    pub is_uploaded: bool,
    pub entity_type: Option<String>,
    pub workspace_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub issue_id: Option<Uuid>,
    pub size: f64,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<file_assets::Model> for IssueAttachmentResponse {
    fn from(m: file_assets::Model) -> Self {
        Self {
            id: m.id,
            attributes: m.attributes,
            asset: m.asset,
            is_uploaded: m.is_uploaded,
            entity_type: m.entity_type,
            workspace_id: m.workspace_id,
            project_id: m.project_id,
            issue_id: m.issue_id,
            size: m.size,
            created_by_id: m.created_by_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InitiateAttachmentRequest {
    pub name: String,
    pub r#type: String,
    pub size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct InitiateAttachmentResponse {
    /// Datos para POST multipart/form-data al bucket (espejo Django/boto3 generate_presigned_post).
    pub upload_data: PresignedPost,
    pub asset_id: Uuid,
    pub attachment: IssueAttachmentResponse,
}

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses((status = 200, description = "Lista de adjuntos"))
)]
pub async fn list_issue_attachments(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;
    let db = &state.db;

    let items = file_assets::Entity::find()
        .filter(file_assets::Column::IssueId.eq(issue_id))
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .filter(file_assets::Column::EntityType.eq(ENTITY_TYPE_ISSUE_ATTACHMENT))
        .filter(file_assets::Column::IsUploaded.eq(true))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .filter(file_assets::Column::DeletedAt.is_null())
        .order_by_desc(file_assets::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<IssueAttachmentResponse> = items.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses((status = 200, description = "Upload presignado iniciado"))
)]
pub async fn initiate_issue_attachment_upload(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<InitiateAttachmentRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;
    let user_id = guard.user.id;
    let db = &state.db;

    let max_size: i64 = 5 * 1024 * 1024;
    let size = body.size.unwrap_or(max_size).min(max_size);

    let asset_key = format!(
        "{}/{}-{}",
        guard.workspace.id,
        Uuid::new_v4().simple(),
        body.name
    );

    let new_asset = file_assets::ActiveModel {
        id: Set(Uuid::new_v4()),
        attributes: Set(serde_json::json!({
            "name": body.name,
            "type": body.r#type,
            "size": size,
        })),
        asset: Set(asset_key.clone()),
        size: Set(size as f64),
        workspace_id: Set(Some(guard.workspace.id)),
        project_id: Set(Some(project_id)),
        issue_id: Set(Some(issue_id)),
        entity_type: Set(Some(ENTITY_TYPE_ISSUE_ATTACHMENT.to_string())),
        is_uploaded: Set(false),
        is_deleted: Set(false),
        is_archived: Set(false),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
        comment_id: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        page_id: Set(None),
        storage_metadata: Set(None),
        user_id: Set(None),
        draft_issue_id: Set(None),
        entity_identifier: Set(None),
    };

    let saved = new_asset.insert(db).await.map_err(AppError::Database)?;

    let upload_data = generate_presigned_post(
        &state.config.aws_s3_bucket,
        &state.config.aws_endpoint,
        &state.config.aws_region,
        &state.config.aws_access_key_id,
        &state.config.aws_secret_access_key,
        &asset_key,
        &body.r#type,
        size,
        3600,
    )?;

    Ok((StatusCode::OK, Json(InitiateAttachmentResponse {
        upload_data,
        asset_id: saved.id,
        attachment: saved.into(),
    })))
}

/// PATCH /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
        ("pk" = Uuid, Path, description = "FileAsset ID"),
    ),
    responses((status = 204, description = "Upload confirmado"))
)]
pub async fn complete_issue_attachment_upload(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;
    let user_id = guard.user.id;
    let db = &state.db;

    let asset = file_assets::Entity::find_by_id(pk)
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .filter(file_assets::Column::IssueId.eq(issue_id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: file_assets::ActiveModel = asset.into();
    active.is_uploaded = Set(true);
    active.created_by_id = Set(Some(user_id));
    active.updated_at = Set(chrono::Utc::now().into());
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
        ("pk" = Uuid, Path, description = "FileAsset ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 403, description = "Solo ADMIN"),
    )
)]
pub async fn delete_issue_attachment(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_ADMIN)?;
    let db = &state.db;

    let asset = file_assets::Entity::find_by_id(pk)
        .filter(file_assets::Column::ProjectId.eq(project_id))
        .filter(file_assets::Column::IssueId.eq(issue_id))
        .filter(file_assets::Column::IsDeleted.eq(false))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: file_assets::ActiveModel = asset.into();
    active.is_deleted = Set(true);
    active.deleted_at = Set(Some(chrono::Utc::now().into()));
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE ARCHIVE / UNARCHIVE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct ArchivedAtResponse {
    pub archived_at: String,
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 200, description = "Issue archivado"),
        (status = 400, description = "Estado inválido"),
        (status = 403, description = "Solo MEMBER o ADMIN"),
    )
)]
pub async fn archive_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;
    let db = &state.db;

    let issue = issues::Entity::find_by_id(pk)
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::DeletedAt.is_null())
        .filter(issues::Column::ArchivedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let state_id = issue.state_id.ok_or_else(|| {
        AppError::BadRequest(
            "Can only archive completed or cancelled state group issue".into(),
        )
    })?;

    let state_model = states::Entity::find_by_id(state_id)
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            AppError::BadRequest(
                "Can only archive completed or cancelled state group issue".into(),
            )
        })?;

    if state_model.group != "completed" && state_model.group != "cancelled" {
        return Err(AppError::BadRequest(
            "Can only archive completed or cancelled state group issue".into(),
        ));
    }

    let now_date = chrono::Utc::now().date_naive();
    let mut active: issues::ActiveModel = issue.into();
    active.archived_at = Set(Some(now_date));
    active.updated_at = Set(chrono::Utc::now().into());
    active.update(db).await.map_err(AppError::Database)?;

    Ok((StatusCode::OK, Json(ArchivedAtResponse {
        archived_at: now_date.to_string(),
    })))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses((status = 204, description = "Desarchivado"))
)]
pub async fn unarchive_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;
    let db = &state.db;

    let issue = issues::Entity::find_by_id(pk)
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::DeletedAt.is_null())
        .filter(issues::Column::ArchivedAt.is_not_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: issues::ActiveModel = issue.into();
    active.archived_at = Set(None);
    active.updated_at = Set(chrono::Utc::now().into());
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /workspaces/{slug}/projects/{project_id}/bulk-archive-issues/
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkArchiveRequest {
    pub issue_ids: Vec<Uuid>,
}

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/bulk-archive-issues/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Issues archivados en bloque"))
)]
pub async fn bulk_archive_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<BulkArchiveRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;
    let db = &state.db;

    if body.issue_ids.is_empty() {
        return Err(AppError::BadRequest("Issue IDs are required".into()));
    }

    let now_date = chrono::Utc::now().date_naive();

    for id in &body.issue_ids {
        let issue = issues::Entity::find_by_id(*id)
            .filter(issues::Column::ProjectId.eq(project_id))
            .filter(issues::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?;

        if let Some(issue) = issue {
            if let Some(state_id) = issue.state_id {
                if let Some(s) = states::Entity::find_by_id(state_id)
                    .one(db)
                    .await
                    .map_err(AppError::Database)?
                {
                    if s.group != "completed" && s.group != "cancelled" {
                        return Err(AppError::BadRequest("INVALID_ARCHIVE_STATE_GROUP".into()));
                    }
                }
            }
            let mut active: issues::ActiveModel = issue.into();
            active.archived_at = Set(Some(now_date));
            active.updated_at = Set(chrono::Utc::now().into());
            active.update(db).await.map_err(AppError::Database)?;
        }
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"archived_at": now_date.to_string()}))))
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE VERSIONS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct IssueVersionResponse {
    pub id: Uuid,
    pub name: String,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sequence_id: i32,
    pub sort_order: f64,
    pub is_draft: bool,
    pub last_saved_at: DateTime<Utc>,
    pub owned_by_id: Uuid,
    pub assignees: Vec<Uuid>,
    pub labels: Vec<Uuid>,
    pub workspace_id: Uuid,
    pub project_id: Uuid,
    pub issue_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<issue_versions::Model> for IssueVersionResponse {
    fn from(m: issue_versions::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            priority: m.priority,
            start_date: m.start_date,
            target_date: m.target_date,
            sequence_id: m.sequence_id,
            sort_order: m.sort_order,
            is_draft: m.is_draft,
            last_saved_at: m.last_saved_at.into(),
            owned_by_id: m.owned_by_id,
            assignees: m.assignees,
            labels: m.labels,
            workspace_id: m.workspace_id,
            project_id: m.project_id,
            issue_id: m.issue_id,
            created_by_id: m.created_by_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses((status = 200, description = "Lista de versiones"))
)]
pub async fn list_issue_versions(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;
    let db = &state.db;

    let versions = issue_versions::Entity::find()
        .filter(issue_versions::Column::ProjectId.eq(project_id))
        .filter(issue_versions::Column::IssueId.eq(issue_id))
        .filter(issue_versions::Column::DeletedAt.is_null())
        .order_by_desc(issue_versions::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<IssueVersionResponse> = versions.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
        ("pk" = Uuid, Path, description = "Version ID"),
    ),
    responses(
        (status = 200, description = "Versión encontrada"),
        (status = 404, description = "No encontrada"),
    )
)]
pub async fn get_issue_version(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;
    let db = &state.db;

    let version = issue_versions::Entity::find_by_id(pk)
        .filter(issue_versions::Column::ProjectId.eq(project_id))
        .filter(issue_versions::Column::IssueId.eq(issue_id))
        .filter(issue_versions::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::OK, Json(IssueVersionResponse::from(version))))
}

// ═══════════════════════════════════════════════════════════════════════════
// BULK UPDATE ISSUE DATES
// ═══════════════════════════════════════════════════════════════════════════
//
// Espejo Django: apps/api/plane/app/views/issue/base.py::IssueBulkUpdateDateEndpoint
//   POST /workspaces/{slug}/projects/{project_id}/issue-dates/
//
// Actualiza `start_date` y/o `target_date` de múltiples issues en una sola
// llamada, validando que para cada issue `start_date <= target_date` tomando
// en cuenta los valores actuales cuando el cliente omite uno de los campos.
//
// Permisos: ADMIN | MEMBER a nivel de proyecto (ROLE_MEMBER).
//
// Notas de diseño:
//   - Todo el trabajo ocurre dentro de una transacción para garantizar
//     atomicidad: si alguna issue viola la validación de fechas, ninguna
//     se persiste (evita estados parciales inconsistentes).
//   - Filtramos por `project_id` (tomado del guard) para que ningún `id`
//     enviado por el cliente pueda modificar issues fuera del proyecto
//     actual, incluso si pertenecen al mismo workspace.
//   - `updated_by_id` se setea al usuario autenticado, espejando el
//     comportamiento de `update_issue` en este crate.
//   - Todavía NO emitimos `issue_activity` porque el resto de endpoints de
//     escritura del port Rust aún no lo hace; introducir un insert ad-hoc
//     aquí sería inconsistente con el resto del código.

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IssueDateUpdate {
    pub id: Uuid,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkUpdateIssueDatesRequest {
    pub updates: Vec<IssueDateUpdate>,
}

/// Devuelve `false` si, tomando en cuenta los valores actuales y los nuevos,
/// `start_date > target_date`. Mantiene la semántica exacta de la función
/// `validate_dates` de Django.
fn validate_dates(
    current_start: Option<NaiveDate>,
    current_target: Option<NaiveDate>,
    new_start: Option<NaiveDate>,
    new_target: Option<NaiveDate>,
) -> bool {
    let start = new_start.or(current_start);
    let target = new_target.or(current_target);
    match (start, target) {
        (Some(s), Some(t)) => s <= t,
        _ => true,
    }
}

/// POST /workspaces/{slug}/projects/{project_id}/issue-dates/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/issue-dates/",
    tag = "Issue Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    request_body = BulkUpdateIssueDatesRequest,
    responses(
        (status = 200, description = "Issues actualizados"),
        (status = 400, description = "Start date excede target date"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn bulk_update_issue_dates(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id)): Path<(String, Uuid)>,
    Json(body): Json<BulkUpdateIssueDatesRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // Early-exit: nada que hacer.
    if body.updates.is_empty() {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({"message": "Issues updated successfully"})),
        ));
    }

    let project_id = guard.project.id;
    let user_id = guard.user.id;

    // Snapshot inmutable de IDs + payload para mover a la transacción sin
    // perder el borrow de `body`.
    let updates = body.updates;

    state
        .db
        .transaction::<_, (), AppError>(move |txn| {
            // `move` en el closure externo toma ownership de `updates`,
            // `project_id` y `user_id`; `async move` los transfiere al
            // future interno. Mismo patrón de ownership que `update_issue`.
            Box::pin(async move {
                // Cargamos todas las issues afectadas en una sola query,
                // filtrando por project_id para evitar que un id malicioso
                // alcance otro proyecto del mismo workspace.
                let ids: Vec<Uuid> = updates.iter().map(|u| u.id).collect();
                let issues_loaded = issues::Entity::find()
                    .filter(issues::Column::Id.is_in(ids))
                    .filter(issues::Column::ProjectId.eq(project_id))
                    .filter(issues::Column::DeletedAt.is_null())
                    .all(txn)
                    .await
                    .map_err(AppError::Database)?;

                // Index by id para lookup O(1).
                let mut by_id: std::collections::HashMap<Uuid, issues::Model> = issues_loaded
                    .into_iter()
                    .map(|m| (m.id, m))
                    .collect();

                // Primera pasada: validación pura (sin writes). Si alguna
                // fila falla, la transacción aborta y nada persiste.
                for update in &updates {
                    if let Some(issue) = by_id.get(&update.id) {
                        if !validate_dates(
                            issue.start_date,
                            issue.target_date,
                            update.start_date,
                            update.target_date,
                        ) {
                            return Err(AppError::BadRequest(
                                "Start date cannot exceed target date".into(),
                            ));
                        }
                    }
                    // Si `by_id` no contiene el issue (no existe, otro
                    // proyecto, o soft-deleted), lo ignoramos silenciosamente
                    // — mismo comportamiento que Django (`if not issue: continue`).
                }

                // Segunda pasada: aplicar updates.
                let now = chrono::Utc::now();
                for update in &updates {
                    let Some(issue) = by_id.remove(&update.id) else {
                        continue;
                    };
                    // Si el cliente no envió ninguna fecha para este issue,
                    // no hay nada que tocar.
                    if update.start_date.is_none() && update.target_date.is_none() {
                        continue;
                    }

                    let mut am: issues::ActiveModel = issue.into();
                    if let Some(sd) = update.start_date {
                        am.start_date = Set(Some(sd));
                    }
                    if let Some(td) = update.target_date {
                        am.target_date = Set(Some(td));
                    }
                    am.updated_by_id = Set(Some(user_id));
                    am.updated_at = Set(now.into());
                    am.update(txn).await.map_err(AppError::Database)?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "Issues updated successfully"})),
    ))
}


// ═══════════════════════════════════════════════════════════════════════════
// BULK DELETE ISSUES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct BulkDeleteIssuesRequest {
    pub issue_ids: Vec<uuid::Uuid>,
}

/// DELETE /workspaces/{slug}/projects/{project_id}/bulk-delete-issues/
///
/// Soft-deletes múltiples issues y sus CycleIssue/ModuleIssue relacionados.
///
/// Espejo de `BulkDeleteIssuesEndpoint`
/// (`apps/api/plane/app/views/issue/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/bulk-delete-issues/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn bulk_delete_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<BulkDeleteIssuesRequest>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    use sea_orm::ActiveValue::Set;
    if guard.project_member.role < ROLE_ADMIN && guard.workspace_member.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }
    if body.issue_ids.is_empty() {
        return Err(AppError::BadRequest("issue_ids is required".into()));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    // Soft-delete cycle_issues y module_issues relacionados
    cycle_issues::Entity::update_many()
        .col_expr(cycle_issues::Column::DeletedAt, sea_orm::sea_query::Expr::value(Some(now)))
        .filter(cycle_issues::Column::IssueId.is_in(body.issue_ids.clone()))
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    module_issues::Entity::update_many()
        .col_expr(module_issues::Column::DeletedAt, sea_orm::sea_query::Expr::value(Some(now)))
        .filter(module_issues::Column::IssueId.is_in(body.issue_ids.clone()))
        .filter(module_issues::Column::DeletedAt.is_null())
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Soft-delete issues
    let total = issues::Entity::update_many()
        .col_expr(issues::Column::DeletedAt, sea_orm::sea_query::Expr::value(Some(now)))
        .filter(issues::Column::Id.is_in(body.issue_ids.clone()))
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::DeletedAt.is_null())
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?
        .rows_affected;

    Ok(axum::Json(serde_json::json!({
        "message": format!("{total} issues were deleted"),
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// ARCHIVED ISSUES
// ═══════════════════════════════════════════════════════════════════════════

/// GET /workspaces/{slug}/projects/{project_id}/archived-issues/
///
/// Lista issues archivados del proyecto.
///
/// Espejo de `IssueArchiveViewSet.list`
/// (`apps/api/plane/app/views/issue/archive.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/archived-issues/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn list_archived_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<axum::Json<Vec<serde_json::Value>>, AppError> {
    use crate::auth::permissions::ROLE_MEMBER;
    use sea_orm::QueryOrder;
    if guard.project_member.role < ROLE_MEMBER && guard.workspace_member.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let archived = issues::Entity::find()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issues::Column::ArchivedAt.is_not_null())
        .filter(issues::Column::DeletedAt.is_null())
        .order_by_desc(issues::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batch-fetch assignees y labels para evitar N+1
    let issue_ids: Vec<uuid::Uuid> = archived.iter().map(|i| i.id).collect();

    let assignees_map = issue_assignees_map(&state.db, &issue_ids).await?;
    let labels_map = issue_labels_map(&state.db, &issue_ids).await?;

    let result = archived.iter().map(|issue| {
        serde_json::json!({
            "id": issue.id,
            "name": issue.name,
            "sequence_id": issue.sequence_id,
            "priority": issue.priority,
            "state_id": issue.state_id,
            "parent_id": issue.parent_id,
            "project_id": issue.project_id,
            "workspace_id": issue.workspace_id,
            "archived_at": issue.archived_at,
            "created_at": issue.created_at,
            "updated_at": issue.updated_at,
            "assignee_ids": assignees_map.get(&issue.id).cloned().unwrap_or_default(),
            "label_ids": labels_map.get(&issue.id).cloned().unwrap_or_default(),
        })
    }).collect();

    Ok(axum::Json(result))
}

/// GET /workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/
///
/// Retorna el detalle de un issue archivado específico.
///
/// Espejo de `IssueArchiveViewSet.retrieve`
/// (`apps/api/plane/app/views/issue/archive.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn get_archived_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let issue = issues::Entity::find_by_id(pk)
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::ArchivedAt.is_not_null())
        .filter(issues::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let issue_ids = vec![issue.id];
    let assignees = issue_assignees_map(&state.db, &issue_ids).await?;
    let labels = issue_labels_map(&state.db, &issue_ids).await?;

    Ok(axum::Json(serde_json::json!({
        "id": issue.id,
        "name": issue.name,
        "sequence_id": issue.sequence_id,
        "priority": issue.priority,
        "state_id": issue.state_id,
        "parent_id": issue.parent_id,
        "project_id": issue.project_id,
        "workspace_id": issue.workspace_id,
        "archived_at": issue.archived_at,
        "description_html": issue.description_html,
        "created_at": issue.created_at,
        "updated_at": issue.updated_at,
        "assignee_ids": assignees.get(&issue.id).cloned().unwrap_or_default(),
        "label_ids": labels.get(&issue.id).cloned().unwrap_or_default(),
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// DELETED ISSUES LIST
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, serde::Deserialize)]
pub struct DeletedIssuesQuery {
    pub updated_at__gt: Option<String>,
}

/// GET /workspaces/{slug}/projects/{project_id}/deleted-issues/
///
/// Retorna IDs de issues eliminados o archivados — usado por el frontend
/// para sincronización local (invalidar caché).
///
/// Espejo de `DeletedIssuesListViewSet`
/// (`apps/api/plane/app/views/issue/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/deleted-issues/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn list_deleted_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(q): Query<DeletedIssuesQuery>,
) -> Result<axum::Json<Vec<uuid::Uuid>>, AppError> {
    use sea_orm::Condition;

    let mut query = issues::Entity::find()
        .select_only()
        .column(issues::Column::Id)
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(
            Condition::any()
                .add(issues::Column::ArchivedAt.is_not_null())
                .add(issues::Column::DeletedAt.is_not_null())
        );

    if let Some(updated_gt) = &q.updated_at__gt {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_gt) {
            query = query.filter(issues::Column::UpdatedAt.gt(dt.fixed_offset()));
        }
    }

    let ids: Vec<uuid::Uuid> = query
        .into_tuple::<uuid::Uuid>()
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(axum::Json(ids))
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE META
// ═══════════════════════════════════════════════════════════════════════════

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta/
///
/// Retorna sequence_id y project_identifier de un issue.
/// Usado por el frontend para construir URLs canónicas de issue.
///
/// Espejo de `IssueMetaEndpoint`
/// (`apps/api/plane/app/views/issue/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn get_issue_meta(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let issue = issues::Entity::find_by_id(issue_id)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(axum::Json(serde_json::json!({
        "sequence_id": issue.sequence_id,
        "project_identifier": guard.project.identifier,
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE BY IDENTIFIER (e.g. /work-items/PROJ-42/)
// ═══════════════════════════════════════════════════════════════════════════

/// GET /workspaces/{slug}/work-items/{project_identifier}-{issue_identifier}/
///
/// Resuelve un issue a partir del identificador de proyecto + número.
/// Ejemplo: `/work-items/PLAN-42/` resuelve al issue con sequence_id=42
/// en el proyecto con identifier="PLAN".
///
/// Espejo de `IssueDetailIdentifierEndpoint`
/// (`apps/api/plane/app/views/issue/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/work-items/{project_identifier_issue_identifier}/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn get_issue_by_identifier(
    State(state): State<AppState>,
    crate::auth::any_auth::AnyAuth(user): crate::auth::any_auth::AnyAuth,
    Path((slug, combined)): Path<(String, String)>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    // Parsear "PROJECT_IDENTIFIER-ISSUE_NUMBER" separando por el último guión
    // seguido de dígitos (para soportar identifiers con guiones como "MY-PROJECT-42").
    let split_idx = combined.rfind('-').ok_or_else(|| {
        AppError::BadRequest("Invalid identifier format. Expected: PROJECT-NUMBER".into())
    })?;

    let project_identifier = &combined[..split_idx];
    let issue_identifier_str = &combined[split_idx + 1..];

    let issue_number: i32 = issue_identifier_str.parse().map_err(|_| {
        AppError::BadRequest("Invalid issue identifier — must be a number".into())
    })?;

    // Buscar workspace
    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Buscar proyecto por identifier (case-insensitive)
    let project = projects::Entity::find()
        .active()
        .filter(sea_orm::sea_query::Expr::col(projects::Column::Identifier).eq(project_identifier.to_uppercase()))
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verificar que el usuario es miembro del proyecto
    let is_member = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    if is_member == 0 {
        return Err(AppError::Forbidden);
    }

    // Buscar issue por sequence_id
    let issue = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project.id))
        .filter(issues::Column::SequenceId.eq(issue_number))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let issue_ids = vec![issue.id];
    let assignees = issue_assignees_map(&state.db, &issue_ids).await?;
    let labels = issue_labels_map(&state.db, &issue_ids).await?;

    Ok(axum::Json(serde_json::json!({
        "id": issue.id,
        "sequence_id": issue.sequence_id,
        "name": issue.name,
        "priority": issue.priority,
        "state_id": issue.state_id,
        "project_id": issue.project_id,
        "workspace_id": issue.workspace_id,
        "project_identifier": project.identifier,
        "created_at": issue.created_at,
        "updated_at": issue.updated_at,
        "assignee_ids": assignees.get(&issue.id).cloned().unwrap_or_default(),
        "label_ids": labels.get(&issue.id).cloned().unwrap_or_default(),
    })))
}

// ── Helpers internos de batch-fetch ──────────────────────────────────────────

/// Batch-fetch de assignee_ids por issue — evita N+1.
async fn issue_assignees_map(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[uuid::Uuid],
) -> Result<std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>>, AppError> {
    if issue_ids.is_empty() { return Ok(Default::default()); }
    let rows = issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut map: std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>> = Default::default();
    for row in rows { map.entry(row.issue_id).or_default().push(row.assignee_id); }
    Ok(map)
}

/// Batch-fetch de label_ids por issue — evita N+1.
async fn issue_labels_map(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[uuid::Uuid],
) -> Result<std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>>, AppError> {
    if issue_ids.is_empty() { return Ok(Default::default()); }
    let rows = issue_labels::Entity::find()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut map: std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>> = Default::default();
    for row in rows { map.entry(row.issue_id).or_default().push(row.label_id); }
    Ok(map)
}
