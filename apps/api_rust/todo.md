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

| Característica                      | SeaORM                                          |
| ----------------------------------- | ----------------------------------------------- |
| Async nativo Tokio                  | ✅                                              |
| Migrations en Rust                  | ✅ sea-orm-migration                            |
| Generar entities desde DB existente | ✅ `sea-orm-cli generate entity`                |
| Relaciones FK / M2M                 | ✅ has_many, belongs_to, many_to_many           |
| Soft delete integrado               | ✅ con ActiveModel hooks o `sea-orm-softdelete` |
| Con Axum                            | ✅ natural                                      |

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
sea migrate generate "baseline_from_django"
# → editar el archivo generado para incluir el SQL del dump

# 3. Generar todos los entities desde la DB existente
sea generate entity \
  --database-url postgres://plane:k1rh814uw1gnCDjTinZK_df7OkAjXJ8QFcjNcFjwsME@localhost/plane \
  --output-dir src/entities \
  --with-serde both \
  --date-time-crate time
```

### Paso 2 — Todas las migraciones futuras en Rust

```bash
# Crear nueva migración
sea migrate generate "add_column_x_to_issues"

# Aplicar en dev
sea migrate up

# Revertir
sea migrate down
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

| Rol             | Librería                         | Reemplaza                        |
| --------------- | -------------------------------- | -------------------------------- |
| HTTP framework  | **Axum**                         | Django REST Framework + uvicorn  |
| Async runtime   | **Tokio**                        | —                                |
| ORM             | **SeaORM**                       | Django ORM                       |
| Migrations      | **sea-orm-migration**            | Django migrations (127 archivos) |
| Serialización   | **serde + serde_json**           | DRF serializers                  |
| Auth JWT        | **jsonwebtoken**                 | DRF TokenAuthentication          |
| Background jobs | **apalis** (backend: Postgres)   | Celery bgworker + RabbitMQ       |
| Cron jobs       | **tokio-cron-scheduler**         | Celery beatworker                |
| Redis           | **fred**                         | django-redis                     |
| S3 / MinIO      | **aws-sdk-s3**                   | boto3 + django-storages          |
| Email           | **lettre**                       | Django email backend             |
| HTTP client     | **reqwest**                      | requests                         |
| Logging         | **tracing + tracing-subscriber** | python-json-logger               |
| Config          | **dotenvy**                      | django settings                  |
| Métricas        | **axum-prometheus**              | scout-apm                        |
| API Docs        | **utoipa + utoipa-swagger-ui**   | drf-spectacular                  |

---

## Arquitectura actual vs futura — diagrama de flujo

### Stack actual (Django + Celery + RabbitMQ)

```mermaid
flowchart TD
    Browser["🌐 Browser / Frontend
(Next.js + React)"]
    Traefik["⚖️ Traefik
(proxy / routing)"]

    subgraph Actual["Stack actual ~750 MB"]
        direction TB
        uvicorn["uvicorn
(servidor ASGI)"]
        django["Django + DRF
(views, serializers, permissions)"]
        orm_django["Django ORM
(127 migraciones)"]
        mq["🐇 RabbitMQ plane-mq
~120 MB"]
        bg["Celery bgworker
~200 MB
(sync GitHub, emails,
exportes, webhooks)"]
        beat["Celery beatworker
~150 MB
(cron: limpiar tokens,
digest notificaciones)"]
        migrator["plane-migrator
(manage.py migrate)"]
    end

    subgraph Infra["Infraestructura compartida"]
        pg[("PostgreSQL
plane-db")]
        redis[("Redis / Valkey
plane-redis")]
        minio[("MinIO
plane-minio")]
        live["plane-live
(Hocuspocus / Y.js CRDT)
Node.js ~60 MB"]
    end

    Browser --> Traefik
    Traefik --> uvicorn
    uvicorn --> django
    django --> orm_django
    orm_django --> pg
    django -- "task.delay()" --> mq
    mq --> bg
    mq --> beat
    bg --> pg
    bg --> redis
    bg --> minio
    beat --> pg
    migrator -.->|"al arrancar"| pg
    live --> pg
    live --> redis
```

### Stack futuro (Rust — ~20 MB total)

```mermaid
flowchart TD
    Browser["🌐 Browser / Frontend
(Next.js + React)"]
    Traefik["⚖️ Traefik
(proxy / routing)"]

    subgraph Futuro["Stack Rust ~20 MB"]
        direction TB
        axum["Axum
(HTTP framework)"]
        seaorm["SeaORM
(ORM + migraciones)"]
        apalis["apalis
(background jobs
vía tabla PostgreSQL)"]
        cron["tokio-cron-scheduler
(cron jobs — mismo proceso)"]
    end

    subgraph Infra["Infraestructura compartida"]
        pg[("PostgreSQL
plane-db")]
        redis[("Redis / Valkey
plane-redis")]
        minio[("MinIO
plane-minio")]
        live["plane-live
(Hocuspocus / Y.js CRDT)
Node.js ~60 MB — NO se toca"]
    end

    Browser --> Traefik
    Traefik --> axum
    axum --> seaorm
    seaorm --> pg
    axum --> apalis
    apalis --> pg
    cron --> apalis
    axum --> redis
    axum --> minio
    live --> pg
    live --> redis
```

### Lo que desaparece

```mermaid
flowchart LR
    A["🐇 RabbitMQ
120 MB"] -- eliminado --> X["❌"]
    B["Celery bgworker
200 MB"] -- eliminado --> X
    C["Celery beatworker
150 MB"] -- eliminado --> X
    D["Django + uvicorn
280 MB"] -- reemplazado --> R["✅ Axum + Rust
~20 MB"]
    E["plane-migrator
Django migrations"] -- reemplazado --> S["✅ sea-orm-migration
(integrado en el binario)"]
```

### Impacto en frontend

> La migración a Rust **no reduce tiempos de carga percibidos** en el browser.
> El cuello de botella es el bundle JS de Next.js y la hydration de React — no la API.
> La ganancia es en el **servidor**: -730 MB RAM, mejor throughput bajo carga concurrente,
> y eliminación de 4 contenedores de infraestructura.

---

### Lo que desaparece

| Contenedor               | RAM         | Resultado     |
| ------------------------ | ----------- | ------------- |
| api (Django + uvicorn)   | ~280 MB     | → Rust ~20 MB |
| bgworker (Celery)        | ~200 MB     | → eliminado   |
| beatworker (Celery beat) | ~150 MB     | → eliminado   |
| plane-mq (RabbitMQ)      | ~120 MB     | → eliminado   |
| plane-migrator (Django)  | —           | → eliminado   |
| **Total**                | **~750 MB** | **~20 MB**    |

### Lo que se mantiene

