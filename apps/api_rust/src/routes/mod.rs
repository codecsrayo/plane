// src/routes/mod.rs
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use crate::AppState;

pub mod health;
// pub mod workspaces;  // Fase 2
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

    // ✅ Scalar UI solo en desarrollo (DEBUG=true).
    // En producción expone el schema completo — proteger con IP allowlist si se necesita en staging.
    if state.config.debug {
        router = router.merge(Scalar::with_url("/api/docs", ApiDoc::openapi()));
        tracing::warn!("Scalar UI habilitado (DEBUG=true) — deshabilitar en producción");
    }

    router.with_state(state)
}
