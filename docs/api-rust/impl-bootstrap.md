---
titulo: Implementación inicial — Bootstrap de la API
aliases:
  - bootstrap
  - main-rs
  - config-rs
  - scalar
tags:
  - rust
  - axum
  - utoipa
  - scalar
  - implementacion
  - parcial
relacionado:
  - "[[MOC]]"
  - "[[vision-stack]]"
  - "[[impl-appstate-repository]]"
  - "[[impl-extractores-auth]]"
  - "[[ref-estructura-archivos]]"
  - "[[plan-fases]]"
estado: parcial
---

# Implementación inicial — Bootstrap de la API

> [!INFO] Objetivo de este documento
> Paso a paso para levantar la API Rust en su versión inicial con:
>
> - `main.rs` completo (AppState + router + Scalar UI)
> - `config.rs` con variables de entorno tipadas
> - `error.rs` unificado
> - Primer endpoint real (`GET /api/health`)
> - Scalar UI disponible en `/api/docs`

---

## Estructura de archivos — versión inicial

```tree
src/
├── main.rs          ✅ bootstrap completo: AppState + router + Scalar + workers
├── config.rs        ✅ env vars tipadas con dotenvy
├── error.rs         ✅ AppError → HTTP responses
├── auth/
│   ├── mod.rs          ✅
│   └── rate_limit.rs   ✅ RateLimitState in-memory (Fase 3 → Redis)
│   ├── extractors.rs   📝 WorkspaceMemberGuard + ProjectMemberGuard — pendiente
│   └── permissions.rs  📝 RBAC — pendiente
├── routes/
│   ├── mod.rs          ✅ build_router() + OpenApi struct
│   └── health.rs       ✅ GET /api/health (primer endpoint real)
├── utils/
│   ├── mod.rs          ✅
│   └── soft_delete.rs  ✅
└── entities/           ✅ generadas por sea-orm-codegen (100+ tablas)
```

---

## `config.rs` — variables de entorno

> [!IMPORTANT] Regla de construcción de URLs
> `.env` solo provee variables **crudas** (`POSTGRES_HOST`, `POSTGRES_PORT`, etc.).
> `config.rs` es el único lugar que **ensambla** `database_url` y `redis_url`.
> Nunca poner `DATABASE_URL` ni `REDIS_URL` completas en `.env`.
> El mismo patrón ya está implementado en `migration/src/main.rs`.

```rust
// src/config.rs
use dotenvy::dotenv;
use std::env;

/// Equivalente a `plane/settings/common.py` en Django.
#[derive(Debug, Clone)]
pub struct Config {
    // Base de datos — construida en from_env() desde POSTGRES_*
    pub database_url:  String,

    // Redis — construida en from_env() desde REDIS_HOST / REDIS_PORT
    pub redis_url:     String,

    // Servidor
    pub host:          String,          // API_HOST — default "0.0.0.0"
    pub port:          u16,             // API_PORT — default 8000

    // App
    pub secret_key:    String,          // SECRET_KEY
    pub debug:         bool,            // DEBUG

    // S3/MinIO
    pub aws_s3_bucket: String,          // AWS_S3_BUCKET_NAME
    pub aws_endpoint:  String,          // AWS_S3_ENDPOINT_URL

    // Cookies
    pub cookie_domain: Option<String>,  // COOKIE_DOMAIN
    pub is_production: bool,            // derivado de DEBUG=0

    // CORS
    pub cors_origins:  Vec<String>,     // CORS_ORIGINS — comma-separated
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // dotenv es best-effort: en producción no hay .env file y eso es normal.
        // Se loguea a nivel debug para no contaminar logs de prod. // silence-patterns-ok
        match dotenv() {
            Ok(path) => tracing::debug!(".env loaded from {}", path.display()),
            Err(dotenvy::Error::Io(_)) => tracing::debug!("No .env file found, using environment variables directly"),
            Err(e) => tracing::warn!("dotenv error (non-fatal): {e}"),
        }

        let debug = env::var("DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            database_url:  build_database_url()?,
            redis_url:     build_redis_url(),
            host:          env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port:          env::var("API_PORT")
                               .unwrap_or_else(|_| "8000".into())
                               .parse()?,
            secret_key:    required("SECRET_KEY")?,
            debug,
            is_production: !debug,
            aws_s3_bucket: env::var("AWS_S3_BUCKET_NAME").unwrap_or_default(),
            aws_endpoint:  env::var("AWS_S3_ENDPOINT_URL").unwrap_or_default(),
            cookie_domain: env::var("COOKIE_DOMAIN").ok(),
            cors_origins:  env::var("CORS_ORIGINS")
                               .unwrap_or_default()
                               .split(',')
                               .map(|s| s.trim().to_string())
                               .filter(|s| !s.is_empty())
                               .collect(),
        })
    }
}

/// Construye `postgresql://user:pass@host:port/db` desde variables crudas.
///
/// Variables leídas (todas con defaults de desarrollo):
/// - `POSTGRES_USER`     (default: "plane")
/// - `POSTGRES_PASSWORD` (default: "")
/// - `POSTGRES_HOST`     (default: "localhost")
/// - `POSTGRES_PORT`     (default: "5432")
/// - `POSTGRES_DB`       (default: "plane")
fn build_database_url() -> anyhow::Result<String> {
    let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "plane".into());
    let pass = env::var("POSTGRES_PASSWORD").unwrap_or_default();
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".into());
    let db   = env::var("POSTGRES_DB").unwrap_or_else(|_| "plane".into());

    // Validar que el puerto sea numérico antes de construir la URL.
    port.parse::<u16>()
        .map_err(|_| anyhow::anyhow!("POSTGRES_PORT inválido: '{port}' — debe ser un número entre 1 y 65535"))?;

    Ok(format!("postgresql://{user}:{pass}@{host}:{port}/{db}"))
}

