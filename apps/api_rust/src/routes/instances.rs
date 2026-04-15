// src/routes/instances.rs
//! Endpoints de Instance — espejo completo de `plane/license/api/views/`.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{
    auth::any_auth::{AnyAuth, OptionalAnyAuth},
    entities::{instance_admins, instance_configurations, instances, users, workspaces},
    error::AppError,
    utils::{
        fernet::{decrypt_config_value, encrypt_config_value},
        instance_config::get_config_value,
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

// ─── Allowlist de claves configurables via PATCH /configurations/ ─────────────

struct ConfigVar {
    is_encrypted: bool,
    category:     &'static str,
}

fn config_var(key: &str) -> Option<ConfigVar> {
    let table: &[(&str, bool, &str)] = &[
        ("ENABLE_SIGNUP",                 false, "AUTHENTICATION"),
        ("ENABLE_EMAIL_PASSWORD",         false, "AUTHENTICATION"),
        ("ENABLE_MAGIC_LINK_LOGIN",       false, "AUTHENTICATION"),
        ("DISABLE_WORKSPACE_CREATION",    false, "WORKSPACE_MANAGEMENT"),
        ("IS_GOOGLE_ENABLED",             false, "GOOGLE"),
        ("GOOGLE_CLIENT_ID",              false, "GOOGLE"),
        ("GOOGLE_CLIENT_SECRET",          true,  "GOOGLE"),
        ("ENABLE_GOOGLE_SYNC",            false, "GOOGLE"),
        ("IS_GITHUB_ENABLED",             false, "GITHUB"),
        ("GITHUB_CLIENT_ID",              false, "GITHUB"),
        ("GITHUB_CLIENT_SECRET",          true,  "GITHUB"),
        ("GITHUB_ORGANIZATION_ID",        false, "GITHUB"),
        ("ENABLE_GITHUB_SYNC",            false, "GITHUB"),
        ("IS_GITHUB_INTEGRATION_ENABLED", false, "GITHUB"),
        ("GITHUB_APP_NAME",               false, "GITHUB"),
        ("GITHUB_APP_ID",                 false, "GITHUB"),
        ("GITHUB_APP_PRIVATE_KEY",        true,  "GITHUB"),
        ("GITHUB_WEBHOOK_SECRET",         true,  "GITHUB"),
        ("IS_SLACK_ENABLED",              false, "SLACK"),
        ("SLACK_CLIENT_ID",               false, "SLACK"),
        ("SLACK_CLIENT_SECRET",           true,  "SLACK"),
        ("IS_GITLAB_ENABLED",             false, "GITLAB"),
        ("IS_GITLAB_INTEGRATION_ENABLED", false, "GITLAB"),
        ("GITLAB_HOST",                   false, "GITLAB"),
        ("GITLAB_CLIENT_ID",              false, "GITLAB"),
        ("GITLAB_CLIENT_SECRET",          true,  "GITLAB"),
        ("ENABLE_GITLAB_SYNC",            false, "GITLAB"),
        ("IS_GITEA_ENABLED",              false, "GITEA"),
        ("GITEA_HOST",                    false, "GITEA"),
        ("GITEA_CLIENT_ID",               false, "GITEA"),
        ("GITEA_CLIENT_SECRET",           true,  "GITEA"),
        ("ENABLE_GITEA_SYNC",             false, "GITEA"),
        ("ENABLE_SMTP",                   false, "SMTP"),
        ("EMAIL_HOST",                    false, "SMTP"),
        ("EMAIL_HOST_USER",               false, "SMTP"),
        ("EMAIL_HOST_PASSWORD",           true,  "SMTP"),
        ("EMAIL_PORT",                    false, "SMTP"),
        ("EMAIL_FROM",                    false, "SMTP"),
        ("EMAIL_USE_TLS",                 false, "SMTP"),
        ("EMAIL_USE_SSL",                 false, "SMTP"),
        ("LLM_API_KEY",                   true,  "AI"),
        ("LLM_PROVIDER",                  false, "AI"),
        ("LLM_MODEL",                     false, "AI"),
        ("GPT_ENGINE",                    false, "AI"),
        ("UNSPLASH_ACCESS_KEY",           true,  "UNSPLASH"),
        ("IS_INTERCOM_ENABLED",           false, "INTERCOM"),
        ("INTERCOM_APP_ID",               false, "INTERCOM"),
    ];
    table.iter().find(|(k, _, _)| *k == key).map(|(_, enc, cat)| ConfigVar {
        is_encrypted: *enc,
        category:     cat,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

async fn require_instance_admin(state: &AppState, user: &users::Model) -> Result<(), AppError> {
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
    if is_admin { Ok(()) } else { Err(AppError::Forbidden) }
}

async fn cfg(state: &AppState, key: &str, fallback: &str) -> String {
    get_config_value(state, key, Some(fallback))
        .await
        .unwrap_or_default()
        .unwrap_or_else(|| fallback.to_owned())
}

async fn serialize_instance(state: &AppState, inst: &instances::Model) -> Value {
    let workspaces_exist = workspaces::Entity::find()
        .active()
        .count(&state.db)
        .await
        .unwrap_or(0) >= 1;
    json!({
        "id":                            inst.id,
        "instance_name":                 inst.instance_name,
        "instance_id":                   inst.instance_id,
        "current_version":               inst.current_version,
        "latest_version":                inst.latest_version,
        "namespace":                     inst.namespace,
        "whitelist_emails":              inst.whitelist_emails,
        "domain":                        inst.domain,
        "edition":                       inst.edition,
        "is_setup_done":                 inst.is_setup_done,
        "is_telemetry_enabled":          inst.is_telemetry_enabled,
        "is_support_required":           inst.is_support_required,
        "is_signup_screen_visited":      inst.is_signup_screen_visited,
        "is_verified":                   inst.is_verified,
        "is_test":                       inst.is_test,
        "is_current_version_deprecated": inst.is_current_version_deprecated,
        "last_checked_at":               inst.last_checked_at,
        "created_at":                    inst.created_at,
        "updated_at":                    inst.updated_at,
        "is_activated":                  true,
        "workspaces_exist":              workspaces_exist,
    })
}

async fn build_config(state: &AppState) -> Value {
    let enable_signup = cfg(state, "ENABLE_SIGNUP",
        &std::env::var("ENABLE_SIGNUP").unwrap_or_else(|_| "0".into())).await;
    let disable_ws = cfg(state, "DISABLE_WORKSPACE_CREATION",
        &std::env::var("DISABLE_WORKSPACE_CREATION").unwrap_or_else(|_| "0".into())).await;
    let is_google      = cfg(state, "IS_GOOGLE_ENABLED", "0").await;
    let is_github      = cfg(state, "IS_GITHUB_ENABLED", "0").await;
    let is_github_int  = cfg(state, "IS_GITHUB_INTEGRATION_ENABLED", "0").await;
    let github_app     = cfg(state, "GITHUB_APP_NAME", "").await;
    let github_cid     = cfg(state, "GITHUB_CLIENT_ID", "").await;
    let is_gitlab      = cfg(state, "IS_GITLAB_ENABLED", "0").await;
    let is_gitlab_int  = cfg(state, "IS_GITLAB_INTEGRATION_ENABLED", "0").await;
    let gitlab_cid     = cfg(state, "GITLAB_CLIENT_ID", "").await;
    let gitlab_host    = cfg(state, "GITLAB_HOST", "https://gitlab.com").await;
    let is_gitea       = cfg(state, "IS_GITEA_ENABLED", "0").await;
    let email_host     = cfg(state, "EMAIL_HOST",
        &std::env::var("EMAIL_HOST").unwrap_or_default()).await;
    let magic_login    = cfg(state, "ENABLE_MAGIC_LINK_LOGIN",
        &std::env::var("ENABLE_MAGIC_LINK_LOGIN").unwrap_or_else(|_| "1".into())).await;
    let email_pw       = cfg(state, "ENABLE_EMAIL_PASSWORD",
        &std::env::var("ENABLE_EMAIL_PASSWORD").unwrap_or_else(|_| "1".into())).await;
    let is_slack       = cfg(state, "IS_SLACK_ENABLED", "0").await;
    let slack_cid      = cfg(state, "SLACK_CLIENT_ID", "").await;
    let posthog_key: Option<String> = get_config_value(state, "POSTHOG_API_KEY",
        std::env::var("POSTHOG_API_KEY").ok().as_deref()).await.unwrap_or(None);
    let posthog_host: Option<String> = get_config_value(state, "POSTHOG_HOST",
        std::env::var("POSTHOG_HOST").ok().as_deref()).await.unwrap_or(None);
    let unsplash       = cfg(state, "UNSPLASH_ACCESS_KEY",
        &std::env::var("UNSPLASH_ACCESS_KEY").unwrap_or_default()).await;
    let llm_key        = cfg(state, "LLM_API_KEY",
        &std::env::var("LLM_API_KEY").unwrap_or_default()).await;
    let intercom_en    = cfg(state, "IS_INTERCOM_ENABLED",
        &std::env::var("IS_INTERCOM_ENABLED").unwrap_or_else(|_| "1".into())).await;
    let intercom_id    = cfg(state, "INTERCOM_APP_ID",
        &std::env::var("INTERCOM_APP_ID").unwrap_or_default()).await;
    json!({
        "enable_signup":                   enable_signup == "1",
        "is_workspace_creation_disabled":  disable_ws == "1",
        "is_google_enabled":               is_google == "1",
        "is_github_enabled":               is_github == "1",
        "is_github_integration_enabled":   is_github_int == "1",
        "is_gitlab_enabled":               is_gitlab == "1",
        "is_gitlab_integration_enabled":   is_gitlab_int == "1",
        "is_gitea_enabled":                is_gitea == "1",
        "is_magic_login_enabled":          magic_login == "1",
        "is_email_password_enabled":       email_pw == "1",
        "is_slack_enabled":                is_slack == "1",
        "github_app_name":  github_app,
        "github_client_id": github_cid,
        "gitlab_client_id": gitlab_cid,
        "gitlab_host":      gitlab_host,
        "slack_client_id":  slack_cid,
        "posthog_api_key": posthog_key,
        "posthog_host":    posthog_host,
        "has_unsplash_configured": !unsplash.is_empty(),
        "has_llm_configured":      !llm_key.is_empty(),
        "is_smtp_configured":      !email_host.is_empty(),
        "file_size_limit": std::env::var("FILE_SIZE_LIMIT")
            .ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(5_242_880.0),
        "is_intercom_enabled": intercom_en == "1",
        "intercom_app_id":     intercom_id,
        "app_base_url":   std::env::var("APP_BASE_URL").unwrap_or_default(),
        "space_base_url": std::env::var("SPACE_BASE_URL").unwrap_or_default(),
        "admin_base_url": std::env::var("ADMIN_BASE_URL").unwrap_or_default(),
        "instance_changelog_url": std::env::var("INSTANCE_CHANGELOG_URL").unwrap_or_default(),
        "is_self_managed": true,
    })
}

fn serialize_config_row(c: &instance_configurations::Model) -> Value {
    let decrypted = if c.is_encrypted {
        decrypt_config_value(c.value.as_deref()).unwrap_or_else(|e| {
            tracing::warn!(key = %c.key, error = %e, "decrypt failed");
            String::new()
        })
    } else {
        c.value.clone().unwrap_or_default()
    };
    json!({
        "id":           c.id,
        "key":          c.key,
        "value":        decrypted,
        "category":     c.category,
        "is_encrypted": c.is_encrypted,
        "created_at":   c.created_at,
        "updated_at":   c.updated_at,
    })
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateInstanceRequest {
    pub instance_name:            Option<String>,
    pub is_telemetry_enabled:     Option<bool>,
    pub is_support_required:      Option<bool>,
    pub is_signup_screen_visited: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateAdminRequest {
    pub email: String,
    pub role:  Option<i32>,
}

#[derive(Deserialize)]
pub struct WorkspaceSlugQuery { pub slug: Option<String> }

#[derive(Deserialize)]
pub struct WorkspaceListQuery {
    pub cursor: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct TestEmailRequest { pub receiver_email: String }

// ─── GET /api/instances/ ──────────────────────────────────────────────────────

#[utoipa::path(get, path = "/instances/", tag = "Instance",
    responses((status = 200, description = "Instance info")))]
pub async fn get_instance(
    State(state): State<AppState>,
    OptionalAnyAuth(_): OptionalAnyAuth,
) -> Result<impl IntoResponse, AppError> {
    match instances::Entity::find().active().one(&state.db).await.map_err(AppError::Database)? {
        None => Ok(Json(json!({"is_activated": false, "is_setup_done": false})).into_response()),
        Some(inst) => Ok(Json(json!({
            "instance": serialize_instance(&state, &inst).await,
            "config":   build_config(&state).await,
        })).into_response()),
    }
}

// ─── PATCH /api/instances/ ────────────────────────────────────────────────────

#[utoipa::path(patch, path = "/instances/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Updated"), (status = 403, description = "Forbidden")))]
pub async fn patch_instance(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<UpdateInstanceRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    let inst = instances::Entity::find().active().one(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Instance not found".into()))?;
    let now = chrono::Utc::now().fixed_offset();
    let mut active: instances::ActiveModel = inst.into();
    if let Some(name) = body.instance_name {
        if name.is_empty() { return Err(AppError::BadRequest("instance_name cannot be empty".into())); }
        active.instance_name = Set(name);
    }
    if let Some(v) = body.is_telemetry_enabled    { active.is_telemetry_enabled    = Set(v); }
    if let Some(v) = body.is_support_required      { active.is_support_required     = Set(v); }
    if let Some(v) = body.is_signup_screen_visited { active.is_signup_screen_visited = Set(v); }
    active.updated_by_id = Set(Some(user.id));
    active.updated_at    = Set(now);
    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(serialize_instance(&state, &updated).await))
}

// ─── POST /api/instances/admins/sign-up-screen-visited/ ───────────────────────

#[utoipa::path(post, path = "/instances/admins/sign-up-screen-visited/", tag = "Instance",
    responses((status = 204, description = "Marked")))]
pub async fn signup_screen_visited(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let inst = instances::Entity::find().active().one(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::BadRequest("Instance is not configured".into()))?;
    let now = chrono::Utc::now().fixed_offset();
    let mut active: instances::ActiveModel = inst.into();
    active.is_signup_screen_visited = Set(true);
    active.updated_at               = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ─── GET /api/instances/admins/ ───────────────────────────────────────────────

#[utoipa::path(get, path = "/instances/admins/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "List")))]
pub async fn list_instance_admins(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    let inst = instances::Entity::find().active().one(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::BadRequest("Instance is not registered yet".into()))?;
    let admins = instance_admins::Entity::find()
        .active()
        .filter(instance_admins::Column::InstanceId.eq(inst.id))
        .all(&state.db).await.map_err(AppError::Database)?;
    let mut result = Vec::with_capacity(admins.len());
    for admin in &admins {
        let user_detail = if let Some(uid) = admin.user_id {
            users::Entity::find_by_id(uid).one(&state.db).await.map_err(AppError::Database)?
                .map(|u| json!({"id": u.id, "email": u.email, "first_name": u.first_name, "last_name": u.last_name}))
        } else { None };
        result.push(json!({
            "id": admin.id, "role": admin.role, "is_verified": admin.is_verified,
            "instance": admin.instance_id, "user": admin.user_id,
            "user_detail": user_detail,
            "created_at": admin.created_at, "updated_at": admin.updated_at,
        }));
    }
    Ok(Json(result))
}

// ─── POST /api/instances/admins/ ──────────────────────────────────────────────

#[utoipa::path(post, path = "/instances/admins/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 201, description = "Created"), (status = 403, description = "Forbidden")))]
pub async fn create_instance_admin(
    State(state): State<AppState>,
    AnyAuth(caller): AnyAuth,
    Json(body): Json<CreateAdminRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &caller).await?;
    if body.email.is_empty() { return Err(AppError::BadRequest("Email is required".into())); }
    let inst = instances::Entity::find().active().one(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::BadRequest("Instance is not registered yet".into()))?;
    let target = users::Entity::find()
        .filter(users::Column::Email.eq(body.email.to_lowercase().trim()))
        .one(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    let role = body.role.unwrap_or(20);
    let now  = chrono::Utc::now().fixed_offset();
    let saved = instance_admins::ActiveModel {
        id: Set(uuid::Uuid::new_v4()), instance_id: Set(inst.id),
        user_id: Set(Some(target.id)), role: Set(role), is_verified: Set(false),
        created_by_id: Set(Some(caller.id)), updated_by_id: Set(Some(caller.id)),
        created_at: Set(now), updated_at: Set(now), deleted_at: Set(None),
    }.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((axum::http::StatusCode::CREATED, Json(json!({
        "id": saved.id, "role": saved.role, "instance": saved.instance_id, "user": saved.user_id,
    }))))
}

// ─── GET /api/instances/admins/me/ ────────────────────────────────────────────

#[utoipa::path(get, path = "/instances/admins/me/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Current admin")))]
pub async fn get_instance_admin_me(AnyAuth(user): AnyAuth) -> impl IntoResponse {
    Json(json!({
        "id": user.id, "email": user.email, "first_name": user.first_name,
        "last_name": user.last_name, "avatar": user.avatar, "is_superuser": user.is_superuser,
    }))
}

// ─── GET /api/instances/admins/session/ ───────────────────────────────────────

#[utoipa::path(get, path = "/instances/admins/session/", tag = "Instance",
    responses((status = 200, description = "Session status")))]
pub async fn get_instance_admin_session(
    State(state): State<AppState>,
    OptionalAnyAuth(user_opt): OptionalAnyAuth,
) -> Result<impl IntoResponse, AppError> {
    let Some(user) = user_opt else {
        return Ok(Json(json!({"is_authenticated": false})));
    };
    let is_admin = user.is_superuser || instance_admins::Entity::find()
        .active()
        .filter(instance_admins::Column::UserId.eq(user.id))
        .one(&state.db).await.map_err(AppError::Database)?.is_some();
    if is_admin {
        Ok(Json(json!({
            "is_authenticated": true,
            "user": {"id": user.id, "email": user.email, "first_name": user.first_name, "last_name": user.last_name}
        })))
    } else {
        Ok(Json(json!({"is_authenticated": false})))
    }
}

// ─── DELETE /api/instances/admins/{pk}/ ───────────────────────────────────────

#[utoipa::path(delete, path = "/instances/admins/{pk}/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 204, description = "Deleted"), (status = 403, description = "Forbidden")))]
pub async fn delete_instance_admin(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(pk): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    if let Some(admin) = instance_admins::Entity::find_by_id(pk)
        .one(&state.db).await.map_err(AppError::Database)?
    {
        let now = chrono::Utc::now().fixed_offset();
        let mut active: instance_admins::ActiveModel = admin.into();
        active.deleted_at = Set(Some(now));
        active.update(&state.db).await.map_err(AppError::Database)?;
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ─── GET /api/instances/configurations/ ──────────────────────────────────────

#[utoipa::path(get, path = "/instances/configurations/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Configurations")))]
pub async fn list_configurations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    let configs = instance_configurations::Entity::find()
        .active()
        .order_by_asc(instance_configurations::Column::Key)
        .all(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(configs.iter().map(serialize_config_row).collect::<Vec<_>>()))
}

// ─── PATCH /api/instances/configurations/ ────────────────────────────────────

#[utoipa::path(patch, path = "/instances/configurations/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Updated configs"), (status = 403, description = "Forbidden")))]
pub async fn update_configurations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<HashMap<String, Value>>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    let now = chrono::Utc::now().fixed_offset();
    let mut updated_keys: Vec<String> = Vec::new();

    for (key, raw_value) in &body {
        let Some(var) = config_var(key) else { continue; };
        let value_str = raw_value.as_str().unwrap_or_default();
        let stored = if var.is_encrypted && !value_str.is_empty() {
            encrypt_config_value(value_str)
        } else {
            value_str.to_owned()
        };
        let existing = instance_configurations::Entity::find()
            .active()
            .filter(instance_configurations::Column::Key.eq(key.as_str()))
            .one(&state.db).await.map_err(AppError::Database)?;
        if let Some(existing) = existing {
            let mut active: instance_configurations::ActiveModel = existing.into();
            active.value         = Set(Some(stored));
            active.is_encrypted  = Set(var.is_encrypted);
            active.category      = Set(var.category.to_owned());
            active.updated_by_id = Set(Some(user.id));
            active.updated_at    = Set(now);
            active.update(&state.db).await.map_err(AppError::Database)?;
        } else {
            instance_configurations::ActiveModel {
                id: Set(uuid::Uuid::new_v4()), key: Set(key.clone()),
                value: Set(Some(stored)), category: Set(var.category.to_owned()),
                is_encrypted: Set(var.is_encrypted),
                created_by_id: Set(Some(user.id)), updated_by_id: Set(Some(user.id)),
                created_at: Set(now), updated_at: Set(now), deleted_at: Set(None),
            }.insert(&state.db).await.map_err(AppError::Database)?;
        }
        updated_keys.push(key.clone());
    }

    let configs = instance_configurations::Entity::find()
        .active()
        .filter(instance_configurations::Column::Key.is_in(updated_keys))
        .all(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(configs.iter().map(serialize_config_row).collect::<Vec<_>>()))
}

// ─── DELETE /api/instances/configurations/disable-email-feature/ ─────────────

#[utoipa::path(delete, path = "/instances/configurations/disable-email-feature/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Disabled"), (status = 403, description = "Forbidden")))]
pub async fn disable_email_feature(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    let smtp_keys = ["EMAIL_HOST","EMAIL_HOST_USER","EMAIL_HOST_PASSWORD","ENABLE_SMTP","EMAIL_PORT","EMAIL_FROM"];
    let now = chrono::Utc::now().fixed_offset();
    for key in smtp_keys {
        if let Some(existing) = instance_configurations::Entity::find()
            .active()
            .filter(instance_configurations::Column::Key.eq(key))
            .one(&state.db).await.map_err(AppError::Database)?
        {
            let new_val = if key == "ENABLE_SMTP" { "0" } else { "" };
            let mut active: instance_configurations::ActiveModel = existing.into();
            active.value         = Set(Some(new_val.to_owned()));
            active.updated_by_id = Set(Some(user.id));
            active.updated_at    = Set(now);
            active.update(&state.db).await.map_err(AppError::Database)?;
        }
    }
    Ok(Json(json!({"message": "Email configuration disabled"})))
}

// ─── POST /api/instances/email-credentials-check/ ────────────────────────────

#[utoipa::path(post, path = "/instances/email-credentials-check/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Email sent"),
        (status = 400, description = "SMTP error"),
        (status = 403, description = "Forbidden"),
    ))]
