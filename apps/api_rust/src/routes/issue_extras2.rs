// src/routes/issue_extras2.rs
//! Endpoints adicionales de Issues.
//!
//! Cubre los equivalentes Django de:
//!   issue/attachment.py  → adjuntos de issue (FileAsset v2)
//!   issue/archive.py     → archivar / desarchivar issues
//!   issue/version.py     → versiones de issue

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_MEMBER},
    },
    entities::{file_assets, issue_versions, issues, states},
    error::AppError,
    utils::s3::{build_s3_client, presigned_put_url},
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
    pub upload_url: String,
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

    let s3 = build_s3_client(&state.config);
    let upload_url = presigned_put_url(
        &s3,
        &state.config.aws_s3_bucket,
        &asset_key,
        &body.r#type,
        3600,
    )
    .await?;

    Ok((StatusCode::OK, Json(InitiateAttachmentResponse {
        upload_url,
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