| Servicio                        | Por qué                                        |
| ------------------------------- | ---------------------------------------------- |
| plane-live (Hocuspocus/Node.js) | protocolo Y.js CRDT — no reemplazable          |
| plane-db (PostgreSQL)           | misma DB, Rust toma ownership del schema       |
| plane-redis (Valkey)            | sigue necesario para plane-live y caché        |
| plane-minio (MinIO)             | sin cambio                                     |
| proxy (Traefik)                 | el mismo, se agrega routing al contenedor Rust |

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

| Cliente    | Open Source | Archivos en git      | Offline | CI/CD          |
| ---------- | ----------- | -------------------- | ------- | -------------- |
| **Bruno**  | ✅          | ✅ archivos `.bru`   | ✅      | ✅ `bruno run` |
| Hoppscotch | ✅          | ⚠️ export manual     | ✅      | ⚠️ limitado    |
| Postman    | ❌          | ❌ cloud propietario | ⚠️      | ✅ Newman      |
| Insomnia   | ⚠️          | ⚠️ export manual     | ✅      | ✅             |

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

| Archivo                                  | Tablas creadas                                                                                                                                                                                       | Estado              |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `m20240101_000001_auth_django`           | auth*group, auth_group_permissions, auth_permission, django*\*, changelogs, instances, integrations                                                                                                  | ✅                  |
| `m20240101_000002_users_and_sessions`    | users, accounts, sessions, devices, device_sessions, file_assets, social_login_connections, user_github_connections, profiles                                                                        | ✅                  |
| `m20240101_000003_workspaces_and_tokens` | workspaces, workspace*members, workspace_member_invites, workspace_themes, workspace_integrations, workspace_user*\*, api_tokens, api_activity_logs, webhooks, webhook_logs, notifications, profiles | ✅                  |
| `m20240101_000004_projects_and_states`   | projects, states, labels, estimates, estimate*points, issue_types, project*\*, **project_deploy_boards** ✅ fix                                                                                      | ✅                  |
| `m20240101_000005_issues_and_modules`    | issues, issue*\*, cycles, cycle*_, modules, module\__, pages, page*\*, draft_issues, \*\*draft_issue*\*\*\* ✅ indexes+UQ fix                                                                        | ✅                  |
| `m20240101_000006_integrations_and_misc` | descriptions, description*versions, intakes, intake_issues, deploy_boards, exporters, importers, github*_, gitlab\__, slack_project_syncs                                                            | 🔄 pendiente prueba |

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

---

## Población de datos iniciales (workspace seed)

### Qué hace el seed en Django

`workspace_seed_task.py` es un **Celery task** disparado con `.delay(workspace_id)`
inmediatamente después de `POST /api/workspaces/`. Crea de forma asíncrona:

```
workspace creado
    └─ bot_user               (User.is_bot=true, bot_type=WORKSPACE_SEED)
    └─ WorkspaceMember        (rol 20 = admin para el bot)
    └─ Project                (nombre = nombre del workspace)
        ├─ ProjectMember      (todos los workspace_members heredan rol)
        ├─ ProjectUserProperty (display_filters + display_properties por user)
        ├─ States             × 5  (Backlog, Todo, In Progress, Done, Cancelled)
        ├─ Labels             × 2  (admin, concepts)
        ├─ Cycles             × 2  (CURRENT: hoy+14d, UPCOMING: siguiente bloque)
        ├─ Modules            × N
        ├─ Issues             × N  (con IssueSequence + IssueActivity + labels/cycles/modules)
        ├─ IssueViews         × N
        └─ Pages              × N  (globales y de proyecto)
```

Los datos de plantilla viven en **8 archivos JSON** en `apps/api/plane/seeds/data/`:

| Archivo          | Descripción                                     |
| ---------------- | ----------------------------------------------- |
| `projects.json`  | 1 proyecto demo con nombre, identifier, logo    |
| `states.json`    | 5 estados con color, grupo (backlog/started/…)  |
| `labels.json`    | 2 labels (admin, concepts)                      |
| `cycles.json`    | 2 ciclos con tipo CURRENT / UPCOMING            |
| `modules.json`   | N módulos con nombre y orden                    |
| `issues.json`    | N issues con description_html, priority, refs   |
| `views.json`     | N vistas con filtros                            |
| `pages.json`     | N páginas con description_html                  |

### Migración a Rust — WorkspaceSeedJob (apalis)

#### Paso 1 — Copiar los JSON al proyecto Rust

```bash
mkdir -p apps/api_rust/seeds/data
cp apps/api/plane/seeds/data/*.json apps/api_rust/seeds/data/
```

Los JSON se incluyen en el binario con `include_str!()` para evitar rutas
en runtime, o se montan como volumen en Docker. Recomendado: `include_str!`
para garantizar que siempre estén presentes.

#### Paso 2 — Estructuras de deserialización

```rust
// src/jobs/workspace_seed/seed_data.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectSeed {
    pub id: i32,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub network: i16,
    pub cover_image: Option<String>,
    pub logo_props: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StateSeed {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub sequence: f64,
    pub group: String,
    pub default: bool,
    pub project_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct LabelSeed {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub sort_order: f64,
    pub project_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CycleSeed {
    pub id: i32,
    pub name: String,
    pub project_id: i32,
    #[serde(rename = "type")]
    pub cycle_type: String, // "CURRENT" | "UPCOMING"
}

#[derive(Debug, Deserialize)]
pub struct IssueSeed {
    pub id: i32,
    pub name: String,
    pub sequence_id: i32,
    pub description_html: Option<String>,
    pub description_stripped: Option<String>,
    pub sort_order: f64,
    pub state_id: i32,
    pub labels: Vec<i32>,
    pub priority: String,
    pub project_id: i32,
    pub cycle_id: Option<i32>,
    pub module_ids: Option<Vec<i32>>,
}
```

#### Paso 3 — El job apalis

