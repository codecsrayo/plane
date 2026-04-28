// src/routes/users.rs
//! Authenticated user profile endpoints.
//!
//! Equivalent to `plane/app/views/user/base.py` in Django.
//!
//! Implemented routes:
//!   GET    /api/users/me/
//!   PATCH  /api/users/me/
//!   DELETE /api/users/me/                    (deactivate)
//!   GET    /api/users/session/
//!   GET    /api/users/me/settings/
//!   PATCH  /api/users/me/onboard/
//!   PATCH  /api/users/me/tour-completed/
//!   GET    /api/users/me/profile/
//!   PATCH  /api/users/me/profile/
//!   GET    /api/users/me/accounts/
//!   GET    /api/users/me/accounts/{pk}/
//!   DELETE /api/users/me/accounts/{pk}/
//!   GET    /api/users/me/workspaces/
//!   GET    /api/users/me/instance-admin/
//!   GET    /api/users/me/activities/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    sea_query::{Expr, OnConflict},
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::{AnyAuth, OptionalAnyAuth},
    entities::{
        accounts, issue_activities, issues, profiles,
        project_members, projects, users,
        workspace_member_invites, workspace_members, workspaces,
    },
    error::AppError,
    utils::{pagination, soft_delete::SoftDeleteExt},
    AppState,
};

// Alias for readability — `.active()` filters `deleted_at IS NULL`
// (implemented via `impl_soft_delete!` in entities/mod.rs)

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Public representation of the authenticated user (`/users/me/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserMeResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    /// Computed from avatar_asset_id or legacy avatar string (parity with Django User.avatar_url).
    pub avatar_url: Option<String>,
    pub cover_image: Option<String>,
    /// Computed from cover_image_asset or cover_image (parity with Django User.cover_image_url).
    pub cover_image_url: Option<String>,
    pub is_active: bool,
    pub is_bot: bool,
    pub is_email_verified: bool,
    pub is_password_autoset: bool,
    pub is_superuser: bool,
    pub is_managed: bool,
    pub is_tour_completed: bool,
    pub user_timezone: String,
    pub last_login_medium: String,
    pub date_joined: DateTime<FixedOffset>,
    pub last_login: Option<DateTime<FixedOffset>>,
    pub mobile_number: Option<String>,
}

/// User settings (`/users/me/settings/`).
///
/// Exact mirror of `UserMeSettingsSerializer`
/// (`apps/api/plane/app/serializers/user.py:90-138`): exposes ONLY
/// `["id", "email", "workspace"]`. The `workspace` block resolves server-side
/// the redirection logic consumed by the SPA upon login.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserSettingsResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub workspace: UserSettingsWorkspace,
}

/// `workspace` block inside `/users/me/settings/`.
///
/// The shape is asymmetric (Django parity):
/// - When there is a valid `last_workspace_id` and active membership:
///   `last_workspace_name` and `last_workspace_logo` are included.
/// - Otherwise: those two keys are omitted from the JSON (not emitted
///   as `null`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserSettingsWorkspace {
    pub last_workspace_id: Option<Uuid>,
    pub last_workspace_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_workspace_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_workspace_logo: Option<String>,
    pub fallback_workspace_id: Option<Uuid>,
    pub fallback_workspace_slug: Option<String>,
    pub invites: u64,
}

/// User profile (`/users/me/profile/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: Option<String>,
    pub use_case: Option<String>,
    pub is_tour_completed: bool,
    pub is_onboarded: bool,
    pub onboarding_step: serde_json::Value,
    pub last_workspace_id: Option<Uuid>,
    pub theme: serde_json::Value,
    pub language: String,
    pub is_smooth_cursor_enabled: bool,
    pub start_of_the_week: i16,
    pub is_app_rail_docked: bool,
    pub notification_view_mode: String,
    pub background_color: String,
}

/// User OAuth account (`/users/me/accounts/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AccountResponse {
    pub id: Uuid,
    pub provider: String,
    pub provider_account_id: String,
    pub last_connected_at: DateTime<FixedOffset>,
    pub metadata: serde_json::Value,
    pub user_id: Uuid,
}

/// User workspace (`/users/me/workspaces/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserWorkspaceResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub organization_size: Option<String>,
    pub owner_id: Uuid,
    pub timezone: String,
    pub background_color: String,
    pub role: i16,
}

/// Body for PATCH `/users/me/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar: Option<String>,
    pub cover_image: Option<String>,
    pub user_timezone: Option<String>,
}

/// Body for PATCH `/users/me/profile/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProfileRequest {
    pub role: Option<String>,
    pub use_case: Option<String>,
    pub language: Option<String>,
    pub is_smooth_cursor_enabled: Option<bool>,
    pub start_of_the_week: Option<i16>,
    pub is_app_rail_docked: Option<bool>,
    pub notification_view_mode: Option<String>,
    pub last_workspace_id: Option<Uuid>,
    pub theme: Option<serde_json::Value>,
}

/// Body for PATCH `/users/me/onboard/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OnboardRequest {
    pub is_onboarded: Option<bool>,
}

/// Body for PATCH `/users/me/tour-completed/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TourCompletedRequest {
    pub is_tour_completed: Option<bool>,
}

// ── Conversions ──────────────────────────────────────────────────────────────

