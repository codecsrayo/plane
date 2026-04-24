
> [ ] = implementado en Rust | ❌ = pendiente
> pruebas de integración

```tree
.
├── Cargo.lock [x]
├── Cargo.toml [x]
├── Dockerfile [x]
├── Dockerfile.dev [x]
├── migration
│   ├── Cargo.toml [x]
│   └── src
│       ├── lib.rs
│       ├── main.rs
│       ├── migrations
│       │   ├── m20240101_000007_seed_data.rs
│       │   ├── m20260410_000001_baseline.rs
│       │   └── mod.rs
│       └── sql
│           └── baseline.sql [x]
├── rust-toolchain.toml [x]
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

## Auth (`authentication/`)

- ✅ `GET  /auth/get-csrf-token`
- ✅ `POST /auth/sign-in`
- ✅ `POST /auth/sign-up`
- ✅ `POST /auth/sign-out`
- ✅ `POST /auth/email-check`
- ✅ `POST /auth/magic-sign-in`
- ✅ `POST /auth/magic-sign-up`
- ✅ `POST /auth/magic-generate`
- ✅ `POST /auth/forgot-password`
- ✅ `POST /auth/reset-password/{uidb64}/{token}`
- ✅ `POST /auth/change-password`
- ✅ `POST /auth/set-password`
- ✅ `GET  /auth/github/callback`
- ✅ `GET  /auth/github/user-callback`
- ✅ `GET  /auth/gitlab`
- ✅ `GET  /auth/gitlab/callback`
- ✅ `GET  /auth/google`
- ✅ `GET  /auth/google/callback`
- ✅ `GET  /auth/gitea`
- ✅ `GET  /auth/gitea/callback`
- ✅ `POST /spaces/sign-in`
- ✅ `POST /spaces/sign-out`
- ✅ `POST /spaces/magic-sign-in`
- ✅ `POST /spaces/magic-sign-up`
- ✅ `POST /spaces/magic-generate`
- ✅ `POST /spaces/email-check`
- ✅ `POST /spaces/forgot-password`
- ✅ `POST /spaces/reset-password/{uidb64}/{token}`

## License / Instance (`license/`)

- ✅ `GET/POST   /instances`
- ✅ `GET        /instances/email-credentials-check`
- ✅ `GET/PATCH  /instances/configurations`
- ✅ `POST       /instances/configurations/disable-email-feature`
- ✅ `GET/POST   /instances/admins`
- ✅ `GET/PATCH/DELETE /instances/admins/{pk}`
- ✅ `POST       /instances/admins/sign-in`
- ✅ `POST       /instances/admins/sign-up`
- ✅ `POST       /instances/admins/sign-out`
- ✅ `GET        /instances/admins/me`
- ✅ `GET/PATCH  /instances/admins/session`
- ✅ `GET        /instances/workspaces`
- ✅ `GET        /instances/workspace-slug-check`
- ✅ `POST       /instances/admins/sign-up-screen-visited`

## Users (`app/urls/user.py`)

- ✅ `GET/PATCH  /users/me`
- ✅ `GET/DELETE /users/session`
- ✅ `PATCH      /users/me/settings`
- ✅ `GET/PATCH  /users/me/profile`
- ✅ `GET        /users/me/accounts`
- ✅ `DELETE     /users/me/accounts/{pk}`
- ✅ `GET/POST   /users/me/instance-admin`
- ✅ `POST       /users/me/onboard`
- ✅ `POST       /users/me/tour-completed`
- ✅ `GET        /users/me/activities`
- ✅ `GET        /users/me/workspaces`
- ✅ `GET        /users/me/workspaces/{slug}/activity-graph`
- ✅ `GET        /users/me/workspaces/{slug}/issues-completed-graph`
- ✅ `GET        /users/me/workspaces/{slug}/dashboard`
- ✅ `GET/PATCH  /users/me/notification-preferences`
- ✅ `GET        /users/last-visited-workspace`
- ✅ `POST       /users/me/email/generate-code`
- ✅ `POST       /users/me/email`

## API Tokens (`app/urls/api.py`)

- ✅ `GET/POST   /api-tokens`
- ✅ `GET/PATCH/DELETE /api-tokens/{pk}`
- ✅ `GET/POST   /users/api-tokens` (alias)
- ✅ `GET/PATCH/DELETE /users/api-tokens/{pk}` (alias)

## Timezones

- ✅ `GET /timezones`

## Workspaces (`app/urls/workspace.py`)

- ✅ `GET        /workspace-slug-check`
- ✅ `GET/POST   /workspaces`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}`
- ✅ `GET/POST   /workspaces/{slug}/invitations`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/invitations/{pk}`
- ✅ `GET        /users/me/workspaces/invitations`
- ✅ `GET        /workspaces/{slug}/members`
- ✅ `GET        /workspaces/{slug}/project-members`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/members/{pk}`
- ✅ `POST       /workspaces/{slug}/members/leave`
- ✅ `GET        /workspaces/{slug}/workspace-members/me`
- ✅ `GET        /workspaces/{slug}/workspace-views`
- ✅ `GET        /workspaces/{slug}/user-stats/{user_id}`
- ✅ `GET        /workspaces/{slug}/user-activity/{user_id}`
- ✅ `GET        /workspaces/{slug}/user-activity/{user_id}/export`
- ✅ `GET        /workspaces/{slug}/user-profile/{user_id}`
- ✅ `GET        /workspaces/{slug}/user-issues/{user_id}`
- ✅ `GET        /workspaces/{slug}/labels`
- ✅ `GET/PATCH  /workspaces/{slug}/user-properties`
- ✅ `GET        /workspaces/{slug}/states`
- ✅ `GET        /workspaces/{slug}/estimates`
- ✅ `GET        /workspaces/{slug}/modules`
- ✅ `GET        /workspaces/{slug}/cycles`
- ✅ `GET/POST   /workspaces/{slug}/user-favorites`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/user-favorites/{favorite_id}`
- ✅ `GET/POST   /workspaces/{slug}/user-favorites/{favorite_id}/children`
- ✅ `GET        /workspaces/{slug}/user-favorites/{favorite_id}/group` (alias de /children)
- ✅ `GET/POST   /workspaces/{slug}/draft-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/draft-issues/{pk}`
- ✅ `POST       /workspaces/{slug}/draft-to-issue/{draft_id}`
- ✅ `GET/POST   /workspaces/{slug}/quick-links`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/quick-links/{pk}`
- ✅ `GET/PATCH  /workspaces/{slug}/home-preferences`
- ✅ `GET/PATCH  /workspaces/{slug}/home-preferences/{key}`
- ✅ `GET        /workspaces/{slug}/recent-visits`
- ✅ `GET/POST   /workspaces/{slug}/stickies`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/stickies/{pk}`
- ✅ `GET/PATCH  /workspaces/{slug}/sidebar-preferences`
- ✅ `POST       /workspaces/{slug}/invitations/{pk}/join`
- ✅ `GET/POST   /workspaces/{slug}/workspace-themes`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-themes/{pk}`

