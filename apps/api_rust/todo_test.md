# Inventario de Endpoints — `api_rust`

Documento de referencia para construir suites de tests E2E con **Playwright** sobre el frontend de Plane (`apps/web`, `apps/space`, `apps/admin`). Cada sección lista los endpoints expuestos por el backend Rust, los métodos HTTP soportados y notas operacionales necesarias para el setup/teardown y los flujos de prueba.

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

</details>

---

## 0. Convenciones generales

| Aspecto | Detalle |
|---|---|
| Base URL backend | `http://localhost:8000` (o `API_BASE_URL` en `.env`) |
| Prefijos | `/api/*` (frontend autenticado por sesión), `/api/v1/*` (público por API key), `/auth/*` (autenticación) |
| Trailing slash | El servidor aplica `NormalizePathLayer::trim_trailing_slash` — `/foo` y `/foo/` son equivalentes. **Playwright debe ser consistente** y preferir la forma canónica sin slash final. |
| Body limit | 1 MB para `/api/*` y `/auth/*`. Pruebas de upload usan endpoints `/assets/v2/*` con presigned URL S3 (no van al backend Rust). |
| Rate limit | Middleware `rate_limit_headers_middleware` activo en `auth_router` y `api_router`. Devuelve cabeceras `X-RateLimit-*`. Tests de auth deben tolerarlas y eventualmente esperarlas. |
| Autenticación `/api/*` | Sesión por cookie + CSRF token (`/auth/get-csrf-token`) |
| Autenticación `/api/v1/*` | Header `X-Api-Key` (token creado vía `/api-tokens`) |
| Auth admin | `/auth/instances/admins/sign-in` (god mode) — sesión separada |
| Documentación | Scalar UI en `/api/docs` cuando `DEBUG=true`. OpenAPI JSON: `/api/docs/openapi.json` |
| Identificadores | `slug` = workspace slug (string), todos los `pk`, `project_id`, `issue_id`, etc. son **UUID v4** |

> **Tests de integración**: los tests Playwright **llaman directamente a la API** (`request` fixture / `APIRequestContext`) para validar el contrato HTTP — status codes, headers, shape del JSON y side-effects en BD. Las llamadas directas a la API son la modalidad principal de prueba; los flujos UI complementan validando que el frontend consume correctamente esos contratos.

---

## 1. Autenticación (`/auth/*`)

Rutas con rate limiting. Todas devuelven cookies de sesión cuando exitosas.

### 1.1 Sesión / CSRF

| Método | Path | Notas |
|---|---|---|
| GET | `/auth/get-csrf-token` | Necesario antes de cualquier POST de auth. Devuelve `csrf_token` en cookie + body. |
| POST | `/auth/sign-in` | Body `{ email, password }`. Espacio: `/auth/spaces/sign-in` |
| POST | `/auth/sign-up` | Crea usuario. Espacio: `/auth/spaces/sign-up` |
| POST | `/auth/sign-out` | Cierra sesión. Espacio: `/auth/spaces/sign-out` |
| POST | `/auth/email-check` | Verifica si email existe. Espacio: `/auth/spaces/email-check` |

### 1.2 Magic link

| Método | Path |
|---|---|
| POST | `/auth/magic-generate` (+ `/auth/spaces/magic-generate`) |
| POST | `/auth/magic-sign-in` (+ `/auth/spaces/magic-sign-in`) |
| POST | `/auth/magic-sign-up` (+ `/auth/spaces/magic-sign-up`) |

### 1.3 Password

| Método | Path |
|---|---|
| POST | `/auth/change-password` |
| POST | `/auth/set-password` |
| POST | `/auth/forgot-password` (+ `/auth/spaces/forgot-password`) |
| POST | `/auth/reset-password/{uidb64}/{token}` (+ versión spaces) |

### 1.4 OAuth (initiate / callback)

| Método | Path | Notas |
|---|---|---|
| GET | `/auth/gitlab` | Initiate |
| GET | `/auth/gitlab/callback` | Callback |
| GET | `/auth/google` | Initiate |
| GET | `/auth/google/callback` | Callback |
| GET | `/auth/gitea` | Initiate |
| GET | `/auth/gitea/callback` | Callback |
| GET | `/auth/github/callback` | GitHub user OAuth (alias bajo /auth) |
| GET / POST | `/auth/github/user-callback` | GitHub user-callback |
| GET | `/github/callback` | GitHub App setup callback (sin auth middleware) |

### 1.5 God Mode (admin)

| Método | Path |
|---|---|
| POST | `/api/instances/admins/sign-up` |
| POST | `/api/instances/admins/sign-in` |
| POST | `/api/instances/admins/sign-out` |

> **Playwright tip**: usar `request.newContext({ storageState })` para preservar sesión entre tests; un `globalSetup` puede iniciar sesión una vez y reutilizar el storage state.

---

## 2. Health / Instance / Timezones