/// Construye `redis://host:port` desde variables crudas.
///
/// Variables leídas:
/// - `REDIS_HOST` (default: "localhost")
/// - `REDIS_PORT` (default: "6379")
fn build_redis_url() -> String {
    let host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".into());
    format!("redis://{host}:{port}")
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("Variable de entorno requerida: {key}"))
}
```

**Variables mínimas para arrancar** (`.env` en desarrollo):

```bash
# PostgreSQL — variables crudas; config.rs ensambla la URL
POSTGRES_USER=plane
POSTGRES_PASSWORD=plane
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=plane

# Redis — variables crudas; config.rs ensambla la URL
REDIS_HOST=localhost
REDIS_PORT=6379

# App
SECRET_KEY=dev-secret-key-change-in-production
DEBUG=1
API_PORT=8000
```

---

## `routes/health.rs` — primer endpoint + anotación utoipa

```rust
// src/routes/health.rs
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{AppState, error::AppError};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status:   String,
    pub version:  String,
    pub database: DbStatus,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DbStatus {
    pub connected: bool,
    // ⚠️ NO incluir el mensaje de error interno — puede contener
    // connection strings, schema names u otros detalles del servidor.
    // Sólo exponer si DEBUG=true para facilitar diagnóstico en desarrollo.
    #[schema(nullable)]
    pub error: Option<String>,
}

/// Verifica el estado del servidor y la conectividad con PostgreSQL.
/// No requiere autenticación — es el único endpoint completamente público.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health",
    responses(
        (status = 200, description = "Servidor activo", body = HealthResponse),
        (status = 503, description = "DB no disponible", body = HealthResponse),
    )
)]
pub async fn health(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;

    let db_result = state.db.ping().await;
    let connected = db_result.is_ok();

    // ✅ En producción: loguear internamente, no exponer al cliente.
    // En desarrollo (debug=true): incluir mensaje para facilitar diagnóstico.
    let error_msg = match &db_result {
        Ok(_)  => None,
        Err(e) => {
            tracing::error!(error = %e, "Database ping failed");
            if state.config.debug {
                Some(e.to_string())
            } else {
                Some("Database unavailable".into()) // mensaje genérico en producción
            }
        }
    };

    let db_status = DbStatus { connected, error: error_msg };
    let response = Json(HealthResponse {
        status:   if connected { "ok".into() } else { "degraded".into() },
        version:  env!("CARGO_PKG_VERSION").to_string(),
        database: db_status,
    });

    // ✅ HTTP 503 cuando DB no está disponible — no retornar 200 en degradado.
    // Los health checks de load balancers/k8s dependen del status code correcto.
    let status = if connected { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, response)
}
```

---

## `routes/mod.rs` — router + OpenApi struct

```rust
// src/routes/mod.rs
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use crate::AppState;

pub mod health;
// pub mod workspaces;  // agregar en Fase 2
// pub mod issues;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Plane API (Rust)",
        version = "0.1.0",
        description = "API REST de Plane — migración Django → Rust",
    ),
    paths(
        health::health,
        // Fase 2: workspaces::list_workspaces,
    ),
    components(
        schemas(
            health::HealthResponse,
            health::DbStatus,
        )
    ),
    tags(
        (name = "Health",     description = "Health check"),
        (name = "Workspaces", description = "Gestión de workspaces"),
        (name = "Projects",   description = "Gestión de proyectos"),
        (name = "Issues",     description = "Issues y work items"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Agrega el esquema de seguridad "TokenAuth" a la spec OpenAPI.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "TokenAuth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key"))),
            );
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/health", get(health::health));
        // .route("/workspaces", get(workspaces::list))  // Fase 2

    let mut router = Router::new()
        .nest("/api", api_router);

    // ✅ Scalar UI sólo disponible en desarrollo (DEBUG=true).
    // En producción expone el schema completo de la API — información valiosa
    // para atacantes que quieran enumerar endpoints y estructuras de datos.
    // Si se necesita en staging, proteger con BasicAuth o IP allowlist via Traefik.
    if state.config.debug {
        router = router.merge(Scalar::with_url("/api/docs", ApiDoc::openapi()));
        tracing::warn!("Scalar UI habilitado (DEBUG=true) — deshabilitar en producción");
    }

    router.with_state(state)
}
```

---

## `main.rs` — bootstrap completo

> **AppState** — definición completa (Fase 1 → Fase 3) en [[impl-appstate-repository#1. AppState — estado global del servidor]].

```rust
// src/main.rs
use std::{net::SocketAddr, sync::Arc};
use sea_orm::Database;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod error;
mod routes;
pub mod entities;
pub mod utils;
pub mod auth;

