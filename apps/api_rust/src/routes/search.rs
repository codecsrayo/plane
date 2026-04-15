// src/routes/search.rs
//! Endpoints de búsqueda global y por proyecto.
//!
//!   GET /api/workspaces/{slug}/search/
//!   GET /api/workspaces/{slug}/projects/{project_id}/search-issues/

use axum::{
    extract::{Query, State},
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_orm::sea_query::extension::postgres::PgExpr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
        permissions::{require_role, ROLE_GUEST},
    },
    entities::{cycles, issues, modules, pages, projects},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SearchQuery {
    pub query: Option<String>,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
}

// ── Response DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SearchResult {
    pub entity_name: String,
    pub id: Uuid,
    pub name: String,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GlobalSearchResponse {
    pub issues: Vec<SearchResult>,
    pub cycles: Vec<SearchResult>,
    pub modules: Vec<SearchResult>,
    pub pages: Vec<SearchResult>,
    pub projects: Vec<SearchResult>,
}

// ── GET /workspaces/{slug}/search/ ────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/search/",
    tag = "Search",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("query" = Option<String>, Query, description = "Texto a buscar"),
        ("type" = Option<String>, Query, description = "Filtrar por tipo: issues|cycles|modules|pages|projects"),
    ),
    responses((status = 200, description = "Resultados de búsqueda")),
    security(("TokenAuth" = []))
)]
pub async fn global_search(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<SearchQuery>,
) -> Result<Json<GlobalSearchResponse>, AppError> {
    let q = params.query.as_deref().unwrap_or("").trim().to_owned();
    let entity_type = params.entity_type.as_deref().unwrap_or("all");

    if q.len() < 2 {
        return Ok(Json(GlobalSearchResponse {
            issues: vec![],
            cycles: vec![],
            modules: vec![],
            pages: vec![],
            projects: vec![],
        }));
    }

    let pattern = format!("%{}%", q.to_lowercase());
    let workspace_id = guard.workspace.id;
    const LIMIT: u64 = 10;

    // ── Issues ───────────────────────────────────────────────────────────────
    let issue_results = if entity_type == "all" || entity_type == "issues" {
        issues::Entity::find()
            .active()
            .filter(issues::Column::WorkspaceId.eq(workspace_id))
            .filter(issues::Column::IsDraft.eq(false))
            .filter(issues::Column::ArchivedAt.is_null())
            .filter(
                sea_orm::Condition::any()
                    .add(issues::Column::Name.contains(&q))
                    .add(
                        sea_orm::sea_query::Expr::col(issues::Column::Name)
                            .ilike(&pattern),
                    ),
            )
            .order_by_desc(issues::Column::UpdatedAt)
            .limit(LIMIT)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|i| SearchResult {
                entity_name: "issue".to_owned(),
                id: i.id,
                name: i.name,
                project_id: Some(i.project_id),
                workspace_id: i.workspace_id,
            })
            .collect()
    } else {
        vec![]
    };

    // ── Cycles ───────────────────────────────────────────────────────────────
    let cycle_results = if entity_type == "all" || entity_type == "cycles" {
        cycles::Entity::find()
            .active()
            .filter(cycles::Column::WorkspaceId.eq(workspace_id))
            .filter(cycles::Column::ArchivedAt.is_null())
            .filter(cycles::Column::Name.contains(&q))
            .order_by_desc(cycles::Column::UpdatedAt)
            .limit(LIMIT)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|c| SearchResult {
                entity_name: "cycle".to_owned(),
                id: c.id,
                name: c.name,
                project_id: Some(c.project_id),
                workspace_id: c.workspace_id,
            })
            .collect()
    } else {
        vec![]
    };

    // ── Modules ──────────────────────────────────────────────────────────────
    let module_results = if entity_type == "all" || entity_type == "modules" {
        modules::Entity::find()
            .active()
            .filter(modules::Column::WorkspaceId.eq(workspace_id))
            .filter(modules::Column::ArchivedAt.is_null())
            .filter(modules::Column::Name.contains(&q))
            .order_by_desc(modules::Column::UpdatedAt)
            .limit(LIMIT)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|m| SearchResult {
                entity_name: "module".to_owned(),
                id: m.id,
                name: m.name,
                project_id: Some(m.project_id),
                workspace_id: m.workspace_id,
            })
            .collect()
    } else {
        vec![]
    };

    // ── Pages ────────────────────────────────────────────────────────────────
    let page_results = if entity_type == "all" || entity_type == "pages" {
        pages::Entity::find()
            .active()
            .filter(pages::Column::WorkspaceId.eq(workspace_id))
            .filter(pages::Column::ArchivedAt.is_null())
            .filter(
                pages::Column::Access.eq(0_i16) // público
                    .or(pages::Column::OwnedById.eq(guard.user.id)),
            )
            .filter(pages::Column::Name.contains(&q))
            .order_by_desc(pages::Column::UpdatedAt)
            .limit(LIMIT)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| SearchResult {
                entity_name: "page".to_owned(),
                id: p.id,
                name: p.name,
                project_id: None, // pages pueden estar en múltiples proyectos
                workspace_id: p.workspace_id,
            })
            .collect()
    } else {
        vec![]
    };

    // ── Projects ─────────────────────────────────────────────────────────────
    let project_results = if entity_type == "all" || entity_type == "projects" {
        projects::Entity::find()
            .filter(projects::Column::WorkspaceId.eq(workspace_id))
            .filter(projects::Column::DeletedAt.is_null())
            .filter(projects::Column::Name.contains(&q))
            .order_by_desc(projects::Column::UpdatedAt)
            .limit(LIMIT)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| SearchResult {
                entity_name: "project".to_owned(),
                id: p.id,
                name: p.name,
                project_id: Some(p.id),
                workspace_id: p.workspace_id,
            })
            .collect()
    } else {
        vec![]
    };

    Ok(Json(GlobalSearchResponse {
        issues: issue_results,
        cycles: cycle_results,
        modules: module_results,
        pages: page_results,
        projects: project_results,
    }))
}

// ── GET /workspaces/{slug}/projects/{project_id}/search-issues/ ───────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/search-issues/",
    tag = "Search",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("query" = Option<String>, Query, description = "Texto a buscar"),
    ),
    responses((status = 200, description = "Issues encontrados")),
    security(("TokenAuth" = []))
)]
pub async fn search_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let q = params.query.as_deref().unwrap_or("").trim().to_owned();

    if q.len() < 2 {
        return Ok(Json(vec![]));
    }

    let rows = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::IsDraft.eq(false))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::Name.contains(&q))
        .order_by_desc(issues::Column::UpdatedAt)
        .limit(20)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|i| SearchResult {
                entity_name: "issue".to_owned(),
                id: i.id,
                name: i.name,
                project_id: Some(i.project_id),
                workspace_id: i.workspace_id,
            })
            .collect(),
    ))
}
