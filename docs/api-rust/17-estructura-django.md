---
titulo: Estructura de carpetas Django (fuente de migración)
tags:
  - django
  - estructura
  - referencia
  - migracion
relacionado:
  - "[[13-estructura-archivos]]"
  - "[[07-fases]]"
  - "[[02-orm-y-migraciones]]"
estado: activo
---

# Estructura de carpetas Django — `apps/api`

> [!INFO] Propósito
> Referencia completa del proyecto Django que se está migrando a Rust.
> Cada módulo Django tiene su equivalente mapeado en la columna "Equivalente Rust".
> El árbol refleja el estado actual de la rama `feature/integrations-panel-fix-17593507967815292912`.

---

## Árbol completo

```
apps/api/
├── manage.py                          ← entry point Django (equiv: src/main.rs)
├── run_tests.py
└── plane/
    ├── asgi.py                        ← ASGI server config
    ├── wsgi.py                        ← WSGI server config
    ├── celery.py                      ← configuración Celery broker (equiv: src/jobs/)
    ├── urls.py                        ← root URL dispatcher (equiv: src/main.rs router)
    │
    ├── settings/                      ← configuración por entorno
    │   ├── common.py                  ← settings base (equiv: src/config.rs)
    │   ├── local.py
    │   ├── production.py
    │   ├── test.py
    │   ├── redis.py                   ← Redis/cache config
    │   ├── storage.py                 ← S3/MinIO config
    │   ├── mongo.py                   ← MongoDB (analytics)
    │   └── openapi.py                 ← Swagger/OpenAPI config
    │
    ├── middleware/                    ← middlewares globales
    │   ├── db_routing.py             ← multi-DB router
    │   ├── logger.py
    │   └── request_body_size.py
    │
    ├── db/                            ← capa de datos principal (equiv: src/entities/ + src/repositories/)
    │   ├── mixins.py                  ← SoftDeleteMixin, TimestampMixin (equiv: src/utils/soft_delete.rs)
    │   ├── models/                    ← 33 archivos de modelos ORM
    │   │   ├── base.py               ← BaseModel con UUID, timestamps, soft-delete
    │   │   ├── user.py               ← User, Profile, Account, Device
    │   │   ├── workspace.py          ← Workspace, WorkspaceMember, WorkspaceTheme, etc.
    │   │   ├── project.py            ← Project, ProjectMember, ProjectIdentifier, etc.
    │   │   ├── issue.py              ← Issue, IssueActivity, IssueRelation, etc.
    │   │   ├── state.py              ← State
    │   │   ├── label.py              ← Label
    │   │   ├── cycle.py              ← Cycle, CycleIssue, CycleFavorite, etc.
    │   │   ├── module.py             ← Module, ModuleIssue, ModuleMember, etc.
    │   │   ├── page.py               ← Page, PageLog, PageLabel, etc.
    │   │   ├── description.py        ← Description, DescriptionVersion
    │   │   ├── view.py               ← IssueView, IssueViewFavorite
    │   │   ├── notification.py       ← Notification, UserNotificationPreference
    │   │   ├── webhook.py            ← Webhook, WebhookLog
    │   │   ├── intake.py             ← Intake, IntakeIssue
    │   │   ├── estimate.py           ← Estimate, EstimatePoint
    │   │   ├── analytic.py           ← AnalyticView
    │   │   ├── asset.py              ← FileAsset
    │   │   ├── api.py                ← APIToken
    │   │   ├── favorite.py           ← UserFavorite
    │   │   ├── draft.py              ← DraftIssue
    │   │   ├── exporter.py           ← ExporterHistory
    │   │   ├── importer.py           ← Importer
    │   │   ├── deploy_board.py       ← DeployBoard
    │   │   ├── device.py             ← Device
    │   │   ├── recent_visit.py       ← RecentVisit
    │   │   ├── session.py            ← Session
    │   │   ├── social_connection.py  ← SocialConnection
    │   │   ├── sticky.py             ← Sticky
    │   │   ├── issue_type.py         ← IssueType
    │   │   └── integration/          ← modelos de integraciones (equiv: migration m006)
    │   │       ├── base.py           ← WorkspaceIntegration, ProjectIntegration
    │   │       ├── github.py         ← GithubRepository, GithubIssue, GithubCommentSync
    │   │       ├── github_pr_state.py← GithubPRStateMapping
    │   │       ├── gitlab.py         ← GitlabIssueSync, GitlabMilestoneSync
    │   │       ├── slack.py          ← SlackProjectSync
    │   │       ├── user_github_connection.py ← UserGithubConnection
    │   │       └── signals.py        ← Django signals para integraciones
    │   │
    │   ├── migrations/               ← 126 migraciones históricas (equiv: migration/src/migrations/)
    │   │   ├── 0001_initial.py       ← schema inicial 2022
    │   │   ├── ...
    │   │   ├── 0122_add_github_gitlab_integrations.py
    │   │   ├── 0123_add_slack_integration.py
    │   │   ├── 0124_githubprstatemapping_usergithubconnection.py
    │   │   ├── 0125_githubprstatemapping_extend_states.py
    │   │   └── 0126_gitlab_sync_models.py   ← última migración
    │   │
    │   └── management/commands/      ← comandos Django CLI (equiv: apalis jobs + cron)
    │       ├── wait_for_db.py
    │       ├── wait_for_migrations.py
    │       ├── create_instance_admin.py
    │       ├── configure_instance.py ← (equivale a workspace_seed en Rust)
    │       ├── create_dummy_data.py
    │       ├── sync_issue_version.py
    │       ├── fix_duplicate_sequences.py
    │       └── ...
    │
    ├── bgtasks/                       ← 36 tareas Celery (equiv: src/jobs/)
    │   ├── issue_activities_task.py   ← actividad de issues
    │   ├── notification_task.py       ← notificaciones push/email
    │   ├── email_notification_task.py
    │   ├── github_sync_task.py        ← sync GitHub (equiv: jobs/github_sync.rs)
    │   ├── export_task.py             ← exportación CSV/Excel
    │   ├── import_task.py
    │   ├── webhook_task.py            ← despacho de webhooks
    │   ├── workspace_seed_task.py     ← seed de workspace (equiv: jobs/workspace_seed/)
    │   ├── page_version_task.py
    │   ├── issue_automation_task.py
    │   ├── magic_link_code_task.py
    │   ├── forgot_password_task.py
    │   ├── cleanup_task.py            ← limpieza de assets expirados
    │   ├── deletion_task.py
    │   ├── copy_s3_object.py
    │   ├── storage_metadata_task.py
    │   └── ...
    │
    ├── app/                           ← API principal autenticada (equiv: src/routes/)
    │   ├── permissions/               ← guards de acceso (equiv: src/auth/permissions.rs)
    │   │   ├── base.py
    │   │   ├── workspace.py           ← WorkspaceMemberPermission
    │   │   ├── project.py             ← ProjectMemberPermission
    │   │   └── page.py
    │   ├── middleware/
    │   │   └── api_authentication.py  ← JWT/Session auth (equiv: src/auth/middleware.rs)
    │   ├── serializers/               ← 24 serializadores DRF (equiv: DTOs en Rust)
    │   │   ├── base.py
    │   │   ├── workspace.py
    │   │   ├── project.py
    │   │   ├── issue.py
    │   │   ├── cycle.py
    │   │   ├── module.py
    │   │   ├── page.py
    │   │   ├── integration.py         ← integraciones
    │   │   └── ...
    │   ├── urls/                      ← 24 archivos de rutas
    │   │   ├── workspace.py
    │   │   ├── project.py
    │   │   ├── issue.py
    │   │   ├── cycle.py
    │   │   ├── module.py
    │   │   ├── page.py
    │   │   ├── integration.py
    │   │   ├── notification.py
    │   │   ├── webhook.py
    │   │   ├── analytic.py
    │   │   └── ...
    │   └── views/                     ← handlers DRF (equiv: src/routes/)
    │       ├── base.py                ← BaseAPIView
    │       ├── workspace/
    │       │   ├── base.py
    │       │   ├── member.py
    │       │   ├── invite.py
    │       │   ├── label.py
    │       │   ├── cycle.py
    │       │   ├── module.py
    │       │   ├── draft.py
    │       │   ├── favorite.py
    │       │   ├── home.py
    │       │   ├── quick_link.py
    │       │   ├── recent_visit.py
    │       │   ├── state.py
    │       │   ├── sticky.py
    │       │   ├── user.py
    │       │   └── user_preference.py
    │       ├── issue/
    │       │   ├── base.py
    │       │   ├── activity.py
    │       │   ├── archive.py
    │       │   ├── attachment.py
    │       │   ├── comment.py
    │       │   ├── label.py
    │       │   ├── link.py
    │       │   ├── reaction.py
    │       │   ├── relation.py
    │       │   ├── sub_issue.py
    │       │   ├── subscriber.py
    │       │   └── version.py
    │       ├── cycle/
    │       │   ├── base.py
    │       │   ├── archive.py
    │       │   └── issue.py
    │       ├── module/
    │       │   ├── base.py
    │       │   ├── archive.py
    │       │   └── issue.py
    │       ├── page/
    │       │   ├── base.py
    │       │   └── version.py
    │       ├── analytic/
    │       │   ├── base.py
    │       │   ├── advance.py
    │       │   └── project_analytics.py
    │       ├── asset/
    │       │   ├── base.py
    │       │   └── v2.py
    │       ├── integration/
    │       │   └── base.py            ← endpoints de integraciones
    │       ├── importer/
    │       │   ├── github.py
    │       │   └── gitlab.py
    │       ├── external/
    │       │   ├── base.py
    │       │   └── sync.py
    │       ├── project/
    │       │   ├── base.py
    │       │   ├── member.py
    │       │   └── invite.py
    │       └── ...
    │
    ├── api/                           ← API pública (API tokens) — subconjunto de app/
    │   ├── middleware/
    │   │   └── api_authentication.py  ← auth por API token
    │   ├── serializers/               ← 14 serializadores
    │   ├── urls/                      ← 14 archivos de rutas
    │   └── views/                     ← handlers para API pública
    │
    ├── authentication/                ← autenticación y OAuth (equiv: pendiente en Rust)
    │   ├── adapter/
    │   │   ├── base.py               ← AuthAdapter base
    │   │   ├── credential.py
    │   │   ├── oauth.py
    │   │   └── error.py
    │   ├── provider/
    │   │   ├── credentials/
    │   │   │   ├── email.py          ← login email/password
    │   │   │   └── magic_code.py     ← magic link
    │   │   └── oauth/
    │   │       ├── github.py         ← OAuth GitHub
    │   │       ├── gitlab.py         ← OAuth GitLab
    │   │       ├── google.py         ← OAuth Google
    │   │       └── gitea.py          ← OAuth Gitea
    │   ├── views/
    │   │   ├── app/                  ← vistas de autenticación para la app
    │   │   │   ├── email.py
    │   │   │   ├── magic.py
    │   │   │   ├── github.py
    │   │   │   ├── gitlab.py
    │   │   │   ├── google.py
    │   │   │   ├── gitea.py
    │   │   │   ├── password_management.py
    │   │   │   ├── check.py
    │   │   │   └── signout.py
    │   │   └── space/                ← vistas auth para space (deploy público)
    │   │       └── ...               ← espejo de app/
    │   ├── middleware/
    │   │   └── session.py
    │   ├── session.py
    │   ├── rate_limit.py
    │   └── urls.py
    │
    ├── space/                         ← API pública sin auth para deploy boards
    │   ├── serializer/
    │   ├── urls/
    │   └── views/
    │
    ├── license/                       ← gestión de instancia/licencia (CE vs Cloud)
    │   ├── api/
    │   │   ├── serializers/
    │   │   ├── views/
    │   │   │   ├── instance.py       ← config de instancia
    │   │   │   ├── admin.py
    │   │   │   └── configuration.py
    │   │   └── permissions/
    │   ├── models/
    │   │   └── instance.py           ← Instance, InstanceConfiguration
    │   ├── migrations/               ← 6 migraciones de licencia
    │   ├── bgtasks/
    │   │   └── tracer.py             ← telemetría
    │   └── management/commands/
    │       ├── register_instance.py
    │       └── configure_instance.py
    │
    ├── analytics/                     ← módulo de analytics (MongoDB)
    │
    ├── utils/                         ← utilidades compartidas
    │   ├── cache.py                   ← helpers Redis
    │   ├── paginator.py               ← paginación cursor-based
    │   ├── grouper.py                 ← agrupación de querysets
    │   ├── issue_filters.py           ← filtros de issues
    │   ├── issue_search.py
    │   ├── email.py                   ← envío de emails
    │   ├── github_app.py              ← cliente GitHub App
    │   ├── filters/
    │   │   ├── filter_backend.py
    │   │   └── filterset.py
    │   ├── exporters/
    │   │   ├── exporter.py
    │   │   └── formatters.py
    │   ├── permissions/               ← helpers de permisos reutilizables
    │   │   ├── workspace.py
    │   │   ├── project.py
    │   │   └── page.py
    │   ├── openapi/                   ← decoradores Swagger
    │   │   ├── decorators.py
    │   │   ├── auth.py
    │   │   └── parameters.py
    │   └── instance_config_variables/
    │       ├── core.py
    │       └── extended.py
    │
    └── tests/                         ← suite de tests Django
        ├── conftest.py
        ├── factories.py               ← factory_boy fixtures
        ├── unit/                      ← tests unitarios
        │   ├── models/
        │   ├── serializers/
        │   ├── middleware/
        │   ├── bg_tasks/
        │   └── settings/
        ├── contract/                  ← tests de contrato por API
        │   ├── api/                   ← API pública
        │   └── app/                   ← API autenticada
        └── smoke/                     ← smoke tests
```