## Projects (`app/urls/project.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects`
- ✅ `GET        /workspaces/{slug}/projects/details`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{pk}`
- ✅ `GET/DELETE /workspaces/{slug}/project-identifiers`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/invitations`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/invitations/{pk}`
- ✅ `GET        /users/me/workspaces/{slug}/project-roles`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/members`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/members/{pk}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/members/leave`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-members/me`
- ✅ `GET        /users/me/workspaces/{slug}/projects/invitations`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/join/{pk}`

- ✅ `GET/POST  /workspaces/{slug}/projects/{project_id}/project-deploy-boards`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}`

## States (`app/urls/state.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/states`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/states/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intake-state`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default`

## Issues (`app/urls/issue.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issue-labels`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issue-labels/{pk}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/bulk-create-labels`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/bulk-delete-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/bulk-archive-issues`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}`
- ✅ `GET/POST   /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments`
- ✅ `DELETE     /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/user-properties`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{pk}/archive`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/deleted-issues`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issue-dates`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta`
- ✅ `GET        /workspaces/{slug}/work-items/{combined}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/list`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues-detail`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/v2/issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}`

## Cycles (`app/urls/cycle.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/cycles`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/cycles/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/cycles/date-check`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/user-favorite-cycles`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/user-favorite-cycles/{cycle_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-cycles`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}`

