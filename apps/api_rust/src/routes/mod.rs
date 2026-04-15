// src/routes/mod.rs
use crate::{auth, AppState};
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Json, Router,
};
use tower_http::normalize_path::NormalizePathLayer;
use tower::ServiceBuilder;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

pub mod analytics;
pub mod assets;
pub mod cycles;
pub mod external;
pub mod exporter;
pub mod importer;
pub mod intake;
pub mod pages;
pub mod search;
pub mod estimates;
pub mod labels;
pub mod notifications;
pub mod webhooks;
pub mod health;
pub mod issue_extras;
pub mod issues;
pub mod modules;
pub mod helpers;
pub mod integrations;
pub mod projects;
pub mod states;
pub mod timezones;
pub mod users;
pub mod views;
pub mod workspaces;
pub mod workspace_extras;
pub mod issue_extras2;
pub mod api_tokens;
pub mod instances;

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
        auth::oauth::gitlab_initiate,
        auth::oauth::gitlab_callback,
        auth::oauth::google_initiate,
        auth::oauth::google_callback,
        auth::oauth::gitea_initiate,
        auth::oauth::gitea_callback,
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
        integrations::github::list_integrations,
        integrations::github::github_app_callback,
        integrations::github::github_user_callback,
        integrations::workspace::list_workspace_integrations,
        integrations::workspace::create_workspace_integration,
        integrations::workspace::get_workspace_integration,
        integrations::workspace::update_workspace_integration,
        integrations::workspace::delete_workspace_integration,
        integrations::workspace::delete_workspace_integration_by_provider,
        integrations::workspace::provider_install,
        integrations::github::list_github_repositories,
        integrations::gitlab::list_gitlab_repositories,
        integrations::github::list_github_repo_syncs,
        integrations::github::create_github_repo_sync,
        integrations::github::delete_github_repo_sync,
        integrations::pr_state::list_pr_state_mappings,
        integrations::pr_state::create_pr_state_mapping,
        integrations::pr_state::delete_pr_state_mapping,
        issues::list_issues,
        issues::create_issue,
        issues::get_issue,
        issues::update_issue,
        issues::delete_issue,
        cycles::list_cycles,
        cycles::create_cycle,
        cycles::get_cycle,
        cycles::update_cycle,
        cycles::delete_cycle,
        cycles::list_cycle_issues,
        cycles::add_issues_to_cycle,
        cycles::remove_issue_from_cycle,
        modules::list_modules,
        modules::create_module,
        modules::get_module,
        modules::update_module,
        modules::delete_module,
        modules::list_module_issues,
        modules::add_issues_to_module,
        modules::remove_issue_from_module,
        labels::list_labels,
        labels::create_label,
        labels::get_label,
        labels::update_label,
        labels::delete_label,
        estimates::list_estimates,
        estimates::create_estimate,
        estimates::get_estimate,
        estimates::update_estimate,
        estimates::delete_estimate,
        estimates::create_estimate_point,
        estimates::update_estimate_point,
        estimates::delete_estimate_point,
        notifications::list_notifications,
        notifications::get_notification,
        notifications::update_notification,
        notifications::delete_notification,
        notifications::mark_read,
        notifications::mark_unread,
        notifications::archive_notification,
        notifications::unarchive_notification,
        notifications::unread_count,
        notifications::mark_all_read,
        webhooks::list_webhooks,
        webhooks::create_webhook,
        webhooks::get_webhook,
        webhooks::update_webhook,
        webhooks::delete_webhook,
        webhooks::regenerate_secret,
        webhooks::list_webhook_logs,
        pages::list_pages,
        pages::create_page,
        pages::get_page,
        pages::update_page,
        pages::delete_page,
        pages::archive_page,
        pages::unarchive_page,
        pages::lock_page,
        pages::unlock_page,
        pages::duplicate_page,
        pages::list_page_versions,
        pages::get_page_version,
        intake::list_intakes,
        intake::create_intake,
        intake::get_intake,
        intake::update_intake,
        intake::delete_intake,
        intake::list_intake_issues,
        intake::create_intake_issue,
        intake::get_intake_issue,
        intake::update_intake_issue,
        intake::delete_intake_issue,
        exporter::export_issues,
        exporter::get_export_status,
        search::global_search,
        search::search_issues,
        timezones::list_timezones,
        users::get_me,
        users::update_me,
        users::deactivate_me,
        users::get_session,
        users::get_settings,
        users::get_instance_admin,
        users::update_onboard,
        users::update_tour_completed,
        users::get_profile,
        users::update_profile,
        users::list_accounts,
        users::get_account,
        users::delete_account,
        users::list_user_workspaces,
        views::list_workspace_views,
        views::create_workspace_view,
        views::get_workspace_view,
        views::update_workspace_view,
        views::delete_workspace_view,
        views::list_project_views,
        views::create_project_view,
        views::get_project_view,
        views::update_project_view,
        views::delete_project_view,
        views::add_favorite_view,
        views::remove_favorite_view,
        analytics::list_analytic_views,
        analytics::create_analytic_view,
        analytics::get_analytic_view,
        analytics::update_analytic_view,
        analytics::delete_analytic_view,
        analytics::get_saved_analytic_view,
        analytics::export_analytics,
        analytics::default_analytics,
        analytics::project_stats,
        analytics::workspace_analytics,
        assets::initiate_user_asset_upload,
        assets::complete_user_asset_upload,
        assets::delete_user_asset,
        assets::initiate_workspace_asset_upload,
        assets::complete_workspace_asset_upload,
        assets::delete_workspace_asset,
        assets::get_workspace_asset,
        assets::get_static_asset,
        importer::list_github_import_repositories,
        importer::list_github_importers,
        importer::create_github_importer,
        importer::delete_github_importer,
        importer::list_gitlab_import_repositories,
        importer::list_gitlab_importers,
        importer::create_gitlab_importer,
        importer::delete_gitlab_importer,
        external::unsplash,
        external::project_ai_assistant,
        external::workspace_ai_assistant,
        issue_extras::list_comments,
        issue_extras::create_comment,
        issue_extras::update_comment,
        issue_extras::delete_comment,
        issue_extras::list_issue_reactions,
        issue_extras::add_issue_reaction,
        issue_extras::remove_issue_reaction,
        issue_extras::add_comment_reaction,
        issue_extras::remove_comment_reaction,
        issue_extras::list_issue_links,
        issue_extras::create_issue_link,
        issue_extras::update_issue_link,
        issue_extras::delete_issue_link,
        issue_extras::list_issue_relations,
        issue_extras::create_issue_relation,
        issue_extras::remove_issue_relation,
        issue_extras::list_issue_activities,
        issue_extras::list_issue_subscribers,
        issue_extras::subscribe_to_issue,
        issue_extras::unsubscribe_from_issue,
        issue_extras::list_sub_issues,
        instances::get_instance,
        instances::patch_instance,
        instances::signup_screen_visited,
        instances::list_instance_admins,
        instances::create_instance_admin,
        instances::get_instance_admin_me,
        instances::get_instance_admin_session,
        instances::delete_instance_admin,
        instances::list_configurations,
        instances::update_configurations,
        instances::disable_email_feature,
        instances::email_credentials_check,
        instances::instance_workspace_slug_check,
        instances::list_instance_workspaces,
        auth::god_mode::admin_sign_up,
        auth::god_mode::admin_sign_in,
        auth::god_mode::admin_sign_out,
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
        (name = "Health",       description = "Health checks"),
        (name = "Instance",     description = "Instance configuration and feature flags"),
        (name = "Auth",         description = "Authentication"),
        (name = "Workspaces",   description = "Workspace management"),
        (name = "Projects",     description = "Project management"),
        (name = "States",       description = "Project state management"),
        (name = "Issues",       description = "Issues and work items"),
        (name = "Cycles",       description = "Sprint cycles"),
        (name = "Modules",      description = "Feature modules"),
        (name = "Labels",        description = "Project labels"),
        (name = "Estimates",     description = "Project estimates"),
        (name = "Notifications", description = "User notifications"),
        (name = "Webhooks",      description = "Workspace webhooks"),
        (name = "Pages",         description = "Project pages"),
        (name = "Intake",        description = "Issue intake / inbox"),
        (name = "Exporter",      description = "Issue export"),
        (name = "Search",        description = "Global and project search"),
        (name = "Analytics",    description = "Workspace analytics and project stats"),
        (name = "Assets",       description = "File assets — user/workspace/project uploads"),
        (name = "External",     description = "AI assistant and Unsplash integration"),
        (name = "Importer",     description = "GitHub and GitLab issue importers"),
        (name = "Timezones",     description = "Supported timezones"),
        (name = "Users",         description = "Current user profile and settings"),
        (name = "Views",         description = "Issue views (workspace & project)"),
        (name = "Integrations", description = "GitHub · GitLab · Slack integrations"),
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
    // ── Rutas sin autenticación ──────────────────────────────────────────────
    let public_routes = Router::new()
        // GitHub App Setup URL callback — sin middleware de auth
        .route("/github/callback/", get(integrations::github_app_callback))
        // GitLab OAuth callback — sin middleware de auth (manejado por frontend)
        .route("/auth/gitlab/callback/", get(auth::oauth::gitlab_callback))
        .route("/auth/google/callback/", get(auth::oauth::google_callback))
        .route("/auth/gitea/callback/", get(auth::oauth::gitea_callback));

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
            "/auth/reset-password/{uidb64}/{token}",
            post(auth::forgot_reset_password::reset_password),
        )
        .route(
            "/auth/spaces/forgot-password",
            post(auth::forgot_reset_password::forgot_password_space),
        )
        .route(
            "/auth/spaces/reset-password/{uidb64}/{token}",
            post(auth::forgot_reset_password::reset_password_space),
        )
        .route("/auth/sign-out", post(auth::logout::logout))
        .route("/auth/spaces/sign-out", post(auth::logout::logout_space))
        // ── GitHub user OAuth callback (con auth) ────────────────────────────
        .route(
            "/auth/github/user-callback/",
            post(integrations::github_user_callback),
        )
        // ── OAuth Initiation ──
        .route("/auth/gitlab/", get(auth::oauth::gitlab_initiate))
        .route("/auth/google/", get(auth::oauth::google_initiate))
        .route("/auth/gitea/", get(auth::oauth::gitea_initiate))
        // ── Integrations globales ────────────────────────────────────────────
        .route("/integrations/", get(integrations::list_integrations))
        // ── Workspaces (Fase 2) ──────────────────────────────────────────────
        // NormalizePathLayer (aplicado al router final) elimina trailing slashes
        // automáticamente, por lo que solo se necesita una variante por ruta.
        .route("/workspace-slug-check", get(workspaces::slug_check))
        .route(
            "/workspaces",
            get(workspaces::list_workspaces).post(workspaces::create_workspace),
        )
        .route(
            "/workspaces/{slug}",
            get(workspaces::get_workspace)
                .patch(workspaces::update_workspace)
                .delete(workspaces::delete_workspace),
        )
        .route("/workspaces/{slug}/members", get(workspaces::list_members))
        .route(
            "/workspaces/{slug}/members/{pk}",
            patch(workspaces::update_member).delete(workspaces::remove_member),
        )
        .route(
            "/workspaces/{slug}/invitations",
            get(workspaces::list_invitations).post(workspaces::create_invitations),
        )
        .route(
            "/workspaces/{slug}/invitations/{pk}",
            delete(workspaces::delete_invitation),
        )
        // ── Workspace integrations ───────────────────────────────────────────
        .route(
            "/workspaces/{slug}/workspace-integrations/",
            get(integrations::list_workspace_integrations)
                .post(integrations::create_workspace_integration),
        )
        // Rutas específicas de GitHub ANTES de las rutas genéricas con :pk
        // para evitar que "github" sea capturado como un UUID
        .route(
            "/workspaces/{slug}/workspace-integrations/github/repo-syncs/",
            get(integrations::list_github_repo_syncs)
                .post(integrations::create_github_repo_sync),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}/",
            delete(integrations::delete_github_repo_sync),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{pk}/",
            get(integrations::get_workspace_integration)
                .patch(integrations::update_workspace_integration)
                .delete(integrations::delete_workspace_integration),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{provider}/provider/",
            delete(integrations::delete_workspace_integration_by_provider),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{provider}/install/",
            post(integrations::provider_install),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/",
            get(integrations::list_github_repositories),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories/",
            get(integrations::list_gitlab_repositories),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/",
            get(integrations::list_pr_state_mappings)
                .post(integrations::create_pr_state_mapping),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/",
            delete(integrations::delete_pr_state_mapping),
        )
        // ── Projects (Fase 2b) ───────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/members",
            get(projects::list_project_members),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/members/{pk}",
            patch(projects::update_project_member).delete(projects::remove_project_member),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/invitations",
            get(projects::list_project_invitations)
                .post(projects::create_project_invitations),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/invitations/{pk}",
            delete(projects::delete_project_invitation),
        )
        // ── States (Fase 3) ──────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/states",
            get(states::list_states).post(states::create_state),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/states/{pk}",
            get(states::get_state)
                .patch(states::update_state)
                .delete(states::delete_state),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-state",
            get(states::intake_state),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default",
            post(states::mark_default),
        )
        // ── Issues ──────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/",
            get(issues::list_issues).post(issues::create_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
            get(issues::get_issue)
                .patch(issues::update_issue)
                .delete(issues::delete_issue),
        )
        // ── Cycles ──────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/",
            get(cycles::list_cycles).post(cycles::create_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{pk}/",
            get(cycles::get_cycle)
                .patch(cycles::update_cycle)
                .delete(cycles::delete_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/",
            get(cycles::list_cycle_issues).post(cycles::add_issues_to_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}/",
            delete(cycles::remove_issue_from_cycle),
        )
        // ── Modules ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/",
            get(modules::list_modules).post(modules::create_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{pk}/",
            get(modules::get_module)
                .patch(modules::update_module)
                .delete(modules::delete_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/",
            get(modules::list_module_issues).post(modules::add_issues_to_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}/",
            delete(modules::remove_issue_from_module),
        )
        // ── Labels ──────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/labels/",
            get(labels::list_labels).post(labels::create_label),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/labels/{pk}/",
            get(labels::get_label)
                .patch(labels::update_label)
                .delete(labels::delete_label),
        )
        // ── Estimates ────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/",
            get(estimates::list_estimates).post(estimates::create_estimate),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/",
            get(estimates::get_estimate)
                .patch(estimates::update_estimate)
                .delete(estimates::delete_estimate),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/",
            post(estimates::create_estimate_point),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}/",
            patch(estimates::update_estimate_point).delete(estimates::delete_estimate_point),
        )
        // ── Notifications ────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/users/notifications/unread/",
            get(notifications::unread_count),
        )
        .route(
            "/workspaces/{slug}/users/notifications/mark-all-read/",
            post(notifications::mark_all_read),
        )
        .route(
            "/workspaces/{slug}/users/notifications/",
            get(notifications::list_notifications),
        )
        .route(
            "/workspaces/{slug}/users/notifications/{pk}/",
            get(notifications::get_notification)
                .patch(notifications::update_notification)
                .delete(notifications::delete_notification),
        )
        .route(
            "/workspaces/{slug}/users/notifications/{pk}/read/",
            post(notifications::mark_read).delete(notifications::mark_unread),
        )
        .route(
            "/workspaces/{slug}/users/notifications/{pk}/archive/",
            post(notifications::archive_notification)
                .delete(notifications::unarchive_notification),
        )
        // ── Webhooks ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/webhooks/",
            get(webhooks::list_webhooks).post(webhooks::create_webhook),
        )
        .route(
            "/workspaces/{slug}/webhooks/{pk}/",
            get(webhooks::get_webhook)
                .patch(webhooks::update_webhook)
                .delete(webhooks::delete_webhook),
        )
        .route(
            "/workspaces/{slug}/webhooks/{pk}/regenerate/",
            post(webhooks::regenerate_secret),
        )
        .route(
            "/workspaces/{slug}/webhook-logs/{webhook_id}/",
            get(webhooks::list_webhook_logs),
        )
        // ── Pages ────────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/",
            get(pages::list_pages).post(pages::create_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/",
            get(pages::get_page)
                .patch(pages::update_page)
                .delete(pages::delete_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/",
            post(pages::archive_page).delete(pages::unarchive_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/",
            post(pages::lock_page).delete(pages::unlock_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate/",
            post(pages::duplicate_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/",
            get(pages::list_page_versions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}/",
            get(pages::get_page_version),
        )
        // ── Intake ───────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/intakes/",
            get(intake::list_intakes).post(intake::create_intake),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
            get(intake::get_intake)
                .patch(intake::update_intake)
                .delete(intake::delete_intake),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-issues/",
            get(intake::list_intake_issues).post(intake::create_intake_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
            get(intake::get_intake_issue)
                .patch(intake::update_intake_issue)
                .delete(intake::delete_intake_issue),
        )
        // ── Exporter ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/export-issues/",
            post(exporter::export_issues),
        )
        .route(
            "/workspaces/{slug}/export-issues/{token}/",
            get(exporter::get_export_status),
        )
        // ── Search ───────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/search/",
            get(search::global_search),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/search-issues/",
            get(search::search_issues),
        )
        // ── Timezones ─────────────────────────────────────────────────────────
        .route("/timezones/", get(timezones::list_timezones))
        // ── Users (me) ───────────────────────────────────────────────────────
        .route(
            "/users/me/",
            get(users::get_me)
                .patch(users::update_me)
                .delete(users::deactivate_me),
        )
        .route("/users/session/", get(users::get_session))
        .route("/users/me/settings/", get(users::get_settings))
        .route("/users/me/instance-admin/", get(users::get_instance_admin))
        .route("/users/me/onboard/", patch(users::update_onboard))
        .route(
            "/users/me/tour-completed/",
            patch(users::update_tour_completed),
        )
        .route(
            "/users/me/profile/",
            get(users::get_profile).patch(users::update_profile),
        )
        .route("/users/me/accounts/", get(users::list_accounts))
        .route(
            "/users/me/accounts/{pk}/",
            get(users::get_account).delete(users::delete_account),
        )
        .route("/users/me/workspaces/", get(users::list_user_workspaces))
        // ── Workspace Views ───────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/views/",
            get(views::list_workspace_views).post(views::create_workspace_view),
        )
        .route(
            "/workspaces/{slug}/views/{pk}/",
            get(views::get_workspace_view)
                .patch(views::update_workspace_view)
                .delete(views::delete_workspace_view),
        )
        // ── Project Views ─────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/views/",
            get(views::list_project_views).post(views::create_project_view),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/views/{pk}/",
            get(views::get_project_view)
                .patch(views::update_project_view)
                .delete(views::delete_project_view),
        )
        // ── View Favorites ────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-views/",
            post(views::add_favorite_view),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}/",
            delete(views::remove_favorite_view),
        )
        // ── Analytics ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/analytics/",
            get(analytics::workspace_analytics),
        )
        .route(
            "/workspaces/{slug}/default-analytics/",
            get(analytics::default_analytics),
        )
        .route(
            "/workspaces/{slug}/project-stats/",
            get(analytics::project_stats),
        )
        .route(
            "/workspaces/{slug}/export-analytics/",
            post(analytics::export_analytics),
        )
        .route(
            "/workspaces/{slug}/analytic-view/",
            get(analytics::list_analytic_views).post(analytics::create_analytic_view),
        )
        .route(
            "/workspaces/{slug}/analytic-view/{pk}/",
            get(analytics::get_analytic_view)
                .patch(analytics::update_analytic_view)
                .delete(analytics::delete_analytic_view),
        )
        .route(
            "/workspaces/{slug}/saved-analytic-view/{analytic_id}/",
            get(analytics::get_saved_analytic_view),
        )
        // ── Assets ───────────────────────────────────────────────────────────
        .route(
            "/assets/v2/user-assets/",
            post(assets::initiate_user_asset_upload),
        )
        .route(
            "/assets/v2/user-assets/{asset_id}/",
            patch(assets::complete_user_asset_upload)
                .delete(assets::delete_user_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}/",
            post(assets::initiate_workspace_asset_upload),
        )
        .route(
            "/assets/v2/workspaces/{slug}/{asset_id}/",
            get(assets::get_workspace_asset)
                .patch(assets::complete_workspace_asset_upload)
                .delete(assets::delete_workspace_asset),
        )
        .route(
            "/assets/v2/static/{asset_id}/",
            get(assets::get_static_asset),
        )
        // ── Importer ──────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/importers/github/repositories/",
            get(importer::list_github_import_repositories),
        )
        .route(
            "/workspaces/{slug}/importers/github/",
            get(importer::list_github_importers).post(importer::create_github_importer),
        )
        .route(
            "/workspaces/{slug}/importers/github/{importer_id}/",
            delete(importer::delete_github_importer),
        )
        .route(
            "/workspaces/{slug}/importers/gitlab/repositories/",
            get(importer::list_gitlab_import_repositories),
        )
        .route(
            "/workspaces/{slug}/importers/gitlab/",
            get(importer::list_gitlab_importers).post(importer::create_gitlab_importer),
        )
        .route(
            "/workspaces/{slug}/importers/gitlab/{importer_id}/",
            delete(importer::delete_gitlab_importer),
        )
        // ── External ──────────────────────────────────────────────────────────
        .route("/unsplash/", get(external::unsplash))
        .route(
            "/workspaces/{slug}/projects/{project_id}/ai-assistant/",
            post(external::project_ai_assistant),
        )
        .route(
            "/workspaces/{slug}/ai-assistant/",
            post(external::workspace_ai_assistant),
        )
        // ── Issue extras (comments, reactions, links, relations, history) ────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/",
            get(issue_extras::list_comments).post(issue_extras::create_comment),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/",
            patch(issue_extras::update_comment).delete(issue_extras::delete_comment),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/",
            get(issue_extras::list_issue_reactions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/",
            post(issue_extras::add_issue_reaction)
                .delete(issue_extras::remove_issue_reaction),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/",
            post(issue_extras::add_comment_reaction)
                .delete(issue_extras::remove_comment_reaction),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/",
            get(issue_extras::list_issue_links).post(issue_extras::create_issue_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}/",
            patch(issue_extras::update_issue_link)
                .delete(issue_extras::delete_issue_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation/",
            get(issue_extras::list_issue_relations)
                .post(issue_extras::create_issue_relation),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation/",
            delete(issue_extras::remove_issue_relation),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history/",
            get(issue_extras::list_issue_activities),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/",
            get(issue_extras::list_issue_subscribers),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/",
            post(issue_extras::subscribe_to_issue)
                .delete(issue_extras::unsubscribe_from_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues/",
            get(issue_extras::list_sub_issues),
        )
        // ── Workspace Extras (favorites, home prefs, quick links, recent visits, stickies) ──
        .route(
            "/workspaces/{slug}/user-favorites/",
            get(workspace_extras::list_favorites).post(workspace_extras::create_favorite),
        )
        .route(
            "/workspaces/{slug}/user-favorites/{favorite_id}/",
            patch(workspace_extras::update_favorite).delete(workspace_extras::delete_favorite),
        )
        .route(
            "/workspaces/{slug}/user-favorites/{favorite_id}/children/",
            get(workspace_extras::list_favorite_children),
        )
        .route(
            "/workspaces/{slug}/home-preference/",
            get(workspace_extras::get_home_preferences),
        )
        .route(
            "/workspaces/{slug}/home-preference/{key}/",
            patch(workspace_extras::update_home_preference),
        )
        .route(
            "/workspaces/{slug}/quick-links/",
            get(workspace_extras::list_quick_links).post(workspace_extras::create_quick_link),
        )
        .route(
            "/workspaces/{slug}/quick-links/{pk}/",
            patch(workspace_extras::update_quick_link).delete(workspace_extras::delete_quick_link),
        )
        .route(
            "/workspaces/{slug}/user-recent-visit/",
            get(workspace_extras::list_recent_visits),
        )
        .route(
            "/workspaces/{slug}/stickies/",
            get(workspace_extras::list_stickies).post(workspace_extras::create_sticky),
        )
        .route(
            "/workspaces/{slug}/stickies/{pk}/",
            patch(workspace_extras::update_sticky).delete(workspace_extras::delete_sticky),
        )
        .route(
            "/workspaces/{slug}/user-preference/",
            get(workspace_extras::get_user_preferences)
                .patch(workspace_extras::update_user_preferences),
        )
        // ── Draft Issues ──────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/draft-issues/",
            get(workspace_extras::list_draft_issues).post(workspace_extras::create_draft_issue),
        )
        .route(
            "/workspaces/{slug}/draft-issues/{pk}/",
            get(workspace_extras::get_draft_issue)
                .patch(workspace_extras::update_draft_issue)
                .delete(workspace_extras::delete_draft_issue),
        )
        // ── Workspace-level aggregate views ──────────────────────────────────
        .route(
            "/workspaces/{slug}/cycles/",
            get(workspace_extras::list_workspace_cycles),
        )
        .route(
            "/workspaces/{slug}/modules/",
            get(workspace_extras::list_workspace_modules),
        )
        .route(
            "/workspaces/{slug}/estimates/",
            get(workspace_extras::list_workspace_estimates),
        )
        .route(
            "/workspaces/{slug}/labels/",
            get(workspace_extras::list_workspace_labels),
        )
        .route(
            "/workspaces/{slug}/states/",
            get(workspace_extras::list_workspace_states),
        )
        // ── Issue attachments ─────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/",
            get(issue_extras2::list_issue_attachments)
                .post(issue_extras2::initiate_issue_attachment_upload),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/",
            patch(issue_extras2::complete_issue_attachment_upload)
                .delete(issue_extras2::delete_issue_attachment),
        )
        // ── Issue archive / unarchive ─────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/",
            post(issue_extras2::archive_issue).delete(issue_extras2::unarchive_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/bulk-archive-issues/",
            post(issue_extras2::bulk_archive_issues),
        )
        // ── Issue versions ────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/",
            get(issue_extras2::list_issue_versions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}/",
            get(issue_extras2::get_issue_version),
        )
        // ── API Tokens ────────────────────────────────────────────────────────
        .route(
            "/api-tokens/",
            get(api_tokens::list_api_tokens).post(api_tokens::create_api_token),
        )
        .route(
            "/api-tokens/{pk}/",
            get(api_tokens::get_api_token)
                .patch(api_tokens::update_api_token)
                .delete(api_tokens::delete_api_token),
        )
        // ── Instance ──────────────────────────────────────────────────────────
        .route(
            "/instances/",
            get(instances::get_instance).patch(instances::patch_instance),
        )
        // God Mode auth — rutas específicas ANTES de /admins/ para evitar conflictos
        .route(
            "/instances/admins/sign-up/",
            post(auth::god_mode::admin_sign_up),
        )
        .route(
            "/instances/admins/sign-in/",
            post(auth::god_mode::admin_sign_in),
        )
        .route(
            "/instances/admins/sign-out/",
            post(auth::god_mode::admin_sign_out),
        )
        .route(
            "/instances/admins/sign-up-screen-visited/",
            post(instances::signup_screen_visited),
        )
        .route(
            "/instances/admins/",
            get(instances::list_instance_admins).post(instances::create_instance_admin),
        )
        // Rutas específicas ANTES de /{pk}/ para evitar captura incorrecta
        .route("/instances/admins/me/",      get(instances::get_instance_admin_me))
        .route("/instances/admins/session/", get(instances::get_instance_admin_session))
        .route(
            "/instances/admins/{pk}/",
            delete(instances::delete_instance_admin),
        )
        // Configurations — disable-email-feature ANTES de la ruta raíz
        .route(
            "/instances/configurations/disable-email-feature/",
            delete(instances::disable_email_feature),
        )
        .route(
            "/instances/configurations/",
            get(instances::list_configurations).patch(instances::update_configurations),
        )
        .route(
            "/instances/email-credentials-check/",
            post(instances::email_credentials_check),
        )
        .route(
            "/instances/workspace-slug-check/",
            get(instances::instance_workspace_slug_check),
        )
        .route(
            "/instances/workspaces/",
            get(instances::list_instance_workspaces),
        )
        .layer(middleware::from_fn(
            auth::rate_limit::rate_limit_headers_middleware,
        ))
        .layer(DefaultBodyLimit::max(1_048_576)); // 1 MB — previene DoS por payload masivo

    let mut router = Router::new()
        .nest("/api", api_router)
        .nest("/api", public_routes);

    // ✅ Scalar UI solo en desarrollo (DEBUG=true).
    if state.config.debug {
        router = router
            .route("/api/docs/openapi.json", get(openapi_json))
            .merge(Scalar::with_url("/api/docs", ApiDoc::openapi()));
        tracing::warn!("Scalar UI habilitado (DEBUG=true) — deshabilitar en producción");
    }

    // NormalizePathLayer elimina trailing slashes antes del routing, evitando
    // 404 cuando el frontend envía /api/workspaces/ vs /api/workspaces.
    // Se aplica al router final para cubrir TODAS las rutas uniformemente.
    router
        .layer(ServiceBuilder::new().layer(NormalizePathLayer::trim_trailing_slash()))
        .with_state(state)
}
