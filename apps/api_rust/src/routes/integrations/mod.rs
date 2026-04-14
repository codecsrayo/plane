// src/routes/integrations/mod.rs
//! Módulo de integraciones (GitHub, GitLab, Slack).
//!
//! Estructura interna:
//!   dtos.rs      — DTOs de request/response compartidos
//!   helpers.rs   — helpers internos (get_or_create_api_token)
//!   workspace.rs — CRUD workspace-integrations + provider_install
//!   github.rs    — GitHub App callback, user callback, repositories, repo-syncs
//!   gitlab.rs    — GitLab repositories
//!   pr_state.rs  — PR state mappings

pub mod dtos;
mod helpers;
pub mod github;
pub mod gitlab;
pub mod pr_state;
pub mod workspace;

// Re-exportar todos los handlers públicos para que `routes/mod.rs` continúe
// usando la sintaxis `integrations::handler_name` sin cambios.
pub use github::{
    github_app_callback,
    github_user_callback,
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
pub use workspace::{
    create_workspace_integration,
    delete_workspace_integration,
    delete_workspace_integration_by_provider,
    get_workspace_integration,
    list_workspace_integrations,
    provider_install,
    update_workspace_integration,
};