use config::Config;
use auth::rate_limit::RateLimitState;

// AppState se define en src/main.rs — ver impl-appstate-repository para la definición completa.
// Fase 1 mínima:
//   pub db: sea_orm::DatabaseConnection
//   pub config: Arc<Config>
//   pub rate_limit: Arc<RateLimitState>

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Logging estructurado
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api_rust=debug,sea_orm=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🦀 Plane API Rust arrancando...");

    // 2. Configuración
    let config = Config::from_env()
        .map_err(|e| { tracing::error!("Error de configuración: {e}"); e })?;

    tracing::info!(port = config.port, host = %config.host, "Configuración cargada");

    // 3. PostgreSQL
    tracing::info!("Conectando a PostgreSQL...");
    let db = Database::connect(&config.database_url)
        .await
        .map_err(|e| { tracing::error!("No se pudo conectar a PostgreSQL: {e}"); e })?;

    tracing::info!("✅ PostgreSQL conectado");

    // 4. AppState
    let state = AppState {
        db,
        config:     Arc::new(config.clone()),
        rate_limit: Arc::new(RateLimitState::default()),
    };

    // 5. Router
    let app = routes::build_router(state);

    // 6. Servidor TCP
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(
        addr = %addr,
        scalar_ui  = format!("http://{}:{}/api/docs", config.host, config.port),
        health     = format!("http://{}:{}/api/health", config.host, config.port),
        "🚀 Servidor listo"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
```

---

## Patrón para agregar un endpoint nuevo

### Paso 1 — Handler con anotación utoipa

```rust
// src/routes/workspaces.rs
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WorkspaceResponse {
    pub id:   uuid::Uuid,
    pub name: String,
    pub slug: String,
}

#[utoipa::path(
    get,
    path = "/api/workspaces",
    tag = "Workspaces",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Lista de workspaces", body = Vec<WorkspaceResponse>),
        (status = 401, description = "No autenticado"),
    )
)]
pub async fn list_workspaces(
    State(state): State<AppState>,
    ApiKeyUser(ctx): ApiKeyUser,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    let user = ctx.user;
    todo!()
}
```

### Paso 2 — Registrar en `routes/mod.rs`

```rust
// src/routes/mod.rs
// En build_router():
.route("/workspaces", get(workspaces::list_workspaces))

// En #[openapi(paths(...))]:
workspaces::list_workspaces,

// En #[openapi(components(schemas(...)))]:
workspaces::WorkspaceResponse,
```

### Paso 3 — Verificar en Scalar UI

```bash
cargo run
# Abrir http://localhost:8000/api/docs
```

---

## Ejecutar y verificar

```bash
cd apps/api_rust

# 1. Aplicar migraciones
cargo run -p migration -- up

# 2. Levantar servidor
cargo run

# 3. Verificar health
curl http://localhost:8000/api/health
# {"status":"ok","version":"0.1.0","database":{"connected":true,"error":null}}

# 4. Abrir Scalar UI
open http://localhost:8000/api/docs
```

---

## Errores comunes en el arranque

| Error                                       | Causa                        | Solución                                         |
| ------------------------------------------- | ---------------------------- | ------------------------------------------------ |
| `Variable de entorno requerida: SECRET_KEY` | No hay `.env` o falta la var | Crear `.env` con las vars del paso anterior      |
| `POSTGRES_PORT inválido: 'abc'`             | `POSTGRES_PORT` no es número | Corregir el valor en `.env`                      |
| `error connecting to database`              | PostgreSQL no disponible     | `docker compose up plane-db`                     |
| `Address already in use`                    | Puerto 8000 ocupado          | `API_PORT=8001` o matar el proceso               |
| `No such file or directory (Cargo.lock)`    | Directorio incorrecto        | `cd apps/api_rust && cargo run`                  |
| Scalar UI carga en blanco                   | Feature faltante             | Verificar `features = ["axum"]` en utoipa-scalar |

---

## Endpoints de la versión inicial (v0.1)

| Método | Path                     | Auth | Estado          |
| ------ | ------------------------ | :--: | --------------- |
| `GET`  | `/api/health`            |  ❌  | ✅ Implementado |
| `GET`  | `/api/docs`              |  ❌  | ✅ Implementado |
| `GET`  | `/api/docs/openapi.json` |  ❌  | ✅ Implementado |

Los endpoints de la Fase 2 se listan en [[plan-fases#Fase 2 — Endpoints de alta frecuencia]].

---

## 🔗 Navegar

← [[plan-fases]] | [[MOC]] | → [[impl-autenticacion]]

**Siguiente paso:** implementar `WorkspaceMemberGuard` y `ProjectMemberGuard` → [[impl-extractores-auth]]

---

_`docs/api-rust/impl-bootstrap.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
