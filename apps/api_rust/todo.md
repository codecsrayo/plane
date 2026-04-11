# Plane — API Rust: Plan de migración

## Objetivo

Reemplazar la API Django + Celery por un stack Rust completo que toma ownership
total del schema PostgreSQL. Django desaparece incluyendo sus migraciones.

**Metas:**
- ~750 MB (Django + Celery + RabbitMQ) → ~20 MB
- Eliminar bgworker, beatworker, RabbitMQ
- Rust es el dueño del schema — no más migraciones Django
- Mismo PostgreSQL, mismo Redis/Valkey, mismo plane-live

---

## plane-live — NO se toca

`plane-live` es un servidor **Hocuspocus (Y.js CRDT)** para edición colaborativa
de Pages e issue descriptions. Implementa sincronización CRDT por WebSocket,
persiste estado binario Y.js + HTML via HTTP a la API, y usa Redis para sync
entre múltiples instancias. Es Node.js (~60 MB), no Python. No es el problema.

**Se mantiene intacto.** Solo se migra Django → Rust.

---

## ORM: SeaORM

**SeaORM es el único ORM del proyecto. No se usa Diesel.**

Razones:

| Característica | SeaORM |
|---|---|
| Async nativo Tokio | ✅ |
| Migrations en Rust | ✅ sea-orm-migration |
| Generar entities desde DB existente | ✅ `sea-orm-cli generate entity` |
| Relaciones FK / M2M | ✅ has_many, belongs_to, many_to_many |
| Soft delete integrado | ✅ con ActiveModel hooks o `sea-orm-softdelete` |
| Con Axum | ✅ natural |

El factor decisivo es `sea-orm-cli generate entity --database-url $DATABASE_URL`:
apunta al Postgres existente (con el schema de Django) y genera automáticamente
todos los entities Rust. Con 127 migraciones y ~50 modelos Django, esto ahorra
semanas de trabajo de transcripción manual.

---

## Migraciones: sea-orm-migration (Rust toma ownership del schema)

Django desaparece — Rust es el nuevo dueño del schema. El flujo es:

### Paso 1 — Baseline (una sola vez, al inicio de Fase 0)

Volcar el schema actual de Django como una migración baseline en SeaORM:

```bash
# 1. Generar el SQL del schema actual de Django
docker compose run --rm api python manage.py sqlmigrate ... # o pg_dump --schema-only

# 2. Crear la migración baseline en SeaORM
sea-orm-cli migrate generate "baseline_from_django"
# → editar el archivo generado para incluir el SQL del dump

# 3. Generar todos los entities desde la DB existente
sea-orm-cli generate entity \
  --database-url postgres://... \
  --output-dir src/entities \
  --with-serde both \
  --date-time-crate time
```

### Paso 2 — Todas las migraciones futuras en Rust

```bash
# Crear nueva migración
sea-orm-cli migrate generate "add_column_x_to_issues"

# Aplicar en dev
sea-orm-cli migrate up

# Revertir
sea-orm-cli migrate down
```

Cada migración es un archivo Rust con `up()` y `down()`:

```rust
// m20240101_000001_add_column_x_to_issues.rs
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Issues::Table)
                    .add_column(ColumnDef::new(Issues::PriorityWeight).integer().not_null().default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Issues::Table)
                    .drop_column(Issues::PriorityWeight)
                    .to_owned(),
            )
            .await
    }
}
```

### En producción (CI/CD)

```bash
# El binario Rust aplica sus propias migraciones al arrancar
# (o vía comando separado antes del deploy)
./plane-api migrate up
```

`plane-migrator` (Django) desaparece del docker-compose.

---

## Stack completo

