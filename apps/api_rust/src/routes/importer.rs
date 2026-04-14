// src/routes/importer.rs
//! Endpoints de importación de issues desde GitHub y GitLab.
//!
//! Equivalente a `plane/app/views/importer/` en Django.
//!
//! Rutas implementadas:
//!   GET    /api/workspaces/{slug}/importers/github/
//!   POST   /api/workspaces/{slug}/importers/github/
//!   DELETE /api/workspaces/{slug}/importers/github/{importer_id}/
//!
//!   GET    /api/workspaces/{slug}/importers/gitlab/
//!   POST   /api/workspaces/{slug}/importers/gitlab/
//!   DELETE /api/workspaces/{slug}/importers/gitlab/{importer_id}/
//!
//!   GET    /api/workspaces/{slug}/importers/github/repositories/
//!   GET    /api/workspaces/{slug}/importers/gitlab/repositories/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::extractors::WorkspaceMemberGuard,
    entities::{api_tokens, importers},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ImporterResponse {
    pub id: Uuid,
    pub service: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGithubImportRequest {
    pub project_id: Uuid,
    pub owner: String,
    pub repo: String,
    pub github_token: Option<String>,
    pub import_labels: Option<bool>,
    pub import_comments: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGitlabImportRequest {
    pub project_id: Uuid,
    /// Namespace del grupo o usuario (p.ej. "myorg").
    pub namespace: String,
    /// Nombre del proyecto en GitLab.
    pub project_name: String,
    /// Project ID numérico de GitLab.
    pub gitlab_project_id: Option<i64>,
    pub personal_access_token: Option<String>,
    pub import_labels: Option<bool>,
    pub import_comments: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GithubReposQuery {
    pub page: Option<u32>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitlabReposQuery {
    pub page: Option<u32>,
    pub token: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn importer_to_response(i: &importers::Model) -> ImporterResponse {
    ImporterResponse {
        id: i.id,
        service: i.service.clone(),
        status: i.status.clone(),
        metadata: i.metadata.clone(),
        project_id: i.project_id,
        workspace_id: i.workspace_id,
        created_at: i.created_at,
        updated_at: i.updated_at,
    }
}

/// Obtiene o crea un API token de servicio para el importer.
async fn get_or_create_importer_token(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    workspace_id: Uuid,
) -> Result<Uuid, AppError> {
    // Buscar token existente de importer para este user+workspace
    let existing = api_tokens::Entity::find()
        .active()
        .filter(api_tokens::Column::UserId.eq(user_id))
        .filter(api_tokens::Column::WorkspaceId.eq(workspace_id))
        .filter(api_tokens::Column::Label.eq("GitHub Importer Token"))
        .filter(api_tokens::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)?;

    if let Some(token) = existing {
        return Ok(token.id);
    }

    // Crear nuevo token
    let now: DateTime<FixedOffset> = Utc::now().into();
    let raw_token = uuid::Uuid::new_v4().simple().to_string();
    let new_token = api_tokens::ActiveModel {
        id: Set(Uuid::new_v4()),
        token: Set(raw_token),
        label: Set("GitHub Importer Token".to_string()),
        description: Set(String::new()),
        user_type: Set(1), // service token
        is_service: Set(true),
        is_active: Set(true),
        user_id: Set(user_id),
        workspace_id: Set(Some(workspace_id)),
        allowed_rate_limit: Set("1000/hour".to_string()),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(now),
        updated_at: Set(now),
        expired_at: Set(None),
        last_used: Set(None),
        deleted_at: Set(None),
    };

    let created = new_token.insert(db).await.map_err(AppError::Database)?;
    Ok(created.id)
}

// ── GitHub Importers ──────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/importers/github/repositories/
///
/// Lista repositorios disponibles en GitHub para importar.
/// Usa installation token del GitHub App si está configurado, o PAT como fallback.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/importers/github/repositories/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("token" = Option<String>, Query, description = "GitHub PAT (fallback)"),
    ),
    responses(
        (status = 200, description = "List of GitHub repositories"),
        (status = 400, description = "No GitHub token available"),
    )
)]
pub async fn list_github_import_repositories(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<GithubReposQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = 30u32;

    // Intentar obtener installation token del workspace integration
    let github_token = params.token;

    let github_token = match github_token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Err(AppError::BadRequest("GitHub token not provided".into()));
        }
    };

    let headers = {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {github_token}").parse().map_err(|_| {
                AppError::BadRequest("Invalid GitHub token format".into())
            })?,
        );
        h.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );
        h
    };

    let api_url = "https://api.github.com/user/repos";
    let response = state
        .http
        .get(api_url)
        .headers(headers)
        .query(&[
            ("page", page.to_string()),
            ("per_page", per_page.to_string()),
            ("sort", "updated".to_string()),
            ("type", "all".to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!("GitHub API error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to reach GitHub API"))
        })?;

    if !response.status().is_success() {
        return Err(AppError::BadRequest(
            "Failed to fetch repositories from GitHub".into(),
        ));
    }

    let repos: serde_json::Value = response.json().await.map_err(|_| {
        AppError::Internal(anyhow::anyhow!("Failed to parse GitHub response"))
    })?;

    let repos_list = repos.as_array().cloned().unwrap_or_default();
    let total_count = repos_list.len();

    let repositories: Vec<serde_json::Value> = repos_list
        .iter()
        .map(|repo| {
            serde_json::json!({
                "id": repo["id"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
                "full_name": repo["full_name"],
                "name": repo["name"],
                "owner": repo["owner"]["login"],
                "description": repo["description"],
                "private": repo["private"],
                "url": repo["html_url"],
                "issues_count": repo["open_issues_count"].as_i64().unwrap_or(0),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "repositories": repositories,
        "total_count": total_count,
        "page": page,
        "is_installation_token": false,
    })))
}

