// src/routes/health.rs
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: DbStatus,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DbStatus {
    pub connected: bool,
    /// Only exposed if DEBUG=true to facilitate diagnosis in development.
    /// In production a generic message is returned to avoid leaking internal details.
    #[schema(nullable)]
    pub error: Option<String>,
}

/// Verifies server status and PostgreSQL connectivity.
/// Does not require authentication — it is the only completely public endpoint.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse),
        (status = 503, description = "Database unavailable", body = HealthResponse),
    )
)]
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let db_result = state.db.ping().await;
    let connected = db_result.is_ok();

    let error_msg = match &db_result {
        Ok(_) => None,
        Err(e) => {
            tracing::error!(error = %e, "Database ping failed");
            if state.config.debug {
                Some(e.to_string())
            } else {
                Some("Database unavailable".into())
            }
        }
    };

    let response = Json(HealthResponse {
        status: if connected {
            "ok".into()
        } else {
            "degraded".into()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DbStatus {
            connected,
            error: error_msg,
        },
    });

    // HTTP 503 when DB is unavailable — k8s health checks depend on the status code.
    let status = if connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, response)
}
