// src/routes/search.rs
//! Endpoints de búsqueda global y por proyecto.
//!
//!   GET /api/workspaces/{slug}/search/
//!   GET /api/workspaces/{slug}/projects/{project_id}/search-issues/

use axum::{
    extract::{Query, State},
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, sea_query::Condition};
use sea_orm::sea_query::extension::postgres::PgExpr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
        permissions::{require_role, ROLE_GUEST},
    },
    entities::{cycles, issues, modules, pages, project_members, projects, users, workspace_members},
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

// ── GET /workspaces/{slug}/entity-search/ ─────────────────────────────────────

/// Búsqueda contextual de entidades (user_mention, project, issue, cycle, module, page).
///
/// Espejo de `SearchEndpoint.get`
/// (`apps/api/plane/app/views/search/base.py`).
///
/// Query params:
///   - `query`:      texto a buscar
///   - `query_type`: tipos separados por coma (default: "user_mention")
///   - `count`:      máx resultados por tipo (default: 5)
///   - `project_id`: UUID del proyecto (opcional — restringe resultados)
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EntitySearchQuery {
    pub query: Option<String>,
    pub query_type: Option<String>,
    pub count: Option<u64>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserMentionResult {
    #[serde(rename = "member__avatar_url")]
    pub avatar_url: Option<String>,
    #[serde(rename = "member__display_name")]
    pub display_name: String,
    #[serde(rename = "member__id")]
    pub id: Uuid,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct EntitySearchResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_mention: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Vec<serde_json::Value>>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/entity-search/",
    tag = "Search",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("query" = Option<String>, Query, description = "Texto a buscar"),
        ("query_type" = Option<String>, Query, description = "Tipos separados por coma: user_mention,project,issue,cycle,module,page"),
        ("count" = Option<u64>, Query, description = "Máximo de resultados por tipo (default: 5)"),
        ("project_id" = Option<Uuid>, Query, description = "UUID del proyecto para restringir búsqueda"),
    ),
    responses((status = 200, description = "Resultados de búsqueda por entidad")),
    security(("TokenAuth" = []), ("SessionCookie" = []))
)]
pub async fn entity_search(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<EntitySearchQuery>,
) -> Result<axum::Json<serde_json::Value>, AppError> {


    let q = params.query.as_deref().unwrap_or("").trim().to_owned();
    let count = params.count.unwrap_or(5).min(50);
    let query_types: Vec<&str> = params
        .query_type
        .as_deref()
        .unwrap_or("user_mention")
        .split(',')
        .map(str::trim)
        .collect();
    let project_id = params.project_id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;

    let mut response = serde_json::Map::new();

    for query_type in &query_types {
        match *query_type {
            "user_mention" => {
                // Busca miembros activos del proyecto (si project_id) o del workspace
                let members: Vec<serde_json::Value> = if let Some(pid) = project_id {
                    let mut qb = project_members::Entity::find()
                        .inner_join(users::Entity)
                        .filter(project_members::Column::ProjectId.eq(pid))
                        .filter(project_members::Column::WorkspaceId.eq(workspace_id))
                        .filter(project_members::Column::IsActive.eq(true))
                        .filter(project_members::Column::DeletedAt.is_null());

                    if !q.is_empty() {
                        qb = qb.filter(
                            Condition::any()
                                .add(users::Column::DisplayName.contains(&q))
                                .add(users::Column::FirstName.contains(&q))
                                .add(users::Column::LastName.contains(&q)),
                        );
                    }

                    qb.limit(count)
                        .all(&state.db)
                        .await
                        .map_err(AppError::Database)?
                        .into_iter()
                        .map(|_pm| serde_json::json!({})) // placeholder — join via separate query
                        .collect()
                } else {
                    let mut qb = workspace_members::Entity::find()
                        .inner_join(users::Entity)
                        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
                        .filter(workspace_members::Column::IsActive.eq(true))
                        .filter(workspace_members::Column::DeletedAt.is_null());

                    if !q.is_empty() {
                        qb = qb.filter(
                            Condition::any()
                                .add(users::Column::DisplayName.contains(&q))
                                .add(users::Column::FirstName.contains(&q))
                                .add(users::Column::LastName.contains(&q)),
                        );
                    }

                    qb.limit(count)
                        .all(&state.db)
                        .await
                        .map_err(AppError::Database)?
                        .into_iter()
                        .map(|_wm| serde_json::json!({}))
                        .collect()
                };
                // Fetch with user join properly via raw members
                let user_results = fetch_user_mentions(&state.db, workspace_id, project_id, &q, count, user_id).await?;
                response.insert("user_mention".into(), serde_json::Value::Array(user_results));
                let _ = members; // used only to trigger query path selection above
            }
            "project" => {
                let mut qb = projects::Entity::find()
                    .filter(projects::Column::WorkspaceId.eq(workspace_id))
                    .filter(projects::Column::DeletedAt.is_null());

                if !q.is_empty() {
                    qb = qb.filter(
                        Condition::any()
                            .add(projects::Column::Name.contains(&q))
                            .add(projects::Column::Identifier.contains(&q)),
                    );
                }

                let results: Vec<serde_json::Value> = qb
                    .order_by_desc(projects::Column::CreatedAt)
                    .limit(count)
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?
                    .into_iter()
                    .map(|p| serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "identifier": p.identifier,
                        "logo_props": p.logo_props,
                    }))
                    .collect();

                response.insert("project".into(), serde_json::Value::Array(results));
            }
            "issue" => {
                let mut qb = issues::Entity::find()
                    .active()
                    .filter(issues::Column::WorkspaceId.eq(workspace_id))
                    .filter(issues::Column::IsDraft.eq(false))
                    .filter(issues::Column::ArchivedAt.is_null());

                if let Some(pid) = project_id {
                    qb = qb.filter(issues::Column::ProjectId.eq(pid));
                }

                if !q.is_empty() {
                    let mut cond = Condition::any().add(issues::Column::Name.contains(&q));
                    if let Ok(seq) = q.parse::<i32>() {
                        cond = cond.add(issues::Column::SequenceId.eq(seq));
                    }
                    qb = qb.filter(cond);
                }

                let results: Vec<serde_json::Value> = qb
                    .order_by_desc(issues::Column::CreatedAt)
                    .limit(count)
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?
                    .into_iter()
                    .map(|i| serde_json::json!({
                        "id": i.id,
                        "name": i.name,
                        "sequence_id": i.sequence_id,
                        "project_id": i.project_id,
                        "priority": i.priority,
                        "state_id": i.state_id,
                    }))
                    .collect();

                response.insert("issue".into(), serde_json::Value::Array(results));
            }
            "cycle" => {
                let mut qb = cycles::Entity::find()
                    .active()
                    .filter(cycles::Column::WorkspaceId.eq(workspace_id))
                    .filter(cycles::Column::ArchivedAt.is_null());

                if let Some(pid) = project_id {
                    qb = qb.filter(cycles::Column::ProjectId.eq(pid));
                }

                if !q.is_empty() {
                    qb = qb.filter(cycles::Column::Name.contains(&q));
                }

                let results: Vec<serde_json::Value> = qb
                    .order_by_desc(cycles::Column::CreatedAt)
                    .limit(count)
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?
                    .into_iter()
                    .map(|c| {
                        let now = chrono::Utc::now();
                        let status = match (c.start_date, c.end_date) {
                            (Some(s), Some(e)) if now < s.with_timezone(&chrono::Utc) => "UPCOMING",
                            (Some(_), Some(e)) if now > e.with_timezone(&chrono::Utc) => "COMPLETED",
                            (Some(_), Some(_)) => "CURRENT",
                            _ => "DRAFT",
                        };
                        serde_json::json!({
                            "id": c.id,
                            "name": c.name,
                            "project_id": c.project_id,
                            "status": status,
                        })
                    })
                    .collect();

                response.insert("cycle".into(), serde_json::Value::Array(results));
            }
            "module" => {
                let mut qb = modules::Entity::find()
                    .active()
                    .filter(modules::Column::WorkspaceId.eq(workspace_id))
                    .filter(modules::Column::ArchivedAt.is_null());

                if let Some(pid) = project_id {
                    qb = qb.filter(modules::Column::ProjectId.eq(pid));
                }

                if !q.is_empty() {
                    qb = qb.filter(modules::Column::Name.contains(&q));
                }

                let results: Vec<serde_json::Value> = qb
                    .order_by_desc(modules::Column::CreatedAt)
                    .limit(count)
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?
                    .into_iter()
                    .map(|m| serde_json::json!({
                        "id": m.id,
                        "name": m.name,
                        "project_id": m.project_id,
                        "status": m.status,
                    }))
                    .collect();

                response.insert("module".into(), serde_json::Value::Array(results));
            }
            "page" => {
                let mut qb = pages::Entity::find()
                    .active()
                    .filter(pages::Column::WorkspaceId.eq(workspace_id))
                    .filter(pages::Column::ArchivedAt.is_null())
                    .filter(
                        Condition::any()
                            .add(pages::Column::Access.eq(0_i16))
                            .add(pages::Column::OwnedById.eq(user_id)),
                    );

                if !q.is_empty() {
                    qb = qb.filter(pages::Column::Name.contains(&q));
                }

                let results: Vec<serde_json::Value> = qb
                    .order_by_desc(pages::Column::CreatedAt)
                    .limit(count)
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?
                    .into_iter()
                    .map(|p| serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "logo_props": p.logo_props,
                    }))
                    .collect();

                response.insert("page".into(), serde_json::Value::Array(results));
            }
            _ => {} // tipo desconocido — ignorar
        }
    }

    Ok(axum::Json(serde_json::Value::Object(response)))
}