---

## Módulos Django → equivalente Rust

| Módulo Django | Propósito | Equivalente en Rust | Estado |
|---------------|-----------|---------------------|--------|
| `plane/db/models/` | ORM Models (33 archivos) | `src/entities/` (122 entidades generadas) | ✅ Baseline completo |
| `plane/db/mixins.py` | SoftDelete, Timestamps | `src/utils/soft_delete.rs` | ✅ Implementado |
| `plane/db/migrations/` | 126 migraciones históricas | `migration/src/migrations/m001_baseline.rs` | ✅ Consolidado en baseline |
| `plane/bgtasks/` | 36 tareas Celery | `src/jobs/` (apalis) | 🔄 Parcial |
| `plane/bgtasks/workspace_seed_task.py` | Seed inicial de workspace | `src/jobs/workspace_seed/` | 📝 Diseñado |
| `plane/bgtasks/github_sync_task.py` | Sync GitHub | `src/jobs/github_sync.rs` | 📝 Planificado |
| `plane/bgtasks/notification_task.py` | Notificaciones | `src/jobs/notifications.rs` | 📝 Planificado |
| `plane/app/views/` | Handlers DRF | `src/routes/` | 🔄 Fase 2 |
| `plane/app/permissions/` | Guards de acceso | `src/auth/permissions.rs` | ✅ Base implementada |
| `plane/app/middleware/api_authentication.py` | JWT/Session auth | `src/auth/middleware.rs` | ✅ Implementado |
| `plane/authentication/` | OAuth providers | Pendiente Fase 3 | 📝 No iniciado |
| `plane/settings/` | Config por entorno | `src/config.rs` (dotenvy) | ✅ Implementado |
| `plane/celery.py` | Broker Celery | apalis (PostgreSQL-backed) | ✅ Apalis configurado |
| `plane/db/models/integration/` | Modelos integración | `migration/src/migrations/m006` | 🔄 Pendiente prueba |
| `plane/license/` | Gestión instancia | Pendiente | 📝 No iniciado |
| `plane/space/` | API pública deploy | Pendiente | 📝 No iniciado |
| `plane/utils/` | Utilidades | `src/utils/` | 🔄 Parcial |

