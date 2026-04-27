# Inventario de Endpoints — `api_rust`

---

## Estructura del crate `api_rust`

<details>
<summary>Tree completo del directorio (clic para expandir)</summary>

```
.
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── Dockerfile.dev
├── migration
│   ├── Cargo.toml
│   └── src
│       ├── lib.rs
│       ├── main.rs
│       ├── migrations
│       │   ├── m20240101_000007_seed_data.rs
│       │   ├── m20260410_000001_baseline.rs
│       │   └── mod.rs
│       └── sql
│           └── baseline.sql
├── rust-toolchain.toml
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
│   ├── lib.rs
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
│       ├── django_defaults.rs
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
├── tests
│   ├── analytics_tests.rs
│   ├── api_tokens.rs
│   ├── assets_v1_legacy.rs
│   ├── assets_v2_project.rs
│   ├── auth_csrf_and_email_check.rs
│   ├── auth_magic.rs
│   ├── auth_oauth.rs
│   ├── auth_password.rs
│   ├── auth_sign_in_up_out.rs
│   ├── auth_spaces.rs
│   ├── common
│   │   └── mod.rs
│   ├── cycles_extras.rs
│   ├── cycles_modules_extras.rs
│   ├── cycles.rs
│   ├── estimates.rs
│   ├── estimates_v1.rs
│   ├── external_ai.rs
│   ├── importers_crud.rs
│   ├── instances_admins.rs
│   ├── instances.rs
│   ├── intake.rs
│   ├── integrations_external.rs
│   ├── issue_attachments.rs
│   ├── issue_extras.rs
│   ├── issues_extras2.rs
│   ├── issues_extras3.rs
│   ├── issues.rs
│   ├── jobs_scheduler.rs
│   ├── labels.rs
│   ├── modules_extras.rs
│   ├── modules.rs
│   ├── notifications.rs
│   ├── pages_extras.rs
│   ├── pages.rs
│   ├── project_extras2.rs
│   ├── project_extras.rs
│   ├── projects.rs
│   ├── project_summary.rs
│   ├── search_tests.rs
│   ├── states.rs
│   ├── users_extended.rs
│   ├── users_me.rs
│   ├── users_settings.rs
│   ├── views.rs
│   ├── webhooks.rs
│   ├── work_items_extras.rs
│   ├── work_items.rs
│   ├── workspace_extras2.rs
│   ├── workspace_extras.rs
│   ├── workspace_integrations_advanced.rs
│   ├── workspace_invitations.rs
│   ├── workspace_members.rs
│   ├── workspaces.rs
│   ├── workspace_themes.rs
│   └── workspace_user_profiles.rs
└── todo_test.md

14 directories, 284 files
```

---

## 1. Autenticación (`/auth/*`)

Rutas con rate limiting. Todas devuelven cookies de sesión cuando exitosas.

### 1.1 Sesión / CSRF ✅ Revisado

| Método | Path                   | Estado | Notas                                                                              |
| ------ | ---------------------- | ------ | ---------------------------------------------------------------------------------- |
| GET    | `/auth/get-csrf-token` | ✅ OK  | Contrato correcto. Cookie + body `{ csrf_token }`. Timing-safe con openssl::memcmp. |
| POST   | `/auth/sign-in`        | ✅ OK  | Form-body `{ email, password, next_path }`. Contrato correcto.                     |
| POST   | `/auth/sign-up`        | ✅ OK  | Mismo form. Contrato correcto.                                                     |
| POST   | `/auth/sign-out`       | ✅ OK  | Acepta `csrfmiddlewaretoken` en form-body — compatible con patrón Django del frontend. |
| POST   | `/auth/email-check`    | 🐛 FIXED | **Faltaba `is_password_autoset`** en respuesta. Corregido en fa067a8.             |

### 1.2 Magic link ✅ Revisado

| Método | Path                                                     | Estado | Notas |
| ------ | -------------------------------------------------------- | ------ | ----- |
| POST   | `/auth/magic-generate` (+ `/auth/spaces/magic-generate`) | ✅ OK  | JSON body `{ email }`. Respuesta `{ key }` ignorada por frontend. Rutas registradas. |
| POST   | `/auth/magic-sign-in` (+ `/auth/spaces/magic-sign-in`)   | 🐛 FIXED | Comparación de código no era constant-time. Corregido con `is_valid_csrf()` (751e998). |
| POST   | `/auth/magic-sign-up` (+ `/auth/spaces/magic-sign-up`)   | 🐛 FIXED | Mismo fix. Form-body `{ email, code, next_path }`. Contrato correcto. |

### 1.3 Password ✅ Revisado

| Método | Path                                                       | Estado | Notas |
| ------ | ---------------------------------------------------------- | ------ | ----- |
| POST   | `/auth/change-password`                                    | ✅ OK  | JSON + X-CSRFTOKEN header. `old_password` opcional si `is_password_autoset`. |
| POST   | `/auth/set-password`                                       | ✅ OK  | JSON + X-CSRFTOKEN header. Guard correcto para usuarios con password real. |
| POST   | `/auth/forgot-password` (+ `/auth/spaces/forgot-password`) | ✅ OK  | JSON `{ email }`. Token almacenado en Redis 24h. |
| POST   | `/auth/reset-password/{uidb64}/{token}` (+ versión spaces) | 🐛 FIXED | `constant_time_eq` custom reemplazado por `openssl::memcmp` (a26a455). Form-body correcto. |

