// src/routes/integrations.rs
//! Endpoints de integraciones: GitHub, GitLab, Slack.
//!
//! Endpoints implementados:
//!   GET    /api/integrations/
//!   GET    /api/github/callback/                              (sin auth)
//!   POST   /api/auth/github/user-callback/                   (auth requerida)
//!   GET    /api/workspaces/{slug}/workspace-integrations/
//!   POST   /api/workspaces/{slug}/workspace-integrations/
//!   GET    /api/workspaces/{slug}/workspace-integrations/{pk}/
//!   PATCH  /api/workspaces/{slug}/workspace-integrations/{pk}/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/{pk}/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/{provider}/provider/
//!   POST   /api/workspaces/{slug}/workspace-integrations/{provider}/install/
//!   GET    /api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/
//!   GET    /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
//!   POST   /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/
//!   GET    /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
//!   POST   /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/

use anyhow::Context as _;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, QueryFilter,
    QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{api_key::ApiKeyUser, extractors::WorkspaceMemberGuard, permissions::require_workspace_admin},
    entities::{
        api_tokens, db_githubprstatemapping, github_repositories, github_repository_syncs,
        integrations, user_github_connections, workspace_integrations, workspace_members,
    },
    error::AppError,
    utils::{
        github_app::get_installation_access_token,
        instance_config::get_instance_config,
        oauth_popup::{postmessage_html, OAuthMessageType},
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

// ── Helpers internos ─────────────────────────────────────────────────────────

/// Busca o crea un api_token para (user, workspace).
/// Replica el comportamiento de `APIToken.objects.get_or_create` de Django.
///
/// La operación se ejecuta dentro de una transacción con nivel SERIALIZABLE
/// para evitar la race condition TOCTOU (check-then-insert) que existía antes.
/// Si dos requests concurrentes pasan el SELECT vacío al mismo tiempo, solo
/// una INSERT tendrá éxito; la otra leerá el token recién creado.
async fn get_or_create_api_token(
    state: &AppState,
    user_id: Uuid,
    workspace_id: Uuid,
    label: &str,
) -> Result<api_tokens::Model, AppError> {
    let label = label.to_owned();

    let token = state
        .db
        .transaction_with_config::<_, api_tokens::Model, AppError>(
            |txn| {
                let label = label.clone();
                Box::pin(async move {
                    // SELECT dentro de la transacción
                    if let Some(token) = api_tokens::Entity::find()
                        .filter(api_tokens::Column::UserId.eq(user_id))
                        .filter(api_tokens::Column::WorkspaceId.eq(workspace_id))
                        .filter(api_tokens::Column::IsActive.eq(true))
                        .filter(api_tokens::Column::DeletedAt.is_null())
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?
                    {
                        return Ok(token);
                    }

                    // INSERT solo si no existía dentro del mismo snapshot
                    let raw = Uuid::new_v4().as_simple().to_string();
                    let new_token = api_tokens::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        token: Set(raw),
                        label: Set(label),
                        user_type: Set(1),
                        user_id: Set(user_id),
                        workspace_id: Set(Some(workspace_id)),
                        description: Set(String::new()),
                        is_active: Set(true),
                        is_service: Set(false),
                        allowed_rate_limit: Set("default".to_owned()),
                        ..Default::default()
                    };

                    new_token.insert(txn).await.map_err(AppError::Database)
                })
            },
            Some(IsolationLevel::Serializable),
            None,
        )
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    Ok(token)
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
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
    fn from_model(m: integrations::Model) -> Self {
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

#[derive(Debug, Serialize)]
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
    fn from_model(
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

#[derive(Debug, Serialize)]
pub struct GithubRepoSyncResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub repo_id: String,
    pub repo_full_name: String,
    pub repo_name: String,
    pub repo_owner: String,
    pub sync_direction: String,
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

#[derive(Debug, Serialize)]
pub struct PrStateMappingResponse {
    pub id: Uuid,
    pub github_pr_state: String,
    pub project_id: Uuid,
    pub state_id: Uuid,
    pub prevent_regression: bool,
    pub workspace_integration_id: Uuid,
}

impl PrStateMappingResponse {
    fn from_model(m: db_githubprstatemapping::Model) -> Self {
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UserGithubCallbackRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct UserGithubConnectionResponse {
    pub id: Uuid,
    pub github_user_id: String,
    pub github_username: String,
    pub github_avatar_url: String,
    pub created: bool,
}

#[derive(Debug, Deserialize)]
pub struct GithubReposQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GithubCallbackQuery {
    pub installation_id: Option<String>,
    pub setup_action: Option<String>,
    pub state: Option<String>, // workspace_slug
}

/// Valores de `github_pr_state` permitidos (enum Postgres).
const VALID_PR_STATES: &[&str] = &[
    "draft_open",
    "open",
    "review_requested",
    "ready_for_merge",
    "merged",
    "closed",
];

// ── 1. GET /api/integrations/ ─────────────────────────────────────────────────

/// Lista todas las integraciones disponibles (GitHub, GitLab, Slack).
#[utoipa::path(
    get,
    path = "/api/integrations/",
    tag = "Integrations",
    responses(
        (status = 200, description = "Lista de integraciones"),
        (status = 401, description = "No autenticado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_integrations(
    State(state): State<AppState>,
    _auth: ApiKeyUser, // valida que hay sesión activa; no necesita workspace
) -> Result<Json<Vec<IntegrationResponse>>, AppError> {
    let rows = integrations::Entity::find()
        .active()
        .order_by_asc(integrations::Column::Title)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(IntegrationResponse::from_model).collect()))
}

// ── 2. GET /api/github/callback/ (sin auth) ───────────────────────────────────

/// Callback de GitHub App — Setup URL registrada en la GitHub App.
/// No requiere autenticación (GitHub redirige el popup aquí directamente).
#[utoipa::path(
    get,
    path = "/api/github/callback/",
    tag = "Integrations",
    responses(
        (status = 200, description = "HTML de cierre de popup"),
    )
)]
pub async fn github_app_callback(
    State(state): State<AppState>,
    Query(params): Query<GithubCallbackQuery>,
) -> axum::response::Html<String> {
    let (Some(installation_id), Some(workspace_slug)) =
        (params.installation_id.as_deref(), params.state.as_deref())
    else {
        return postmessage_html(
            false,
            OAuthMessageType::GithubIntegration,
            Some("Missing installation_id or workspace context."),
        );
    };

    let setup_action = params
        .setup_action
        .as_deref()
        .unwrap_or("install")
        .to_owned();

    match github_app_callback_inner(&state, installation_id, &setup_action, workspace_slug).await {
        Ok(()) => postmessage_html(true, OAuthMessageType::GithubIntegration, None),
        Err(e) => {
            tracing::error!(error = %e, "GithubAppCallback failed");
            postmessage_html(
                false,
                OAuthMessageType::GithubIntegration,
                Some("Installation failed. Please try again."),
            )
        }
    }
}

async fn github_app_callback_inner(
    state: &AppState,
    installation_id: &str,
    setup_action: &str,
    workspace_slug: &str,
) -> anyhow::Result<()> {
    use crate::entities::workspaces;

    let workspace = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(workspace_slug))
        .one(&state.db)
        .await?
        .context("Workspace not found")?;

    let integration = integrations::Entity::find()
        .active()
        .filter(integrations::Column::Provider.eq("github"))
        .one(&state.db)
        .await?
        .context("GitHub integration not found in DB")?;

    // Este callback es sin auth — usar el primer workspace admin como actor
    let admin_member = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace.id))
        .filter(workspace_members::Column::Role.gte(ROLE_ADMIN))
        .filter(workspace_members::Column::IsActive.eq(true))
        .order_by_asc(workspace_members::Column::CreatedAt)
        .one(&state.db)
        .await?
        .context("No workspace admin found — installation_id cannot be persisted")?;

    let actor_id = admin_member.member_id;

    let api_token =
        get_or_create_api_token(state, actor_id, workspace.id, "GitHub Integration Token").await?;

    let metadata = serde_json::json!({
        "installation_id": installation_id,
        "setup_action": setup_action,
    });
    let config = serde_json::json!({ "installation_id": installation_id });

    // UPSERT: update_or_create
    let existing = workspace_integrations::Entity::find()
        .filter(workspace_integrations::Column::WorkspaceId.eq(workspace.id))
        .filter(workspace_integrations::Column::IntegrationId.eq(integration.id))
        .filter(workspace_integrations::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?;

    if let Some(wi) = existing {
        let mut am: workspace_integrations::ActiveModel = wi.into();
        am.metadata = Set(metadata);
        am.config = Set(config);
        am.actor_id = Set(actor_id);
        am.api_token_id = Set(api_token.id);
        am.update(&state.db).await?;
    } else {
        workspace_integrations::ActiveModel {
            id: Set(Uuid::new_v4()),
            workspace_id: Set(workspace.id),
            integration_id: Set(integration.id),
            actor_id: Set(actor_id),
            api_token_id: Set(api_token.id),
            metadata: Set(metadata),
            config: Set(config),
            ..Default::default()
        }
        .insert(&state.db)
        .await?;
    }

    Ok(())
}