```rust
// src/jobs/workspace_seed/mod.rs
use apalis::prelude::*;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSeedJob {
    pub workspace_id: Uuid,
}

pub async fn handle_workspace_seed(
    job: WorkspaceSeedJob,
    ctx: Data<DatabaseConnection>,
) -> Result<(), apalis::prelude::Error> {
    let db = ctx.as_ref();
    seed_workspace(db, job.workspace_id).await
        .map_err(|e| apalis::prelude::Error::Failed(e.to_string().into()))
}

async fn seed_workspace(db: &DatabaseConnection, workspace_id: Uuid) -> anyhow::Result<()> {
    // 1. Verificar idempotencia
    let existing = projects::Entity::find()
        .filter(projects::Column::WorkspaceId.eq(workspace_id))
        .count(db).await?;
    if existing > 0 {
        tracing::info!("Workspace {workspace_id} already seeded, skipping");
        return Ok(());
    }

    // 2. Cargar seeds desde JSON embebido en el binario
    let projects_tpl: Vec<ProjectSeed> = serde_json::from_str(
        include_str!("../../seeds/data/projects.json"))?;
    let states_tpl: Vec<StateSeed> = serde_json::from_str(
        include_str!("../../seeds/data/states.json"))?;
    let labels_tpl: Vec<LabelSeed> = serde_json::from_str(
        include_str!("../../seeds/data/labels.json"))?;
    let cycles_tpl: Vec<CycleSeed> = serde_json::from_str(
        include_str!("../../seeds/data/cycles.json"))?;
    let issues_tpl: Vec<IssueSeed> = serde_json::from_str(
        include_str!("../../seeds/data/issues.json"))?;
    // …modules, views, pages igual

    // 3. Crear bot user + agregar a workspace
    let bot_id = create_bot_user(db, workspace_id).await?;
    add_bot_to_workspace(db, workspace_id, bot_id).await?;

    // 4. Obtener workspace_members para propagar al proyecto
    let members = workspace_members::Entity::find()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
        .all(db).await?;

    // 5. Crear proyecto + miembros + user_properties
    // Mapa seed_id(i32) → real_uuid
    let project_map: HashMap<i32, Uuid> =
        create_project_and_members(db, workspace_id, &projects_tpl, &members, bot_id).await?;

    // 6. Estados, labels, ciclos, módulos — en orden (FKs)
    let state_map  = create_states(db, workspace_id, &states_tpl, &project_map, bot_id).await?;
    let label_map  = create_labels(db, workspace_id, &labels_tpl, &project_map, bot_id).await?;
    let cycle_map  = create_cycles(db, workspace_id, &cycles_tpl, &project_map, bot_id).await?;
    let module_map = create_modules(db, workspace_id, &modules_tpl, &project_map, bot_id).await?;

    // 7. Issues con todas sus relaciones
    create_issues(db, workspace_id, &issues_tpl,
        &project_map, &state_map, &label_map, &cycle_map, &module_map, bot_id).await?;

    // 8. Views y pages
    create_views(db, workspace_id, &views_tpl, &project_map, bot_id).await?;
    create_pages(db, workspace_id, &pages_tpl, &project_map, bot_id).await?;

    tracing::info!("Workspace {workspace_id} seeded successfully");
    Ok(())
}
```

#### Paso 4 — Encolar el job desde el handler de workspaces

```rust
// src/routes/workspaces.rs — POST /api/workspaces/
async fn create_workspace(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    // … crear workspace y workspace_member …

    // Encolar seed job asíncrono — no bloquea la respuesta HTTP
    if let Err(e) = state.job_storage
        .push(WorkspaceSeedJob { workspace_id: new_workspace.id })
        .await
    {
        tracing::warn!("Failed to enqueue workspace seed: {e}");
        // No fallar la request — el seed es best-effort
    }

    Ok(Json(workspace_response))
}
```

#### Mapa de IDs — patrón crítico

Los JSON usan IDs enteros temporales (1, 2, 3…). Al insertar en Postgres
se generan UUIDs reales. `HashMap<i32, Uuid>` resuelve referencias cruzadas:

```rust
let mut state_map: HashMap<i32, Uuid> = HashMap::new();

for seed in &states_tpl {
    let model = states::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(seed.name.clone()),
        // …
    }.insert(db).await?;
    state_map.insert(seed.id, model.id);
}

// Al crear issue: resolver FK
let real_state_id = state_map[&issue_seed.state_id];
```

### Datos globales (estáticos) vs datos de workspace (dinámicos)

| Tipo                        | Dónde                          | Cuándo              |
| --------------------------- | ------------------------------ | ------------------- |
| `integrations` (3 filas)    | migración `m007_seed_data`     | al arrancar DB      |
| `instance_configurations`   | migración `m007_seed_data`     | al arrancar DB      |
| `auth_permission` / grupos  | migración `m001_auth_django`   | al arrancar DB      |
| Proyecto demo + issues      | `WorkspaceSeedJob` (apalis)    | al crear workspace  |
| Bot user por workspace      | `WorkspaceSeedJob` (apalis)    | al crear workspace  |

---

## Patrones de diseño y arquitectura

### 1. Repository Pattern — aislar SeaORM de los handlers

Los handlers Axum no deben contener queries SeaORM directamente. El módulo
`src/repositories/` encapsula todo el acceso a datos:

```
src/
├── repositories/
│   ├── mod.rs
│   ├── issues.rs        ← list_issues, get_issue, create_issue, update_issue
│   ├── workspaces.rs    ← get_workspace_by_slug, list_workspaces_for_user
│   ├── projects.rs
│   └── states.rs
├── routes/
│   └── issues.rs        ← solo recibe AppState, llama a repositories::issues::*
```

```rust
// src/repositories/issues.rs
pub async fn list_issues(
    db: &DatabaseConnection,
    project_id: Uuid,
    filters: IssueFilters,
) -> Result<Vec<issues::Model>, DbErr> {
    issues::Entity::find()
        .active()   // WHERE deleted_at IS NULL — via SoftDeleteExt
        .filter(issues::Column::ProjectId.eq(project_id))
        .order_by_asc(issues::Column::SortOrder)
        .all(db)
        .await
}
```

Ventaja principal: los tests pueden mockear el repository sin levantar DB real.

---

### 2. AppState — estado global del servidor

```rust
// src/main.rs
#[derive(Clone)]
pub struct AppState {
    pub db:          DatabaseConnection,       // pool SeaORM (Postgres)
    pub redis:       fred::clients::Pool,      // pool Redis/Valkey
    pub s3:          aws_sdk_s3::Client,       // cliente S3/MinIO
    pub config:      Arc<Config>,              // env vars tipadas (dotenvy)
    pub job_storage: PgPool,                   // apalis backend (Postgres)
}
```

Se registra en Axum con `.with_state(state)`. Los handlers lo reciben
con `State(state): State<AppState>`.

---

### 3. Extractor Pattern — autenticación y permisos

Axum permite extractors personalizados que corren **antes** del handler,
implementando auth + RBAC sin middleware global:

```rust
// src/auth/middleware.rs

/// Extrae y valida el token desde la tabla authtoken_token.
/// Si falla → 401 automático antes de entrar al handler.
pub struct CurrentUser(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts)?;
        let user = validate_token(&state.as_ref().db, &token).await?;
        Ok(CurrentUser(user))
    }
}

/// Extrae workspace_member con su rol.
/// Falla con 403 si el usuario no es miembro del workspace.
pub struct WorkspaceMemberGuard {
    pub user:   users::Model,
    pub member: workspace_members::Model,
}

/// Extrae project_member. Verifica membership en workspace Y proyecto.
pub struct ProjectMemberGuard {
    pub user:           users::Model,
    pub project_member: project_members::Model,
}
```

Uso en handlers — los guards se componen directamente en la firma:

```rust
// Solo autenticación
async fn list_workspaces(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,  // ← 401 si token inválido
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> { … }

// Auth + membership en workspace
async fn get_project(
    State(state): State<AppState>,
    WorkspaceMemberGuard { user, member }: WorkspaceMemberGuard, // ← 403 si no es miembro
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectResponse>, AppError> { … }
```

