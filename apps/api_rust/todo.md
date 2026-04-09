# Plane — API Rust: Plan de migración

## Objetivo

Reemplazar progresivamente la API Django + Celery por un stack Rust que comparte
la misma base de datos PostgreSQL. No es un rewrite big-bang — se migra endpoint
por endpoint detrás del proxy Traefik existente.

**Metas concretas:**
- Reducir uso de RAM por contenedor de ~300 MB (Django + uvicorn) a ~15–30 MB
- Eliminar los workers de Celery (bgworker + beatworker + RabbitMQ)
- Mantener 100% de compatibilidad de API (mismas URLs, mismos payloads)
- Mismo PostgreSQL, mismo Redis/Valkey

---

## Stack elegido

| Rol | Librería | Por qué |
|---|---|---|
| HTTP framework | **Axum** (tokio-rs) | Ergonómico, async nativo, ecosistema maduro, no macro-heavy |
| Async runtime | **Tokio** | Estándar de facto en Rust async |
| ORM / query builder | **SQLx** | Queries compiladas en tiempo de build contra la DB real — sin ORM mágico |
| Serialización | **serde + serde_json** | Estándar |
| Autenticación JWT | **jsonwebtoken** | Simple, sin dependencias gordas |
| Background jobs | **apalis** | Reemplaza Celery — jobs persistidos en PostgreSQL, sin RabbitMQ |
| Cron jobs | **tokio-cron-scheduler** | Reemplaza Celery beat |
| WebSockets | **axum::extract::ws** | Built-in en Axum, reemplaza Django Channels |
| Redis | **fred** | Cliente Redis async, reemplaza django-redis |
| S3 / MinIO | **aws-sdk-s3** | Reemplaza boto3 / django-storages |
| Email | **lettre** | Reemplaza Django email backend |
| HTTP client (webhooks/OAuth) | **reqwest** | Reemplaza requests |
| Logging | **tracing + tracing-subscriber** | JSON structured logs |
| Config / env | **dotenvy** | Reemplaza django settings |
| Métricas | **axum-prometheus** | Prometheus built-in |

### Lo que se elimina del stack actual

| Actual | Reemplazado por |
|---|---|
| Celery bgworker | apalis (jobs en Postgres) |
| Celery beatworker | tokio-cron-scheduler |
| RabbitMQ (plane-mq) | eliminado — apalis usa Postgres directamente |
| uvicorn | eliminado — Axum tiene su propio servidor HTTP |
| Django Channels (plane-live) | axum::extract::ws + Redis pub/sub |
| Django ORM + migrations | SQLx (queries) — migraciones siguen siendo de Django |

> Las migraciones Django se conservan porque la DB es compartida. Rust solo lee/escribe,
> nunca hace ALTER TABLE.

---

## Estructura del proyecto

```
apps/api_rust/
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── src/
│   ├── main.rs                  ← bootstrap: router + DB pool + estado global
│   ├── config.rs                ← variables de entorno tipadas
│   ├── db.rs                    ← PgPool setup
│   ├── error.rs                 ← AppError → HTTP response mapping
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── middleware.rs        ← extractor JWT → CurrentUser
│   │   └── session.rs          ← lectura de tabla authtoken_token de Django
│   ├── models/                  ← structs que mapean tablas existentes (SQLx FromRow)
│   │   ├── mod.rs
│   │   ├── workspace.rs
│   │   ├── project.rs
│   │   ├── issue.rs
│   │   └── ...
│   ├── routes/                  ← un archivo por dominio funcional
│   │   ├── mod.rs
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   └── integrations.rs
│   ├── jobs/                    ← background tasks (reemplazan Celery tasks)
│   │   ├── mod.rs
│   │   ├── github_sync.rs       ← github_initial_issue_sync_task
│   │   ├── notifications.rs
│   │   └── export.rs
│   └── ws/                      ← WebSocket handlers (reemplazan Django Channels)
│       └── mod.rs
└── Dockerfile
```

---

## Estrategia de migración — por fases

### Fase 0 — Scaffolding (1–2 días)

