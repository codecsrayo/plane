// src/routes/mod.rs
use crate::{auth, AppState};
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Json, Router,
};
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
pub mod issue_filters;
pub mod issue_pagination;
pub mod issues;
pub mod modules;
pub mod helpers;
pub mod integrations;
pub mod projects;
pub mod project_user_properties;
pub mod states;
pub mod timezones;
pub mod users;
pub mod views;
pub mod workspaces;
pub mod workspace_extras;
pub mod workspace_view_issues;
pub mod user_profile_issues;
pub mod issue_description_versions;
pub mod issue_extras2;
pub mod api_tokens;
pub mod instances;
pub mod v1_router;

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
        auth::oauth::github_initiate,
        auth::oauth::github_auth_callback,
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
        workspaces::get_member,
        workspaces::update_member,
        workspaces::remove_member,
        workspaces::leave_workspace,
        workspaces::get_project_members,
        workspaces::update_workspace_views,
        workspaces::get_workspace_views,
                workspaces::get_workspace_member_me,
        workspaces::get_user_profile,
        workspaces::get_user_stats,
        workspaces::get_workspace_user_activity,
        workspaces::export_workspace_user_activity,
        user_profile_issues::list_user_profile_issues,
        workspaces::list_invitations,
        workspaces::create_invitations,
        workspaces::get_invitation,
        workspaces::update_invitation,
        workspaces::delete_invitation,
        workspaces::join_workspace_invitation,
        workspaces::get_invitation_join,
        workspaces::list_workspace_themes,
        workspaces::create_workspace_theme,
        workspaces::get_workspace_theme,
        workspaces::update_workspace_theme,
        workspaces::delete_workspace_theme,
        projects::list_projects,
        projects::list_projects_detail,
        projects::create_project,
        projects::get_project,
        projects::update_project,
        projects::delete_project,
        projects::list_project_members,
        projects::get_project_member_me,
        project_user_properties::get_project_user_properties,
        project_user_properties::update_project_user_properties,
        projects::update_project_member,
        projects::remove_project_member,
        projects::list_project_invitations,
        projects::create_project_invitations,
        projects::delete_project_invitation,
        projects::get_project_invitation,
        projects::get_project_member,
        projects::create_project_members,
        projects::leave_project,
        projects::update_project_views,
        projects::get_project_user_views,
        projects::get_project_summary,
        projects::list_project_favorites,
        projects::create_project_favorite,
        projects::delete_project_favorite,
        projects::archive_project,
        projects::unarchive_project,
        projects::check_project_identifier,
        projects::delete_project_identifier,
        projects::get_project_deploy_board,
        projects::upsert_project_deploy_board,
        projects::update_project_deploy_board,
        projects::delete_project_deploy_board,
        projects::get_project_member_preferences,
        projects::update_project_member_preferences,
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
        issues::list_issues_by_ids,
        issues::list_issues_detail,
        issues::list_issues_v2,
        cycles::list_cycles,
        cycles::create_cycle,
        cycles::get_cycle,
        cycles::update_cycle,
        cycles::delete_cycle,
        cycles::list_cycle_issues,
        cycles::add_issues_to_cycle,
        cycles::remove_issue_from_cycle,
        cycles::cycle_analytics,
        cycles::cycle_progress,
        cycles::get_cycle_user_properties,
        cycles::update_cycle_user_properties,
        cycles::cycle_date_check,
        cycles::list_favorite_cycles,
        cycles::create_favorite_cycle,
        cycles::delete_favorite_cycle,
        cycles::transfer_cycle_issues,
        cycles::archive_cycle,
        cycles::unarchive_cycle,
        cycles::list_archived_cycles,
        cycles::get_archived_cycle,
        modules::list_modules,
        modules::create_module,
        modules::get_module,
        modules::update_module,
        modules::delete_module,
        modules::list_module_issues,
        modules::add_issues_to_module,
        modules::remove_issue_from_module,
        modules::get_module_user_properties,
        modules::update_module_user_properties,
        modules::set_issue_modules,
        modules::list_module_links,
        modules::create_module_link,
        modules::get_module_link,
        modules::update_module_link,
        modules::delete_module_link,
        modules::list_favorite_modules,
        modules::create_favorite_module,
        modules::delete_favorite_module,
        modules::archive_module,
        modules::unarchive_module,
        modules::list_archived_modules,
        modules::get_archived_module,
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
        estimates::list_project_estimates,
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
        notifications::get_user_notification_preferences,
        notifications::update_user_notification_preferences,
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
        pages::list_favorite_pages,
        pages::list_archived_pages,
        pages::move_page,
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
        exporter::list_export_issues,
        exporter::get_export_status,
        search::global_search,
        search::search_issues,
        search::entity_search,
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
        users::get_user_project_roles,
        users::list_user_workspace_invitations,
        users::join_user_workspace_invitations,
        users::get_my_activities,
        users::get_activity_graph,
        users::get_issues_completed_graph,
        users::get_workspace_dashboard,
        users::get_last_workspace,
        workspace_extras::draft_to_issue,
        workspace_view_issues::list_workspace_view_issues,
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
        views::list_user_favorite_views,
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
        analytics::advance_analytics,
        analytics::advance_analytics_stats,
        analytics::advance_analytics_charts,
        analytics::project_advance_analytics,
        analytics::project_advance_analytics_stats,
        analytics::project_advance_analytics_charts,
        assets::initiate_user_asset_upload,
        assets::complete_user_asset_upload,
        assets::delete_user_asset,
        assets::initiate_workspace_asset_upload,
        assets::complete_workspace_asset_upload,
        assets::delete_workspace_asset,
        assets::get_workspace_asset,
        assets::get_static_asset,
        assets::list_issue_attachments_v2,
        assets::initiate_issue_attachment_upload_v2,
        assets::complete_issue_attachment_upload_v2,
        assets::delete_issue_attachment_v2,
        assets::restore_workspace_asset,
        assets::initiate_project_asset_upload,
        assets::complete_project_asset_upload,
        assets::delete_project_asset,
        assets::get_project_asset,
        assets::bulk_project_assets,
        assets::check_workspace_asset,
        assets::duplicate_workspace_asset,
        assets::download_workspace_asset,
        assets::download_project_asset,
        assets::delete_legacy_workspace_file_asset,
        assets::restore_legacy_workspace_file_asset,
        assets::delete_legacy_user_file_asset,
        workspace_extras::list_workspace_active_cycles,
        importer::list_all_importers,
        users::generate_email_code,
        users::update_user_email,
        projects::join_project_invitation,
        projects::join_user_project_invitations,
        projects::list_user_project_invitations,
        external::github_webhook,
        external::gitlab_webhook,
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
        external::rephrase_grammar,
        issue_extras::list_comments,
        issue_extras::create_comment,
        issue_extras::update_comment,
        issue_extras::delete_comment,
        issue_extras::list_issue_reactions,
        issue_extras::add_issue_reaction,
        issue_extras::remove_issue_reaction,
        issue_extras::list_comment_reactions,
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
        issue_extras::get_issue_activity,
        issue_extras::list_issue_subscribers,
        issue_extras::subscribe_to_issue,
        issue_extras::unsubscribe_from_issue,
        issue_extras::delete_issue_subscriber,
        issue_extras::list_sub_issues,
        issue_extras::assign_sub_issues,
        issue_description_versions::list_description_versions,
        issue_description_versions::get_description_version,
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
            workspaces::WorkspaceMemberNestedResponse,
            workspaces::UserLiteDto,
            workspaces::WorkspaceMemberMeResponse,
            workspaces::UserProfileResponse,
            workspaces::UserProfileData,
            workspaces::ProjectProfileData,
            workspaces::UpdateMemberRoleRequest,
            workspaces::InvitationResponse,
            workspaces::CreateInvitationRequest,
            workspaces::InviteEmail,
            workspaces::SlugCheckResponse,
            workspaces::ExportUserActivityBody,
            projects::ProjectResponse,
            projects::ProjectListResponse,
            projects::ProjectDetailResponse,
            projects::CreateProjectRequest,
            projects::UpdateProjectRequest,
            projects::ProjectMemberResponse,
            projects::ProjectMemberMeResponse,
            projects::WorkspaceLiteDto,
            projects::ProjectLiteDto,
            project_user_properties::ProjectUserPropertyResponse,
            project_user_properties::UpdateProjectUserPropertyRequest,
            cycles::CycleUserPropertiesResponse,
            cycles::UpdateCycleUserPropertiesRequest,
            modules::ModuleUserPropertiesResponse,
            modules::UpdateModuleUserPropertiesRequest,
            projects::UpdateProjectMemberRequest,
            projects::ProjectInvitationResponse,
            projects::CreateProjectInvitationRequest,
            projects::ProjectInviteEmail,
            states::StateResponse,
            states::CreateStateRequest,
            states::UpdateStateRequest,
            timezones::TimezoneEntry,
            timezones::TimezonesResponse,
            notifications::UserNotificationPreferenceResponse,
            notifications::UpdateUserNotificationPreferenceRequest,
            external::RephraseGrammarRequest,
            external::RephraseGrammarResponse,
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