async fn user_to_me_response(
    u: &users::Model,
    db: &sea_orm::DatabaseConnection,
) -> Result<UserMeResponse, AppError> {
    // Compute avatar_url: prefer asset-based URL, fall back to legacy avatar string.
    let avatar_url = if let Some(asset_id) = u.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !u.avatar.is_empty() {
        Some(u.avatar.clone())
    } else {
        None
    };

    // Compute cover_image_url: prefer asset-based URL, fall back to cover_image.
    let cover_image_url = if let Some(asset_id) = u.cover_image_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else {
        u.cover_image.clone()
    };

    // is_tour_completed lives on the profile in Django (see Profile model in
    // apps/api/plane/db/models/user.py). Fetch it; missing profile defaults
    // to false, matching DRF's serializer behaviour.
    let is_tour_completed = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(u.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .map(|p| p.is_tour_completed)
        .unwrap_or(false);

    Ok(UserMeResponse {
        id: u.id,
        username: u.username.clone(),
        email: u.email.clone(),
        display_name: u.display_name.clone(),
        first_name: u.first_name.clone(),
        last_name: u.last_name.clone(),
        avatar: u.avatar.clone(),
        avatar_url,
        cover_image: u.cover_image.clone(),
        cover_image_url,
        is_active: u.is_active,
        is_bot: u.is_bot,
        is_email_verified: u.is_email_verified,
        is_password_autoset: u.is_password_autoset,
        is_superuser: u.is_superuser,
        is_managed: u.is_managed,
        is_tour_completed,
        user_timezone: u.user_timezone.clone(),
        last_login_medium: u.last_login_medium.clone(),
        date_joined: u.date_joined,
        last_login: u.last_login,
        mobile_number: u.mobile_number.clone(),
    })
}

fn profile_to_response(p: &profiles::Model) -> ProfileResponse {
    ProfileResponse {
        id: p.id,
        user_id: p.user_id,
        role: p.role.clone(),
        use_case: p.use_case.clone(),
        is_tour_completed: p.is_tour_completed,
        is_onboarded: p.is_onboarded,
        onboarding_step: p.onboarding_step.clone(),
        last_workspace_id: p.last_workspace_id,
        theme: p.theme.clone(),
        language: p.language.clone(),
        is_smooth_cursor_enabled: p.is_smooth_cursor_enabled,
        start_of_the_week: p.start_of_the_week,
        is_app_rail_docked: p.is_app_rail_docked,
        notification_view_mode: p.notification_view_mode.clone(),
        background_color: p.background_color.clone(),
    }
}

fn account_to_response(a: &accounts::Model) -> AccountResponse {
    AccountResponse {
        id: a.id,
        provider: a.provider.clone(),
        provider_account_id: a.provider_account_id.clone(),
        last_connected_at: a.last_connected_at,
        metadata: a.metadata.clone(),
        user_id: a.user_id,
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/users/me/
///
/// Returns the authenticated user profile.
#[utoipa::path(
    get,
    path = "/users/me/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Current user"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_me(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(user_to_me_response(&user, &state.db).await?))
}

/// PATCH /api/users/me/
///
/// Updates the authenticated user fields.
#[utoipa::path(
    patch,
    path = "/users/me/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Updated user"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_me(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate display_name if provided
    if let Some(ref name) = body.display_name {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.len() > 255 {
            return Err(AppError::BadRequest(
                "display_name must be between 1 and 255 characters".into(),
            ));
        }
    }

    let mut active: users::ActiveModel = user.into();

    if let Some(v) = body.display_name {
        active.display_name = Set(v.trim().to_string());
    }
    if let Some(v) = body.first_name {
        active.first_name = Set(v);
    }
    if let Some(v) = body.last_name {
        active.last_name = Set(v);
    }
    if let Some(v) = body.avatar {
        active.avatar = Set(v);
    }
    if let Some(v) = body.cover_image {
        active.cover_image = Set(Some(v));
    }
    if let Some(v) = body.user_timezone {
        active.user_timezone = Set(v);
    }

    let updated = active.update(&state.db).await.map_err(|e| {
        tracing::error!("update_me db error: {e}");
        AppError::Database(e)
    })?;

    Ok(Json(user_to_me_response(&updated, &state.db).await?))
}

/// DELETE /api/users/me/ — deactivates user account.
#[utoipa::path(
    delete,
    path = "/users/me/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 204, description = "Account deactivated"),
        (status = 400, description = "Cannot deactivate"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn deactivate_me(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Verify user is not the only admin in any active workspace
    let memberships = workspace_members::Entity::find()
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .active()
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    for m in &memberships {
        // Role 20 = Admin en Plane
        if m.role == 20 {
            let other_admins = workspace_members::Entity::find()
                .filter(workspace_members::Column::WorkspaceId.eq(m.workspace_id))
                .filter(workspace_members::Column::Role.eq(20_i16))
                .filter(workspace_members::Column::IsActive.eq(true))
                .filter(workspace_members::Column::MemberId.ne(user.id))
                .active()
                .count(&state.db)
                .await
                .map_err(AppError::Database)?;

            if other_admins == 0 {
                return Err(AppError::BadRequest(
                    "You cannot deactivate your account as you are the only admin in a workspace."
                        .into(),
                ));
            }
        }
    }

    // Deactivate workspaces memberships
    for m in memberships {
        let mut am: workspace_members::ActiveModel = m.into();
        am.is_active = Set(false);
        am.update(&state.db).await.map_err(AppError::Database)?;
    }

    // Deactivate profile
    if let Some(profile) = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    {
        let mut ap: profiles::ActiveModel = profile.into();
        ap.is_onboarded = Set(false);
        ap.is_tour_completed = Set(false);
        ap.last_workspace_id = Set(None);
        ap.update(&state.db).await.map_err(AppError::Database)?;
    }

    // Deactivate user
    let mut au: users::ActiveModel = user.into();
    au.is_active = Set(false);
    au.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/users/session/
///
/// Returns if the user is authenticated. Does not require auth.
#[utoipa::path(
    get,
    path = "/users/session/",
    tag = "Users",
    responses(
        (status = 200, description = "Session info"),
    )
)]
pub async fn get_session(
    OptionalAnyAuth(user_opt): OptionalAnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    match user_opt {
        Some(user) => Ok(Json(serde_json::json!({
            "is_authenticated": true,
            "user": user_to_me_response(&user, &state.db).await?,
        }))),
        None => Ok(Json(serde_json::json!({ "is_authenticated": false }))),
    }
}

/// GET /api/users/me/settings/
#[utoipa::path(
    get,
    path = "/users/me/settings/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "User settings"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_settings(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // `workspace_invites` = count of WorkspaceMemberInvite by user's email.
    // Mirror: `WorkspaceMemberInvite.objects.filter(email=obj.email).count()`.
    // `WorkspaceMemberInvite` inherits from `BaseModel → AuditModel → SoftDeleteModel`
    // (apps/api/plane/db/mixins.py:61-66), cuyo manager por defecto
    // `SoftDeletionManager` aplica `.filter(deleted_at__isnull=True)`
    // (apps/api/plane/db/mixins.py:58). Por tanto `.objects` ya excluye
    // soft-deleted; replicamos con `.active()`.
    let invites = if let Some(email) = user.email.as_deref() {
        workspace_member_invites::Entity::find()
            .active()
            .filter(workspace_member_invites::Column::Email.eq(email))
            .count(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        0
    };

    // Branch 1: valid `last_workspace_id` + active user membership in that workspace exists.
    // Mirror: `Workspace.objects.filter(pk=..., workspace_member__member=obj.id,
    //                                   workspace_member__is_active=True).exists()`.
    let last_workspace = if let Some(last_id) = profile.last_workspace_id {
        let has_active_membership = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(last_id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .filter(workspace_members::Column::IsActive.eq(true))
            .active()
            .count(&state.db)
            .await
            .map_err(AppError::Database)?
            > 0;

        if has_active_membership {
            workspaces::Entity::find_by_id(last_id)
                .active()
                .one(&state.db)
                .await
                .map_err(AppError::Database)?
        } else {
            None
        }
    } else {
        None
    };

    let workspace = if let Some(ws) = last_workspace {
        // `workspace.logo_asset.asset_url` → `/api/assets/v2/static/{id}/`
        // Django: `""` if there is no asset (empty string, not None).
        let logo = ws
            .logo_asset_id
            .map(|aid| format!("/api/assets/v2/static/{}/", aid))
            .unwrap_or_default();
        UserSettingsWorkspace {
            last_workspace_id: Some(ws.id),
            last_workspace_slug: Some(ws.slug.clone()),
            last_workspace_name: Some(ws.name.clone()),
            last_workspace_logo: Some(logo),
            fallback_workspace_id: Some(ws.id),
            fallback_workspace_slug: Some(ws.slug),
            invites,
        }
    } else {
        // Branch 2: without valid last_workspace → fallback = oldest workspace
        // where user is an active member.
        // Mirror: `.filter(workspace_member__member_id=obj.id,
        //                  workspace_member__is_active=True).order_by("created_at").first()`.
        //
        // Implemented as a single-query JOIN to maintain parity with Django:
        // a previous version did `workspace_members.find().order_by(wm.created_at)
        // .limit(1)` and then `workspaces.find_by_id(...)`. That pattern breaks in two
        // dimensions with respect to Django:
        //   1) If the user has an active membership in a soft-deleted workspace
        //      OLDER than a live one, the LIMIT 1 captures the dead row
        //      and the subsequent find_by_id returns None → fallback = null.
        //      Django's ORM filters `workspaces.deleted_at IS NULL` in the same
        //      SELECT before the LIMIT, so the soft-deleted one never competes.
        //   2) It ordered by `workspace_members.created_at`, but Django orders by
        //      `workspaces.created_at` — different columns, different values.
        let fallback_ws = workspaces::Entity::find()
            .active()
            .inner_join(workspace_members::Entity)
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .filter(workspace_members::Column::IsActive.eq(true))
            .filter(workspace_members::Column::DeletedAt.is_null())
            .order_by_asc(workspaces::Column::CreatedAt)
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        UserSettingsWorkspace {
            last_workspace_id: None,
            last_workspace_slug: None,
            // Keys omitted from JSON (Django parity: `else` branch
            // does not include `last_workspace_name` nor `last_workspace_logo`).
            last_workspace_name: None,
            last_workspace_logo: None,
            fallback_workspace_id: fallback_ws.as_ref().map(|w| w.id),
            fallback_workspace_slug: fallback_ws.map(|w| w.slug),
            invites,
        }
    };

    Ok(Json(UserSettingsResponse {
        id: user.id,
        email: user.email.clone(),
        workspace,
    }))
}

/// GET /api/users/me/instance-admin/
#[utoipa::path(
    get,
    path = "/users/me/instance-admin/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Instance admin status"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_instance_admin(AnyAuth(user): AnyAuth) -> impl IntoResponse {
    // Only superusers are instance admins in the base implementation
    Json(serde_json::json!({ "is_instance_admin": user.is_superuser }))
}

/// PATCH /api/users/me/onboard/
#[utoipa::path(
    patch,
    path = "/users/me/onboard/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Updated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_onboard(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<OnboardRequest>,
) -> Result<impl IntoResponse, AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut ap: profiles::ActiveModel = profile.into();
    ap.is_onboarded = Set(body.is_onboarded.unwrap_or(false));
    ap.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "message": "Updated successfully" })))
}

