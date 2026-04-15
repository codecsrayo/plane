// src/routes/users.rs
//! Endpoints de perfil de usuario autenticado.
//!
//! Equivalente a `plane/app/views/user/base.py` en Django.
//!
//! Rutas implementadas:
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

use axum::{
    extract::{Path, State},
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
    entities::{accounts, profiles, project_members, users, workspace_member_invites, workspace_members, workspaces},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// Alias para legibilidad — `.active()` filtra `deleted_at IS NULL`
// (implementado via `impl_soft_delete!` en entities/mod.rs)

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Representación pública del usuario autenticado (`/users/me/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserMeResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    pub cover_image: Option<String>,
    pub is_active: bool,
    pub is_bot: bool,
    pub is_email_verified: bool,
    pub is_password_autoset: bool,
    pub is_superuser: bool,
    pub is_managed: bool,
    pub user_timezone: String,
    pub date_joined: DateTime<FixedOffset>,
    pub last_login: Option<DateTime<FixedOffset>>,
}

/// Settings del usuario (`/users/me/settings/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserSettingsResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub display_name: String,
    pub avatar: String,
    pub cover_image: Option<String>,
    pub is_email_verified: bool,
    pub is_onboarded: bool,
    pub onboarding_step: serde_json::Value,
    pub last_workspace_id: Option<Uuid>,
}

/// Perfil del usuario (`/users/me/profile/`).
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

/// Cuenta OAuth del usuario (`/users/me/accounts/`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AccountResponse {
    pub id: Uuid,
    pub provider: String,
    pub provider_account_id: String,
    pub last_connected_at: DateTime<FixedOffset>,
    pub metadata: serde_json::Value,
    pub user_id: Uuid,
}

/// Workspace del usuario (`/users/me/workspaces/`).
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

/// Body para PATCH `/users/me/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar: Option<String>,
    pub cover_image: Option<String>,
    pub user_timezone: Option<String>,
}

/// Body para PATCH `/users/me/profile/`.
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

/// Body para PATCH `/users/me/onboard/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OnboardRequest {
    pub is_onboarded: Option<bool>,
}

/// Body para PATCH `/users/me/tour-completed/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TourCompletedRequest {
    pub is_tour_completed: Option<bool>,
}

// ── Conversiones ─────────────────────────────────────────────────────────────

fn user_to_me_response(u: &users::Model) -> UserMeResponse {
    UserMeResponse {
        id: u.id,
        username: u.username.clone(),
        email: u.email.clone(),
        display_name: u.display_name.clone(),
        first_name: u.first_name.clone(),
        last_name: u.last_name.clone(),
        avatar: u.avatar.clone(),
        cover_image: u.cover_image.clone(),
        is_active: u.is_active,
        is_bot: u.is_bot,
        is_email_verified: u.is_email_verified,
        is_password_autoset: u.is_password_autoset,
        is_superuser: u.is_superuser,
        is_managed: u.is_managed,
        user_timezone: u.user_timezone.clone(),
        date_joined: u.date_joined,
        last_login: u.last_login,
    }
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
/// Devuelve el perfil del usuario autenticado.
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
pub async fn get_me(AnyAuth(user): AnyAuth) -> impl IntoResponse {
    Json(user_to_me_response(&user))
}

/// PATCH /api/users/me/
///
/// Actualiza campos del usuario autenticado.
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
    // Validar display_name si viene
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

    Ok(Json(user_to_me_response(&updated)))
}

/// DELETE /api/users/me/ — desactiva la cuenta del usuario.
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
    // Verificar que no es el único admin en algún workspace activo
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

    // Desactivar workspaces memberships
    for m in memberships {
        let mut am: workspace_members::ActiveModel = m.into();
        am.is_active = Set(false);
        am.update(&state.db).await.map_err(AppError::Database)?;
    }

    // Desactivar perfil
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

    // Desactivar usuario
    let mut au: users::ActiveModel = user.into();
    au.is_active = Set(false);
    au.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/users/session/