---

## Modelos de integración Django (críticos para m006)

Estos modelos son los que se materializan en la migración `m006_integrations`:

```
plane/db/models/integration/
├── base.py
│   ├── WorkspaceIntegration     ← workspace ↔ proveedor OAuth
│   └── ProjectIntegration       ← proyecto ↔ workspace_integration
├── github.py
│   ├── GithubRepository         ← repos importados
│   ├── GithubIssue              ← issues importados de GitHub
│   └── GithubCommentSync        ← sync de comentarios
├── github_pr_state.py
│   └── GithubPRStateMapping     ← mapeo PR state → Plane state
├── user_github_connection.py
│   └── UserGithubConnection     ← token OAuth por usuario
├── gitlab.py
│   ├── GitlabIssueSync          ← sync issues GitLab
│   └── GitlabMilestoneSync      ← sync milestones → cycles
├── slack.py
│   └── SlackProjectSync         ← notificaciones Slack por proyecto
└── signals.py                   ← Django signals (no aplica en Rust)
```

---

## Background tasks Celery → Rust jobs mapping

| Celery task | Equivalente Rust | Trigger |
|-------------|-----------------|---------|
| `workspace_seed_task` | `WorkspaceSeedJob` (apalis) | Evento: workspace creado |
| `github_sync_task` | `GithubSyncJob` | Webhook GitHub / cron |
| `notification_task` | `NotificationJob` | Cola de eventos |
| `email_notification_task` | `EmailJob` | Cola de eventos |
| `webhook_task` | `WebhookDispatchJob` | Post-mutación |
| `export_task` | `ExportJob` | Request usuario |
| `cleanup_task` | `CleanupCron` | `tokio-cron-scheduler` |
| `issue_automation_task` | `AutomationCron` | Cron diario |
| `page_version_task` | `PageVersionJob` | Cola de eventos |
| `magic_link_code_task` | — | Pendiente auth Rust |
| `forgot_password_task` | — | Pendiente auth Rust |