| Rol | Librería | Reemplaza |
|---|---|---|
| HTTP framework | **Axum** | Django REST Framework + uvicorn |
| Async runtime | **Tokio** | — |
| ORM | **SeaORM** | Django ORM |
| Migrations | **sea-orm-migration** | Django migrations (127 archivos) |
| Serialización | **serde + serde_json** | DRF serializers |
| Auth JWT | **jsonwebtoken** | DRF TokenAuthentication |
| Background jobs | **apalis** (backend: Postgres) | Celery bgworker + RabbitMQ |
| Cron jobs | **tokio-cron-scheduler** | Celery beatworker |
| Redis | **fred** | django-redis |
| S3 / MinIO | **aws-sdk-s3** | boto3 + django-storages |
| Email | **lettre** | Django email backend |
| HTTP client | **reqwest** | requests |
| Logging | **tracing + tracing-subscriber** | python-json-logger |
| Config | **dotenvy** | django settings |
| Métricas | **axum-prometheus** | scout-apm |
| API Docs | **utoipa + utoipa-swagger-ui** | drf-spectacular |

### Lo que desaparece

| Contenedor | RAM | Resultado |
|---|---|---|
| api (Django + uvicorn) | ~280 MB | → Rust ~20 MB |
| bgworker (Celery) | ~200 MB | → eliminado |
| beatworker (Celery beat) | ~150 MB | → eliminado |
| plane-mq (RabbitMQ) | ~120 MB | → eliminado |
| plane-migrator (Django) | — | → eliminado |
| **Total** | **~750 MB** | **~20 MB** |

### Lo que se mantiene

| Servicio | Por qué |
|---|---|
| plane-live (Hocuspocus/Node.js) | protocolo Y.js CRDT — no reemplazable |
| plane-db (PostgreSQL) | misma DB, Rust toma ownership del schema |
| plane-redis (Valkey) | sigue necesario para plane-live y caché |
| plane-minio (MinIO) | sin cambio |
| proxy (Traefik) | el mismo, se agrega routing al contenedor Rust |

---

## Testing de la API

### Documentación interactiva: utoipa + Swagger UI

`utoipa` genera el spec OpenAPI 3.x a partir de macros Rust. Se monta en Axum:

```toml
# Cargo.toml
utoipa = { version = "4", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "6", features = ["axum"] }
```

```rust
// main.rs — montar Swagger UI en /docs
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    .merge(SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi()));
```

Acceder a `http://localhost:8000/docs` para explorar y probar endpoints manualmente.

### Tests automatizados en Rust

**Opción recomendada: `axum-test`**

Permite hacer requests HTTP directamente al router de Axum sin levantar un
servidor TCP real. Ideal para tests de integración rápidos:

```toml
[dev-dependencies]
axum-test = "14"
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::test]
async fn test_get_issues() {
    let app = build_app(test_db_state()).await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/workspaces/my-ws/projects/1/issues/")
        .add_header("Authorization", "Bearer test-token")
        .await;

    response.assert_status_ok();
    response.assert_json_contains(&json!({ "count": 0 }));
}
```

**Alternativa ligera: `httpc-test`**

Más simple, sin estado entre requests. Útil para smoke tests rápidos:

```toml
[dev-dependencies]
httpc-test = "0.1"
```

```rust
#[tokio::test]
async fn test_health() -> httpc_test::Result<()> {
    let hc = httpc_test::new_client("http://localhost:8000")?;
    let res = hc.do_get("/api/health/").await?;
    res.print().await?;
    Ok(())
}
```

### Cliente externo: Bruno (recomendado sobre Postman/Insomnia)

Bruno es open source, almacena las colecciones como archivos en el repo (no
en la nube), y funciona offline. Las colecciones viven en `apps/api_rust/tests/bruno/`.

```bash
# Instalar Bruno
brew install bruno  # macOS
# o descargar desde https://www.usebruno.com/

# Estructura de colección en el repo
apps/api_rust/tests/bruno/
├── bruno.json
├── environments/
│   ├── local.bru
│   └── staging.bru
├── health/
│   └── get_health.bru
├── issues/
│   ├── list_issues.bru
│   └── create_issue.bru
└── workspaces/
    └── list_workspaces.bru
```

**Comparativa de clientes externos:**

