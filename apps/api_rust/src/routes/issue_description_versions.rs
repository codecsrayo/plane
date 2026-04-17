// src/routes/issue_description_versions.rs
//! Endpoints de descripción-versiones de work items.
//!
//! Espejo de `WorkItemDescriptionVersionEndpoint` en
//! `apps/api/plane/app/views/issue/version.py`.
//!
//! Endpoints implementados:
//!   GET  /api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/
//!   GET  /api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}/
//!
//! # Lógica de permisos (mirror Django)
//! - Rol mínimo: GUEST.
//! - Si el usuario es GUEST Y `project.guest_view_all_features = false`
//!   Y no es el `created_by` del issue → HTTP 403.
//!
//! # Shape de lista
//! Paginado estilo Django con los campos:
//!   id, workspace, project, issue, last_saved_at, owned_by,
//!   created_at, updated_at, created_by, updated_by.
//!
//! # Shape de detalle (pk)
//! Todos los campos incluyendo description_binary, description_html,
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

/// Shape de la lista paginada.
///
/// Espejo EXACTO de los `required_fields` en `version.py:WorkItemDescriptionVersionEndpoint.get`
/// cuando no hay `pk`:
///   id, workspace, project, issue, last_saved_at, owned_by,
///   created_at, updated_at, created_by, updated_by.
///
/// Las FK se serializa como su UUID (convención Django cuando el serializer
/// usa `source` FK: devuelve el UUID del objeto relacionado, no el objeto).
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

/// Shape del detalle (cuando se pasa pk).
///
/// Espejo de `IssueDescriptionVersionDetailSerializer` en
/// `apps/api/plane/app/serializers/issue.py:1010-1029`.
///
/// `description_binary` se codifica en base64 porque es un campo binario y
/// Django lo serializa vía DRF como bytes; el frontend espera un string
/// base64 o null.
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
    // Campos de contenido — solo en el detalle
    pub description_binary: Option<String>, // base64 o null
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub description_json: serde_json::Value,
}

// ── Conversiones desde entidad ────────────────────────────────────────────────

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

/// Verifica la restricción de guest sobre un issue específico.
///
/// Espejo de la lógica en `version.py:WorkItemDescriptionVersionEndpoint.get`:
///   si el user es GUEST Y `project.guest_view_all_features = false`
///   Y `issue.created_by != request.user` → 403.
///
/// Retorna `Err(AppError::Forbidden)` si la condición se cumple.
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

    // El guest solo puede ver el issue si es su creador
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
/// Lista paginada de versiones de descripción de un work item.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/",
    tag = "Issues",
    params(
        ("slug"          = String, Path,  description = "Workspace slug"),
        ("project_id"    = Uuid,   Path,  description = "Project ID"),
        ("work_item_id"  = Uuid,   Path,  description = "Work item (issue) ID"),
        ("cursor"        = Option<String>, Query, description = "Cursor Django: {per_page}:{page}:{is_prev}"),
        ("per_page"      = Option<u64>,    Query, description = "Tamaño de página"),
    ),
    responses(
        (status = 200, description = "Lista paginada de versiones de descripción"),
        (status = 403, description = "Sin acceso"),
        (status = 404, description = "Work item no encontrado"),
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
/// Detalle completo de una versión de descripción (incluye contenido binario/HTML/JSON).
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
        (status = 200, description = "Detalle de versión de descripción"),
        (status = 403, description = "Sin acceso"),
        (status = 404, description = "No encontrado"),
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
