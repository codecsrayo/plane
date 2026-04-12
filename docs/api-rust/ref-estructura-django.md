---
titulo: Estructura de carpetas Django — fuente de migración
aliases:
  - estructura-django
  - arbol-django
tags:
  - django
  - estructura
  - referencia
  - migracion
relacionado:
  - "[[MOC]]"
  - "[[ref-estructura-archivos]]"
  - "[[plan-fases]]"
  - "[[fundamentos-orm]]"
  - "[[impl-autenticacion]]"
estado: activo
---

# Estructura de carpetas Django — `apps/api`

> [!INFO] Propósito
> Referencia completa del proyecto Django que se está migrando a Rust.
> El árbol refleja el estado actual de la rama `feature/integrations-panel-fix-17593507967815292912`.

---

## Árbol completo

```
apps/api/
├── manage.py                          ← entry point Django (equiv: src/main.rs)
└── plane/
    ├── settings/                      ← configuración por entorno
    │   ├── common.py                  ← settings base (equiv: src/config.rs)
    │   ├── redis.py                   ← Redis/cache config
    │   └── storage.py                 ← S3/MinIO config
    │
    ├── celery.py                      ← configuración Celery broker (equiv: src/jobs/)
    ├── urls.py                        ← root URL dispatcher (equiv: src/routes/mod.rs)
    │
    ├── db/                            ← capa de datos principal
    │   ├── mixins.py                  ← SoftDeleteMixin (equiv: src/utils/soft_delete.rs)
    │   ├── models/                    ← 33 archivos de modelos ORM
    │   │   ├── base.py               ← BaseModel con UUID, timestamps, soft-delete
    │   │   ├── user.py               ← User, Profile, Account, Device
    │   │   ├── workspace.py          ← Workspace, WorkspaceMember, etc.
    │   │   ├── project.py            ← Project, ProjectMember, etc.
    │   │   ├── issue.py              ← Issue, IssueActivity, etc.
    │   │   ├── state.py              ← State
    │   │   ├── label.py              ← Label
    │   │   ├── cycle.py              ← Cycle, CycleIssue, etc.
    │   │   ├── module.py             ← Module, ModuleIssue, etc.
    │   │   ├── page.py               ← Page, PageLog, etc.
    │   │   ├── description.py        ← Description, DescriptionVersion
    │   │   ├── view.py               ← IssueView, IssueViewFavorite
    │   │   ├── notification.py       ← Notification
    │   │   ├── webhook.py            ← Webhook, WebhookLog
    │   │   ├── intake.py             ← Intake, IntakeIssue
    │   │   ├── estimate.py           ← Estimate, EstimatePoint
    │   │   ├── asset.py              ← FileAsset
    │   │   ├── api.py                ← APIToken
    │   │   ├── draft.py              ← DraftIssue
    │   │   ├── exporter.py           ← ExporterHistory
    │   │   ├── deploy_board.py       ← DeployBoard
    │   │   ├── session.py            ← Session (tabla custom con user_id directo)
    │   │   └── integration/          ← modelos de integraciones
    │   │       ├── base.py           ← WorkspaceIntegration
    │   │       ├── github.py         ← GithubRepository, GithubIssue
    │   │       ├── github_pr_state.py← GithubPRStateMapping
    │   │       ├── gitlab.py         ← GitlabIssueSync
    │   │       ├── slack.py          ← SlackProjectSync
    │   │       └── user_github_connection.py ← UserGithubConnection
    │   │
    │   ├── migrations/               ← 126 migraciones históricas
    │   │   ├── 0001_initial.py       ← schema inicial 2022
    │   │   ├── ...
    │   │   └── 0126_gitlab_sync_models.py   ← última migración
    │   │
    │   └── management/commands/      ← comandos Django CLI (equiv: apalis jobs + cron)
    │       ├── configure_instance.py ← equivale a workspace_seed en Rust
    │       └── ...
    │
    ├── bgtasks/                       ← 36 tareas Celery (equiv: src/jobs/)
    │   ├── workspace_seed_task.py     ← seed de workspace (equiv: jobs/workspace_seed/)
    │   ├── github_sync_task.py        ← sync GitHub (equiv: jobs/github_sync.rs)
    │   ├── notification_task.py       ← notificaciones
    │   ├── export_task.py             ← exportación CSV/Excel
    │   ├── webhook_task.py            ← despacho de webhooks
    │   ├── cleanup_task.py            ← limpieza de assets expirados
    │   └── ...
    │
    ├── app/                           ← API principal autenticada (equiv: src/routes/)
    │   ├── permissions/               ← guards de acceso (equiv: src/auth/permissions.rs)
    │   │   ├── base.py               ← God mode workspace admin override
    │   │   ├── workspace.py           ← WorkspaceMemberPermission
    │   │   └── project.py             ← ProjectMemberPermission
    │   ├── middleware/
    │   │   └── api_authentication.py  ← Session + Token auth (equiv: src/auth/)
    │   ├── serializers/               ← 24 serializadores DRF (equiv: DTOs en Rust)
    │   ├── urls/                      ← 24 archivos de rutas
    │   └── views/                     ← handlers DRF (equiv: src/routes/)
    │       ├── workspace/
    │       ├── issue/
    │       ├── cycle/
    │       ├── module/
    │       ├── page/
    │       ├── integration/
    │       └── project/
    │
    ├── authentication/                ← autenticación y OAuth
    │   ├── provider/
    │   │   ├── credentials/
    │   │   │   ├── email.py          ← login email/password
    │   │   │   └── magic_code.py     ← magic link
    │   │   └── oauth/
    │   │       ├── github.py         ← OAuth GitHub
    │   │       ├── gitlab.py         ← OAuth GitLab
    │   │       ├── google.py         ← OAuth Google
    │   │       └── gitea.py          ← OAuth Gitea
    │   └── views/app/                ← vistas de autenticación
    │       ├── email.py
    │       ├── github.py
    │       └── signout.py
    │
    ├── license/                       ← gestión de instancia/licencia
    │   ├── models/
    │   │   └── instance.py           ← Instance, InstanceConfiguration
    │   └── api/views/
    │       └── instance.py           ← config de instancia (God Mode)
    │
    └── utils/                         ← utilidades compartidas
        ├── github_app.py              ← cliente GitHub App (equiv: src/utils/github_app.rs)
        ├── paginator.py               ← paginación cursor-based
        ├── issue_filters.py           ← filtros de issues
        └── instance_config_variables/ ← variables de configuración de instancia
```