pub async fn email_credentials_check(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<TestEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;

    if body.receiver_email.is_empty() {
        return Err(AppError::BadRequest("receiver_email is required".into()));
    }

    let email_host = cfg(&state, "EMAIL_HOST",
        &std::env::var("EMAIL_HOST").unwrap_or_default()).await;
    if email_host.is_empty() {
        return Err(AppError::BadRequest("SMTP not configured".into()));
    }

    let email_port: u16 = cfg(&state, "EMAIL_PORT",
        &std::env::var("EMAIL_PORT").unwrap_or_else(|_| "587".into()))
        .await.parse().unwrap_or(587);

    let email_user = cfg(&state, "EMAIL_HOST_USER",
        &std::env::var("EMAIL_HOST_USER").unwrap_or_default()).await;

    let raw_pw = instance_configurations::Entity::find()
        .active()
        .filter(instance_configurations::Column::Key.eq("EMAIL_HOST_PASSWORD"))
        .one(&state.db).await.map_err(AppError::Database)?
        .and_then(|c| c.value);
    let email_password = decrypt_config_value(raw_pw.as_deref()).unwrap_or_default();

    let email_from = cfg(&state, "EMAIL_FROM",
        &std::env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@plane.so".into())).await;
    let use_tls = cfg(&state, "EMAIL_USE_TLS",
        &std::env::var("EMAIL_USE_TLS").unwrap_or_default()).await == "1";

    use lettre::{
        message::header::ContentType,
        transport::smtp::{authentication::Credentials, client::Tls},
        AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    };

    let email = Message::builder()
        .from(email_from.parse()
            .map_err(|_| AppError::BadRequest("Invalid EMAIL_FROM address".into()))?)
        .to(body.receiver_email.parse()
            .map_err(|_| AppError::BadRequest("Invalid receiver_email".into()))?)
        .subject("Email Notification from Plane")
        .header(ContentType::TEXT_PLAIN)
        .body("This is a sample email notification sent from Plane application.".to_owned())
        .map_err(|e| AppError::BadRequest(format!("Email build error: {e}")))?;

    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&email_host)
        .map_err(|e| AppError::BadRequest(format!("Invalid EMAIL_HOST: {e}")))?
        .port(email_port);

    if !email_user.is_empty() {
        builder = builder.credentials(Credentials::new(email_user, email_password));
    }
    if !use_tls {
        builder = builder.tls(Tls::None);
    }

    builder.build().send(email).await
        .map_err(|e| AppError::BadRequest(format!("Send failed: {e}")))?;

    Ok(Json(json!({"message": "Email successfully sent."})))
}

