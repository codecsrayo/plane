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

// Alias para legibilidad ÃÂ¢ÃÂÃÂ `.active()` filtra `deleted_at IS NULL`
// (implementado via `impl_soft_delete!` en entities/mod.rs)

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ DTOs ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

/// RepresentaciÃÂÃÂ³n pÃÂÃÂºblica del usuario autenticado (`/users/me/`).
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
///
/// Mirror exacto de `UserMeSettingsSerializer`
/// (`apps/api/plane/app/serializers/user.py:90-138`): expone SOLO
/// `["id", "email", "workspace"]`. El bloque `workspace` resuelve server-side
/// la lÃÂÃÂ³gica de redirecciÃÂÃÂ³n que consume el SPA al iniciar sesiÃÂÃÂ³n.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserSettingsResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub workspace: UserSettingsWorkspace,
}

/// Bloque `workspace` dentro de `/users/me/settings/`.
///
/// La shape es asimÃÂÃÂ©trica (paridad con Django):
/// - Cuando hay `last_workspace_id` vÃÂÃÂ¡lido y membresÃÂÃÂ­a activa: se incluyen
///   `last_workspace_name` y `last_workspace_logo`.
/// - En caso contrario: esas dos claves se omiten del JSON (no se emiten
///   como `null`).
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Conversiones ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Handlers ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

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

/// DELETE /api/users/me/ ÃÂ¢ÃÂÃÂ desactiva la cuenta del usuario.
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
    // Verificar que no es el ÃÂÃÂºnico admin en algÃÂÃÂºn workspace activo
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
/// Retorna si el usuario estÃÂÃÂ¡ autenticado. No requiere auth.
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

    // `workspace_invites` = count de WorkspaceMemberInvite por email del user.
    // Mirror: `WorkspaceMemberInvite.objects.filter(email=obj.email).count()`.
    // `WorkspaceMemberInvite` hereda de `BaseModel ÃÂ¢ÃÂÃÂ AuditModel ÃÂ¢ÃÂÃÂ SoftDeleteModel`
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

    // Rama 1: hay `last_workspace_id` + membresÃÂÃÂ­a activa del user en ese workspace.
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
        // `workspace.logo_asset.asset_url` ÃÂ¢ÃÂÃÂ `/api/assets/v2/static/{id}/`
        // Django: `"" ` si no hay asset (string vacÃÂÃÂ­o, no None).
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
        // Rama 2: sin last_workspace vÃÂÃÂ¡lido ÃÂ¢ÃÂÃÂ fallback = workspace mÃÂÃÂ¡s antiguo
        // donde el user es miembro activo.
        // Mirror: `.filter(workspace_member__member_id=obj.id,
        //                  workspace_member__is_active=True).order_by("created_at").first()`.
        //
        // Implementado como JOIN single-query para mantener paridad con Django:
        // una versiÃÂÃÂ³n previa hacÃÂÃÂ­a `workspace_members.find().order_by(wm.created_at)
        // .limit(1)` y luego `workspaces.find_by_id(...)`. Ese patrÃÂÃÂ³n rompe en dos
        // dimensiones respecto a Django:
        //   1) Si el user tiene una membresÃÂÃÂ­a activa en un workspace
        //      soft-deleted MÃÂÃÂS ANTIGUO que uno vivo, el LIMIT 1 captura la fila
        //      muerta y el find_by_id posterior devuelve None ÃÂ¢ÃÂÃÂ fallback = null.
        //      El ORM de Django filtra `workspaces.deleted_at IS NULL` en el mismo
        //      SELECT antes del LIMIT, asÃÂÃÂ­ que el soft-deleted nunca compite.
        //   2) Ordenaba por `workspace_members.created_at`, pero Django ordena por
        //      `workspaces.created_at` ÃÂ¢ÃÂÃÂ columnas distintas, valores distintos.
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
            // Claves omitidas del JSON (paridad con Django: rama `else` no
            // incluye `last_workspace_name` ni `last_workspace_logo`).
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
    // Solo superusers son instance admins en la implementaciÃÂÃÂ³n base
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
    // Obtener membresÃÂÃÂ­as activas del usuario
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ GET /api/users/me/workspaces/{slug}/project-roles ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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
/// el usuario tiene `is_active = true` Y existe membresÃÂÃÂ­a activa al
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
    //    Equivale a: el usuario debe tener una membresÃÂÃÂ­a activa AL WORKSPACE
    //    ademÃÂÃÂ¡s de a los proyectos. Si no la tiene, devolvemos mapa vacÃÂÃÂ­o
    //    (Django retorna {} tambiÃÂÃÂ©n porque el queryset queda vacÃÂÃÂ­o, no 403).
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
    //    asÃÂÃÂ­ que el filtro por igualdad ya descarta NULLs.
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ /api/users/me/workspaces/invitations/ (GET + POST) ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
//
// Espejo de Django `UserWorkspaceInvitationsViewSet` en
// `plane/app/views/workspace/invite.py:244`. El frontend lo consume durante
// onboarding (`apps/web/app/(all)/onboarding/page.tsx:43`) para mostrar
// invitaciones pendientes y aceptarlas en bulk.