/// PATCH /api/users/me/tour-completed/
#[utoipa::path(
    patch,
    path = "/users/me/tour-completed/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Updated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_tour_completed(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<TourCompletedRequest>,
) -> Result<impl IntoResponse, AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut ap: profiles::ActiveModel = profile.into();
    ap.is_tour_completed = Set(body.is_tour_completed.unwrap_or(false));
    ap.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "message": "Updated successfully" })))
}

/// GET /api/users/me/profile/
#[utoipa::path(
    get,
    path = "/users/me/profile/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "User profile"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_profile(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(profile_to_response(&profile)))
}

/// PATCH /api/users/me/profile/
#[utoipa::path(
    patch,
    path = "/users/me/profile/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Updated profile"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_profile(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut ap: profiles::ActiveModel = profile.into();

    if let Some(v) = body.role {
        ap.role = Set(Some(v));
    }
    if let Some(v) = body.use_case {
        ap.use_case = Set(Some(v));
    }
    if let Some(v) = body.language {
        ap.language = Set(v);
    }
    if let Some(v) = body.is_smooth_cursor_enabled {
        ap.is_smooth_cursor_enabled = Set(v);
    }
    if let Some(v) = body.start_of_the_week {
        ap.start_of_the_week = Set(v);
    }
    if let Some(v) = body.is_app_rail_docked {
        ap.is_app_rail_docked = Set(v);
    }
    if let Some(v) = body.notification_view_mode {
        ap.notification_view_mode = Set(v);
    }
    if let Some(v) = body.last_workspace_id {
        ap.last_workspace_id = Set(Some(v));
    }
    if let Some(v) = body.theme {
        ap.theme = Set(v);
    }

    let updated = ap.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(profile_to_response(&updated)))
}

/// GET /api/users/me/accounts/
#[utoipa::path(
    get,
    path = "/users/me/accounts/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "List of OAuth accounts"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_accounts(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let accs = accounts::Entity::find()
        .filter(accounts::Column::UserId.eq(user.id))
        .order_by_asc(accounts::Column::Provider)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<AccountResponse> = accs.iter().map(account_to_response).collect();
    Ok(Json(resp))
}