---

## Módulos Django → equivalente Rust

| Módulo Django                                | Propósito                | Equivalente Rust                            | Estado         |
| -------------------------------------------- | ------------------------ | ------------------------------------------- | -------------- |
| `plane/db/models/`                           | ORM Models (33 archivos) | `src/entities/` (122 entidades)             | ✅ Baseline    |
| `plane/db/mixins.py`                         | SoftDelete, Timestamps   | `src/utils/soft_delete.rs`                  | ✅             |
| `plane/db/migrations/`                       | 126 migraciones          | `migration/src/migrations/m001_baseline.rs` | ✅             |
| `plane/bgtasks/`                             | 36 tareas Celery         | `src/jobs/` (apalis)                        | 🔄 Parcial     |
| `plane/bgtasks/workspace_seed_task.py`       | Seed workspace           | `src/jobs/workspace_seed/`                  | 📝 Diseñado    |
| `plane/bgtasks/github_sync_task.py`          | Sync GitHub              | `src/jobs/github_sync.rs`                   | 📝 Planificado |
| `plane/app/views/`                           | Handlers DRF             | `src/routes/`                               | 🔄 Fase 2      |
| `plane/app/permissions/`                     | Guards de acceso         | `src/auth/permissions.rs`                   | ✅ Base        |
| `plane/app/middleware/api_authentication.py` | Session + Token auth     | `src/auth/`                                 | 📝 Diseñado    |
| `plane/authentication/`                      | OAuth providers          | Pendiente Fase 3                            | 📝 No iniciado |
| `plane/settings/`                            | Config por entorno       | `src/config.rs` (dotenvy)                   | ✅             |
| `plane/celery.py`                            | Broker Celery            | apalis (PostgreSQL-backed)                  | ✅             |
| `plane/db/models/integration/`               | Modelos integración      | `migration/src/migrations/m001 (baseline)`  | ✅             |
| `plane/license/`                             | Gestión instancia        | Pendiente                                   | 📝 No iniciado |
| `plane/utils/github_app.py`                  | Cliente GitHub App       | `src/utils/github_app.rs`                   | 📝 Diseñado    |

---

## Background tasks Celery → apalis jobs

| Celery task               | Equivalente Rust     | Trigger               |
| ------------------------- | -------------------- | --------------------- |
| `workspace_seed_task`     | `WorkspaceSeedJob`   | Workspace creado      |
| `github_sync_task`        | `GithubSyncJob`      | Webhook GitHub / cron |
| `notification_task`       | `NotificationJob`    | Cola de eventos       |
| `email_notification_task` | `EmailJob`           | Cola de eventos       |
| `webhook_task`            | `WebhookDispatchJob` | Post-mutación         |
| `export_task`             | `ExportJob`          | Request usuario       |
| `cleanup_task`            | `CleanupCron`        | tokio-cron-scheduler  |
| `issue_automation_task`   | `AutomationCron`     | Cron diario           |
| `magic_link_code_task`    | —                    | Pendiente auth Rust   |

---

## Migraciones históricas más relevantes

| Migración Django                                 | Qué introduce                               |
| ------------------------------------------------ | ------------------------------------------- |
| `0001_initial`                                   | Schema base 2022                            |
| `0047_webhook_*`                                 | Webhooks y API tokens                       |
| `0085_intake_*`                                  | Módulo Intake                               |
| `0101_description_descriptionversion`            | Versiones de descripción                    |
| `0122_add_github_gitlab_integrations`            | Modelos integración GitHub/GitLab           |
| `0123_add_slack_integration`                     | Modelo Slack                                |
| `0124_githubprstatemapping_usergithubconnection` | PR mapping + user OAuth                     |
| `0126_gitlab_sync_models`                        | GitLab sync completo — **última migración** |

> Las últimas 5 son las que materializa la migración Rust `m001` (Baseline SQL).

---

## Comandos Django vs Rust equivalentes

```bash
# Django: aplicar migraciones
python manage.py migrate
# Rust equivalente:
cargo run -p migration -- up

# Django: seed de datos de instancia
python manage.py configure_instance
# Rust equivalente:
# Se dispara automáticamente vía WorkspaceSeedJob al crear workspace

# Django: esperar DB
python manage.py wait_for_db
# Rust equivalente:
# El binario intenta conectar con reintentos al arrancar
```

---

## 🔗 Navegar

← [[ref-estructura-archivos]] | [[MOC]]

**Relacionado:** ORM y migraciones: [[fundamentos-orm]] | Autenticación Django → Rust: [[impl-autenticacion]] | Plan de migración: [[plan-fases]]
