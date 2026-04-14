// src/routes/mod.rs
use crate::{auth, AppState};
use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Json, Router,
};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

pub mod health;
pub mod projects;
pub mod states;
pub mod workspaces;

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
        workspaces::slug_check,
        workspaces::list_workspaces,
        workspaces::create_workspace,
        workspaces::get_workspace,
        workspaces::update_workspace,
        workspaces::delete_workspace,
        workspaces::list_members,
        workspaces::update_member,
        workspaces::remove_member,
        workspaces::list_invitations,
        workspaces::create_invitations,
        workspaces::delete_invitation,
        projects::list_projects,
        projects::create_project,
        projects::get_project,
        projects::update_project,
        projects::delete_project,
        projects::list_project_members,
        projects::update_project_member,
        projects::remove_project_member,
        projects::list_project_invitations,
        projects::create_project_invitations,
        projects::delete_project_invitation,
        states::list_states,
        states::get_state,
        states::create_state,
        states::update_state,
        states::delete_state,
        states::intake_state,
        states::mark_default,
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
            workspaces::WorkspaceResponse,
            workspaces::CreateWorkspaceRequest,
            workspaces::UpdateWorkspaceRequest,
            workspaces::WorkspaceMemberResponse,
            workspaces::UpdateMemberRoleRequest,
            workspaces::InvitationResponse,
            workspaces::CreateInvitationRequest,
            workspaces::InviteEmail,
            workspaces::SlugCheckResponse,
            projects::ProjectResponse,
            projects::CreateProjectRequest,
            projects::UpdateProjectRequest,
            projects::ProjectMemberResponse,
            projects::UpdateProjectMemberRequest,
            projects::ProjectInvitationResponse,
            projects::CreateProjectInvitationRequest,
            projects::ProjectInviteEmail,
            states::StateResponse,
            states::CreateStateRequest,
            states::UpdateStateRequest,
        )
    ),
    tags(
        (name = "Health",     description = "Health checks"),
        (name = "Auth",       description = "Authentication"),
        (name = "Workspaces", description = "Workspace management"),
        (name = "Projects",   description = "Project management"),
        (name = "States",     description = "Project state management"),
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
        // ── Workspaces (Fase 2) ──────────────────────────────────────────────
        .route(
            "/workspace-slug-check",
            get(workspaces::slug_check),
        )
        .route(
            "/workspaces",
            get(workspaces::list_workspaces).post(workspaces::create_workspace),
        )
        .route(
            "/workspaces/:slug",
            get(workspaces::get_workspace)
                .patch(workspaces::update_workspace)
                .delete(workspaces::delete_workspace),
        )
        .route(
            "/workspaces/:slug/members",
            get(workspaces::list_members),
        )
        .route(
            "/workspaces/:slug/members/:pk",
            patch(workspaces::update_member).delete(workspaces::remove_member),
        )
        .route(
            "/workspaces/:slug/invitations",
            get(workspaces::list_invitations).post(workspaces::create_invitations),
        )
        .route(
            "/workspaces/:slug/invitations/:pk",
            delete(workspaces::delete_invitation),
        )
        // ── Projects (Fase 2b) ───────────────────────────────────────────────
        .route(
            "/workspaces/:slug/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/workspaces/:slug/projects/:project_id",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/members",
            get(projects::list_project_members),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/members/:pk",
            patch(projects::update_project_member).delete(projects::remove_project_member),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/invitations",
            get(projects::list_project_invitations)
                .post(projects::create_project_invitations),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/invitations/:pk",
            delete(projects::delete_project_invitation),
        )
        // ── States (Fase 3) ──────────────────────────────────────────────────
        .route(
            "/workspaces/:slug/projects/:project_id/states",
            get(states::list_states).post(states::create_state),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/states/:pk",
            get(states::get_state)
                .patch(states::update_state)
                .delete(states::delete_state),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/intake-state",
            get(states::intake_state),
        )
        .route(
            "/workspaces/:slug/projects/:project_id/states/:pk/mark-default",
            post(states::mark_default),
        )
        .layer(middleware::from_fn(
            auth::rate_limit::rate_limit_headers_middleware,
        ));

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
