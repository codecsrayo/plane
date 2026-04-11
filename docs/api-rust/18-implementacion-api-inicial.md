---
titulo: Implementación inicial de la API — bootstrap y API Docs
tags:
  - rust
  - axum
  - utoipa
  - swagger
  - implementacion
relacionado:
  - "[[10-patrones]]"
  - "[[03-stack]]"
  - "[[13-estructura-archivos]]"
  - "[[17-estructura-django]]"
estado: activo
---

# Implementación inicial de la API

> [!INFO] Objetivo de este documento
> Paso a paso para levantar la API Rust en su versión inicial con:
> - `main.rs` completo (AppState + router + Swagger UI)
> - `config.rs` con variables de entorno tipadas
> - `error.rs` unificado
> - Primer endpoint real (`GET /api/health`)
> - Swagger UI disponible en `/api/docs`
>
> El estado actual del repo tiene `main.rs` como stub mínimo y los archivos
> reales en `.todo.rs`. Este doc documenta la versión objetivo de arranque.

---

## 1. Prerequisitos

```toml
# apps/api_rust/Cargo.toml — dependencias ya presentes
axum            = { version = "0.8.8", features = ["macros", "multipart"] }
utoipa          = { version = "5.4.0", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "9.0.2", features = ["axum"] }
tower-http      = { version = "0.6.8", features = ["cors", "trace", "compression-gzip"] }
sea-orm         = { version = "1.1.20", features = ["sqlx-postgres", "runtime-tokio-rustls", ...] }
dotenvy         = "0.15.7"
tracing         = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter", "json"] }
thiserror       = "2.0.18"
tokio           = { version = "1.51.1", features = ["full"] }
serde           = { version = "1.0.228", features = ["derive"] }
serde_json      = "1.0.149"
uuid            = { version = "1.23.0", features = ["v4", "serde"] }
chrono          = { version = "0.4.44", features = ["serde"] }
```

No se requiere agregar ninguna dependencia nueva — todo ya está en el `Cargo.toml`.

---

## 2. Estructura de archivos — versión inicial

El objetivo es pasar de esto:

```
src/
├── main.rs          ← stub: println!("Hello, world!")
├── utils/
│   ├── mod.rs
│   └── soft_delete.rs
└── entities/        ← 122 entidades generadas ✅
```

A esto:

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

## 3. `config.rs` — variables de entorno

```rust
// src/config.rs
use dotenvy::dotenv;
use std::env;

/// Todas las env vars del servidor tipadas.
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
    pub secret_key:    String,   // SECRET_KEY — para tokens JWT futuros
    pub debug:         bool,     // DEBUG — activa logs verbose

    // S3/MinIO
    pub aws_s3_bucket: String,   // AWS_S3_BUCKET_NAME
    pub aws_endpoint:  String,   // AWS_S3_ENDPOINT_URL

    // Cors
    pub cors_origins:  Vec<String>, // CORS_ORIGINS — comma-separated
}

impl Config {
    /// Carga las variables de entorno.
    /// Llama a dotenv() primero para leer `.env` si existe.
    /// Falla en startup si falta una variable obligatoria.
    pub fn from_env() -> anyhow::Result<Self> {
        // Cargar .env si existe — en producción las vars vienen del entorno
        dotenv().ok();

        Ok(Self {
            database_url:  required("DATABASE_URL")?,
            redis_url:     env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into()),
            host:          env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port:          env::var("API_PORT")
                               .unwrap_or_else(|_| "8000".into())
                               .parse()?,
            secret_key:    required("SECRET_KEY")?,
            debug:         env::var("DEBUG").map(|v| v == "1" || v == "true").unwrap_or(false),
            aws_s3_bucket: env::var("AWS_S3_BUCKET_NAME").unwrap_or_default(),
            aws_endpoint:  env::var("AWS_S3_ENDPOINT_URL").unwrap_or_default(),
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
    env::var(key).map_err(|_| anyhow::anyhow!("Variable de entorno requerida no encontrada: {key}"))
}
```