---

### 4. Error unificado — AppError

```rust
// src/error.rs
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound      => (StatusCode::NOT_FOUND,            self.to_string()),
            AppError::Unauthorized  => (StatusCode::UNAUTHORIZED,         self.to_string()),
            AppError::Forbidden     => (StatusCode::FORBIDDEN,            self.to_string()),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            AppError::Database(_)   => (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()),
            AppError::Internal(_)   => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

Todos los handlers retornan `Result<T, AppError>`. Los errores SeaORM
y `anyhow` se convierten automáticamente via `#[from]`.

---

### 5. Job Pattern — apalis workers

Cada job sigue el mismo patrón de registro. Todos los workers corren en
el **mismo proceso** que Axum (mismo binario Tokio), eliminando RabbitMQ
y los contenedores bgworker/beatworker:

```rust
// src/jobs/mod.rs
pub fn build_monitor(db: DatabaseConnection) -> Monitor {
    let storage = PostgresStorage::new(db.clone());

    Monitor::new()
        .register(
            WorkerBuilder::new("workspace-seed-worker")
                .data(db.clone())
                .build_fn(workspace_seed::handle_workspace_seed),
        )
        .register(
            WorkerBuilder::new("github-sync-worker")
                .data(db.clone())
                .build_fn(github_sync::handle_github_sync),
        )
        .register(
            WorkerBuilder::new("notification-worker")
                .data(db.clone())
                .build_fn(notifications::handle_notification),
        )
}
```

---

### 6. Cron jobs — tokio-cron-scheduler

Reemplaza Celery beatworker. Corre en el mismo proceso:

```rust
// src/jobs/scheduled.rs
pub async fn start_scheduler(db: DatabaseConnection) -> anyhow::Result<()> {
    let scheduler = JobScheduler::new().await?;

    // Limpieza de tokens expirados (diario 3am UTC)
    scheduler.add(
        Job::new_async("0 0 3 * * *", move |_, _| {
            let db = db.clone();
            Box::pin(async move {
                if let Err(e) = cleanup_expired_tokens(&db).await {
                    tracing::error!("Token cleanup failed: {e}");
                }
            })
        })?
    ).await?;

    scheduler.start().await?;
    Ok(())
}
```

---

## Integración en el flujo — diagramas de secuencia

### Creación de workspace + seed asíncrono

```mermaid
sequenceDiagram
    actor User as 🧑 Usuario
    participant Axum as Axum Handler<br/>(POST /api/workspaces/)
    participant DB as PostgreSQL
    participant Apalis as apalis<br/>(tabla apalis_jobs)
    participant Worker as WorkspaceSeed<br/>Worker (Tokio)

    User->>Axum: POST /api/workspaces/ { name, slug }
    Axum->>DB: INSERT INTO workspaces
    DB-->>Axum: workspace { id, slug }
    Axum->>DB: INSERT INTO workspace_members (owner, role=20)
    Axum->>Apalis: push(WorkspaceSeedJob { workspace_id })
    Note over Apalis: INSERT en apalis_jobs<br/>(no bloquea la respuesta)
    Axum-->>User: 201 Created { workspace }

    Note over Worker: poll cada ~1s
    Worker->>Apalis: pull job
    Worker->>DB: SELECT workspace
    Worker->>DB: INSERT bot_user
    Worker->>DB: INSERT workspace_member (bot)
    Worker->>DB: INSERT project + members + user_properties
    Worker->>DB: INSERT states × 5
    Worker->>DB: INSERT labels × 2
    Worker->>DB: INSERT cycles × 2
    Worker->>DB: INSERT modules × N
    Worker->>DB: INSERT issues × N (+ sequences + activities + label/cycle/module)
    Worker->>DB: INSERT views × N
    Worker->>DB: INSERT pages × N
    Worker-->>Apalis: job completado ✅
```

### Flujo de request autenticado (issues)

```mermaid
sequenceDiagram
    actor User as 🧑 Usuario
    participant Axum as Axum Router
    participant Auth as CurrentUser<br/>Extractor
    participant Guard as ProjectMember<br/>Guard
    participant Repo as repositories::<br/>issues
    participant DB as PostgreSQL

    User->>Axum: GET /api/workspaces/my-ws/projects/abc/issues/
    Note over Axum: Tower middleware: tracing, CORS, gzip
    Axum->>Auth: from_request_parts()
    Auth->>DB: SELECT FROM authtoken_token WHERE key = ?
    Auth->>DB: SELECT FROM users WHERE id = ?
    DB-->>Auth: User ✅
    Axum->>Guard: from_request_parts()
    Guard->>DB: SELECT workspace_members WHERE slug=? AND member_id=?
    Guard->>DB: SELECT project_members WHERE project_id=? AND member_id=?
    DB-->>Guard: ProjectMember { role } ✅
    Axum->>Repo: list_issues(db, project_id, filters)
    Repo->>DB: SELECT FROM issues WHERE deleted_at IS NULL AND project_id=?
    DB-->>Repo: Vec<issues::Model>
    Axum-->>User: 200 OK [{ id, name, state, ... }]
```

---

## Cosas críticas a tener en cuenta

### 1. Bot user — enum `bot_type` en Postgres

El bot user se crea con `is_bot=true` y `bot_type='WORKSPACE_SEED'`.
`bot_type` es un enum Postgres. Verificar que el enum `bot_type_enum`
en la migración baseline incluye el valor `'WORKSPACE_SEED'` antes de
ejecutar el seed job — si no existe → error en runtime.

### 2. Soft delete en todas las entidades del seed

Todas las entidades creadas por el seed tienen `deleted_at = NULL`.
Los queries en rutas deben siempre usar `.active()` para no devolver
registros eliminados por el usuario.

### 3. Ciclos — fechas relativas, no absolutas

Los JSON de `cycles.json` tienen `type: "CURRENT" | "UPCOMING"`, no fechas.
Calcular en runtime:

```rust
let now = Utc::now();
let (start_date, end_date) = match cycle_seed.cycle_type.as_str() {
    "CURRENT"  => (now, now + Duration::days(14)),
    "UPCOMING" => {
        let last = cycles::Entity::find()
            .filter(cycles::Column::ProjectId.eq(real_project_id))
            .order_by_desc(cycles::Column::EndDate)
            .one(db).await?;
        match last {
            Some(c) => (c.end_date + Duration::days(1), c.end_date + Duration::days(15)),
            None    => (now + Duration::days(14), now + Duration::days(28)),
        }
    }
    _ => return Err(anyhow::anyhow!("Unknown cycle type: {}", cycle_seed.cycle_type)),
};
```

### 4. `IssueSequence` — tabla separada obligatoria

