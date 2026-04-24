
> ✅ = implementado en Rust | ❌ = pendiente

```tree
.
├── Cargo.lock ✅
├── Cargo.toml ✅
├── Dockerfile ✅
├── Dockerfile.dev ✅
├── migration
│   ├── Cargo.toml ✅
│   └── src
│       ├── lib.rs
│       ├── main.rs
│       ├── migrations
│       │   ├── m20240101_000007_seed_data.rs
│       │   ├── m20260410_000001_baseline.rs
│       │   └── mod.rs
│       └── sql
│           └── baseline.sql ✅
├── rust-toolchain.toml ✅
├── src
│   ├── auth
│   │   ├── any_auth.rs
│   │   ├── api_key.rs
│   │   ├── csrf.rs
│   │   ├── email_auth.rs
│   │   ├── email_check.rs
│   │   ├── extractors.rs
│   │   ├── forgot_reset_password.rs
│   │   ├── god_mode.rs
│   │   ├── logout.rs
│   │   ├── magic_auth.rs
│   │   ├── mod.rs
│   │   ├── oauth.rs
│   │   ├── password_management.rs
│   │   ├── permissions.rs
│   │   ├── rate_limit.rs
│   │   ├── responses.rs
│   │   └── session.rs
│   ├── config.rs
│   ├── entities
│   │   ├── accounts.rs
│   │   ├── analytic_views.rs
│   │   ├── api_activity_logs.rs
│   │   ├── api_tokens.rs
│   │   ├── auth_group_permissions.rs
│   │   ├── auth_group.rs
│   │   ├── auth_permission.rs
│   │   ├── changelogs.rs
│   │   ├── comment_reactions.rs
│   │   ├── cycle_issues.rs
│   │   ├── cycles.rs
│   │   ├── cycle_user_properties.rs
│   │   ├── db_githubprstatemapping.rs
│   │   ├── deploy_boards.rs
│   │   ├── descriptions.rs
│   │   ├── description_versions.rs
│   │   ├── device_sessions.rs
│   │   ├── devices.rs
│   │   ├── django_celery_beat_clockedschedule.rs
│   │   ├── django_celery_beat_crontabschedule.rs
│   │   ├── django_celery_beat_intervalschedule.rs
│   │   ├── django_celery_beat_periodictask.rs
│   │   ├── django_celery_beat_periodictasks.rs
│   │   ├── django_celery_beat_solarschedule.rs
│   │   ├── django_content_type.rs
│   │   ├── django_migrations.rs
│   │   ├── django_session.rs
│   │   ├── draft_issue_assignees.rs
│   │   ├── draft_issue_cycles.rs
│   │   ├── draft_issue_labels.rs
│   │   ├── draft_issue_modules.rs
│   │   ├── draft_issues.rs
│   │   ├── email_notification_logs.rs
│   │   ├── estimate_points.rs
│   │   ├── estimates.rs
│   │   ├── exporters.rs
│   │   ├── file_assets.rs
│   │   ├── github_comment_syncs.rs
│   │   ├── github_issue_syncs.rs
│   │   ├── github_repositories.rs
│   │   ├── github_repository_syncs.rs
│   │   ├── gitlab_comment_syncs.rs
│   │   ├── gitlab_issue_syncs.rs
│   │   ├── gitlab_repositories.rs
│   │   ├── gitlab_repository_syncs.rs
│   │   ├── importers.rs
│   │   ├── instance_admins.rs
│   │   ├── instance_configurations.rs
│   │   ├── instances.rs
│   │   ├── intake_issues.rs
│   │   ├── intakes.rs
│   │   ├── integrations.rs
│   │   ├── issue_activities.rs
│   │   ├── issue_assignees.rs
│   │   ├── issue_attachments.rs
│   │   ├── issue_blockers.rs
│   │   ├── issue_comments.rs
│   │   ├── issue_description_versions.rs
│   │   ├── issue_labels.rs
│   │   ├── issue_links.rs
│   │   ├── issue_mentions.rs
│   │   ├── issue_reactions.rs
│   │   ├── issue_relations.rs
│   │   ├── issue_sequences.rs
│   │   ├── issues.rs
│   │   ├── issue_subscribers.rs
│   │   ├── issue_types.rs
│   │   ├── issue_versions.rs
│   │   ├── issue_views.rs
│   │   ├── issue_votes.rs
│   │   ├── labels.rs
│   │   ├── mod.rs
│   │   ├── module_issues.rs
│   │   ├── module_links.rs
│   │   ├── module_members.rs
│   │   ├── modules.rs
│   │   ├── module_user_properties.rs
│   │   ├── notifications.rs
│   │   ├── page_labels.rs
│   │   ├── page_logs.rs
│   │   ├── pages.rs
│   │   ├── page_versions.rs
│   │   ├── prelude.rs
│   │   ├── profiles.rs
│   │   ├── project_deploy_boards.rs
│   │   ├── project_identifiers.rs
│   │   ├── project_issue_types.rs
│   │   ├── project_member_invites.rs
│   │   ├── project_members.rs
│   │   ├── project_pages.rs
│   │   ├── project_public_members.rs
│   │   ├── projects.rs
│   │   ├── project_user_properties.rs
│   │   ├── project_webhooks.rs
│   │   ├── sessions.rs
│   │   ├── slack_project_syncs.rs
│   │   ├── social_login_connections.rs
│   │   ├── states.rs
│   │   ├── stickies.rs
│   │   ├── teams.rs
│   │   ├── user_favorites.rs
│   │   ├── user_github_connections.rs
│   │   ├── user_notification_preferences.rs
│   │   ├── user_recent_visits.rs
│   │   ├── users_groups.rs
│   │   ├── users.rs
│   │   ├── users_user_permissions.rs
│   │   ├── webhook_logs.rs
│   │   ├── webhooks.rs
│   │   ├── workspace_home_preferences.rs
│   │   ├── workspace_integrations.rs
│   │   ├── workspace_member_invites.rs
│   │   ├── workspace_members.rs
│   │   ├── workspaces.rs
│   │   ├── workspace_themes.rs
│   │   ├── workspace_user_links.rs
│   │   ├── workspace_user_preferences.rs
│   │   └── workspace_user_properties.rs
│   ├── error.rs
│   ├── jobs
│   │   ├── cleanup.rs
│   │   ├── cron.rs
│   │   ├── email_notification.rs
│   │   ├── export.rs
│   │   ├── github_sync.rs
│   │   ├── instance_traces.rs
│   │   ├── mod.rs
│   │   ├── notifications.rs
│   │   ├── scheduled.rs
│   │   ├── webhook_delivery.rs
│   │   └── workspace_seed.rs
│   ├── main.rs
│   ├── routes
│   │   ├── analytics.rs
│   │   ├── api_tokens.rs
│   │   ├── assets.rs
│   │   ├── cycles.rs
│   │   ├── estimates.rs
│   │   ├── exporter.rs
│   │   ├── external.rs
│   │   ├── health.rs
│   │   ├── helpers.rs
│   │   ├── importer.rs
│   │   ├── instances.rs
│   │   ├── intake.rs
│   │   ├── integrations
│   │   │   ├── dtos.rs
│   │   │   ├── github.rs
│   │   │   ├── gitlab.rs
│   │   │   ├── helpers.rs
│   │   │   ├── mod.rs
│   │   │   ├── pr_state.rs
│   │   │   └── workspace.rs
│   │   ├── issue_description_versions.rs
│   │   ├── issue_extras2.rs
│   │   ├── issue_extras.rs
│   │   ├── issue_filters.rs
│   │   ├── issue_pagination.rs
│   │   ├── issues.rs
│   │   ├── labels.rs
│   │   ├── mod.rs
│   │   ├── modules.rs
│   │   ├── notifications.rs
│   │   ├── pages.rs
│   │   ├── projects.rs
│   │   ├── project_user_properties.rs
│   │   ├── search.rs
│   │   ├── states.rs
│   │   ├── timezones.rs
│   │   ├── user_profile_issues.rs
│   │   ├── users.rs
│   │   ├── v1_router.rs
│   │   ├── views.rs
│   │   ├── webhooks.rs
│   │   ├── workspace_extras.rs
│   │   ├── workspaces.rs
│   │   └── workspace_view_issues.rs
│   └── utils
│       ├── color.rs
│       ├── content_validator.rs
│       ├── csv_sanitize.rs
│       ├── django_sessions.rs
│       ├── fernet.rs
│       ├── github_app.rs
│       ├── instance_config.rs
│       ├── mod.rs
│       ├── oauth_popup.rs
│       ├── pagination.rs
│       ├── passwords.rs
│       ├── posthog.rs
│       ├── s3_presigned_post.rs
│       ├── s3.rs
│       ├── serde_date.rs
│       ├── serde_empty.rs
│       ├── soft_delete.rs
│       ├── startup.rs
│       ├── token_cipher.rs
│       ├── url.rs
│       └── webhook_dispatch.rs
└── todo.md

```

12 directories, 226 files
