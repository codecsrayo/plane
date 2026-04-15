// src/routes/instances.rs
//! Endpoints de Instance — espejo de `plane/license/api/views/instance.py`.
//!
//! Rutas:
//!   GET    /api/instances/                    → InstanceEndpoint::get
//!   PATCH  /api/instances/                    → InstanceEndpoint::patch  (admin)
//!   POST   /api/instances/signup-screen-visited/ → SignUpScreenVisitedEndpoint::post

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    auth::any_auth::{AnyAuth, OptionalAnyAuth},
    entities::{instance_admins, instances, workspaces},
    error::AppError,
    utils::{instance_config::get_config_value, soft_delete::SoftDeleteExt},
    AppState,
};

// ─── DTOs ─────────────────────────────────────────────────────────────────────

/// Respuesta del GET cuando la instancia aún no está activada.
#[derive(Serialize)]
struct NotActivatedResponse {
    is_activated: bool,
    is_setup_done: bool,
}

/// Respuesta completa del GET cuando la instancia existe.
#[derive(Serialize)]
struct InstanceInfoResponse {
    instance: Value,
    config: Value,
}

/// Cuerpo del PATCH — todos los campos son opcionales (partial update).
#[derive(Deserialize)]
pub struct UpdateInstanceRequest {
    pub instance_name: Option<String>,
    pub is_telemetry_enabled: Option<bool>,
    pub is_support_required: Option<bool>,
    pub is_signup_screen_visited: Option<bool>,
}

// ─── Helpers internos ─────────────────────────────────────────────────────────