Por cada issue creado en el seed se debe crear también un `IssueSequence`.
Es lo que genera el `#` de referencia visible en la UI. Sin esto las issues
no son navegables desde el frontend.

### 5. Dependencias en el seed — orden estricto

```
workspace → bot_user → project → states → labels → cycles → modules → issues → views → pages
```

Issues referencian states, labels, cycles y modules via FK. Insertar fuera
de orden genera violaciones de FK en runtime.

### 6. `ProjectIdentifier` — tabla separada obligatoria

Al crear el proyecto en el seed, crear también la fila en `project_identifiers`:

```rust
project_identifiers::ActiveModel {
    id: Set(Uuid::new_v4()),
    workspace_id: Set(workspace_id),
    project_id: Set(real_project_id),
    identifier: Set(identifier.clone()),
    created_by_id: Set(bot_id),
    ..Default::default()
}.insert(db).await?;
```

Sin esto el proyecto no tiene identifier único y el frontend no puede
construir las rutas de issues (`WS-1`, `WS-2`…).

### 7. `description_html` en issues — insertar verbatim

Las issues del seed tienen HTML rico con imágenes externas
(`media.docs.plane.so`), callouts y listas. No transformar ni validar.
Se pasa como `String` directo a SeaORM.

### 8. `display_filters` y `display_properties` — JSONB exacto

Al crear `ProjectUserProperty` en el seed, usar los mismos defaults que
Django (ver `workspace_seed_task.py` líneas ~90-115). El frontend los
consume directamente sin transformación y espera las claves exactas.

### 9. Entidades Django legacy — NO borrar hasta Fase 5

Las entities `django_celery_beat_*`, `django_content_type`,
`django_migrations`, `django_session` siguen presentes en la DB mientras
Django esté activo. No borrar los archivos `.rs` correspondientes hasta
completar la Fase 5 (shutdown Django).

### 10. `apalis_jobs` table — inicializar antes del primer job

```rust
// En main.rs, al arrancar, antes de registrar workers
PostgresStorage::setup(&db).await?;
```

Sin esto el primer intento de push/pull de job falla con "table not found".

### 11. Migración incremental — Traefik routing dual (Fases 1–4)

Durante la migración Django y Rust corren en paralelo. Traefik enruta
por path prefix, con Rust tomando mayor prioridad:

```yaml
# Rust — alta prioridad, toma los endpoints ya migrados
- "traefik.http.routers.api-rust.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-rust.priority=10"
# Django — baja prioridad, solo recibe lo que Rust no maneja aún
- "traefik.http.routers.api-django.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-django.priority=5"
```

Esto permite mover endpoints uno a uno sin downtime.

### 12. Redis — `fred` v10, API diferente a `redis-rs`

El proyecto usa `fred` v10 (pool nativo async). Los pipelines usan
`client.pipeline()`. Para pub/sub (plane-live sync entre instancias)
usar `subscriber_client`. No mezclar con `deadpool-redis`.

### 13. Todas las fechas en UTC — `chrono::DateTime<Utc>`

SeaORM + Postgres almacena en UTC. Los seeds calculan fechas de ciclos
en UTC. El frontend convierte a timezone local en el cliente.

---

## Estructura completa de archivos — estado objetivo

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
│   ├── lib.rs
│   ├── auth/
│   │   ├── middleware.rs    ← CurrentUser extractor
│   │   └── permissions.rs  ← WorkspaceMemberGuard, ProjectMemberGuard
│   ├── entities/            ← 122 entidades generadas por sea-orm-cli (NO editar)
│   ├── repositories/        ← acceso a DB aislado, un archivo por dominio
│   │   ├── mod.rs
│   │   ├── issues.rs
│   │   ├── workspaces.rs
│   │   ├── projects.rs
│   │   ├── states.rs
│   │   └── ...
│   ├── routes/              ← handlers Axum, llaman a repositories
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
│   │   ├── github_sync.rs
│   │   ├── notifications.rs
│   │   ├── export.rs
│   │   └── scheduled.rs     ← tokio-cron-scheduler (reemplaza beatworker)
│   └── utils/
│       ├── mod.rs
│       └── soft_delete.rs   ← SoftDeleteExt trait + impl_soft_delete! macro ✅
├── migration/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            ← Migrator con todas las migraciones en orden
│       ├── main.rs
│       └── migrations/
│           ├── mod.rs
│           ├── m20260410_000001_baseline.rs  ← schema completo desde Django ✅
│           └── m20240101_000007_seed_data.rs ← integrations + instance_configs ✅
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

## Integraciones — GitHub, GitLab, Slack

### Visión general

Plane soporta 3 integraciones externas. Sus filas maestras viven en la tabla
`integrations` (3 filas estáticas, insertadas por `m007_seed_data`). La
instalación por workspace se registra en `workspace_integrations`.

| Integración | Provider key | Autenticación          | Función principal                         |
| ----------- | ------------ | ---------------------- | ----------------------------------------- |
| **GitHub**  | `github`     | GitHub App (JWT+token) | Sync bidireccional issues/PRs, webhooks   |
| **GitLab**  | `gitlab`     | OAuth 2.0 code flow    | Sync bidireccional issues/comentarios     |
| **Slack**   | `slack`      | OAuth 2.0 code flow    | Notificaciones de actividad en canales    |

---

### Modelo de datos — relaciones clave

```
integrations (3 filas estáticas)
    └─ workspace_integrations           (1 por workspace por proveedor)
            ├─ metadata: { installation_id }   (GitHub)
            ├─ config:   { installation_id }   (GitHub)
            ├─ actor_id → users.id             (admin que instaló)
            └─ api_token_id → api_tokens.id

github_repositories       (1 por repo conectado a un proyecto)
    └─ github_repository_syncs          (1 por proyecto+repo)
            ├─ credentials: { sync_direction, issue_open_state, issue_closed_state }
            ├─ workspace_integration_id
            ├─ actor_id
            └─ label_id (opcional)

github_issue_syncs         (1 por issue importado de GitHub)
    ├─ issue_id → issues.id
    └─ repository_sync_id

github_comment_syncs       (1 por comment importado)
    ├─ comment_id → issue_comments.id
    └─ issue_sync_id

db_githubprstatemapping    (mapeo PR state → Plane state)
    ├─ workspace_integration_id
    ├─ project_id
    ├─ state_id → states.id
    └─ github_pr_state: draft_open|open|review_requested|ready_for_merge|merged|closed

slack_project_syncs        (1 por proyecto con Slack conectado)
    ├─ access_token, bot_user_id, webhook_url
    ├─ team_id, team_name
    └─ workspace_integration_id

gitlab_repositories / gitlab_repository_syncs / gitlab_issue_syncs / gitlab_comment_syncs
    (mismo patrón que GitHub)

user_github_connections    (conexión OAuth personal por usuario)
    ├─ github_user_id, github_username, github_avatar_url
    └─ access_token (token OAuth personal del usuario)
```

---

