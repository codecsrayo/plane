// src/lib.rs — public library exposed to the binary and integration tests.
//
// Reason: integration tests in `tests/` compile as an external crate and
// require access to project modules. Exposing them here allows:
//   - `use api_rust::AppState;`
//   - `use api_rust::routes::build_router;`
//   - property tests (proptest) against pure helpers (e.g. CSRF, pagination).
//
// The binary (`main.rs`) imports the same symbols via `use api_rust::…`,
// avoiding duplication of the module tree between lib and bin.

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

/// Global server state.
///
/// Clonable (all fields are `Arc` or pools with internal `Arc`), so
/// each handler receives a cheap copy via `State<AppState>` without
/// additional contention.
#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub db: sea_orm::DatabaseConnection,
    pub redis: RedisPool,
    pub config: Arc<Config>,
    pub rate_limit: Arc<RateLimitState>,
    /// Shared `sqlx::PgPool` for apalis job enqueuing.
    /// Avoids creating a new connection for each enqueue.
    pub pg_pool: sqlx::PgPool,
}
