// migration/src/migrations/m20240101_000007_seed_data.rs
use sea_orm_migration::prelude::*;

/// Seeds static reference data that Django inserts automatically via RunPython migrations:
///
/// 1. `integrations` — 3 rows (github, gitlab, slack) from Django migrations 0122 & 0123.
/// 2. `instance_configurations` — all config keys from configure_instance management command.
///
/// All inserts use ON CONFLICT DO NOTHING so the migration is idempotent and safe
/// to run against a DB that was already seeded by Django.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000007_seed_data"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── 1. integrations seed ──────────────────────────────────────────────
        // Mirrors Django migrations 0122_add_github_gitlab_integrations &
        // 0123_add_slack_integration. Required non-null fields that Django
        // leaves at defaults are set to empty string / empty jsonb.
        db.execute_unprepared(
            r#"
            INSERT INTO integrations (
                id, created_at, updated_at,
                title, provider, network, verified,
                description, author, webhook_url, webhook_secret, redirect_url, metadata
            ) VALUES
                (gen_random_uuid(), NOW(), NOW(), 'GitHub', 'github', 2, TRUE, '{}', '', '', '', '', '{}'),
                (gen_random_uuid(), NOW(), NOW(), 'GitLab', 'gitlab', 2, TRUE, '{}', '', '', '', '', '{}'),
                (gen_random_uuid(), NOW(), NOW(), 'Slack',  'slack',  2, TRUE, '{}', '', '', '', '', '{}')
            ON CONFLICT (provider) DO NOTHING
            "#,
        )
        .await?;

        // ── 2. instance_configurations seed ───────────────────────────────────
        // Mirrors configure_instance management command (core_config_variables).
        // Values here are the hardcoded defaults; env-overrides are applied at
        // runtime by the Django command, not at migration time.
        // is_encrypted flag is preserved so the app knows which keys to decrypt.
        let configs: &[(&str, Option<&str>, &str, bool)] = &[
            // AUTHENTICATION
            ("ENABLE_SIGNUP", Some("1"), "AUTHENTICATION", false),
            ("ENABLE_EMAIL_PASSWORD", Some("1"), "AUTHENTICATION", false),
            (
                "ENABLE_MAGIC_LINK_LOGIN",
                Some("0"),
                "AUTHENTICATION",
                false,
            ),
            // WORKSPACE_MANAGEMENT
            (
                "DISABLE_WORKSPACE_CREATION",
                Some("0"),
                "WORKSPACE_MANAGEMENT",
                false,
            ),
            // GOOGLE
            ("IS_GOOGLE_ENABLED", Some("0"), "GOOGLE", false),
            ("GOOGLE_CLIENT_ID", Some(""), "GOOGLE", false),
            ("GOOGLE_CLIENT_SECRET", Some(""), "GOOGLE", true),
            ("ENABLE_GOOGLE_SYNC", Some("0"), "GOOGLE", false),
            // GITHUB
            ("IS_GITHUB_ENABLED", Some("0"), "GITHUB", false),
            ("GITHUB_CLIENT_ID", Some(""), "GITHUB", false),
            ("GITHUB_CLIENT_SECRET", Some(""), "GITHUB", true),
            ("GITHUB_ORGANIZATION_ID", Some(""), "GITHUB", false),
            ("ENABLE_GITHUB_SYNC", Some("0"), "GITHUB", false),
            ("IS_GITHUB_INTEGRATION_ENABLED", Some("0"), "GITHUB", false),
            ("GITHUB_APP_NAME", Some(""), "GITHUB", false),
            ("GITHUB_APP_ID", Some(""), "GITHUB", false),
            ("GITHUB_APP_PRIVATE_KEY", Some(""), "GITHUB", true),
            ("GITHUB_WEBHOOK_SECRET", Some(""), "GITHUB", true),
            // GITLAB
            ("IS_GITLAB_ENABLED", Some("0"), "GITLAB", false),
            ("IS_GITLAB_INTEGRATION_ENABLED", Some("0"), "GITLAB", false),
            ("GITLAB_HOST", Some("https://gitlab.com"), "GITLAB", false),
            ("GITLAB_CLIENT_ID", Some(""), "GITLAB", false),
            ("GITLAB_CLIENT_SECRET", Some(""), "GITLAB", true),
            ("ENABLE_GITLAB_SYNC", Some("0"), "GITLAB", false),
            // GITEA
            ("IS_GITEA_ENABLED", Some("0"), "GITEA", false),
            ("GITEA_HOST", Some(""), "GITEA", false),
            ("GITEA_CLIENT_ID", Some(""), "GITEA", false),
            ("GITEA_CLIENT_SECRET", Some(""), "GITEA", true),
            ("ENABLE_GITEA_SYNC", Some("0"), "GITEA", false),
            // SLACK
            ("IS_SLACK_ENABLED", Some("0"), "SLACK", false),
            ("SLACK_CLIENT_ID", Some(""), "SLACK", false),
            ("SLACK_CLIENT_SECRET", Some(""), "SLACK", true),
            // SMTP
            ("ENABLE_SMTP", Some("0"), "SMTP", false),
            ("EMAIL_HOST", Some(""), "SMTP", false),
            ("EMAIL_HOST_USER", Some(""), "SMTP", false),
            ("EMAIL_HOST_PASSWORD", Some(""), "SMTP", true),
            ("EMAIL_PORT", Some("587"), "SMTP", false),
            ("EMAIL_FROM", Some(""), "SMTP", false),
            ("EMAIL_USE_TLS", Some("1"), "SMTP", false),
            ("EMAIL_USE_SSL", Some("0"), "SMTP", false),
            // AI
            ("LLM_API_KEY", None, "AI", true),
            ("LLM_PROVIDER", Some("openai"), "AI", false),
            ("LLM_MODEL", Some("gpt-4o-mini"), "AI", false),
            ("GPT_ENGINE", Some("gpt-3.5-turbo"), "AI", false),
            // UNSPLASH
            ("UNSPLASH_ACCESS_KEY", Some(""), "UNSPLASH", true),
            // INTERCOM
            ("IS_INTERCOM_ENABLED", Some("1"), "INTERCOM", false),
            ("INTERCOM_APP_ID", Some(""), "INTERCOM", false),
        ];

        for (key, value, category, is_encrypted) in configs {
            let value_sql = match value {
                Some(v) => format!("'{}'", v.replace('\'', "''")),
                None => "NULL".to_string(),
            };
            db.execute_unprepared(&format!(
                r#"
                INSERT INTO instance_configurations (id, created_at, updated_at, key, value, category, is_encrypted)
                VALUES (gen_random_uuid(), NOW(), NOW(), '{key}', {value_sql}, '{category}', {is_encrypted})
                ON CONFLICT (key) DO NOTHING
                "#,
            ))
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove the 3 integration rows
        db.execute_unprepared(
            "DELETE FROM integrations WHERE provider IN ('github', 'gitlab', 'slack') AND verified = TRUE",
        )
        .await?;

        // Remove all seeded config keys
        let keys: &[&str] = &[
            "ENABLE_SIGNUP",
            "ENABLE_EMAIL_PASSWORD",
            "ENABLE_MAGIC_LINK_LOGIN",
            "DISABLE_WORKSPACE_CREATION",
            "IS_GOOGLE_ENABLED",
            "GOOGLE_CLIENT_ID",
            "GOOGLE_CLIENT_SECRET",
            "ENABLE_GOOGLE_SYNC",
            "IS_GITHUB_ENABLED",
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
            "GITHUB_ORGANIZATION_ID",
            "ENABLE_GITHUB_SYNC",
            "IS_GITHUB_INTEGRATION_ENABLED",
            "GITHUB_APP_NAME",
            "GITHUB_APP_ID",
            "GITHUB_APP_PRIVATE_KEY",
            "GITHUB_WEBHOOK_SECRET",
            "IS_GITLAB_ENABLED",
            "IS_GITLAB_INTEGRATION_ENABLED",
            "GITLAB_HOST",
            "GITLAB_CLIENT_ID",
            "GITLAB_CLIENT_SECRET",
            "ENABLE_GITLAB_SYNC",
            "IS_GITEA_ENABLED",
            "GITEA_HOST",
            "GITEA_CLIENT_ID",
            "GITEA_CLIENT_SECRET",
            "ENABLE_GITEA_SYNC",
            "IS_SLACK_ENABLED",
            "SLACK_CLIENT_ID",
            "SLACK_CLIENT_SECRET",
            "ENABLE_SMTP",
            "EMAIL_HOST",
            "EMAIL_HOST_USER",
            "EMAIL_HOST_PASSWORD",
            "EMAIL_PORT",
            "EMAIL_FROM",
            "EMAIL_USE_TLS",
            "EMAIL_USE_SSL",
            "LLM_API_KEY",
            "LLM_PROVIDER",
            "LLM_MODEL",
            "GPT_ENGINE",
            "UNSPLASH_ACCESS_KEY",
            "IS_INTERCOM_ENABLED",
            "INTERCOM_APP_ID",
        ];

        let keys_sql = keys
            .iter()
            .map(|k| format!("'{k}'"))
            .collect::<Vec<_>>()
            .join(", ");
        db.execute_unprepared(&format!(
            "DELETE FROM instance_configurations WHERE key IN ({keys_sql})"
        ))
        .await?;

        Ok(())
    }
}