// ── 3. POST /api/auth/github/user-callback/ ───────────────────────────────────

/// Intercambia un OAuth code de GitHub por un token personal de usuario.
#[utoipa::path(
    post,
    path = "/api/auth/github/user-callback/",
    tag = "Integrations",
    responses(
        (status = 201, description = "Conexión creada"),
        (status = 200, description = "Conexión actualizada"),
        (status = 400, description = "Error de validación"),
        (status = 502, description = "Error al contactar GitHub"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn github_user_callback(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<UserGithubCallbackRequest>,
) -> Result<(StatusCode, Json<UserGithubConnectionResponse>), AppError> {
    let client_id = get_instance_config(&state, "GITHUB_CLIENT_ID")
        .await?
        .ok_or_else(|| AppError::BadRequest("GitHub OAuth is not configured".into()))?;

    let client_secret = get_instance_config(&state, "GITHUB_CLIENT_SECRET")
        .await?
        .ok_or_else(|| AppError::BadRequest("GitHub OAuth is not configured".into()))?;

    // Intercambiar code por access_token
    let token_resp: reqwest::Response = state
        .http
        .post("https://github.com/login/oauth/access_token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", body.code.as_str()),
        ])
        .header("Accept", "application/json")
        .send()
        .await
        .context("Failed to contact GitHub token endpoint")
        .map_err(AppError::Internal)?;

    if !token_resp.status().is_success() {
        return Err(AppError::BadRequest(
            "Failed to exchange GitHub authorization code".into(),
        ));
    }

    let token_data: serde_json::Value = token_resp
        .json()
        .await
        .context("GitHub token response is not JSON")
        .map_err(AppError::Internal)?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| {
            let desc = token_data["error_description"]
                .as_str()
                .unwrap_or("GitHub did not return an access token");
            AppError::BadRequest(desc.to_owned())
        })?
        .to_owned();

    // Obtener perfil del usuario en GitHub
    let user_resp = state
        .http
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Accept", "application/json")
        .header("User-Agent", "plane-api-rust/0.1")
        .send()
        .await
        .context("Failed to fetch GitHub user profile")
        .map_err(AppError::Internal)?;

    if !user_resp.status().is_success() {
        return Err(AppError::BadRequest(
            "Failed to fetch GitHub user profile".into(),
        ));
    }

    let github_user: serde_json::Value = user_resp
        .json()
        .await
        .context("GitHub user response is not JSON")
        .map_err(AppError::Internal)?;

    let github_user_id = github_user["id"]
        .as_i64()
        .map(|id| id.to_string())
        .ok_or_else(|| AppError::BadRequest("GitHub returned incomplete user profile".into()))?;

    let github_username = github_user["login"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("GitHub returned incomplete user profile".into()))?
        .to_owned();

    let github_avatar_url = github_user["avatar_url"]
        .as_str()
        .unwrap_or("")
        .to_owned();

    let user_id = guard.user.id;

    // Upsert user_github_connections
    let existing = user_github_connections::Entity::find()
        .filter(user_github_connections::Column::UserId.eq(user_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // SECURITY: `access_token` se almacena en texto plano para mantener
    // interoperabilidad con la API Django que comparte esta tabla.
    // El modelo Django tiene el mismo comportamiento ("encrypted in production ideally").
    // Corrección pendiente: migración coordinada a cifrado simétrico (ej. Fernet/AES-GCM)
    // en ambos servicios simultáneamente. Ver docs/api-rust/SECURITY.md.
    let (conn, created) = if let Some(conn) = existing {
        let mut am: user_github_connections::ActiveModel = conn.into();
        am.github_user_id = Set(github_user_id);
        am.github_username = Set(github_username);
        am.github_avatar_url = Set(github_avatar_url);
        am.access_token = Set(access_token);
        (am.update(&state.db).await.map_err(AppError::Database)?, false)
    } else {
        let new_conn = user_github_connections::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            github_user_id: Set(github_user_id),
            github_username: Set(github_username),
            github_avatar_url: Set(github_avatar_url),
            access_token: Set(access_token),
            ..Default::default()
        };
        (new_conn.insert(&state.db).await.map_err(AppError::Database)?, true)
    };

    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    Ok((
        status,
        Json(UserGithubConnectionResponse {
            id: conn.id,
            github_user_id: conn.github_user_id,
            github_username: conn.github_username,
            github_avatar_url: conn.github_avatar_url,
            created,
        }),
    ))
}

