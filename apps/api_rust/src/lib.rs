// src/lib.rs — biblioteca pública expuesta al binario y a los tests de integración.
//
// Razón: los tests de integración en `tests/` compilan como crate externo y
// requieren acceso a módulos del proyecto. Exponerlos aquí permite:
//   - `use api_rust::AppState;`
//   - `use api_rust::routes::build_router;`
//   - tests de propiedad (proptest) contra helpers puros (p.ej. CSRF, pagination).
//
// El binario (`main.rs`) importa los mismos símbolos vía `use api_rust::…`,
// evitando duplicar el árbol de módulos entre lib y bin.

pub mod auth;
pub mod config;
pub mod entities;
pub mod error;
pub mod jobs;
pub mod routes;
pub mod utils;

use std::sync::Arc;

use auth::rate_limit::RateLimitState;
use config::Config;
use fred::prelude::Pool as RedisPool;

/// Estado global del servidor.
///
/// Clonable (todos los campos son `Arc` o pools con `Arc` interno), por lo
/// que cada handler recibe una copia barata vía `State<AppState>` sin
/// contención adicional.
#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub db: sea_orm::DatabaseConnection,
    pub redis: RedisPool,
    pub config: Arc<Config>,
    pub rate_limit: Arc<RateLimitState>,
    /// `sqlx::PgPool` compartido para el enqueue de jobs de apalis.
    /// Evita crear una conexión nueva por cada enqueue.
    pub pg_pool: sqlx::PgPool,
}