/// Adds the "TokenAuth" security scheme to the OpenAPI spec.
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
    // ── Public routes (no authentication) ────────────────────────────────────
    let public_routes = Router::new()
        // GitHub App Setup URL callback — without auth middleware
        .route("/github/callback", get(integrations::github_app_callback));

    // ── Auth routes — nested at /auth to match Django's path("auth/", ...) ──
    // Public auth routes (no auth middleware, e.g. OAuth callbacks)
    let auth_public_routes = Router::new()
        .route("/gitlab/callback", get(auth::oauth::gitlab_callback))
        .route("/google/callback", get(auth::oauth::google_callback))
        .route("/gitea/callback", get(auth::oauth::gitea_callback))
        // GitHub user OAuth callback (login flow - NOT the GitHub App setup).
        // The App setup callback lives at /github/callback (no /auth prefix).
        .route("/github/callback", get(auth::oauth::github_auth_callback));

    // Auth routes with rate limiting (mirrors plane.authentication.urls)
    let auth_router = Router::new()
        .route("/get-csrf-token", get(auth::csrf::get_csrf_token))
        .route("/sign-in", post(auth::email_auth::sign_in))
        .route("/sign-up", post(auth::email_auth::sign_up))
        .route(
            "/magic-generate",
            post(auth::magic_auth::magic_generate),
        )
        .route("/magic-sign-in", post(auth::magic_auth::magic_sign_in))
        .route("/magic-sign-up", post(auth::magic_auth::magic_sign_up))
        .route(
            "/spaces/sign-in",
            post(auth::email_auth::sign_in_space),
        )
        .route(
            "/spaces/sign-up",
            post(auth::email_auth::sign_up_space),
        )
        .route(
            "/spaces/magic-generate",
            post(auth::magic_auth::magic_generate_space),
        )
        .route(
            "/spaces/magic-sign-in",
            post(auth::magic_auth::magic_sign_in_space),
        )
        .route(
            "/spaces/magic-sign-up",
            post(auth::magic_auth::magic_sign_up_space),
        )
        .route("/email-check", post(auth::email_check::email_check))
        .route(
            "/spaces/email-check",
            post(auth::email_check::email_check_space),
        )
        .route(
            "/change-password",
            post(auth::password_management::change_password),
        )
        .route(
            "/set-password",
            post(auth::password_management::set_password),
        )
        .route(
            "/forgot-password",
            post(auth::forgot_reset_password::forgot_password),
        )
        .route(
            "/reset-password/{uidb64}/{token}",
            post(auth::forgot_reset_password::reset_password),
        )
        .route(
            "/spaces/forgot-password",
            post(auth::forgot_reset_password::forgot_password_space),
        )
        .route(
            "/spaces/reset-password/{uidb64}/{token}",
            post(auth::forgot_reset_password::reset_password_space),
        )
        .route("/sign-out", post(auth::logout::logout))
        .route("/spaces/sign-out", post(auth::logout::logout_space))
        // ── GitHub user OAuth callback (with auth) ───────────────────────────
        .route(
            "/github/user-callback",
            get(integrations::github_user_callback_get_stub)
                .post(integrations::github_user_callback),
        )
        // ── OAuth Initiation ──
        .route("/gitlab", get(auth::oauth::gitlab_initiate))
        .route("/google", get(auth::oauth::google_initiate))
        .route("/gitea", get(auth::oauth::gitea_initiate))
        .route("/github", get(auth::oauth::github_initiate))
        .layer(middleware::from_fn(
            auth::rate_limit::rate_limit_headers_middleware,
        ))
        .layer(DefaultBodyLimit::max(1_048_576));

    let api_router = Router::new()
        .route("/health", get(health::health))
        // Alias: frontend calls POST /api/auth/github/user-callback/ (prefixed with /api)
        // while auth_router registers it at /auth/github/user-callback (no /api prefix).
        .route(
            "/auth/github/user-callback",
            get(integrations::github_user_callback_get_stub)
                .post(integrations::github_user_callback),
        )
        // ── Global Integrations ─────────────────────────────────────────────
        .route("/integrations", get(integrations::list_integrations))
        // ── Workspaces (Phase 2) ──────────────────────────────────────────────
        // NormalizePathLayer (applied to the final router) removes trailing slashes
        // automatically, so only one variant per route is needed.
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
        .route(
            "/workspaces/{slug}/members",
            get(workspaces::list_members),
        )
        .route(
            "/workspaces/{slug}/members/leave",
            post(workspaces::leave_workspace),
        )
        .route(
            "/workspaces/{slug}/members/{pk}",
            get(workspaces::get_member)
                .patch(workspaces::update_member)
                .delete(workspaces::remove_member),
        )
        // Mirror Django: workspaces/<slug>/project-members/ -> WorkspaceProjectMemberEndpoint
        .route(
            "/workspaces/{slug}/project-members",
            get(workspaces::get_project_members),
        )
        // Mirror Django: workspaces/<slug>/workspace-views/ -> WorkspaceMemberUserViewsEndpoint
        .route(
            "/workspaces/{slug}/workspace-views",
            get(workspaces::get_workspace_views).post(workspaces::update_workspace_views),
        )
        // Mirror Django: workspaces/<slug>/workspace-members/me// -> WorkspaceMemberUserEndpoint
        .route(
            "/workspaces/{slug}/workspace-members/me",
            get(workspaces::get_workspace_member_me),
        )
        // Mirror Django: workspaces/<slug>/user-profile/<user_id>/ -> WorkspaceUserProfileEndpoint
        .route(
            "/workspaces/{slug}/user-profile/{user_id}",
            get(workspaces::get_user_profile),
        )
        // Mirror Django: workspaces/<slug>/user-stats/<user_id>/ -> WorkspaceUserProfileStatsEndpoint
        .route(
            "/workspaces/{slug}/user-stats/{user_id}",
            get(workspaces::get_user_stats),
        )
        // Mirror Django: workspaces/<slug>/user-activity/<user_id>/ -> WorkspaceUserActivityEndpoint
        .route(
            "/workspaces/{slug}/user-activity/{user_id}",
            get(workspaces::get_workspace_user_activity),
        )
        // Mirror Django: workspaces/<slug>/user-activity/<user_id>/export/ -> ExportWorkspaceUserActivityEndpoint
        // CSV download of the user activity log for a given date.
        .route(
            "/workspaces/{slug}/user-activity/{user_id}/export",
            get(workspaces::export_workspace_user_activity_get)
                .post(workspaces::export_workspace_user_activity),
        )
        // Mirror Django: workspaces/<slug>/user-issues/<user_id>/ -> WorkspaceUserProfileIssuesEndpoint
        // Serves the Assigned / Created / Subscribed tabs of the user profile.
        .route(
            "/workspaces/{slug}/user-issues/{user_id}",
            get(user_profile_issues::list_user_profile_issues),
        )
        .route(
            "/workspaces/{slug}/invitations",
            get(workspaces::list_invitations).post(workspaces::create_invitations),
        )
        .route(
            "/workspaces/{slug}/invitations/{pk}",
            get(workspaces::get_invitation)
                .patch(workspaces::update_invitation)
                .delete(workspaces::delete_invitation),
        )
        .route(
            "/workspaces/{slug}/invitations/{pk}/join",
            get(workspaces::get_invitation_join)
                .post(workspaces::join_workspace_invitation),
        )
        .route(
            "/workspaces/{slug}/workspace-themes",
            get(workspaces::list_workspace_themes).post(workspaces::create_workspace_theme),
        )
        .route(
            "/workspaces/{slug}/workspace-themes/{pk}",
            get(workspaces::get_workspace_theme)
                .patch(workspaces::update_workspace_theme)
                .delete(workspaces::delete_workspace_theme),
        )
        // ── Workspace integrations ───────────────────────────────────────────
        .route(
            "/workspaces/{slug}/workspace-integrations",
            get(integrations::list_workspace_integrations)
                .post(integrations::create_workspace_integration),
        )
        // GitHub specific routes BEFORE generic routes with :pk
        // to prevent "github" from being captured as a UUID
        .route(
            "/workspaces/{slug}/workspace-integrations/github/repo-syncs",
            get(integrations::list_github_repo_syncs)
                .post(integrations::create_github_repo_sync),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}",
            delete(integrations::delete_github_repo_sync),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{pk}",
            get(integrations::get_workspace_integration)
                .patch(integrations::update_workspace_integration)
                .delete(integrations::delete_workspace_integration),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{provider}/provider",
            delete(integrations::delete_workspace_integration_by_provider),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{provider}/install",
            post(integrations::provider_install),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories",
            get(integrations::list_github_repositories),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories",
            get(integrations::list_gitlab_repositories),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings",
            get(integrations::list_pr_state_mappings)
                .post(integrations::create_pr_state_mapping),
        )
        .route(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}",
            delete(integrations::delete_pr_state_mapping),
        )
        // ── Projects (Phase 2b) ───────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        // IMPORTANT: this literal route must come BEFORE /{project_id} so that
        // "details" is not interpreted as a project UUID.
        .route(
            "/workspaces/{slug}/projects/details",
            get(projects::list_projects_detail),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/members",
            get(projects::list_project_members)
                .post(projects::create_project_members),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/members/{pk}",
            get(projects::get_project_member)
                .patch(projects::update_project_member)
                .delete(projects::remove_project_member),
        )
        // Django URL: `workspaces/<slug>/projects/<project_id>/project-members/me/`
        // (`apps/api/plane/app/urls/project.py:98-100`). Returns the ProjectMember
        // of the authenticated user. The frontend (base-permissions.store) depends
        // on this endpoint to load project permissions.
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-members/me",
            get(projects::get_project_member_me),
        )
        // Mirror Django: /members/leave/ (literal before /{pk}/)
        .route(
            "/workspaces/{slug}/projects/{project_id}/members/leave",
            post(projects::leave_project),
        )
        // Mirror Django: /project-views/
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-views",
            get(projects::get_project_user_views).post(projects::update_project_views),
        )
        // Project summary — internal variant (no admin); v1 remains active.
        .route(
            "/workspaces/{slug}/projects/{project_id}/summary",
            get(projects::get_project_summary),
        )
        // Mirror Django: /archive/
        .route(
            "/workspaces/{slug}/projects/{project_id}/archive",
            post(projects::archive_project).delete(projects::unarchive_project),
        )
        // Mirror Django: /user-favorite-projects/
        .route(
            "/workspaces/{slug}/user-favorite-projects",
            get(projects::list_project_favorites).post(projects::create_project_favorite),
        )
        .route(
            "/workspaces/{slug}/user-favorite-projects/{project_id}",
            delete(projects::delete_project_favorite),
        )
        // Mirror Django: /project-identifiers/
        .route(
            "/workspaces/{slug}/project-identifiers",
            get(projects::check_project_identifier).delete(projects::delete_project_identifier),
        )
        // Mirror Django: workspaces/<slug>/projects/<project_id>/project-deploy-boards/
        // (`apps/api/plane/app/urls/project.py:113-120`). GET→list, POST→upsert deploy board.
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-deploy-boards",
            get(projects::get_project_deploy_board)
                .post(projects::upsert_project_deploy_board),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}",
            get(projects::get_project_deploy_board)
                .patch(projects::update_project_deploy_board)
                .delete(projects::delete_project_deploy_board),
        )
        // Mirror Django: workspaces/<slug>/projects/<project_id>/preferences/member/<member_id>/
        // (`apps/api/plane/app/urls/project.py:128`). GET/PATCH member preferences JSON.
        .route(
            "/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}",
            get(projects::get_project_member_preferences)
                .patch(projects::update_project_member_preferences),
        )
        // Django URL: `workspaces/<slug>/projects/<project_id>/user-properties/`
        // (`apps/api/plane/app/urls/issue.py:216-219`). GET performs get_or_create
        // so it NEVER returns 404 if the project exists.
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-properties",
            get(project_user_properties::get_project_user_properties)
                .patch(project_user_properties::update_project_user_properties),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/invitations",
            get(projects::list_project_invitations)
                .post(projects::create_project_invitations),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/invitations/{pk}",
            get(projects::get_project_invitation)
                .delete(projects::delete_project_invitation),
        )
        // ── States (Phase 3) ──────────────────────────────────────────────────
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
            "/workspaces/{slug}/projects/{project_id}/issues",
            get(issues::list_issues).post(issues::create_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/list",
            get(issues::list_issues_by_ids),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues-detail",
            get(issues::list_issues_detail),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/v2/issues",
            get(issues::list_issues_v2),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{pk}",
            get(issues::get_issue)
                .patch(issues::update_issue)
                .delete(issues::delete_issue),
        )
        // ── Cycles ──────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles",
            get(cycles::list_cycles).post(cycles::create_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{pk}",
            get(cycles::get_cycle)
                .patch(cycles::update_cycle)
                .delete(cycles::delete_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues",
            get(cycles::list_cycle_issues).post(cycles::add_issues_to_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}",
            delete(cycles::remove_issue_from_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics",
            get(cycles::cycle_analytics),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties",
            get(cycles::get_cycle_user_properties)
                .patch(cycles::update_cycle_user_properties),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress",
            get(cycles::cycle_progress),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/date-check",
            post(cycles::cycle_date_check),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-cycles",
            get(cycles::list_favorite_cycles).post(cycles::create_favorite_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-cycles/{cycle_id}",
            delete(cycles::delete_favorite_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues",
            post(cycles::transfer_cycle_issues),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive",
            post(cycles::archive_cycle).delete(cycles::unarchive_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-cycles",
            get(cycles::list_archived_cycles),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}",
            get(cycles::get_archived_cycle).delete(cycles::unarchive_cycle),
        )
        // ── Modules ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules",
            get(modules::list_modules).post(modules::create_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{pk}",
            get(modules::get_module)
                .patch(modules::update_module)
                .delete(modules::delete_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues",
            get(modules::list_module_issues).post(modules::add_issues_to_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}",
            delete(modules::remove_issue_from_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties",
            get(modules::get_module_user_properties)
                .patch(modules::update_module_user_properties),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/modules",
            post(modules::set_issue_modules),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links",
            get(modules::list_module_links).post(modules::create_module_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}",
            get(modules::get_module_link)
                .patch(modules::update_module_link)
                .delete(modules::delete_module_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-modules",
            get(modules::list_favorite_modules).post(modules::create_favorite_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-modules/{module_id}",
            delete(modules::delete_favorite_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive",
            post(modules::archive_module).delete(modules::unarchive_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-modules",
            get(modules::list_archived_modules),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-modules/{pk}",
            get(modules::get_archived_module).delete(modules::unarchive_module),
        )
        // ── Labels ──────────────────────────────────────────────────────────
        // Canonical routes (/labels/)
        .route(
            "/workspaces/{slug}/projects/{project_id}/bulk-create-labels",
            post(labels::bulk_create_labels),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/labels",
            get(labels::list_labels).post(labels::create_label),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/labels/{pk}",
            get(labels::get_label)
                .patch(labels::update_label)
                .delete(labels::delete_label),
        )
        // Django-compatible alias (/issue-labels/) – same handlers
        .route(
            "/workspaces/{slug}/projects/{project_id}/issue-labels",
            get(labels::list_labels).post(labels::create_label),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issue-labels/{pk}",
            get(labels::get_label)
                .patch(labels::update_label)
                .delete(labels::delete_label),
        )
        // ── Estimates ────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-estimates",
            get(estimates::list_project_estimates),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates",
            get(estimates::list_estimates).post(estimates::create_estimate),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}",
            get(estimates::get_estimate)
                .patch(estimates::update_estimate)
                .delete(estimates::delete_estimate),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points",
            post(estimates::create_estimate_point),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}",
            patch(estimates::update_estimate_point).delete(estimates::delete_estimate_point),
        )
        // ── Notifications ────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/users/notifications/unread",
            get(notifications::unread_count),
        )
        .route(
            "/workspaces/{slug}/users/notifications/mark-all-read",
            post(notifications::mark_all_read),
        )
        .route(
            "/workspaces/{slug}/users/notifications",
            get(notifications::list_notifications),
        )
        .route(
            "/workspaces/{slug}/users/notifications/{pk}",
            get(notifications::get_notification)
                .patch(notifications::update_notification)
                .delete(notifications::delete_notification),
        )
        .route(
            "/workspaces/{slug}/users/notifications/{pk}/read",
            post(notifications::mark_read).delete(notifications::mark_unread),
        )
        .route(
            "/workspaces/{slug}/users/notifications/{pk}/archive",
            post(notifications::archive_notification)
                .delete(notifications::unarchive_notification),
        )
        // ── Webhooks ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/webhooks",
            get(webhooks::list_webhooks).post(webhooks::create_webhook),
        )
        .route(
            "/workspaces/{slug}/webhooks/{pk}",
            get(webhooks::get_webhook)
                .patch(webhooks::update_webhook)
                .delete(webhooks::delete_webhook),
        )
        .route(
            "/workspaces/{slug}/webhooks/{pk}/regenerate",
            post(webhooks::regenerate_secret),
        )
        .route(
            "/workspaces/{slug}/webhook-logs/{webhook_id}",
            get(webhooks::list_webhook_logs),
        )
        // ── Pages ────────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages-summary",
            get(pages::pages_summary),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages",
            get(pages::list_pages).post(pages::create_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}",
            get(pages::get_page)
                .patch(pages::update_page)
                .delete(pages::delete_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/access",
            post(pages::update_page_access),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/favorite-pages",
            get(pages::list_favorite_pages),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/favorite-pages/{page_id}",
            post(pages::add_page_favorite).delete(pages::remove_page_favorite),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-pages",
            get(pages::list_archived_pages),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive",
            post(pages::archive_page).delete(pages::unarchive_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/move",
            post(pages::move_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock",
            post(pages::lock_page).delete(pages::unlock_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate",
            post(pages::duplicate_page),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions",
            get(pages::list_page_versions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}",
            get(pages::get_page_version),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/pages/{page_id}/description",
            get(pages::get_page_description).patch(pages::update_page_description),
        )
        // ── Intake ───────────────────────────────────────────────────────────
        // Django registers both names (`intakes/` / `inboxes/`,
        // `intake-issues/` / `inbox-issues/`) pointing to the same
        // ViewSet — see apps/api/plane/app/urls/intake.py:17-55. The
        // current frontend uses the legacy name `inbox-issues`
        // (apps/web/core/services/inbox/inbox-issue.service.ts) and therefore
        // returned 404 when creating intake issues until these aliases
        // were added. All handlers are shared; there is no divergence.
        .route(
            "/workspaces/{slug}/projects/{project_id}/intakes",
            get(intake::list_intakes).post(intake::create_intake),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intakes/{pk}",
            get(intake::get_intake)
                .patch(intake::update_intake)
                .delete(intake::delete_intake),
        )
        // Alias legacy (Django: name="inbox").
        .route(
            "/workspaces/{slug}/projects/{project_id}/inboxes",
            get(intake::list_intakes).post(intake::create_intake),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/inboxes/{pk}",
            get(intake::get_intake)
                .patch(intake::update_intake)
                .delete(intake::delete_intake),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-issues",
            get(intake::list_intake_issues).post(intake::create_intake_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}",
            get(intake::get_intake_issue)
                .patch(intake::update_intake_issue)
                .delete(intake::delete_intake_issue),
        )
        // Legacy alias (Django: name="inbox-issue"). The current frontend
        // uses this URL in apps/web/core/services/inbox/inbox-issue.service.ts.
        .route(
            "/workspaces/{slug}/projects/{project_id}/inbox-issues",
            get(intake::list_intake_issues).post(intake::create_intake_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/inbox-issues/{pk}",
            get(intake::get_intake_issue)
                .patch(intake::update_intake_issue)
                .delete(intake::delete_intake_issue),
        )
        // ── Exporter ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/export-issues",
            get(exporter::list_export_issues).post(exporter::export_issues),
        )
        .route(
            "/workspaces/{slug}/export-issues/{token}",
            get(exporter::get_export_status),
        )
        // ── Search ───────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/search",
            get(search::global_search),
        )
        // Workspace-level search for work-items. Reuses global_search with
        // symmetric contract (200 with auth; 401 without). Frontend mirror.
        .route(
            "/workspaces/{slug}/work-items/search",
            get(search::global_search),
        )
        // Legacy alias for the `issues/` prefix — the old frontend continued
        // consuming it with the same shape; maintaining parity avoids regressions.
        .route(
            "/workspaces/{slug}/issues/search",
            get(search::global_search),
        )
        .route(
            "/workspaces/{slug}/issues/{combined}",
            get(issue_extras2::get_issue_by_identifier),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/search-issues",
            get(search::search_issues),
        )
        .route(
            "/workspaces/{slug}/entity-search",
            get(search::entity_search),
        )
        // ── Timezones ─────────────────────────────────────────────────────────
        .route("/timezones", get(timezones::list_timezones))
        // ── Users (me) ───────────────────────────────────────────────────────
        .route(
            "/users/me",
            get(users::get_me)
                .patch(users::update_me)
                .delete(users::deactivate_me),
        )
        .route("/users/session", get(users::get_session))
        .route("/users/me/settings", get(users::get_settings))
        .route("/users/me/instance-admin", get(users::get_instance_admin))
        // Mirror Django: users/me/notification-preferences/
        //   → UserNotificationPreferenceEndpoint
        //   (apps/api/plane/app/urls/notification.py:47-51)
        //
        // GET/PATCH over the single authenticated user preference row.
        // If the row does not exist (migrated users or users created
        // through flows that do not trigger the Django signal), it is created
        // with model defaults. Avoids the 500 that Django would have with `.get()`.
        .route(
            "/users/me/notification-preferences",
            get(notifications::get_user_notification_preferences)
                .patch(notifications::update_user_notification_preferences),
        )
        .route("/users/me/onboard", patch(users::update_onboard))
        .route(
            "/users/me/tour-completed",
            patch(users::update_tour_completed),
        )
        .route(
            "/users/me/profile",
            get(users::get_profile).patch(users::update_profile),
        )
        .route("/users/me/accounts", get(users::list_accounts))
        .route(
            "/users/me/accounts/{pk}",
            get(users::get_account).delete(users::delete_account),
        )
        .route("/users/last-visited-workspace", get(users::get_last_workspace))
        .route("/users/me/workspaces", get(users::list_user_workspaces))
        // Mirror Django: users/me/activities/ -> UserActivityEndpoint
        // (plane/app/urls/user.py:65, plane/app/views/user/base.py:380).
        // Returns all IssueActivity for the requester (cross-workspace)
        // with Django-style cursor pagination.
        .route("/users/me/activities", get(users::get_my_activities))
        // Mirror Django: users/me/workspaces/invitations/ -> UserWorkspaceInvitationsViewSet
        // (GET list pending invites, POST bulk-accept). Called by the frontend
        // onboarding flow.
        .route(
            "/users/me/workspaces/invitations",
            get(users::list_user_workspace_invitations)
                .post(users::join_user_workspace_invitations),
        )
        // Mirror Django: users/me/workspaces/<slug>/project-roles/ -> UserProjectRolesEndpoint
        .route(
            "/users/me/workspaces/{slug}/project-roles",
            get(users::get_user_project_roles),
        )
        .route(
            "/users/me/workspaces/{slug}/activity-graph",
            get(users::get_activity_graph),
        )
        .route(
            "/users/me/workspaces/{slug}/issues-completed-graph",
            get(users::get_issues_completed_graph),
        )
        .route(
            "/users/me/workspaces/{slug}/dashboard",
            get(users::get_workspace_dashboard),
        )
        // ── Workspace View Issues (global view / spreadsheet) ─────────────────
        // Mirror Django: workspaces/<slug>/issues/ → WorkspaceViewIssuesViewSet
        // (plane/app/urls/views.py:52). Returns issues from all workspace
        // projects the user has access to.
        .route(
            "/workspaces/{slug}/issues",
            get(workspace_view_issues::list_workspace_view_issues),
        )
        // ── Workspace Views ───────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/views",
            get(views::list_workspace_views).post(views::create_workspace_view),
        )
        .route(
            "/workspaces/{slug}/views/{pk}",
            get(views::get_workspace_view)
                .patch(views::update_workspace_view)
                .delete(views::delete_workspace_view),
        )
        // ── Project Views ─────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/views",
            get(views::list_project_views).post(views::create_project_view),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/views/{pk}",
            get(views::get_project_view)
                .patch(views::update_project_view)
                .delete(views::delete_project_view),
        )
        // ── View Favorites ────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-views",
            get(views::list_user_favorite_views).post(views::add_favorite_view),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}",
            delete(views::remove_favorite_view),
        )
        // ── Analytics ─────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/analytics",
            get(analytics::workspace_analytics),
        )
        .route(
            "/workspaces/{slug}/default-analytics",
            get(analytics::default_analytics),
        )
        .route(
            "/workspaces/{slug}/project-stats",
            get(analytics::project_stats),
        )
        .route(
            "/workspaces/{slug}/export-analytics",
            post(analytics::export_analytics),
        )
        .route(
            "/workspaces/{slug}/analytic-view",
            get(analytics::list_analytic_views).post(analytics::create_analytic_view),
        )
        .route(
            "/workspaces/{slug}/analytic-view/{pk}",
            get(analytics::get_analytic_view)
                .patch(analytics::update_analytic_view)
                .delete(analytics::delete_analytic_view),
        )
        .route(
            "/workspaces/{slug}/saved-analytic-view/{analytic_id}",
            get(analytics::get_saved_analytic_view),
        )
        // ── Advance Analytics ─────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/advance-analytics",
            get(analytics::advance_analytics),
        )
        .route(
            "/workspaces/{slug}/advance-analytics-stats",
            get(analytics::advance_analytics_stats),
        )
        .route(
            "/workspaces/{slug}/advance-analytics-charts",
            get(analytics::advance_analytics_charts),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/advance-analytics",
            get(analytics::project_advance_analytics),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/advance-analytics-stats",
            get(analytics::project_advance_analytics_stats),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/advance-analytics-charts",
            get(analytics::project_advance_analytics_charts),
        )
        // ── Assets ───────────────────────────────────────────────────────────
        .route(
            "/assets/v2/user-assets",
            post(assets::initiate_user_asset_upload),
        )
        .route(
            "/assets/v2/user-assets/{asset_id}",
            patch(assets::complete_user_asset_upload)
                .delete(assets::delete_user_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}",
            post(assets::initiate_workspace_asset_upload),
        )
        .route(
            "/assets/v2/workspaces/{slug}/{asset_id}",
            get(assets::get_workspace_asset)
                .patch(assets::complete_workspace_asset_upload)
                .delete(assets::delete_workspace_asset),
        )
        .route(
            "/assets/v2/static/{asset_id}",
            get(assets::get_static_asset),
        )
        // Issue attachments V2 (mirror Django IssueAttachmentV2Endpoint)
        // URL Django: apps/api/plane/app/urls/issue.py:137-146
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments",
            get(assets::list_issue_attachments_v2)
                .post(assets::initiate_issue_attachment_upload_v2),
        )
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}",
            patch(assets::complete_issue_attachment_upload_v2)
                .delete(assets::delete_issue_attachment_v2),
        )
        // Alias for work-items serviceType (same handlers)
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments",
            get(assets::list_issue_attachments_v2)
                .post(assets::initiate_issue_attachment_upload_v2),
        )
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments/{pk}",
            patch(assets::complete_issue_attachment_upload_v2)
                .delete(assets::delete_issue_attachment_v2),
        )
        // Restore, project assets, check, duplicate, download
        .route(
            "/assets/v2/workspaces/{slug}/restore/{asset_id}",
            post(assets::restore_workspace_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}/check/{asset_id}",
            get(assets::check_workspace_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}",
            post(assets::duplicate_workspace_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}/download/{asset_id}",
            get(assets::download_workspace_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}",
            post(assets::initiate_project_asset_upload),
        )
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}",
            get(assets::get_project_asset)
                .patch(assets::complete_project_asset_upload)
                .delete(assets::delete_project_asset),
        )
        .route(
            "/assets/v2/workspaces/{slug}/{entity_id}/bulk",
            post(assets::bulk_workspace_assets),
        )
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk",
            post(assets::bulk_project_assets),
        )
        .route(
            "/assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}",
            get(assets::download_project_asset),
        )
        // Legacy V1 file-asset endpoints (pre-v2, still called by FileService)
        .route(
            "/workspaces/file-assets/{workspace_id}/{asset_key}",
            delete(assets::delete_legacy_workspace_file_asset),
        )
        .route(
            "/workspaces/file-assets/{workspace_id}/{asset_key}/restore",
            post(assets::restore_legacy_workspace_file_asset),
        )
        .route(
            "/users/file-assets/{asset_key}",
            delete(assets::delete_legacy_user_file_asset),
        )
        // User email update
        .route("/users/me/email/generate-code", post(users::generate_email_code))
        .route("/users/me/email", post(users::update_user_email).patch(users::update_user_email))
        // Project join (public) + user project invitations
        .route(
            "/workspaces/{slug}/projects/{project_id}/join/{pk}",
            post(projects::join_project_invitation),
        )
        .route(
            "/users/me/workspaces/{slug}/projects/invitations",
            get(projects::list_user_project_invitations)
                .post(projects::join_user_project_invitations),
        )
        // Incoming webhooks (public)
        .route("/github-webhook", post(external::github_webhook))
        .route("/gitlab-webhook", post(external::gitlab_webhook))
        // ── Importer ──────────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/importers/github/repositories",
            get(importer::list_github_import_repositories),
        )
        .route(
            "/workspaces/{slug}/importers/github",
            get(importer::list_github_importers).post(importer::create_github_importer),
        )
        .route(
            "/workspaces/{slug}/importers/github/{importer_id}",
            delete(importer::delete_github_importer),
        )
        .route(
            "/workspaces/{slug}/importers/gitlab/repositories",
            get(importer::list_gitlab_import_repositories),
        )
        .route(
            "/workspaces/{slug}/importers/gitlab",
            get(importer::list_gitlab_importers).post(importer::create_gitlab_importer),
        )
        .route(
            "/workspaces/{slug}/importers/gitlab/{importer_id}",
            delete(importer::delete_gitlab_importer),
        )
        // Generic importers list — returns all services (github + gitlab)
        // Frontend: IntegrationService.getImporterServicesList
        .route(
            "/workspaces/{slug}/importers",
            get(importer::list_all_importers),
        )
        // -- External -------------------------------------------------------
        .route("/unsplash", get(external::unsplash))
        .route(
            "/workspaces/{slug}/projects/{project_id}/ai-assistant",
            post(external::project_ai_assistant),
        )
        .route(
            "/workspaces/{slug}/ai-assistant",
            post(external::workspace_ai_assistant),
        )
        // Mirror Django: workspaces/<slug>/rephrase-grammar/ → RephraseGrammarEndpoint
        // (apps/api/plane/app/urls/external.py). Critical for the page editor
        // (Ask Pi + text rephrasing). Bug tracked in todo.md.
        .route(
            "/workspaces/{slug}/rephrase-grammar",
            post(external::rephrase_grammar),
        )
        // ── Issue extras (comments, reactions, links, relations, history) ────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments",
            get(issue_extras::list_comments).post(issue_extras::create_comment),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}",
            get(issue_extras::get_comment)
                .patch(issue_extras::update_comment)
                .delete(issue_extras::delete_comment),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions",
            get(issue_extras::list_issue_reactions)
                .post(issue_extras::add_issue_reaction),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}",
            delete(issue_extras::remove_issue_reaction),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions",
            get(issue_extras::list_comment_reactions)
                .post(issue_extras::add_comment_reaction),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}",
            delete(issue_extras::remove_comment_reaction),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links",
            get(issue_extras::list_issue_links).post(issue_extras::create_issue_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}",
            patch(issue_extras::update_issue_link)
                .delete(issue_extras::delete_issue_link),
        )
        // Legacy frontend alias (short path `/links` without `issue-` prefix).
        // Covers historical issues/ shape and the new work-items/.
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links",
            get(issue_extras::list_issue_links).post(issue_extras::create_issue_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links/{pk}",
            patch(issue_extras::update_issue_link)
                .delete(issue_extras::delete_issue_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links",
            get(issue_extras::list_issue_links).post(issue_extras::create_issue_link),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links/{pk}",
            patch(issue_extras::update_issue_link)
                .delete(issue_extras::delete_issue_link),
        )
        // Activities by PK (404 if not found). Short path for the new frontend.
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/{pk}",
            get(issue_extras::get_issue_activity),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}",
            get(issue_extras::get_issue_activity),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities",
            get(issue_extras::list_issue_activities),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation",
            get(issue_extras::list_issue_relations)
                .post(issue_extras::create_issue_relation),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation",
            // Django Parity: apps/api/plane/app/urls/issue.py:241-242 routes
            // **POST** to `IssueRelationViewSet.remove_relation`. Previously
            // this was registered with `delete()`, which returned 405 to the
            // frontend (which sends POST) and broke the integration test.
            post(issue_extras::remove_issue_relation),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history",
            get(issue_extras::list_issue_activities),
        )
        // Alias `/activities` → same handler as `/history`. Django only
        // exposes `/history` (`apps/api/plane/app/urls/issue.py:150`), but
        // the public v1 router (`src/routes/v1_router.rs:551`) and the
        // recent frontend use `/activities`. Keeping both paths under
        // a single handler avoids logic duplication and unifies the contract:
        // /history continues to work for strict Django-parity clients,
        // /activities for clients following the v1 shape.
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities",
            get(issue_extras::list_issue_activities),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers",
            get(issue_extras::list_issue_subscribers),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}",
            delete(issue_extras::delete_issue_subscriber),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe",
            get(issue_extras::get_issue_subscription_status)
                .post(issue_extras::subscribe_to_issue)
                .delete(issue_extras::unsubscribe_from_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues",
            get(issue_extras::list_sub_issues).post(issue_extras::assign_sub_issues),
        )
        // ── Workspace Extras (favorites, home prefs, quick links, recent visits, stickies) ──
        .route(
            "/workspaces/{slug}/user-favorites",
            get(workspace_extras::list_favorites).post(workspace_extras::create_favorite),
        )
        .route(
            "/workspaces/{slug}/user-favorites/{favorite_id}",
            patch(workspace_extras::update_favorite).delete(workspace_extras::delete_favorite),
        )
        .route(
            "/workspaces/{slug}/user-favorites/{favorite_id}/children",
            get(workspace_extras::list_favorite_children),
        )
        // Mirror Django: user-favorites/<favorite_id>/group/ (alias de /children)
        // (`apps/api/plane/app/urls/workspace.py:198-200`)
        .route(
            "/workspaces/{slug}/user-favorites/{favorite_id}/group",
            get(workspace_extras::list_favorite_children),
        )
        .route(
            "/workspaces/{slug}/home-preferences",
            get(workspace_extras::get_home_preferences),
        )
        .route(
            "/workspaces/{slug}/home-preferences/{key}",
            get(workspace_extras::get_home_preference_key)
                .patch(workspace_extras::update_home_preference),
        )
        .route(
            "/workspaces/{slug}/quick-links",
            get(workspace_extras::list_quick_links).post(workspace_extras::create_quick_link),
        )
        .route(
            "/workspaces/{slug}/quick-links/{pk}",
            patch(workspace_extras::update_quick_link).delete(workspace_extras::delete_quick_link),
        )
        .route(
            "/workspaces/{slug}/recent-visits",
            get(workspace_extras::list_recent_visits),
        )
        .route(
            "/workspaces/{slug}/stickies",
            get(workspace_extras::list_stickies).post(workspace_extras::create_sticky),
        )
        .route(
            "/workspaces/{slug}/stickies/{pk}",
            patch(workspace_extras::update_sticky).delete(workspace_extras::delete_sticky),
        )
        .route(
            "/workspaces/{slug}/sidebar-preferences",
            get(workspace_extras::get_user_preferences)
                .patch(workspace_extras::update_user_preferences),
        )
        // Mirror Django: workspaces/<slug>/user-properties/
        //   → WorkspaceUserPropertiesEndpoint (plane/app/urls/workspace.py:162)
        .route(
            "/workspaces/{slug}/user-properties",
            get(workspace_extras::get_workspace_user_properties)
                .patch(workspace_extras::update_workspace_user_properties),
        )
        // ── Draft Issues ──────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/draft-issues",
            get(workspace_extras::list_draft_issues).post(workspace_extras::create_draft_issue),
        )
        .route(
            "/workspaces/{slug}/draft-to-issue/{draft_id}",
            post(workspace_extras::draft_to_issue),
        )
        .route(
            "/workspaces/{slug}/draft-issues/{pk}",
            get(workspace_extras::get_draft_issue)
                .patch(workspace_extras::update_draft_issue)
                .delete(workspace_extras::delete_draft_issue),
        )
        // ── Workspace-level aggregate views ──────────────────────────────────
        .route(
            "/workspaces/{slug}/cycles",
            get(workspace_extras::list_workspace_cycles),
        )
        // Active cycles across all projects -- paginated, sidebar widget
        // Frontend: CycleService.workspaceActiveCycles
        .route(
            "/workspaces/{slug}/active-cycles",
            get(workspace_extras::list_workspace_active_cycles),
        )
        .route(
            "/workspaces/{slug}/modules",
            get(workspace_extras::list_workspace_modules),
        )
        .route(
            "/workspaces/{slug}/estimates",
            get(workspace_extras::list_workspace_estimates),
        )
        .route(
            "/workspaces/{slug}/labels",
            get(workspace_extras::list_workspace_labels),
        )
        .route(
            "/workspaces/{slug}/states",
            get(workspace_extras::list_workspace_states),
        )
        // ── Issue attachments ─────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments",
            get(issue_extras2::list_issue_attachments)
                .post(issue_extras2::initiate_issue_attachment_upload),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}",
            patch(issue_extras2::complete_issue_attachment_upload)
                .delete(issue_extras2::delete_issue_attachment),
        )
        // ── Issue archive / unarchive ─────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive",
            get(issue_extras2::get_archived_issue)
                .post(issue_extras2::archive_issue)
                .delete(issue_extras2::unarchive_issue),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-issues",
            get(issue_extras2::list_archived_issues),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/deleted-issues",
            get(issue_extras2::list_deleted_issues),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/bulk-delete-issues",
            // Django exposes this endpoint as `def delete(...)` (HTTP DELETE),
            // but the Plane frontend and many proxies/CDNs do not reliably
            // support DELETE-with-body (RFC 9110 allows it but defines
            // the body as "no semantic meaning" -> some middleboxes discard it).
            // We accept BOTH methods on the same handler to provide strict
            // Django parity (DELETE) and maintain operational compatibility
            // with clients sending POST.
            delete(issue_extras2::bulk_delete_issues)
                .post(issue_extras2::bulk_delete_issues),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta",
            get(issue_extras2::get_issue_meta),
        )
        .route(
            "/workspaces/{slug}/work-items/{combined}",
            get(issue_extras2::get_issue_by_identifier),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/bulk-archive-issues",
            post(issue_extras2::bulk_archive_issues),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/bulk-operation-issues",
            post(issue_extras2::bulk_operation_issues),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issue-dates",
            post(issue_extras2::bulk_update_issue_dates),
        )
        // ── Issue versions ────────────────────────────────────────────────────
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions",
            get(issue_extras2::list_issue_versions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}",
            get(issue_extras2::get_issue_version),
        )
        // ── Work item description versions ────────────────────────────────────
        // Mirror Django: WorkItemDescriptionVersionEndpoint
        // (apps/api/plane/app/urls/issue.py:267-274)
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions",
            get(issue_description_versions::list_description_versions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}",
            get(issue_description_versions::get_description_version),
        )
        // Alias de intake-work-items: mirror Django IntakeWorkItemDescriptionVersionEndpoint
        // (apps/api/plane/app/urls/intake.py:57-65). Mismo handler que work-items.
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions",
            get(issue_description_versions::list_description_versions),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions/{pk}",
            get(issue_description_versions::get_description_version),
        )
        // ── API Tokens ────────────────────────────────────────────────────────
        .route(
            "/api-tokens",
            get(api_tokens::list_api_tokens).post(api_tokens::create_api_token),
        )
        .route(
            "/api-tokens/{pk}",
            get(api_tokens::get_api_token)
                .patch(api_tokens::update_api_token)
                .delete(api_tokens::delete_api_token),
        )
        // Mirror Django: users/api-tokens/ (alias de /api-tokens/)
        // (`apps/api/plane/app/urls/api.py:11-18`)
        .route(
            "/users/api-tokens",
            get(api_tokens::list_api_tokens).post(api_tokens::create_api_token),
        )
        .route(
            "/users/api-tokens/{pk}",
            get(api_tokens::get_api_token)
                .patch(api_tokens::update_api_token)
                .delete(api_tokens::delete_api_token),
        )
        // ── Instance ──────────────────────────────────────────────────────────
        .route(
            "/instances",
            get(instances::get_instance).patch(instances::patch_instance),
        )
        // God Mode auth — specific routes BEFORE /admins/ to avoid conflicts
        .route(
            "/instances/admins/sign-up",
            post(auth::god_mode::admin_sign_up),
        )
        .route(
            "/instances/admins/sign-in",
            post(auth::god_mode::admin_sign_in),
        )
        .route(
            "/instances/admins/sign-out",
            post(auth::god_mode::admin_sign_out),
        )
        .route(
            "/instances/admins/sign-up-screen-visited",
            post(instances::signup_screen_visited),
        )
        .route(
            "/instances/admins",
            get(instances::list_instance_admins).post(instances::create_instance_admin),
        )
        // Specific routes BEFORE /{pk}/ to avoid incorrect capture
        .route("/instances/admins/me",      get(instances::get_instance_admin_me))
        .route("/instances/admins/session", get(instances::get_instance_admin_session))
        .route(
            "/instances/admins/{pk}",
            delete(instances::delete_instance_admin),
        )
        // Configurations — disable-email-feature BEFORE the root route
        .route(
            "/instances/configurations/disable-email-feature",
            delete(instances::disable_email_feature),
        )
        .route(
            "/instances/configurations",
            get(instances::list_configurations).patch(instances::update_configurations),
        )
        .route(
            "/instances/email-credentials-check",
            post(instances::email_credentials_check),
        )
        .route(
            "/instances/workspace-slug-check",
            get(instances::instance_workspace_slug_check),
        )
        .route(
            "/instances/workspaces",
            get(instances::list_instance_workspaces),
        )
        .layer(middleware::from_fn(
            auth::rate_limit::rate_limit_headers_middleware,
        ))
        .layer(DefaultBodyLimit::max(1_048_576)); // 1 MB — prevents DoS by massive payload

    // ✅ Scalar UI only in development (DEBUG=true).
    //
    // FIX (axum 0.8): Scalar routes are merged into the root router BEFORE
    // the `nest("/api", ...)` calls. Previously they were added after and returned 404:
    // `nest("/api", api_router)` internally registers a wildcard that
    // captured `/api/docs`, routed to `api_router` (where `/docs` does
    // not exist) and responded with 404 without falling into the
    // `.route("/api/docs", ...)` of the root router.
    //
    // By registering Scalar first, the static route `/api/docs` remains as
    // more specific than the nest catch-all and axum prioritizes it correctly.
    //
    // Also: docs does NOT pass through the rate-limit middleware (applied to the
    // api_router), which is correct — we don't want rate-limiting on docs.
    let root = if state.config.debug {
        tracing::warn!("Scalar UI enabled (DEBUG=true) — disable in production");
        Router::new()
            .route("/api/docs/openapi.json", get(openapi_json))
            .merge(Scalar::with_url("/api/docs", ApiDoc::openapi()))
    } else {
        Router::new()
    };

    // Public api/v1 router with API key authentication (x-api-key).
    // Mirror of Django path("api/v1/", include("plane.api.urls")).
    // Mounted BEFORE /api so that the more specific prefix wins.
    let v1 = v1_router::v1_router(state.clone());

    let router = root
        .nest("/api/v1", v1)
        .nest("/api", api_router)
        .nest("/api", public_routes)
        // Auth routes at /auth/* — matches Django: path("auth/", include("plane.authentication.urls"))
        // Caddy routes /auth/* to the API server, frontend calls /auth/email-check/ etc.
        .nest("/auth", auth_router)
        .nest("/auth", auth_public_routes);

    // NormalizePathLayer is applied in main.rs wrapping the Router *from
    // the outside* with `NormalizePathLayer::trim_trailing_slash().layer(router)`.
    //
    // Reason: `Router::layer()` in axum 0.8 executes the middleware AFTER
    // path-matching, so the trailing slash is not stripped in time.
    // Wrapping externally, the layer runs BEFORE routing.
    router.with_state(state)
}