### Endpoints a implementar en Rust

#### Globales (sin workspace)

```
GET  /api/github/callback/
     ← GithubAppCallbackEndpoint (sin auth — GitHub App Setup URL)
     ← Parámetros: installation_id, setup_action, state={workspace_slug}
     ← Devuelve HTML con postMessage al opener + cierra popup

POST /api/auth/github/user-callback/
     ← UserGithubConnectionView (auth requerida)
     ← Body: { code }
     ← Intercambia code por access_token OAuth personal del usuario
```

#### Por workspace (requieren WorkspaceAdmin)

```
GET    /api/integrations/
       ← Lista las 3 integraciones globales (github, gitlab, slack)

GET    /api/workspaces/{slug}/workspace-integrations/
POST   /api/workspaces/{slug}/workspace-integrations/
GET    /api/workspaces/{slug}/workspace-integrations/{pk}/
PATCH  /api/workspaces/{slug}/workspace-integrations/{pk}/
DELETE /api/workspaces/{slug}/workspace-integrations/{pk}/
DELETE /api/workspaces/{slug}/workspace-integrations/{provider}/provider/

POST   /api/workspaces/{slug}/workspace-integrations/{provider}/install/
       ← OAuth callback: body varía por proveedor:
           github: { installation_id }
           gitlab: { code }
           slack:  { code }  ← intercambia code por token Slack

GET    /api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/
       ← Lista repos accesibles por la GitHub App installation
       ← Paginado: ?page=1 (30 repos por página)

GET    /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
POST   /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
DELETE /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/

GET    /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
POST   /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
DELETE /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/
```

---

### Flujo OAuth — GitHub App (el más complejo)

```mermaid
sequenceDiagram
    actor User as 🧑 Admin workspace
    participant Frontend as Frontend (popup)
    participant GH as GitHub App
    participant Rust as Axum (Rust)
    participant DB as PostgreSQL

    User->>Frontend: Click "Connect GitHub"
    Frontend->>GH: Abrir popup → github.com/apps/{app}/installations/new?state={slug}
    GH->>User: Pedir autorización
    User->>GH: Aprobar
    GH->>Rust: GET /api/github/callback/?installation_id=XXX&state={slug}
    Note over Rust: Sin autenticación (GitHub redirige directamente)
    Rust->>DB: SELECT workspace WHERE slug = state
    Rust->>DB: SELECT workspace_members WHERE role >= 20 (admin)
    Rust->>DB: UPSERT workspace_integrations SET metadata={installation_id}
    Rust-->>Frontend: HTML con window.postMessage({type:"github-integration", success:true})
    Frontend->>Frontend: Popup se cierra, parent recibe el postMessage
    Frontend->>Rust: POST /api/workspaces/{slug}/workspace-integrations/github/install/
                     Body: { installation_id }
    Rust->>DB: GET OR CREATE WorkspaceIntegration
    Rust-->>Frontend: 201 { workspace_integration }
```

#### Generación del JWT para GitHub App

```rust
// src/utils/github_app.rs

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde_json::Value;

#[derive(serde::Serialize)]
struct AppClaims {
    iat: i64,
    exp: i64,
    iss: String,
}

/// Genera un installation access token usando la GitHub App private key.
/// Configuración leída desde instance_configurations:
///   - GITHUB_APP_ID        (numérico)
///   - GITHUB_APP_PRIVATE_KEY (PEM base64-encoded)
pub async fn get_installation_access_token(
    db: &DatabaseConnection,
    installation_id: &str,
) -> anyhow::Result<Option<String>> {
    let app_id = get_instance_config(db, "GITHUB_APP_ID").await?;
    let private_key_b64 = get_instance_config(db, "GITHUB_APP_PRIVATE_KEY").await?;

    let (Some(app_id), Some(key_b64)) = (app_id, private_key_b64) else {
        return Ok(None); // GitHub App no configurado
    };

    // Decodificar PEM desde base64
    let pem = general_purpose::STANDARD.decode(&key_b64)?;
    let encoding_key = EncodingKey::from_rsa_pem(&pem)?;

    // JWT firmado con RS256 — válido 10 minutos
    let now = chrono::Utc::now().timestamp();
    let claims = AppClaims { iat: now - 60, exp: now + 600, iss: app_id };
    let app_jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;

    // Intercambiar JWT por installation access token
    let resp = Client::new()
        .post(format!(
            "https://api.github.com/app/installations/{installation_id}/access_tokens"
        ))
        .header("Authorization", format!("Bearer {app_jwt}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send().await?;

    if resp.status().is_success() {
        let body: Value = resp.json().await?;
        Ok(body["token"].as_str().map(String::from))
    } else {
        Ok(None)
    }
}
```

**Dependencias Cargo.toml necesarias:**

```toml
jsonwebtoken = { version = "9", features = [] }
base64 = "0.22"
```

---

### Flujo OAuth — Slack

```mermaid
sequenceDiagram
    actor User as 🧑 Admin workspace
    participant Frontend
    participant Slack as Slack OAuth
    participant Rust as Axum

    User->>Frontend: Click "Connect Slack"
    Frontend->>Slack: Abrir popup → slack.com/oauth/v2/authorize?client_id=...
    Slack->>User: Autorizar
    Slack-->>Frontend: Redirect con ?code=XXX
    Frontend->>Rust: POST /api/workspaces/{slug}/workspace-integrations/slack/install/
                     Body: { code }
    Rust->>Slack: POST https://slack.com/api/oauth.v2.access
                  { client_id, client_secret, code }
    Slack-->>Rust: { access_token, team: { id, name }, ... }
    Rust->>DB: UPSERT workspace_integrations
               metadata = slack_response
               config = { access_token, team_id, team_name }
    Rust-->>Frontend: 201 { workspace_integration }
```

**Variables de instancia requeridas (instance_configurations):**
- `SLACK_CLIENT_ID`
- `SLACK_CLIENT_SECRET`

---

### Job apalis — GithubInitialIssueSyncJob

Al crear un `GithubRepositorySync`, se dispara un job apalis para
importar en bulk los issues existentes del repo en GitHub:

