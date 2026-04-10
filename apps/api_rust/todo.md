# Plane — API Rust: Plan de migración

## Objetivo

Reemplazar progresivamente la API Django + Celery por un stack Rust que comparte
la misma base de datos PostgreSQL. No es un rewrite big-bang — se migra endpoint
por endpoint detrás del proxy Traefik existente.

**Metas concretas:**
- Reducir uso de RAM: ~750 MB (Django + Celery + RabbitMQ) → ~20 MB
- Eliminar los workers de Celery (bgworker + beatworker) y RabbitMQ
- Mantener 100% de compatibilidad de API (mismas URLs, mismos payloads)
- Mismo PostgreSQL, mismo Redis/Valkey

---

## plane-live: NO se toca

`plane-live` es un servidor **Hocuspocus (Y.js CRDT)** para edición colaborativa
en tiempo real de Pages e issue descriptions. No es un WebSocket genérico — implementa
el protocolo de sincronización CRDT de Y.js, con:

- `@hocuspocus/extension-redis` → pub/sub entre múltiples instancias del servidor
- `@hocuspocus/extension-database` → persiste estado binario Y.js + HTML + JSON
  de vuelta a la API vía HTTP
- `TitleSyncExtension` → sincroniza el título del documento cuando cambia en el editor
- `ForceCloseHandler` → expulsa conexiones cuando el documento está bloqueado/archivado

**Por qué no se reemplaza con Axum WebSockets:**
Hocuspocus es un protocolo específico (Y.js awareness + sync messages). Existe `y-crdt`
(port Rust de Y.js) pero construir un servidor Hocuspocus-compatible desde cero en Rust
sería semanas de trabajo para cero beneficio — plane-live es **Node.js** (~50–80 MB RAM),
no Python. No es el problema.

**Conclusión:** `plane-live` se mantiene intacto. Solo se migra Django → Rust.

---

## Migraciones de base de datos

**Django sigue siendo el dueño del schema.** Rust nunca hace DDL (CREATE TABLE,
ALTER TABLE, etc.).

### En desarrollo / CI

SQLx verifica las queries en tiempo de compilación contra la DB real. Para eso necesita
`DATABASE_URL` al hacer `cargo build`:

```bash
# 1. Levantar la DB (la misma que usa Django)
docker compose up -d plane-db

# 2. Aplicar migraciones de Django (única fuente de verdad del schema)
docker compose run --rm api python manage.py migrate

# 3. Generar el cache de SQLx (archivos en .sqlx/) para compilación offline
cd apps/api_rust
cargo sqlx prepare

# 4. Commit del directorio .sqlx/ al repo
git add .sqlx/
```

### En producción (CI/CD)

El binario Rust se compila con `SQLX_OFFLINE=true` — usa los archivos `.sqlx/`
cacheados, no necesita acceso a la DB en build time:

```bash
SQLX_OFFLINE=true cargo build --release
```

El pipeline de deploy es:
1. `plane-migrator` (Django) aplica migraciones → schema actualizado
2. Contenedor Rust arranca con el schema ya listo

### Flujo cuando se agrega un campo nuevo

```
1. Crear migración Django normalmente (apps/api/plane/db/migrations/)
2. Aplicar en la DB de dev: python manage.py migrate
3. Actualizar el struct Rust correspondiente en src/models/
4. Actualizar la query SQLx afectada en src/routes/ o src/jobs/
5. Regenerar cache: cargo sqlx prepare
6. Commit de ambos (migración Django + .sqlx/ actualizado)
```

**Nunca se usa `sqlx migrate`** — ese comando es para proyectos donde Rust es el
dueño del schema. Aquí Django lo es.

---

## Stack elegido