| Cliente | Open Source | Archivos en git | Offline | CI/CD |
|---|---|---|---|---|
| **Bruno** | ✅ | ✅ archivos `.bru` | ✅ | ✅ `bruno run` |
| Hoppscotch | ✅ | ⚠️ export manual | ✅ | ⚠️ limitado |
| Postman | ❌ | ❌ cloud propietario | ⚠️ | ✅ Newman |
| Insomnia | ⚠️ | ⚠️ export manual | ✅ | ✅ |

**Veredicto**: Bruno para exploración manual + `axum-test` para tests
automatizados en CI. Son complementarios, no excluyentes.

### Resumen de estrategia de testing

```
Desarrollo manual    → utoipa Swagger UI  (/docs)
Tests automatizados  → axum-test          (cargo test)
Exploración/QA       → Bruno              (colecciones en repo)
CI/CD pipeline       → cargo test + bruno run --env staging
```

---

## Estructura del proyecto

```
apps/api_rust/
├── Cargo.toml
├── src/
│   ├── main.rs                  ← bootstrap: router + AppState + migraciones
│   ├── config.rs                ← env vars tipadas
│   ├── error.rs                 ← AppError → HTTP response
│   ├── auth/
│   │   ├── middleware.rs        ← extractor de token → CurrentUser
│   │   └── permissions.rs      ← workspace/project role checks
│   ├── entities/                ← generado por sea-orm-cli (NO editar a mano)
│   │   ├── issue.rs
│   │   ├── project.rs
│   │   ├── workspace.rs
│   │   ├── state.rs
│   │   └── ...
│   ├── routes/                  ← un archivo por dominio
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   ├── cycles.rs
│   │   ├── modules.rs
│   │   └── integrations.rs
│   └── jobs/                    ← apalis workers + cron
│       ├── github_sync.rs
│       ├── notifications.rs
│       ├── export.rs
│       └── scheduled.rs
├── migration/                   ← sea-orm-migration crate separado
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── m20240101_000000_baseline_from_django.rs   ← dump inicial
│       └── m20240201_000001_...rs                     ← futuras migraciones
├── tests/
│   └── bruno/                   ← colecciones Bruno versionadas en git
│       ├── bruno.json
│       ├── environments/
│       └── ...
└── Dockerfile
```

---

## Soft delete — implementado ✅

**Decisión:** trait custom en `src/utils/soft_delete.rs` — NO se usa `seaorm-soft-delete` (crate 0.1.0, riesgo de incompatibilidad con sea-orm 1.1.x).

**Archivos:**
- `src/utils/soft_delete.rs` — trait `SoftDeleteExt<E>` + macro `impl_soft_delete!`
- `src/entities/mod.rs` — macro aplicado a las 98 entidades con `deleted_at`
- `src/utils/mod.rs` — módulo registrado
- `src/main.rs` — `pub mod utils` agregado

**Uso en rutas:**
```rust
use crate::utils::soft_delete::SoftDeleteExt;

let issues = issues::Entity::find()
    .active()                                      // WHERE deleted_at IS NULL
    .filter(issues::Column::ProjectId.eq(project_id))
    .all(&db)
    .await?;
```

**Errores del borrador original corregidos:**
- `pub use ..utils::...` → `use crate::utils::...` (ruta relativa inválida)
- `pub use SoftDeleteExt` → `use ... as _` (no re-exportar, solo activar impls)

---

## Estrategia de migración — por fases

### Fase 0 — Scaffolding + Baseline (2–3 días)

- [ ] `cargo new plane-api && cargo new migration`
- [ ] `Cargo.toml` con SeaORM, Axum, Tokio, apalis, utoipa
- [ ] Generar baseline SQL desde la DB actual de Django
- [ ] Crear migración `m_baseline_from_django` con ese SQL
- [ ] Generar entities con `sea-orm-cli generate entity`
- [ ] `GET /api/health/` funcionando contra la DB
- [ ] Dockerfile multi-stage
- [ ] Traefik: routing condicional por path
- [ ] Montar Swagger UI en `/docs`
- [ ] Colección Bruno inicial en `tests/bruno/`

