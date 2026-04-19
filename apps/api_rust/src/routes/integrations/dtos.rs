// src/routes/integrations/dtos.rs
//! DTOs de request/response y constantes compartidas para el módulo
//! de integraciones (GitHub, GitLab, Slack).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    db_githubprstatemapping, integrations, workspace_integrations,
};

// ── Constantes ────────────────────────────────────────────────────────────────

/// Valores de `github_pr_state` permitidos (enum Postgres).
pub const VALID_PR_STATES: &[&str] = &[
    "draft_open",
    "open",
    "review_requested",
    "ready_for_merge",
    "merged",
    "closed",
];

// ── Response DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntegrationResponse {
    pub id: Uuid,
    pub title: String,
    pub provider: String,
    pub network: i32,
    pub description: serde_json::Value,
    pub author: String,
    pub avatar_url: Option<String>,
    pub verified: bool,
}

impl IntegrationResponse {
    pub fn from_model(m: integrations::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            provider: m.provider,
            network: m.network,
            description: m.description,
            author: m.author,
            avatar_url: m.avatar_url,
            verified: m.verified,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceIntegrationResponse {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub workspace_id: Uuid,
    pub actor_id: Uuid,
    pub metadata: serde_json::Value,
    pub config: serde_json::Value,
    pub integration: Option<IntegrationResponse>,
}

impl WorkspaceIntegrationResponse {
    pub fn from_model(
        wi: workspace_integrations::Model,
        integration: Option<integrations::Model>,
    ) -> Self {
        Self {
            id: wi.id,
            integration_id: wi.integration_id,
            workspace_id: wi.workspace_id,
            actor_id: wi.actor_id,
            metadata: wi.metadata,
            config: wi.config,
            integration: integration.map(IntegrationResponse::from_model),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GithubRepoSyncResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    /// Nombre legible del proyecto — espeja `project_name` del Django serializer.
    pub project_name: String,
    /// Identificador corto del proyecto (e.g. "PLANE") — espeja `project_identifier`.
    pub project_identifier: String,
    pub repo_id: String,
    pub repo_full_name: String,
    pub repo_name: String,
    pub repo_owner: String,
    pub sync_direction: String,
    pub issue_open_state: Option<String>,
    pub issue_closed_state: Option<String>,
    /// Timestamp de creación ISO 8601 — espeja `created_at` del Django serializer.
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PrStateMappingResponse {
    pub id: Uuid,
    pub github_pr_state: String,
    pub project_id: Uuid,
    pub state_id: Uuid,
    pub prevent_regression: bool,
    pub workspace_integration_id: Uuid,
}

impl PrStateMappingResponse {
    pub fn from_model(m: db_githubprstatemapping::Model) -> Self {
        Self {
            id: m.id,
            github_pr_state: m.github_pr_state,
            project_id: m.project_id,
            state_id: m.state_id,
            prevent_regression: m.prevent_regression,
            workspace_integration_id: m.workspace_integration_id,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserGithubConnectionResponse {
    pub id: Uuid,
    pub github_user_id: String,
    pub github_username: String,
    pub github_avatar_url: String,
    pub created: bool,
}

// ── Request DTOs ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWorkspaceIntegrationRequest {
    pub integration: Uuid,
    pub metadata: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkspaceIntegrationRequest {
    pub metadata: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProviderInstallRequest {
    // GitHub
    pub installation_id: Option<String>,
    // GitLab / Slack
    pub code: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GithubRepoSyncCreateRequest {
    pub repo_id: serde_json::Value,
    pub repo_full_name: Option<String>,
    pub project_id: Uuid,
    pub sync_direction: Option<String>,
    pub issue_open_state: Option<String>,
    pub issue_closed_state: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PrStateMappingCreateRequest {
    pub github_pr_state: String,
    pub project_id: Uuid,
    pub state_id: Uuid,
    pub prevent_regression: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UserGithubCallbackRequest {
    pub code: String,
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ExternalReposQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GithubCallbackQuery {
    pub installation_id: Option<String>,
    pub setup_action: Option<String>,
    pub state: Option<String>, // workspace_slug
}