---

## Migraciones históricas más relevantes

Las migraciones clave que influyeron en el schema baseline de Rust:

| Migración Django | Qué introduce |
|------------------|---------------|
| `0001_initial` | Schema base 2022 |
| `0047_webhook_*` | Webhooks y API tokens |
| `0063_state_is_triage` | Estado triage |
| `0085_intake_*` | Módulo Intake (reemplaza Inbox) |
| `0101_description_descriptionversion` | Versiones de descripción |
| `0122_add_github_gitlab_integrations` | Modelos integración GitHub/GitLab |
| `0123_add_slack_integration` | Modelo Slack |
| `0124_githubprstatemapping_usergithubconnection` | PR mapping + user OAuth |
| `0126_gitlab_sync_models` | GitLab sync completo |

> Estas últimas 5 migraciones son las que materializa la migración Rust `m006`.

---

## Comandos Django relevantes para la migración

```bash
# Django: esperar DB
python manage.py wait_for_db

# Django: aplicar migraciones
python manage.py migrate

# Django: seed de datos de instancia
python manage.py configure_instance

# Django: crear admin
python manage.py create_instance_admin

# Rust equivalente: aplicar migraciones
cargo run -p migration -- up

# Rust equivalente: seed (job apalis)
# Se dispara automáticamente vía WorkspaceSeedJob al crear workspace
```

---

*Generado automáticamente desde `apps/api/` — rama `feature/integrations-panel-fix-17593507967815292912`*
