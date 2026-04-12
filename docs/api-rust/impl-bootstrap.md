---
titulo: Implementación inicial — Bootstrap de la API
aliases:
  - bootstrap
  - main-rs
  - config-rs
  - swagger
tags:
  - rust
  - axum
  - utoipa
  - swagger
  - implementacion
relacionado:
  - "[[MOC]]"
  - "[[vision-stack]]"
  - "[[impl-appstate-repository]]"
  - "[[impl-extractores-auth]]"
  - "[[ref-estructura-archivos]]"
  - "[[plan-fases]]"
estado: activo
---

# Implementación inicial — Bootstrap de la API

> [!INFO] Objetivo de este documento
> Paso a paso para levantar la API Rust en su versión inicial con:
>
> - `main.rs` completo (AppState + router + Swagger UI)
> - `config.rs` con variables de entorno tipadas
> - `error.rs` unificado
> - Primer endpoint real (`GET /api/health`)
> - Swagger UI disponible en `/api/docs`

---

## Estructura de archivos — versión inicial

```
src/
├── main.rs          ← bootstrap completo: AppState + router + Swagger + workers
├── config.rs        ← env vars tipadas con dotenvy
├── error.rs         ← AppError → HTTP responses
├── lib.rs           ← re-exports públicos
├── auth/
│   ├── mod.rs
│   └── middleware.rs   ← CurrentUser extractor
├── routes/
│   ├── mod.rs          ← build_router() + OpenApi struct
│   └── health.rs       ← GET /api/health (primer endpoint real)
├── utils/
│   ├── mod.rs
│   └── soft_delete.rs  ← ya existe ✅
└── entities/           ← ya existen ✅
```

---

## `config.rs` — variables de entorno

```rust
// src/config.rs
use dotenvy::dotenv;
use std::env;

/// Equivalente a `plane/settings/common.py` en Django.
#[derive(Debug, Clone)]
pub struct Config {
    // Base de datos
    pub database_url:  String,   // DATABASE_URL — postgresql://user:pass@host/db

    // Redis
    pub redis_url:     String,   // REDIS_URL — redis://host:6379

    // Servidor
    pub host:          String,   // API_HOST — default "0.0.0.0"
    pub port:          u16,      // API_PORT — default 8000

    // App
    pub secret_key:    String,   // SECRET_KEY
    pub debug:         bool,     // DEBUG

    // S3/MinIO
    pub aws_s3_bucket: String,   // AWS_S3_BUCKET_NAME
    pub aws_endpoint:  String,   // AWS_S3_ENDPOINT_URL

    // Cookies
    pub cookie_domain:    Option<String>, // COOKIE_DOMAIN
    pub is_production:    bool,           // derivado de DEBUG=0

    // CORS
    pub cors_origins:  Vec<String>, // CORS_ORIGINS — comma-separated
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv().ok();

        let debug = env::var("DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            database_url:  required("DATABASE_URL")?,
            redis_url:     env::var("REDIS_URL")
                               .unwrap_or_else(|_| "redis://localhost:6379".into()),
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

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("Variable de entorno requerida: {key}"))
}
```

**Variables mínimas para arrancar** (`.env` en desarrollo):

```bash
DATABASE_URL=postgresql://plane:plane@localhost:5432/plane
REDIS_URL=redis://localhost:6379
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
    #[schema(nullable)]
    pub error: Option<String>,
}

/// Verifica el estado del servidor y la conectividad con PostgreSQL.
/// No requiere autenticación.
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
) -> Result<Json<HealthResponse>, AppError> {
    let db_status = match state.db.ping().await {
        Ok(_)  => DbStatus { connected: true, error: None },
        Err(e) => DbStatus { connected: false, error: Some(e.to_string()) },
    };

    Ok(Json(HealthResponse {
        status:   if db_status.connected { "ok".into() } else { "degraded".into() },
        version:  env!("CARGO_PKG_VERSION").to_string(),
        database: db_status,
    }))
}
```

---

## `routes/mod.rs` — router + OpenApi struct

```rust
// src/routes/mod.rs
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
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

/// Agrega el esquema de seguridad "Token" a la spec OpenAPI.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "TokenAuth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/health", get(health::health));
        // .route("/workspaces", get(workspaces::list))  // Fase 2

    let swagger = SwaggerUi::new("/api/docs")
        .url("/api/docs/openapi.json", ApiDoc::openapi());

    Router::new()
        .nest("/api", api_router)
        .merge(swagger)
        .with_state(state)
}
```

---

## `main.rs` — bootstrap completo

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

#[derive(Clone)]
pub struct AppState {
    pub db:     sea_orm::DatabaseConnection,
    pub config: Arc<Config>,
    // Fase 2: pub redis: fred::clients::Pool,
    // Fase 3: pub job_storage: PgPool,
}

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
        config: Arc::new(config.clone()),
    };

    // 5. Router
    let app = routes::build_router(state);

    // 6. Servidor TCP
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(
        addr = %addr,
        swagger_ui = format!("http://{}:{}/api/docs", config.host, config.port),
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
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    todo!()
}
```

### Paso 2 — Registrar en `routes/mod.rs`

```rust
// En build_router():
.route("/workspaces", get(workspaces::list_workspaces))

// En #[openapi(paths(...))]:
workspaces::list_workspaces,

// En #[openapi(components(schemas(...)))]:
workspaces::WorkspaceResponse,
```

### Paso 3 — Verificar en Swagger UI

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

# 4. Abrir Swagger UI
open http://localhost:8000/api/docs
```

---

## Errores comunes en el arranque

| Error                                         | Causa                    | Solución                                             |
| --------------------------------------------- | ------------------------ | ---------------------------------------------------- |
| `Variable de entorno requerida: DATABASE_URL` | No hay `.env`            | Crear `.env` con las vars del paso anterior          |
| `error connecting to database`                | PostgreSQL no disponible | `docker compose up plane-db`                         |
| `Address already in use`                      | Puerto 8000 ocupado      | `API_PORT=8001` o matar el proceso                   |
| `No such file or directory (Cargo.lock)`      | Directorio incorrecto    | `cd apps/api_rust && cargo run`                      |
| Swagger UI carga en blanco                    | Feature faltante         | Verificar `features = ["axum"]` en utoipa-swagger-ui |

---

## Endpoints de la versión inicial (v0.1)

| Método | Path                     | Auth | Estado         |
| ------ | ------------------------ | :--: | -------------- |
| `GET`  | `/api/health`            |  ❌  | 📝 Implementar |
| `GET`  | `/api/docs`              |  ❌  | 📝 Implementar |
| `GET`  | `/api/docs/openapi.json` |  ❌  | 📝 Implementar |

Los endpoints de la Fase 2 se listan en [[plan-fases#Fase 2 — Endpoints de alta frecuencia]].

---

## 🔗 Navegar

← [[plan-fases]] | [[MOC]] | → [[impl-autenticacion]]

**Siguiente paso:** implementar `CurrentUser` extractor → [[impl-extractores-auth]]