### 1.4 OAuth (initiate / callback) ✅ Revisado

| Método     | Path                         | Estado | Notas |
| ---------- | ---------------------------- | ------ | ----- |
| GET        | `/auth/gitlab`               | ✅ OK  | Correcto. |
| GET        | `/auth/gitlab/callback`      | ✅ OK  | Correcto. |
| GET        | `/auth/google`               | ✅ OK  | Correcto. |
| GET        | `/auth/google/callback`      | ✅ OK  | Correcto. |
| GET        | `/auth/gitea`                | ✅ OK  | Correcto. |
| GET        | `/auth/gitea/callback`       | ✅ OK  | Correcto. |
| GET        | `/auth/github`               | 🐛 FIXED | **Endpoint faltante** — implementado `github_initiate` (fc42528). |
| GET        | `/auth/github/callback`      | 🐛 FIXED | Estaba mapeado al handler de GitHub App. Reemplazado con `github_auth_callback` con fallback a `/user/emails` (fc42528). |
| GET / POST | `/auth/github/user-callback` | 🐛 FIXED | Path mismatch `/api/auth/...` vs `/auth/...` + faltaba `provider` en respuesta (6c0bea4). |
| GET        | `/github/callback`           | ✅ OK  | GitHub App setup callback correcto. |

### 1.5 God Mode (admin) ✅ Revisado

| Método | Path                             | Estado | Notas |
| ------ | -------------------------------- | ------ | ----- |
| POST   | `/api/instances/admins/sign-up`  | 🐛 FIXED | `is_telemetry_enabled` case-insensitive — frontend envía "True" no "true" (c73c217). |
| POST   | `/api/instances/admins/sign-in`  | ✅ OK  | Form-body correcto. |
| POST   | `/api/instances/admins/sign-out` | ✅ OK  | CSRF validado correctamente. |

---

## 2. Health / Instance / Timezones ✅ Revisado

| Método      | Path                                                  | Auth  | Estado | Notas |
| ----------- | ----------------------------------------------------- | ----- | ------ | ----- |
| GET         | `/api/health`                                         | No    | ✅ OK  | Correcto. |
| GET         | `/api/timezones`                                      | Sí    | ✅ OK  | Respuesta `{ timezones: [] }` coincide con `TTimezones`. |
| GET / PATCH | `/api/instances`                                      | Admin | ✅ OK  | `IInstanceInfo { instance, config }` completo. |
| POST        | `/api/instances/admins/sign-up-screen-visited`        | Admin | ✅ OK  | Correcto. |
| GET / POST  | `/api/instances/admins`                               | Admin | 🐛 FIXED | `user_detail` faltaba `display_name`, `avatar_url`, `is_bot`; `created_by`/`updated_by` faltantes (7e16041). |
| GET         | `/api/instances/admins/me`                            | Admin | 🐛 FIXED | Retornaba `avatar` en vez de `avatar_url`, faltaban campos `IUser` (7e16041). |
| GET         | `/api/instances/admins/session`                       | Admin | ✅ OK  | Correcto. |
| DELETE      | `/api/instances/admins/{pk}`                          | Admin | ✅ OK  | Soft delete correcto. |
| GET / PATCH | `/api/instances/configurations`                       | Admin | ✅ OK  | Cifrado/descifrado correcto. |
| DELETE      | `/api/instances/configurations/disable-email-feature` | Admin | ✅ OK  | Correcto. |
| POST        | `/api/instances/email-credentials-check`              | Admin | ✅ OK  | `{ receiver_email }` correcto. |
| GET         | `/api/instances/workspace-slug-check`                 | Admin | ✅ OK  | Correcto. |
| GET         | `/api/instances/workspaces`                           | Admin | ✅ OK  | Correcto. |
| GET         | `/api/instances/changelog`                            | No    | ℹ️ N/A | Método `changelog()` definido en servicio pero nunca llamado en el frontend. `instance_changelog_url` se expone via `/api/instances`. |

---

## 3. Users (current user / `me`) ✅ Revisado

Todos requieren sesión activa.