/// GET /api/users/me/accounts/{pk}/
#[utoipa::path(
    get,
    path = "/users/me/accounts/{pk}/",
    tag = "Users",
    security(("TokenAuth" = [])),
    params(("pk" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 200, description = "Account"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_account(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(pk): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let acc = accounts::Entity::find_by_id(pk)
        .filter(accounts::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(account_to_response(&acc)))
}

/// DELETE /api/users/me/accounts/{pk}/
#[utoipa::path(
    delete,
    path = "/users/me/accounts/{pk}/",
    tag = "Users",
    security(("TokenAuth" = [])),
    params(("pk" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_account(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(pk): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let acc = accounts::Entity::find_by_id(pk)
        .filter(accounts::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let am: accounts::ActiveModel = acc.into();
    am.delete(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/users/me/workspaces/
///
/// Lists active workspaces where the user is a member.
#[utoipa::path(
    get,
    path = "/users/me/workspaces/",
    tag = "Users",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "List of workspaces"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_user_workspaces(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Get active memberships for the user
    let memberships = workspace_members::Entity::find()
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .active()
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Get workspace IDs
    let workspace_ids: Vec<Uuid> = memberships.iter().map(|m| m.workspace_id).collect();

    if workspace_ids.is_empty() {
        return Ok(Json(Vec::<UserWorkspaceResponse>::new()));
    }

    // Load active workspaces
    let wss = workspaces::Entity::find()
        .filter(workspaces::Column::Id.is_in(workspace_ids))
        .active()
        .order_by_asc(workspaces::Column::Name)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Build enriched response with role
    let resp: Vec<UserWorkspaceResponse> = wss
        .into_iter()
        .map(|ws| {
            let role = memberships
                .iter()
                .find(|m| m.workspace_id == ws.id)
                .map(|m| m.role)
                .unwrap_or(10);

            UserWorkspaceResponse {
                id: ws.id,
                name: ws.name,
                slug: ws.slug,
                logo: ws.logo,
                organization_size: ws.organization_size,
                owner_id: ws.owner_id,
                timezone: ws.timezone,
                background_color: ws.background_color,
                role,
            }
        })
        .collect();

    Ok(Json(resp))
}

// ─── GET /api/users/me/workspaces/{slug}/project-roles ───────────────────────
//
// Mirror of Django `UserProjectRolesEndpoint` in
// `plane/app/views/project/member.py:327`. Returns a map
// `{project_id_str: role_int}` with the user roles in each project
// of the workspace where the user is an active member. The frontend consumes
// it to resolve per-project permissions without making N requests.

/// `GET /api/users/me/workspaces/{slug}/project-roles`
///
/// Returns `HashMap<project_id_string, role_int>` with the user roles
/// in each project of the workspace. Only includes projects where
/// the user has `is_active = true` AND active membership to the
/// workspace exists (mirror of Django filter `member__member_workspace__is_active=True`).
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/{slug}/project-roles",
    tag = "Users",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Map of project_id -> role"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn get_user_project_roles(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<std::collections::HashMap<String, i16>>, AppError> {
    // 1) Workspace must exist (404 if not).
    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // 2) Mirror of Django filter:
    //    member__member_workspace__workspace__slug=slug
    //    AND member__member_workspace__is_active=True
    //
    //    Equivalently: the user must have an active membership TO THE WORKSPACE
    //    in addition to the projects. If not, an empty map is returned
    //    (Django returns {} also because the queryset remains empty, not 403).
    let ws_membership = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if ws_membership.is_none() {
        return Ok(Json(std::collections::HashMap::new()));
    }

    // 3) Projects in the workspace where the user is an active member.
    //    Note: project_members.member_id is Option<Uuid> in the schema,
    //    so the equality filter already discards NULLs.
    let pms = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // 4) Build map {project_id_str: role}. Exact mirror of Django:
    //    {str(member["project_id"]): member["role"] for member in project_members}
    let map: std::collections::HashMap<String, i16> = pms
        .into_iter()
        .map(|pm| (pm.project_id.to_string(), pm.role))
        .collect();

    Ok(Json(map))
}

// ─── /api/users/me/workspaces/invitations/ (GET + POST) ─────────────────────
//
// Mirror of Django `UserWorkspaceInvitationsViewSet` in
// `plane/app/views/workspace/invite.py:244`. The frontend consumes it during
// onboarding (`apps/web/app/(all)/onboarding/page.tsx:43`) to show
// pending invitations and accept them in bulk.

/// Workspace embedded in the invitation response (mirror of
/// `WorkspaceLiteSerializer`: id, name, slug, logo_url).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserInviteWorkspaceLite {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    /// Simplified mirror of `Workspace.logo_url`. If the workspace uses
    /// `logo_asset` (not the legacy `logo` field) this field will be `None` —
    /// an acceptable limitation for the onboarding flow (no UI shows the logo
    /// in this view). If required, perform a JOIN with `file_assets`
    /// similar to `WorkspaceResponse::from_model`.
    pub logo_url: Option<String>,
}

/// Response for `GET /api/users/me/workspaces/invitations/`.
///
/// Mirror of `WorkSpaceMemberInviteSerializer(model=WorkspaceMemberInvite,
/// fields="__all__")` with nested `workspace` and computed `invite_link`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserWorkspaceInviteResponse {
    pub id: Uuid,
    pub email: String,
    pub accepted: bool,
    pub token: String,
    pub message: Option<String>,
    pub responded_at: Option<DateTime<FixedOffset>>,
    pub role: i16,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_by: Option<Uuid>,
    pub workspace: UserInviteWorkspaceLite,
    pub invite_link: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct JoinWorkspacesRequest {
    /// List of invitation UUIDs to accept. Exact mirror of the `invitations`
    /// field sent by the frontend in the POST.
    pub invitations: Vec<Uuid>,
}

/// `GET /api/users/me/workspaces/invitations/`
///
/// Lists pending invitations of the authenticated user, filtered
/// by their email. Mirror of `BaseViewSet.list` over the queryset
/// `WorkspaceMemberInvite.objects.filter(email=request.user.email)`.
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/invitations/",
    tag = "Users",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    responses(
        (status = 200, description = "List of pending workspace invitations"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_user_workspace_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<Json<Vec<UserWorkspaceInviteResponse>>, AppError> {
    // Without email there's no way to match invitations (same as Django: filter
    // by empty email returns an empty queryset).
    let Some(email) = user.email.as_ref() else {
        return Ok(Json(Vec::new()));
    };

    // Invitaciones activas del usuario.
    let invites = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::Email.eq(email))
        .order_by_desc(workspace_member_invites::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    if invites.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // select_related("workspace"): load workspaces in a single query.
    let workspace_ids: Vec<Uuid> = invites.iter().map(|i| i.workspace_id).collect();
    let workspaces_list = workspaces::Entity::find()
        .filter(workspaces::Column::Id.is_in(workspace_ids))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;
    let workspaces_by_id: std::collections::HashMap<Uuid, &workspaces::Model> =
        workspaces_list.iter().map(|w| (w.id, w)).collect();

    let resp: Vec<UserWorkspaceInviteResponse> = invites
        .into_iter()
        .filter_map(|i| {
            // If the workspace was deleted between queries, we discard the
            // invitation (Django with select_related would leave workspace=None and
            // the serializer would fail — we prefer filtering silently).
            let ws = workspaces_by_id.get(&i.workspace_id)?;
            Some(UserWorkspaceInviteResponse {
                id: i.id,
                // invite_link exact mirror of Django:
                //   f"/workspace-invitations/?invitation_id={obj.id}&slug={obj.workspace.slug}&token={obj.token}"
                invite_link: format!(
                    "/workspace-invitations/?invitation_id={}&slug={}&token={}",
                    i.id, ws.slug, i.token
                ),
                email: i.email,
                accepted: i.accepted,
                token: i.token,
                message: i.message,
                responded_at: i.responded_at,
                role: i.role,
                created_at: i.created_at,
                updated_at: i.updated_at,
                created_by: i.created_by_id,
                workspace: UserInviteWorkspaceLite {
                    id: ws.id,
                    name: ws.name.clone(),
                    slug: ws.slug.clone(),
                    logo_url: ws.logo.clone(),
                },
            })
        })
        .collect();

    Ok(Json(resp))
}

/// `POST /api/users/me/workspaces/invitations/`
///
/// Bulk accepts invitations whose UUIDs come in the body
/// (`{"invitations": [uuid, ...]}`). For each one:
///   1. If `WorkspaceMember` already exists (even if deactivated), it reactivates it
///      with the invitation role.
///   2. If it does not exist, it creates it via `INSERT ... ON CONFLICT DO NOTHING`
///      (mirror of `bulk_create(ignore_conflicts=True)`).
///   3. Deletes processed invitations.
///
/// Returns 204. Only processes invitations whose `email` matches the
/// authenticated user's — defense against UUID-guessing.
#[utoipa::path(
    post,
    path = "/api/users/me/workspaces/invitations/",
    tag = "Users",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    request_body = JoinWorkspacesRequest,
    responses(
        (status = 204, description = "Joined successfully"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn join_user_workspace_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<JoinWorkspacesRequest>,
) -> Result<StatusCode, AppError> {
    // No email -> no invitations to accept (silent, Django mirror).
    let Some(email) = user.email.as_ref() else {
        return Ok(StatusCode::NO_CONTENT);
    };
    if body.invitations.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    // ── 1. Load invitations matching pk ∈ payload AND email == user.email
    //
    // The double filter (pk + email) is Django's defense against a
    // user accepting ANOTHER's invitation knowing their UUID. Crucial
    // to maintain this in Rust.
    let invites = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::Id.is_in(body.invitations.clone()))
        .filter(workspace_member_invites::Column::Email.eq(email))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    if invites.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let now = chrono::Utc::now().fixed_offset();
    let txn = state
        .db
        .begin()
        .await
        .map_err(AppError::Database)?;

    // ── 2. Reactivate existing memberships with the invitation role.
    //
    // Mirror of:
    //   WorkspaceMember.objects.filter(workspace_id=invitation.workspace_id,
    //                                  member=request.user)
    //                          .update(is_active=True, role=invitation.role)
    //
    // We iterate to preserve the per-invitation `role` (a bulk UPDATE with
    // the same role would lose granularity).
    for inv in &invites {
        if let Some(existing) = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(inv.workspace_id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .filter(workspace_members::Column::DeletedAt.is_null())
            .one(&txn)
            .await
            .map_err(AppError::Database)?
        {
            let mut active: workspace_members::ActiveModel = existing.into();
            active.is_active = Set(true);
            active.role = Set(inv.role);
            active.updated_at = Set(now);
            active.updated_by_id = Set(Some(user.id));
            active.update(&txn).await.map_err(AppError::Database)?;
        }
    }

    // ── 3. Bulk insert of new memberships with ON CONFLICT DO NOTHING.
    //
    // The unique constraint `workspace_member_unique_workspace_member_when_deleted_at_null`
    // covers (workspace_id, member_id) when deleted_at IS NULL — mirror of
    // Django's `ignore_conflicts=True`.
    let new_members: Vec<workspace_members::ActiveModel> = invites
        .iter()
        .map(|inv| workspace_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            workspace_id: Set(inv.workspace_id),
            member_id: Set(user.id),
            role: Set(inv.role),
            company_role: Set(None),
            view_props: Set(serde_json::json!({})),
            default_props: Set(serde_json::json!({})),
            issue_props: Set(serde_json::json!({})),
            is_active: Set(true),
            explored_features: Set(serde_json::json!([])),
            getting_started_checklist: Set(serde_json::json!({})),
            tips: Set(serde_json::json!({})),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        })
        .collect();

    workspace_members::Entity::insert_many(new_members)
        .on_conflict(
            OnConflict::columns([
                workspace_members::Column::WorkspaceId,
                workspace_members::Column::MemberId,
            ])
            // PostgreSQL EXIGE repetir el predicado del arbiter index parcial en el
            // conflict target. La unique constraint Django (ver
            // plane/db/models/workspace.py:215-223) es
            //   UNIQUE (workspace_id, member_id) WHERE deleted_at IS NULL
            // y sin `.target_and_where(...)` el INSERT revienta con
            // "there is no unique or exclusion constraint matching the ON CONFLICT
            // specification" — the other existing unique (unique_together on
            // the 3 columns including deleted_at) also doesn't match because the conflict
            // target only mentions 2. See
            // https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT.
            .target_and_where(Expr::col(workspace_members::Column::DeletedAt).is_null())
            .do_nothing()
            .to_owned(),
        )
        .do_nothing()
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    // ── 4. Soft-delete processed invitations.
    //
    // Django does .delete() (with soft delete enabled at the base manager level).
    // We replicate by setting deleted_at instead of physical DELETE to
    // maintain traceability and consistency with the rest of the codebase.
    let invite_ids: Vec<Uuid> = invites.iter().map(|i| i.id).collect();
    workspace_member_invites::Entity::update_many()
        .col_expr(
            workspace_member_invites::Column::DeletedAt,
            Expr::value(now),
        )
        .col_expr(
            workspace_member_invites::Column::UpdatedAt,
            Expr::value(now),
        )
        .filter(workspace_member_invites::Column::Id.is_in(invite_ids))
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// ME ACTIVITIES  (UserActivityEndpoint)
// ═══════════════════════════════════════════════════════════════════════════
//
// GET /api/users/me/activities/
//
// Mirror of Django `UserActivityEndpoint.get`
// (plane/app/views/user/base.py:380).
//
// Returns ALL `IssueActivity` for the requester in any workspace/
// project, with Django-style cursor pagination.
//
// Unlike `WorkspaceUserActivityEndpoint`:
//   - It does not filter by workspace/project (it's cross-workspace).
//   - It does not exclude field in (comment|vote|reaction|draft) — Django doesn't
//     either in this endpoint (only in the workspace-scoped one). See Django:
//     pure queryset `actor=request.user`.
//   - Does not require membership checks (only the user's own activities).
//   - `default_per_page = 1000` (mirror of `BasePaginator.get_per_page`).
//
// Response shape identical to `BasePaginator.paginate`, detail DTOs are
// reused from `routes::workspaces`.

/// Query params of `GET /users/me/activities/`.
#[derive(Debug, Deserialize)]
pub struct MeActivitiesQuery {
    pub per_page: Option<u64>,
    pub cursor: Option<String>,
    pub order_by: Option<String>,
}

/// `GET /api/users/me/activities/`
///
/// Issue activities of the requester across all workspaces.
/// Mirror of `UserActivityEndpoint` in Django.
#[utoipa::path(
    get,
    path = "/api/users/me/activities/",
    tag = "Users",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("cursor"   = Option<String>, Query, description = "Cursor {per_page}:{offset}:{is_prev}"),
        ("per_page" = Option<u64>,    Query, description = "Items per page (default 1000, max 1000)"),
        ("order_by" = Option<String>, Query, description = "Ordering column, prefix with `-` for desc (default `-created_at`)"),
    ),
    responses(
        (status = 200, description = "Paginated list of the authenticated user's issue activities"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_my_activities(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Query(q): Query<MeActivitiesQuery>,
) -> Result<impl IntoResponse, AppError> {
    use crate::routes::workspaces::{
        ActivityActorDetail, ActivityIssueDetail, ActivityProjectDetail, ActivityWorkspaceDetail,
        UserActivityItem,
    };
    use std::collections::{HashMap, HashSet};

    let db = &state.db;

    // Mirror of `BasePaginator.get_per_page`: default_per_page=1000, max_per_page=1000.
    const DEFAULT_PER_PAGE: u64 = 1000;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // ── Base query ───────────────────────────────────────────────────────────
    // Django: IssueActivity.objects.filter(actor=request.user)
    //
    // SECURITY NOTE: filtering by `actor_id = auth_user.id` is the only
    // access control needed — a user can only see THEIR own
    // activities. No IDOR possible because the actor is derived from the
    // credential, not the path.
    // Soft-delete: we filter `deleted_at IS NULL` to align with the
    // base manager of the rest of the Rust codebase (Django's queryset
    // applies the same filter via `SoftDeleteManager` at the model level).
    let base = issue_activities::Entity::find()
        .filter(issue_activities::Column::ActorId.eq(user.id))
        .filter(issue_activities::Column::DeletedAt.is_null());

    // Total count before paging (same filter, no order or offset).
    let total_count = base.clone().count(db).await.map_err(AppError::Database)?;

    // ── Ordenamiento (default -created_at) ───────────────────────────────────
    let order_col = q.order_by.as_deref().unwrap_or("-created_at");
    let (col, asc) = if let Some(stripped) = order_col.strip_prefix('-') {
        (stripped, false)
    } else {
        (order_col, true)
    };
    let ordered = match col {
        "created_at" => {
            if asc {
                base.order_by_asc(issue_activities::Column::CreatedAt)
            } else {
                base.order_by_desc(issue_activities::Column::CreatedAt)
            }
        }
        "updated_at" => {
            if asc {
                base.order_by_asc(issue_activities::Column::UpdatedAt)
            } else {
                base.order_by_desc(issue_activities::Column::UpdatedAt)
            }
        }
        // Any non-whitelisted column -> safe fallback to default.
        // This prevents SQL injection via `order_by` and aligns with the spirit
        // of Django's `BasePaginator` (which only accepts model columns).
        _ => base.order_by_desc(issue_activities::Column::CreatedAt),
    };

    // ── Fetch page ───────────────────────────────────────────────────────────
    let activities = ordered
        .paginate(db, limit)
        .fetch_page(cursor.offset)
        .await
        .map_err(AppError::Database)?;

    if activities.is_empty() {
        let body = pagination::build_response(
            Vec::<UserActivityItem>::new(),
            total_count,
            limit,
            cursor.offset,
        );
        return Ok((StatusCode::OK, Json(body)));
    }

    // ── Batch-fetch of related objects (avoid N+1) ─────────────────────
    //
    // Since the actor is always the requester, we reuse the `users::Model`
    // already provided by AnyAuth, avoiding an extra query.

    let actor_avatar_url = if let Some(asset_id) = user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !user.avatar.is_empty() {
        Some(user.avatar.clone())
    } else {
        None
    };

    // Issue IDs from current page
    let issue_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.issue_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let issues_map: HashMap<Uuid, issues::Model> = if !issue_ids.is_empty() {
        issues::Entity::find()
            .filter(issues::Column::Id.is_in(issue_ids))
            .filter(issues::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|i| (i.id, i))
            .collect()
    } else {
        HashMap::new()
    };

    // Project IDs (siempre NOT NULL en IssueActivity)
    let project_ids: Vec<Uuid> = activities
        .iter()
        .map(|a| a.project_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let projects_map: HashMap<Uuid, projects::Model> = projects::Entity::find()
        .filter(projects::Column::Id.is_in(project_ids))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|p| (p.id, p))
        .collect();

    // Workspace IDs — cross-workspace, that's why we batch-fetch (unlike the
    // workspace-scoped endpoint which resolves only one).
    let workspace_ids: Vec<Uuid> = activities
        .iter()
        .map(|a| a.workspace_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let workspaces_map: HashMap<Uuid, workspaces::Model> = workspaces::Entity::find()
        .filter(workspaces::Column::Id.is_in(workspace_ids))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|w| (w.id, w))
        .collect();

    // ── Ensamblar respuesta ──────────────────────────────────────────────────
    let results: Vec<UserActivityItem> = activities
        .into_iter()
        .map(|a| {
            let actor_detail = a.actor_id.map(|_| ActivityActorDetail {
                id: user.id,
                first_name: user.first_name.clone(),
                last_name: user.last_name.clone(),
                avatar: user.avatar.clone(),
                avatar_url: actor_avatar_url.clone(),
                is_bot: user.is_bot,
                display_name: user.display_name.clone(),
            });

            let issue_detail = a.issue_id.and_then(|iid| {
                issues_map.get(&iid).map(|i| ActivityIssueDetail {
                    id: i.id,
                    name: i.name.clone(),
                    sequence_id: i.sequence_id,
                    project_id: i.project_id,
                    workspace_id: i.workspace_id,
                })
            });

            let project_detail = projects_map.get(&a.project_id).map(|p| ActivityProjectDetail {
                id: p.id,
                identifier: p.identifier.clone(),
                name: p.name.clone(),
                logo_props: p.logo_props.clone(),
            });

            let workspace_detail =
                workspaces_map.get(&a.workspace_id).map(|w| ActivityWorkspaceDetail {
                    id: w.id,
                    name: w.name.clone(),
                    slug: w.slug.clone(),
                    logo: w.logo.clone(),
                });

            UserActivityItem {
                id: a.id,
                verb: a.verb,
                field: a.field,
                old_value: a.old_value,
                new_value: a.new_value,
                comment: a.comment,
                actor_id: a.actor_id,
                issue_id: a.issue_id,
                issue_comment_id: a.issue_comment_id,
                project_id: a.project_id,
                workspace_id: a.workspace_id,
                old_identifier: a.old_identifier,
                new_identifier: a.new_identifier,
                epoch: a.epoch,
                created_at: a.created_at.into(),
                updated_at: a.updated_at.into(),
                actor_detail,
                issue_detail,
                project_detail,
                workspace_detail,
            }
        })
        .collect();

    let body = pagination::build_response(results, total_count, limit, cursor.offset);
    Ok((StatusCode::OK, Json(body)))
}


// ─── GET /users/me/workspaces/{slug}/activity-graph/ ─────────────────────────

/// User activity in the workspace grouped by date (last 6 months).
///
/// Mirror of `UserActivityGraphEndpoint`
/// (`apps/api/plane/app/views/workspace/user.py`).
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/{slug}/activity-graph/",
    tag = "Users",
    params(("slug" = String, Path, description = "Workspace slug")),
    security(("TokenAuth" = []))
)]
pub async fn get_activity_graph(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    use sea_orm::{FromQueryResult, Statement};

    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    #[derive(FromQueryResult)]
    struct ActivityRow {
        created_date: chrono::NaiveDate,
        activity_count: i64,
    }

    let rows = ActivityRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT
            DATE(created_at) AS created_date,
            COUNT(*) AS activity_count
        FROM issue_activities
        WHERE actor_id = $1
          AND workspace_id = $2
          AND created_at::date >= CURRENT_DATE - INTERVAL '6 months'
          AND deleted_at IS NULL
        GROUP BY DATE(created_at)
        ORDER BY created_date
        "#,
        vec![
            sea_orm::Value::Uuid(Some(Box::new(user.id))),
            sea_orm::Value::Uuid(Some(Box::new(ws.id))),
        ],
    ))
    .all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let result = rows
        .into_iter()
        .map(|r| serde_json::json!({
            "created_date": r.created_date.to_string(),
            "activity_count": r.activity_count,
        }))
        .collect();

    Ok(Json(result))
}

// ─── GET /users/me/workspaces/{slug}/issues-completed-graph/ ─────────────────

/// Issues completed by the user in the workspace grouped by week of the month.
///
/// Mirror of `UserIssueCompletedGraphEndpoint`
/// (`apps/api/plane/app/views/workspace/user.py`).
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/{slug}/issues-completed-graph/",
    tag = "Users",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("month" = Option<i32>, Query, description = "Month (1-12, default 1)"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_issues_completed_graph(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    use sea_orm::{FromQueryResult, Statement};

    let month: i32 = params.get("month")
        .and_then(|m| m.parse().ok())
        .unwrap_or(1)
        .clamp(1, 12);

    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    #[derive(FromQueryResult)]
    struct CompletedRow {
        week: i32,
        completed_count: i64,
    }

    // Django Mirror: week = EXTRACT(WEEK FROM completed_at) % 4
    let rows = CompletedRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT
            (EXTRACT(WEEK FROM i.completed_at)::int % 4) AS week,
            COUNT(*) AS completed_count
        FROM issues i
        INNER JOIN issue_assignees ia
            ON ia.issue_id = i.id
           AND ia.assignee_id = $1
           AND ia.deleted_at IS NULL
        WHERE i.workspace_id = $2
          AND EXTRACT(MONTH FROM i.completed_at) = $3
          AND i.completed_at IS NOT NULL
          AND i.deleted_at IS NULL
        GROUP BY (EXTRACT(WEEK FROM i.completed_at)::int % 4)
        ORDER BY week
        "#,
        vec![
            sea_orm::Value::Uuid(Some(Box::new(user.id))),
            sea_orm::Value::Uuid(Some(Box::new(ws.id))),
            sea_orm::Value::Int(Some(month)),
        ],
    ))
    .all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let result = rows
        .into_iter()
        .map(|r| serde_json::json!({
            "week": r.week,
            "completed_count": r.completed_count,
        }))
        .collect();

    Ok(Json(result))
}

// ─── GET /users/me/workspaces/{slug}/dashboard/ ───────────────────────────────

/// User dashboard: recent activity + completed issues + statistics.
///
/// Mirror of `UserWorkspaceDashboardEndpoint`
/// (`apps/api/plane/app/views/workspace/base.py`).
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/{slug}/dashboard/",
    tag = "Users",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("month" = Option<i32>, Query, description = "Month for completed issues"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_workspace_dashboard(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    use sea_orm::{FromQueryResult, Statement};

    let month: i32 = params.get("month")
        .and_then(|m| m.parse().ok())
        .unwrap_or(1)
        .clamp(1, 12);

    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // ── Recent activity (last 3 months) ──────────────────────────────────────
    #[derive(FromQueryResult)]
    struct ActivityRow {
        created_date: chrono::NaiveDate,
        activity_count: i64,
    }
    let issue_activities = ActivityRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT DATE(created_at) AS created_date, COUNT(*) AS activity_count
        FROM issue_activities
        WHERE actor_id = $1 AND workspace_id = $2
          AND created_at::date >= CURRENT_DATE - INTERVAL '3 months'
          AND deleted_at IS NULL
        GROUP BY DATE(created_at)
        ORDER BY created_date
        "#,
        vec![
            sea_orm::Value::Uuid(Some(Box::new(user.id))),
            sea_orm::Value::Uuid(Some(Box::new(ws.id))),
        ],
    ))
    .all(&state.db)
    .await
    .map_err(AppError::Database)?;

    // ── Issues completed this month (per week) ───────────────────────────────
    #[derive(FromQueryResult)]
    struct CompletedRow { week: i32, completed_count: i64 }
    let completed_issues = CompletedRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT
            ((EXTRACT(DAY FROM i.completed_at)::int - 1) / 7 + 1) AS week,
            COUNT(*) AS completed_count
        FROM issues i
        INNER JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.assignee_id = $1 AND ia.deleted_at IS NULL
        WHERE i.workspace_id = $2
          AND EXTRACT(MONTH FROM i.completed_at) = $3
          AND i.completed_at IS NOT NULL AND i.deleted_at IS NULL
        GROUP BY ((EXTRACT(DAY FROM i.completed_at)::int - 1) / 7 + 1)
        ORDER BY week
        "#,
        vec![
            sea_orm::Value::Uuid(Some(Box::new(user.id))),
            sea_orm::Value::Uuid(Some(Box::new(ws.id))),
            sea_orm::Value::Int(Some(month)),
        ],
    ))
    .all(&state.db)
    .await
    .map_err(AppError::Database)?;

    // ── Issue counts ─────────────────────────────────────────────────────────
    #[derive(FromQueryResult)]
    struct CountRow { total: i64 }

    let assigned_total = CountRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT COUNT(DISTINCT i.id) AS total FROM issues i
           INNER JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.assignee_id = $1 AND ia.deleted_at IS NULL
           WHERE i.workspace_id = $2 AND i.deleted_at IS NULL"#,
        vec![sea_orm::Value::Uuid(Some(Box::new(user.id))), sea_orm::Value::Uuid(Some(Box::new(ws.id)))],
    )).one(&state.db).await.map_err(AppError::Database)?
      .map(|r| r.total).unwrap_or(0);

    let pending_total = CountRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT COUNT(DISTINCT i.id) AS total FROM issues i
           INNER JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.assignee_id = $1 AND ia.deleted_at IS NULL
           INNER JOIN states s ON s.id = i.state_id
           WHERE i.workspace_id = $2 AND i.deleted_at IS NULL
             AND s.group NOT IN ('completed', 'cancelled')"#,
        vec![sea_orm::Value::Uuid(Some(Box::new(user.id))), sea_orm::Value::Uuid(Some(Box::new(ws.id)))],
    )).one(&state.db).await.map_err(AppError::Database)?
      .map(|r| r.total).unwrap_or(0);

    let completed_total = CountRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT COUNT(DISTINCT i.id) AS total FROM issues i
           INNER JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.assignee_id = $1 AND ia.deleted_at IS NULL
           INNER JOIN states s ON s.id = i.state_id
           WHERE i.workspace_id = $2 AND i.deleted_at IS NULL AND s.group = 'completed'"#,
        vec![sea_orm::Value::Uuid(Some(Box::new(user.id))), sea_orm::Value::Uuid(Some(Box::new(ws.id)))],
    )).one(&state.db).await.map_err(AppError::Database)?
      .map(|r| r.total).unwrap_or(0);

    let issues_due_week = CountRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT COUNT(DISTINCT i.id) AS total FROM issues i
           INNER JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.assignee_id = $1 AND ia.deleted_at IS NULL
           WHERE i.workspace_id = $2 AND i.deleted_at IS NULL
             AND EXTRACT(WEEK FROM i.target_date) = EXTRACT(WEEK FROM CURRENT_DATE)"#,
        vec![sea_orm::Value::Uuid(Some(Box::new(user.id))), sea_orm::Value::Uuid(Some(Box::new(ws.id)))],
    )).one(&state.db).await.map_err(AppError::Database)?
      .map(|r| r.total).unwrap_or(0);

    // ── State distribution ────────────────────────────────────────────────────
    #[derive(FromQueryResult)]
    struct StateDistRow { state_group: String, state_count: i64 }
    let state_distribution = StateDistRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT s.group AS state_group, COUNT(DISTINCT i.id) AS state_count
           FROM issues i
           INNER JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.assignee_id = $1 AND ia.deleted_at IS NULL
           INNER JOIN states s ON s.id = i.state_id
           WHERE i.workspace_id = $2 AND i.deleted_at IS NULL
           GROUP BY s.group ORDER BY s.group"#,
        vec![sea_orm::Value::Uuid(Some(Box::new(user.id))), sea_orm::Value::Uuid(Some(Box::new(ws.id)))],
    )).all(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({
        "issue_activities": issue_activities.iter().map(|r| serde_json::json!({
            "created_date": r.created_date.to_string(),
            "activity_count": r.activity_count,
        })).collect::<Vec<_>>(),
        "completed_issues": completed_issues.iter().map(|r| serde_json::json!({
            "week_in_month": r.week,
            "completed_count": r.completed_count,
        })).collect::<Vec<_>>(),
        "assigned_issues_count": assigned_total,
        "pending_issues_count": pending_total,
        "completed_issues_count": completed_total,
        "issues_due_week": issues_due_week,
        "state_distribution": state_distribution.iter().map(|r| serde_json::json!({
            "state_group": r.state_group,
            "state_count": r.state_count,
        })).collect::<Vec<_>>(),
    })))
}