/// Verifica que el usuario autenticado sea instance admin (superuser o en
/// la tabla `instance_admins`).
async fn require_instance_admin(
    state: &AppState,
    user: &crate::entities::users::Model,
) -> Result<(), AppError> {
    if user.is_superuser {
        return Ok(());
    }

    let is_admin = instance_admins::Entity::find()
        .active()
        .filter(instance_admins::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if is_admin {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Lee un config valor (DB → env) y devuelve "" si no existe.
async fn cfg(state: &AppState, key: &str, fallback: &str) -> String {
    get_config_value(state, key, Some(fallback))
        .await
        .unwrap_or_default()
        .unwrap_or_else(|| fallback.to_owned())
}

/// Serializa un `instances::Model` al formato que espera el frontend,
/// incluyendo `is_activated` y `workspaces_exist`.
async fn serialize_instance(state: &AppState, instance: &instances::Model) -> Value {
    let workspaces_exist = workspaces::Entity::find()
        .active()
        .count(&state.db)
        .await
        .unwrap_or(0)
        >= 1;

    json!({
        "id":                            instance.id,
        "instance_name":                 instance.instance_name,
        "instance_id":                   instance.instance_id,
        "current_version":               instance.current_version,
        "latest_version":                instance.latest_version,
        "namespace":                     instance.namespace,
        "whitelist_emails":              instance.whitelist_emails,
        "domain":                        instance.domain,
        "edition":                       instance.edition,
        "is_setup_done":                 instance.is_setup_done,
        "is_telemetry_enabled":          instance.is_telemetry_enabled,
        "is_support_required":           instance.is_support_required,
        "is_signup_screen_visited":      instance.is_signup_screen_visited,
        "is_verified":                   instance.is_verified,
        "is_test":                       instance.is_test,
        "is_current_version_deprecated": instance.is_current_version_deprecated,
        "last_checked_at":               instance.last_checked_at,
        "created_at":                    instance.created_at,
        "updated_at":                    instance.updated_at,
        // Campos extra que agrega Django antes de devolver
        "is_activated":   true,
        "workspaces_exist": workspaces_exist,
    })
}

/// Construye el bloque `config` leyendo instance_configurations (DB) y, si no
/// existe el valor, variables de entorno — exactamente como hace Django en
/// `InstanceEndpoint.get()` mediante `get_configuration_value`.
async fn build_config(state: &AppState) -> Value {
    // Leer todos los valores en paralelo sería ideal, pero la legibilidad
    // tiene más peso aquí. El endpoint no está en hot path.
    let enable_signup = cfg(state, "ENABLE_SIGNUP",
        &std::env::var("ENABLE_SIGNUP").unwrap_or_else(|_| "0".into())).await;
    let disable_ws_creation = cfg(state, "DISABLE_WORKSPACE_CREATION",
        &std::env::var("DISABLE_WORKSPACE_CREATION").unwrap_or_else(|_| "0".into())).await;
    let is_google_enabled        = cfg(state, "IS_GOOGLE_ENABLED",        "0").await;
    let is_github_enabled        = cfg(state, "IS_GITHUB_ENABLED",        "0").await;
    let is_github_integration_en = cfg(state, "IS_GITHUB_INTEGRATION_ENABLED", "0").await;
    let github_app_name          = cfg(state, "GITHUB_APP_NAME",          "").await;
    let github_client_id         = cfg(state, "GITHUB_CLIENT_ID",         "").await;
    let is_gitlab_enabled        = cfg(state, "IS_GITLAB_ENABLED",        "0").await;
    let is_gitlab_integration_en = cfg(state, "IS_GITLAB_INTEGRATION_ENABLED", "0").await;
    let gitlab_client_id         = cfg(state, "GITLAB_CLIENT_ID",         "").await;
    let gitlab_host              = cfg(state, "GITLAB_HOST", "https://gitlab.com").await;
    let is_gitea_enabled         = cfg(state, "IS_GITEA_ENABLED",         "0").await;
    let email_host               = cfg(state, "EMAIL_HOST",
        &std::env::var("EMAIL_HOST").unwrap_or_default()).await;
    let enable_magic_login       = cfg(state, "ENABLE_MAGIC_LINK_LOGIN",
        &std::env::var("ENABLE_MAGIC_LINK_LOGIN").unwrap_or_else(|_| "1".into())).await;
    let enable_email_password    = cfg(state, "ENABLE_EMAIL_PASSWORD",
        &std::env::var("ENABLE_EMAIL_PASSWORD").unwrap_or_else(|_| "1".into())).await;
    let is_slack_enabled         = cfg(state, "IS_SLACK_ENABLED",         "0").await;
    let slack_client_id          = cfg(state, "SLACK_CLIENT_ID",          "").await;

    // PostHog — puede ser null
    let posthog_api_key: Option<String> = get_config_value(
        state, "POSTHOG_API_KEY",
        std::env::var("POSTHOG_API_KEY").ok().as_deref(),
    ).await.unwrap_or(None);
    let posthog_host: Option<String> = get_config_value(
        state, "POSTHOG_HOST",
        std::env::var("POSTHOG_HOST").ok().as_deref(),
    ).await.unwrap_or(None);

    let unsplash_access_key = cfg(state, "UNSPLASH_ACCESS_KEY",
        &std::env::var("UNSPLASH_ACCESS_KEY").unwrap_or_default()).await;
    let llm_api_key = cfg(state, "LLM_API_KEY",
        &std::env::var("LLM_API_KEY").unwrap_or_default()).await;

    // Intercom
    let is_intercom_enabled = cfg(state, "IS_INTERCOM_ENABLED",
        &std::env::var("IS_INTERCOM_ENABLED").unwrap_or_else(|_| "1".into())).await;
    let intercom_app_id = cfg(state, "INTERCOM_APP_ID",
        &std::env::var("INTERCOM_APP_ID").unwrap_or_default()).await;

    json!({
        // Auth feature flags
        "enable_signup":                   enable_signup == "1",
        "is_workspace_creation_disabled":  disable_ws_creation == "1",
        "is_google_enabled":               is_google_enabled == "1",
        "is_github_enabled":               is_github_enabled == "1",
        "is_github_integration_enabled":   is_github_integration_en == "1",
        "is_gitlab_enabled":               is_gitlab_enabled == "1",
        "is_gitlab_integration_enabled":   is_gitlab_integration_en == "1",
        "is_gitea_enabled":                is_gitea_enabled == "1",
        "is_magic_login_enabled":          enable_magic_login == "1",
        "is_email_password_enabled":       enable_email_password == "1",
        "is_slack_enabled":                is_slack_enabled == "1",

        // OAuth client IDs (public, no secrets)
        "github_app_name":  github_app_name,
        "github_client_id": github_client_id,
        "gitlab_client_id": gitlab_client_id,
        "gitlab_host":      gitlab_host,
        "slack_client_id":  slack_client_id,

        // Analytics
        "posthog_api_key": posthog_api_key,
        "posthog_host":    posthog_host,

        // Features
        "has_unsplash_configured": !unsplash_access_key.is_empty(),
        "has_llm_configured":      !llm_api_key.is_empty(),
        "is_smtp_configured":      !email_host.is_empty(),
        "file_size_limit": std::env::var("FILE_SIZE_LIMIT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(5_242_880.0),

        // Intercom
        "is_intercom_enabled": is_intercom_enabled == "1",
        "intercom_app_id":     intercom_app_id,

        // Base URLs (desde Config, no desde DB)
        "app_base_url":   std::env::var("APP_BASE_URL").unwrap_or_default(),
        "space_base_url": std::env::var("SPACE_BASE_URL").unwrap_or_default(),
        "admin_base_url": std::env::var("ADMIN_BASE_URL").unwrap_or_default(),
        "instance_changelog_url": std::env::var("INSTANCE_CHANGELOG_URL").unwrap_or_default(),
        "is_self_managed": true,
    })
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// `GET /api/instances/`
///
/// Endpoint público: devuelve la configuración de la instancia y los feature
/// flags que el frontend necesita para renderizar el flujo de autenticación.
/// Si no existe ninguna instancia en DB, responde con
/// `{"is_activated": false, "is_setup_done": false}`.
///
/// Equivalente Django: `InstanceEndpoint.get()`
#[utoipa::path(
    get,
    path = "/instances/",
    tag = "Instance",
    responses(
        (status = 200, description = "Instance info"),
    )
)]
pub async fn get_instance(
    State(state): State<AppState>,
    // No se require auth — AllowAny equivalente
    OptionalAnyAuth(_user_opt): OptionalAnyAuth,
) -> Result<impl IntoResponse, AppError> {
    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    match instance {
        None => {
            // Instancia no configurada todavía
            Ok(Json(json!({
                "is_activated": false,
                "is_setup_done": false,
            }))
            .into_response())
        }
        Some(inst) => {
            let instance_data = serialize_instance(&state, &inst).await;
            let config_data = build_config(&state).await;

            Ok(Json(json!({
                "instance": instance_data,
                "config":   config_data,
            }))
            .into_response())
        }
    }
}

/// `PATCH /api/instances/`
///
/// Actualiza campos editables de la instancia. Solo puede ser llamado por un
/// instance admin (superuser o entrada en `instance_admins`).
///
/// Equivalente Django: `InstanceEndpoint.patch()`
#[utoipa::path(
    patch,
    path = "/instances/",
    tag = "Instance",
    security((\"TokenAuth\" = [])),
    responses(
        (status = 200, description = "Updated instance"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — not an instance admin"),
        (status = 404, description = "Instance not found"),
    )
)]
pub async fn patch_instance(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<UpdateInstanceRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;

    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Instance not found".into()))?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: instances::ActiveModel = instance.into();

    if let Some(name) = body.instance_name {
        if name.is_empty() {
            return Err(AppError::BadRequest("instance_name cannot be empty".into()));
        }
        active.instance_name = Set(name);
    }
    if let Some(v) = body.is_telemetry_enabled {
        active.is_telemetry_enabled = Set(v);
    }
    if let Some(v) = body.is_support_required {
        active.is_support_required = Set(v);
    }
    if let Some(v) = body.is_signup_screen_visited {
        active.is_signup_screen_visited = Set(v);
    }

    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    let instance_data = serialize_instance(&state, &updated).await;

    Ok(Json(instance_data))
}

/// `POST /api/instances/signup-screen-visited/`
///
/// Marca `is_signup_screen_visited = true` en la instancia.
/// Requiere que la instancia exista. No requiere autenticación.
///
/// Equivalente Django: `SignUpScreenVisitedEndpoint.post()`
#[utoipa::path(
    post,
    path = "/instances/signup-screen-visited/",
    tag = "Instance",
    responses(
        (status = 204, description = "Marked as visited"),
        (status = 400, description = "Instance not configured"),
    )
)]
pub async fn signup_screen_visited(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::BadRequest("Instance is not configured".into()))?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: instances::ActiveModel = instance.into();
    active.is_signup_screen_visited = Set(true);
    active.updated_at = Set(now);

    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
