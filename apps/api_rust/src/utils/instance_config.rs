use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    entities::instance_configurations, error::AppError, utils::soft_delete::SoftDeleteExt, AppState,
};

/// Claves permitidas para `get_instance_config`.
///
/// [Fix #19] Sin allowlist, cualquier código que llame a esta función con
/// una clave arbitraria puede leer cualquier variable de entorno del proceso
/// (DATABASE_URL, SECRET_KEY, AWS_SECRET_ACCESS_KEY, etc.) si no existe en DB.
const ALLOWED_INSTANCE_CONFIG_KEYS: &[&str] = &[
    "GITHUB_APP_ID",
    "GITHUB_APP_PRIVATE_KEY",
    "GITHUB_CLIENT_ID",
    "GITHUB_CLIENT_SECRET",
    "GITHUB_WEBHOOK_SECRET",
    "GITLAB_CLIENT_ID",
    "GITLAB_CLIENT_SECRET",
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

/// Obtiene un valor de configuración de instancia validado por allowlist.
///
/// Prioridad: DB (instance_configurations) → variable de entorno.
/// Rechaza claves no incluidas en `ALLOWED_INSTANCE_CONFIG_KEYS`.
pub async fn get_instance_config(
    state: &AppState,
    key: &str,
) -> Result<Option<String>, AppError> {
    if !ALLOWED_INSTANCE_CONFIG_KEYS.contains(&key) {
        tracing::error!(key, "get_instance_config: clave no permitida");
        return Err(AppError::BadRequest(
            format!("Clave de configuración no permitida: '{key}'"),
        ));
    }

    // 1. Buscar en instance_configurations (prioridad DB sobre env)
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

    // 2. Fallback a variable de entorno (solo claves del allowlist)
    Ok(std::env::var(key).ok())
}