| Rol | Librería | Reemplaza |
|---|---|---|
| HTTP framework | **Axum** (tokio-rs) | Django REST Framework + uvicorn |
| Async runtime | **Tokio** | — |
| Query builder | **SQLx** (offline mode) | Django ORM |
| Serialización | **serde + serde_json** | DRF serializers |
| Auth JWT | **jsonwebtoken** | DRF TokenAuthentication |
| Background jobs | **apalis** (backend: Postgres) | Celery bgworker + RabbitMQ |
| Cron jobs | **tokio-cron-scheduler** | Celery beatworker |
| Redis | **fred** | django-redis |
| S3 / MinIO | **aws-sdk-s3** | boto3 + django-storages |
| Email | **lettre** | Django email backend |
| HTTP client | **reqwest** | requests |
| Logging | **tracing + tracing-subscriber** | python-json-logger |
| Config | **dotenvy** | django settings / .env |
| Métricas | **axum-prometheus** | scout-apm |

### Lo que se elimina

| Actual | RAM | Resultado |
|---|---|---|
| api (Django + uvicorn) | ~280 MB | → Rust ~20 MB |
| bgworker (Celery) | ~200 MB | → eliminado (apalis en el mismo binario) |
| beatworker (Celery beat) | ~150 MB | → eliminado (tokio-cron-scheduler) |
| plane-mq (RabbitMQ) | ~120 MB | → eliminado (apalis usa Postgres) |
| **Total** | **~750 MB** | **→ ~20 MB** |

### Lo que se mantiene intacto

| Servicio | Por qué |
|---|---|
| plane-live (Hocuspocus/Node.js) | protocolo Y.js CRDT — no reemplazable trivialmente |
| plane-db (PostgreSQL) | misma DB compartida |
| plane-redis (Valkey) | sigue siendo necesario para plane-live y caché |
| plane-minio (MinIO) | almacenamiento de archivos, sin cambio |
| proxy (Traefik) | el mismo, solo se agrega routing al contenedor Rust |

---

## Estructura del proyecto

```
apps/api_rust/
├── Cargo.toml
├── Cargo.lock
├── .sqlx/                       ← cache SQLx para compilación offline (commit al repo)
├── .env.example
├── src/
│   ├── main.rs                  ← bootstrap: router + AppState + servidor HTTP
│   ├── config.rs                ← env vars tipadas (mismas que Django)
│   ├── db.rs                    ← PgPool setup
│   ├── error.rs                 ← AppError → HTTP response (equivale a DRF exception handler)
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── middleware.rs        ← extractor de token → CurrentUser (lee authtoken_token)
│   │   └── permissions.rs      ← workspace/project role checks
│   ├── models/                  ← structs mapeando tablas existentes (SQLx FromRow)
│   │   ├── mod.rs
│   │   ├── issue.rs
│   │   ├── project.rs
│   │   ├── workspace.rs
│   │   ├── state.rs
│   │   └── ...
│   ├── routes/                  ← un archivo por dominio, registrados en main.rs
│   │   ├── mod.rs
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   ├── cycles.rs
│   │   ├── modules.rs
│   │   └── integrations.rs
│   └── jobs/                    ← apalis workers + cron (reemplazan Celery)
│       ├── mod.rs
│       ├── github_sync.rs
│       ├── notifications.rs
│       ├── export.rs
│       └── scheduled.rs        ← cleanup, exporter_expired, etc.
└── Dockerfile                   ← multi-stage: builder (600 MB) → runner (~15 MB)
```

---

## Estrategia de migración — por fases

### Fase 0 — Scaffolding (1–2 días)

- [ ] `cargo init --name plane-api`
- [ ] `Cargo.toml` con dependencias base
- [ ] `config.rs` con las mismas env vars que Django
- [ ] Conexión a PostgreSQL con `sqlx::PgPool`
- [ ] `GET /api/health/` → `{"status": "ok"}`
- [ ] Generar `.sqlx/` con `cargo sqlx prepare` y commitear
- [ ] Dockerfile multi-stage
- [ ] Agregar contenedor `api_rust` en `docker-compose.yml`
- [ ] Traefik: priority routing — Rust maneja solo paths migrados, Django el resto

### Fase 1 — Auth middleware (prerequisito de todo)

- [ ] Leer tabla `authtoken_token` → `CurrentUser` extractor de Axum
- [ ] Middleware de role check (workspace_member, project_member)
- [ ] Tests de integración contra DB real

### Fase 2 — Endpoints de alta frecuencia