| Método | Path | Auth | Notas |
|---|---|---|---|
| GET | `/api/health` | No | Status + DB ping. Útil para `waitForHealth` en globalSetup. |
| GET | `/api/timezones` | Sí | Lista TZ. Datos estáticos. |
| GET / PATCH | `/api/instances` | Admin | Configuración de instancia. |
| POST | `/api/instances/admins/sign-up-screen-visited` | Admin | |
| GET / POST | `/api/instances/admins` | Admin | |
| GET | `/api/instances/admins/me` | Admin | |
| GET | `/api/instances/admins/session` | Admin | |
| DELETE | `/api/instances/admins/{pk}` | Admin | |
| GET / PATCH | `/api/instances/configurations` | Admin | |
| DELETE | `/api/instances/configurations/disable-email-feature` | Admin | |
| POST | `/api/instances/email-credentials-check` | Admin | |
| GET | `/api/instances/workspace-slug-check` | Admin | |
| GET | `/api/instances/workspaces` | Admin | |

---

## 3. Users (current user / `me`)

Todos requieren sesión activa.

| Método | Path | Notas |
|---|---|---|
| GET / PATCH / DELETE | `/api/users/me` | DELETE = desactivar cuenta |
| GET | `/api/users/session` | |
| GET | `/api/users/me/settings` | |
| GET | `/api/users/me/instance-admin` | |
| GET / PATCH | `/api/users/me/notification-preferences` | get_or_create — nunca 404 |
| PATCH | `/api/users/me/onboard` | |
| PATCH | `/api/users/me/tour-completed` | |
| GET / PATCH | `/api/users/me/profile` | |
| GET | `/api/users/me/accounts` | |
| GET / DELETE | `/api/users/me/accounts/{pk}` | |
| GET | `/api/users/last-visited-workspace` | |
| GET | `/api/users/me/workspaces` | |
| GET | `/api/users/me/activities` | Cursor-paginated cross-workspace |
| GET / POST | `/api/users/me/workspaces/invitations` | POST = bulk-accept |
| GET | `/api/users/me/workspaces/{slug}/project-roles` | |
| GET | `/api/users/me/workspaces/{slug}/activity-graph` | |
| GET | `/api/users/me/workspaces/{slug}/issues-completed-graph` | |
| GET | `/api/users/me/workspaces/{slug}/dashboard` | |
| POST | `/api/users/me/email/generate-code` | |
| POST | `/api/users/me/email` | Update email |
| GET | `/api/users/me/workspaces/{slug}/projects/invitations` | |

---

## 4. Workspaces

| Método | Path | Notas |
|---|---|---|
| GET | `/api/workspace-slug-check` | Validar slug único antes de crear |
| GET / POST | `/api/workspaces` | List / Create |
| GET / PATCH / DELETE | `/api/workspaces/{slug}` | |
| GET | `/api/workspaces/{slug}/members` | |
| POST | `/api/workspaces/{slug}/members/leave` | |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/members/{pk}` | |
| GET | `/api/workspaces/{slug}/project-members` | |
| GET / POST | `/api/workspaces/{slug}/workspace-views` | |
| GET | `/api/workspaces/{slug}/workspace-members/me` | |
| GET | `/api/workspaces/{slug}/user-profile/{user_id}` | |
| GET | `/api/workspaces/{slug}/user-stats/{user_id}` | |
| GET | `/api/workspaces/{slug}/user-activity/{user_id}` | |
| GET / POST | `/api/workspaces/{slug}/user-activity/{user_id}/export` | CSV download |
| GET | `/api/workspaces/{slug}/user-issues/{user_id}` | Pestañas Assigned/Created/Subscribed |
| GET / POST | `/api/workspaces/{slug}/invitations` | |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/invitations/{pk}` | |
| POST | `/api/workspaces/{slug}/invitations/{pk}/join` | |
| GET / POST | `/api/workspaces/{slug}/workspace-themes` | |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/workspace-themes/{pk}` | |

### 4.1 Workspace extras (favorites, home, sidebar, stickies, drafts)

| Método | Path |
|---|---|
| GET / POST | `/api/workspaces/{slug}/user-favorites` |
| PATCH / DELETE | `/api/workspaces/{slug}/user-favorites/{favorite_id}` |
| GET | `/api/workspaces/{slug}/user-favorites/{favorite_id}/children` |
| GET | `/api/workspaces/{slug}/user-favorites/{favorite_id}/group` (alias `/children`) |
| GET | `/api/workspaces/{slug}/home-preferences` |
| GET / PATCH | `/api/workspaces/{slug}/home-preferences/{key}` |
| GET / POST | `/api/workspaces/{slug}/quick-links` |
| PATCH / DELETE | `/api/workspaces/{slug}/quick-links/{pk}` |
| GET | `/api/workspaces/{slug}/recent-visits` |
| GET / POST | `/api/workspaces/{slug}/stickies` |
| PATCH / DELETE | `/api/workspaces/{slug}/stickies/{pk}` |
| GET / PATCH | `/api/workspaces/{slug}/sidebar-preferences` |
| GET / PATCH | `/api/workspaces/{slug}/user-properties` |
| GET / POST | `/api/workspaces/{slug}/draft-issues` |
| POST | `/api/workspaces/{slug}/draft-to-issue/{draft_id}` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/draft-issues/{pk}` |

