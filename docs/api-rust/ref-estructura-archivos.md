---
titulo: Estructura completa de archivos — proyecto Rust
aliases:
  - estructura-archivos
  - arbol-rust
tags:
  - estructura
  - rust
  - archivos
relacionado:
  - "[[MOC]]"
  - "[[impl-bootstrap]]"
  - "[[impl-appstate-repository]]"
  - "[[ref-estructura-django]]"
  - "[[ref-testing]]"
estado: activo
---

# Estructura completa de archivos — estado objetivo

```
apps/api_rust/
├── Cargo.toml
├── Dockerfile
├── seeds/
│   └── data/
│       ├── projects.json    ← copiado de apps/api/plane/seeds/data/
│       ├── states.json
│       ├── labels.json
│       ├── cycles.json
│       ├── modules.json
│       ├── issues.json
│       ├── views.json
│       └── pages.json
├── src/
│   ├── main.rs              ← bootstrap: AppState + router + workers + scheduler
│   ├── config.rs            ← env vars tipadas (dotenvy)
│   ├── error.rs             ← AppError → HTTP responses (thiserror)
│   ├── lib.rs               ← re-exports públicos
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── extractors.rs    ← WorkspaceMemberGuard, ProjectMemberGuard
│   │   ├── session.rs       ← SessionUser extractor (Cookie)
│   │   ├── api_key.rs       ← ApiKeyUser extractor (X-Api-Key)
│   │   ├── any_auth.rs      ← AnyAuth — session OR api key
│   │   ├── csrf.rs          ← GET /auth/get-csrf-token/
│   │   ├── logout.rs        ← handlers POST /auth/sign-out/ y /auth/spaces/sign-out/
│   │   ├── rate_limit.rs    ← middleware Tower rate limit
│   │   └── permissions.rs   ← require_role(), constantes ROLE_*
│   ├── entities/            ← 122 entidades generadas por sea-orm-cli (NO editar)
│   │   ├── mod.rs           ← impl_soft_delete! aplicado a las 98 entidades con deleted_at
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   ├── workspace_members.rs
│   │   ├── users.rs
│   │   ├── sessions.rs
│   │   ├── api_tokens.rs
│   │   ├── authtoken_token.rs
│   │   └── ...              ← 114 entidades más
│   ├── repositories/        ← acceso a DB aislado, un archivo por dominio
│   │   ├── mod.rs
│   │   ├── issues.rs        ← list_issues, get_issue, create_issue, update_issue
│   │   ├── workspaces.rs    ← get_workspace_by_slug, list_workspaces_for_user
│   │   ├── projects.rs
│   │   ├── states.rs
│   │   └── ...
│   ├── routes/              ← handlers Axum, llaman a repositories
│   │   ├── mod.rs           ← build_router() + OpenApi struct + ApiDoc
│   │   ├── health.rs        ← GET /api/health
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   ├── cycles.rs
│   │   ├── modules.rs
│   │   └── integrations.rs
│   ├── jobs/                ← apalis workers + cron
│   │   ├── mod.rs           ← build_monitor() — registra todos los workers
│   │   ├── workspace_seed/
│   │   │   ├── mod.rs       ← WorkspaceSeedJob, handle_workspace_seed
│   │   │   └── seed_data.rs ← structs de deserialización de JSON
│   │   ├── github_sync.rs   ← GithubInitialIssueSyncJob
│   │   ├── notifications.rs ← NotificationJob
│   │   ├── export.rs        ← ExportJob
│   │   ├── webhooks.rs      ← WebhookDeliveryJob
│   │   └── scheduled.rs     ← tokio-cron-scheduler (reemplaza beatworker)
│   └── utils/
│       ├── mod.rs
│       ├── soft_delete.rs   ← SoftDeleteExt trait + impl_soft_delete! macro ✅
│       ├── github_app.rs    ← get_installation_access_token (JWT RS256)
│       ├── instance_config.rs ← get_instance_config (DB + env fallback)
│       ├── oauth_popup.rs   ← postmessage_html helper (GitHub, GitLab, Slack callbacks)
│       └── github_webhooks.rs ← register_github_webhook
├── migration/
│   ├── Cargo.toml           ← incluye dotenvy
│   └── src/
│       ├── lib.rs            ← Migrator con todas las migraciones en orden
│       ├── main.rs           ← auto-carga .env, construye DATABASE_URL desde POSTGRES_*
│       └── migrations/
│           ├── mod.rs
│           ├── m20260410_000001_baseline.rs         ← schema completo desde Django ✅
│           ├── m20240101_000006_integrations.rs     ← GitHub, GitLab, Slack 🔄
│           └── m20240101_000007_seed_data.rs        ← integrations + instance_configs
└── tests/
    └── bruno/
        ├── bruno.json
        ├── environments/
        │   ├── local.bru
        │   └── staging.bru
        └── health/
            └── get_health.bru
```

---

## Reglas de la estructura

| Directorio          | Regla                                                                      |
| ------------------- | -------------------------------------------------------------------------- |
| `src/entities/`     | **NO editar a mano** — regenerar con `sea-orm-cli generate entity`         |
| `src/repositories/` | Un archivo por dominio; los handlers llaman aquí, no a SeaORM directamente |
| `src/routes/`       | Handlers puros — reciben AppState, llaman a repositories, retornan JSON    |
| `src/jobs/`         | Un archivo por job apalis; un archivo `scheduled.rs` para cron             |
| `src/auth/`         | Extractores, session, api_key — todo lo relacionado con autenticación      |
| `src/utils/`        | Helpers reutilizables — soft delete, GitHub App, OAuth popup               |
| `seeds/data/`       | Copiar desde `apps/api/plane/seeds/data/*.json` — no modificar             |
| `tests/bruno/`      | Colecciones versionadas en git — correr con `bruno run --env local`        |

---

## Correspondencia Django → Rust

| Directorio Django                            | Equivalente Rust                            |
| -------------------------------------------- | ------------------------------------------- |
| `plane/db/models/`                           | `src/entities/` (generado)                  |
| `plane/db/mixins.py` (SoftDelete)            | `src/utils/soft_delete.rs`                  |
| `plane/db/migrations/` (126 archivos)        | `migration/src/migrations/m001_baseline.rs` |
| `plane/bgtasks/` (36 tasks Celery)           | `src/jobs/` (apalis)                        |
| `plane/app/views/` (handlers DRF)            | `src/routes/`                               |
| `plane/app/permissions/`                     | `src/auth/permissions.rs` + extractors      |
| `plane/app/middleware/api_authentication.py` | `src/auth/extractors.rs` + `session.rs`     |
| `plane/settings/`                            | `src/config.rs`                             |
| `plane/utils/`                               | `src/utils/`                                |
| `plane/celery.py`                            | apalis en `src/jobs/mod.rs`                 |

Ver [[ref-estructura-django]] para el árbol completo de Django.

---

## 🔗 Navegar

← [[MOC]] | Estructura Django: [[ref-estructura-django]] | Bootstrap: [[impl-bootstrap]] | Testing: [[ref-testing]]

---

_`docs/api-rust/ref-estructura-archivos.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
