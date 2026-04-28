// src/routes/issue_description_versions.rs
//! Work item description version endpoints.
//!
//! Mirror of `WorkItemDescriptionVersionEndpoint` in
//! `apps/api/plane/app/views/issue/version.py`.
//!
//! Implemented endpoints:
//!   GET  /api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/
//!   GET  /api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}/
//!
//! # Permission logic (Django mirror)
//! - Minimum role: GUEST.
//! - If user is GUEST AND `project.guest_view_all_features = false`
//!   AND is not the `created_by` of the issue → HTTP 403.
//!
//! # List Shape
//! Django-style paginated with fields:
//!   id, workspace, project, issue, last_saved_at, owned_by,
//!   created_at, updated_at, created_by, updated_by.
//!
//! # Detail Shape (pk)
//! All fields including description_binary, description_html,
//! description_json, description_stripped.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST},
    },
    entities::{issue_description_versions, issues},
    error::AppError,
    routes::issue_pagination::{
        empty_paginated_response, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
    },
    AppState,
};

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DescriptionVersionsQuery {
    pub cursor: Option<String>,
    pub per_page: Option<u64>,
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

/// Paginated list shape.
///
/// EXACT mirror of `required_fields` in `version.py:WorkItemDescriptionVersionEndpoint.get`
/// when there is no `pk`:
///   id, workspace, project, issue, last_saved_at, owned_by,
///   created_at, updated_at, created_by, updated_by.
///
/// FKs are serialized as their UUID (Django convention when serializer
/// uses `source` FK: returns the UUID of the related object, not the object).
#[derive(Debug, Serialize)]
pub struct DescriptionVersionListItem {
    pub id: Uuid,
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    #[serde(rename = "project")]
    pub project_id: Uuid,
    #[serde(rename = "issue")]
    pub issue_id: Uuid,
    pub last_saved_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "owned_by")]
    pub owned_by_id: Uuid,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
}

/// Detail shape (when pk is passed).
///
/// Mirror of `IssueDescriptionVersionDetailSerializer` in
/// `apps/api/plane/app/serializers/issue.py:1010-1029`.
///
/// `description_binary` is base64 encoded because it's a binary field and
/// Django serializes it via DRF as bytes; frontend expects a
/// base64 string or null.
#[derive(Debug, Serialize)]
pub struct DescriptionVersionDetail {
    pub id: Uuid,
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    #[serde(rename = "project")]
    pub project_id: Uuid,
    #[serde(rename = "issue")]
    pub issue_id: Uuid,
    pub last_saved_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "owned_by")]
    pub owned_by_id: Uuid,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    // Content fields — only in detail
    pub description_binary: Option<String>, // base64 o null
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub description_json: serde_json::Value,
}

// ── Entity conversions ────────────────────────────────────────────────

impl From<&issue_description_versions::Model> for DescriptionVersionListItem {
    fn from(m: &issue_description_versions::Model) -> Self {
        Self {
            id: m.id,
            workspace_id: m.workspace_id,
            project_id: m.project_id,
            issue_id: m.issue_id,
            last_saved_at: m.last_saved_at,
            owned_by_id: m.owned_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
        }
    }
}

impl From<issue_description_versions::Model> for DescriptionVersionDetail {
    fn from(m: issue_description_versions::Model) -> Self {
        let description_binary = m
            .description_binary
            .as_ref()
            .map(|b| BASE64.encode(b));
        Self {
            id: m.id,
            workspace_id: m.workspace_id,
            project_id: m.project_id,
            issue_id: m.issue_id,
            last_saved_at: m.last_saved_at,
            owned_by_id: m.owned_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
            description_binary,
            description_html: m.description_html,
            description_stripped: m.description_stripped,
            description_json: m.description_json,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Verifies guest restriction on a specific issue.
///
/// Mirror of logic in `version.py:WorkItemDescriptionVersionEndpoint.get`:
///   if user is GUEST AND `project.guest_view_all_features = false`
///   AND `issue.created_by != request.user` → 403.
///
/// Returns `Err(AppError::Forbidden)` if the condition is met.
async fn check_guest_issue_access(
    db: &sea_orm::DatabaseConnection,
    guard: &ProjectMemberGuard,
    work_item_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    const ROLE_GUEST_VALUE: i16 = 5;

    let is_restricted_guest =
        guard.project_member.role == ROLE_GUEST_VALUE && !guard.project.guest_view_all_features;

    if !is_restricted_guest {
        return Ok(());
    }

    // The guest can only see the issue if they are its creator
    let issue = issues::Entity::find_by_id(work_item_id)
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if issue.created_by_id != Some(user_id) {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/
///
/// Paginated list of work item description versions.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/",
    tag = "Issues",
    params(
        ("slug"          = String, Path,  description = "Workspace slug"),
        ("project_id"    = Uuid,   Path,  description = "Project ID"),
        ("work_item_id"  = Uuid,   Path,  description = "Work item (issue) ID"),
        ("cursor"        = Option<String>, Query, description = "Django cursor: {per_page}:{page}:{is_prev}"),
        ("per_page"      = Option<u64>,    Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Paginated list of description versions"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Work item not found"),
    )
)]
pub async fn list_description_versions(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<DescriptionVersionsQuery>,
    Path((_slug, _project_id, work_item_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let user_id = guard.user.id;
    let project_id = guard.project.id;
    let db = &state.db;

    check_guest_issue_access(db, &guard, work_item_id, user_id).await?;

    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    let base_query = issue_description_versions::Entity::find()
        .filter(issue_description_versions::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issue_description_versions::Column::ProjectId.eq(project_id))
        .filter(issue_description_versions::Column::IssueId.eq(work_item_id))
        .filter(issue_description_versions::Column::DeletedAt.is_null())
        .order_by_desc(issue_description_versions::Column::CreatedAt);

    let total_results = base_query
        .clone()
        .count(db)
        .await
        .map_err(AppError::Database)?;

    if total_results == 0 {
        return Ok((StatusCode::OK, Json(empty_paginated_response(page_size))));
    }

    let offset = current_page * page_size;
    let rows = base_query
        .offset(offset)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let items: Vec<DescriptionVersionListItem> = rows.iter().map(Into::into).collect();

    Ok((
        StatusCode::OK,
        Json(paginated_response(items, page_size, current_page, total_results)),
    ))
}

/// GET /workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}/
///
/// Full detail of a description version (includes binary/HTML/JSON content).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}/",
    tag = "Issues",
    params(
        ("slug"         = String, Path, description = "Workspace slug"),
        ("project_id"   = Uuid,   Path, description = "Project ID"),
        ("work_item_id" = Uuid,   Path, description = "Work item (issue) ID"),
        ("pk"           = Uuid,   Path, description = "Version ID"),
    ),
    responses(
        (status = 200, description = "Description version detail"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_description_version(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, work_item_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let user_id = guard.user.id;
    let project_id = guard.project.id;
    let db = &state.db;

    check_guest_issue_access(db, &guard, work_item_id, user_id).await?;

    let version = issue_description_versions::Entity::find_by_id(pk)
        .filter(issue_description_versions::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issue_description_versions::Column::ProjectId.eq(project_id))
        .filter(issue_description_versions::Column::IssueId.eq(work_item_id))
        .filter(issue_description_versions::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::OK, Json(DescriptionVersionDetail::from(version))))
}