**Variables de entorno mínimas para arrancar:**

```bash
# .env (desarrollo local)
DATABASE_URL=postgresql://plane:plane@localhost:5432/plane
REDIS_URL=redis://localhost:6379
SECRET_KEY=dev-secret-key-change-in-production
DEBUG=1
API_PORT=8000
```

---

## 4. `error.rs` — error unificado

```rust
// src/error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;

/// Error central de la aplicación.
/// Todos los handlers retornan `Result<T, AppError>`.
/// Equivalente a las excepciones DRF en Django.
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

/// Convierte AppError en respuesta HTTP con body JSON.
/// Axum llama esto automáticamente cuando un handler retorna Err(AppError).
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound      => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Unauthorized  => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden     => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            AppError::Database(e)   => {
                tracing::error!("DB error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Internal(e)   => {
                tracing::error!("Internal error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

---

## 5. `routes/health.rs` — primer endpoint + anotación utoipa

Este es el primer endpoint real. Sirve dos propósitos:
1. Verificar que el servidor arranca y la DB conecta
2. Demostrar el patrón completo de anotación para Swagger UI

```rust
// src/routes/health.rs
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{AppState, error::AppError};

/// Respuesta del endpoint de health check.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Estado general del servicio
    pub status: String,
    /// Versión del binario (de Cargo.toml)
    pub version: String,
    /// Conectividad con PostgreSQL
    pub database: DbStatus,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DbStatus {
    pub connected: bool,
    #[schema(nullable)]
    pub error: Option<String>,
}

/// Verifica el estado del servidor y la conectividad con PostgreSQL.
///
/// Equivalente a `GET /health/` en el proxy Traefik actual.
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
    // Verificar conectividad con PostgreSQL
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

## 6. `routes/mod.rs` — router + OpenApi struct

Este es el archivo central que registra todos los endpoints y construye
la especificación OpenAPI 3.0 que consume Swagger UI:

```rust
// src/routes/mod.rs
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::AppState;

pub mod health;

/// Especificación OpenAPI completa de la API.
///
/// Cada handler anotado con #[utoipa::path] debe estar listado
/// en `paths` y cada schema en `components(schemas(...))`.
///
/// A medida que se agregan endpoints, se añaden aquí.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Plane API (Rust)",
        version = "0.1.0",
        description = "API REST de Plane — migración Django → Rust",
        contact(name = "Plane Engineering"),
        license(name = "AGPL-3.0"),
    ),
    paths(
        health::health,
        // Fase 2: workspaces, projects, issues, states, labels...
        // workspaces::list_workspaces,
        // workspaces::get_workspace,
        // issues::list_issues,
    ),
    components(
        schemas(
            health::HealthResponse,
            health::DbStatus,
            // Fase 2: request/response DTOs
            // workspaces::WorkspaceResponse,
            // issues::IssueResponse,
        )
    ),
    tags(
        (name = "Health",     description = "Health check y estado del servidor"),
        (name = "Workspaces", description = "Gestión de workspaces"),
        (name = "Projects",   description = "Gestión de proyectos"),
        (name = "Issues",     description = "Issues y work items"),
        (name = "States",     description = "Estados de issues"),
        (name = "Labels",     description = "Etiquetas"),
    ),
    // Esquemas de seguridad para Swagger UI
    // Los endpoints protegidos usan el guard CurrentUser que lee el header
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Agrega el esquema de seguridad "Token" (header: Authorization: Token <key>)
/// a la especificación OpenAPI. Permite usar "Authorize" en Swagger UI.
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

/// Construye el router principal de la aplicación.
///
/// Incluye:
/// - Rutas de la API bajo `/api/`
/// - Swagger UI bajo `/api/docs`
/// - Spec JSON bajo `/api/docs/openapi.json`
pub fn build_router(state: AppState) -> Router {
    // Router con rutas de la API
    let api_router = Router::new()
        .route("/health", get(health::health))
        // Fase 2: agregar aquí
        // .route("/workspaces", get(workspaces::list))
        // .route("/workspaces/:slug", get(workspaces::get))
        ;

    // Swagger UI — sirve la UI en /api/docs y el JSON en /api/docs/openapi.json
    let swagger = SwaggerUi::new("/api/docs")
        .url("/api/docs/openapi.json", ApiDoc::openapi());

    Router::new()
        .nest("/api", api_router)
        .merge(swagger)
        .with_state(state)
}
```