/// Workspace embebido en la respuesta de invitaciÃÂÃÂ³n (mirror de
/// `WorkspaceLiteSerializer`: id, name, slug, logo_url).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserInviteWorkspaceLite {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    /// Mirror simplificado de `Workspace.logo_url`. Si el workspace usa
    /// `logo_asset` (no el campo `logo` legacy) este campo serÃÂÃÂ¡ `None` ÃÂ¢ÃÂÃÂ
    /// limitaciÃÂÃÂ³n aceptable para el flujo de onboarding (no hay UI que
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
    /// `invitations` que envÃÂÃÂ­a el frontend en el POST.
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
    // por email vacÃÂÃÂ­o devuelve queryset vacÃÂÃÂ­o).
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
            // invitaciÃÂÃÂ³n (Django con select_related dejarÃÂÃÂ­a workspace=None y
            // el serializer fallarÃÂÃÂ­a ÃÂ¢ÃÂÃÂ preferimos filtrar silenciosamente).
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
///      con el rol de la invitaciÃÂÃÂ³n.
///   2. Si no existe, lo crea via `INSERT ... ON CONFLICT DO NOTHING`
///      (mirror de `bulk_create(ignore_conflicts=True)`).
///   3. Borra las invitaciones procesadas.
///
/// Devuelve 204. Solo procesa invitaciones cuyo `email` coincide con el
/// del usuario autenticado ÃÂ¢ÃÂÃÂ defensa contra UUID-guessing.
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ 1. Cargar invitaciones que matchean pk ÃÂ¢ÃÂÃÂ payload AND email == user.email
    //
    // El doble filtro (pk + email) es la defensa de Django contra que un
    // usuario acepte la invitaciÃÂÃÂ³n de OTRO conociendo su UUID. Crucial
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ 2. Reactivar membresÃÂÃÂ­as existentes con el rol de la invitaciÃÂÃÂ³n.
    //
    // Mirror de:
    //   WorkspaceMember.objects.filter(workspace_id=invitation.workspace_id,
    //                                  member=request.user)
    //                          .update(is_active=True, role=invitation.role)
    //
    // Iteramos para preservar el `role` por invitaciÃÂÃÂ³n (un UPDATE bulk con
    // mismo rol perderÃÂÃÂ­a la granularidad).
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ 3. Bulk insert de membresÃÂÃÂ­as nuevas con ON CONFLICT DO NOTHING.
    //
    // La unique constraint `workspace_member_unique_workspace_member_when_deleted_at_null`
    // cubre (workspace_id, member_id) cuando deleted_at IS NULL ÃÂ¢ÃÂÃÂ mirror del
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
            // PostgreSQL EXIGE repetir el predicado del arbiter index parcial en el
            // conflict target. La unique constraint Django (ver
            // plane/db/models/workspace.py:215-223) es
            //   UNIQUE (workspace_id, member_id) WHERE deleted_at IS NULL
            // y sin `.target_and_where(...)` el INSERT revienta con
            // "there is no unique or exclusion constraint matching the ON CONFLICT
            // specification" ÃÂ¢ÃÂÃÂ el otro unique existente (unique_together sobre las
            // 3 columnas incluyendo deleted_at) tampoco matchea porque el conflict
            // target solo menciona 2. Ver
            // https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT.
            .target_and_where(Expr::col(workspace_members::Column::DeletedAt).is_null())
            .do_nothing()
            .to_owned(),
        )
        .do_nothing()
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ 4. Soft-delete de las invitaciones procesadas.
    //
    // Django hace .delete() (con soft delete habilitado a nivel de manager
    // base). Replicamos seteando deleted_at en lugar de DELETE fÃÂÃÂ­sico para
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
// ME ACTIVITIES  (UserActivityEndpoint)
// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
//
// GET /api/users/me/activities/
//
// Mirror de Django `UserActivityEndpoint.get`
// (plane/app/views/user/base.py:380).
//
// Devuelve TODAS las `IssueActivity` del requester en cualquier workspace/
// proyecto, con paginaciÃÂÃÂ³n cursor estilo Django.
//
// A diferencia de `WorkspaceUserActivityEndpoint`:
//   - No filtra por workspace/proyecto (es cross-workspace).
//   - No excluye field in (comment|vote|reaction|draft) ÃÂ¢ÃÂÃÂ Django tampoco lo hace
//     en este endpoint (sÃÂÃÂ³lo en el workspace-scoped). Ver Django: queryset
//     puro `actor=request.user`.
//   - No requiere membership checks (sÃÂÃÂ³lo actividades del propio usuario).
//   - `default_per_page = 1000` (mirror de `BasePaginator.get_per_page`).
//
// Response shape idÃÂÃÂ©ntico a `BasePaginator.paginate`, los DTOs de detalle se
// re-usan desde `routes::workspaces`.