### 4.2 Workspace aggregate (read-only, cross-project)

| Método | Path |
|---|---|
| GET | `/api/workspaces/{slug}/cycles` |
| GET | `/api/workspaces/{slug}/modules` |
| GET | `/api/workspaces/{slug}/estimates` |
| GET | `/api/workspaces/{slug}/labels` |
| GET | `/api/workspaces/{slug}/states` |
| GET | `/api/workspaces/{slug}/issues` |

---

## 5. Projects

| Método | Path | Notas |
|---|---|---|
| GET / POST | `/api/workspaces/{slug}/projects` | |
| GET | `/api/workspaces/{slug}/projects/details` | **Literal antes que `/{project_id}`** |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}` | |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/members` | |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/members/{pk}` | |
| GET | `/api/workspaces/{slug}/projects/{project_id}/project-members/me` | Permisos del usuario |
| POST | `/api/workspaces/{slug}/projects/{project_id}/members/leave` | |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/project-views` | User views |
| GET | `/api/workspaces/{slug}/projects/{project_id}/summary` | |
| POST / DELETE | `/api/workspaces/{slug}/projects/{project_id}/archive` | DELETE = unarchive |
| GET / POST | `/api/workspaces/{slug}/user-favorite-projects` | |
| DELETE | `/api/workspaces/{slug}/user-favorite-projects/{project_id}` | |
| GET / DELETE | `/api/workspaces/{slug}/project-identifiers` | |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards` | POST = upsert |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}` | |
| GET / PATCH | `/api/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}` | |
| GET / PATCH | `/api/workspaces/{slug}/projects/{project_id}/user-properties` | get_or_create |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/invitations` | |
| GET / DELETE | `/api/workspaces/{slug}/projects/{project_id}/invitations/{pk}` | |
| POST | `/api/workspaces/{slug}/projects/{project_id}/join/{pk}` | Public join |

---

## 6. States, Labels, Estimates

### 6.1 States

| Método | Path |
|---|---|
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/states` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/states/{pk}` |
| GET | `/api/workspaces/{slug}/projects/{project_id}/intake-state` |
| POST | `/api/workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default` |

### 6.2 Labels

| Método | Path | Notas |
|---|---|---|
| POST | `/api/workspaces/{slug}/projects/{project_id}/bulk-create-labels` | |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/labels` | Canónica |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/labels/{pk}` | Canónica |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/issue-labels` | **Alias Django-compat** |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/issue-labels/{pk}` | Alias |

### 6.3 Estimates

| Método | Path |
|---|---|
| GET | `/api/workspaces/{slug}/projects/{project_id}/project-estimates` |
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/estimates` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}` |
| POST | `/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points` |
| PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}` |

---

## 7. Issues / Work Items (núcleo)

> **Paridad de paths**: el frontend usa indistintamente `issues/` (legacy) y `work-items/` (nuevo). Ambos shapes están registrados sobre los mismos handlers. **Para tests de regresión, recomendable cubrir ambos.**

### 7.1 CRUD básico

