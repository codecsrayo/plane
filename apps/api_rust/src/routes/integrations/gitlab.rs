// src/routes/integrations/gitlab.rs
//! GitLab specific endpoints.
//!
//! Implemented endpoints:
//!   GET /api/workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories/

use anyhow::Context as _;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    auth::{extractors::WorkspaceMemberGuard, permissions::require_workspace_admin},
    entities::workspace_integrations,
    error::AppError,
    utils::{instance_config::get_instance_config, soft_delete::SoftDeleteExt},
    AppState,
};

use super::dtos::ExternalReposQuery;

// ── GET /workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories/ ─

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
        ("page" = Option<u32>, Query, description = "Page (default 1)"),
        ("per_page" = Option<u32>, Query, description = "Projects per page (default 30)"),
        ("token" = Option<String>, Query, description = "GitLab PAT (optional if token in env)"),
    ),
    responses(
        (status = 200, description = "GitLab projects list"),
        (status = 400, description = "Configuration error or missing token"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_gitlab_repositories(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id)): Path<(String, Uuid)>,
    Query(params): Query<ExternalReposQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wi = workspace_integrations::Entity::find_by_id(wi_id)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let gitlab_host = get_instance_config(&state, "GITLAB_HOST")
        .await?
        .unwrap_or_else(|| "https://gitlab.com".to_owned())
        .trim_end_matches('/')
        .to_owned();

    let gitlab_token = params
        .token
        .or_else(|| wi.metadata.get("code").and_then(|v: &serde_json::Value| v.as_str()).map(str::to_owned))
        .or_else(|| std::env::var("GITLAB_ACCESS_TOKEN").ok())
        .ok_or_else(|| AppError::BadRequest("GitLab token not provided".into()))?;

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(30).min(100);

    let resp = state
        .http
        .get(format!("{gitlab_host}/api/v4/projects"))
        .header("PRIVATE-TOKEN", &gitlab_token)
        .query(&[
            ("page", page.to_string()),
            ("per_page", per_page.to_string()),
            ("membership", "true".to_owned()),
            ("simple", "true".to_owned()),
            ("order_by", "updated_at".to_owned()),
        ])
        .send()
        .await
        .context("Failed to contact GitLab API")
        .map_err(AppError::Internal)?;

    if !resp.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "Failed to fetch repositories from GitLab at {gitlab_host}"
        )));
    }

    let total_count = resp
        .headers()
        .get("x-total")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let repos: Vec<serde_json::Value> = resp
        .json()
        .await
        .context("GitLab projects response is not JSON")
        .map_err(AppError::Internal)?;

    let mapped: Vec<serde_json::Value> = repos
        .iter()
        .map(|repo| {
            serde_json::json!({
                "id": repo["id"].as_i64().unwrap_or(0).to_string(),
                "full_name": repo["path_with_namespace"],
                "name": repo["name"],
                "owner": repo["namespace"]["name"],
                "description": repo.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                "private": repo.get("visibility").and_then(|v| v.as_str()).map(|v| v != "public").unwrap_or(true),
                "url": repo["web_url"],
                "issues_count": repo.get("open_issues_count").and_then(|v| v.as_u64()).unwrap_or(0),
            })
        })
        .collect();

    let total = total_count.unwrap_or(mapped.len() as u64);

    Ok(Json(serde_json::json!({
        "repositories": mapped,
        "total_count": total,
        "page": page,
    })))
}