| Método               | Path                                                     | Estado | Notas |
| -------------------- | -------------------------------------------------------- | ------ | ----- |
| GET / PATCH / DELETE | `/api/users/me`                                          | ✅ OK  | DELETE soft-delete. IUser contract correcto. |
| GET                  | `/api/users/session`                                     | ✅ OK  | Correcto. |
| GET                  | `/api/users/me/settings`                                 | ✅ OK  | `IUserSettings` completo con fallback workspace. |
| GET                  | `/api/users/me/instance-admin`                           | ✅ OK  | `{ is_instance_admin: bool }` correcto. |
| GET / PATCH          | `/api/users/me/notification-preferences`                 | ✅ OK  | get_or_create correcto, nunca 404. |
| PATCH                | `/api/users/me/onboard`                                  | ✅ OK  | `{ is_onboarded: true }` correcto. |
| PATCH                | `/api/users/me/tour-completed`                           | ✅ OK  | Correcto. |
| GET / PATCH          | `/api/users/me/profile`                                  | ✅ OK  | `TUserProfile` correcto. |
| GET                  | `/api/users/me/accounts`                                 | ✅ OK  | Correcto. |
| GET / DELETE         | `/api/users/me/accounts/{pk}`                            | ✅ OK  | Correcto. |
| GET                  | `/api/users/last-visited-workspace`                      | ✅ OK  | Correcto. |
| GET                  | `/api/users/me/workspaces`                               | ✅ OK  | Correcto. |
| GET                  | `/api/users/me/activities`                               | ✅ OK  | Cursor paginado, `IUserActivityResponse` correcto. |
| GET / POST           | `/api/users/me/workspaces/invitations`                   | ✅ OK  | POST bulk-accept correcto. |
| GET                  | `/api/users/me/workspaces/{slug}/project-roles`          | ✅ OK  | Correcto. |
| GET                  | `/api/users/me/workspaces/{slug}/activity-graph`         | ✅ OK  | Correcto. |
| GET                  | `/api/users/me/workspaces/{slug}/issues-completed-graph` | ✅ OK  | Correcto. |
| GET                  | `/api/users/me/workspaces/{slug}/dashboard`              | ✅ OK  | Correcto. |
| POST                 | `/api/users/me/email/generate-code`                      | ✅ OK  | `{ email }` → código Redis TTL 10min. |
| POST / PATCH         | `/api/users/me/email`                                    | 🐛 FIXED | Frontend usa PATCH, Rust solo tenía POST. Añadido alias PATCH (0b3b86f). |
| GET / POST           | `/api/users/me/workspaces/{slug}/projects/invitations`   | 🐛 FIXED | POST bulk-join faltaba. Implementado `join_user_project_invitations` con `{ project_ids[] }` (0b3b86f). |

---

## 4. Workspaces ✅ Revisado

| Método               | Path                                                    | Estado | Notas |
| -------------------- | ------------------------------------------------------- | ------ | ----- |
| GET                  | `/api/workspace-slug-check`                             | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces`                                       | ✅ OK  | Correcto. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}`                                | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/members`                        | ✅ OK  | Correcto. |
| POST                 | `/api/workspaces/{slug}/members/leave`                  | ✅ OK  | Correcto. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/members/{pk}`                   | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/project-members`                | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/workspace-views`                | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/workspace-members/me`           | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/user-profile/{user_id}`         | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/user-stats/{user_id}`           | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/user-activity/{user_id}`        | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/user-activity/{user_id}/export` | ✅ OK  | CSV correcto. |
| GET                  | `/api/workspaces/{slug}/user-issues/{user_id}`          | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/invitations`                    | ✅ OK  | Correcto. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/invitations/{pk}`               | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/invitations/{pk}/join`          | 🐛 FIXED | GET faltaba (página pública de accept/reject). Timing attack en token comparison. Ambos corregidos (a94766f). |
| GET / POST           | `/api/workspaces/{slug}/workspace-themes`               | ✅ OK  | Correcto. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/workspace-themes/{pk}`          | ✅ OK  | Correcto. |

### 4.1 Workspace extras (favorites, home, sidebar, stickies, drafts) ✅ Revisado

| Método               | Path                                                                            | Estado |
| -------------------- | ------------------------------------------------------------------------------- | ------ |
| GET / POST           | `/api/workspaces/{slug}/user-favorites`                                         | ✅ OK  |
| PATCH / DELETE       | `/api/workspaces/{slug}/user-favorites/{favorite_id}`                           | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/user-favorites/{favorite_id}/children`                  | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/user-favorites/{favorite_id}/group` (alias `/children`) | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/home-preferences`                                       | ✅ OK  |
| GET / PATCH          | `/api/workspaces/{slug}/home-preferences/{key}`                                 | ✅ OK  |
| GET / POST           | `/api/workspaces/{slug}/quick-links`                                            | ✅ OK  |
| PATCH / DELETE       | `/api/workspaces/{slug}/quick-links/{pk}`                                       | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/recent-visits`                                          | ✅ OK  |
| GET / POST           | `/api/workspaces/{slug}/stickies`                                               | ✅ OK  |
| PATCH / DELETE       | `/api/workspaces/{slug}/stickies/{pk}`                                          | ✅ OK  |
| GET / PATCH          | `/api/workspaces/{slug}/sidebar-preferences`                                    | ✅ OK  |
| GET / PATCH          | `/api/workspaces/{slug}/user-properties`                                        | ✅ OK  |
| GET / POST           | `/api/workspaces/{slug}/draft-issues`                                           | ✅ OK  |
| POST                 | `/api/workspaces/{slug}/draft-to-issue/{draft_id}`                              | ✅ OK  |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/draft-issues/{pk}`                                      | ✅ OK  |