// ── 4. GET /workspaces/{slug}/workspace-integrations/ ─────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/",
    tag = "Integrations",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Lista de integraciones del workspace"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin permiso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_workspace_integrations(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<Vec<WorkspaceIntegrationResponse>>, AppError> {
    require_workspace_admin(&guard.member)?;

    let rows = workspace_integrations::Entity::find()
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batch-fetch todas las integraciones referenciadas en una sola query
    // evita N+1: antes se hacía una query por cada workspace_integration.
    let integration_ids: Vec<Uuid> = rows.iter().map(|wi| wi.integration_id).collect();
    let integrations_map: std::collections::HashMap<Uuid, integrations::Model> =
        integrations::Entity::find()
            .filter(integrations::Column::Id.is_in(integration_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|i| (i.id, i))
            .collect();

    let result = rows
        .into_iter()
        .map(|wi| {
            let integration = integrations_map.get(&wi.integration_id).cloned();
            WorkspaceIntegrationResponse::from_model(wi, integration)
        })
        .collect();

    Ok(Json(result))
}

// ── 5. POST /workspaces/{slug}/workspace-integrations/ ───────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-integrations/",
    tag = "Integrations",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Integración instalada"),
        (status = 400, description = "Error de validación"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin permiso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_workspace_integration(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateWorkspaceIntegrationRequest>,
) -> Result<(StatusCode, Json<WorkspaceIntegrationResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

    let integration = integrations::Entity::find_by_id(body.integration)
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let api_token = get_or_create_api_token(
        &state,
        guard.user.id,
        guard.workspace.id,
        &format!("{} Integration Token", integration.title),
    )
    .await?;

    // Verificar que no exista ya
    let existing = workspace_integrations::Entity::find()
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(workspace_integrations::Column::IntegrationId.eq(integration.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Integration already installed".into(),
        ));
    }

    let wi = workspace_integrations::ActiveModel {
        id: Set(Uuid::new_v4()),
        workspace_id: Set(guard.workspace.id),
        integration_id: Set(integration.id),
        actor_id: Set(guard.user.id),
        api_token_id: Set(api_token.id),
        metadata: Set(body.metadata.unwrap_or(serde_json::json!({}))),
        config: Set(body.config.unwrap_or(serde_json::json!({}))),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(WorkspaceIntegrationResponse::from_model(wi, Some(integration))),
    ))
}