## Modules (`app/urls/module.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/modules/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/modules`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/user-favorite-modules`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/user-favorite-modules/{module_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-modules`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/archived-modules/{pk}`

## Estimates (`app/urls/estimate.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/estimates`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-estimates`

## Pages (`app/urls/page.py`)

- ✅ `GET        /workspaces/{slug}/projects/{project_id}/pages-summary`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/pages`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}`
- ✅ `POST/DELETE /workspaces/{slug}/projects/{project_id}/favorite-pages/{page_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive`
- ✅ `POST/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/pages/{page_id}/access`
- ✅ `GET/POST/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}/description`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate`

## Views (`app/urls/views.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/views`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/views/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/views`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/views/{pk}`
- ✅ `GET        /workspaces/{slug}/issues`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/user-favorite-views`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-views`

## Analytics (`app/urls/analytic.py`)

- ✅ `GET/POST   /workspaces/{slug}/analytics`
- ✅ `GET/POST   /workspaces/{slug}/analytic-view`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/analytic-view/{pk}`
- ✅ `GET/POST/PATCH/DELETE /workspaces/{slug}/saved-analytic-view/{analytic_id}`
- ✅ `POST       /workspaces/{slug}/export-analytics`
- ✅ `GET        /workspaces/{slug}/default-analytics`
- ✅ `GET        /workspaces/{slug}/project-stats`
- ✅ `GET        /workspaces/{slug}/advance-analytics`
- ✅ `GET        /workspaces/{slug}/advance-analytics-stats`
- ✅ `GET        /workspaces/{slug}/advance-analytics-charts`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/advance-analytics`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/advance-analytics-stats`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/advance-analytics-charts`

## Notifications (`app/urls/notification.py`)

- ✅ `GET        /workspaces/{slug}/users/notifications`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/users/notifications/{pk}`
- ✅ `POST       /workspaces/{slug}/users/notifications/{pk}/read`
- ✅ `POST       /workspaces/{slug}/users/notifications/{pk}/archive`
- ✅ `GET        /workspaces/{slug}/users/notifications/unread`
- ✅ `POST       /workspaces/{slug}/users/notifications/mark-all-read`
- ✅ `GET/PATCH  /users/me/notification-preferences`

## Search (`app/urls/search.py`)