### 4.2 Workspace aggregate (read-only, cross-project) ✅ Revisado

| Método | Path                               | Estado |
| ------ | ---------------------------------- | ------ |
| GET    | `/api/workspaces/{slug}/cycles`    | ✅ OK  |
| GET    | `/api/workspaces/{slug}/modules`   | ✅ OK  |
| GET    | `/api/workspaces/{slug}/estimates` | ✅ OK  |
| GET    | `/api/workspaces/{slug}/labels`    | ✅ OK  |
| GET    | `/api/workspaces/{slug}/states`    | ✅ OK  |
| GET    | `/api/workspaces/{slug}/issues`    | ✅ OK  |

---

## 5. Projects ✅ Revisado

| Método               | Path                                                                          | Estado | Notas |
| -------------------- | ----------------------------------------------------------------------------- | ------ | ----- |
| GET / POST           | `/api/workspaces/{slug}/projects`                                             | ✅ OK  | `IPartialProject` completo. |
| GET                  | `/api/workspaces/{slug}/projects/details`                                     | ✅ OK  | Literal registrado antes de `/{project_id}`. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}`                                | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/members`                        | ✅ OK  | Correcto. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/members/{pk}`                   | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/project-members/me`             | ✅ OK  | Correcto. |
| POST                 | `/api/workspaces/{slug}/projects/{project_id}/members/leave`                  | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/project-views`                  | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/summary`                        | ✅ OK  | Correcto. |
| POST / DELETE        | `/api/workspaces/{slug}/projects/{project_id}/archive`                        | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/user-favorite-projects`                               | ✅ OK  | Correcto. |
| DELETE               | `/api/workspaces/{slug}/user-favorite-projects/{project_id}`                  | ✅ OK  | Correcto. |
| GET / DELETE         | `/api/workspaces/{slug}/project-identifiers`                                  | ✅ OK  | `{ exists: count, identifiers: [] }` correcto. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards`          | 🐛 FIXED | `project_id`→`project`, `workspace_id`→`workspace`; faltaban `created_by`, `updated_by`, `inbox`, `project_details`, `workspace_detail` (430a23a). |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}`     | 🐛 FIXED | Mismo fix struct. |
| GET / PATCH          | `/api/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}` | ✅ OK  | Correcto. |
| GET / PATCH          | `/api/workspaces/{slug}/projects/{project_id}/user-properties`                | ✅ OK  | get_or_create correcto. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/invitations`                    | ✅ OK  | Correcto. |
| GET / DELETE         | `/api/workspaces/{slug}/projects/{project_id}/invitations/{pk}`               | ✅ OK  | Correcto. |
| POST                 | `/api/workspaces/{slug}/projects/{project_id}/join/{pk}`                      | ✅ OK  | Correcto. |

---

## 6. States, Labels, Estimates ✅ Revisado

### 6.1 States ✅

| Método               | Path                                                                    | Estado | Notas |
| -------------------- | ----------------------------------------------------------------------- | ------ | ----- |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/states`                   | 🐛 FIXED | Faltaba campo `order` en `StateResponse` (usado en `StateGroupIcon`). Añadido como alias de `sequence` (d741df7). |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/states/{pk}`              | ✅ OK  | Correcto. |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/intake-state`             | ✅ OK  | Correcto. |
| POST                 | `/api/workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default` | ✅ OK  | Correcto. |

### 6.2 Labels ✅

| Método               | Path                                                              | Estado | Notas |
| -------------------- | ----------------------------------------------------------------- | ------ | ----- |
| POST                 | `/api/workspaces/{slug}/projects/{project_id}/bulk-create-labels` | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/labels`             | 🐛 FIXED | `parent_id` → serde rename a `parent` (IIssueLabel.parent). (d741df7). |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/labels/{pk}`        | 🐛 FIXED | Mismo fix. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/issue-labels`       | 🐛 FIXED | Alias, mismo fix. |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/issue-labels/{pk}`  | 🐛 FIXED | Alias, mismo fix. |

### 6.3 Estimates ✅

| Método               | Path                                                                                        | Estado | Notas |
| -------------------- | ------------------------------------------------------------------------------------------- | ------ | ----- |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/project-estimates`                            | ✅ OK  | Correcto. |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/estimates`                                    | 🐛 FIXED | `project_id`→`project`, `workspace_id`→`workspace`, `created_by_id`→`created_by`; faltaba `updated_by` (d741df7). |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}`                      | 🐛 FIXED | Mismo fix struct. |
| POST                 | `/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points`      | ✅ OK  | Correcto. |
| PATCH / DELETE       | `/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}` | ✅ OK  | Correcto. |

---

## 7. Issues / Work Items (núcleo) ✅ Revisado

### 7.1 CRUD básico ✅