// ── 6. GET /workspaces/{slug}/workspace-integrations/{pk}/ ───────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/{pk}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 200, description = "Detalle de la integración"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_workspace_integration(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<WorkspaceIntegrationResponse>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wi = workspace_integrations::Entity::find_by_id(pk)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let integration = integrations::Entity::find_by_id(wi.integration_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(WorkspaceIntegrationResponse::from_model(wi, integration)))
}

// ── 7. PATCH /workspaces/{slug}/workspace-integrations/{pk}/ ─────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/workspace-integrations/{pk}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 200, description = "Integración actualizada"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_workspace_integration(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateWorkspaceIntegrationRequest>,
) -> Result<Json<WorkspaceIntegrationResponse>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wi = workspace_integrations::Entity::find_by_id(pk)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let integration = integrations::Entity::find_by_id(wi.integration_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let mut am: workspace_integrations::ActiveModel = wi.into();
    if let Some(metadata) = body.metadata {
        am.metadata = Set(metadata);
    }
    if let Some(config) = body.config {
        am.config = Set(config);
    }
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(WorkspaceIntegrationResponse::from_model(updated, integration)))
}

// ── 8. DELETE /workspaces/{slug}/workspace-integrations/{pk}/ ────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/workspace-integrations/{pk}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 204, description = "Eliminada"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_workspace_integration(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let wi = workspace_integrations::Entity::find_by_id(pk)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: workspace_integrations::ActiveModel = wi.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── 9. DELETE /workspaces/{slug}/workspace-integrations/{provider}/provider/ ──

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/workspace-integrations/{provider}/provider/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("provider" = String, Path, description = "Provider (github|gitlab|slack)"),
    ),
    responses(
        (status = 204, description = "Eliminada"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_workspace_integration_by_provider(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, provider)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let wi = workspace_integrations::Entity::find()
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .inner_join(integrations::Entity)
        .filter(integrations::Column::Provider.eq(&provider))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: workspace_integrations::ActiveModel = wi.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── 10. POST /workspaces/{slug}/workspace-integrations/{provider}/install/ ────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-integrations/{provider}/install/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("provider" = String, Path, description = "Provider (github|gitlab|slack)"),
    ),
    responses(
        (status = 201, description = "Integración instalada"),
        (status = 200, description = "Integración actualizada"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn provider_install(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, provider)): Path<(String, String)>,
    Json(body): Json<ProviderInstallRequest>,
) -> Result<(StatusCode, Json<WorkspaceIntegrationResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

    let integration = integrations::Entity::find()
        .active()
        .filter(integrations::Column::Provider.eq(&provider))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let (metadata, config) = match provider.as_str() {
        "github" => {
            let installation_id = body.installation_id.ok_or_else(|| {
                AppError::BadRequest("installation_id is required for GitHub integration".into())
            })?;
            let m = serde_json::json!({ "installation_id": installation_id });
            let c = serde_json::json!({ "installation_id": installation_id });
            (m, c)
        }
        "gitlab" => {
            let code = body
                .code
                .ok_or_else(|| AppError::BadRequest("code is required for GitLab integration".into()))?;
            (serde_json::json!({ "code": code }), serde_json::json!({}))
        }
        "slack" => {
            let code = body
                .code
                .ok_or_else(|| AppError::BadRequest("code is required for Slack integration".into()))?;

            // `?` propaga errores de OAuth al cliente (400/502) en lugar de
            // devolver 201 con metadata incompleta de forma silenciosa.
            build_slack_metadata(&state, &code).await?
        }
        _ => return Err(AppError::BadRequest(format!("Unknown provider: {provider}"))),
    };

    let api_token = get_or_create_api_token(
        &state,
        guard.user.id,
        guard.workspace.id,
        &format!("{} Integration Token", integration.title),
    )
    .await?;

    // get_or_create con upsert en caso de reinsatalación
    let existing = workspace_integrations::Entity::find()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(workspace_integrations::Column::IntegrationId.eq(integration.id))
        .filter(workspace_integrations::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let (wi, created) = if let Some(wi) = existing {
        let mut am: workspace_integrations::ActiveModel = wi.into();
        am.metadata = Set(metadata);
        am.config = Set(config);
        (am.update(&state.db).await.map_err(AppError::Database)?, false)
    } else {
        let new_wi = workspace_integrations::ActiveModel {
            id: Set(Uuid::new_v4()),
            workspace_id: Set(guard.workspace.id),
            integration_id: Set(integration.id),
            actor_id: Set(guard.user.id),
            api_token_id: Set(api_token.id),
            metadata: Set(metadata),
            config: Set(config),
            ..Default::default()
        };
        (new_wi.insert(&state.db).await.map_err(AppError::Database)?, true)
    };

    let status = if created { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(WorkspaceIntegrationResponse::from_model(wi, Some(integration)))))
}

/// Intercambia el code de Slack por access_token y construye metadata/config.
///
/// # Degradación controlada vs. error real
/// - Si `SLACK_CLIENT_ID`/`SLACK_CLIENT_SECRET` no están configurados en DB ni
///   en env, devuelve `Ok((code_json, {}))` — instalación parcial intencional.
/// - Si las credenciales sí están configuradas pero el exchange falla (red,
///   code inválido, respuesta de Slack con `ok: false`), devuelve `Err` para
///   que `provider_install` pueda retornar 400/502 al cliente.
///
/// Antipatrón corregido: la versión anterior absorbía todos los errores como
/// fallback silencioso, por lo que una instalación de Slack fallida devolvía
/// 201 al cliente con metadata incompleta sin ningún indicador de fallo.
async fn build_slack_metadata(
    state: &AppState,
    code: &str,
) -> Result<(serde_json::Value, serde_json::Value), AppError> {
    // Sin credenciales: degradación intencional — no es un error.
    let client_id = match get_instance_config(state, "SLACK_CLIENT_ID").await? {
        Some(v) if !v.is_empty() => v,
        _ => return Ok((serde_json::json!({ "code": code }), serde_json::json!({}))),
    };
    let client_secret = match get_instance_config(state, "SLACK_CLIENT_SECRET").await? {
        Some(v) if !v.is_empty() => v,
        _ => return Ok((serde_json::json!({ "code": code }), serde_json::json!({}))),
    };

    // Con credenciales configuradas, fallos de red son errores reales.
    let resp = state
        .http
        .post("https://slack.com/api/oauth.v2.access")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
        ])
        .send()
        .await
        .context("Error al contactar Slack OAuth endpoint")
        .map_err(AppError::Internal)?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(AppError::BadRequest(format!(
            "Slack OAuth token exchange failed: HTTP {status}"
        )));
    }

    let slack_data: serde_json::Value = resp
        .json()
        .await
        .context("Slack OAuth response is not valid JSON")
        .map_err(AppError::Internal)?;

    // Slack devuelve siempre HTTP 200; el campo `ok` indica el resultado real.
    if !slack_data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let err_msg = slack_data
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_error");
        return Err(AppError::BadRequest(format!(
            "Slack OAuth error: {err_msg}"
        )));
    }

    let config = serde_json::json!({
        "access_token": slack_data.get("access_token"),
        "team_id":      slack_data.get("team").and_then(|t| t.get("id")),
        "team_name":    slack_data.get("team").and_then(|t| t.get("name")),
    });

    Ok((slack_data, config))
}