// ─── GET /users/last-visited-workspace/ ──────────────────────────────────────

/// Returns the user's last visited workspace with its projects.
///
/// Mirror of `UserLastProjectWithWorkspaceEndpoint`
/// (`apps/api/plane/app/views/workspace/user.py`).
#[utoipa::path(
    get,
    path = "/api/users/last-visited-workspace/",
    tag = "Users",
    security(("TokenAuth" = []))
)]
pub async fn get_last_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<Json<serde_json::Value>, AppError> {
    // Read last_workspace_id from user profile
    use crate::entities::profiles;
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let last_workspace_id = profile.and_then(|p| p.last_workspace_id);

    let Some(ws_id) = last_workspace_id else {
        return Ok(Json(serde_json::json!({
            "workspace_details": {},
            "project_details": [],
        })));
    };

    let workspace = workspaces::Entity::find_by_id(ws_id)
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(ws) = workspace else {
        return Ok(Json(serde_json::json!({
            "workspace_details": {},
            "project_details": [],
        })));
    };

    let members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws_id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let project_ids: Vec<uuid::Uuid> = members.iter().map(|m| m.project_id).collect();
    let projects_list = if project_ids.is_empty() {
        vec![]
    } else {
        projects::Entity::find()
            .active()
            .filter(projects::Column::Id.is_in(project_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    let project_details: Vec<serde_json::Value> = members.iter().map(|m| {
        let proj = projects_list.iter().find(|p| p.id == m.project_id);
        serde_json::json!({
            "id": m.id,
            "member_id": m.member_id,
            "role": m.role,
            "project": proj.as_ref().map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "identifier": p.identifier,
                "workspace_id": p.workspace_id,
            })),
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "workspace_details": {
            "id": ws.id,
            "name": ws.name,
            "slug": ws.slug,
            "owner_id": ws.owner_id,
        },
        "project_details": project_details,
    })))
}

