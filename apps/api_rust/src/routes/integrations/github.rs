// src/routes/integrations/github.rs
//! Endpoints especÃ­ficos de GitHub.
//!
//! Endpoints implementados:
//!   GET  /api/github/callback/                                         (sin auth)
//!   POST /auth/github/user-callback/
//!   GET  /api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/
//!   GET  /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
//!   POST /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/

use anyhow::Context as _;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};

use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        extractors::WorkspaceMemberGuard,
        permissions::{require_workspace_admin, require_workspace_member, ROLE_ADMIN},
    },
    entities::{
        github_repositories, github_repository_syncs, integrations, projects,
        user_github_connections, workspace_integrations, workspace_members, workspaces,
    },
    error::AppError,
    utils::{
        github_app::get_installation_access_token,
        instance_config::get_instance_config,
        oauth_popup::{postmessage_html, OAuthMessageType},
        soft_delete::SoftDeleteExt,
        token_cipher::encrypt_token,
    },
    AppState,
};

use super::{
    dtos::{
        ExternalReposQuery, GithubCallbackQuery, GithubRepoSyncCreateRequest,
        GithubRepoSyncResponse, IntegrationResponse, UserGithubCallbackRequest,
        UserGithubConnectionResponse,
    },
    helpers::get_or_create_api_token,
};

// ââ GET /api/github/callback/ (sin auth) âââââââââââââââââââââââââââââââââââââ

/// Callback de GitHub App â Setup URL registrada en la GitHub App.
/// No requiere autenticaciÃ³n (GitHub redirige el popup aquÃ­ directamente).
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
            None,
        );
    };

    let setup_action = params
        .setup_action
        .as_deref()
        .unwrap_or("install")
        .to_owned();

    match github_app_callback_inner(&state, installation_id, &setup_action, workspace_slug).await {
        Ok(()) => postmessage_html(true, OAuthMessageType::GithubIntegration, None, None),
        Err(e) => {
            tracing::error!(error = %e, "GithubAppCallback failed");
            postmessage_html(
                false,
                OAuthMessageType::GithubIntegration,
                Some("Installation failed. Please try again."),
                None,
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

    // Este callback es sin auth â usar el primer workspace admin como actor.
    let admin_member = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace.id))
        .filter(workspace_members::Column::Role.gte(ROLE_ADMIN))
        .filter(workspace_members::Column::IsActive.eq(true))
        .order_by_asc(workspace_members::Column::CreatedAt)
        .one(&state.db)
        .await?
        .context("No workspace admin found â installation_id cannot be persisted")?;

    let actor_id = admin_member.member_id;
    let api_token =
        get_or_create_api_token(state, actor_id, workspace.id, "GitHub Integration Token")
            .await?;

    let metadata = serde_json::json!({
        "installation_id": installation_id,
        "setup_action": setup_action,
    });
    let config = serde_json::json!({ "installation_id": installation_id });

    // Antipatron corregido: el SELECT + INSERT/UPDATE debe ejecutarse dentro de
    // una transaccion SERIALIZABLE para eliminar la race condition TOCTOU.
    // Django usa update_or_create dentro de transaction.atomic() + captura
    // IntegrityError como fallback. Aqui replicamos ese contrato de forma segura.
    state
        .db
        .transaction_with_config::<_, (), anyhow::Error>(
            |txn| {
                let metadata = metadata.clone();
                let config = config.clone();
                let workspace_id = workspace.id;
                let integration_id = integration.id;
                let api_token_id = api_token.id;
                Box::pin(async move {
                    let existing = workspace_integrations::Entity::find()
                        .filter(workspace_integrations::Column::WorkspaceId.eq(workspace_id))
                        .filter(workspace_integrations::Column::IntegrationId.eq(integration_id))
                        .filter(workspace_integrations::Column::DeletedAt.is_null())
                        .one(txn)
                        .await?;

                    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

                    if let Some(wi) = existing {
                        let mut am: workspace_integrations::ActiveModel = wi.into();
                        am.metadata = Set(metadata);
                        am.config = Set(config);
                        am.actor_id = Set(actor_id);
                        am.api_token_id = Set(api_token_id);
                        // Django: TimeAuditModel auto_now=True.
                        am.updated_at = Set(now);
                        am.update(txn).await?;
                    } else {
                        // created_at/updated_at explicitos (NOT NULL sin DEFAULT).
                        workspace_integrations::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            workspace_id: Set(workspace_id),
                            integration_id: Set(integration_id),
                            actor_id: Set(actor_id),
                            api_token_id: Set(api_token_id),
                            metadata: Set(metadata),
                            config: Set(config),
                            created_at: Set(now),
                            updated_at: Set(now),
                            deleted_at: Set(None),
                            ..Default::default()
                        }
                        .insert(txn)
                        .await?;
                    }

                    Ok(())
                })
            },
            Some(sea_orm::IsolationLevel::Serializable),
            None,
        )
        .await?;

    Ok(())
}

