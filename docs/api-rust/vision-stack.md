---
titulo: Stack completo Rust
aliases:
  - stack
  - dependencias
  - librerías
tags:
  - rust
  - stack
  - axum
  - seaorm
  - apalis
relacionado:
  - "[[MOC]]"
  - "[[vision-objetivo]]"
  - "[[vision-arquitectura]]"
  - "[[impl-appstate-repository]]"
  - "[[impl-error-jobs-cron]]"
estado: activo
---

# Stack completo Rust

> [!TIP] Ver en práctica
> Para ver cómo cada librería se usa en código real, ver [[impl-appstate-repository]], [[impl-extractores-auth]], [[impl-error-jobs-cron]].

---

## Tabla completa de dependencias

| Rol             | Librería                         | Versión   | Reemplaza                            |
| --------------- | -------------------------------- | --------- | ------------------------------------ |
| HTTP framework  | **Axum**                         | 0.8.8     | Django REST Framework + uvicorn      |
| Async runtime   | **Tokio**                        | 1.51.1    | —                                    |
| ORM             | **SeaORM**                       | 1.1.20    | Django ORM                           |
| Migrations      | **sea-orm-migration**            | 1.1.20    | Django migrations (126 archivos)     |
| Serialización   | **serde + serde_json**           | 1.0       | DRF serializers                      |
| Auth cookies    | **axum-extra** (feature: cookie) | 0.10      | SessionAuthentication Django         |
| Auth JWT        | **jsonwebtoken**                 | 9         | DRF TokenAuthentication (GitHub App) |
| Background jobs | **apalis** (backend: Postgres)   | latest    | Celery bgworker + RabbitMQ           |
| Cron jobs       | **tokio-cron-scheduler**         | latest    | Celery beatworker                    |
| Redis           | **fred**                         | v10       | django-redis                         |
| S3 / MinIO      | **aws-sdk-s3**                   | latest    | boto3 + django-storages              |
| Email           | **lettre**                       | latest    | Django email backend                 |
| HTTP client     | **reqwest**                      | latest    | requests                             |
| Logging         | **tracing + tracing-subscriber** | 0.1       | python-json-logger                   |
| Config          | **dotenvy**                      | 0.15.7    | django settings                      |
| Métricas        | **axum-prometheus**              | latest    | scout-apm                            |
| API Docs        | **utoipa + utoipa-swagger-ui**   | 5.4 / 9.0 | drf-spectacular                      |
| Error handling  | **thiserror**                    | 2.0       | DRF exceptions                       |
| Misc UUIDs      | **uuid** (feature: v4, serde)    | 1.23      | uuid Django                          |
| Fechas          | **chrono** (feature: serde)      | 0.4       | Django timezone                      |
| Base64          | **base64**                       | 0.22      | Python base64 (GitHub JWT)           |

---

## `Cargo.toml` — dependencias mínimas de arranque

Las siguientes ya están presentes en `apps/api_rust/Cargo.toml`:

```toml
# ── HTTP & Async ──────────────────────────────────────────────────────────────
axum            = { version = "0.8.8", features = ["macros", "multipart"] }
axum-extra      = { version = "0.10", features = ["cookie"] }
tokio           = { version = "1.51.1", features = ["full"] }
tower           = "0.5.3"
tower-http      = { version = "0.6.8", features = ["cors", "trace", "compression-gzip"] }
time            = "0.3"

# ── ORM & DB ─────────────────────────────────────────────────────────────────
sea-orm         = { version = "1.1.20", features = [
    "sqlx-postgres", "runtime-tokio-rustls", "macros",
    "with-uuid", "with-chrono", "with-json",
] }
dotenvy         = "0.15.7"

# ── Background jobs ──────────────────────────────────────────────────────────
apalis          = { version = "0.7.4", features = ["limit", "timeout", "retry"] }
apalis-sql      = { version = "0.7.4", features = ["postgres", "tokio-comp"] }

# ── Redis ────────────────────────────────────────────────────────────────────
fred            = { version = "10.1.0", features = ["i-std", "i-client", ...] }

# ── Auth & Crypto ─────────────────────────────────────────────────────────────
jsonwebtoken    = { version = "9", features = [] }
base64          = "0.22"
sha2            = "0.10"

# ── Serialización ─────────────────────────────────────────────────────────────
serde           = { version = "1.0.228", features = ["derive"] }
serde_json      = "1.0.149"
uuid            = { version = "1.23.0", features = ["v4", "serde"] }
chrono          = { version = "0.4.44", features = ["serde"] }

# ── API Docs ──────────────────────────────────────────────────────────────────
utoipa          = { version = "5.4.0", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "9.0.2", features = ["axum"] }

# ── Observabilidad ────────────────────────────────────────────────────────────
tracing         = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter", "json"] }

# ── Error handling ────────────────────────────────────────────────────────────
thiserror       = "2.0.18"
anyhow          = "1"
```

> El `Cargo.toml` real en `apps/api_rust/Cargo.toml` es la fuente de verdad — este bloque es una vista consolidada de las dependencias principales.

---

## Notas sobre librerías críticas

### fred v10 — API diferente a `redis-rs`

- Pool nativo async: `fred::clients::Pool`
- Pipelines: `client.pipeline()`
- Pub/Sub (para plane-live sync): `subscriber_client()`
- **No mezclar con `deadpool-redis`**
- Ver punto 12 en [[plan-riesgos]]

### apalis — backend PostgreSQL

- Elimina RabbitMQ completamente
- Jobs corren en el **mismo proceso** que Axum (mismo binario Tokio)
- Requiere `PostgresStorage::setup(&pg_pool).await?` al arrancar — donde `pg_pool` es `sqlx::PgPool` extraído vía `db.get_postgres_connection_pool()` (no el `DatabaseConnection` de SeaORM directamente)
- Ver [[impl-error-jobs-cron]] para registro de workers
- Ver [[dominio-workspace-seed]] para ejemplo completo

### axum-extra — cookies

- Feature `cookie` incluye `CookieJar` y `PrivateCookieJar`
- Se usa `CookieJar` (no PrivateCookieJar) porque las sesiones Django no están cifradas por el servidor
- Compatible con axum 0.8

---

## 🔗 Navegar

← [[vision-objetivo]] | [[MOC]] | → [[vision-arquitectura]] | Código: [[impl-bootstrap]]