---

## 7. `main.rs` — bootstrap completo

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

/// Estado global compartido por todos los handlers.
/// Se clona barato — cada campo es un Arc o un pool de conexiones.
#[derive(Clone)]
pub struct AppState {
    pub db:     sea_orm::DatabaseConnection,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Configurar tracing (logging estructurado JSON en prod, pretty en debug)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api_rust=debug,sea_orm=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🦀 Plane API Rust arrancando...");

    // 2. Cargar configuración desde variables de entorno
    let config = Config::from_env()
        .map_err(|e| { tracing::error!("Error de configuración: {e}"); e })?;

    tracing::info!(port = config.port, host = %config.host, "Configuración cargada");

    // 3. Conectar a PostgreSQL via SeaORM
    tracing::info!("Conectando a PostgreSQL...");
    let db = Database::connect(&config.database_url)
        .await
        .map_err(|e| { tracing::error!("No se pudo conectar a PostgreSQL: {e}"); e })?;

    tracing::info!("✅ PostgreSQL conectado");

    // 4. Construir AppState
    let state = AppState {
        db,
        config: Arc::new(config.clone()),
    };

    // 5. Construir router (API + Swagger UI)
    let app = routes::build_router(state);

    // 6. Levantar servidor TCP
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(
        addr = %addr,
        swagger_ui = format!("http://{}:{}/api/docs", config.host, config.port),
        health = format!("http://{}:{}/api/health", config.host, config.port),
        "🚀 Servidor listo"
    );

