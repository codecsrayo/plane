// src/routes/integrations/mod.rs
//! Integrations module (GitHub, GitLab, Slack).
//!
//! Internal structure:
//!   dtos.rs      — Shared request/response DTOs
//!   helpers.rs   — Internal helpers (get_or_create_api_token)
//!   workspace.rs — workspace-integrations CRUD + provider_install
//!   github.rs    — GitHub App callback, user callback, repositories, repo-syncs
//!   gitlab.rs    — GitLab repositories
//!   pr_state.rs  — PR state mappings
//!   slack.rs     — Slack project sync (per-project channel binding)

pub mod dtos;
mod helpers;
pub mod github;
pub mod gitlab;
pub mod pr_state;
pub mod slack;
pub mod workspace;

// Re-export all public handlers so `routes/mod.rs` continues
// using `integrations::handler_name` syntax without changes.
pub use github::{
    github_app_callback,
    github_callback_auth_alias,
    github_user_callback,
    github_user_callback_get_stub,
    list_github_repositories,
    list_github_repo_syncs,
    create_github_repo_sync,
    delete_github_repo_sync,
    list_integrations,
};
pub use gitlab::list_gitlab_repositories;
pub use pr_state::{
    create_pr_state_mapping,
    delete_pr_state_mapping,
    list_pr_state_mappings,
};
pub use slack::{
    create_project_slack_sync,
    delete_project_slack_sync,
    list_project_slack_syncs,
};
pub use workspace::{
    create_workspace_integration,
    delete_workspace_integration,
    delete_workspace_integration_by_provider,
    get_workspace_integration,
    list_workspace_integrations,
    provider_install,
    update_workspace_integration,
};