// ── 11. GET /workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/ ─

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
        ("page" = Option<u32>, Query, description = "Página (default 1)"),
        ("per_page" = Option<u32>, Query, description = "Repos por página (default 30)"),
    ),
    responses(
        (status = 200, description = "Lista de repositorios GitHub"),
        (status = 400, description = "Error de configuración"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_github_repositories(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id)): Path<(String, Uuid)>,
    Query(params): Query<GithubReposQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_workspace_admin(&guard.member)?;

    let wi = workspace_integrations::Entity::find_by_id(wi_id)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let installation_id = wi.metadata
        .get("installation_id")
        .and_then(|v| v.as_str())
        .map(str::to_owned);

    let (github_token, is_installation_token) = if let Some(ref iid) = installation_id {
        match get_installation_access_token(&state, iid).await? {
            Some(token) => (token, true),
            None => {
                return Err(AppError::BadRequest(
                    "GitHub App is not configured. Set GITHUB_APP_ID and GITHUB_APP_PRIVATE_KEY."
                        .into(),
                ));
            }
        }
    } else {
        return Err(AppError::BadRequest(
            "GitHub integration has no installation_id".into(),
        ));
    };

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(30).min(100);

    // Installation token → /installation/repositories; PAT → /user/repos
    let (api_url, query_params): (String, Vec<(&str, String)>) = if is_installation_token {
        (
            "https://api.github.com/installation/repositories".into(),
            vec![
                ("per_page", per_page.to_string()),
                ("page", page.to_string()),
            ],
        )
    } else {
        (
            "https://api.github.com/user/repos".into(),
            vec![
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
                ("sort", "updated".into()),
                ("type", "all".into()),
            ],
        )
    };

    // reqwest 0.13 con default-features=false no expone .query() en RequestBuilder;
    // se construye la query string manualmente — los valores son numéricos o ASCII simple.
    let api_url_with_params = {
        let qs: String = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            api_url.clone()
        } else {
            format!("{}?{}", api_url, qs)
        }
    };

    let resp: reqwest::Response = state
        .http
        .get(&api_url_with_params)
        .header("Authorization", format!("Bearer {github_token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "plane-api-rust/0.1")
        .send()
        .await
        .context("Failed to contact GitHub API")
        .map_err(AppError::Internal)?;

    if !resp.status().is_success() {
        return Err(AppError::BadRequest(
            "Failed to fetch repositories from GitHub".into(),
        ));
    }

    let payload: serde_json::Value = resp
        .json::<serde_json::Value>()
        .await
        .context("GitHub repos response is not JSON")
        .map_err(AppError::Internal)?;

    // Usar `if let` evita el `unwrap()` que hubiera pánico si `payload` cambia
    // de tipo entre la comprobación `is_array()` y el acceso — antipatrón TOCTOU
    // en memoria (aunque aquí no hay concurrencia, es mala práctica de Rust).
    let (repos, total_count): (Vec<serde_json::Value>, usize) = if let Some(arr) = payload.as_array() {
        let len = arr.len();
        (arr.clone(), len)
    } else {
        let repos: Vec<serde_json::Value> = payload
            .get("repositories")
            .and_then(|r: &serde_json::Value| r.as_array())
            .cloned()
            .unwrap_or_default();
        let total = payload
            .get("total_count")
            .and_then(|t: &serde_json::Value| t.as_u64())
            .unwrap_or(repos.len() as u64) as usize;
        (repos, total)
    };

    let manage_url = installation_id.as_deref().map(|iid| {
        format!("https://github.com/settings/installations/{iid}")
    });

    let mapped: Vec<serde_json::Value> = repos
        .iter()
        .map(|repo: &serde_json::Value| {
            serde_json::json!({
                "id": repo["id"].as_i64().unwrap_or(0).to_string(),
                "full_name": repo["full_name"],
                "name": repo["name"],
                "owner": repo["owner"]["login"],
                "description": repo.get("description").and_then(|d: &serde_json::Value| d.as_str()).unwrap_or(""),
                "private": repo["private"],
                "url": repo["html_url"],
                "issues_count": repo.get("open_issues_count").and_then(|c: &serde_json::Value| c.as_u64()).unwrap_or(0),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "repositories": mapped,
        "total_count": total_count,
        "page": page,
        "is_installation_token": is_installation_token,
        "manage_installation_url": manage_url,
    })))
}

// ── 12. GET /workspaces/{slug}/workspace-integrations/github/repo-syncs/ ──────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/github/repo-syncs/",
    tag = "Integrations",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Lista de repo syncs"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_github_repo_syncs(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<Vec<GithubRepoSyncResponse>>, AppError> {
    require_workspace_admin(&guard.member)?;

    // Obtener workspace_integration de GitHub para este workspace
    let wi = workspace_integrations::Entity::find()
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .inner_join(integrations::Entity)
        .filter(integrations::Column::Provider.eq("github"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(wi) = wi else {
        return Ok(Json(vec![]));
    };

    let syncs = github_repository_syncs::Entity::find()
        .active()
        .filter(github_repository_syncs::Column::WorkspaceIntegrationId.eq(wi.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batch-fetch todos los repositorios referenciados en una sola query.
    // Antes se hacía una query por sync dentro del loop → N+1 antipattern.
    let repo_ids: Vec<Uuid> = syncs.iter().map(|s| s.repository_id).collect();
    let repos_map: std::collections::HashMap<Uuid, github_repositories::Model> =
        github_repositories::Entity::find()
            .filter(github_repositories::Column::Id.is_in(repo_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|r| (r.id, r))
            .collect();

    let result = syncs
        .into_iter()
        .map(|sync| {
            let credentials = &sync.credentials;
            let sync_direction = credentials
                .get("sync_direction")
                .and_then(|v| v.as_str())
                .unwrap_or("bidirectional")
                .to_owned();
            let issue_open_state = credentials
                .get("issue_open_state")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
            let issue_closed_state = credentials
                .get("issue_closed_state")
                .and_then(|v| v.as_str())
                .map(str::to_owned);

            let (repo_id, repo_name, repo_owner) =
                if let Some(r) = repos_map.get(&sync.repository_id) {
                    (r.repository_id.to_string(), r.name.clone(), r.owner.clone())
                } else {
                    (String::new(), String::new(), String::new())
                };

            GithubRepoSyncResponse {
                id: sync.id,
                project_id: sync.project_id,
                repo_id,
                repo_full_name: format!("{repo_owner}/{repo_name}"),
                repo_name,
                repo_owner,
                sync_direction,
                issue_open_state,
                issue_closed_state,
            }
        })
        .collect();

    Ok(Json(result))
}

// ── 13. POST /workspaces/{slug}/workspace-integrations/github/repo-syncs/ ────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-integrations/github/repo-syncs/",
    tag = "Integrations",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Repo sync creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_github_repo_sync(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<GithubRepoSyncCreateRequest>,
) -> Result<(StatusCode, Json<GithubRepoSyncResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

    // Validar repo_id numérico
    let repo_id_int: i64 = match &body.repo_id {
        serde_json::Value::Number(n) => n.as_i64().ok_or_else(|| {
            AppError::BadRequest("repo_id must be a numeric GitHub repository ID".into())
        })?,
        serde_json::Value::String(s) => s.parse::<i64>().map_err(|_| {
            AppError::BadRequest("repo_id must be a numeric GitHub repository ID".into())
        })?,
        _ => return Err(AppError::BadRequest("repo_id is required".into())),
    };

    let repo_full_name = body.repo_full_name.as_deref().unwrap_or("");
    let parts: Vec<&str> = repo_full_name.splitn(2, '/').collect();
    let (repo_owner, repo_name) = if parts.len() == 2 {
        (parts[0].to_owned(), parts[1].to_owned())
    } else {
        (String::new(), repo_full_name.to_owned())
    };

    // Obtener workspace_integration de GitHub
    let wi = workspace_integrations::Entity::find()
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .inner_join(integrations::Entity)
        .filter(integrations::Column::Provider.eq("github"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            AppError::BadRequest(
                "GitHub integration not installed for this workspace".into(),
            )
        })?;

    // Buscar GithubRepository — incluyendo soft-deleted para no violar unique constraint
    let existing_repo = github_repositories::Entity::find()
        .filter(github_repositories::Column::RepositoryId.eq(repo_id_int))
        .filter(github_repositories::Column::ProjectId.eq(body.project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let repo = match existing_repo {
        None => {
            github_repositories::ActiveModel {
                id: Set(Uuid::new_v4()),
                repository_id: Set(repo_id_int),
                project_id: Set(body.project_id),
                workspace_id: Set(guard.workspace.id),
                name: Set(repo_name.clone()),
                owner: Set(repo_owner.clone()),
                url: Set(Some(format!("https://github.com/{repo_full_name}"))),
                config: Set(serde_json::json!({})),
                ..Default::default()
            }
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?
        }
        Some(r) if r.deleted_at.is_some() => {
            // Resucitar soft-deleted
            let mut am: github_repositories::ActiveModel = r.into();
            am.deleted_at = Set(None);
            am.name = Set(repo_name.clone());
            am.owner = Set(repo_owner.clone());
            am.url = Set(Some(format!("https://github.com/{repo_full_name}")));
            am.update(&state.db).await.map_err(AppError::Database)?
        }
        Some(r) => r,
    };

    let credentials = serde_json::json!({
        "sync_direction": body.sync_direction.as_deref().unwrap_or("bidirectional"),
        "issue_open_state": body.issue_open_state,
        "issue_closed_state": body.issue_closed_state,
    });

    // Verificar sync existente (incluyendo soft-deleted)
    let existing_sync = github_repository_syncs::Entity::find()
        .filter(github_repository_syncs::Column::RepositoryId.eq(repo.id))
        .filter(github_repository_syncs::Column::ProjectId.eq(body.project_id))
        .filter(github_repository_syncs::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let sync = match existing_sync {
        Some(s) if s.deleted_at.is_none() => {
            return Err(AppError::BadRequest(
                "A sync for this project and repository already exists".into(),
            ));
        }
        Some(s) => {
            // Resucitar soft-deleted sync
            let mut am: github_repository_syncs::ActiveModel = s.into();
            am.deleted_at = Set(None);
            am.actor_id = Set(guard.user.id);
            am.workspace_integration_id = Set(wi.id);
            am.credentials = Set(credentials.clone());
            am.update(&state.db).await.map_err(AppError::Database)?
        }
        None => {
            github_repository_syncs::ActiveModel {
                id: Set(Uuid::new_v4()),
                repository_id: Set(repo.id),
                project_id: Set(body.project_id),
                workspace_id: Set(guard.workspace.id),
                actor_id: Set(guard.user.id),
                workspace_integration_id: Set(wi.id),
                credentials: Set(credentials.clone()),
                ..Default::default()
            }
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?
        }
    };

    // Registrar webhook en GitHub — best-effort, no bloquea la respuesta
    // silence-patterns-ok: el registro es opcional; si falla se loggea y se continúa
    if let Err(e) = register_github_webhook(&state, &wi, &repo_owner, &repo_name).await {
        tracing::warn!(
            repo_sync_id = %sync.id,
            "Failed to register GitHub webhook: {e}"
        );
    }

    let sync_direction = credentials
        .get("sync_direction")
        .and_then(|v| v.as_str())
        .unwrap_or("bidirectional")
        .to_owned();

    Ok((
        StatusCode::CREATED,
        Json(GithubRepoSyncResponse {
            id: sync.id,
            project_id: sync.project_id,
            repo_id: repo.repository_id.to_string(),
            repo_full_name: format!("{}/{}", repo.owner, repo.name),
            repo_name: repo.name,
            repo_owner: repo.owner,
            sync_direction,
            issue_open_state: credentials
                .get("issue_open_state")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            issue_closed_state: credentials
                .get("issue_closed_state")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        }),
    ))
}

/// Registra el webhook de Plane en el repositorio GitHub.
/// [Fix webhook best-effort] Nunca debe fallar el handler padre.
async fn register_github_webhook(
    state: &AppState,
    wi: &workspace_integrations::Model,
    owner: &str,
    repo_name: &str,
) -> anyhow::Result<()> {
    let installation_id = wi
        .metadata
        .get("installation_id")
        .and_then(|v| v.as_str())
        .context("No installation_id en workspace_integration")?;

    let token = get_installation_access_token(state, installation_id)
        .await?
        .context("No se pudo obtener installation token")?;

    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET").unwrap_or_default();
    let web_url = std::env::var("WEB_URL").context("WEB_URL no configurado")?;

    let resp = state
        .http
        .post(format!(
            "https://api.github.com/repos/{owner}/{repo_name}/hooks"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "plane-api-rust/0.1")
        .json(&serde_json::json!({
            "name": "web",
            "config": {
                "url": format!("{}/api/github-webhook/", web_url.trim_end_matches('/')),
                "content_type": "json",
                "secret": webhook_secret,
            },
            "events": ["issues", "pull_request", "issue_comment"],
            "active": true,
        }))
        .send()
        .await
        .context("Error al llamar GitHub webhooks API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GitHub webhook registration failed: {status} — {body}");
    }

    Ok(())
}

// ── 14. DELETE /workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/ ─

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "RepoSync ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_github_repo_sync(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let sync = github_repository_syncs::Entity::find_by_id(pk)
        .active()
        .filter(github_repository_syncs::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let repo_id = sync.repository_id;

    // Ambos soft-deletes (sync + repo huérfano) deben ser atómicos.
    // Sin transacción, un fallo en el segundo update deja el sync eliminado
    // pero el repo activo → estado inconsistente en la base de datos.
    state
        .db
        .transaction::<_, (), AppError>(|txn| {
            Box::pin(async move {
                let now: chrono::DateTime<chrono::FixedOffset> =
                    chrono::Utc::now().into();

                // Soft-delete sync
                let mut am: github_repository_syncs::ActiveModel = sync.into();
                am.deleted_at = Set(Some(now));
                am.update(txn).await.map_err(AppError::Database)?;

                // Soft-delete repo huérfano
                if let Some(repo) = github_repositories::Entity::find_by_id(repo_id)
                    .one(txn)
                    .await
                    .map_err(AppError::Database)?
                {
                    let mut ram: github_repositories::ActiveModel = repo.into();
                    ram.deleted_at = Set(Some(now));
                    ram.update(txn).await.map_err(AppError::Database)?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ── 15. GET /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/ ─

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 200, description = "Lista de mappings"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_pr_state_mappings(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PrStateMappingResponse>>, AppError> {
    require_workspace_admin(&guard.member)?;

    let mappings = db_githubprstatemapping::Entity::find()
        .active()
        .filter(db_githubprstatemapping::Column::WorkspaceIntegrationId.eq(wi_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        mappings.into_iter().map(PrStateMappingResponse::from_model).collect(),
    ))
}

// ── 16. POST /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/ ─

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 201, description = "Mapping creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_pr_state_mapping(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id)): Path<(String, Uuid)>,
    Json(body): Json<PrStateMappingCreateRequest>,
) -> Result<(StatusCode, Json<PrStateMappingResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

    // Validar que el PR state sea un valor del enum Postgres
    if !VALID_PR_STATES.contains(&body.github_pr_state.as_str()) {
        return Err(AppError::BadRequest(format!(
            "github_pr_state inválido: '{}'. Valores permitidos: {}",
            body.github_pr_state,
            VALID_PR_STATES.join(", ")
        )));
    }

    // Verificar que la workspace_integration existe y pertenece al workspace
    let _ = workspace_integrations::Entity::find_by_id(wi_id)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mapping = db_githubprstatemapping::ActiveModel {
        id: Set(Uuid::new_v4()),
        github_pr_state: Set(body.github_pr_state),
        project_id: Set(body.project_id),
        state_id: Set(body.state_id),
        workspace_integration_id: Set(wi_id),
        prevent_regression: Set(body.prevent_regression.unwrap_or(false)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(PrStateMappingResponse::from_model(mapping)),
    ))
}

// ── 17. DELETE /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/ ─

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
        ("pk" = Uuid, Path, description = "Mapping ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_pr_state_mapping(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let mapping = db_githubprstatemapping::Entity::find_by_id(pk)
        .active()
        .filter(db_githubprstatemapping::Column::WorkspaceIntegrationId.eq(wi_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: db_githubprstatemapping::ActiveModel = mapping.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