///
/// Retorna si el usuario está autenticado. No requiere auth.
#[utoipa::path(
    get,
    path = "/users/session/",
    tag = "Users",
    responses(
        (status = 200, description = "Session info"),
    )
)]
pub async fn get_session(OptionalAnyAuth(user_opt): OptionalAnyAuth) -> impl IntoResponse {
    match user_opt {
        Some(user) => Json(serde_json::json!({
            "is_authenticated": true,
            "user": user_to_me_response(&user),
        })),
        None => Json(serde_json::json!({ "is_authenticated": false })),
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

    Ok(Json(UserSettingsResponse {
        id: user.id,
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        avatar: user.avatar.clone(),
        cover_image: user.cover_image.clone(),
        is_email_verified: user.is_email_verified,
        is_onboarded: profile.is_onboarded,
        onboarding_step: profile.onboarding_step,
        last_workspace_id: profile.last_workspace_id,
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
    // Solo superusers son instance admins en la implementación base
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
/// Lista los workspaces activos donde el usuario es miembro.
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
    // Obtener membresías activas del usuario
    let memberships = workspace_members::Entity::find()
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .active()
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Obtener IDs de workspaces
    let workspace_ids: Vec<Uuid> = memberships.iter().map(|m| m.workspace_id).collect();

    if workspace_ids.is_empty() {
        return Ok(Json(Vec::<UserWorkspaceResponse>::new()));
    }

    // Cargar workspaces activos
    let wss = workspaces::Entity::find()
        .filter(workspaces::Column::Id.is_in(workspace_ids))
        .active()
        .order_by_asc(workspaces::Column::Name)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Construir respuesta enriquecida con rol
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
// Espejo de Django `UserProjectRolesEndpoint` en
// `plane/app/views/project/member.py:327`. Devuelve un mapa
// `{project_id_str: role_int}` con los roles del usuario en cada proyecto
// del workspace donde es miembro activo. El frontend lo consume para
// resolver permisos por proyecto sin hacer N requests.

/// `GET /api/users/me/workspaces/{slug}/project-roles`
///
/// Devuelve `HashMap<project_id_string, role_int>` con los roles del
/// usuario en cada proyecto del workspace. Solo incluye proyectos donde
/// el usuario tiene `is_active = true` Y existe membresía activa al
/// workspace (mirror del filtro `member__member_workspace__is_active=True`
/// de Django).
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
    // 1) Workspace debe existir (404 si no).
    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // 2) Mirror del filtro Django:
    //    member__member_workspace__workspace__slug=slug
    //    AND member__member_workspace__is_active=True
    //
    //    Equivale a: el usuario debe tener una membresía activa AL WORKSPACE
    //    además de a los proyectos. Si no la tiene, devolvemos mapa vacío
    //    (Django retorna {} también porque el queryset queda vacío, no 403).
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

    // 3) Proyectos del workspace donde el usuario es miembro activo.
    //    Nota: project_members.member_id es Option<Uuid> en el schema,
    //    así que el filtro por igualdad ya descarta NULLs.
    let pms = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // 4) Construir mapa {project_id_str: role}. Mirror exacto de Django:
    //    {str(member["project_id"]): member["role"] for member in project_members}
    let map: std::collections::HashMap<String, i16> = pms
        .into_iter()
        .map(|pm| (pm.project_id.to_string(), pm.role))
        .collect();

    Ok(Json(map))
}

// ─── /api/users/me/workspaces/invitations/ (GET + POST) ─────────────────────
//
// Espejo de Django `UserWorkspaceInvitationsViewSet` en
// `plane/app/views/workspace/invite.py:244`. El frontend lo consume durante
// onboarding (`apps/web/app/(all)/onboarding/page.tsx:43`) para mostrar
// invitaciones pendientes y aceptarlas en bulk.

/// Workspace embebido en la respuesta de invitación (mirror de
/// `WorkspaceLiteSerializer`: id, name, slug, logo_url).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserInviteWorkspaceLite {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    /// Mirror simplificado de `Workspace.logo_url`. Si el workspace usa
    /// `logo_asset` (no el campo `logo` legacy) este campo será `None` —
    /// limitación aceptable para el flujo de onboarding (no hay UI que
    /// muestre el logo en esta vista). Si se requiere, hacer JOIN con
    /// `file_assets` similar a `WorkspaceResponse::from_model`.
    pub logo_url: Option<String>,
}

/// Respuesta de `GET /api/users/me/workspaces/invitations/`.
///
/// Mirror de `WorkSpaceMemberInviteSerializer(model=WorkspaceMemberInvite,
/// fields="__all__")` con `workspace` nested y `invite_link` computado.
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
    /// Lista de UUIDs de invitaciones a aceptar. Mirror exacto del campo
    /// `invitations` que envía el frontend en el POST.
    pub invitations: Vec<Uuid>,
}

/// `GET /api/users/me/workspaces/invitations/`
///
/// Lista las invitaciones pendientes del usuario autenticado, filtradas
/// por su email. Mirror de `BaseViewSet.list` sobre el queryset
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
    // Sin email no hay forma de matchear invitaciones (Django igual: filter
    // por email vacío devuelve queryset vacío).
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

    // select_related("workspace"): cargar workspaces en una sola query.
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
            // Si el workspace fue eliminado entre queries, descartamos la
            // invitación (Django con select_related dejaría workspace=None y
            // el serializer fallaría — preferimos filtrar silenciosamente).
            let ws = workspaces_by_id.get(&i.workspace_id)?;
            Some(UserWorkspaceInviteResponse {
                id: i.id,
                // invite_link mirror exacto de Django:
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
/// Acepta en bulk las invitaciones cuyos UUIDs vienen en el body
/// (`{"invitations": [uuid, ...]}`). Para cada una:
///   1. Si ya existe `WorkspaceMember` (incluso desactivado), lo reactiva
///      con el rol de la invitación.
///   2. Si no existe, lo crea via `INSERT ... ON CONFLICT DO NOTHING`
///      (mirror de `bulk_create(ignore_conflicts=True)`).
///   3. Borra las invitaciones procesadas.
///
/// Devuelve 204. Solo procesa invitaciones cuyo `email` coincide con el
/// del usuario autenticado — defensa contra UUID-guessing.
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
    // Sin email -> no hay invitaciones a aceptar (silencioso, mirror Django).
    let Some(email) = user.email.as_ref() else {
        return Ok(StatusCode::NO_CONTENT);
    };
    if body.invitations.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    // ── 1. Cargar invitaciones que matchean pk ∈ payload AND email == user.email
    //
    // El doble filtro (pk + email) es la defensa de Django contra que un
    // usuario acepte la invitación de OTRO conociendo su UUID. Crucial
    // mantenerlo en Rust.
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

    // ── 2. Reactivar membresías existentes con el rol de la invitación.
    //
    // Mirror de:
    //   WorkspaceMember.objects.filter(workspace_id=invitation.workspace_id,
    //                                  member=request.user)
    //                          .update(is_active=True, role=invitation.role)
    //
    // Iteramos para preservar el `role` por invitación (un UPDATE bulk con
    // mismo rol perdería la granularidad).
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

    // ── 3. Bulk insert de membresías nuevas con ON CONFLICT DO NOTHING.
    //
    // La unique constraint `workspace_member_unique_workspace_member_when_deleted_at_null`
    // cubre (workspace_id, member_id) cuando deleted_at IS NULL — mirror del
    // `ignore_conflicts=True` de Django.
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
            .do_nothing()
            .to_owned(),
        )
        .do_nothing()
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    // ── 4. Soft-delete de las invitaciones procesadas.
    //
    // Django hace .delete() (con soft delete habilitado a nivel de manager
    // base). Replicamos seteando deleted_at en lugar de DELETE físico para
    // mantener trazabilidad y consistencia con el resto del codebase.
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
