// src/routes/issue_extras2.rs
//! Additional Issue endpoints.
//!
//! Covers Django equivalents of:
//!   issue/attachment.py  → issue attachments (FileAsset v2)
//!   issue/archive.py     → archive / unarchive issues
//!   issue/version.py     → issue versions
//!   issue/base.py        → bulk update of issue dates (IssueBulkUpdateDateEndpoint)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
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
        cycle_issues, file_assets, issue_assignees,
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
    /// POST data for multipart/form-data to the bucket (mirror Django/boto3 generate_presigned_post).
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
    responses((status = 200, description = "List of attachments"))
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
    responses((status = 200, description = "Presigned upload initiated"))
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
    responses((status = 204, description = "Upload confirmed"))
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
        (status = 204, description = "Deleted"),
        (status = 403, description = "Only ADMIN"),
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
        (status = 200, description = "Issue archived"),
        (status = 400, description = "Invalid state"),
        (status = 403, description = "Only MEMBER or ADMIN"),
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
    responses((status = 204, description = "Unarchived"))
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
    responses((status = 200, description = "List of versions"))
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
        (status = 200, description = "Version found"),
        (status = 404, description = "Not found"),
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
// Django mirror: apps/api/plane/app/views/issue/base.py::IssueBulkUpdateDateEndpoint
//   POST /workspaces/{slug}/projects/{project_id}/issue-dates/
//
// Updates `start_date` and/or `target_date` of multiple issues in a single
// call, validating that for each issue `start_date <= target_date` taking
// into account the current values when the client omits one of the fields.
//
// Permissions: ADMIN | MEMBER at project level (ROLE_MEMBER).
//
// Design notes:
//   - All work occurs within a transaction to ensure
//     atomicity: if any issue violates date validation, none
//     are persisted (avoids inconsistent partial states).
//   - We filter by `project_id` (taken from the guard) so that no `id`
//     sent by the client can modify issues outside the current
//     project, even if they belong to the same workspace.
//   - `updated_by_id` is set to the authenticated user, mirroring the
//     behavior of `update_issue` in this crate.
//   - We DO NOT yet emit `issue_activity` because the rest of the write endpoints
//     in the Rust port don't do it yet; introducing an ad-hoc insert
//     here would be inconsistent with the rest of the code.

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