    // 7. Servir (bloqueante)
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## 8. Swagger UI — cómo funciona

Una vez que el servidor arranca, Swagger UI está disponible en:

```
http://localhost:8000/api/docs
```

La especificación OpenAPI en JSON:

```
http://localhost:8000/api/docs/openapi.json
```

### Qué genera utoipa automáticamente

Cada handler anotado con `#[utoipa::path(...)]` contribuye a la especificación:

```
#[utoipa::path]   →   Path item en OpenAPI spec
#[derive(ToSchema)] →   Schema en components/schemas
#[openapi(paths(...))] →   Registra el path en el ApiDoc
```

El flujo completo:

```
Handler fn    →  #[utoipa::path] macro
                      ↓
              OpenApi spec (JSON)
                      ↓
              SwaggerUi::new("/api/docs")
                      ↓
              GET /api/docs  → HTML + Swagger UI JS
              GET /api/docs/openapi.json → spec JSON
```

### Agregar autorización en Swagger UI

El `SecurityAddon` definido en `routes/mod.rs` agrega el botón "Authorize"
en Swagger UI. Para testear endpoints protegidos:

1. Abrir `http://localhost:8000/api/docs`
2. Click en "Authorize" (candado 🔒)
3. En `TokenAuth (apiKey)` ingresar: `Token <tu_token_aquí>`
4. Click "Authorize" y cerrar
5. Todos los requests subsiguientes llevarán el header `Authorization: Token ...`

---

## 9. Patrón para agregar un endpoint nuevo

Cada endpoint nuevo sigue este proceso de 3 pasos:

### Paso 1 — Crear el handler con anotación utoipa

```rust
// src/routes/workspaces.rs
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{AppState, error::AppError, auth::middleware::CurrentUser};

/// DTO de respuesta para workspace.
/// ToSchema lo expone en la spec OpenAPI.
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
    security(("TokenAuth" = [])),   // ← indica que requiere auth en Swagger UI
    responses(
        (status = 200, description = "Lista de workspaces del usuario", body = Vec<WorkspaceResponse>),
        (status = 401, description = "Token inválido o ausente"),
    )
)]
pub async fn list_workspaces(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    // ... lógica
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
# El endpoint GET /api/workspaces aparece en la sección "Workspaces"
```

---

## 10. Ejecutar y verificar

```bash
# Desde apps/api_rust/
cd apps/api_rust

# 1. Aplicar migraciones (si no están aplicadas)
cargo run -p migration -- up

# 2. Levantar el servidor
cargo run

# Salida esperada:
# 🦀 Plane API Rust arrancando...
# Conectando a PostgreSQL...
# ✅ PostgreSQL conectado
# 🚀 Servidor listo  addr=0.0.0.0:8000
#                    swagger_ui=http://0.0.0.0:8000/api/docs
#                    health=http://0.0.0.0:8000/api/health

# 3. Verificar health
curl http://localhost:8000/api/health
# {"status":"ok","version":"0.1.0","database":{"connected":true,"error":null}}

# 4. Verificar spec OpenAPI
curl http://localhost:8000/api/docs/openapi.json | jq .info
# {"title":"Plane API (Rust)","version":"0.1.0",...}

# 5. Abrir Swagger UI
open http://localhost:8000/api/docs
```

---

## 11. Integración con docker-compose

Para correr junto al resto de Plane en desarrollo:

```yaml
# docker-compose-dev.yml — agregar el servicio api_rust
services:
  api-rust:
    build:
      context: ./apps/api_rust
      dockerfile: Dockerfile
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://plane:plane@plane-db:5432/plane
      REDIS_URL: redis://plane-redis:6379
      SECRET_KEY: ${SECRET_KEY}
      API_PORT: "8000"
      DEBUG: "1"
    depends_on:
      - plane-db
      - plane-redis
    volumes:
      - ./apps/api_rust:/app   # hot reload en desarrollo
```

El proxy Traefik ya existente redirige `/api/` a `api-rust:8000` una vez
que el servicio Django se reemplaza (Fase 5 del plan de migración).

---

## 12. Endpoints de la versión inicial (v0.1)

| Método | Path | Auth | Descripción | Estado |
|--------|------|:----:|-------------|--------|
| `GET` | `/api/health` | ❌ | Health check + DB ping | 📝 Implementar |
| `GET` | `/api/docs` | ❌ | Swagger UI | 📝 Implementar |
| `GET` | `/api/docs/openapi.json` | ❌ | OpenAPI spec JSON | 📝 Implementar |

Los endpoints de la Fase 2 (workspaces, projects, issues, states) se
documentan en `[[07-fases]]` y se implementan una vez que esta base arranca.

---

## 13. Errores comunes en el arranque

| Error | Causa | Solución |
|-------|-------|----------|
| `Variable de entorno requerida no encontrada: DATABASE_URL` | No hay `.env` ni var de entorno | Crear `.env` con las vars del punto 3 |
| `error connecting to database` | PostgreSQL no disponible | Verificar que plane-db corre: `docker compose up plane-db` |
| `Address already in use` | Puerto 8000 ocupado | Cambiar `API_PORT=8001` o matar el proceso |
| `No such file or directory (Cargo.lock)` | Ejecutar desde directorio incorrecto | `cd apps/api_rust && cargo run` |
| Swagger UI carga en blanco | CSS/JS de utoipa-swagger-ui no embedidos | Verificar que `utoipa-swagger-ui` tiene `features = ["axum"]` |

---

*Próximo paso: implementar `CurrentUser` extractor en `src/auth/middleware.rs` → ver [[10-patrones]] sección 3.*