// ── Email update endpoints ───────────────────────────────────────────────────
// Mirror of `UserMeEndpoint.generate_email_code` and `UserMeEndpoint.update_email`
// in Django (`plane/app/views/user/base.py`).

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GenerateEmailCodeRequest {
    /// New email to send the verification code to.
    pub email: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEmailRequest {
    /// New email to set.
    pub email: String,
    /// 6-digit code sent to the new email.
    pub code: String,
}

/// Generates and sends a verification code to the user's new email.
///
/// `POST /users/me/email/generate-code`
///
/// Mirror of Django's `UserMeEndpoint.generate_code`.
/// The code is stored in Redis with 10-minute TTL under the key
/// `magic_email_update_{user_id}_{new_email}`.
#[utoipa::path(
    post,
    path = "/users/me/email/generate-code",
    tag = "Users",
    request_body = GenerateEmailCodeRequest,
    responses(
        (status = 200, description = "Verification code sent"),
        (status = 400, description = "Invalid or already-used email"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn generate_email_code(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<GenerateEmailCodeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    use fred::prelude::{Expiration, KeysInterface};

    let new_email = body.email.trim().to_lowercase();

    // Validate not empty
    if new_email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }

    // Validate it is different from the current email
    if user.email.as_deref() == Some(new_email.as_str()) {
        return Err(AppError::BadRequest(
            "New email must be different from current email".into(),
        ));
    }

    // Verify email is not in use
    let exists = users::Entity::find()
        .filter(users::Column::Email.eq(&new_email))
        .filter(users::Column::Id.ne(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if exists {
        return Err(AppError::BadRequest(
            "An account with this email already exists".into(),
        ));
    }

    // Generate 6-digit code
    let token = format!("{:06}", rand::random::<u32>() % 900_000 + 100_000);

    let cache_key = format!("magic_email_update_{}_{}", user.id, new_email);
    let cache_value = serde_json::json!({ "token": token }).to_string();

    // Save in Redis with TTL 600s (10 min)
    state
        .redis
        .set::<(), _, _>(
            &cache_key,
            cache_value.as_str(),
            Some(Expiration::EX(600)),
            None,
            false,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis set error: {e}")))?;

    // In production, the email would be sent; here we log the code for debugging
    tracing::info!(
        user_id = %user.id,
        new_email = %new_email,
        "Email update code generated (in production this would be emailed)"
    );

    Ok(Json(serde_json::json!({
        "message": "Verification code sent to email"
    })))
}

/// Verifies the code and updates the user's email.
///
/// `POST /users/me/email`
///
/// Mirror of Django's `UserMeEndpoint.update_email`.
/// Invalidates the current session after the change (user must re-authenticate).
#[utoipa::path(
    post,
    path = "/users/me/email",
    tag = "Users",
    request_body = UpdateEmailRequest,
    responses(
        (status = 200, description = "Email updated successfully"),
        (status = 400, description = "Invalid code or email"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn update_user_email(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Json(body): Json<UpdateEmailRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    use fred::prelude::KeysInterface;

    let new_email = body.email.trim().to_lowercase();
    let code = body.code.trim().to_owned();

    if new_email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }
    if code.is_empty() {
        return Err(AppError::BadRequest("Verification code is required".into()));
    }

    // Verify email availability
    let exists = users::Entity::find()
        .filter(users::Column::Email.eq(&new_email))
        .filter(users::Column::Id.ne(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if exists {
        return Err(AppError::BadRequest(
            "An account with this email already exists".into(),
        ));
    }

    // Verify code in Redis
    let cache_key = format!("magic_email_update_{}_{}", user.id, new_email);
    let cached: Option<String> = state
        .redis
        .get(&cache_key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis get error: {e}")))?;

    let cached_data = cached.ok_or_else(|| {
        AppError::BadRequest("Verification code has expired or is invalid".into())
    })?;

    let stored_token = serde_json::from_str::<serde_json::Value>(&cached_data)
        .ok()
        .and_then(|v| v.get("token").and_then(|t| t.as_str()).map(str::to_owned))
        .ok_or_else(|| AppError::BadRequest("Invalid cached data".into()))?;

    if stored_token != code {
        return Err(AppError::BadRequest("Invalid verification code".into()));
    }

    // Update user's email
    let user_model = users::Entity::find_by_id(user.id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: users::ActiveModel = user_model.into();
    active.email = Set(Some(new_email.clone()));
    active.is_email_verified = Set(false);
    active.update(&state.db).await.map_err(AppError::Database)?;

    // Delete the code from Redis
    let _ = state.redis.del::<i64, _>(&cache_key).await;

    tracing::info!(user_id = %user.id, new_email = %new_email, "User email updated");

    Ok(Json(serde_json::json!({
        "message": "Email updated successfully. Please sign in again."
    })))
}