/// Query params de `GET /users/me/activities/`.
#[derive(Debug, Deserialize)]
pub struct MeActivitiesQuery {
    pub per_page: Option<u64>,
    pub cursor: Option<String>,
    pub order_by: Option<String>,
}

/// `GET /api/users/me/activities/`
///
/// Actividades (issue activities) del requester en todos los workspaces.
/// Mirror de `UserActivityEndpoint` en Django.
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

    // Mirror de `BasePaginator.get_per_page`: default_per_page=1000, max_per_page=1000.
    const DEFAULT_PER_PAGE: u64 = 1000;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Base query ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
    // Django: IssueActivity.objects.filter(actor=request.user)
    //
    // SECURITY NOTE: el filtro por `actor_id = auth_user.id` es el ÃÂÃÂºnico
    // control de acceso necesario ÃÂ¢ÃÂÃÂ un usuario sÃÂÃÂ³lo puede ver SUS propias
    // actividades. No hay IDOR posible porque el actor se deriva de la
    // credencial, no del path.
    // Soft-delete: filtramos `deleted_at IS NULL` para alinear con el manager
    // base del resto del codebase Rust (el queryset de Django aplica el mismo
    // filtro vÃÂÃÂ­a `SoftDeleteManager` a nivel de modelo).
    let base = issue_activities::Entity::find()
        .filter(issue_activities::Column::ActorId.eq(user.id))
        .filter(issue_activities::Column::DeletedAt.is_null());

    // Total antes de paginar (mismo filtro, sin orden ni offset).
    let total_count = base.clone().count(db).await.map_err(AppError::Database)?;

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Ordenamiento (default -created_at) ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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
        // Cualquier columna no whitelisted ÃÂ¢ÃÂÃÂ fallback seguro al default.
        // Esto previene SQL injection vÃÂÃÂ­a `order_by` y alinea con el spirit
        // del `BasePaginator` de Django (que sÃÂÃÂ³lo acepta columnas del modelo).
        _ => base.order_by_desc(issue_activities::Column::CreatedAt),
    };

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Fetch de la pÃÂÃÂ¡gina ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Batch-fetch de objetos relacionados (evitar N+1) ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
    //
    // Como el actor siempre es el requester, reutilizamos el `users::Model`
    // que ya trae AnyAuth, evitando una query extra.

    let actor_avatar_url = if let Some(asset_id) = user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !user.avatar.is_empty() {
        Some(user.avatar.clone())
    } else {
        None
    };

    // Issue IDs de la pÃÂÃÂ¡gina actual
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

    // Workspace IDs ÃÂ¢ÃÂÃÂ cross-workspace, por eso batch-fetch (a diferencia del
    // endpoint workspace-scoped que resuelve uno solo).
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Ensamblar respuesta ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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


// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ GET /users/me/workspaces/{slug}/activity-graph/ ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