| Método | Path |
|---|---|
| GET / POST | `/api/workspaces/{slug}/projects/{project_id}/issues` |
| GET | `/api/workspaces/{slug}/projects/{project_id}/issues/list` |
| GET | `/api/workspaces/{slug}/projects/{project_id}/issues-detail` |
| GET | `/api/workspaces/{slug}/projects/{project_id}/v2/issues` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/projects/{project_id}/issues/{pk}` |
| GET | `/api/workspaces/{slug}/work-items/{combined}` (lookup por `PROJ-1234`) |
| GET | `/api/workspaces/{slug}/issues/{combined}` (alias) |

### 7.2 Comments / Reactions / Links / Relations / Subscribers

| Método | Path |
|---|---|
| GET / POST | `…/issues/{issue_id}/comments` |
| GET / PATCH / DELETE | `…/issues/{issue_id}/comments/{pk}` |
| GET / POST | `…/issues/{issue_id}/reactions` |
| DELETE | `…/issues/{issue_id}/reactions/{reaction_code}` |
| GET / POST | `…/comments/{comment_id}/reactions` |
| DELETE | `…/comments/{comment_id}/reactions/{reaction_code}` |
| GET / POST | `…/issues/{issue_id}/issue-links` (canónica) |
| PATCH / DELETE | `…/issues/{issue_id}/issue-links/{pk}` |
| GET / POST | `…/issues/{issue_id}/links` (alias corto) |
| PATCH / DELETE | `…/issues/{issue_id}/links/{pk}` |
| GET / POST | `…/work-items/{issue_id}/links` (alias work-items) |
| PATCH / DELETE | `…/work-items/{issue_id}/links/{pk}` |
| GET / POST | `…/issues/{issue_id}/issue-relation` |
| POST | `…/issues/{issue_id}/remove-relation` (**POST**, no DELETE) |
| GET | `…/issues/{issue_id}/issue-subscribers` |
| DELETE | `…/issues/{issue_id}/issue-subscribers/{subscriber_id}` |
| POST / DELETE | `…/issues/{issue_id}/subscribe` |
| GET / POST | `…/issues/{issue_id}/sub-issues` |

### 7.3 Activity / History / Versions

| Método | Path |
|---|---|
| GET | `…/issues/{issue_id}/activities` (alias `/history`) |
| GET | `…/issues/{issue_id}/history` |
| GET | `…/issues/{issue_id}/activities/{pk}` |
| GET | `…/work-items/{issue_id}/activities` |
| GET | `…/work-items/{issue_id}/activities/{pk}` |
| GET | `…/issues/{issue_id}/versions` |
| GET | `…/issues/{issue_id}/versions/{pk}` |
| GET | `…/work-items/{work_item_id}/description-versions` |
| GET | `…/work-items/{work_item_id}/description-versions/{pk}` |
| GET | `…/intake-work-items/{work_item_id}/description-versions` |
| GET | `…/intake-work-items/{work_item_id}/description-versions/{pk}` |

### 7.4 Attachments (V2)

| Método | Path |
|---|---|
| GET / POST | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments` |
| PATCH / DELETE | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}` |
| GET / POST | `…/issues/{issue_id}/issue-attachments` (legacy) |
| PATCH / DELETE | `…/issues/{issue_id}/issue-attachments/{pk}` |

### 7.5 Archive / Bulk

| Método | Path | Notas |
|---|---|---|
| GET / POST / DELETE | `…/issues/{pk}/archive` | |
| GET | `…/archived-issues` | |
| GET | `…/deleted-issues` | |
| DELETE / POST | `…/bulk-delete-issues` | Acepta ambos métodos (DELETE-with-body inestable en CDNs) |
| GET | `…/issues/{issue_id}/meta` | |
| POST | `…/bulk-archive-issues` | |
| POST | `…/issue-dates` | Bulk update fechas |

---

## 8. Cycles

| Método | Path |
|---|---|
| GET / POST | `…/projects/{project_id}/cycles` |
| GET / PATCH / DELETE | `…/cycles/{pk}` |
| GET / POST | `…/cycles/{cycle_id}/cycle-issues` |
| DELETE | `…/cycles/{cycle_id}/cycle-issues/{issue_id}` |
| GET | `…/cycles/{cycle_id}/analytics` |
| GET / PATCH | `…/cycles/{cycle_id}/user-properties` |
| GET | `…/cycles/{cycle_id}/progress` |
| POST | `…/cycles/date-check` |
| GET / POST | `…/user-favorite-cycles` |
| DELETE | `…/user-favorite-cycles/{cycle_id}` |
| POST | `…/cycles/{cycle_id}/transfer-issues` |
| POST / DELETE | `…/cycles/{cycle_id}/archive` |
| GET | `…/archived-cycles` |
| GET / DELETE | `…/archived-cycles/{pk}` |

---

## 9. Modules

| Método | Path |
|---|---|
| GET / POST | `…/projects/{project_id}/modules` |
| GET / PATCH / DELETE | `…/modules/{pk}` |
| GET / POST | `…/modules/{module_id}/issues` |
| DELETE | `…/modules/{module_id}/issues/{issue_id}` |
| GET / PATCH | `…/modules/{module_id}/user-properties` |
| POST | `…/issues/{issue_id}/modules` |
| GET / POST | `…/modules/{module_id}/module-links` |
| GET / PATCH / DELETE | `…/modules/{module_id}/module-links/{pk}` |
| GET / POST | `…/user-favorite-modules` |
| DELETE | `…/user-favorite-modules/{module_id}` |
| POST / DELETE | `…/modules/{module_id}/archive` |
| GET | `…/archived-modules` |
| GET / DELETE | `…/archived-modules/{pk}` |

---

## 10. Pages

| Método | Path |
|---|---|
| GET | `…/projects/{project_id}/pages-summary` |
| GET / POST | `…/projects/{project_id}/pages` |
| GET / PATCH / DELETE | `…/pages/{page_id}` |
| POST | `…/pages/{page_id}/access` |
| POST / DELETE | `…/favorite-pages/{page_id}` |
| POST / DELETE | `…/pages/{page_id}/archive` |
| POST / DELETE | `…/pages/{page_id}/lock` |
| POST | `…/pages/{page_id}/duplicate` |
| GET | `…/pages/{page_id}/versions` |
| GET | `…/pages/{page_id}/versions/{pk}` |
| GET / PATCH | `…/pages/{page_id}/description` |

---

## 11. Intake (Inbox)

> **Aliases**: `/intakes/` ↔ `/inboxes/` y `/intake-issues/` ↔ `/inbox-issues/`. Mismo handler. El frontend actual usa `inbox-issues`.

| Método | Path |
|---|---|
| GET / POST | `…/intakes` (+ `/inboxes`) |
| GET / PATCH / DELETE | `…/intakes/{pk}` (+ `/inboxes/{pk}`) |
| GET / POST | `…/intake-issues` (+ `/inbox-issues`) |
| GET / PATCH / DELETE | `…/intake-issues/{pk}` (+ `/inbox-issues/{pk}`) |

---

## 12. Views

| Método | Path |
|---|---|
| GET / POST | `/api/workspaces/{slug}/views` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/views/{pk}` |
| GET / POST | `…/projects/{project_id}/views` |
| GET / PATCH / DELETE | `…/projects/{project_id}/views/{pk}` |
| GET / POST | `…/projects/{project_id}/user-favorite-views` |
| DELETE | `…/projects/{project_id}/user-favorite-views/{view_id}` |

