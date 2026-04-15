// src/utils/startup.rs
//! Equivalente Rust de los comandos Django `register_instance` y
//! `configure_instance`.  Se ejecuta una vez al arranque antes de
//! que el servidor HTTP comience a aceptar conexiones.
//!
//! - `ensure_instance_registered`: crea el registro `Instance` si no existe.
//! - `ensure_configurations_seeded`: inserta los valores por defecto en
//!   `InstanceConfiguration` solo para las claves que aún no existen.

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use std::env;

use crate::{
    entities::{instance_configurations, instances},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ─── register_instance ───────────────────────────────────────────────────────

/// Garantiza que exista exactamente un registro `Instance` en la base de datos.
/// Si no existe se crea con `is_setup_done = false`; si ya existe se actualiza
/// la versión y la fecha de verificación.
pub async fn ensure_instance_registered(state: &AppState) -> Result<(), AppError> {
    let existing = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let now = Utc::now();
    let version = env::var("APP_VERSION").unwrap_or_else(|_| "v0.1.0".to_owned());

    if let Some(inst) = existing {
        // Actualizar versión y last_checked_at en cada arranque
        let mut active: instances::ActiveModel = inst.into();
        active.current_version = Set(version.clone());
        active.last_checked_at = Set(now.into());
        active.is_test = Set(env::var("IS_TEST").as_deref() == Ok("1"));
        active.updated_at = Set(now.into());
        active
            .update(&state.db)
            .await
            .map_err(AppError::Database)?;
        tracing::info!("Instance already registered, updated version to {version}");
    } else {
        // Crear la instancia (is_setup_done = false hasta que God Mode completa el setup)
        let instance_id = uuid::Uuid::new_v4().simple().to_string()[..24].to_owned();
        instances::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            instance_name: Set("Plane Community Edition".to_owned()),
            instance_id: Set(instance_id),
            current_version: Set(version.clone()),
            latest_version: Set(None),
            last_checked_at: Set(now.into()),
            is_setup_done: Set(false),
            is_telemetry_enabled: Set(true),
            is_support_required: Set(true),
            is_signup_screen_visited: Set(false),
            is_verified: Set(false),
            is_test: Set(env::var("IS_TEST").as_deref() == Ok("1")),
            is_current_version_deprecated: Set(false),
            edition: Set("community".to_owned()),
            namespace: Set(None),
            domain: Set(String::new()),
            whitelist_emails: Set(None),
            deleted_at: Set(None),
            created_by_id: Set(None),
            updated_by_id: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;
        tracing::info!("Instance registered (version {version})");
    }

    Ok(())
}

// ─── configure_instance ──────────────────────────────────────────────────────

/// Tabla de variables de configuración con sus valores por defecto.
/// Solo se inserta si la clave NO existe ya en `InstanceConfiguration`.
/// Si existe, no se toca (preserva los valores configurados por el admin).
struct ConfigDefault {
    key: &'static str,
    /// Nombre de la env var que puede sobreescribir el default (None = no env var).
    env_var: Option<&'static str>,
    /// Valor por defecto cuando la env var no está seteada.
    default: &'static str,
    category: &'static str,
    is_encrypted: bool,
}

static CONFIG_DEFAULTS: &[ConfigDefault] = &[
    ConfigDefault { key: "ENABLE_SIGNUP",                 env_var: Some("ENABLE_SIGNUP"),                 default: "1",                    category: "AUTHENTICATION",      is_encrypted: false },
    ConfigDefault { key: "ENABLE_EMAIL_PASSWORD",         env_var: Some("ENABLE_EMAIL_PASSWORD"),         default: "1",                    category: "AUTHENTICATION",      is_encrypted: false },
    ConfigDefault { key: "ENABLE_MAGIC_LINK_LOGIN",       env_var: Some("ENABLE_MAGIC_LINK_LOGIN"),       default: "0",                    category: "AUTHENTICATION",      is_encrypted: false },
    ConfigDefault { key: "DISABLE_WORKSPACE_CREATION",    env_var: Some("DISABLE_WORKSPACE_CREATION"),    default: "0",                    category: "WORKSPACE_MANAGEMENT", is_encrypted: false },
    ConfigDefault { key: "IS_GOOGLE_ENABLED",             env_var: None,                                  default: "0",                    category: "GOOGLE",              is_encrypted: false },
    ConfigDefault { key: "GOOGLE_CLIENT_ID",              env_var: Some("GOOGLE_CLIENT_ID"),              default: "",                     category: "GOOGLE",              is_encrypted: false },
    ConfigDefault { key: "GOOGLE_CLIENT_SECRET",          env_var: Some("GOOGLE_CLIENT_SECRET"),          default: "",                     category: "GOOGLE",              is_encrypted: true  },
    ConfigDefault { key: "IS_GITHUB_ENABLED",             env_var: None,                                  default: "0",                    category: "GITHUB",              is_encrypted: false },
    ConfigDefault { key: "GITHUB_CLIENT_ID",              env_var: Some("GITHUB_CLIENT_ID"),              default: "",                     category: "GITHUB",              is_encrypted: false },
    ConfigDefault { key: "GITHUB_CLIENT_SECRET",          env_var: Some("GITHUB_CLIENT_SECRET"),          default: "",                     category: "GITHUB",              is_encrypted: true  },
    ConfigDefault { key: "IS_GITHUB_INTEGRATION_ENABLED", env_var: None,                                  default: "0",                    category: "GITHUB",              is_encrypted: false },
    ConfigDefault { key: "GITHUB_APP_NAME",               env_var: Some("GITHUB_APP_NAME"),               default: "",                     category: "GITHUB",              is_encrypted: false },
    ConfigDefault { key: "GITHUB_APP_ID",                 env_var: Some("GITHUB_APP_ID"),                 default: "",                     category: "GITHUB",              is_encrypted: false },
    ConfigDefault { key: "GITHUB_APP_PRIVATE_KEY",        env_var: Some("GITHUB_APP_PRIVATE_KEY"),        default: "",                     category: "GITHUB",              is_encrypted: true  },
    ConfigDefault { key: "GITHUB_WEBHOOK_SECRET",         env_var: Some("GITHUB_WEBHOOK_SECRET"),         default: "",                     category: "GITHUB",              is_encrypted: true  },
    ConfigDefault { key: "IS_GITLAB_ENABLED",             env_var: None,                                  default: "0",                    category: "GITLAB",              is_encrypted: false },
    ConfigDefault { key: "IS_GITLAB_INTEGRATION_ENABLED", env_var: None,                                  default: "0",                    category: "GITLAB",              is_encrypted: false },
    ConfigDefault { key: "GITLAB_HOST",                   env_var: Some("GITLAB_HOST"),                   default: "https://gitlab.com",   category: "GITLAB",              is_encrypted: false },
    ConfigDefault { key: "GITLAB_CLIENT_ID",              env_var: Some("GITLAB_CLIENT_ID"),              default: "",                     category: "GITLAB",              is_encrypted: false },
    ConfigDefault { key: "GITLAB_CLIENT_SECRET",          env_var: Some("GITLAB_CLIENT_SECRET"),          default: "",                     category: "GITLAB",              is_encrypted: true  },
    ConfigDefault { key: "IS_GITEA_ENABLED",              env_var: None,                                  default: "0",                    category: "GITEA",               is_encrypted: false },
    ConfigDefault { key: "GITEA_HOST",                    env_var: Some("GITEA_HOST"),                    default: "",                     category: "GITEA",               is_encrypted: false },
    ConfigDefault { key: "GITEA_CLIENT_ID",               env_var: Some("GITEA_CLIENT_ID"),               default: "",                     category: "GITEA",               is_encrypted: false },
    ConfigDefault { key: "GITEA_CLIENT_SECRET",           env_var: Some("GITEA_CLIENT_SECRET"),           default: "",                     category: "GITEA",               is_encrypted: true  },
    ConfigDefault { key: "IS_SLACK_ENABLED",              env_var: None,                                  default: "0",                    category: "SLACK",               is_encrypted: false },
    ConfigDefault { key: "SLACK_CLIENT_ID",               env_var: Some("SLACK_CLIENT_ID"),               default: "",                     category: "SLACK",               is_encrypted: false },
    ConfigDefault { key: "SLACK_CLIENT_SECRET",           env_var: Some("SLACK_CLIENT_SECRET"),           default: "",                     category: "SLACK",               is_encrypted: true  },
    ConfigDefault { key: "ENABLE_SMTP",                   env_var: None,                                  default: "0",                    category: "SMTP",                is_encrypted: false },
    ConfigDefault { key: "EMAIL_HOST",                    env_var: Some("EMAIL_HOST"),                    default: "",                     category: "SMTP",                is_encrypted: false },
    ConfigDefault { key: "EMAIL_HOST_USER",               env_var: Some("EMAIL_HOST_USER"),               default: "",                     category: "SMTP",                is_encrypted: false },
    ConfigDefault { key: "EMAIL_HOST_PASSWORD",           env_var: Some("EMAIL_HOST_PASSWORD"),           default: "",                     category: "SMTP",                is_encrypted: true  },
    ConfigDefault { key: "EMAIL_PORT",                    env_var: Some("EMAIL_PORT"),                    default: "587",                  category: "SMTP",                is_encrypted: false },
    ConfigDefault { key: "EMAIL_FROM",                    env_var: Some("EMAIL_FROM"),                    default: "Team Plane <team@mailer.plane.so>", category: "SMTP", is_encrypted: false },
    ConfigDefault { key: "EMAIL_USE_TLS",                 env_var: Some("EMAIL_USE_TLS"),                 default: "1",                    category: "SMTP",                is_encrypted: false },
    ConfigDefault { key: "EMAIL_USE_SSL",                 env_var: Some("EMAIL_USE_SSL"),                 default: "0",                    category: "SMTP",                is_encrypted: false },
    ConfigDefault { key: "UNSPLASH_ACCESS_KEY",           env_var: Some("UNSPLASH_ACCESS_KEY"),           default: "",                     category: "UNSPLASH",            is_encrypted: true  },
    ConfigDefault { key: "LLM_API_KEY",                   env_var: Some("LLM_API_KEY"),                   default: "",                     category: "AI",                  is_encrypted: true  },
    ConfigDefault { key: "LLM_PROVIDER",                  env_var: Some("LLM_PROVIDER"),                  default: "",                     category: "AI",                  is_encrypted: false },
    ConfigDefault { key: "LLM_MODEL",                     env_var: Some("LLM_MODEL"),                     default: "",                     category: "AI",                  is_encrypted: false },
    ConfigDefault { key: "IS_INTERCOM_ENABLED",           env_var: Some("IS_INTERCOM_ENABLED"),           default: "0",                    category: "INTERCOM",            is_encrypted: false },
    ConfigDefault { key: "INTERCOM_APP_ID",               env_var: Some("INTERCOM_APP_ID"),               default: "",                     category: "INTERCOM",            is_encrypted: false },
];

/// Siembra los valores por defecto en `InstanceConfiguration`.
/// Equivalente a `manage.py configure_instance`.
///
/// Solo inserta filas que no existan. Las filas ya presentes no se modifican,
/// preservando los cambios hechos por el admin via God Mode.
pub async fn ensure_configurations_seeded(state: &AppState) -> Result<(), AppError> {
    use crate::utils::fernet::encrypt_config_value;

    let now = Utc::now();
    let mut created = 0usize;

    for cfg in CONFIG_DEFAULTS {
        // ¿Ya existe esta clave?
        let exists = instance_configurations::Entity::find()
            .filter(instance_configurations::Column::Key.eq(cfg.key))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .is_some();

        if exists {
            continue;
        }

        // Resolver valor: env var → default hardcoded
        let raw_value = cfg
            .env_var
            .and_then(|var| env::var(var).ok())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| cfg.default.to_owned());

        let stored_value = if cfg.is_encrypted && !raw_value.is_empty() {
            encrypt_config_value(&raw_value)
        } else {
            raw_value
        };

        instance_configurations::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            key: Set(cfg.key.to_owned()),
            value: Set(Some(stored_value)),
            category: Set(cfg.category.to_owned()),
            is_encrypted: Set(cfg.is_encrypted),
            created_by_id: Set(None),
            updated_by_id: Set(None),
            deleted_at: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;

        created += 1;
    }

    if created > 0 {
        tracing::info!("Seeded {created} instance configuration defaults");
    } else {
        tracing::info!("Instance configurations already present, skipping seed");
    }

    Ok(())
}