- [ ] `GET  /api/workspaces/{slug}/projects/`
- [ ] `GET  /api/workspaces/{slug}/projects/{id}/issues/`
- [ ] `GET  /api/workspaces/{slug}/projects/{id}/issues/{id}/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/issues/`
- [ ] `PATCH /api/workspaces/{slug}/projects/{id}/issues/{id}/`
- [ ] `GET  /api/workspaces/{slug}/projects/{id}/states/`
- [ ] `GET  /api/workspaces/{slug}/projects/{id}/members/`
- [ ] `GET  /api/workspaces/{slug}/projects/{id}/cycles/`
- [ ] `GET  /api/workspaces/{slug}/projects/{id}/modules/`

### Fase 3 — Background jobs (eliminar Celery + RabbitMQ)

- [ ] Setup apalis con backend PostgreSQL (tabla `apalis.jobs`)
- [ ] Migrar `github_initial_issue_sync_task`
- [ ] Migrar `sync_issue_to_github_task` / `sync_comment_to_github_task`
- [ ] Migrar `notification_task`
- [ ] Migrar `export_task`
- [ ] Migrar `issue_activities_task`
- [ ] Migrar tasks de email (magic link, invitaciones, activaciones)
- [ ] Setup tokio-cron-scheduler para tareas periódicas:
  - `cleanup_task` (daily)
  - `exporter_expired_task` (hourly)
- [ ] Eliminar `bgworker`, `beatworker`, `plane-mq` del docker-compose

### Fase 4 — Endpoints restantes (~275 paths)

Cubrir el resto priorizando por frecuencia de uso en logs de producción.

### Fase 5 — Shutdown Django

- [ ] 0 tráfico va al contenedor `api` en producción
- [ ] Eliminar contenedor `api` del docker-compose
- [ ] Mantener migraciones Django en un contenedor de utilidad (`plane-migrator`)
  para futuros schema changes — solo corre `manage.py migrate`, no levanta servidor

---

## Decisiones de diseño

### SQLx sin ORM — por qué

Django ORM genera N+1 queries y abstrae demasiado. Con SQLx las queries son SQL
plano verificado en compile time. Más verboso, más predecible, más rápido.

```rust
// Ejemplo: listar issues con estado, sin N+1
let issues = sqlx::query_as!(
    IssueRow,
    r#"
    SELECT
        i.id, i.name, i.description_html, i.priority, i.created_at,
        s.id AS state_id, s.name AS state_name, s.color AS state_color
    FROM issues i
    JOIN states s ON s.id = i.state_id
    WHERE i.project_id = $1
      AND i.deleted_at IS NULL
    ORDER BY i.created_at DESC
    LIMIT $2 OFFSET $3
    "#,
    project_id, limit, offset,
)
.fetch_all(&pool)
.await?;
```

### Soft delete — obligatorio en cada query

Todos los modelos tienen `deleted_at`. En Django lo maneja el `SoftDeletionManager`.
En Rust no hay manager mágico — cada query debe incluir `AND deleted_at IS NULL`
o tendrá datos erróneos.

```rust
// ⚠️ SIEMPRE incluir deleted_at IS NULL
WHERE project_id = $1 AND deleted_at IS NULL
```

### Auth durante la transición (Fases 0–4)

Rust lee la tabla `authtoken_token` de Django directamente — no reimplementa login.
Django sigue siendo el auth server. En Fase 5 (shutdown Django) se decide si se
migra el auth también o se mantiene como servicio separado.

### apalis — por qué Postgres en vez de RabbitMQ

apalis persiste los jobs en una tabla `apalis.jobs`. Sin broker externo.
Ventajas:
- Un servicio menos en docker-compose (~120 MB de RabbitMQ eliminados)
- Los jobs sobreviven reinicios del worker
- Visibilidad directa: `SELECT * FROM apalis.jobs WHERE status = 'Failed'`
- Reintentos configurables con backoff
- Dashboard opcional via apalis-web

---

## Próximo paso inmediato — Fase 0

```bash
cd apps/api_rust
cargo init --name plane-api
# Luego agregar Cargo.toml con dependencias base
```