- ✅ `GET        /workspaces/{slug}/search`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/search-issues`
- ✅ `GET        /workspaces/{slug}/entity-search`

## Assets v2 (`app/urls/asset.py`)

- ✅ `GET/POST   /assets/v2/workspaces/{slug}`
- ✅ `GET/PATCH/DELETE /assets/v2/workspaces/{slug}/{asset_id}`
- ✅ `GET/POST   /assets/v2/user-assets`
- ✅ `GET/PATCH/DELETE /assets/v2/user-assets/{asset_id}`
- ✅ `GET        /assets/v2/static/{asset_id}`
- ✅ `GET/POST   /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments`
- ✅ `DELETE     /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}`
- ✅ `POST       /assets/v2/workspaces/{slug}/restore/{asset_id}`
- ✅ `GET/POST   /assets/v2/workspaces/{slug}/projects/{project_id}`
- ✅ `GET/PATCH/DELETE /assets/v2/workspaces/{slug}/projects/{project_id}/{pk}`
- ✅ `POST       /assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk`
- ✅ `GET        /assets/v2/workspaces/{slug}/check/{asset_id}`
- ✅ `POST       /assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}`
- ✅ `GET        /assets/v2/workspaces/{slug}/download/{asset_id}`
- ✅ `GET        /assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}`

## Exporter (`app/urls/exporter.py`)

- ✅ `GET/POST   /workspaces/{slug}/export-issues`
- ✅ `GET        /workspaces/{slug}/export-issues/{token}`

## Intake (`app/urls/intake.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intakes`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intakes/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intake-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intake-issues/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/inboxes`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/inboxes/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/inbox-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/inbox-issues/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions/{pk}`

## Integrations (`app/urls/integration.py`)

- ✅ `GET        /github/callback`
- ✅ `GET        /github/user-callback`
- ✅ `GET        /integrations`
- ✅ `GET/POST   /workspaces/{slug}/workspace-integrations`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-integrations/{pk}`
- ✅ `GET        /workspaces/{slug}/workspace-integrations/{provider}/provider`
- ✅ `POST       /workspaces/{slug}/workspace-integrations/{provider}/install`
- ✅ `GET/POST   /workspaces/{slug}/workspace-integrations/github/repo-syncs`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}`
- ✅ `GET        /workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories`
- ✅ `GET        /workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories`
- ✅ `GET/POST   /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}`

## Importer (`app/urls/importer.py`)

- ✅ `GET        /workspaces/{slug}/importers/github/repositories`
- ✅ `GET/POST   /workspaces/{slug}/importers/github`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/importers/github/{importer_id}`
- ✅ `GET        /workspaces/{slug}/importers/gitlab/repositories`
- ✅ `GET/POST   /workspaces/{slug}/importers/gitlab`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/importers/gitlab/{importer_id}`

## External / AI (`app/urls/external.py`)

- ✅ `GET        /unsplash`
- ✅ `POST       /workspaces/{slug}/ai-assistant`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/ai-assistant`
- ✅ `POST       /workspaces/{slug}/rephrase-grammar`
- ✅ `POST       /github-webhook`  ← webhook entrante de GitHub
- ✅ `POST       /gitlab-webhook`  ← webhook entrante de GitLab

## Webhooks (`app/urls/webhook.py`)

- ✅ `GET/POST   /workspaces/{slug}/webhooks`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/webhooks/{pk}`
- ✅ `POST       /workspaces/{slug}/webhooks/{pk}/regenerate`
- ✅ `GET        /workspaces/{slug}/webhook-logs/{webhook_id}`


### Users

- ✅ `GET/PATCH  /users/me`

### Workspace Members

- ✅ `GET        /workspaces/{slug}/members`

### Projects

- ✅ `GET/POST   /workspaces/{slug}/projects`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{pk}`
- ✅ `POST/DELETE /workspaces/{slug}/projects/{project_id}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/summary`

### Project Members

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/members`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/members/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-members`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-members/{pk}`

### States

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/states`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/states/{state_id}`

### Labels

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/labels`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/labels/{pk}`

### Estimates

- ✅ `GET        /workspaces/{slug}/projects/{project_id}/estimates`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points`
- ✅ `PATCH/DELETE /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}`

### Cycles

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/cycles`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/cycles/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-cycles`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}/unarchive`

### Modules

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/modules/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues/{issue_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/modules/{pk}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-modules`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-modules/{pk}`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/archived-modules/{pk}/unarchive`

### Work Items — nuevo prefijo `/work-items/`

- ✅ `GET        /workspaces/{slug}/work-items/search`
- ✅ `GET        /workspaces/{slug}/work-items/{combined}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/work-items`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/work-items/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links`
- ✅ `PATCH/DELETE /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}` ← handler nuevo
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments`
- ✅ `PATCH/DELETE /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/relations`

### Work Items — prefijo legacy `/issues/`

- ✅ `GET        /workspaces/{slug}/issues/search`
- ✅ `GET        /workspaces/{slug}/issues/{combined}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links`
- ✅ `PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/{pk}` ← handler nuevo 
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments`
- ✅ `PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}`

### Intake Issues

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intake-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intake-issues/{pk}`

### Assets (api/v1 — sin prefijo v2/)

- ✅ `POST       /assets/user-assets`
- ✅ `PATCH/DELETE /assets/user-assets/{asset_id}`
- ✅ `POST       /assets/user-assets/server`
- ✅ `POST       /assets/user-assets/{asset_id}/server`
- ✅ `POST       /workspaces/{slug}/assets`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/assets/{asset_id}`

---