// src/jobs/github_sync.rs
//! Job: initial import of issues from GitHub when a GithubRepositorySync is created.
//!
//! Equivalent to `plane/bgtasks/github_sync_task.py::github_initial_issue_sync_task`.
//!
//! Flow:
//!   1. Obtain installation token from the GitHub App
//!   2. Paginate `GET /repos/{owner}/{repo}/issues?state=all`
//!   3. For each non-synchronized issue: create Issue + GithubIssueSync
//!
//! The job is idempotent: it checks GithubIssueSync before inserting.

use apalis::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{
        github_issue_syncs, github_repository_syncs, github_repositories,
        issues, states, workspace_integrations,
    },
    utils::{
        github_app::get_installation_access_token,
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

// ── Job payload ───────────────────────────────────────────────────────────────

/// Job payload: UUID of the GithubRepositorySync to process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubInitialSyncJob {
    pub repo_sync_id: Uuid,
}


// ── Handler ───────────────────────────────────────────────────────────────────

/// Executes the initial import of issues from GitHub.
pub async fn handle_github_initial_sync(
    job: GithubInitialSyncJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();
    let repo_sync_id = job.repo_sync_id;

    if let Err(e) = run_sync(&state, repo_sync_id).await {
        tracing::error!(
            repo_sync_id = %repo_sync_id,
            error = %e,
            "github_initial_sync: job failed"
        );
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

async fn run_sync(state: &AppState, repo_sync_id: Uuid) -> anyhow::Result<()> {
    use anyhow::Context as _;

    // 1. Load sync with its relations
    let sync = github_repository_syncs::Entity::find_by_id(repo_sync_id)
        .active()
        .one(&state.db)
        .await?
        .context("GithubRepositorySync not found")?;

    let repo = github_repositories::Entity::find_by_id(sync.repository_id)
        .one(&state.db)
        .await?
        .context("GithubRepository not found")?;

    let wi = workspace_integrations::Entity::find_by_id(sync.workspace_integration_id)
        .one(&state.db)
        .await?
        .context("WorkspaceIntegration not found")?;

    // 2. Obtain installation token
    let installation_id = wi
        .metadata
        .get("installation_id")
        .and_then(|v| v.as_str())
        .context("No installation_id in workspace_integration metadata")?;

    let token = get_installation_access_token(state, installation_id)
        .await?
        .context("Could not obtain installation access token")?;

    // 3. Resolve open/closed states from sync credentials
    let credentials = &sync.credentials;
    let open_state_id: Option<Uuid> = credentials
        .get("issue_open_state")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok());
    let closed_state_id: Option<Uuid> = credentials
        .get("issue_closed_state")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok());

    let open_state = resolve_state(&state.db, sync.project_id, open_state_id, false).await?;
    let closed_state = resolve_state(&state.db, sync.project_id, closed_state_id, true).await?;

    // 4. Load set of already synchronized IDs (idempotency)
    let existing: std::collections::HashSet<i64> =
        github_issue_syncs::Entity::find()
            .filter(github_issue_syncs::Column::RepositorySyncId.eq(repo_sync_id))
            .filter(github_issue_syncs::Column::DeletedAt.is_null())
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| s.github_issue_id)
            .collect();

    // 5. Paginate issues from GitHub
    let owner = &repo.owner;
    let repo_name = &repo.name;
    let actor_id = sync.actor_id;
    let project_id = sync.project_id;
    let workspace_id = sync.workspace_id;

    let mut page = 1u32;
    let mut imported = 0usize;
    let mut skipped = 0usize;

    loop {
        let resp = state
            .http
            .get(format!("https://api.github.com/repos/{owner}/{repo_name}/issues"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "plane-api-rust/0.1")
            .query(&[
                ("state", "all"),
                ("per_page", "100"),
                ("page", &page.to_string()),
                ("sort", "created"),
                ("direction", "asc"),
            ])
            .send()
            .await
            .context("Error contacting GitHub API")?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            anyhow::bail!("GitHub API 401 — installation token expired or invalid");
        }

        if !resp.status().is_success() {
            let status = resp.status();
            tracing::warn!(%status, page, "github_initial_sync: GitHub returned error, stopping pagination");
            break;
        }

        let gh_issues: Vec<serde_json::Value> = resp.json().await?;
        if gh_issues.is_empty() {
            break; // No more pages
        }

        for gh_issue in &gh_issues {
            // GitHub includes PRs in /issues — ignore them
            if gh_issue.get("pull_request").is_some() {
                skipped += 1;
                continue;
            }

            let gh_id = match gh_issue["id"].as_i64() {
                Some(id) => id,
                None => {
                    skipped += 1;
                    continue;
                }
            };

            if existing.contains(&gh_id) {
                skipped += 1;
                continue;
            }

            let gh_number = gh_issue["number"].as_i64().unwrap_or(0);
            let title = gh_issue["title"].as_str().unwrap_or("(no title)");
            let body = gh_issue["body"].as_str().unwrap_or("");
            let gh_state = gh_issue["state"].as_str().unwrap_or("open");
            let issue_url = gh_issue["html_url"].as_str().unwrap_or("");

            let target_state_id = if gh_state == "closed" {
                closed_state.or(open_state)
            } else {
                open_state.or(closed_state)
            };

            // Calculate sequence_id simply (not strictly SERIALIZABLE
            // here because job is single-threaded by apalis design)
            let max_seq: Option<i64> = issues::Entity::find()
                .filter(issues::Column::ProjectId.eq(project_id))
                .select_only()
                .column_as(
                    sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
                    "max_seq",
                )
                .into_tuple()
                .one(&state.db)
                .await?;
            let sequence_id = (max_seq.unwrap_or(0) + 1) as i32;

            let plane_issue = issues::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(title.chars().take(255).collect()),
                description_html: Set(body.to_owned()),
                description_json: Set(serde_json::json!({})),
                priority: Set("none".to_owned()),
                state_id: Set(target_state_id),
                sequence_id: Set(sequence_id),
                sort_order: Set(65535.0),
                project_id: Set(project_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(actor_id)),
                updated_by_id: Set(Some(actor_id)),
                is_draft: Set(false),
                description_stripped: Set(None),
                ..Default::default()
            }
            .insert(&state.db)
            .await?;

            github_issue_syncs::ActiveModel {
                id: Set(Uuid::new_v4()),
                issue_id: Set(plane_issue.id),
                repository_sync_id: Set(repo_sync_id),
                repo_issue_id: Set(gh_number),
                github_issue_id: Set(gh_id),
                issue_url: Set(issue_url.to_owned()),
                project_id: Set(project_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(actor_id)),
                updated_by_id: Set(Some(actor_id)),
                ..Default::default()
            }
            .insert(&state.db)
            .await?;

            imported += 1;
        }

        page += 1;
    }

    tracing::info!(
        repo_sync_id = %repo_sync_id,
        imported,
        skipped,
        "github_initial_sync: completed"
    );

    Ok(())
}

/// Looks up a state by ID or falls back to project default state.
/// If `prefer_closed` is `true`, looks for a state in the "cancelled" group as fallback.
async fn resolve_state(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    state_id: Option<Uuid>,
    prefer_closed: bool,
) -> anyhow::Result<Option<Uuid>> {
    if let Some(id) = state_id {
        let exists = states::Entity::find_by_id(id)
            .filter(states::Column::ProjectId.eq(project_id))
            .filter(states::Column::DeletedAt.is_null())
            .one(db)
            .await?;
        if exists.is_some() {
            return Ok(Some(id));
        }
    }

    // Fallback: project default state
    let fallback_group = if prefer_closed { "cancelled" } else { "backlog" };

    let fallback = states::Entity::find()
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::DeletedAt.is_null())
        .filter(states::Column::Default.eq(true))
        .one(db)
        .await?
        .or({
            // If no default, search by group
            None // search below
        });

    if let Some(s) = fallback {
        return Ok(Some(s.id));
    }

    // Last resort: first state of desired group
    let by_group = states::Entity::find()
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::DeletedAt.is_null())
        .filter(states::Column::Group.eq(fallback_group))
        .one(db)
        .await?;

    Ok(by_group.map(|s| s.id))
}