/// Recupera usuarios para user_mention con join correcto sobre `users`.
async fn fetch_user_mentions(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Option<Uuid>,
    q: &str,
    count: u64,
    _caller_id: Uuid,
) -> Result<Vec<serde_json::Value>, AppError> {



    let results = if let Some(pid) = project_id {
        // Búsqueda restringida al proyecto
        let members = project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(pid))
            .filter(project_members::Column::WorkspaceId.eq(workspace_id))
            .filter(project_members::Column::IsActive.eq(true))
            .filter(project_members::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        let user_ids: Vec<Uuid> = members.iter().filter_map(|m| m.member_id).collect();

        let mut uq = users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .filter(users::Column::IsBot.eq(false));

        if !q.is_empty() {
            uq = uq.filter(
                Condition::any()
                    .add(users::Column::DisplayName.contains(q))
                    .add(users::Column::FirstName.contains(q))
                    .add(users::Column::LastName.contains(q)),
            );
        }

        uq.limit(count)
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| {
                let avatar_url = if !u.avatar.is_empty() { Some(u.avatar.clone()) } else { u.avatar_asset_id.map(|id| format!("/api/assets/v2/static/{id}/")) };
                serde_json::json!({
                    "member__id": u.id,
                    "member__display_name": u.display_name,
                    "member__avatar_url": avatar_url,
                })
            })
            .collect()
    } else {
        // Búsqueda en workspace completo
        let members = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
            .filter(workspace_members::Column::IsActive.eq(true))
            .filter(workspace_members::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        let user_ids: Vec<Uuid> = members.iter().filter_map(|m| m.member_id).collect();

        let mut uq = users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .filter(users::Column::IsBot.eq(false));

        if !q.is_empty() {
            uq = uq.filter(
                Condition::any()
                    .add(users::Column::DisplayName.contains(q))
                    .add(users::Column::FirstName.contains(q))
                    .add(users::Column::LastName.contains(q)),
            );
        }

        uq.limit(count)
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| {
                let avatar_url = if !u.avatar.is_empty() { Some(u.avatar.clone()) } else { u.avatar_asset_id.map(|id| format!("/api/assets/v2/static/{id}/")) };
                serde_json::json!({
                    "member__id": u.id,
                    "member__display_name": u.display_name,
                    "member__avatar_url": avatar_url,
                })
            })
            .collect()
    };

    Ok(results)
}