/// GET /api/workspaces/{slug}/importers/github/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/importers/github/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of GitHub imports"),
    )
)]
pub async fn list_github_importers(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    let items = importers::Entity::find()
        .active()
        .filter(importers::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(importers::Column::Service.eq("github"))
        .order_by_desc(importers::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<ImporterResponse> = items.iter().map(importer_to_response).collect();
    Ok(Json(resp))
}

/// POST /api/workspaces/{slug}/importers/github/
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/importers/github/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Import initiated"),
        (status = 400, description = "Missing required fields"),
    )
)]
pub async fn create_github_importer(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateGithubImportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let github_token = body.github_token.unwrap_or_default();
    if github_token.is_empty() {
        return Err(AppError::BadRequest(
            "github_token is required".into(),
        ));
    }
    if body.owner.is_empty() || body.repo.is_empty() {
        return Err(AppError::BadRequest(
            "owner and repo are required".into(),
        ));
    }

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    let token_id = get_or_create_importer_token(
        &state.db,
        guard.user.id,
        guard.workspace.id,
    )
    .await?;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_importer = importers::ActiveModel {
        id: Set(Uuid::new_v4()),
        service: Set("github".to_string()),
        status: Set("queued".to_string()),
        project_id: Set(body.project_id),
        workspace_id: Set(guard.workspace.id),
        initiated_by_id: Set(guard.user.id),
        token_id: Set(token_id),
        metadata: Set(serde_json::json!({
            "owner": body.owner,
            "name": body.repo,
            "repository": format!("{}/{}", body.owner, body.repo),
        })),
        config: Set(serde_json::json!({
            "github_token": github_token,
            "import_labels": body.import_labels.unwrap_or(true),
            "import_comments": body.import_comments.unwrap_or(true),
        })),
        data: Set(serde_json::json!({})),
        imported_data: Set(None),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_importer
        .insert(&txn)
        .await
        .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": created.id,
            "status": created.status,
        })),
    ))
}

