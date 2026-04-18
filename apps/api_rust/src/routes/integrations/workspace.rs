// src/routes/integrations/workspace.rs
//! Endpoints de gestión de workspace-integrations (CRUD) y provider install.
//!
//! Endpoints implementados:
//!   GET    /api/workspaces/{slug}/workspace-integrations/
//!   POST   /api/workspaces/{slug}/workspace-integrations/
//!   GET    /api/workspaces/{slug}/workspace-integrations/{pk}/
//!   PATCH  /api/workspaces/{slug}/workspace-integrations/{pk}/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/{pk}/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/{provider}/provider/
//!   POST   /api/workspaces/{slug}/workspace-integrations/{provider}/install/

use anyhow::Context as _;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, QueryFilter,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{
    auth::{extractors::WorkspaceMemberGuard, permissions::require_workspace_admin},
    entities::{integrations, workspace_integrations},
    error::AppError,
    utils::{instance_config::get_instance_config, soft_delete::SoftDeleteExt},
    AppState,
};

use super::{
    dtos::{
        CreateWorkspaceIntegrationRequest, ProviderInstallRequest,
        UpdateWorkspaceIntegrationRequest, WorkspaceIntegrationResponse,
    },
    helpers::get_or_create_api_token,
};

// ── GET /workspaces/{slug}/workspace-integrations/ ───────────────────────────

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

    // Batch-fetch evita N+1: una sola query para todas las integraciones.
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

// ── POST /workspaces/{slug}/workspace-integrations/ ──────────────────────────

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

    // Antipatrón corregido: SELECT luego INSERT sin transacción es TOCTOU.
    let wi = state
        .db
        .transaction_with_config::<_, workspace_integrations::Model, AppError>(
            |txn| {
                let integration_id = integration.id;
                let workspace_id = guard.workspace.id;
                let user_id = guard.user.id;
                let api_token_id = api_token.id;
                let metadata = body.metadata.clone().unwrap_or(serde_json::json!({}));
                let config = body.config.clone().unwrap_or(serde_json::json!({}));
                Box::pin(async move {
                    let existing = workspace_integrations::Entity::find()
                        .active()
                        .filter(workspace_integrations::Column::WorkspaceId.eq(workspace_id))
                        .filter(workspace_integrations::Column::IntegrationId.eq(integration_id))
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?;

                    if existing.is_some() {
                        return Err(AppError::BadRequest("Integration already installed".into()));
                    }

                    // created_at/updated_at explícitos (NOT NULL sin DEFAULT).
                    let now: chrono::DateTime<chrono::FixedOffset> =
                        chrono::Utc::now().into();

                    workspace_integrations::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        workspace_id: Set(workspace_id),
                        integration_id: Set(integration_id),
                        actor_id: Set(user_id),
                        api_token_id: Set(api_token_id),
                        metadata: Set(metadata),
                        config: Set(config),
                        created_at: Set(now),
                        updated_at: Set(now),
                        deleted_at: Set(None),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await
                    .map_err(AppError::Database)
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

    Ok((
        StatusCode::CREATED,
        Json(WorkspaceIntegrationResponse::from_model(wi, Some(integration))),
    ))
}

// ── GET /workspaces/{slug}/workspace-integrations/{pk}/ ──────────────────────

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

// ── PATCH /workspaces/{slug}/workspace-integrations/{pk}/ ────────────────────

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

// ── DELETE /workspaces/{slug}/workspace-integrations/{pk}/ ───────────────────

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

// ── DELETE /workspaces/{slug}/workspace-integrations/{provider}/provider/ ────

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

// ── POST /workspaces/{slug}/workspace-integrations/{provider}/install/ ────────

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
            let code = body.code.ok_or_else(|| {
                AppError::BadRequest("code is required for GitLab integration".into())
            })?;
            (serde_json::json!({ "code": code }), serde_json::json!({}))
        }
        "slack" => {
            let code = body.code.ok_or_else(|| {
                AppError::BadRequest("code is required for Slack integration".into())
            })?;
            // `?` propaga errores de OAuth al cliente (400/502).
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

    // Upsert en transacción SERIALIZABLE — mismo patrón que get_or_create_api_token.
    let (wi, created) = state
        .db
        .transaction_with_config::<_, (workspace_integrations::Model, bool), AppError>(
            |txn| {
                let metadata = metadata.clone();
                let config = config.clone();
                let integration_id = integration.id;
                let workspace_id = guard.workspace.id;
                let user_id = guard.user.id;
                let api_token_id = api_token.id;
                Box::pin(async move {
                    let existing = workspace_integrations::Entity::find()
                        .filter(workspace_integrations::Column::WorkspaceId.eq(workspace_id))
                        .filter(workspace_integrations::Column::IntegrationId.eq(integration_id))
                        .filter(workspace_integrations::Column::DeletedAt.is_null())
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?;

                    let now: chrono::DateTime<chrono::FixedOffset> =
                        chrono::Utc::now().into();

                    if let Some(wi) = existing {
                        let mut am: workspace_integrations::ActiveModel = wi.into();
                        am.metadata = Set(metadata);
                        am.config = Set(config);
                        // Django: TimeAuditModel auto_now=True.
                        am.updated_at = Set(now);
                        Ok((am.update(txn).await.map_err(AppError::Database)?, false))
                    } else {
                        // created_at/updated_at explícitos (NOT NULL sin DEFAULT).
                        let new_wi = workspace_integrations::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            workspace_id: Set(workspace_id),
                            integration_id: Set(integration_id),
                            actor_id: Set(user_id),
                            api_token_id: Set(api_token_id),
                            metadata: Set(metadata),
                            config: Set(config),
                            created_at: Set(now),
                            updated_at: Set(now),
                            deleted_at: Set(None),
                            ..Default::default()
                        };
                        Ok((new_wi.insert(txn).await.map_err(AppError::Database)?, true))
                    }
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

    let status = if created { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(WorkspaceIntegrationResponse::from_model(wi, Some(integration)))))
}

// ── Slack OAuth helper ────────────────────────────────────────────────────────

/// Intercambia el code de Slack por access_token y construye metadata/config.
///
/// # Degradación controlada vs. error real
/// - Sin credenciales configuradas → `Ok((code_json, {}))` — instalación parcial
///   intencional; no es un error.
/// - Con credenciales pero exchange fallido → `Err` — propaga 400/502 al cliente.
///
/// Antipatrón corregido: la versión anterior absorbía todos los errores como
/// fallback silencioso, devolviendo 201 con metadata incompleta.
async fn build_slack_metadata(
    state: &AppState,
    code: &str,
) -> Result<(serde_json::Value, serde_json::Value), AppError> {
    let client_id = match get_instance_config(state, "SLACK_CLIENT_ID").await? {
        Some(v) if !v.is_empty() => v,
        _ => return Ok((serde_json::json!({ "code": code }), serde_json::json!({}))),
    };
    let client_secret = match get_instance_config(state, "SLACK_CLIENT_SECRET").await? {
        Some(v) if !v.is_empty() => v,
        _ => return Ok((serde_json::json!({ "code": code }), serde_json::json!({}))),
    };

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

    // Slack devuelve siempre HTTP 200; `ok` indica el resultado real.
    if !slack_data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let err_msg = slack_data
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_error");
        return Err(AppError::BadRequest(format!("Slack OAuth error: {err_msg}")));
    }

    let config = serde_json::json!({
        "access_token": slack_data.get("access_token"),
        "team_id":      slack_data.get("team").and_then(|t| t.get("id")),
        "team_name":    slack_data.get("team").and_then(|t| t.get("name")),
    });

    Ok((slack_data, config))
}