| Método               | Path                                                                    | Estado |
| -------------------- | ----------------------------------------------------------------------- | ------ |
| GET / POST           | `/api/workspaces/{slug}/projects/{project_id}/issues`                   | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/issues/list`              | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/issues-detail`            | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/projects/{project_id}/v2/issues`                | ✅ OK  |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/issues/{pk}`              | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/work-items/{combined}` (lookup por `PROJ-1234`) | ✅ OK  |
| GET                  | `/api/workspaces/{slug}/issues/{combined}` (alias)                      | ✅ OK  |

### 7.2 Comments / Reactions / Links / Relations / Subscribers ✅

| Método               | Path                                                        | Estado | Notas |
| -------------------- | ----------------------------------------------------------- | ------ | ----- |
| GET / POST           | `…/issues/{issue_id}/comments`                              | ✅ OK  |       |
| GET / PATCH / DELETE | `…/issues/{issue_id}/comments/{pk}`                         | ✅ OK  |       |
| GET / POST           | `…/issues/{issue_id}/reactions`                             | ✅ OK  |       |
| DELETE               | `…/issues/{issue_id}/reactions/{reaction_code}`             | ✅ OK  |       |
| GET / POST           | `…/comments/{comment_id}/reactions`                         | ✅ OK  |       |
| DELETE               | `…/comments/{comment_id}/reactions/{reaction_code}`         | ✅ OK  |       |
| GET / POST           | `…/issues/{issue_id}/issue-links`                           | ✅ OK  |       |
| PATCH / DELETE       | `…/issues/{issue_id}/issue-links/{pk}`                      | ✅ OK  |       |
| GET / POST           | `…/issues/{issue_id}/links` (alias)                         | ✅ OK  |       |
| PATCH / DELETE       | `…/issues/{issue_id}/links/{pk}`                            | ✅ OK  |       |
| GET / POST           | `…/work-items/{issue_id}/links`                             | ✅ OK  |       |
| PATCH / DELETE       | `…/work-items/{issue_id}/links/{pk}`                        | ✅ OK  |       |
| GET / POST           | `…/issues/{issue_id}/issue-relation`                        | ✅ OK  |       |
| POST                 | `…/issues/{issue_id}/remove-relation`                       | ✅ OK  |       |
| GET                  | `…/issues/{issue_id}/issue-subscribers`                     | ✅ OK  |       |
| DELETE               | `…/issues/{issue_id}/issue-subscribers/{subscriber_id}`     | ✅ OK  |       |
| GET / POST / DELETE  | `…/issues/{issue_id}/subscribe`                             | 🐛 FIXED | GET faltaba — status check `{ subscribed: bool }` (2d3140e). |
| GET / POST           | `…/issues/{issue_id}/sub-issues`                            | ✅ OK  |       |

### 7.3 Activity / History / Versions ✅

| Método | Path                                                           | Estado | Notas |
| ------ | -------------------------------------------------------------- | ------ | ----- |
| GET    | `…/issues/{issue_id}/activities` (alias `/history`)            | 🐛 FIXED | `ActivityResponse` faltaba `actor_detail`, `project_detail`, `workspace_detail`, renames `actor`/`issue`/`project`. (ecc1a02). |
| GET    | `…/issues/{issue_id}/history`                                  | 🐛 FIXED | Mismo fix. |
| GET    | `…/issues/{issue_id}/activities/{pk}`                          | 🐛 FIXED | Mismo fix. |
| GET    | `…/work-items/{issue_id}/activities`                           | 🐛 FIXED | Mismo fix. |
| GET    | `…/work-items/{issue_id}/activities/{pk}`                      | 🐛 FIXED | Mismo fix. |
| GET    | `…/issues/{issue_id}/versions`                                 | ✅ OK  |       |
| GET    | `…/issues/{issue_id}/versions/{pk}`                            | ✅ OK  |       |
| GET    | `…/work-items/{work_item_id}/description-versions`             | ✅ OK  |       |
| GET    | `…/work-items/{work_item_id}/description-versions/{pk}`        | ✅ OK  |       |
| GET    | `…/intake-work-items/{work_item_id}/description-versions`      | ✅ OK  |       |
| GET    | `…/intake-work-items/{work_item_id}/description-versions/{pk}` | ✅ OK  |       |

### 7.4 Attachments (V2) ✅

| Método         | Path                                                                                        | Estado | Notas |
| -------------- | ------------------------------------------------------------------------------------------- | ------ | ----- |
| GET / POST     | `/api/assets/v2/…/issues/{issue_id}/attachments`                                            | ✅ OK  |       |
| PATCH / DELETE | `/api/assets/v2/…/issues/{issue_id}/attachments/{pk}`                                       | ✅ OK  |       |
| GET / POST     | `/api/assets/v2/…/work-items/{issue_id}/attachments`                                        | 🐛 FIXED | Alias faltaba para serviceType='work-items' (2472b0d). |
| PATCH / DELETE | `/api/assets/v2/…/work-items/{issue_id}/attachments/{pk}`                                   | 🐛 FIXED | Alias faltaba (2472b0d). |
| GET / POST     | `…/issues/{issue_id}/issue-attachments` (legacy)                                            | ✅ OK  |       |
| PATCH / DELETE | `…/issues/{issue_id}/issue-attachments/{pk}`                                                | ✅ OK  |       |

### 7.5 Archive / Bulk ✅

| Método              | Path                       | Estado | Notas |
| ------------------- | -------------------------- | ------ | ----- |
| GET / POST / DELETE | `…/issues/{pk}/archive`    | ✅ OK  |       |
| GET                 | `…/archived-issues`        | ✅ OK  |       |
| GET                 | `…/deleted-issues`         | ✅ OK  |       |
| DELETE / POST       | `…/bulk-delete-issues`     | ✅ OK  |       |
| GET                 | `…/issues/{issue_id}/meta` | ✅ OK  |       |
| POST                | `…/bulk-archive-issues`    | ✅ OK  |       |
| POST                | `…/bulk-operation-issues`  | 🐛 FIXED | Faltaba — llamado por bulkUpdateProperties del store (2d3140e). |
| POST                | `…/issue-dates`            | ✅ OK  |       |

---

## 8. Cycles

| Método               | Path                                          |
| -------------------- | --------------------------------------------- |
| GET / POST           | `…/projects/{project_id}/cycles`              |
| GET / PATCH / DELETE | `…/cycles/{pk}`                               |
| GET / POST           | `…/cycles/{cycle_id}/cycle-issues`            |
| DELETE               | `…/cycles/{cycle_id}/cycle-issues/{issue_id}` |
| GET                  | `…/cycles/{cycle_id}/analytics`               |
| GET / PATCH          | `…/cycles/{cycle_id}/user-properties`         |
| GET                  | `…/cycles/{cycle_id}/progress`                |
| POST                 | `…/cycles/date-check`                         |
| GET / POST           | `…/user-favorite-cycles`                      |
| DELETE               | `…/user-favorite-cycles/{cycle_id}`           |
| POST                 | `…/cycles/{cycle_id}/transfer-issues`         |
| POST / DELETE        | `…/cycles/{cycle_id}/archive`                 |
| GET                  | `…/archived-cycles`                           |
| GET / DELETE         | `…/archived-cycles/{pk}`                      |

---

## 9. Modules

| Método               | Path                                      |
| -------------------- | ----------------------------------------- |
| GET / POST           | `…/projects/{project_id}/modules`         |
| GET / PATCH / DELETE | `…/modules/{pk}`                          |
| GET / POST           | `…/modules/{module_id}/issues`            |
| DELETE               | `…/modules/{module_id}/issues/{issue_id}` |
| GET / PATCH          | `…/modules/{module_id}/user-properties`   |
| POST                 | `…/issues/{issue_id}/modules`             |
| GET / POST           | `…/modules/{module_id}/module-links`      |
| GET / PATCH / DELETE | `…/modules/{module_id}/module-links/{pk}` |
| GET / POST           | `…/user-favorite-modules`                 |
| DELETE               | `…/user-favorite-modules/{module_id}`     |
| POST / DELETE        | `…/modules/{module_id}/archive`           |
| GET                  | `…/archived-modules`                      |
| GET / DELETE         | `…/archived-modules/{pk}`                 |

---

## 10. Pages

| Método               | Path                                    |
| -------------------- | --------------------------------------- |
| GET                  | `…/projects/{project_id}/pages-summary` |
| GET / POST           | `…/projects/{project_id}/pages`         |
| GET / PATCH / DELETE | `…/pages/{page_id}`                     |
| POST                 | `…/pages/{page_id}/access`              |
| POST / DELETE        | `…/favorite-pages/{page_id}`            |
| POST / DELETE        | `…/pages/{page_id}/archive`             |
| POST / DELETE        | `…/pages/{page_id}/lock`                |
| POST                 | `…/pages/{page_id}/duplicate`           |
| GET                  | `…/pages/{page_id}/versions`            |
| GET                  | `…/pages/{page_id}/versions/{pk}`       |
| GET / PATCH          | `…/pages/{page_id}/description`         |

---

## 11. Intake (Inbox)

| Método               | Path                                            |
| -------------------- | ----------------------------------------------- |
| GET / POST           | `…/intakes` (+ `/inboxes`)                      |
| GET / PATCH / DELETE | `…/intakes/{pk}` (+ `/inboxes/{pk}`)            |
| GET / POST           | `…/intake-issues` (+ `/inbox-issues`)           |
| GET / PATCH / DELETE | `…/intake-issues/{pk}` (+ `/inbox-issues/{pk}`) |

---

## 12. Views

| Método               | Path                                                    |
| -------------------- | ------------------------------------------------------- |
| GET / POST           | `/api/workspaces/{slug}/views`                          |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/views/{pk}`                     |
| GET / POST           | `…/projects/{project_id}/views`                         |
| GET / PATCH / DELETE | `…/projects/{project_id}/views/{pk}`                    |
| GET / POST           | `…/projects/{project_id}/user-favorite-views`           |
| DELETE               | `…/projects/{project_id}/user-favorite-views/{view_id}` |