/// Returns `false` if, taking into account the current and new values,
/// `start_date > target_date`. Maintains the exact semantics of Django's
/// `validate_dates` function.
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
        (status = 200, description = "Issues updated"),
        (status = 400, description = "Start date exceeds target date"),
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

    // Immutable snapshot of IDs + payload to move into the transaction without
    // losing the `body` borrow.
    let updates = body.updates;

    state
        .db
        .transaction::<_, (), AppError>(move |txn| {
            // `move` in the outer closure takes ownership of `updates`,
            // `project_id` and `user_id`; `async move` transfers them to the
            // inner future. Same ownership pattern as `update_issue`.
            Box::pin(async move {
                // We load all affected issues in a single query,
                // filtering by project_id to prevent a malicious id
                // from reaching another project in the same workspace.
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

                // First pass: pure validation (no writes). If any
                // row fails, the transaction aborts and nothing persists.
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
                    // If `by_id` doesn't contain the issue (doesn't exist, another
                    // project, or soft-deleted), we ignore it silently
                    // — same behavior as Django (`if not issue: continue`).
                }

                // Second pass: apply updates.
                let now = chrono::Utc::now();
                for update in &updates {
                    let Some(issue) = by_id.remove(&update.id) else {
                        continue;
                    };
                    // If the client didn't send any date for this issue,
                    // there's nothing to touch.
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
/// Soft-deletes multiple issues and their related CycleIssue/ModuleIssue.
///
/// Mirror of `BulkDeleteIssuesEndpoint`
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
    if guard.project_member.role < ROLE_ADMIN && guard.workspace_member.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }
    if body.issue_ids.is_empty() {
        return Err(AppError::BadRequest("issue_ids is required".into()));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    // Soft-delete related cycle_issues and module_issues
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
/// Lists archived issues of the project.
///
/// Mirror of `IssueArchiveViewSet.list`
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

    // Batch-fetch assignees and labels to avoid N+1
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
/// Returns the detail of a specific archived issue.
///
/// Mirror of `IssueArchiveViewSet.retrieve`
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
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<String>,
}

/// GET /workspaces/{slug}/projects/{project_id}/deleted-issues/
///
/// Returns IDs of deleted or archived issues — used by the frontend
/// for local synchronization (cache invalidation).
///
/// Mirror of `DeletedIssuesListViewSet`
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

    use sea_orm::QuerySelect;
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

    if let Some(updated_gt) = &q.updated_at_gt {
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
/// Returns sequence_id and project_identifier of an issue.
/// Used by the frontend to build canonical issue URLs.
///
/// Mirror of `IssueMetaEndpoint`
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
/// Resolves an issue from project identifier + number.
/// Example: `/work-items/PLAN-42/` resolves to the issue with sequence_id=42
/// in the project with identifier="PLAN".
///
/// Mirror of `IssueDetailIdentifierEndpoint`
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
    // Parse "PROJECT_IDENTIFIER-ISSUE_NUMBER" by splitting at the last dash
    // followed by digits (to support identifiers with dashes like "MY-PROJECT-42").
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

    // Search project by identifier (case-insensitive)
    let project = projects::Entity::find()
        .active()
        .filter(sea_orm::sea_query::Expr::col(projects::Column::Identifier).eq(project_identifier.to_uppercase()))
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verify that the user is a member of the project
    use sea_orm::PaginatorTrait;
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

    // Search issue by sequence_id
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

// ── Internal batch-fetch helpers ──────────────────────────────────────────

/// Batch-fetch of assignee_ids per issue — avoids N+1.
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

/// Batch-fetch of label_ids per issue — avoids N+1.
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

// ── POST /workspaces/{slug}/projects/{project_id}/bulk-operation-issues/ ──────
//
// Bulk-updates properties of multiple issues in a single request.
// Mirror of BulkIssueOperationEndpoint in Django.
// Payload: { issue_ids: [], properties: { state_id?, priority?, label_ids?,
//            assignee_ids?, start_date?, target_date?, module_ids?, cycle_id? } }

#[derive(Debug, serde::Deserialize)]
pub struct BulkOperationProperties {
    pub state_id: Option<Uuid>,
    pub priority: Option<String>,
    pub label_ids: Option<Vec<Uuid>>,
    pub assignee_ids: Option<Vec<Uuid>>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub module_ids: Option<Vec<Uuid>>,
    pub cycle_id: Option<Uuid>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkOperationRequest {
    pub issue_ids: Vec<Uuid>,
    pub properties: BulkOperationProperties,
}

pub async fn bulk_operation_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<BulkOperationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.issue_ids.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    let props = &body.properties;

    for issue_id in &body.issue_ids {
        let issue = issues::Entity::find_by_id(*issue_id)
            .active()
            .filter(issues::Column::ProjectId.eq(guard.project.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let Some(issue) = issue else { continue };

        let mut am: issues::ActiveModel = issue.into();
        if let Some(v) = props.state_id { am.state_id = Set(Some(v)); }
        if let Some(ref v) = props.priority { am.priority = Set(v.clone()); }
        if let Some(ref v) = props.start_date {
            am.start_date = Set(v.parse::<chrono::NaiveDate>().ok());
        }
        if let Some(ref v) = props.target_date {
            am.target_date = Set(v.parse::<chrono::NaiveDate>().ok());
        }
        am.updated_at = Set(now);
        am.updated_by_id = Set(Some(guard.user.id));
        am.update(&state.db).await.map_err(AppError::Database)?;

        // Update assignees
        if let Some(ref assignee_ids) = props.assignee_ids {
            // Remove existing
            let existing = issue_assignees::Entity::find()
                .filter(issue_assignees::Column::IssueId.eq(*issue_id))
                .filter(issue_assignees::Column::DeletedAt.is_null())
                .all(&state.db)
                .await
                .map_err(AppError::Database)?;
            for ea in existing {
                let mut a: issue_assignees::ActiveModel = ea.into();
                a.deleted_at = Set(Some(now));
                a.update(&state.db).await.map_err(AppError::Database)?;
            }
            // Insert new
            for uid in assignee_ids {
                issue_assignees::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    issue_id: Set(*issue_id),
                    assignee_id: Set(*uid),
                    project_id: Set(guard.project.id),
                    workspace_id: Set(guard.workspace.id),
                    created_by_id: Set(Some(guard.user.id)),
                    updated_by_id: Set(Some(guard.user.id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                }.insert(&state.db).await.map_err(AppError::Database)?;
            }
        }

        // Update labels
        if let Some(ref label_ids) = props.label_ids {
            let existing = issue_labels::Entity::find()
                .filter(issue_labels::Column::IssueId.eq(*issue_id))
                .filter(issue_labels::Column::DeletedAt.is_null())
                .all(&state.db)
                .await
                .map_err(AppError::Database)?;
            for el in existing {
                let mut l: issue_labels::ActiveModel = el.into();
                l.deleted_at = Set(Some(now));
                l.update(&state.db).await.map_err(AppError::Database)?;
            }
            for lid in label_ids {
                issue_labels::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    issue_id: Set(*issue_id),
                    label_id: Set(*lid),
                    project_id: Set(guard.project.id),
                    workspace_id: Set(guard.workspace.id),
                    created_by_id: Set(Some(guard.user.id)),
                    updated_by_id: Set(Some(guard.user.id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                }.insert(&state.db).await.map_err(AppError::Database)?;
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