### Fase 1 — Auth middleware

- [ ] Leer tabla `authtoken_token` → `CurrentUser` extractor Axum
- [ ] Role check (workspace_member, project_member) via SeaORM
- [ ] Tests de integración con `axum-test` contra DB real

### Fase 2 — Endpoints de alta frecuencia

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/states/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/members/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/modules/`
- [ ] `GET /api/workspaces/{slug}/projects/`

### Fase 3 — Background jobs (eliminar Celery + RabbitMQ)

- [ ] Setup apalis con backend PostgreSQL
- [ ] Migrar todos los Celery tasks a apalis workers
- [ ] Setup tokio-cron-scheduler para tareas periódicas
- [ ] Eliminar `bgworker`, `beatworker`, `plane-mq` del docker-compose

### Fase 4 — Endpoints restantes (~275 paths)

Cubrir el resto priorizando por frecuencia de uso en logs.

### Fase 5 — Shutdown Django completo

- [ ] 0 tráfico hacia el contenedor `api`
- [ ] Eliminar `api`, `plane-migrator` del docker-compose
- [ ] Eliminar el directorio `apps/api/` del repo (o archivar en rama)
- [ ] Rust aplica sus propias migraciones en el deploy

---

## Próximo paso — Fase 0

```bash
cd apps/api_rust

# Inicializar workspace Cargo
cargo init --name plane-api

# Inicializar crate de migraciones
cargo new migration --lib

# Instalar CLI de SeaORM
cargo install sea-orm-cli

# Generar entities desde la DB existente de Django
sea-orm-cli generate entity \
  --database-url "postgres://plane:plane@localhost:5432/plane" \
  --output-dir src/entities \
  --with-serde both
```

---

## Estado de migraciones — seguimiento

### Archivos de migración

| Archivo | Tablas creadas | Estado |
|---|---|---|
| `m20240101_000001_auth_django` | auth_group, auth_group_permissions, auth_permission, django_*, changelogs, instances, integrations | ✅ |
| `m20240101_000002_users_and_sessions` | users, accounts, sessions, devices, device_sessions, file_assets, social_login_connections, user_github_connections, profiles | ✅ |
| `m20240101_000003_workspaces_and_tokens` | workspaces, workspace_members, workspace_member_invites, workspace_themes, workspace_integrations, workspace_user_*, api_tokens, api_activity_logs, webhooks, webhook_logs, notifications, profiles | ✅ |
| `m20240101_000004_projects_and_states` | projects, states, labels, estimates, estimate_points, issue_types, project_*, **project_deploy_boards** ✅ fix | ✅ |
| `m20240101_000005_issues_and_modules` | issues, issue_*, cycles, cycle_*, modules, module_*, pages, page_*, draft_issues, **draft_issue_*** ✅ indexes+UQ fix | ✅ |
| `m20240101_000006_integrations_and_misc` | descriptions, description_versions, intakes, intake_issues, deploy_boards, exporters, importers, github_*, gitlab_*, slack_project_syncs | 🔄 pendiente prueba |

### Fixes aplicados (10 abr 2026)

- `fix`: `needless_borrows_for_generic_args` — removido `&` en 5 llamadas `.name(&format!(...))` en m005
- `fix`: `migration/src/main.rs` — auto-carga `.env` desde raíz del proyecto y construye `DATABASE_URL` desde `POSTGRES_*`
- `fix`: `migration/Cargo.toml` — agregado `dotenvy` como dependencia
- `fix`: `project_deploy_boards` — tabla faltante agregada a m004 (FK a `intakes` sigue diferida a m006)
- `fix`: `draft_issue_assignees/cycles/labels/modules` — agregados indexes y unique constraints parciales (`WHERE deleted_at IS NULL`) en m005

### Comandos de migración

```bash
# Aplicar todas las pendientes
task rust:migrations:up

# Revertir la última
task rust:migrations:down

# Estado completo
task rust:migrations:status

# Drop + re-aplicar todo (dev only)
task rust:migrations:fresh
```