---

## 13. Notifications

| Método               | Path                                  |
| -------------------- | ------------------------------------- |
| GET                  | `…/users/notifications/unread`        |
| POST                 | `…/users/notifications/mark-all-read` |
| GET                  | `…/users/notifications`               |
| GET / PATCH / DELETE | `…/users/notifications/{pk}`          |
| POST / DELETE        | `…/users/notifications/{pk}/read`     |
| POST / DELETE        | `…/users/notifications/{pk}/archive`  |

---

## 14. Webhooks

| Método               | Path                                               |
| -------------------- | -------------------------------------------------- |
| GET / POST           | `/api/workspaces/{slug}/webhooks`                  |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/webhooks/{pk}`             |
| POST                 | `/api/workspaces/{slug}/webhooks/{pk}/regenerate`  |
| GET                  | `/api/workspaces/{slug}/webhook-logs/{webhook_id}` |
| POST                 | `/api/github-webhook` (público, externo)           |
| POST                 | `/api/gitlab-webhook` (público, externo)           |

---

## 15. Search / Exporter / Importer

### 15.1 Search

| Método | Path                                                  |
| ------ | ----------------------------------------------------- |
| GET    | `/api/workspaces/{slug}/search`                       |
| GET    | `/api/workspaces/{slug}/work-items/search`            |
| GET    | `/api/workspaces/{slug}/issues/search` (alias legacy) |
| GET    | `…/projects/{project_id}/search-issues`               |
| GET    | `/api/workspaces/{slug}/entity-search`                |

### 15.2 Exporter

| Método     | Path                                           |
| ---------- | ---------------------------------------------- |
| GET / POST | `/api/workspaces/{slug}/export-issues`         |
| GET        | `/api/workspaces/{slug}/export-issues/{token}` |

### 15.3 Importer (GitHub / GitLab)

| Método     | Path                               |
| ---------- | ---------------------------------- |
| GET        | `…/importers/github/repositories`  |
| GET / POST | `…/importers/github`               |
| DELETE     | `…/importers/github/{importer_id}` |
| GET        | `…/importers/gitlab/repositories`  |
| GET / POST | `…/importers/gitlab`               |
| DELETE     | `…/importers/gitlab/{importer_id}` |

---

## 16. Analytics

### 16.1 Workspace analytics (legacy)

| Método               | Path                                                       |
| -------------------- | ---------------------------------------------------------- |
| GET                  | `/api/workspaces/{slug}/analytics`                         |
| GET                  | `/api/workspaces/{slug}/default-analytics`                 |
| GET                  | `/api/workspaces/{slug}/project-stats`                     |
| POST                 | `/api/workspaces/{slug}/export-analytics`                  |
| GET / POST           | `/api/workspaces/{slug}/analytic-view`                     |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/analytic-view/{pk}`                |
| GET                  | `/api/workspaces/{slug}/saved-analytic-view/{analytic_id}` |

