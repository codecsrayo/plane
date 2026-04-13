// src/routes/mod.rs
use crate::{auth, AppState};
use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

pub mod health;
// pub mod workspaces;  // Fase 2
// pub mod issues;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Plane API (Rust)",
        version = "0.1.0",
        description = "Plane REST API",
    ),
    paths(
        health::health,
        auth::csrf::get_csrf_token,
        auth::email_auth::sign_in,
        auth::email_auth::sign_up,
        auth::email_auth::sign_in_space,
        auth::email_auth::sign_up_space,
        auth::email_check::email_check,
        auth::email_check::email_check_space,
        auth::magic_auth::magic_generate,
        auth::magic_auth::magic_generate_space,
        auth::magic_auth::magic_sign_in,
        auth::magic_auth::magic_sign_up,
        auth::magic_auth::magic_sign_in_space,
        auth::magic_auth::magic_sign_up_space,
        auth::forgot_reset_password::forgot_password,
        auth::forgot_reset_password::reset_password,
        auth::forgot_reset_password::forgot_password_space,
        auth::forgot_reset_password::reset_password_space,
        auth::logout::logout,
        auth::logout::logout_space,
        auth::password_management::change_password,
        auth::password_management::set_password,
        // Fase 2: workspaces::list_workspaces,
    ),
    components(
        schemas(
            auth::responses::AuthErrorBody,
            auth::csrf::CsrfTokenResponse,
            auth::email_auth::CredentialAuthForm,
            auth::email_check::EmailCheckRequest,
            auth::magic_auth::MagicAuthForm,
            auth::magic_auth::MagicGenerateRequest,
            auth::magic_auth::MagicGenerateResponse,
            auth::forgot_reset_password::ForgotPasswordRequest,
            auth::forgot_reset_password::ResetPasswordForm,
            auth::password_management::ChangePasswordRequest,
            auth::password_management::SetPasswordRequest,
            auth::responses::EmailCheckResponse,
            auth::responses::PasswordMessageResponse,
            health::HealthResponse,
            health::DbStatus,
        )
    ),
    tags(
        (name = "Health",     description = "Health checks"),
        (name = "Auth",       description = "Authentication"),
        (name = "Workspaces", description = "Workspace management"),
        (name = "Projects",   description = "Project management"),
        (name = "Issues",     description = "Issues and work items"),
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

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn build_router(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/health", get(health::health))
        .route("/auth/get-csrf-token", get(auth::csrf::get_csrf_token))
        .route("/auth/sign-in", post(auth::email_auth::sign_in))
        .route("/auth/sign-up", post(auth::email_auth::sign_up))
        .route(
            "/auth/magic-generate",
            post(auth::magic_auth::magic_generate),
        )
        .route("/auth/magic-sign-in", post(auth::magic_auth::magic_sign_in))
        .route("/auth/magic-sign-up", post(auth::magic_auth::magic_sign_up))
        .route(
            "/auth/spaces/sign-in",
            post(auth::email_auth::sign_in_space),
        )
        .route(
            "/auth/spaces/sign-up",
            post(auth::email_auth::sign_up_space),
        )
        .route(
            "/auth/spaces/magic-generate",
            post(auth::magic_auth::magic_generate_space),
        )
        .route(
            "/auth/spaces/magic-sign-in",
            post(auth::magic_auth::magic_sign_in_space),
        )
        .route(
            "/auth/spaces/magic-sign-up",
            post(auth::magic_auth::magic_sign_up_space),
        )
        .route("/auth/email-check", post(auth::email_check::email_check))
        .route(
            "/auth/spaces/email-check",
            post(auth::email_check::email_check_space),
        )
        .route(
            "/auth/change-password",
            post(auth::password_management::change_password),
        )
        .route(
            "/auth/set-password",
            post(auth::password_management::set_password),
        )
        .route(
            "/auth/forgot-password",
            post(auth::forgot_reset_password::forgot_password),
        )
        .route(
            "/auth/reset-password/:uidb64/:token",
            post(auth::forgot_reset_password::reset_password),
        )
        .route(
            "/auth/spaces/forgot-password",
            post(auth::forgot_reset_password::forgot_password_space),
        )
        .route(
            "/auth/spaces/reset-password/:uidb64/:token",
            post(auth::forgot_reset_password::reset_password_space),
        )
        .route("/auth/sign-out", post(auth::logout::logout))
        .route("/auth/spaces/sign-out", post(auth::logout::logout_space))
        .layer(middleware::from_fn(
            auth::rate_limit::rate_limit_headers_middleware,
        ));
    // .route("/workspaces", get(workspaces::list))  // Fase 2

    let mut router = Router::new().nest("/api", api_router);

    // ✅ Scalar UI solo en desarrollo (DEBUG=true).
    // En producción expone el schema completo — proteger con IP allowlist si se necesita en staging.
    if state.config.debug {
        router = router
            .route("/api/docs/openapi.json", get(openapi_json))
            .merge(Scalar::with_url("/api/docs", ApiDoc::openapi()));
        tracing::warn!("Scalar UI habilitado (DEBUG=true) — deshabilitar en producción");
    }

    router.with_state(state)
}