---

## 13. Notifications

| Método | Path |
|---|---|
| GET | `…/users/notifications/unread` |
| POST | `…/users/notifications/mark-all-read` |
| GET | `…/users/notifications` |
| GET / PATCH / DELETE | `…/users/notifications/{pk}` |
| POST / DELETE | `…/users/notifications/{pk}/read` |
| POST / DELETE | `…/users/notifications/{pk}/archive` |

---

## 14. Webhooks

| Método | Path |
|---|---|
| GET / POST | `/api/workspaces/{slug}/webhooks` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/webhooks/{pk}` |
| POST | `/api/workspaces/{slug}/webhooks/{pk}/regenerate` |
| GET | `/api/workspaces/{slug}/webhook-logs/{webhook_id}` |
| POST | `/api/github-webhook` (público, externo) |
| POST | `/api/gitlab-webhook` (público, externo) |

---

## 15. Search / Exporter / Importer

### 15.1 Search

| Método | Path |
|---|---|
| GET | `/api/workspaces/{slug}/search` |
| GET | `/api/workspaces/{slug}/work-items/search` |
| GET | `/api/workspaces/{slug}/issues/search` (alias legacy) |
| GET | `…/projects/{project_id}/search-issues` |
| GET | `/api/workspaces/{slug}/entity-search` |

### 15.2 Exporter

| Método | Path |
|---|---|
| GET / POST | `/api/workspaces/{slug}/export-issues` |
| GET | `/api/workspaces/{slug}/export-issues/{token}` |

### 15.3 Importer (GitHub / GitLab)

| Método | Path |
|---|---|
| GET | `…/importers/github/repositories` |
| GET / POST | `…/importers/github` |
| DELETE | `…/importers/github/{importer_id}` |
| GET | `…/importers/gitlab/repositories` |
| GET / POST | `…/importers/gitlab` |
| DELETE | `…/importers/gitlab/{importer_id}` |

---

## 16. Analytics

### 16.1 Workspace analytics (legacy)

| Método | Path |
|---|---|
| GET | `/api/workspaces/{slug}/analytics` |
| GET | `/api/workspaces/{slug}/default-analytics` |
| GET | `/api/workspaces/{slug}/project-stats` |
| POST | `/api/workspaces/{slug}/export-analytics` |
| GET / POST | `/api/workspaces/{slug}/analytic-view` |
| GET / PATCH / DELETE | `/api/workspaces/{slug}/analytic-view/{pk}` |
| GET | `/api/workspaces/{slug}/saved-analytic-view/{analytic_id}` |

### 16.2 Advance analytics

| Método | Path |
|---|---|
| GET | `/api/workspaces/{slug}/advance-analytics` |
| GET | `/api/workspaces/{slug}/advance-analytics-stats` |
| GET | `/api/workspaces/{slug}/advance-analytics-charts` |
| GET | `…/projects/{project_id}/advance-analytics` |
| GET | `…/projects/{project_id}/advance-analytics-stats` |
| GET | `…/projects/{project_id}/advance-analytics-charts` |

---

## 17. Assets (V2)

> Flujo de upload: `POST initiate` → backend devuelve presigned URL S3 → cliente sube directamente a S3 → `PATCH complete`. Tests Playwright deben **stubbear S3** (route handler) o usar MinIO local.

| Método | Path |
|---|---|
| POST | `/api/assets/v2/user-assets` |
| PATCH / DELETE | `/api/assets/v2/user-assets/{asset_id}` |
| POST | `/api/assets/v2/workspaces/{slug}` |
| GET / PATCH / DELETE | `/api/assets/v2/workspaces/{slug}/{asset_id}` |
| GET | `/api/assets/v2/static/{asset_id}` |
| POST | `/api/assets/v2/workspaces/{slug}/restore/{asset_id}` |
| GET | `/api/assets/v2/workspaces/{slug}/check/{asset_id}` |
| POST | `/api/assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}` |
| GET | `/api/assets/v2/workspaces/{slug}/download/{asset_id}` |
| POST | `/api/assets/v2/workspaces/{slug}/projects/{project_id}` |
| GET / PATCH / DELETE | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}` |
| POST | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk` |
| GET | `/api/assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}` |

---

## 18. External (AI / Unsplash)

| Método | Path |
|---|---|
| GET | `/api/unsplash` |
| POST | `/api/workspaces/{slug}/projects/{project_id}/ai-assistant` |
| POST | `/api/workspaces/{slug}/ai-assistant` |
| POST | `/api/workspaces/{slug}/rephrase-grammar` |

---

## 19. Integrations (GitHub / GitLab)

| Método | Path |
|---|---|
| GET | `/api/integrations` |
| GET / POST | `…/workspace-integrations` |
| GET / PATCH / DELETE | `…/workspace-integrations/{pk}` |
| DELETE | `…/workspace-integrations/{provider}/provider` |
| POST | `…/workspace-integrations/{provider}/install` |
| GET | `…/workspace-integrations/{wi_id}/github-repositories` |
| GET | `…/workspace-integrations/{wi_id}/gitlab-repositories` |
| GET / POST | `…/workspace-integrations/github/repo-syncs` |
| DELETE | `…/workspace-integrations/github/repo-syncs/{pk}` |
| GET / POST | `…/workspace-integrations/{wi_id}/pr-state-mappings` |
| DELETE | `…/workspace-integrations/{wi_id}/pr-state-mappings/{pk}` |

---

## 20. API Tokens (sesión-based)

| Método | Path |
|---|---|
| GET / POST | `/api/api-tokens` |
| GET / PATCH / DELETE | `/api/api-tokens/{pk}` |
| GET / POST | `/api/users/api-tokens` (alias) |
| GET / PATCH / DELETE | `/api/users/api-tokens/{pk}` (alias) |

---

## 21. API v1 pública (`/api/v1/*`)

> Auth: header `X-Api-Key`. Espejo de `plane.api.urls`. Útil para tests de **integraciones externas** (no UI). Mismos métodos que las contrapartes en `/api/*`.

```
/api/v1/users/me                                                                 GET, PATCH
/api/v1/workspaces/{slug}/members                                                GET
/api/v1/workspaces/{slug}/projects                                               GET, POST
/api/v1/workspaces/{slug}/projects/{pk}                                          GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/archive                          POST, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/summary                          GET
/api/v1/workspaces/{slug}/projects/{project_id}/members                          GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/members/{pk}                     GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/project-members                  GET
/api/v1/workspaces/{slug}/projects/{project_id}/project-members/{pk}             GET
/api/v1/workspaces/{slug}/projects/{project_id}/states                           GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/states/{state_id}                GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/labels                           GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/labels/{pk}                      GET, PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/estimates                        GET
/api/v1/.../estimates/{estimate_id}/estimate-points                              POST
/api/v1/.../estimates/{estimate_id}/estimate-points/{pk}                         PATCH, DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/cycles                           GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/cycles/{pk}                      GET, PATCH, DELETE
/api/v1/.../cycles/{cycle_id}/cycle-issues                                       GET, POST
/api/v1/.../cycles/{cycle_id}/cycle-issues/{issue_id}                            DELETE
/api/v1/.../cycles/{cycle_id}/transfer-issues                                    POST
/api/v1/.../cycles/{cycle_id}/archive                                            POST
/api/v1/.../archived-cycles                                                      GET
/api/v1/.../archived-cycles/{pk}                                                 GET
/api/v1/.../archived-cycles/{pk}/unarchive                                       DELETE
/api/v1/workspaces/{slug}/projects/{project_id}/modules                          GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/modules/{pk}                     GET, PATCH, DELETE
/api/v1/.../modules/{module_id}/module-issues                                    GET, POST
/api/v1/.../modules/{module_id}/module-issues/{issue_id}                         DELETE
/api/v1/.../modules/{pk}/archive                                                 POST
/api/v1/.../archived-modules                                                     GET
/api/v1/.../archived-modules/{pk}                                                GET
/api/v1/.../archived-modules/{pk}/unarchive                                      DELETE
/api/v1/workspaces/{slug}/work-items/search                                      GET
/api/v1/workspaces/{slug}/work-items/{combined}                                  GET
/api/v1/workspaces/{slug}/projects/{project_id}/work-items                       GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/work-items/{pk}                  GET, PATCH, DELETE
/api/v1/.../work-items/{issue_id}/links                                          GET, POST
/api/v1/.../work-items/{issue_id}/links/{pk}                                     PATCH, DELETE
/api/v1/.../work-items/{issue_id}/comments                                       GET, POST
/api/v1/.../work-items/{issue_id}/comments/{pk}                                  GET, PATCH, DELETE
/api/v1/.../work-items/{issue_id}/activities                                     GET
/api/v1/.../work-items/{issue_id}/activities/{pk}                                GET
/api/v1/.../work-items/{issue_id}/attachments                                    GET, POST
/api/v1/.../work-items/{issue_id}/attachments/{pk}                               PATCH, DELETE
/api/v1/.../work-items/{issue_id}/relations                                      GET, POST
/api/v1/workspaces/{slug}/issues/search                                          GET   (alias legacy)
/api/v1/workspaces/{slug}/issues/{combined}                                      GET   (alias legacy)
/api/v1/workspaces/{slug}/projects/{project_id}/issues                           GET, POST
/api/v1/workspaces/{slug}/projects/{project_id}/issues/{pk}                      GET, PATCH, DELETE
/api/v1/.../issues/{issue_id}/{links|comments}                                   GET, POST
/api/v1/.../issues/{issue_id}/{links|comments}/{pk}                              PATCH, DELETE
/api/v1/.../issues/{issue_id}/activities[/{pk}]                                  GET
/api/v1/.../issues/{issue_id}/issue-attachments                                  GET, POST
/api/v1/.../issues/{issue_id}/issue-attachments/{pk}                             PATCH, DELETE
/api/v1/.../intake-issues                                                        GET, POST
/api/v1/.../intake-issues/{pk}                                                   GET, PATCH, DELETE
/api/v1/assets/user-assets                                                       POST
/api/v1/assets/user-assets/{asset_id}                                            PATCH, DELETE
/api/v1/assets/user-assets/server                                                POST
/api/v1/assets/user-assets/{asset_id}/server                                     POST
/api/v1/workspaces/{slug}/assets                                                 POST
/api/v1/workspaces/{slug}/assets/{asset_id}                                      GET, PATCH, DELETE
```

---

## 22. Estrategia de integration testing con Playwright

Los tests Playwright invocan la API **directamente** vía `APIRequestContext` (`request` fixture). Cada test valida el contrato extremo a extremo: request → respuesta HTTP → estado persistido → respuesta de un GET subsiguiente.

### 22.1 Patrón base (request fixture)

```ts
import { test, expect } from '@playwright/test';

test('crear y obtener issue', async ({ request }) => {
  // CREATE
  const create = await request.post(
    `/api/workspaces/${slug}/projects/${projectId}/issues`,
    { data: { name: 'Test issue', state_id: stateId } }
  );
  expect(create.status()).toBe(201);
  const issue = await create.json();
  expect(issue).toMatchObject({ name: 'Test issue', id: expect.any(String) });

  // READ-back (verificar persistencia)
  const get = await request.get(
    `/api/workspaces/${slug}/projects/${projectId}/issues/${issue.id}`
  );
  expect(get.status()).toBe(200);
  expect(await get.json()).toMatchObject({ id: issue.id, name: 'Test issue' });
});
```

### 22.2 Aspectos a validar en cada endpoint

| Aspecto | Cómo validar |
|---|---|
| Status code | `expect(res.status()).toBe(200/201/204/400/401/403/404)` |
| Shape del response | `expect(body).toMatchObject({...})` o JSON Schema |
| Headers críticos | `Content-Type`, `X-RateLimit-*`, `Set-Cookie` (CSRF/session) |
| Idempotencia | Repetir PATCH/DELETE — el segundo call debe dar el resultado esperado (no 500) |
| Side-effect en BD | GET subsiguiente al endpoint mutador |
| Permisos | Repetir el call con sesión sin acceso → esperar 403/404 |
| Validación de input | Enviar payloads inválidos → esperar 400 con mensaje específico |
| Cascada | `DELETE /workspaces/{slug}` → confirmar que `GET /projects` devuelve 404 |

### 22.3 globalSetup

1. `GET /api/health` — esperar `200 ok` antes de iniciar la suite.
2. `GET /auth/get-csrf-token` → guardar cookie `csrftoken` + token en body.
3. `POST /auth/sign-up` (o `sign-in` si el user ya existe) con header `X-CSRFToken`.
4. Persistir storage con `request.storageState({ path: 'state.json' })`.
5. `POST /api/workspaces` con slug único `e2e-${Date.now()}`.
6. `POST /api/workspaces/{slug}/projects` con `identifier` corto y aleatorio.
7. Verificar estados por defecto con `GET /states` (Backlog, Todo, In Progress, Done) — el backend los crea automáticamente al crear proyecto en la mayoría de los flujos; si no, sembrar manualmente.
8. Persistir `slug`, `project_id`, `state_ids` en `process.env` o en `playwright.config.ts > use.extraHTTPHeaders`.

### 22.4 globalTeardown

1. `DELETE /api/workspaces/{slug}` (cascade) — elimina projects, issues, cycles, modules, etc.
2. `DELETE /api/users/me` solo si el usuario fue creado durante el run.

### 22.5 Fixtures por test (creación + cleanup automático)

| Fixture | Setup | Teardown |
|---|---|---|
| `freshIssue` | `POST .../issues` | `DELETE .../issues/{pk}` |
| `freshCycle` | `POST .../cycles` | `DELETE .../cycles/{pk}` |
| `freshLabel` | `POST .../labels` | `DELETE .../labels/{pk}` |
| `freshPage` | `POST .../pages` | `DELETE .../pages/{pk}` |
| `freshModule` | `POST .../modules` | `DELETE .../modules/{pk}` |
| `freshView` | `POST .../views` | `DELETE .../views/{pk}` |
| `freshWebhook` | `POST .../webhooks` | `DELETE .../webhooks/{pk}` |
| `freshApiToken` | `POST /api/api-tokens` | `DELETE /api/api-tokens/{pk}` |

### 22.6 Pre-checks antes de cada suite

- `GET /api/users/me` → confirmar sesión activa (401 → re-login).
- `GET /api/workspaces/{slug}/workspace-members/me` → confirmar rol y permisos de workspace.
- `GET /api/workspaces/{slug}/projects/{project_id}/project-members/me` → permisos de proyecto.

### 22.7 Suites recomendadas (organización)

| Suite | Cobertura |
|---|---|
| `auth.spec.ts` | sign-in / sign-up / magic-link / sign-out / CSRF / rate-limit headers |
| `workspaces.spec.ts` | CRUD workspace, miembros, invitaciones, themes |
| `projects.spec.ts` | CRUD project, miembros, invitaciones, archive/unarchive, identifiers |
| `issues.spec.ts` | CRUD issue, paths legacy `issues/` y nuevos `work-items/` (cubrir ambos), bulk ops |
| `issues-extras.spec.ts` | comments, reactions, links, relations, subscribers, sub-issues |
| `cycles.spec.ts` | CRUD, cycle-issues, transfer, archive, analytics |
| `modules.spec.ts` | CRUD, module-issues, module-links, archive |
| `pages.spec.ts` | CRUD, archive, lock, duplicate, versions |
| `intake.spec.ts` | aliases `intakes`/`inboxes`, `intake-issues`/`inbox-issues` |
| `analytics.spec.ts` | workspace, project-stats, advance-analytics |
| `assets.spec.ts` | flujo presigned URL S3 (con MinIO local) |
| `integrations.spec.ts` | github / gitlab / pr-state-mappings |
| `api-v1.spec.ts` | endpoints `/api/v1/*` con header `X-Api-Key` |
| `permissions.spec.ts` | matrix de roles (admin/member/viewer/guest) × endpoints |
| `contracts.spec.ts` | validación contra OpenAPI spec (`/api/docs/openapi.json`) |

### 22.8 Validación de contratos contra OpenAPI

Usar `/api/docs/openapi.json` como fuente de verdad. Librería sugerida: `openapi-response-validator` o `ajv` con el schema importado:

```ts
import Ajv from 'ajv';
import openapi from './openapi.json';

const ajv = new Ajv();
const validate = ajv.compile(openapi.components.schemas.WorkspaceResponse);

const res = await request.get(`/api/workspaces/${slug}`);
const body = await res.json();
expect(validate(body), JSON.stringify(validate.errors)).toBe(true);
```

### 22.9 Concurrencia

`playwright.config.ts > workers` debe limitarse a `1-2` mientras los tests compartan workspace, o usar **un workspace por worker** (`process.env.TEST_WORKER_INDEX`) para evitar contención en endpoints como `/states`, `/labels`, `/issues`.

---

## 23. Notas de comportamiento críticas para tests

1. **Trailing slash**: el `NormalizePathLayer` se aplica antes del routing → ambas formas funcionan.
2. **`bulk-delete-issues`** acepta DELETE **y** POST (workaround para CDNs que descartan body en DELETE).
3. **`remove-relation`** es **POST**, no DELETE (paridad Django).
4. **`pages/{page_id}/archive`**: POST = archivar, DELETE = desarchivar.
5. **`projects/{project_id}/archive`**: POST = archivar, DELETE = desarchivar.
6. **`subscribe`** sobre issue: POST suscribe, DELETE desuscribe.
7. **`cycles/{cycle_id}/archive`**: POST archiva, DELETE desarchiva.
8. **Modules/cycles/issues**: el endpoint `/archived-*/{pk}` maneja unarchive vía DELETE.
9. **Workspace integrations**: las rutas literales `github/repo-syncs` van **antes** de `/{pk}` para evitar que `"github"` sea capturado como UUID.
10. **`projects/details`** literal va antes de `/{project_id}` por la misma razón.
11. **Dos sistemas de auth paralelos**: usuarios normales (sesión + cookie) y admins god-mode (sesión separada en `/api/instances/admins/*`).
12. **CSRF**: requerido en todos los POST/PATCH/DELETE de `/api/*` con auth de sesión. Playwright debe extraer el cookie `csrftoken` y reenviarlo en header `X-CSRFToken`.
13. **API v1** (`/api/v1/*`) **no requiere CSRF** (auth por header).
14. **Aliases**: muchos paths tienen variantes (`issues/` ↔ `work-items/`, `intakes/` ↔ `inboxes/`, `intake-issues/` ↔ `inbox-issues/`, `links` ↔ `issue-links`, `activities` ↔ `history`). Para regresión es válido cubrir el alias que usa el frontend actual y delegar el resto a tests de contrato.

---

## 24. Pendientes / observaciones para los tests

- [ ] Verificar lista completa de roles en respuestas de `/members` y mapear a constantes en Playwright (member: 5/10/15/20).
- [ ] Documentar payload exacto de creación de issue (campos requeridos vs. opcionales) — extraer de `src/routes/issues.rs`.
- [ ] Verificar formato del cursor de paginación (`?cursor=` vs. `?per_page=&page=`).
- [ ] Identificar endpoints que requieren feature flags habilitados en `/api/instances/configurations`.
- [ ] Cubrir flujo de assets V2 con MinIO local (presigned URL S3) vs. mock con `page.route()`.
- [ ] Validar comportamiento de `entity-search` con distintos `entity_name` (issue, page, cycle, module, view, project).
- [ ] Determinar si el frontend `apps/admin` consume endpoints adicionales fuera de `/api/instances/*`.
- [ ] Confirmar payloads de webhook (GitHub/GitLab) que son disparados desde el integration tests.