### 16.2 Advance analytics

| Método | Path                                               |
| ------ | -------------------------------------------------- |
| GET    | `/api/workspaces/{slug}/advance-analytics`         |
| GET    | `/api/workspaces/{slug}/advance-analytics-stats`   |
| GET    | `/api/workspaces/{slug}/advance-analytics-charts`  |
| GET    | `…/projects/{project_id}/advance-analytics`        |
| GET    | `…/projects/{project_id}/advance-analytics-stats`  |
| GET    | `…/projects/{project_id}/advance-analytics-charts` |

---

## 17. Assets (V2)

| Método               | Path                                                                         |
| -------------------- | ---------------------------------------------------------------------------- |
| POST                 | `/api/assets/v2/user-assets`                                                 |
| PATCH / DELETE       | `/api/assets/v2/user-assets/{asset_id}`                                      |
| POST                 | `/api/assets/v2/workspaces/{slug}`                                           |
| GET / PATCH / DELETE | `/api/assets/v2/workspaces/{slug}/{asset_id}`                                |
| GET                  | `/api/assets/v2/static/{asset_id}`                                           |
| POST                 | `/api/assets/v2/workspaces/{slug}/restore/{asset_id}`                        |
| GET                  | `/api/assets/v2/workspaces/{slug}/check/{asset_id}`                          |
| POST                 | `/api/assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}`               |
| GET                  | `/api/assets/v2/workspaces/{slug}/download/{asset_id}`                       |
| POST                 | `/api/assets/v2/workspaces/{slug}/projects/{project_id}`                     |
| GET / PATCH / DELETE | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}`                |
| POST                 | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk`    |
| GET                  | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}` |

---

## 18. External (AI / Unsplash)

| Método | Path                                                        |
| ------ | ----------------------------------------------------------- |
| GET    | `/api/unsplash`                                             |
| POST   | `/api/workspaces/{slug}/projects/{project_id}/ai-assistant` |
| POST   | `/api/workspaces/{slug}/ai-assistant`                       |
| POST   | `/api/workspaces/{slug}/rephrase-grammar`                   |

---

## 19. Integrations (GitHub / GitLab)

| Método               | Path                                                      |
| -------------------- | --------------------------------------------------------- |
| GET                  | `/api/integrations`                                       |
| GET / POST           | `…/workspace-integrations`                                |
| GET / PATCH / DELETE | `…/workspace-integrations/{pk}`                           |
| DELETE               | `…/workspace-integrations/{provider}/provider`            |
| POST                 | `…/workspace-integrations/{provider}/install`             |
| GET                  | `…/workspace-integrations/{wi_id}/github-repositories`    |
| GET                  | `…/workspace-integrations/{wi_id}/gitlab-repositories`    |
| GET / POST           | `…/workspace-integrations/github/repo-syncs`              |
| DELETE               | `…/workspace-integrations/github/repo-syncs/{pk}`         |
| GET / POST           | `…/workspace-integrations/{wi_id}/pr-state-mappings`      |
| DELETE               | `…/workspace-integrations/{wi_id}/pr-state-mappings/{pk}` |

---

## 20. API Tokens (sesión-based)

| Método               | Path                                 |
| -------------------- | ------------------------------------ |
| GET / POST           | `/api/api-tokens`                    |
| GET / PATCH / DELETE | `/api/api-tokens/{pk}`               |
| GET / POST           | `/api/users/api-tokens` (alias)      |
| GET / PATCH / DELETE | `/api/users/api-tokens/{pk}` (alias) |

---

## 21. API v1 pública (`/api/v1/*`)

``
/api/v1/users/me GET, PATCH
/api/v1/workspaces/{slug}/members GET
/api/v1/workspaces/{slug}/projects GET, POST
/api/v1/workspaces/{slug}/projects/{pk} GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/archive POST, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/summary GET
/api/v1/workspaces/{slug}/projects/{project_id}/members GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/members/{pk} GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/project-members GET
/api/v1/workspaces/{slug}/projects/{project_id}/project-members/{pk} GET
/api/v1/workspaces/{slug}/projects/{project_id}/states GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/states/{state_id} GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/labels GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/labels/{pk} GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/estimates GET
/api/v1/.../estimates/{estimate_id}/estimate-points POST
/api/v1/.../estimates/{estimate_id}/estimate-points/{pk} PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/cycles GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/cycles/{pk} GET, PATCH, DELETE
/api/v1/.../cycles/{cycle_id}/cycle-issues GET, POST
/api/v1/.../cycles/{cycle_id}/cycle-issues/{issue_id} DELETE
/api/v1/.../cycles/{cycle_id}/transfer-issues POST
/api/v1/.../cycles/{cycle_id}/archive POST
/api/v1/.../archived-cycles GET
/api/v1/.../archived-cycles/{pk} GET
/api/v1/.../archived-cycles/{pk}/unarchive DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/modules GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/modules/{pk} GET, PATCH, DELETE
/api/v1/.../modules/{module_id}/module-issues GET, POST
/api/v1/.../modules/{module_id}/module-issues/{issue_id} DELETE
/api/v1/.../modules/{pk}/archive POST
/api/v1/.../archived-modules GET
/api/v1/.../archived-modules/{pk} GET
/api/v1/.../archived-modules/{pk}/unarchive DELETE
/api/v1/workspaces/{slug}/work-items/search GET
/api/v1/workspaces/{slug}/work-items/{combined} GET
/api/v1/workspaces/{slug}/projects/{project_id}/work-items GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/work-items/{pk} GET, PATCH, DELETE
/api/v1/.../work-items/{issue_id}/links GET, POST
/api/v1/.../work-items/{issue_id}/links/{pk} PATCH, DELETE
/api/v1/.../work-items/{issue_id}/comments GET, POST
/api/v1/.../work-items/{issue_id}/comments/{pk} GET, PATCH, DELETE
/api/v1/.../work-items/{issue_id}/activities GET
/api/v1/.../work-items/{issue_id}/activities/{pk} GET
/api/v1/.../work-items/{issue_id}/attachments GET, POST
/api/v1/.../work-items/{issue_id}/attachments/{pk} PATCH, DELETE
/api/v1/.../work-items/{issue_id}/relations GET, POST
/api/v1/workspaces/{slug}/issues/search GET (alias legacy)
/api/v1/workspaces/{slug}/issues/{combined} GET (alias legacy)
/api/v1/workspaces/{slug}/projects/{project_id}/issues GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/issues/{pk} GET, PATCH, DELETE
/api/v1/.../issues/{issue_id}/{links|comments} GET, POST
/api/v1/.../issues/{issue_id}/{links|comments}/{pk} PATCH, DELETE
/api/v1/.../issues/{issue_id}/activities[/{pk}] GET
/api/v1/.../issues/{issue_id}/issue-attachments GET, POST
/api/v1/.../issues/{issue_id}/issue-attachments/{pk} PATCH, DELETE
/api/v1/.../intake-issues GET, POST
/api/v1/.../intake-issues/{pk} GET, PATCH, DELETE
/api/v1/assets/user-assets POST
/api/v1/assets/user-assets/{asset_id} PATCH, DELETE
/api/v1/assets/user-assets/server POST
/api/v1/assets/user-assets/{asset_id}/server POST
/api/v1/workspaces/{slug}/assets POST
/api/v1/workspaces/{slug}/assets/{asset_id} GET, PATCH, DELETE

```

---
```