// ─── GET /api/instances/workspace-slug-check/ ────────────────────────────────

#[utoipa::path(get, path = "/instances/workspace-slug-check/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Slug availability")))]
pub async fn instance_workspace_slug_check(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Query(params): Query<WorkspaceSlugQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;
    let slug = params.slug.unwrap_or_default();
    if slug.is_empty() {
        return Err(AppError::BadRequest("Workspace Slug is required".into()));
    }
    let taken = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(slug.to_lowercase().as_str()))
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(json!({"status": !taken})))
}

// ─── GET /api/instances/workspaces/ ──────────────────────────────────────────

#[utoipa::path(get, path = "/instances/workspaces/", tag = "Instance",
    security(("TokenAuth" = [])),
    responses((status = 200, description = "Paginated workspaces")))]
pub async fn list_instance_workspaces(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Query(params): Query<WorkspaceListQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_instance_admin(&state, &user).await?;

    const PER_PAGE: u64 = 10;
    let page: u64 = params.cursor.as_deref()
        .and_then(|c| base64::engine::general_purpose::STANDARD.decode(c).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut query = workspaces::Entity::find().active();
    if let Some(ref s) = params.search {
        if !s.is_empty() { query = query.filter(workspaces::Column::Name.contains(s.as_str())); }
    }

    let total   = query.clone().count(&state.db).await.map_err(AppError::Database)?;
    let results = query
        .order_by_asc(workspaces::Column::Name)
        .paginate(&state.db, PER_PAGE)
        .fetch_page(page)
        .await
        .map_err(AppError::Database)?;

    let has_next   = (page + 1) * PER_PAGE < total;
    let next_cursor = has_next.then(|| {
        base64::engine::general_purpose::STANDARD.encode((page + 1).to_string())
    });

    let items: Vec<Value> = results.iter().map(|ws| json!({
        "id": ws.id, "name": ws.name, "slug": ws.slug, "logo": ws.logo,
        "owner": ws.owner_id, "created_at": ws.created_at, "updated_at": ws.updated_at,
    })).collect();

    Ok(Json(json!({
        "results": items, "total_count": total,
        "next_cursor": next_cursor, "prev_cursor": null,
        "next_page_results": has_next, "prev_page_results": false,
        "extra_stats": {},
    })))
}
