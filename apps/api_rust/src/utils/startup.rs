// src/utils/startup.rs
//! Rust equivalent of Django commands `register_instance` and
//! `configure_instance`. Runs once at startup before the
//! HTTP server begins accepting connections.
//!
//! - `ensure_instance_registered`: creates the `Instance` record if it doesn't exist.
//! - `ensure_configurations_seeded`: inserts default values into
//!   `InstanceConfiguration` only for keys that do not yet exist.

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

/// Ensures that exactly one `Instance` record exists in the database.
/// If it doesn't exist, it is created with `is_setup_done = false`; if it already exists,
/// the version and check date are updated.
pub async fn ensure_instance_registered(state: &AppState) -> Result<(), AppError> {
    let existing = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let now = Utc::now();
    let version = env::var("APP_VERSION").unwrap_or_else(|_| "v0.1.0".to_owned());

    if let Some(inst) = existing {
        // Update version and last_checked_at at each startup
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
        // Create the instance (is_setup_done = false until God Mode completes the setup)
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

/// Configuration variables table with their default values.
/// Only inserted if the key does NOT already exist in `InstanceConfiguration`.
/// If it exists, it is not touched (preserves values configured by the admin).
struct ConfigDefault {
    key: &'static str,
    /// Name of the env var that can override the default (None = no env var).
    env_var: Option<&'static str>,
    /// Default value when the env var is not set.
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

/// Seeds default values into `InstanceConfiguration`.
/// Equivalent to `manage.py configure_instance`.
///
/// Only inserts rows that do not exist. Already present rows are not modified,
/// preserving changes made by the admin via God Mode.
pub async fn ensure_configurations_seeded(state: &AppState) -> Result<(), AppError> {
    use crate::utils::fernet::encrypt_config_value;

    let now = Utc::now();
    let mut created = 0usize;

    for cfg in CONFIG_DEFAULTS {
        // Does this key already exist?
        let exists = instance_configurations::Entity::find()
            .filter(instance_configurations::Column::Key.eq(cfg.key))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .is_some();

        if exists {
            continue;
        }

        // Resolve value: env var → hardcoded default
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