```rust
// src/jobs/github_sync.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubInitialIssueSyncJob {
    pub repo_sync_id: Uuid,
}

pub async fn handle_github_initial_sync(
    job: GithubInitialIssueSyncJob,
    ctx: Data<DatabaseConnection>,
) -> Result<(), apalis::prelude::Error> {
    let db = ctx.as_ref();

    let sync = github_repository_syncs::Entity::find_by_id(job.repo_sync_id)
        .find_also_related(github_repositories::Entity)
        .one(db).await
        .map_err(|e| apalis::prelude::Error::Failed(e.to_string().into()))?;

    let Some((sync, Some(repo))) = sync else {
        tracing::warn!("GithubInitialIssueSyncJob: sync {} not found", job.repo_sync_id);
        return Ok(());
    };

    // Obtener installation_id desde workspace_integration
    let wi = workspace_integrations::Entity::find_by_id(sync.workspace_integration_id)
        .one(db).await?
        .ok_or_else(|| apalis::prelude::Error::Failed("workspace_integration not found".into()))?;

    let installation_id = wi.metadata["installation_id"]
        .as_str()
        .ok_or_else(|| apalis::prelude::Error::Failed("no installation_id".into()))?;

    let token = get_installation_access_token(db, installation_id).await
        .map_err(|e| apalis::prelude::Error::Failed(e.to_string().into()))?
        .ok_or_else(|| apalis::prelude::Error::Failed("could not obtain GitHub token".into()))?;

    // Paginar GET /repos/{owner}/{repo}/issues?state=all&per_page=100
    import_github_issues(db, &sync, &repo, &token).await
        .map_err(|e| apalis::prelude::Error::Failed(e.to_string().into()))
}

async fn import_github_issues(
    db: &DatabaseConnection,
    sync: &github_repository_syncs::Model,
    repo: &github_repositories::Model,
    token: &str,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let credentials = sync.credentials.as_object().cloned().unwrap_or_default();
    let open_state_id = credentials.get("issue_open_state")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let closed_state_id = credentials.get("issue_closed_state")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let mut page = 1u32;
    loop {
        let resp: Vec<serde_json::Value> = client
            .get(format!(
                "https://api.github.com/repos/{}/{}/issues",
                repo.owner, repo.name
            ))
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .query(&[("state", "all"), ("per_page", "100"), ("page", &page.to_string())])
            .send().await?
            .json().await?;

        if resp.is_empty() { break; }

        for gh_issue in &resp {
            // Skip PRs (GitHub API devuelve PRs en /issues si tienen pull_request key)
            if gh_issue.get("pull_request").is_some() { continue; }

            let gh_issue_id = gh_issue["id"].as_i64().unwrap_or(0);
            let gh_issue_number = gh_issue["number"].as_i64().unwrap_or(0);

            // Verificar idempotencia: no reimportar si ya existe GithubIssueSync
            let exists = github_issue_syncs::Entity::find()
                .filter(github_issue_syncs::Column::GithubIssueId.eq(gh_issue_id))
                .filter(github_issue_syncs::Column::RepositorySyncId.eq(sync.id))
                .count(db).await? > 0;
            if exists { continue; }

            // Determinar state de Plane según estado del issue en GitHub
            let gh_state = gh_issue["state"].as_str().unwrap_or("open");
            let plane_state_id = if gh_state == "closed" {
                closed_state_id
            } else {
                open_state_id
            };

            // Crear Issue en Plane
            let issue = issues::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(gh_issue["title"].as_str().unwrap_or("").to_string()),
                description_html: Set(Some(format!(
                    "<p>{}</p>",
                    gh_issue["body"].as_str().unwrap_or("")
                ))),
                state_id: Set(plane_state_id),
                project_id: Set(sync.project_id),
                workspace_id: Set(sync.workspace_id),
                // …otros campos
                ..Default::default()
            }.insert(db).await?;

            // Crear GithubIssueSync
            github_issue_syncs::ActiveModel {
                id: Set(Uuid::new_v4()),
                repo_issue_id: Set(gh_issue_number),
                github_issue_id: Set(gh_issue_id),
                issue_url: Set(gh_issue["html_url"].as_str().unwrap_or("").to_string()),
                issue_id: Set(issue.id),
                repository_sync_id: Set(sync.id),
                project_id: Set(sync.project_id),
                workspace_id: Set(sync.workspace_id),
                ..Default::default()
            }.insert(db).await?;
        }

        page += 1;
    }
    Ok(())
}
```

**Disparar el job desde el handler de creación de repo sync:**

```rust
// En routes/integrations.rs — POST github/repo-syncs/
state.job_storage
    .push(GithubInitialIssueSyncJob { repo_sync_id: new_sync.id })
    .await
    .ok(); // best-effort, no bloquear la respuesta
```

---

### Helper — HTML postMessage para callbacks OAuth

Reutilizable para GitHub, GitLab, Slack — devuelve una página HTML
mínima que cierra el popup y notifica al parent:

```rust
// src/utils/oauth_popup.rs

use axum::response::Html;

pub fn postmessage_html(success: bool, message_type: &str, error: Option<&str>) -> Html<String> {
    let error_json = error
        .map(|e| format!(r#", "error": "{}""#, e.replace('"', "\\\"")))
        .unwrap_or_default();
    let success_str = if success { "true" } else { "false" };

    Html(format!(r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Connecting…</title></head>
<body>
<script>
(function(){{
  try{{
    window.opener && window.opener.postMessage(
      {{"type":"{message_type}","success":{success_str}{error_json}}},
      window.location.origin
    );
  }}catch(e){{}}
  window.close();
}})();
</script>
<p style="font-family:sans-serif;text-align:center;margin-top:4rem;">
  {}
</p>
</body>
</html>"#,
        if success {
            "Integration connected. You may close this window."
        } else {
            "An error occurred. You may close this window."
        }
    ))
}
```

---

### Registro del webhook en GitHub

Al crear un `GithubRepositorySync`, Rust debe registrar el webhook
en GitHub para recibir eventos en tiempo real:

```rust
// src/utils/github_webhooks.rs

pub async fn register_github_webhook(
    db: &DatabaseConnection,
    workspace_integration: &workspace_integrations::Model,
    owner: &str,
    repo_name: &str,
    web_url: &str,
    webhook_secret: &str,
) -> anyhow::Result<()> {
    let installation_id = workspace_integration.metadata["installation_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no installation_id"))?;

    let token = get_installation_access_token(db, installation_id).await?
        .ok_or_else(|| anyhow::anyhow!("could not obtain token"))?;

    let client = reqwest::Client::new();
    client
        .post(format!("https://api.github.com/repos/{owner}/{repo_name}/hooks"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&serde_json::json!({
            "name": "web",
            "config": {
                "url": format!("{web_url}/api/github-webhook/"),
                "content_type": "json",
                "secret": webhook_secret,
            },
            "events": ["issues", "pull_request", "issue_comment"],
            "active": true,
        }))
        .send().await?;
    // Fallo silencioso — no bloquear creación del sync
    Ok(())
}
```

---

### Variables de entorno por integración

Leídas desde `instance_configurations` (vía helper `get_instance_config`)
con fallback a variables de entorno. Se configuran en "God Mode" (admin panel).