// ââ POST /auth/github/user-callback/ âââââââââââââââââââââââââââââââââââââ

/// Intercambia un OAuth code de GitHub por un token personal de usuario.
#[utoipa::path(
    post,
    path = "/auth/github/user-callback/",
    tag = "Integrations",
    responses(
        (status = 201, description = "ConexiÃ³n creada"),
        (status = 200, description = "ConexiÃ³n actualizada"),
        (status = 400, description = "Error de validaciÃ³n"),
        (status = 502, description = "Error al contactar GitHub"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn github_user_callback(
    State(state): State<AppState>,
    auth: AnyAuth,
    Json(body): Json<UserGithubCallbackRequest>,
) -> Result<(StatusCode, Json<UserGithubConnectionResponse>), AppError> {
    // Django: UserGithubConnectionView usa IsAuthenticated sin scope de workspace.
    // WorkspaceMemberGuard requería {slug} en el path, que esta ruta no tiene,
    // causando 404 en todas las llamadas. Corregido: AnyAuth (sesión o API token).
    let client_id = get_instance_config(&state, "GITHUB_CLIENT_ID")
        .await?
        .ok_or_else(|| AppError::BadRequest("GitHub OAuth is not configured".into()))?;

    let client_secret = get_instance_config(&state, "GITHUB_CLIENT_SECRET")
        .await?
        .ok_or_else(|| AppError::BadRequest("GitHub OAuth is not configured".into()))?;

    // Django incluye redirect_uri en el token exchange para evitar
    // redirect_uri_mismatch si la GitHub App lo tiene configurado.
    let redirect_uri = state
        .config
        .web_url
        .as_deref()
        .map(|base| {
            format!(
                "{}/auth/github/user-callback/",
                base.trim_end_matches('/')
            )
        });

    let mut form_params: Vec<(&str, String)> = vec![
        ("client_id", client_id.clone()),
        ("client_secret", client_secret.clone()),
        ("code", body.code.clone()),
    ];
    if let Some(ref uri) = redirect_uri {
        form_params.push(("redirect_uri", uri.clone()));
    }

    let token_resp = state
        .http
        .post("https://github.com/login/oauth/access_token")
        .form(&form_params)
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

    let github_avatar_url = github_user["avatar_url"].as_str().unwrap_or("").to_owned();
    let user_id = auth.0.id;

    // El access_token se cifra con AES-256-GCM antes de ser almacenado.
    let encrypted_token = encrypt_token(&access_token);

    let existing = user_github_connections::Entity::find()
        .filter(user_github_connections::Column::UserId.eq(user_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let (conn, created) = if let Some(conn) = existing {
        let mut am: user_github_connections::ActiveModel = conn.into();
        am.github_user_id = Set(github_user_id);
        am.github_username = Set(github_username);
        am.github_avatar_url = Set(github_avatar_url);
        am.access_token = Set(encrypted_token);
        // Django: TimeAuditModel auto_now=True.
        am.updated_at = Set(now);
        (am.update(&state.db).await.map_err(AppError::Database)?, false)
    } else {
        // created_at/updated_at explÃ­citos (NOT NULL sin DEFAULT).
        let new_conn = user_github_connections::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            github_user_id: Set(github_user_id),
            github_username: Set(github_username),
            github_avatar_url: Set(github_avatar_url),
            access_token: Set(encrypted_token),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        };
        (
            new_conn.insert(&state.db).await.map_err(AppError::Database)?,
            true,
        )
    };

    let status = if created { StatusCode::CREATED } else { StatusCode::OK };

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

// ââ GET /workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/ â

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
        ("page" = Option<u32>, Query, description = "PÃ¡gina (default 1)"),
        ("per_page" = Option<u32>, Query, description = "Repos por pÃ¡gina (default 30)"),
    ),
    responses(
        (status = 200, description = "Lista de repositorios GitHub"),
        (status = 400, description = "Error de configuraciÃ³n"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_github_repositories(
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

    let installation_id = wi
        .metadata
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

    let send_result = state
        .http
        .get(&api_url)
        .query(&query_params)
        .header("Authorization", format!("Bearer {github_token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "plane-api-rust/0.1")
        .send()
        .await;
    let resp: reqwest::Response = send_result
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

    let (repos, total_count): (Vec<serde_json::Value>, usize) = if let Some(arr) = payload.as_array() {
        let len = arr.len();
        (arr.clone(), len)
    } else {
        let repos: Vec<serde_json::Value> = payload
            .get("repositories")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        let total = payload
            .get("total_count")
            .and_then(|t| t.as_u64())
            .unwrap_or(repos.len() as u64) as usize;
        (repos, total)
    };

    let manage_url = installation_id
        .as_deref()
        .map(|iid| format!("https://github.com/settings/installations/{iid}"));

    let mapped: Vec<serde_json::Value> = repos
        .iter()
        .map(|repo| {
            serde_json::json!({
                "id": repo["id"].as_i64().unwrap_or(0).to_string(),
                "full_name": repo["full_name"],
                "name": repo["name"],
                "owner": repo["owner"]["login"],
                "description": repo.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                "private": repo["private"],
                "url": repo["html_url"],
                "issues_count": repo.get("open_issues_count").and_then(|c| c.as_u64()).unwrap_or(0),
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

// ââ GET /workspaces/{slug}/workspace-integrations/github/repo-syncs/ âââââââââ

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
    // Django: @allow_permission([ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    // Tanto admins como members pueden listar los syncs del workspace.
    require_workspace_member(&guard.member)?;

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

    // Batch-fetch de repositorios â evita N+1.
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

    // Batch-fetch de proyectos para incluir `project_name` y `project_identifier`
    // tal como hace Django en GithubRepoSyncViewSet.list().
    let project_ids: Vec<Uuid> = syncs.iter().map(|s| s.project_id).collect();
    let projects_map: std::collections::HashMap<Uuid, projects::Model> =
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.id, p))
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

            let (project_name, project_identifier) =
                if let Some(p) = projects_map.get(&sync.project_id) {
                    (p.name.clone(), p.identifier.clone())
                } else {
                    (String::new(), String::new())
                };

            GithubRepoSyncResponse {
                id: sync.id,
                project_id: sync.project_id,
                project_name,
                project_identifier,
                repo_id,
                repo_full_name: format!("{repo_owner}/{repo_name}"),
                repo_name,
                repo_owner,
                sync_direction,
                issue_open_state,
                issue_closed_state,
                created_at: sync.created_at,
            }
        })
        .collect();

    Ok(Json(result))
}

// ââ POST /workspaces/{slug}/workspace-integrations/github/repo-syncs/ âââââââââ

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-integrations/github/repo-syncs/",
    tag = "Integrations",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Repo sync creado"),
        (status = 400, description = "Error de validaciÃ³n"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_github_repo_sync(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<GithubRepoSyncCreateRequest>,
) -> Result<(StatusCode, Json<GithubRepoSyncResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

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

    let wi = workspace_integrations::Entity::find()
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .inner_join(integrations::Entity)
        .filter(integrations::Column::Provider.eq("github"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            AppError::BadRequest("GitHub integration not installed for this workspace".into())
        })?;

    // Buscar GithubRepository â incluyendo soft-deleted para no violar unique constraint.
    let existing_repo = github_repositories::Entity::find()
        .filter(github_repositories::Column::RepositoryId.eq(repo_id_int))
        .filter(github_repositories::Column::ProjectId.eq(body.project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // now() compartido para ambos bloques (repositorio + sync). Columnas
    // created_at/updated_at NOT NULL sin DEFAULT en ambas entities.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

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
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
                ..Default::default()
            }
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?
        }
        Some(r) if r.deleted_at.is_some() => {
            let mut am: github_repositories::ActiveModel = r.into();
            am.deleted_at = Set(None);
            am.name = Set(repo_name.clone());
            am.owner = Set(repo_owner.clone());
            am.url = Set(Some(format!("https://github.com/{repo_full_name}")));
            // Django: TimeAuditModel auto_now=True.
            am.updated_at = Set(now);
            am.update(&state.db).await.map_err(AppError::Database)?
        }
        Some(r) => r,
    };

    let credentials = serde_json::json!({
        "sync_direction": body.sync_direction.as_deref().unwrap_or("bidirectional"),
        "issue_open_state": body.issue_open_state,
        "issue_closed_state": body.issue_closed_state,
    });

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
            let mut am: github_repository_syncs::ActiveModel = s.into();
            am.deleted_at = Set(None);
            am.actor_id = Set(guard.user.id);
            am.workspace_integration_id = Set(wi.id);
            am.credentials = Set(credentials.clone());
            // Django: TimeAuditModel auto_now=True.
            am.updated_at = Set(now);
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
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
                ..Default::default()
            }
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?
        }
    };

    // Registrar webhook en GitHub â best-effort, no bloquea la respuesta.
    if let Err(e) = register_github_webhook(&state, &wi, &repo_owner, &repo_name).await {
        tracing::warn!(
            repo_sync_id = %sync.id,
            "Failed to register GitHub webhook: {e}"
        );
    }

    // Fetch project para incluir project_name e project_identifier en la
    // respuesta â espeja el campo que Django devuelve en GithubRepoSyncViewSet.create().
    let project = projects::Entity::find_by_id(sync.project_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let (project_name, project_identifier) = project
        .map(|p| (p.name, p.identifier))
        .unwrap_or_default();

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
            project_name,
            project_identifier,
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
            created_at: sync.created_at,
        }),
    ))
}

// ââ DELETE /workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/ â

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

    // Ambos soft-deletes deben ser atÃ³micos â sin transacciÃ³n quedarÃ­a estado
    // inconsistente si el segundo update falla.
    state
        .db
        .transaction::<_, (), AppError>(|txn| {
            Box::pin(async move {
                let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

                let mut am: github_repository_syncs::ActiveModel = sync.into();
                am.deleted_at = Set(Some(now));
                am.update(txn).await.map_err(AppError::Database)?;

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

// ââ GitHub webhook registration (helper interno) ââââââââââââââââââââââââââââââ

/// Registra el webhook de Plane en el repositorio GitHub.
/// Best-effort: nunca debe fallar el handler padre.
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

    // AntipatrÃ³n corregido: get_instance_config en lugar de std::env::var().
    let webhook_secret = get_instance_config(state, "GITHUB_WEBHOOK_SECRET")
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    let web_url = state
        .config
        .web_url
        .as_deref()
        .context("WEB_URL no configurado â requerido para registrar webhooks de GitHub")?
        .to_owned();

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
        anyhow::bail!("GitHub webhook registration failed: {status} â {body}");
    }

    Ok(())
}

// ââ list_integrations (endpoint global) âââââââââââââââââââââââââââââââââââââ
//
// CatÃ¡logo global de integraciones disponibles. En Django el equivalente
// (`IntegrationViewSet`) usa `IsAuthenticated`, que acepta tanto sesiÃ³n como
// API token. Usamos `AnyAuth` para replicar ese contrato â de lo contrario el
// frontend, que manda cookie de sesiÃ³n, recibe 401 y cae en el loop del
// interceptor (`/settings/integrations/` â `/?next_path=â¦`).

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
    _auth: AnyAuth,
) -> Result<Json<Vec<IntegrationResponse>>, AppError> {
    // Django usa `self.model.objects.all()` â sin filtro de soft-delete â
    // para garantizar que todas las integraciones aparezcan en el panel,
    // incluidas las no verificadas. Replicamos ese comportamiento aquÃ­.
    // AntipatrÃ³n evitado: no usar `.active()` que filtrarÃ­a registros vÃ¡lidos.
    let rows = integrations::Entity::find()
        .order_by_asc(integrations::Column::Title)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(IntegrationResponse::from_model).collect()))
}