/// Actividad del usuario en el workspace agrupada por fecha (ÃÂÃÂºltimos 6 meses).
///
/// Espejo de `UserActivityGraphEndpoint`
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ GET /users/me/workspaces/{slug}/issues-completed-graph/ ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

/// Issues completados por el usuario en el workspace agrupados por semana del mes.
///
/// Espejo de `UserIssueCompletedGraphEndpoint`
/// (`apps/api/plane/app/views/workspace/user.py`).
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/{slug}/issues-completed-graph/",
    tag = "Users",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("month" = Option<i32>, Query, description = "Mes (1-12, default 1)"),
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

    // Espejo Django: week = EXTRACT(WEEK FROM completed_at) % 4
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ GET /users/me/workspaces/{slug}/dashboard/ ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

/// Dashboard del usuario: actividad reciente + issues completados + estadÃÂÃÂ­sticas.
///
/// Espejo de `UserWorkspaceDashboardEndpoint`
/// (`apps/api/plane/app/views/workspace/base.py`).
#[utoipa::path(
    get,
    path = "/api/users/me/workspaces/{slug}/dashboard/",
    tag = "Users",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("month" = Option<i32>, Query, description = "Mes para issues completados"),
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Actividad reciente (ÃÂÃÂºltimos 3 meses) ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Issues completados este mes (por semana) ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ Conteos de issues ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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

    // ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ State distribution ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ
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

// ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ GET /users/last-visited-workspace/ ÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂÃÂ¢ÃÂÃÂ

/// Retorna el ÃÂÃÂºltimo workspace visitado por el usuario con sus proyectos.
///
/// Espejo de `UserLastProjectWithWorkspaceEndpoint`
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
    // Leer last_workspace_id del perfil del usuario
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

// ── Email update endpoints ────────────────────────────────────────────────────
// Mirror de `UserMeEndpoint.generate_email_code` y `UserMeEndpoint.update_email`
// en Django (`plane/app/views/user/base.py`).

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GenerateEmailCodeRequest {
    /// Nuevo email al que enviar el código de verificación.
    pub email: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEmailRequest {
    /// Nuevo email a establecer.
    pub email: String,
    /// Código de 6 dígitos enviado al nuevo email.
    pub code: String,
}

/// Genera y envía un código de verificación al nuevo email del usuario.
///
/// `POST /users/me/email/generate-code`
///
/// Mirror de `UserMeEndpoint.generate_code` en Django.
/// El código se almacena en Redis con TTL de 10 minutos bajo la clave
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

    // Validar que no esté vacío
    if new_email.is_empty() {
        return Err(AppError::BadRequest("Email is required".into()));
    }

    // Validar que sea distinto al email actual
    if user.email.as_deref() == Some(new_email.as_str()) {
        return Err(AppError::BadRequest(
            "New email must be different from current email".into(),
        ));
    }

    // Verificar que el email no esté en uso
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

    // Generar código de 6 dígitos
    let token = format!("{:06}", rand::random::<u32>() % 900_000 + 100_000);

    let cache_key = format!("magic_email_update_{}_{}", user.id, new_email);
    let cache_value = serde_json::json!({ "token": token }).to_string();

    // Guardar en Redis con TTL 600s (10 min)
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

    // En producción se enviaría el email; aquí logueamos el código para debugging
    tracing::info!(
        user_id = %user.id,
        new_email = %new_email,
        "Email update code generated (in production this would be emailed)"
    );

    Ok(Json(serde_json::json!({
        "message": "Verification code sent to email"
    })))
}

/// Verifica el código y actualiza el email del usuario.
///
/// `POST /users/me/email`
///
/// Mirror de `UserMeEndpoint.update_email` en Django.
/// Invalida la sesión actual tras el cambio (el usuario debe re-autenticarse).
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

    // Verificar disponibilidad del email
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

    // Verificar código en Redis
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

    // Actualizar email del usuario
    let user_model = users::Entity::find_by_id(user.id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: users::ActiveModel = user_model.into();
    active.email = Set(Some(new_email.clone()));
    active.is_email_verified = Set(false);
    active.update(&state.db).await.map_err(AppError::Database)?;

    // Eliminar el código de Redis
    let _ = state.redis.del::<i64, _>(&cache_key).await;

    tracing::info!(user_id = %user.id, new_email = %new_email, "User email updated");

    Ok(Json(serde_json::json!({
        "message": "Email updated successfully. Please sign in again."
    })))
}