| Variable                  | Integración | Uso                                        |
| ------------------------- | ----------- | ------------------------------------------ |
| `GITHUB_APP_ID`           | GitHub      | ID de la GitHub App para firmar JWT        |
| `GITHUB_APP_PRIVATE_KEY`  | GitHub      | PEM base64 para firmar JWT RS256           |
| `GITHUB_CLIENT_ID`        | GitHub      | OAuth personal connection (user-callback)  |
| `GITHUB_CLIENT_SECRET`    | GitHub      | OAuth personal connection (user-callback)  |
| `GITHUB_WEBHOOK_SECRET`   | GitHub      | Verificar firma HMAC de webhooks entrantes |
| `SLACK_CLIENT_ID`         | Slack       | OAuth app install flow                     |
| `SLACK_CLIENT_SECRET`     | Slack       | Intercambiar code por access_token         |

```rust
// src/utils/instance_config.rs

pub async fn get_instance_config(
    db: &DatabaseConnection,
    key: &str,
) -> anyhow::Result<Option<String>> {
    // 1. Buscar en instance_configurations (prioridad DB sobre env)
    if let Some(row) = instance_configurations::Entity::find()
        .filter(instance_configurations::Column::Key.eq(key))
        .one(db).await?
    {
        if !row.value.is_empty() {
            return Ok(Some(row.value));
        }
    }
    // 2. Fallback a variable de entorno
    Ok(std::env::var(key).ok())
}
```

---

### Cosas críticas — Integraciones

#### 1. `GET /api/github/callback/` es **sin autenticación**

GitHub App redirige el popup directamente a esta URL. El handler no puede
usar `CurrentUser` extractor. Debe tener `authentication_classes = []`.
En Rust: no agregar `CurrentUser` a la firma del handler, usar un
`Router` separado sin middleware de auth para esta ruta.

#### 2. GithubAppCallbackEndpoint busca el primer admin del workspace

El callback es unauthenticated pero `workspace_integrations.actor_id` es
NOT NULL. Se busca el primer `WorkspaceMember` con `role >= 20`. Si no
existe → error. **No usar el usuario actual** (no hay uno).

#### 3. Soft-delete en `GithubRepository` y `GithubRepositorySync`

Al crear un repo sync, primero buscar con `all_objects` (incluyendo
`deleted_at IS NOT NULL`) para no violar el constraint unique de Postgres.
Si existe uno soft-deleted → resucitar con `deleted_at = NULL`.
En Rust: **no usar `.active()`** en estas queries de creación — usar
filtro explícito sin `.active()` para incluir soft-deleted.

#### 4. GitHub devuelve PRs en el endpoint de issues

`GET /repos/{owner}/{repo}/issues` incluye Pull Requests. Filtrar
los que tengan `"pull_request"` key en el JSON antes de crear Issues en Plane.

#### 5. PR State Mapping — enum Postgres

`github_pr_state` es un enum con valores exactos:
`draft_open`, `open`, `review_requested`, `ready_for_merge`, `merged`, `closed`.
En SeaORM se puede modelar como `String` o como enum Rust derivado.
**Constraint unique:** `(workspace_integration_id, project_id, github_pr_state)`.

#### 6. Instalación Slack — intercambio de code en el backend

A diferencia de GitHub (que llama al callback del servidor directamente),
Slack envía el `code` al **frontend** que lo reenvía al backend via
`POST /install/`. El backend hace el intercambio con `slack.com/api/oauth.v2.access`.
Esto significa que `SLACK_CLIENT_ID` y `SLACK_CLIENT_SECRET` son requeridos
en el backend Rust, no en el frontend.

#### 7. Paginación de repos GitHub — dos endpoints distintos

- **GitHub App token** → `GET /installation/repositories` (devuelve `{ total_count, repositories: [...] }`)
- **Personal OAuth token** → `GET /user/repos` (devuelve array plano)

La lógica de selección de endpoint es: si `workspace_integration.metadata.installation_id`
existe → usar endpoint de installation. Rust debe manejar ambos casos en el mismo handler.

#### 8. Webhook registration — best-effort, never block

El registro del webhook en GitHub al crear un repo sync puede fallar
(credenciales no configuradas, GitHub caído). **Nunca** hacer que esto
falle la creación del sync. Usar `let _ = register_github_webhook(...).await;`
o `if let Err(e) = ... { tracing::warn!(...) }`.

#### 9. `user_github_connections` — token personal del usuario, no del workspace

Es una conexión individual (tabla `user_github_connections`) distinta al
`workspace_integrations`. Se crea/actualiza via `POST /auth/github/user-callback/`
con el OAuth personal del usuario (no la GitHub App). Requiere
`GITHUB_CLIENT_ID` y `GITHUB_CLIENT_SECRET` (OAuth App, no GitHub App).

---

### Entidades Rust ya generadas ✅

Todas las entidades de integraciones están generadas en `src/entities/`:

| Entidad Rust                           | Tabla DB                        |
| -------------------------------------- | ------------------------------- |
| `integrations.rs`                      | `integrations`                  |
| `workspace_integrations.rs`            | `workspace_integrations`        |
| `github_repositories.rs`              | `github_repositories`           |
| `github_repository_syncs.rs`          | `github_repository_syncs`       |
| `github_issue_syncs.rs`               | `github_issue_syncs`            |
| `github_comment_syncs.rs`             | `github_comment_syncs`          |
| `db_githubprstatemapping.rs`          | `db_githubprstatemapping`       |
| `user_github_connections.rs`          | `user_github_connections`       |
| `gitlab_repositories.rs`             | `gitlab_repositories`           |
| `gitlab_repository_syncs.rs`         | `gitlab_repository_syncs`       |
| `gitlab_issue_syncs.rs`              | `gitlab_issue_syncs`            |
| `gitlab_comment_syncs.rs`            | `gitlab_comment_syncs`          |
| `slack_project_syncs.rs`             | `slack_project_syncs`           |

---

### Plan de implementación — integraciones en Rust (Fase 2 extendida)

```
[ ] src/utils/github_app.rs         — get_installation_access_token (JWT RS256)
[ ] src/utils/instance_config.rs    — get_instance_config (DB + env fallback)
[ ] src/utils/oauth_popup.rs        — postmessage_html helper
[ ] src/utils/github_webhooks.rs    — register_github_webhook
[ ] src/routes/integrations.rs      — todos los endpoints listados arriba
[ ] src/jobs/github_sync.rs         — GithubInitialIssueSyncJob (apalis)
[ ] Cargo.toml: agregar jsonwebtoken = "9", base64 = "0.22"
[ ] Router: ruta /api/github/callback/ sin middleware de auth
```

Prioridad de implementación:

1. `GET /api/integrations/` — sin lógica compleja, desbloquea el panel de integraciones en el frontend
2. `GET/POST/DELETE /api/workspaces/{slug}/workspace-integrations/` — CRUD básico
3. `GET /api/github/callback/` + `POST /install/` — flujo de instalación completo
4. `GET /github-repositories/` — lista de repos (requiere GitHub App JWT)
5. `POST/DELETE github/repo-syncs/` — crear sync + disparar job de importación
6. `GET/POST/DELETE pr-state-mappings/` — CRUD de mappings PR state
7. `POST /auth/github/user-callback/` — conexión personal GitHub