/// DELETE /api/workspaces/{slug}/importers/github/{importer_id}/
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/importers/github/{importer_id}/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("importer_id" = Uuid, Path, description = "Importer ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_github_importer(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, importer_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let item = importers::Entity::find_by_id(importer_id)
        .active()
        .filter(importers::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(importers::Column::Service.eq("github"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: importers::ActiveModel = item.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GitLab Importers ──────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/importers/gitlab/repositories/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/importers/gitlab/repositories/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("token" = Option<String>, Query, description = "GitLab PAT"),
    ),
    responses(
        (status = 200, description = "List of GitLab repositories"),
        (status = 400, description = "No token provided"),
    )
)]
pub async fn list_gitlab_import_repositories(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<GitlabReposQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = 30u32;

    let token = match params.token {
        Some(t) if !t.is_empty() => t,
        _ => return Err(AppError::BadRequest("GitLab token not provided".into())),
    };

    let response = state
        .http
        .get("https://gitlab.com/api/v4/projects")
        .header("PRIVATE-TOKEN", &token)
        .query(&[
            ("page", page.to_string()),
            ("per_page", per_page.to_string()),
            ("membership", "true".to_string()),
            ("order_by", "last_activity_at".to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!("GitLab API error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to reach GitLab API"))
        })?;

    if !response.status().is_success() {
        return Err(AppError::BadRequest(
            "Failed to fetch repositories from GitLab".into(),
        ));
    }

    let projects: Vec<serde_json::Value> = response.json().await.map_err(|_| {
        AppError::Internal(anyhow::anyhow!("Failed to parse GitLab response"))
    })?;

    let repositories: Vec<serde_json::Value> = projects
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p["id"],
                "full_name": p["path_with_namespace"],
                "name": p["name"],
                "namespace": p["namespace"]["path"],
                "description": p["description"],
                "private": p["visibility"] != "public",
                "url": p["web_url"],
                "issues_count": p["open_issues_count"].as_i64().unwrap_or(0),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "repositories": repositories,
        "total_count": repositories.len(),
        "page": page,
    })))
}

/// GET /api/workspaces/{slug}/importers/gitlab/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/importers/gitlab/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of GitLab imports"),
    )
)]
pub async fn list_gitlab_importers(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    let items = importers::Entity::find()
        .active()
        .filter(importers::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(importers::Column::Service.eq("gitlab"))
        .order_by_desc(importers::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<ImporterResponse> = items.iter().map(importer_to_response).collect();
    Ok(Json(resp))
}

/// POST /api/workspaces/{slug}/importers/gitlab/
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/importers/gitlab/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Import initiated"),
        (status = 400, description = "Missing required fields"),
    )
)]
pub async fn create_gitlab_importer(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateGitlabImportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pat = body.personal_access_token.unwrap_or_default();
    if pat.is_empty() {
        return Err(AppError::BadRequest(
            "personal_access_token is required".into(),
        ));
    }
    if body.namespace.is_empty() || body.project_name.is_empty() {
        return Err(AppError::BadRequest(
            "namespace and project_name are required".into(),
        ));
    }

    let token_id = get_or_create_importer_token(
        &state.db,
        guard.user.id,
        guard.workspace.id,
    )
    .await?;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_importer = importers::ActiveModel {
        id: Set(Uuid::new_v4()),
        service: Set("gitlab".to_string()),
        status: Set("queued".to_string()),
        project_id: Set(body.project_id),
        workspace_id: Set(guard.workspace.id),
        initiated_by_id: Set(guard.user.id),
        token_id: Set(token_id),
        metadata: Set(serde_json::json!({
            "namespace": body.namespace,
            "name": body.project_name,
            "gitlab_project_id": body.gitlab_project_id,
            "repository": format!("{}/{}", body.namespace, body.project_name),
        })),
        config: Set(serde_json::json!({
            "personal_access_token": pat,
            "import_labels": body.import_labels.unwrap_or(true),
            "import_comments": body.import_comments.unwrap_or(true),
        })),
        data: Set(serde_json::json!({})),
        imported_data: Set(None),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_importer
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": created.id,
            "status": created.status,
        })),
    ))
}

/// DELETE /api/workspaces/{slug}/importers/gitlab/{importer_id}/
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/importers/gitlab/{importer_id}/",
    tag = "Importer",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("importer_id" = Uuid, Path, description = "Importer ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_gitlab_importer(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, importer_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let item = importers::Entity::find_by_id(importer_id)
        .active()
        .filter(importers::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(importers::Column::Service.eq("gitlab"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: importers::ActiveModel = item.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
