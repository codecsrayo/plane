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

**Por qué SeaORM y no Diesel:**

| | SeaORM | Diesel + diesel-async |
|---|---|---|
| Async nativo Tokio | ✅ | ⚠️ wrapper sobre sync |
| Migrations en Rust | ✅ sea-orm-migration | ✅ diesel_migrations! |
| Generar entities desde DB existente | ✅ `sea-orm-cli generate entity` | ⚠️ solo structs básicos |
| Relaciones FK / M2M | ✅ has_many, belongs_to, many_to_many | ⚠️ M2M manual |
| Soft delete integrado | ✅ con ActiveModel hooks | ⚠️ manual |
| Con Axum | ✅ natural | ✅ funciona |

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
└── Dockerfile
```

---

## Soft delete — consideración importante

Los modelos de Django usan `SoftDeletionManager` que filtra `deleted_at IS NULL`
automáticamente. En SeaORM no hay manager mágico.

**Solución: `SoftDeleteActiveModel` trait custom:**

```rust
// src/entities/traits.rs
pub trait SoftDelete {
    fn is_deleted(&self) -> bool;
}

// En cada query, agregar condición explícita:
Issue::find()
    .filter(issue::Column::DeletedAt.is_null())
    .filter(issue::Column::ProjectId.eq(project_id))
    .all(&db)
    .await?
```

Alternativamente, usar `sea-orm-softdelete` crate que añade el filtro automático
similar al manager de Django.

---

## Estrategia de migración — por fases

### Fase 0 — Scaffolding + Baseline (2–3 días)

- [ ] `cargo new plane-api && cargo new migration`
- [ ] `Cargo.toml` con SeaORM, Axum, Tokio, apalis
- [ ] Generar baseline SQL desde la DB actual de Django
- [ ] Crear migración `m_baseline_from_django` con ese SQL
- [ ] Generar entities con `sea-orm-cli generate entity`
- [ ] `GET /api/health/` funcionando contra la DB
- [ ] Dockerfile multi-stage
- [ ] Traefik: routing condicional por path

### Fase 1 — Auth middleware

- [ ] Leer tabla `authtoken_token` → `CurrentUser` extractor Axum
- [ ] Role check (workspace_member, project_member) via SeaORM
- [ ] Tests de integración contra DB real

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