- [ ] Inicializar `Cargo.toml` con dependencias base
- [ ] `config.rs` leyendo las mismas env vars que Django
- [ ] Conexión a PostgreSQL con `sqlx::PgPool`
- [ ] Health endpoint `GET /api/health/` → `{"status": "ok"}`
- [ ] Dockerfile multi-stage (build ~600 MB → imagen final ~15 MB)
- [ ] Integrar contenedor en `docker-compose.yml` apuntando al mismo Postgres
- [ ] Traefik: priority routing — Rust maneja paths migrados, Django el resto

### Fase 1 — Auth middleware (prerequisito de todo lo demás)

- [ ] Leer tabla `authtoken_token` de Django → `CurrentUser` extractor
- [ ] Middleware de workspace/project permission
- [ ] Tests de integración contra DB real

### Fase 2 — Endpoints de alta frecuencia (mayor impacto de performance)

- [ ] `GET /api/workspaces/{slug}/projects/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/issues/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/issues/{id}/`
- [ ] `PATCH /api/workspaces/{slug}/projects/{id}/issues/{id}/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/states/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/members/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/modules/`

### Fase 3 — Background jobs (eliminar Celery + RabbitMQ)

- [ ] Setup de apalis con backend PostgreSQL
- [ ] Migrar `github_initial_issue_sync_task`
- [ ] Migrar `sync_issue_to_github_task`
- [ ] Migrar `notification_task`
- [ ] Migrar `export_task`
- [ ] Migrar `issue_activities_task`
- [ ] Migrar tasks de email (magic link, invitaciones)
- [ ] Reemplazar Celery beat con tokio-cron-scheduler (cleanup, exporter_expired)
- [ ] Eliminar `bgworker`, `beatworker`, `plane-mq` del docker-compose

### Fase 4 — WebSockets (eliminar Django Channels)

- [ ] Endpoint WS con axum
- [ ] Pub/sub via Redis (mismo Valkey) para broadcast de cambios en tiempo real
- [ ] Eliminar o reemplazar `plane-live`

### Fase 5 — Endpoints restantes (~275 paths)

Cubrir el resto priorizando por frecuencia de uso según logs.

### Fase 6 — Shutdown Django

- [ ] 0 tráfico va a Django en producción
- [ ] Eliminar contenedor `api` del docker-compose
- [ ] Mantener migraciones Django en contenedor de utilidad solo para schema changes

---

## Decisiones de diseño

### SQLx sin ORM

Django ORM genera N+1 queries y abstrae demasiado. Con SQLx las queries son SQL
plano verificado en compile time contra el schema real.

```rust
let issues = sqlx::query_as!(
    Issue,
    r#"
    SELECT id, name, description_html, state_id, project_id, created_at
    FROM issues
    WHERE project_id = $1 AND deleted_at IS NULL
    ORDER BY created_at DESC
    LIMIT $2 OFFSET $3
    "#,
    project_id, limit, offset,
)
.fetch_all(&pool)
.await?;
```

### Soft delete

Todos los modelos usan `deleted_at IS NULL`. Cada query debe incluirlo explícitamente
— no hay manager mágico como en Django.

### Sesiones Django compartidas durante la transición

Rust lee los tokens de la tabla `authtoken_token` de Django. No hay que reimplementar
login — Django sigue siendo el auth server hasta Fase 6.

### apalis vs RabbitMQ

apalis persiste jobs en una tabla PostgreSQL. Sin broker externo. Los jobs sobreviven
reinicios y son visibles directamente en la DB — un servicio menos que operar.

---

## Estimación de recursos post-migración completa

| Contenedor | RAM actual | RAM Rust |
|---|---|---|
| api (Django + uvicorn) | ~280 MB | ~20 MB |
| bgworker (Celery) | ~200 MB | eliminado |
| beatworker (Celery beat) | ~150 MB | eliminado |
| plane-mq (RabbitMQ) | ~120 MB | eliminado |
| **Total** | **~750 MB** | **~20 MB** |

---

## Próximo paso inmediato — Fase 0

```bash
cd apps/api_rust
cargo init --name plane-api
```
