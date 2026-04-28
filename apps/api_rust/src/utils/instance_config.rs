use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    entities::instance_configurations, error::AppError, utils::soft_delete::SoftDeleteExt, AppState,
};

/// Allowed keys for `get_instance_config`.
///
/// [Fix #19] Without allowlist, any code calling this function with
/// an arbitrary key can read any environment variable of the process
/// (DATABASE_URL, SECRET_KEY, AWS_SECRET_ACCESS_KEY, etc.) if it doesn't exist in DB.
const ALLOWED_INSTANCE_CONFIG_KEYS: &[&str] = &[
    "GITHUB_APP_ID",
    "GITHUB_APP_PRIVATE_KEY",
    "GITHUB_CLIENT_ID",
    "GITHUB_CLIENT_SECRET",
    "GITHUB_WEBHOOK_SECRET",
    "GITLAB_CLIENT_ID",
    "GITLAB_CLIENT_SECRET",
    "GITLAB_HOST",
    "GOOGLE_CLIENT_ID",
    "GOOGLE_CLIENT_SECRET",
    "GITEA_CLIENT_ID",
    "GITEA_CLIENT_SECRET",
    "GITEA_HOST",
    "SLACK_CLIENT_ID",
    "SLACK_CLIENT_SECRET",
    "OPENAI_API_KEY",
    "EMAIL_HOST",
    "EMAIL_HOST_USER",
    "EMAIL_HOST_PASSWORD",
    "EMAIL_PORT",
    "EMAIL_USE_TLS",
];

pub async fn get_config_value(
    state: &AppState,
    key: &str,
    fallback: Option<&str>,
) -> Result<Option<String>, AppError> {
    let value = instance_configurations::Entity::find()
        .active()
        .filter(instance_configurations::Column::Key.eq(key))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .and_then(|config| config.value)
        .or_else(|| fallback.map(str::to_owned));

    Ok(value)
}

/// Gets an instance configuration value validated by allowlist.
///
/// Priority: DB (instance_configurations) → environment variable.
/// Rejects keys not included in `ALLOWED_INSTANCE_CONFIG_KEYS`.
pub async fn get_instance_config(
    state: &AppState,
    key: &str,
) -> Result<Option<String>, AppError> {
    if !ALLOWED_INSTANCE_CONFIG_KEYS.contains(&key) {
        tracing::error!(key, "get_instance_config: key not allowed");
        return Err(AppError::BadRequest(
            format!("Configuration key not allowed: '{key}'"),
        ));
    }

    // 1. Search in instance_configurations (DB priority over env)
    let db_value = instance_configurations::Entity::find()
        .active()
        .filter(instance_configurations::Column::Key.eq(key))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .and_then(|c| c.value)
        .filter(|v| !v.is_empty());

    if db_value.is_some() {
        return Ok(db_value);
    }

    // 2. Fallback to environment variable (only allowlist keys)
    Ok(std::env::var(key).ok())
}
