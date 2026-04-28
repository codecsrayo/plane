// src/routes/workspaces.rs
// NOTE: get_user_profile added below — mirror of WorkspaceUserProfileEndpoint
//! Workspace endpoints — Phase 2.
//!
//! Equivalent to `plane/app/views/workspace/base.py` and `member.py` in Django.
//! Authentication: session cookie **or** API key (via `AnyAuth`).
//! Authorization: active workspace membership; Admin role required for mutations.

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use openssl::memcmp as ct_memcmp;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement, TransactionTrait,
};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        permissions::{require_workspace_admin, ROLE_ADMIN, ROLE_GUEST, ROLE_MEMBER, ROLE_VIEWER},
    },
    entities::{
        draft_issues, issue_activities, issues, profiles, project_members, projects, users,
        workspace_member_invites, workspace_members, workspaces,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::{
        color::get_random_color, csv_sanitize::sanitize_csv_cell,
        instance_config::get_config_value,
        pagination,
        posthog::{track_event, EVENT_WORKSPACE_DELETED},
        soft_delete::SoftDeleteExt, url::contains_url,
    },
    AppState,
};

// ─── Permission aliases ──────────────────────────────────────────────────────

/// Local alias for [`require_workspace_admin`].
///
/// Allows using `require_admin(&member)?` concisely across all
/// handlers in this module without importing an additional symbol for each call.
#[inline(always)]
fn require_admin(member: &workspace_members::Model) -> Result<(), AppError> {
    require_workspace_admin(member)
}

// ─── Reserved slugs ──────────────────────────────────────────────────────────

/// Reserved slugs — exact mirror of `plane/utils/constants.py::RESTRICTED_WORKSPACE_SLUGS`.
/// Update here when the Python file is modified.
const RESTRICTED_SLUGS: &[&str] = &[
    "404",
    "accounts",
    "api",
    "create-workspace",
    "god-mode",
    "installations",
    "invitations",
    "onboarding",
    "profile",
    "spaces",
    "workspace-invitations",
    "password",
    "flags",
    "monitor",
    "monitoring",
    "ingest",
    "plane-pro",
    "plane-ultimate",
    "enterprise",
    "plane-enterprise",
    "disco",
    "silo",
    "chat",
    "calendar",
    "drive",
    "channels",
    "upgrade",
    "billing",
    "sign-in",
    "sign-up",
    "signin",
    "signup",
    "config",
    "live",
    "admin",
    "m",
    "import",
    "importers",
    "integrations",
    "integration",
    "configuration",
    "initiatives",
    "initiative",
    "workflow",
    "workflows",
    "epics",
    "epic",
    "story",
    "mobile",
    "dashboard",
    "desktop",
    "onload",
    "real-time",
    "one",
    "pages",
    "business",
    "pro",
    "settings",
    "license",
    "licenses",
    "instances",
    "instance",
];

// ─── Slug helpers ────────────────────────────────────────────────────────────
//
// `workspace_by_slug` and `require_workspace_member` live in `helpers.rs`
// and are imported above. Only local slug format validation remains.

fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.is_empty() || slug.len() > 48 {
        return Err(AppError::BadRequest(
            "Slug must be between 1 and 48 characters".into(),
        ));
    }
    if !slug.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest(
            "Slug can only contain letters, numbers, hyphens and underscores".into(),
        ));
    }
    if RESTRICTED_SLUGS.contains(&slug) {
        return Err(AppError::BadRequest("Slug is reserved".into()));
    }
    Ok(())
}

// ─── DTOs ────────────────────────────────────────────────────────────────────

/// Public representation of a workspace.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    /// Computed logo URL: prefers logo_asset, falls back to logo field.
    /// Mirrors Django's `Workspace.logo_url` property.
    pub logo_url: Option<String>,
    pub organization_size: Option<String>,
    pub owner_id: Uuid,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub timezone: String,
    pub background_color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Number of active members (excludes bots). `None` in contexts without joins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_members: Option<i64>,
    /// Authenticated user's role in this workspace (only in list/detail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i16>,
}

impl WorkspaceResponse {
    /// Build response from a workspace model.
    ///
    /// `logo_url` mirrors Django's computed property:
    ///   - If `logo_asset` exists → use its `asset_url` (resolved via join)
    ///   - Else if `logo` is set → use it directly
    ///   - Else → `None`
    ///
    /// When the caller hasn't performed a join on `file_assets`, pass `None`
    /// for `logo_asset_url` and the function falls back to `ws.logo`.
    fn from_model(
        ws: &workspaces::Model,
        total_members: Option<i64>,
        role: Option<i16>,
        logo_asset_url: Option<String>,
    ) -> Self {
        // Compute logo_url mirroring Django: logo_asset > logo > None
        let logo_url = logo_asset_url.or_else(|| ws.logo.clone());

        Self {
            id: ws.id,
            name: ws.name.clone(),
            slug: ws.slug.clone(),
            logo: ws.logo.clone(),
            logo_url,
            organization_size: ws.organization_size.clone(),
            owner_id: ws.owner_id,
            created_by: ws.created_by_id,
            updated_by: ws.updated_by_id,
            timezone: ws.timezone.clone(),
            background_color: ws.background_color.clone(),
            created_at: ws.created_at.into(),
            updated_at: ws.updated_at.into(),
            total_members,
            role,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub slug: String,
    pub organization_size: Option<String>,
    pub timezone: Option<String>,
    /// Logo URL — mirrors Django's `logo` TextField.
    pub logo: Option<String>,
    /// Company role of the creating user — stored on WorkspaceMember.
    /// Mirrors Django: `request.data.get("company_role", "")`.
    pub company_role: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub organization_size: Option<String>,
    pub timezone: Option<String>,
    pub background_color: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMemberResponse {
    pub id: Uuid,
    pub member_id: Uuid,
    pub role: i16,
    pub company_role: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&workspace_members::Model> for WorkspaceMemberResponse {
    fn from(m: &workspace_members::Model) -> Self {
        Self {
            id: m.id,
            member_id: m.member_id,
            role: m.role,
            company_role: m.company_role.clone(),
            is_active: m.is_active,
            created_at: m.created_at.into(),
        }
    }
}

// ─── Django-compat DTOs for member listing ──────────────────────────────────
//
// Django exposes workspace members with the `member` NESTED as a user object
// (see `WorkSpaceMemberSerializer` in
// `apps/api/plane/app/serializers/workspace.py:85-90` with
// `member = UserLiteSerializer(read_only=True)`). The frontend consumes this
// shape directly in `workspace-member.store.ts:240`:
//
//     set(this.memberRoot?.memberMap, member.member.id, { ...member.member, ... });
//
// The flat DTO above (`WorkspaceMemberResponse`) IS NOT compatible with that
// access to `member.member.id` and causes
//   TypeError: Cannot read properties of undefined (reading 'id')
// when the frontend points to Rust. These new DTOs exactly reflect the
// output of the Django serializer with `fields=("id", "member", "role")`.

/// Mirror of `UserLiteSerializer` + admin branch of `UserAdminLiteSerializer`
/// (`apps/api/plane/app/serializers/user.py:141-170`).
///
/// `email` and `last_login_medium` are only emitted when caller is non-Guest
/// (Django: `if workspace_member.role > 5` in
/// `apps/api/plane/app/views/workspace/member.py:51`). `skip_serializing_if`
/// maintains the JSON shape identical to Django when caller is Guest —
/// keys do not appear, they are not sent as `null`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserLiteDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub display_name: String,
    /// Only present if caller is non-Guest (parity with `UserAdminLiteSerializer`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Only present if caller is non-Guest (parity with `UserAdminLiteSerializer`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_medium: Option<String>,
}

/// Builds a `UserLiteDto` from SeaORM model, applying the same
/// `avatar_url` logic as `User.avatar_url` in Django
/// (`apps/api/plane/db/models/user.py:142-151`):
///
///   1. If `avatar_asset_id` exists, returns `/api/assets/v2/static/{id}/`.
///      This path is the `USER_AVATAR` branch of `FileAsset.asset_url`
///      (`apps/api/plane/db/models/asset.py:79-100`), so it's composed
///      directly from the UUID without a JOIN to `file_assets` —
///      the `entity_type` of the asset pointed to by `users.avatar_asset_id` is
///      always `USER_AVATAR` by Django model invariant.
///   2. Otherwise, returns legacy `avatar` string if not empty.
///   3. In any other case, `None`.
///
/// `is_admin` enables fields that `UserAdminLiteSerializer` adds over
/// `UserLiteSerializer`.
pub(crate) fn user_to_lite(user: &users::Model, is_admin: bool) -> UserLiteDto {
    let avatar_url = if let Some(asset_id) = user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !user.avatar.is_empty() {
        Some(user.avatar.clone())
    } else {
        None
    };

    UserLiteDto {
        id: user.id,
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        avatar: user.avatar.clone(),
        avatar_url,
        is_bot: user.is_bot,
        display_name: user.display_name.clone(),
        email: if is_admin { user.email.clone() } else { None },
        last_login_medium: if is_admin {
            Some(user.last_login_medium.clone())
        } else {
            None
        },
    }
}

/// Mirror of `WorkSpaceMemberSerializer` with `fields=("id","member","role")`
/// (explicit use in
/// `apps/api/plane/app/views/workspace/member.py:52,54,71,73`). DOES NOT include
/// `company_role`, `is_active`, `created_at`, etc. — the Django serializer with
/// that whitelist doesn't emit them either, and emitting extra fields would break
/// consumers doing `{ ...member }` spread (any extra field would override
/// MobX store properties).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMemberNestedResponse {
    pub id: Uuid,
    pub member: UserLiteDto,
    pub role: i16,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role: i16,
    pub company_role: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub role: i16,
    pub accepted: bool,
    pub message: Option<String>,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<&workspace_member_invites::Model> for InvitationResponse {
    fn from(i: &workspace_member_invites::Model) -> Self {
        Self {
            id: i.id,
            email: i.email.clone(),
            role: i.role,
            accepted: i.accepted,
            message: i.message.clone(),
            workspace_id: i.workspace_id,
            created_at: i.created_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateInvitationRequest {
    pub emails: Vec<InviteEmail>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InviteEmail {
    pub email: String,
    pub role: i16,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SlugCheckQuery {
    pub slug: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SlugCheckResponse {
    pub status: bool,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// `GET /api/workspace-slug-check/?slug={slug}`
///
/// Verifies slug availability. Does not require authentication.
#[utoipa::path(
    get,
    path = "/api/workspace-slug-check/",
    tag = "Workspaces",
    params(("slug" = String, Query, description = "Slug to check")),
    responses(
        (status = 200, description = "Availability result", body = SlugCheckResponse),
    )
)]
pub async fn slug_check(
    State(state): State<AppState>,
    Query(q): Query<SlugCheckQuery>,
) -> Result<Json<SlugCheckResponse>, AppError> {
    let slug = q.slug.to_lowercase();
    if RESTRICTED_SLUGS.contains(&slug.as_str()) {
        return Ok(Json(SlugCheckResponse { status: false }));
    }
    // Do not use .active() here: the DB unique constraint applies to ALL
    // rows (including soft-deleted). If we filter only active ones, we report
    // the slug as available but the real INSERT fails due to the constraint.
    let exists = workspaces::Entity::find()
        .filter(workspaces::Column::Slug.eq(&slug))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(SlugCheckResponse { status: exists == 0 }))
}

/// `GET /api/workspaces/`
///
/// Lists workspaces where the authenticated user is an active member.
#[utoipa::path(
    get,
    path = "/api/workspaces/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    responses(
        (status = 200, description = "List of workspaces", body = Vec<WorkspaceResponse>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_workspaces(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    // User's active memberships
    let memberships = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    if memberships.is_empty() {
        return Ok(Json(vec![]));
    }

    let workspace_ids: Vec<Uuid> = memberships.iter().map(|m| m.workspace_id).collect();
    let member_role_map: std::collections::HashMap<Uuid, i16> = memberships
        .iter()
        .map(|m| (m.workspace_id, m.role))
        .collect();

    let workspaces_list = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Id.is_in(workspace_ids))
        .order_by_asc(workspaces::Column::Name)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let responses: Vec<WorkspaceResponse> = workspaces_list
        .iter()
        .map(|ws| {
            let role = member_role_map.get(&ws.id).copied();
            WorkspaceResponse::from_model(ws, None, role, None)
        })
        .collect();

    Ok(Json(responses))
}

/// `POST /api/workspaces/`
///
/// Creates a new workspace and registers the user as Admin + Owner.
#[utoipa::path(
    post,
    path = "/api/workspaces/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Workspace creation disabled"),
        (status = 409, description = "Slug already exists"),
    )
)]
pub async fn create_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check instance flag
    let disabled = get_config_value(&state, "DISABLE_WORKSPACE_CREATION", Some("0"))
        .await?
        .unwrap_or_default();
    if disabled == "1" {
        return Err(AppError::Forbidden);
    }

    if body.name.is_empty() || body.name.len() > 80 {
        return Err(AppError::BadRequest(
            "Name must be between 1 and 80 characters".into(),
        ));
    }
    if contains_url(&body.name) {
        return Err(AppError::BadRequest(
            "Name cannot contain a URL".into(),
        ));
    }
    let slug = body.slug.to_lowercase();
    validate_slug(&slug)?;

    // We don't perform a pre-check for slug availability: it's a TOCTOU pattern
    // (time-of-check vs time-of-use). Two concurrent requests can both pass verification
    // and one fails on INSERT. We let the DB unique constraint be the source of truth
    // and map the violation to 409 Conflict (like Django).

    let ws_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    // Create workspace
    let new_ws = workspaces::ActiveModel {
        id: Set(ws_id),
        name: Set(body.name.clone()),
        slug: Set(slug),
        logo: Set(body.logo.clone()),
        organization_size: Set(body.organization_size.clone()),
        owner_id: Set(user.id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        timezone: Set(body.timezone.unwrap_or_else(|| "UTC".into())),
        background_color: Set(get_random_color()),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        logo_asset_id: Set(None),
    };
    let ws = new_ws.insert(&txn).await.map_err(|e| {
        // Map unique constraint violation → 409 Conflict (Django mirror)
        if let sea_orm::DbErr::Query(ref runtime_err) = e {
            let msg = runtime_err.to_string();
            if msg.contains("unique") || msg.contains("duplicate key") {
                return AppError::Conflict(
                    "The workspace with the slug already exists".into(),
                );
            }
        }
        // Also capture Exec errors that SeaORM might emit on insert
        if let sea_orm::DbErr::Exec(ref runtime_err) = e {
            let msg = runtime_err.to_string();
            if msg.contains("unique") || msg.contains("duplicate key") {
                return AppError::Conflict(
                    "The workspace with the slug already exists".into(),
                );
            }
        }
        tracing::error!(error = %e, "Failed to insert workspace");
        AppError::Database(e)
    })?;

    // Create Admin membership — mirrors Django: role=20, company_role from request
    let new_member = workspace_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        workspace_id: Set(ws_id),
        member_id: Set(user.id),
        role: Set(ROLE_ADMIN),
        company_role: Set(body.company_role.clone()),
        is_active: Set(true),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        view_props: Set(serde_json::json!({})),
        default_props: Set(serde_json::json!({})),
        issue_props: Set(serde_json::json!({})),
        explored_features: Set(serde_json::json!({})),
        getting_started_checklist: Set(serde_json::json!({})),
        tips: Set(serde_json::json!({})),
    };
    new_member.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert workspace member");
        AppError::Database(e)
    })?;

    txn.commit().await.map_err(AppError::Database)?;

    // Enqueue initial data seed — best-effort (does not block response).
    // Uses the shared pg_pool from AppState instead of opening a new connection.
    {
        use crate::jobs::workspace_seed::WorkspaceSeedJob;
        use apalis::prelude::Storage;
        use apalis_sql::postgres::PostgresStorage;

        let mut seed_storage: PostgresStorage<WorkspaceSeedJob> =
            PostgresStorage::new(state.pg_pool.clone());
        if let Err(e) = seed_storage
            .push(WorkspaceSeedJob {
                workspace_id: ws_id,
                owner_id: user.id,
                workspace_name: body.name.clone(),
            })
            .await
        {
            tracing::warn!(error = %e, "Failed to enqueue workspace seed job");
        }
    }

    let resp = WorkspaceResponse::from_model(&ws, Some(1), Some(ROLE_ADMIN), None);
    Ok((StatusCode::CREATED, Json(resp)))
}

/// `GET /api/workspaces/{slug}/`
///
/// Returns workspace detail. Requires active membership.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Workspace detail", body = WorkspaceResponse),
        (status = 403, description = "Not a member"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    // Non-members receive 404 (not 403) to avoid leaking workspace existence
    // to unauthorized users — standard security practice, symmetric with `get_project`.
    let member = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let total = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(WorkspaceResponse::from_model(
        &ws,
        Some(total as i64),
        Some(member.role),
        None,
    )))
}

/// `PATCH /api/workspaces/{slug}/`
///
/// Updates name or other fields. Requires Admin role.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    request_body = UpdateWorkspaceRequest,
    responses(
        (status = 200, description = "Updated workspace", body = WorkspaceResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    if let Some(ref name) = body.name {
        if name.is_empty() || name.len() > 80 {
            return Err(AppError::BadRequest(
                "Name must be between 1 and 80 characters".into(),
            ));
        }
        if contains_url(name) {
            return Err(AppError::BadRequest(
                "Name cannot contain a URL".into(),
            ));
        }
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspaces::ActiveModel = ws.into();
    if let Some(name) = body.name {
        active.name = Set(name);
    }
    if let Some(org_size) = body.organization_size {
        active.organization_size = Set(Some(org_size));
    }
    if let Some(tz) = body.timezone {
        active.timezone = Set(tz);
    }
    if let Some(bg) = body.background_color {
        active.background_color = Set(bg);
    }
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(WorkspaceResponse::from_model(
        &updated,
        None,
        Some(member.role),
        None,
    )))
}

/// `DELETE /api/workspaces/{slug}/`
///
/// Workspace soft-delete. Only the owner can delete it.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 204, description = "Workspace deleted"),
        (status = 403, description = "Forbidden — only owner can delete"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    // Only the owner can delete the workspace (Django equivalent)
    if ws.owner_id != user.id {
        let member = require_workspace_member(&state.db, ws.id, user.id).await?;
        require_admin(&member)?;
        // Admin non-owner can only delete if they have permission — here only owner
        return Err(AppError::Forbidden);
    }

    let now = chrono::Utc::now().fixed_offset();
    let ws_id = ws.id;
    let ws_name = ws.name.clone();
    let ws_slug = ws.slug.clone();

    // Mirror of `WorkspaceViewSet.destroy`
    // (`apps/api/plane/app/views/workspace/base.py:184-201`):
    //
    //   Profile.objects.filter(last_workspace_id=id).update(last_workspace_id=None)
    //   return super().destroy(...)
    //
    // Without this step, profiles pointing to the deleted workspace continue
    // pointing to it — `GET /users/me/profile/` returns the dead ID via
    // `profile_to_response` (users.rs) and the frontend redirects to the dead href
    // instead of the workspace selection screen. Wrapped in transaction so
    // profile cleanup and workspace soft-delete are atomic against concurrent readers.
    //
    // Parity note: Django uses `QuerySet.update()`, which explicitly
    // **does not** trigger `auto_now=True` on `updated_at`
    // (`apps/api/plane/db/mixins.py:20`). Affected profiles retain their
    // previous `updated_at`. We replicate that behavior: only `last_workspace_id`
    // is touched, not `updated_at`.
    let txn = state.db.begin().await.map_err(AppError::Database)?;

    profiles::Entity::update_many()
        .col_expr(
            profiles::Column::LastWorkspaceId,
            Expr::value(Option::<Uuid>::None),
        )
        .filter(profiles::Column::LastWorkspaceId.eq(ws_id))
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    // ── Restricted soft-delete cascade ───────────────────────────────────
    //
    // Soft-deletes direct workspace dependencies to maintain the
    // invisibility assumed by listing endpoints and helpers —
    // audited: list_workspaces, list_user_workspaces, list_members,
    // list_invitations, list_projects, list_projects_detail,
    // list_project_members, list_project_invitations + workspace_by_slug,
    // require_workspace_member, project_by_id, project_member_for_user
    // (all filter with `.active()`).
    //
    // PARTIAL parity with Django: destroy() calls super().destroy() which
    // triggers `soft_delete_related_objects.delay(...)` — a Celery job
    // reflective and recursive (apps/api/plane/bgtasks/deletion_task.py:17)
    // that also traverses transitive reverse relations of `projects`
    // (issues, cycles, modules, pages, views, labels, states, ...).
    //
    // Here we only apply the first level: workspace_members,
    // workspace_member_invites, projects, project_members. The rest
    // (issues/cycles/etc.) don't need marking because their endpoints
    // go through project_by_id → workspace_by_slug, which already filter
    // `.active()` at higher levels: nothing accessible via API survives
    // this soft-delete. The full purge of orphaned rows is done by
    // the hard-delete job (Django `hard_delete` parity).
    //
    // We use `col_expr(DeletedAt, ...)` just like the profiles update
    // above: exact mirror of Django's `QuerySet.update()` — DOES NOT
    // trigger `updated_at` bump (apps/api/plane/db/mixins.py:20). The
    // `DeletedAt.is_null()` filter preserves the historical timestamp of
    // rows that were already soft-deleted before this delete (e.g.
    // a member removed months ago keeps their original `deleted_at`).
    let now_expr = Expr::value(Some(now));

    workspace_members::Entity::update_many()
        .col_expr(workspace_members::Column::DeletedAt, now_expr.clone())
        .filter(workspace_members::Column::WorkspaceId.eq(ws_id))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    workspace_member_invites::Entity::update_many()
        .col_expr(
            workspace_member_invites::Column::DeletedAt,
            now_expr.clone(),
        )
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws_id))
        .filter(workspace_member_invites::Column::DeletedAt.is_null())
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    project_members::Entity::update_many()
        .col_expr(project_members::Column::DeletedAt, now_expr.clone())
        .filter(project_members::Column::WorkspaceId.eq(ws_id))
        .filter(project_members::Column::DeletedAt.is_null())
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    projects::Entity::update_many()
        .col_expr(projects::Column::DeletedAt, now_expr)
        .filter(projects::Column::WorkspaceId.eq(ws_id))
        .filter(projects::Column::DeletedAt.is_null())
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    let mut active: workspaces::ActiveModel = ws.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&txn).await.map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    // Mirror of `track_event.delay(event_name=WORKSPACE_DELETED, ...)` in
    // `base.py:188-200`. Django runs it via Celery; we via `tokio::spawn`
    // inside `utils::posthog::track_event`. The event is emitted only after
    // successful commit — if the transaction had failed, we don't want to report
    // a delete that didn't happen.
    //
    // Note on `role`: Django hardcodes `"role": "owner"` in the view and then
    // `preprocess_data_properties` recalculates it by querying `Workspace.objects`
    // with the slug. But at this point the workspace is already soft-deleted, and the
    // default manager filters by `deleted_at IS NULL` → `DoesNotExist` →
    // `"role": "unknown"`. In Rust we are deterministic: we already validated above
    // that `ws.owner_id == user.id`, so the role is always `owner`.
    let mut props = serde_json::Map::new();
    props.insert("user_id".into(), serde_json::json!(user.id.to_string()));
    props.insert("workspace_id".into(), serde_json::json!(ws_id.to_string()));
    props.insert("workspace_slug".into(), serde_json::json!(ws_slug));
    props.insert("workspace_name".into(), serde_json::json!(ws_name));
    props.insert("role".into(), serde_json::json!("owner"));
    props.insert("deleted_at".into(), serde_json::json!(now.to_rfc3339()));
    track_event(
        &state,
        user.id,
        EVENT_WORKSPACE_DELETED,
        slug,
        props,
    );

    Ok(StatusCode::NO_CONTENT)
}

// ─── Members ─────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/members/`
///
/// Lists active workspace members with nested user.
///
/// Exact mirror of `WorkSpaceMemberViewSet.list`
/// (`apps/api/plane/app/views/workspace/member.py:45-55`). Response shape
/// identical to Django with `WorkSpaceMemberSerializer(fields=("id","member",
/// "role"))` — necessary because the frontend (`workspace-member.store.ts:240`)
/// accesses `member.member.id` and fails with TypeError if shape is not nested.
///
/// The admin/no-admin branch follows Django: `if workspace_member.role > 5`
/// (ROLE_GUEST) uses `UserAdminLiteSerializer` (includes `email`/
/// `last_login_medium`); otherwise `UserLiteSerializer`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/members/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Member list", body = Vec<WorkspaceMemberNestedResponse>),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<WorkspaceMemberNestedResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Django parity: `if workspace_member.role > 5` uses AdminSerializer.
    // ROLE_GUEST = 5 in plane/app/permissions/base.py and in auth/permissions.rs.
    let is_admin = caller.role > ROLE_GUEST;

    // ── 1. Memberships fetch ─────────────────────────────────────────────────
    //
    // Django: `.filter(workspace__slug=self.kwargs.get("slug"))` on the
    // viewset base queryset + additional list-view filters by is_active=True.
    // `.active()` applies soft-delete filter (deleted_at IS NULL).
    let members = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .order_by_asc(workspace_members::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    if members.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // ── 2. Batch-fetch of referenced users ───────────────────────────────────
    //
    // Two queries total (no N+1). Django does the equivalent with
    // `select_related("member", "member__avatar_asset")` — we resolve
    // avatar_url without additional JOIN to `file_assets` because the path is computed
    // directly from `avatar_asset_id` (see `user_to_lite`). Deduplicate via
    // HashSet in case of unexpected duplicates (shouldn't happen, partial unique
    // constraint prevents it, but cost is low for protection).
    let member_ids: Vec<Uuid> = members
        .iter()
        .map(|m| m.member_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let users_vec = users::Entity::find()
        .filter(users::Column::Id.is_in(member_ids))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let users_by_id: std::collections::HashMap<Uuid, users::Model> =
        users_vec.into_iter().map(|u| (u.id, u)).collect();

    // ── 3. Assembling the Django-compatible shape ────────────────────────────
    //
    // Django does INNER JOIN via `select_related("member")`: if a user was
    // hard-deleted but the membership remained orphaned, the row disappears from
    // the resultset (implicit behavior of join required). We replicate with
    // `filter_map`: if there is no entry in `users_by_id`, the row is omitted. This
    // avoids returning `member: null` to the frontend (which would crash anyway in
    // `member.member.id`).
    let response: Vec<WorkspaceMemberNestedResponse> = members
        .iter()
        .filter_map(|m| {
            users_by_id.get(&m.member_id).map(|u| WorkspaceMemberNestedResponse {
                id: m.id,
                member: user_to_lite(u, is_admin),
                role: m.role,
            })
        })
        .collect();

    Ok(Json(response))
}

/// `PATCH /api/workspaces/{slug}/members/{pk}/`
///
/// Updates a member's role. Requires Admin.
/// Cannot downgrade owner.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/members/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Member record UUID"),
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Updated member", body = WorkspaceMemberResponse),
        (status = 400, description = "Cannot downgrade owner"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Member not found"),
    )
)]
pub async fn update_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateMemberRoleRequest>,
) -> Result<Json<WorkspaceMemberResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&caller)?;

    // Validate role
    if ![ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&body.role) {
        return Err(AppError::BadRequest("Invalid role value".into()));
    }

    let target = workspace_members::Entity::find_by_id(pk)
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Do not downgrade workspace owner
    if target.member_id == ws.owner_id && body.role < ROLE_ADMIN {
        return Err(AppError::BadRequest(
            "Cannot downgrade workspace owner".into(),
        ));
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspace_members::ActiveModel = target.into();
    active.role = Set(body.role);
    if let Some(cr) = body.company_role {
        active.company_role = Set(Some(cr));
    }
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(WorkspaceMemberResponse::from(&updated)))
}

/// `DELETE /api/workspaces/{slug}/members/{pk}/`
///
/// Deactivates a member (soft-remove). Requires Admin.
/// The owner cannot be removed.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/members/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Member record UUID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 400, description = "Cannot remove owner"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Member not found"),
    )
)]
pub async fn remove_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&caller)?;

    let target = workspace_members::Entity::find_by_id(pk)
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if target.member_id == ws.owner_id {
        return Err(AppError::BadRequest(
            "Cannot remove the workspace owner".into(),
        ));
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspace_members::ActiveModel = target.into();
    // Mark inactive + soft-delete for consistency
    active.is_active = Set(false);
    active.deleted_at = Set(Some(now));
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Invitations ─────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/invitations/`
///
/// Lists pending workspace invitations. Requires Admin.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/invitations/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Invitation list", body = Vec<InvitationResponse>),
        (status = 403, description = "Forbidden — requires Admin"),
    )
)]
pub async fn list_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<InvitationResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    let invites = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::Accepted.eq(false))
        .order_by_desc(workspace_member_invites::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(invites.iter().map(InvitationResponse::from).collect()))
}

/// `POST /api/workspaces/{slug}/invitations/`
///
/// Creates one or more workspace invitations. Requires Admin.
///
/// Note: email sending will be delegated to `InvitationEmailJob` (Phase 3).
/// For now, the record is persisted and the token remains in the DB.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/invitations/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    request_body = CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitations created", body = Vec<InvitationResponse>),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — requires Admin"),
    )
)]
pub async fn create_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateInvitationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.emails.is_empty() {
        return Err(AppError::BadRequest("emails list is required".into()));
    }

    // ── 1. Role validation — upfront, before any DB query ────────────────────
    //
    // Fail fast avoids a partially invalid request emitting queries
    // or insertions before discovering the error.
    for invite_req in &body.emails {
        if ![ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&invite_req.role) {
            return Err(AppError::BadRequest(format!(
                "Invalid role {} for email {}",
                invite_req.role, invite_req.email
            )));
        }
    }

    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    // ── 2. In-memory payload deduplication ────────────────────────────────────
    //
    // A single email listed twice in the request should not produce two rows;
    // normalize to lowercase for case-insensitive comparison (same as Django).
    let mut seen_in_payload: HashSet<String> = HashSet::with_capacity(body.emails.len());
    let unique_invites: Vec<&InviteEmail> = body.emails
        .iter()
        .filter(|i| seen_in_payload.insert(i.email.to_lowercase()))
        .collect();

    // ── 3. Bulk-check — 1 SELECT replaces N COUNT queries ─────────────────────
    //
    // Before: per email → COUNT(*) WHERE email = ? (N queries)
    // Now: 1 query → SELECT email WHERE email IN (e1, e2, …) AND accepted = false
    let candidate_emails: Vec<String> =
        unique_invites.iter().map(|i| i.email.clone()).collect();

    let already_invited: HashSet<String> = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::Email.is_in(candidate_emails))
        .filter(workspace_member_invites::Column::Accepted.eq(false))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|m| m.email.to_lowercase())
        .collect();

    // ── 4. Build ActiveModels and Models in memory ───────────────────────────
    //
    // UUIDs are generated locally — no `exec_with_returning` needed
    // nor a second query to retrieve newly inserted rows.
    let now = chrono::Utc::now().fixed_offset();
    let mut active_models: Vec<workspace_member_invites::ActiveModel> =
        Vec::with_capacity(unique_invites.len());
    let mut created_models: Vec<workspace_member_invites::Model> =
        Vec::with_capacity(unique_invites.len());

    for invite_req in &unique_invites {
        if already_invited.contains(&invite_req.email.to_lowercase()) {
            tracing::warn!(
                email = %invite_req.email,
                workspace = %ws.slug,
                "Skipping duplicate pending invitation"
            );
            continue;
        }

        let id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();

        active_models.push(workspace_member_invites::ActiveModel {
            id: Set(id),
            workspace_id: Set(ws.id),
            email: Set(invite_req.email.clone()),
            role: Set(invite_req.role),
            accepted: Set(false),
            token: Set(token.clone()),
            message: Set(None),
            responded_at: Set(None),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        });

        // Local Model — reflects exactly what is about to be persisted.
        created_models.push(workspace_member_invites::Model {
            id,
            workspace_id: ws.id,
            email: invite_req.email.clone(),
            role: invite_req.role,
            accepted: false,
            token,
            message: None,
            responded_at: None,
            created_by_id: Some(user.id),
            updated_by_id: Some(user.id),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        });
    }

    // ── 5. Bulk insert in transaction — 1 INSERT … VALUES (…), (…) ───────────
    //
    // If there is nothing new to insert (all were duplicates) the transaction
    // is skipped entirely.
    if !active_models.is_empty() {
        let txn = state.db.begin().await.map_err(AppError::Database)?;
        workspace_member_invites::Entity::insert_many(active_models)
            .exec(&txn)
            .await
            .map_err(AppError::Database)?;
        txn.commit().await.map_err(AppError::Database)?;
    }

    let responses: Vec<InvitationResponse> =
        created_models.iter().map(InvitationResponse::from).collect();
    Ok((StatusCode::CREATED, Json(responses)))
}

/// `DELETE /api/workspaces/{slug}/invitations/{pk}/`
///
/// Invitation soft-delete. Requires Admin.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/invitations/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 204, description = "Invitation deleted"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    let invite = workspace_member_invites::Entity::find_by_id(pk)
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspace_member_invites::ActiveModel = invite.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── GET /api/workspaces/{slug}/workspace-members/me ─────────────────────────
//
// Mirror of Django `WorkspaceMemberUserEndpoint.get` in
// `plane/app/views/workspace/member.py:217`. Returns the authenticated user's
// full workspace membership, with an extra field `draft_issue_count` which the
// frontend uses for "Drafts" badges.
//
// Django behavior: if user DOES NOT have an active membership,
// `WorkspaceMemberMeSerializer(None).data` produces an empty object `{}` with
// status 200 — not 404 or 403. We replicate exactly to not break the
// frontend which assumes stable 200 and only reads optional fields.

/// Response for `GET /api/workspaces/{slug}/workspace-members/me`.
///
/// Mirror of `WorkspaceMemberMeSerializer(model=WorkspaceMember, fields="__all__")`
/// plus the annotated field `draft_issue_count`. All JSONB fields
/// (`view_props`, `default_props`, etc.) are exposed as stored by
/// Postgres, same as in Django.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMemberMeResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: i16,
    pub company_role: Option<String>,
    pub view_props: serde_json::Value,
    pub default_props: serde_json::Value,
    pub issue_props: serde_json::Value,
    pub is_active: bool,
    pub explored_features: serde_json::Value,
    pub getting_started_checklist: serde_json::Value,
    pub tips: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub member: Uuid,
    pub workspace: Uuid,
    pub draft_issue_count: u64,
}

/// `GET /api/workspaces/{slug}/workspace-members/me`
///
/// Returns the authenticated user's workspace membership, or an
/// empty object if not an active member (Django compat).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-members/me",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Current user's workspace membership (or empty object if not a member)"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn get_workspace_member_me(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let membership = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(m) = membership else {
        // Django compat: serializer over None produces {} with status 200.
        return Ok(Json(serde_json::json!({})));
    };

    // draft_issue_count: drafts created by the user in this workspace
    // (not soft-deleted). Mirror of annotated subquery in Django:
    //   DraftIssue.objects.filter(created_by=request.user,
    //                             workspace_id=OuterRef("workspace_id"))
    //     .annotate(count=Count("id"))
    let draft_issue_count = draft_issues::Entity::find()
        .active()
        .filter(draft_issues::Column::CreatedById.eq(user.id))
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp = WorkspaceMemberMeResponse {
        id: m.id,
        created_at: m.created_at.into(),
        updated_at: m.updated_at.into(),
        role: m.role,
        company_role: m.company_role,
        view_props: m.view_props,
        default_props: m.default_props,
        issue_props: m.issue_props,
        is_active: m.is_active,
        explored_features: m.explored_features,
        getting_started_checklist: m.getting_started_checklist,
        tips: m.tips,
        created_by: m.created_by_id,
        updated_by: m.updated_by_id,
        member: m.member_id,
        workspace: m.workspace_id,
        draft_issue_count,
    };

    Ok(Json(serde_json::to_value(resp).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("serialize WorkspaceMemberMeResponse: {e}"))
    })?))
}

// ─── User Profile ─────────────────────────────────────────────────────────────
//
// Mirror of `WorkspaceUserProfileEndpoint.get`
// (`apps/api/plane/app/views/workspace/user.py:280`).
//
// Response shape:
//   { project_data: [...], user_data: { email, first_name, ... } }
//
// `project_data` is only included when `requesting_workspace_member.role >= 15`
// (MEMBER+). Each entry contains issue counters for the target user in that
// project. The filtered projects are those where the REQUESTER is an active
// member (not the target) — same as in Django.

/// Result row for project statistics SQL.
///
/// `logo_props` is retrieved as `String` (serialized JSON) because SeaORM
/// does not implement `FromQueryResult` for `serde_json::Value` directly in
/// raw queries. It is deserialized during DTO assembly.
#[derive(Debug, FromQueryResult)]
struct ProjectProfileRow {
    id: Uuid,
    logo_props: String,
    created_issues: i64,
    assigned_issues: i64,
    completed_issues: i64,
    pending_issues: i64,
}

/// `project_data` entry in the final response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectProfileData {
    pub id: Uuid,
    pub logo_props: serde_json::Value,
    pub created_issues: i64,
    pub assigned_issues: i64,
    pub completed_issues: i64,
    pub pending_issues: i64,
}

/// Target user data in the response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProfileData {
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub date_joined: DateTime<Utc>,
    pub user_timezone: String,
    pub display_name: String,
}

/// Response for `GET /api/workspaces/{slug}/user-profile/{user_id}/`.
///
/// Exact mirror of `WorkspaceUserProfileEndpoint.get` in Django.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProfileResponse {
    /// Stats per project — empty if requester is Guest/Viewer.
    pub project_data: Vec<ProjectProfileData>,
    pub user_data: UserProfileData,
}

/// `GET /api/workspaces/{slug}/user-profile/{user_id}/`
///
/// Returns user profile in workspace context:
/// personal data + issue statistics per project.
///
/// `project_data` is only populated when the requester has role >= Member (15).
/// The projects returned are those where the **requester** is an active member,
/// and the issue counters refer to the **target** user (`user_id`).
///
/// Mirror of `WorkspaceUserProfileEndpoint`
/// (`apps/api/plane/app/views/workspace/user.py:280`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-profile/{user_id}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"    = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid,   Path, description = "Target user UUID"),
    ),
    responses(
        (status = 200, description = "User profile", body = UserProfileResponse),
        (status = 403, description = "Not a workspace member"),
        (status = 404, description = "User or workspace not found"),
    )
)]
pub async fn get_user_profile(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let requester = require_workspace_member(&state.db, ws.id, user.id).await?;

    // ── 1. Target user fetch ──────────────────────────────────────────────────
    //
    // Django: `User.objects.get(pk=user_id)` — raises 404 if it doesn	 exist.
    let target_user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // ── 2. Compute avatar_url — mirror of User.avatar_url (Django) ────────────
    //
    // Django: `avatar_asset.asset_url` > `avatar` > None
    // Path asset: `/api/assets/v2/static/{id}/` (entity_type = USER_AVATAR)
    let avatar_url = if let Some(asset_id) = target_user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !target_user.avatar.is_empty() {
        Some(target_user.avatar.clone())
    } else {
        None
    };

    // ── 3. Compute cover_image_url — mirror of User.cover_image_url (Django) ──
    let cover_image_url = if let Some(asset_id) = target_user.cover_image_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else {
        target_user.cover_image.clone()
    };

    let user_data = UserProfileData {
        email: target_user.email.clone(),
        first_name: target_user.first_name.clone(),
        last_name: target_user.last_name.clone(),
        avatar_url,
        cover_image_url,
        date_joined: target_user.date_joined.into(),
        user_timezone: target_user.user_timezone.clone(),
        display_name: target_user.display_name.clone(),
    };

    // ── 4. Project stats — only if requester.role >= MEMBER (15) ─────────────
    //
    // Django: `if requesting_workspace_member.role >= 15`
    // Roles: Guest=5, Viewer=10, Member=15, Admin=20
    let project_data = if requester.role >= ROLE_MEMBER {
        let ws_id = ws.id;

        // A single query aggregates all counters per project, avoiding
        // N+1 queries. Correlated subqueries are replaced by LEFT JOINs
        // over grouped subqueries — same plan as if Django did
        // `annotate(Count(...))` in bulk.
        //
        // Project filter: archived = false, REQUESTER is active member.
        // Counters: TARGET user issues (created / assigned / completed / pending).
        //
        // Placeholders: $1 = requester_id, $2 = user_id, $3 = ws_id.
        // Bind parameters are used instead of `format!` interpolation to
        // shield against SQL injection (defense in depth). `group` is
        // escaped as `"group"` because its a reserved word in PostgreSQL.
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT
                p.id                                  AS id,
                p.logo_props::text                    AS logo_props,
                COALESCE(ci.cnt,    0)                AS created_issues,
                COALESCE(ai.cnt,    0)                AS assigned_issues,
                COALESCE(compi.cnt, 0)                AS completed_issues,
                COALESCE(pi2.cnt,   0)                AS pending_issues
            FROM projects p
            JOIN project_members pm
                ON  pm.project_id  = p.id
                AND pm.member_id   = $1
                AND pm.is_active   = true
                AND pm.deleted_at  IS NULL

            -- created_issues: issues created by the target user in the project
            LEFT JOIN (
                SELECT project_id, COUNT(*) AS cnt
                FROM   issues
                WHERE  created_by_id = $2
                  AND  archived_at   IS NULL
                  AND  is_draft      = false
                  AND  deleted_at    IS NULL
                GROUP BY project_id
            ) ci ON ci.project_id = p.id

            -- assigned_issues: issues assigned to the target user
            LEFT JOIN (
                SELECT i.project_id, COUNT(DISTINCT ia.id) AS cnt
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                GROUP BY i.project_id
            ) ai ON ai.project_id = p.id

            -- completed_issues: assigned + completed
            LEFT JOIN (
                SELECT i.project_id, COUNT(DISTINCT ia.id) AS cnt
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                WHERE  ia.assignee_id    = $2
                  AND  ia.deleted_at     IS NULL
                  AND  i.completed_at    IS NOT NULL
                  AND  i.archived_at     IS NULL
                  AND  i.is_draft        = false
                  AND  i.deleted_at      IS NULL
                GROUP BY i.project_id
            ) compi ON compi.project_id = p.id

            -- pending_issues: assigned in backlog/unstarted/started groups
            LEFT JOIN (
                SELECT i.project_id, COUNT(DISTINCT ia.id) AS cnt
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   states s ON s.id = i.state_id
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  s."group"      IN ('backlog', 'unstarted', 'started')
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                GROUP BY i.project_id
            ) pi2 ON pi2.project_id = p.id

            WHERE p.workspace_id = $3
              AND p.archived_at  IS NULL
              AND p.deleted_at   IS NULL
            "#,
            vec![requester.member_id.into(), user_id.into(), ws_id.into()],
        );

        let rows = ProjectProfileRow::find_by_statement(stmt)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?;

        rows.into_iter()
            .map(|row| {
                // logo_props comes as JSON String; deserialize with safe fallback.
                let logo_props = serde_json::from_str(&row.logo_props)
                    .unwrap_or(serde_json::json!({}));
                ProjectProfileData {
                    id: row.id,
                    logo_props,
                    created_issues: row.created_issues,
                    assigned_issues: row.assigned_issues,
                    completed_issues: row.completed_issues,
                    pending_issues: row.pending_issues,
                }
            })
            .collect()
    } else {
        // Guest / Viewer — without access to project statistics (Django parity)
        Vec::new()
    };

    Ok(Json(UserProfileResponse {
        project_data,
        user_data,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE USER STATS  (WorkspaceUserProfileStatsEndpoint)
// ═══════════════════════════════════════════════════════════════════════════
//
// GET /workspaces/{slug}/user-stats/{user_id}/
//
// Mirror of Django `WorkspaceUserProfileStatsEndpoint.get`
// (plane/app/views/workspace/user.py).
//
// Returns:
//   - state_distribution     : [{state_group, state_count}]
//   - priority_distribution  : [{priority, priority_count}]
//   - created_issues         : i64
//   - assigned_issues        : i64
//   - completed_issues       : i64
//   - pending_issues         : i64
//   - subscribed_issues      : i64
//   - present_cycles         : [{cycle__name, cycle__id, cycle__project_id}]
//   - upcoming_cycles        : [{cycle__name, cycle__id, cycle__project_id}]
//
// Permission: active workspace member (any role).

#[derive(Debug, Serialize, FromQueryResult)]
pub struct StateDistributionRow {
    pub state_group: String,
    pub state_count: i64,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct PriorityDistributionRow {
    pub priority: String,
    pub priority_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct CycleInfoRow {
    pub cycle_name: String,
    pub cycle_id: Uuid,
    pub cycle_project_id: Uuid,
}

// Serialization that reproduces the Django form: double underscore keys
// like `cycle__name`, `cycle__id`, `cycle__project_id`.
impl serde::Serialize for CycleInfoRow {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(3))?;
        map.serialize_entry("cycle__name", &self.cycle_name)?;
        map.serialize_entry("cycle__id", &self.cycle_id)?;
        map.serialize_entry("cycle__project_id", &self.cycle_project_id)?;
        map.end()
    }
}

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub state_distribution: Vec<StateDistributionRow>,
    pub priority_distribution: Vec<PriorityDistributionRow>,
    pub created_issues: i64,
    pub assigned_issues: i64,
    pub completed_issues: i64,
    pub pending_issues: i64,
    pub subscribed_issues: i64,
    pub present_cycles: Vec<CycleInfoRow>,
    pub upcoming_cycles: Vec<CycleInfoRow>,
}

/// GET /workspaces/{slug}/user-stats/{user_id}/
///
/// Mirror Django `WorkspaceUserProfileStatsEndpoint.get`.
/// Requires active membership of the requester in the workspace; statistics
/// calculated for the `user_id` indicated in the URL.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-stats/{user_id}/",
    tag = "Workspaces",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid, Path, description = "Target user ID"),
    ),
    responses(
        (status = 200, description = "User statistics in workspace"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "Not an active workspace member"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn get_user_stats(
    State(state): State<AppState>,
    AnyAuth(requester): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
) -> Result<Json<UserStatsResponse>, AppError> {
    let db = &state.db;
    let ws = workspace_by_slug(db, &slug).await?;
    // Authorization: requester must be an active member (any role)
    let requester_id = requester.id;
    let _member = require_workspace_member(db, ws.id, requester_id).await?;
    let ws_id = ws.id;

    // ── state_distribution ───────────────────────────────────────────────────
    // Mirror: issues assigned to user_id, grouped by state.group, excludes
    // issue_assignees with deleted_at != NULL (parity with the condition
    // `Q(issue_assignee__deleted_at__isnull=True)` in Django).
    //
    // NOTE: `group` is a reserved word in PostgreSQL — must be escaped with
    // double quotes (`s."group"`) to avoid parsing errors in
    // `GROUP BY` / `ORDER BY`. Bind parameters ($1..$N) instead of
    // `format!` interpolation to prevent deep SQL injection.
    let state_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT s."group" AS state_group, COUNT(DISTINCT ia.id) AS state_count
        FROM issue_assignees ia
        JOIN issues      i  ON i.id  = ia.issue_id
        JOIN states      s  ON s.id  = i.state_id
        JOIN projects    p  ON p.id  = i.project_id
        JOIN project_members pm
             ON  pm.project_id = p.id
             AND pm.member_id  = $1
             AND pm.is_active  = true
             AND pm.deleted_at IS NULL
        WHERE ia.assignee_id = $2
          AND ia.deleted_at  IS NULL
          AND i.workspace_id = $3
          AND i.archived_at  IS NULL
          AND i.is_draft     = false
          AND i.deleted_at   IS NULL
          AND p.deleted_at   IS NULL
        GROUP BY s."group"
        ORDER BY s."group"
        "#,
        vec![requester_id.into(), user_id.into(), ws_id.into()],
    );

    let state_distribution = StateDistributionRow::find_by_statement(state_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── priority_distribution ────────────────────────────────────────────────
    // Mirror: same assigned issues, grouped by priority.
    // Django order: urgent=0, high=1, medium=2, low=3, none=4.
    let priority_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT i.priority,
               COUNT(DISTINCT ia.id) AS priority_count,
               CASE i.priority
                   WHEN 'urgent' THEN 0
                   WHEN 'high'   THEN 1
                   WHEN 'medium' THEN 2
                   WHEN 'low'    THEN 3
                   ELSE 4
               END AS priority_order
        FROM issue_assignees ia
        JOIN issues   i  ON i.id  = ia.issue_id
        JOIN projects p  ON p.id  = i.project_id
        JOIN project_members pm
             ON  pm.project_id = p.id
             AND pm.member_id  = $1
             AND pm.is_active  = true
             AND pm.deleted_at IS NULL
        WHERE ia.assignee_id = $2
          AND ia.deleted_at  IS NULL
          AND i.workspace_id = $3
          AND i.archived_at  IS NULL
          AND i.is_draft     = false
          AND i.deleted_at   IS NULL
          AND p.deleted_at   IS NULL
        GROUP BY i.priority
        HAVING COUNT(DISTINCT ia.id) >= 1
        ORDER BY priority_order
        "#,
        vec![requester_id.into(), user_id.into(), ws_id.into()],
    );

    let priority_distribution = PriorityDistributionRow::find_by_statement(priority_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── scalar counters ──────────────────────────────────────────────────────
    // A single multi-column query to reduce DB round-trips.
    // Placeholders: $1 = requester_id, $2 = user_id, $3 = ws_id.
    // Postgres allows reusing the same `$n` multiple times in the same query.
    let counters_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT
            -- created_issues
            (
                SELECT COUNT(*)
                FROM   issues i
                JOIN   projects p  ON p.id  = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  i.workspace_id  = $3
                  AND  i.created_by_id = $2
                  AND  i.archived_at   IS NULL
                  AND  i.is_draft      = false
                  AND  i.deleted_at    IS NULL
                  AND  p.deleted_at    IS NULL
            ) AS created_issues,

            -- assigned_issues
            (
                SELECT COUNT(DISTINCT ia.id)
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   projects p ON p.id = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  i.workspace_id = $3
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                  AND  p.deleted_at   IS NULL
            ) AS assigned_issues,

            -- completed_issues
            (
                SELECT COUNT(DISTINCT ia.id)
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   states s ON s.id = i.state_id
                JOIN   projects p ON p.id = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  s."group"      = 'completed'
                  AND  i.workspace_id = $3
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                  AND  p.deleted_at   IS NULL
            ) AS completed_issues,

            -- pending_issues: not completed or cancelled
            (
                SELECT COUNT(DISTINCT ia.id)
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   states s ON s.id = i.state_id
                JOIN   projects p ON p.id = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  s."group"      NOT IN ('completed', 'cancelled')
                  AND  i.workspace_id = $3
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                  AND  p.deleted_at   IS NULL
            ) AS pending_issues,

            -- subscribed_issues
            (
                SELECT COUNT(DISTINCT isub.id)
                FROM   issue_subscribers isub
                JOIN   projects p ON p.id = isub.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  isub.subscriber_id = $2
                  AND  isub.workspace_id  = $3
                  AND  p.archived_at      IS NULL
                  AND  p.deleted_at       IS NULL
            ) AS subscribed_issues
        "#,
        vec![requester_id.into(), user_id.into(), ws_id.into()],
    );

    #[derive(Debug, FromQueryResult)]
    struct CountersRow {
        created_issues: i64,
        assigned_issues: i64,
        completed_issues: i64,
        pending_issues: i64,
        subscribed_issues: i64,
    }

    let counters = CountersRow::find_by_statement(counters_stmt)
        .one(db)
        .await
        .map_err(AppError::Database)?
        .unwrap_or(CountersRow {
            created_issues: 0,
            assigned_issues: 0,
            completed_issues: 0,
            pending_issues: 0,
            subscribed_issues: 0,
        });

    // ── upcoming_cycles ──────────────────────────────────────────────────────
    // Mirror: CycleIssue where cycle.start_date > now() and issue has the user assigned.
    let upcoming_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT DISTINCT
               c.name       AS cycle_name,
               c.id         AS cycle_id,
               c.project_id AS cycle_project_id
        FROM   cycle_issues ci
        JOIN   cycles c ON c.id = ci.cycle_id
        JOIN   issue_assignees ia ON ia.issue_id = ci.issue_id AND ia.deleted_at IS NULL
        WHERE  c.workspace_id = $1
          AND  c.start_date   > NOW()
          AND  ia.assignee_id = $2
          AND  ci.deleted_at  IS NULL
          AND  c.deleted_at   IS NULL
        "#,
        vec![ws_id.into(), user_id.into()],
    );

    let upcoming_cycles = CycleInfoRow::find_by_statement(upcoming_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── present_cycles ───────────────────────────────────────────────────────
    // Mirror: start_date < now() AND end_date > now().
    let present_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT DISTINCT
               c.name       AS cycle_name,
               c.id         AS cycle_id,
               c.project_id AS cycle_project_id
        FROM   cycle_issues ci
        JOIN   cycles c ON c.id = ci.cycle_id
        JOIN   issue_assignees ia ON ia.issue_id = ci.issue_id AND ia.deleted_at IS NULL
        WHERE  c.workspace_id = $1
          AND  c.start_date   < NOW()
          AND  c.end_date     > NOW()
          AND  ia.assignee_id = $2
          AND  ci.deleted_at  IS NULL
          AND  c.deleted_at   IS NULL
        "#,
        vec![ws_id.into(), user_id.into()],
    );

    let present_cycles = CycleInfoRow::find_by_statement(present_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(UserStatsResponse {
        state_distribution,
        priority_distribution,
        created_issues: counters.created_issues,
        assigned_issues: counters.assigned_issues,
        completed_issues: counters.completed_issues,
        pending_issues: counters.pending_issues,
        subscribed_issues: counters.subscribed_issues,
        present_cycles,
        upcoming_cycles,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE USER ACTIVITY  (WorkspaceUserActivityEndpoint)
// ═══════════════════════════════════════════════════════════════════════════
//
// GET /workspaces/{slug}/user-activity/{user_id}/
//
// Mirror of Django `WorkspaceUserActivityEndpoint.get`
// (plane/app/views/workspace/user.py:370).
//
// Returns `IssueActivity` activities of actor `user_id` in the workspace,
// excluding virtual fields (comment, vote, reaction, draft), and filtering
// only projects where the **requester** is an active non-archived member.
//
// Optional query params:
//   - `project` (repeatable): Project UUIDs to filter
//   - `per_page`: records per page (default 10)
//   - `cursor`: Django-style pagination cursor
//   - `order_by`: sort column (default `-created_at`)
//
// Serialization: mirror of `IssueActivitySerializer(fields="__all__")` with
// nested `actor_detail`, `issue_detail`, `project_detail`, `workspace_detail`.

/// Mirror of `IssueActivitySerializer.actor_detail` → `UserLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ActivityActorDetail {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub display_name: String,
}

/// Mirror of `IssueFlatSerializer` — minimum fields the frontend consumes in
/// user profile activity panel.
#[derive(Debug, Serialize)]
pub struct ActivityIssueDetail {
    pub id: Uuid,
    pub name: String,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
}

/// Mirror of `ProjectLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ActivityProjectDetail {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub logo_props: serde_json::Value,
}

/// Mirror of `WorkspaceLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ActivityWorkspaceDetail {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
}

/// Activity response — mirror of `IssueActivitySerializer(fields="__all__")`.
#[derive(Debug, Serialize)]
pub struct UserActivityItem {
    // IssueActivity model fields
    pub id: Uuid,
    pub verb: String,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub comment: String,
    pub actor_id: Option<Uuid>,
    pub issue_id: Option<Uuid>,
    pub issue_comment_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub old_identifier: Option<Uuid>,
    pub new_identifier: Option<Uuid>,
    pub epoch: Option<f64>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    // Nested fields (None when the referenced object was deleted)
    pub actor_detail: Option<ActivityActorDetail>,
    pub issue_detail: Option<ActivityIssueDetail>,
    pub project_detail: Option<ActivityProjectDetail>,
    pub workspace_detail: Option<ActivityWorkspaceDetail>,
}

/// Query params for `GET /workspaces/{slug}/user-activity/{user_id}/`.
#[derive(Debug, Deserialize)]
pub struct UserActivityQuery {
    /// Project filter (multi-value: ?project=A&project=B).
    #[serde(default)]
    pub project: Vec<Uuid>,
    pub per_page: Option<u64>,
    pub cursor: Option<String>,
    pub order_by: Option<String>,
}

/// `GET /workspaces/{slug}/user-activity/{user_id}/`
///
/// Activities of actor `user_id` visible to the requester (only projects
/// where the requester is an active member).
///
/// Mirror of `WorkspaceUserActivityEndpoint` in Django with cursor pagination.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-activity/{user_id}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"    = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid,   Path, description = "Actor user UUID"),
    ),
    responses(
        (status = 200, description = "Paginated user activity list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a workspace member"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn get_workspace_user_activity(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
    Query(q): Query<UserActivityQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let ws = workspace_by_slug(db, &slug).await?;
    // Authorization: requester must be an active member of the workspace
    let _member = require_workspace_member(db, ws.id, auth_user.id).await?;

    // ── Pagination ────────────────────────────────────────────────────────────
    const DEFAULT_PER_PAGE: u64 = 10;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // ── Visible projects for the requester ────────────────────────────────────
    // Django: `project__project_projectmember__member=request.user`
    //         `project__project_projectmember__is_active=True`
    //         `project__archived_at__isnull=True`
    //
    // We resolve in Rust with a separate query to get the project IDs
    // accessible, then use them as an IN filter on IssueActivity.
    // This avoids a complex JOIN in SeaORM and keeps the code readable.
    let accessible_project_ids: Vec<Uuid> = {
        let memberships = project_members::Entity::find()
            .filter(project_members::Column::MemberId.eq(auth_user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .filter(project_members::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        if memberships.is_empty() {
            // No accessible projects → empty paginated response
            let body = pagination::build_response(
                Vec::<UserActivityItem>::new(),
                0,
                limit,
                cursor.offset,
            );
            return Ok((axum::http::StatusCode::OK, axum::Json(body)));
        }

        // Filter only current workspace projects and non-archived
        let project_ids_from_memberships: Vec<Uuid> =
            memberships.iter().map(|pm| pm.project_id).collect();

        let visible_projects = projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids_from_memberships))
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .filter(projects::Column::ArchivedAt.is_null())
            .filter(projects::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        visible_projects.into_iter().map(|p| p.id).collect()
    };

    if accessible_project_ids.is_empty() {
        let body = pagination::build_response(
            Vec::<UserActivityItem>::new(),
            0,
            limit,
            cursor.offset,
        );
        return Ok((axum::http::StatusCode::OK, axum::Json(body)));
    }

    // ── Excluded fields — Django mirror: ~Q(field__in=["comment","vote","reaction","draft"]) ──
    const EXCLUDED_FIELDS: &[&str] = &["comment", "vote", "reaction", "draft"];

    // ── Build base query ─────────────────────────────────────────────────────
    let mut base = issue_activities::Entity::find()
        .filter(issue_activities::Column::WorkspaceId.eq(ws.id))
        .filter(issue_activities::Column::ActorId.eq(user_id))
        .filter(issue_activities::Column::ProjectId.is_in(accessible_project_ids))
        .filter(issue_activities::Column::DeletedAt.is_null())
        // Exclude virtual fields (NOT IN)
        .filter(
            sea_orm::Condition::any()
                .add(issue_activities::Column::Field.is_null())
                .add(
                    issue_activities::Column::Field
                        .is_not_in(EXCLUDED_FIELDS.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
                ),
        );

    // Optional filter by project (?project=UUID)
    if !q.project.is_empty() {
        base = base.filter(issue_activities::Column::ProjectId.is_in(q.project.clone()));
    }

    // ── Count (same filters, without order or offset) ────────────────────────
    let total_count = base.clone().count(db).await.map_err(AppError::Database)?;

    // ── Ordering — Django mirror: default -created_at ────────────────────────
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
        return Ok((axum::http::StatusCode::OK, axum::Json(body)));
    }

    // ── Batch-fetch related objects (avoid N+1) ──────────────────────────────

    // Actor IDs (always the same user_id, but keeping generic pattern)
    let actor_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.actor_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let actors_map: std::collections::HashMap<Uuid, users::Model> = users::Entity::find()
        .filter(users::Column::Id.is_in(actor_ids))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    // Issue IDs
    let issue_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.issue_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let issues_map: std::collections::HashMap<Uuid, issues::Model> = if !issue_ids.is_empty() {
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
        std::collections::HashMap::new()
    };

    // Project IDs of activities on this page
    let page_project_ids: Vec<Uuid> = activities
        .iter()
        .map(|a| a.project_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let projects_map: std::collections::HashMap<Uuid, projects::Model> =
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(page_project_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.id, p))
            .collect();

    // Workspace detail (single for all activities on the page)
    let workspace_detail = ActivityWorkspaceDetail {
        id: ws.id,
        name: ws.name.clone(),
        slug: ws.slug.clone(),
        logo: ws.logo.clone(),
    };

    // ── Assemble response ────────────────────────────────────────────────────
    let results: Vec<UserActivityItem> = activities
        .into_iter()
        .map(|a| {
            // actor_detail
            let actor_detail = a.actor_id.and_then(|aid| {
                actors_map.get(&aid).map(|u| {
                    let avatar_url = if let Some(asset_id) = u.avatar_asset_id {
                        Some(format!("/api/assets/v2/static/{}/", asset_id))
                    } else if !u.avatar.is_empty() {
                        Some(u.avatar.clone())
                    } else {
                        None
                    };
                    ActivityActorDetail {
                        id: u.id,
                        first_name: u.first_name.clone(),
                        last_name: u.last_name.clone(),
                        avatar: u.avatar.clone(),
                        avatar_url,
                        is_bot: u.is_bot,
                        display_name: u.display_name.clone(),
                    }
                })
            });

            // issue_detail
            let issue_detail = a.issue_id.and_then(|iid| {
                issues_map.get(&iid).map(|i| ActivityIssueDetail {
                    id: i.id,
                    name: i.name.clone(),
                    sequence_id: i.sequence_id,
                    project_id: i.project_id,
                    workspace_id: i.workspace_id,
                })
            });

            // project_detail
            let project_detail = projects_map.get(&a.project_id).map(|p| {
                ActivityProjectDetail {
                    id: p.id,
                    identifier: p.identifier.clone(),
                    name: p.name.clone(),
                    logo_props: p.logo_props.clone(),
                }
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
                workspace_detail: Some(ActivityWorkspaceDetail {
                    id: workspace_detail.id,
                    name: workspace_detail.name.clone(),
                    slug: workspace_detail.slug.clone(),
                    logo: workspace_detail.logo.clone(),
                }),
            }
        })
        .collect();

    let body = pagination::build_response(results, total_count, limit, cursor.offset);
    Ok((axum::http::StatusCode::OK, axum::Json(body)))
}

// ═══════════════════════════════════════════════════════════════════════════
// EXPORT WORKSPACE USER ACTIVITY  (ExportWorkspaceUserActivityEndpoint)
// ═══════════════════════════════════════════════════════════════════════════
//
// POST /workspaces/{slug}/user-activity/{user_id}/export
//
// Mirror of Django `ExportWorkspaceUserActivityEndpoint.post`
// (plane/app/views/workspace/base.py:368).
//
// Body JSON:   `{ "date": "YYYY-MM-DD" }`  (required)
// Response:   `text/csv` with `Content-Disposition: attachment; filename="workspace-user-activity.csv"`
//
// CSV Parity:
//   - 9 columns in this exact order: Actor name, Issue ID, Project,
//     Created at, Updated at, Action, Field, Old value, New value.
//   - `Issue ID` is `"{identifier} - {sequence_id_or_empty}"` (the separator
//     Django has spaces on both sides — it is respected literally).
//   - All values quoted (`QUOTE_ALL` in Django ⇒ `QuoteStyle::Always`
//     in the `csv` crate writer).
//   - Each cell passes through `sanitize_csv_cell` to prevent CSV injection
//     (parity with `sanitize_csv_row` in Django).
//   - Cap of 10,000 rows (mirror of `[:10000]` in the Django queryset).
//
// Filters applied to the queryset (same as homologous GET plus date filter):
//   * workspace_id = ws.id
//   * actor_id = user_id (path parameter)
//   * project_id ∈ accessible_project_ids
//     (workspace projects in which the *requester* is an active member
//     and project is not archived or deleted)
//   * field NOT IN (comment, vote, reaction, draft) OR field IS NULL
//   * created_at ∈ [date 00:00:00 UTC, date+1 00:00:00 UTC)
//   * deleted_at IS NULL (soft-delete respected)
//
// Authorization:
//   - El requester debe ser miembro activo del workspace (`require_workspace_member`).
//   - Django aplica `WorkspaceEntityPermission` ⇒ en POST exige role ∈
//     {Admin=20, Member=15}. Guests y Viewers reciben 403.
//
// Timezone note:
//   - Django evaluates `created_at__date` against active TZ. Standard Plane
//     deploys use `TIME_ZONE='UTC'` with `USE_TZ=True`, so we compare
//     against UTC. If in the future Plane makes workspace TZ configurable,
//     this handler must recalibrate.
//
// Sorting note:
//   - Django doesn	 add `.order_by(...)` before `[:10000]`, leaving order
//     undefined. In Rust we add explicit `-created_at` so the CSV
//     is deterministic between runs; same criterion as GET.

/// Query params of `GET /workspaces/{slug}/user-activity/{user_id}/export`.
///
/// El handler GET se mantiene como alias del POST (ambos exportan CSV).
/// If `date` is not sent, `today()` UTC is used, allowing the frontend
/// to invoke it without form (direct download link).
#[derive(Debug, Deserialize)]
pub struct ExportUserActivityQuery {
    pub date: Option<String>,
}

/// JSON body for `POST /workspaces/{slug}/user-activity/{user_id}/export`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ExportUserActivityBody {
    /// Local date (UTC) for which activities will be exported.
    /// ISO 8601 format `YYYY-MM-DD`.
    pub date: Option<String>,
}

/// `POST /workspaces/{slug}/user-activity/{user_id}/export`
///
/// Exports in CSV format the activities of actor `user_id` recorded on
/// a specific date. Requires role ≥ Member for the requester.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/user-activity/{user_id}/export",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"    = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid,   Path, description = "Actor user UUID"),
    ),
    request_body = ExportUserActivityBody,
    responses(
        (status = 200, description = "CSV file with the days activities"),
        (status = 400, description = "Missing `date` or invalid format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a workspace member or role < Member"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn export_workspace_user_activity(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
    Json(body): Json<ExportUserActivityBody>,
) -> Result<axum::response::Response, AppError> {
    let db = &state.db;

    // ── Body validation ───────────────────────────────────────────────────
    // Django: `if not request.data.get("date"): return 400 {"error": "Date is required"}`
    let date_str = body.date.as_deref().unwrap_or("").trim();
    if date_str.is_empty() {
        return Err(AppError::BadRequest("Date is required".into()));
    }
    let target_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest("Invalid date format. Expected YYYY-MM-DD".into())
    })?;

    // Range [00:00:00, +1 day) in UTC. We use `and_hms_opt` + `single()` —
    // both return Option because UTC midnight does not suffer from DST, but the
    // chrono API doesn	 know statically. Failing here would be a chrono bug,
    // not user input, so we map to 500.
    let start_of_day_utc = target_date
        .and_hms_opt(0, 0, 0)
        .and_then(|ndt| Utc.from_local_datetime(&ndt).single())
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Failed to build start_of_day UTC")))?;
    let end_of_day_utc = start_of_day_utc + Duration::days(1);

    // Convert to `DateTime<FixedOffset>` — the alias for `DateTimeWithTimeZone`
    // in SeaORM. Consistent pattern with `src/jobs/cleanup.rs:207`.
    let start_of_day: DateTime<chrono::FixedOffset> = start_of_day_utc.into();
    let end_of_day: DateTime<chrono::FixedOffset> = end_of_day_utc.into();

    // ── Workspace + authorization ─────────────────────────────────────────
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, auth_user.id).await?;

    // Parity with `WorkspaceEntityPermission` in POST:
    // role ∈ {Admin(20), Member(15)} ⇒ Guest/Viewer rechazados.
    if member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    // ── Accessible projects for the requester ─────────────────────────────
    // Same logic as `get_workspace_user_activity` — intentional duplication:
    // refactorizar a helper solo cuando aparezca un 3er call-site.
    let accessible_project_ids: Vec<Uuid> = {
        let memberships = project_members::Entity::find()
            .filter(project_members::Column::MemberId.eq(auth_user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .filter(project_members::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        if memberships.is_empty() {
            return build_empty_user_activity_csv_response();
        }

        let project_ids_from_memberships: Vec<Uuid> =
            memberships.iter().map(|pm| pm.project_id).collect();

        let visible_projects = projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids_from_memberships))
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .filter(projects::Column::ArchivedAt.is_null())
            .filter(projects::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        visible_projects.into_iter().map(|p| p.id).collect()
    };

    if accessible_project_ids.is_empty() {
        return build_empty_user_activity_csv_response();
    }

    // ── Activities query ──────────────────────────────────────────────────
    const EXCLUDED_FIELDS: &[&str] = &["comment", "vote", "reaction", "draft"];
    const ROW_CAP: u64 = 10_000;

    let activities = issue_activities::Entity::find()
        .filter(issue_activities::Column::WorkspaceId.eq(ws.id))
        .filter(issue_activities::Column::ActorId.eq(user_id))
        .filter(issue_activities::Column::ProjectId.is_in(accessible_project_ids))
        .filter(issue_activities::Column::DeletedAt.is_null())
        .filter(issue_activities::Column::CreatedAt.gte(start_of_day))
        .filter(issue_activities::Column::CreatedAt.lt(end_of_day))
        // NOT IN with NULL-safe: SQL `NOT IN (...)` evaluates to NULL when field
        // is NULL, excluding those rows — we use Condition::any to
        // capturing them explicitly (same pattern as homologous GET).
        .filter(
            sea_orm::Condition::any()
                .add(issue_activities::Column::Field.is_null())
                .add(
                    issue_activities::Column::Field.is_not_in(
                        EXCLUDED_FIELDS.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    ),
                ),
        )
        // Explicit ordering — justified divergence with Django (see header).
        .order_by_desc(issue_activities::Column::CreatedAt)
        .limit(ROW_CAP)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if activities.is_empty() {
        return build_empty_user_activity_csv_response();
    }

    // ── Batch-load related objects (avoid N+1) ────────────────────────────
    let actor_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.actor_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let actors_map: std::collections::HashMap<Uuid, users::Model> = if !actor_ids.is_empty() {
        users::Entity::find()
            .filter(users::Column::Id.is_in(actor_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    let issue_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.issue_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let issues_map: std::collections::HashMap<Uuid, issues::Model> = if !issue_ids.is_empty() {
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
        std::collections::HashMap::new()
    };

    let page_project_ids: Vec<Uuid> = activities
        .iter()
        .map(|a| a.project_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let projects_map: std::collections::HashMap<Uuid, projects::Model> =
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(page_project_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.id, p))
            .collect();

    // ── CSV serialization ─────────────────────────────────────────────────
    // QuoteStyle::Always ≡ Django `csv.QUOTE_ALL`.
    let bytes = encode_user_activity_csv(&activities, &actors_map, &issues_map, &projects_map)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV encoding failed: {e}")))?;

    build_user_activity_csv_response(bytes)
}

/// `GET /workspaces/{slug}/user-activity/{user_id}/export?date=YYYY-MM-DD`
///
/// GET alias of POST: allows using the URL as a direct download link
/// (`<a href>`). Si `date` falta se usa hoy UTC. Reusa el handler POST
/// reassembling the body — single validation/auth/serialization flow.
pub async fn export_workspace_user_activity_get(
    state: State<AppState>,
    auth: AnyAuth,
    path: Path<(String, Uuid)>,
    Query(q): Query<ExportUserActivityQuery>,
) -> Result<axum::response::Response, AppError> {
    let date = q
        .date
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string());
    export_workspace_user_activity(
        state,
        auth,
        path,
        Json(ExportUserActivityBody { date: Some(date) }),
    )
    .await
}

/// Formats a `DateTime<FixedOffset>` to the same string as `str(datetime)` in
/// Python: `YYYY-MM-DD HH:MM:SS.ffffff+HH:MM`. Used for columns
/// `Created at` / `Updated at` of the CSV for visual parity with Django.
fn format_activity_datetime(dt: &sea_orm::prelude::DateTimeWithTimeZone) -> String {
    // `%:z` produces `+HH:MM` (with colon), same as `datetime` repr.
    // `%.6f` produces microseconds with six digits (zero padding).
    dt.format("%Y-%m-%d %H:%M:%S%.6f%:z").to_string()
}

/// Builds the CSV buffer — extracted for testability and readability of
/// handler. Returns the bytes ready to send as the response body.
fn encode_user_activity_csv(
    activities: &[issue_activities::Model],
    actors_map: &std::collections::HashMap<Uuid, users::Model>,
    issues_map: &std::collections::HashMap<Uuid, issues::Model>,
    projects_map: &std::collections::HashMap<Uuid, projects::Model>,
) -> Result<Vec<u8>, csv::Error> {
    let mut wtr = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .from_writer(Vec::<u8>::new());

    // Header — exact order and labels from Django.
    wtr.write_record([
        "Actor name",
        "Issue ID",
        "Project",
        "Created at",
        "Updated at",
        "Action",
        "Field",
        "Old value",
        "New value",
    ])?;

    // Each row passes through `sanitize_csv_cell` to prevent CSV injection.
    // Nullable fields are converted to "" when they are None (parity with
    // Python: `csv.writer.writerow([..., None, ...])` writes an empty cell).
    for a in activities {
        let actor_display = a
            .actor_id
            .and_then(|id| actors_map.get(&id))
            .map(|u| u.display_name.as_str())
            .unwrap_or("");

        let project = projects_map.get(&a.project_id);
        let project_identifier = project.map(|p| p.identifier.as_str()).unwrap_or("");
        let project_name = project.map(|p| p.name.as_str()).unwrap_or("");

        // Django: `f"{identifier} - {issue.sequence_id if issue else ''}"`
        // Espacios alrededor del guion se respetan literalmente.
        let issue_seq = a
            .issue_id
            .and_then(|id| issues_map.get(&id))
            .map(|i| i.sequence_id.to_string())
            .unwrap_or_default();
        let issue_id_col = format!("{project_identifier} - {issue_seq}");

        let created_at = format_activity_datetime(&a.created_at);
        let updated_at = format_activity_datetime(&a.updated_at);

        let field = a.field.as_deref().unwrap_or("");
        let old_value = a.old_value.as_deref().unwrap_or("");
        let new_value = a.new_value.as_deref().unwrap_or("");

        wtr.write_record([
            &sanitize_csv_cell(actor_display),
            &sanitize_csv_cell(&issue_id_col),
            &sanitize_csv_cell(project_name),
            &sanitize_csv_cell(&created_at),
            &sanitize_csv_cell(&updated_at),
            &sanitize_csv_cell(&a.verb),
            &sanitize_csv_cell(field),
            &sanitize_csv_cell(old_value),
            &sanitize_csv_cell(new_value),
        ])?;
    }

    // `into_inner()` returns `Result<Vec<u8>, csv::IntoInnerError<_>>` y
    // `into_error()` en csv 1.4 devuelve `std::io::Error` (no `csv::Error`).
    // The `?` does coercion via `impl From<io::Error> for csv::Error`.
    Ok(wtr.into_inner().map_err(|e| e.into_error())?)
}

/// "Empty" CSV response — header only, no rows. Used when requester
/// has no visible projects or when there are no activities for the date.
/// Django in those cases returns a header-only CSV (the list comprehension
/// does not produce rows but the header is always written).
fn build_empty_user_activity_csv_response() -> Result<axum::response::Response, AppError> {
    let mut wtr = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .from_writer(Vec::<u8>::new());
    wtr.write_record([
        "Actor name",
        "Issue ID",
        "Project",
        "Created at",
        "Updated at",
        "Action",
        "Field",
        "Old value",
        "New value",
    ])
    .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV header write failed: {e}")))?;
    let bytes = wtr
        .into_inner()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("CSV flush failed: {e}")))?;
    build_user_activity_csv_response(bytes)
}

/// Wraps bytes in an HTTP response with the download headers expected
/// by the frontend (`text/csv` + `Content-Disposition: attachment`).
fn build_user_activity_csv_response(bytes: Vec<u8>) -> Result<axum::response::Response, AppError> {
    let headers = [
        (header::CONTENT_TYPE, "text/csv"),
        (
            header::CONTENT_DISPOSITION,
            r#"attachment; filename="workspace-user-activity.csv""#,
        ),
    ];
    Ok((StatusCode::OK, headers, bytes).into_response())
}

// ─── GET /workspaces/{slug}/members/{pk}/ ────────────────────────────────────

/// Returns a specific workspace member.
///
/// Mirror of `WorkSpaceMemberViewSet.retrieve`
/// (`apps/api/plane/app/views/workspace/member.py:58-73`).
/// Requires active membership (GUEST, VIEWER, MEMBER, ADMIN).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/members/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Member record UUID"),
    ),
    responses(
        (status = 200, description = "Member detail"),
        (status = 403, description = "Not a member"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<WorkspaceMemberNestedResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Django Parity: admins see email/last_login_medium, guests don't.
    let is_admin = caller.role > ROLE_GUEST;

    let member = workspace_members::Entity::find_by_id(pk)
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let user_model = users::Entity::find_by_id(member.member_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(WorkspaceMemberNestedResponse {
        id: member.id,
        member: user_to_lite(&user_model, is_admin),
        role: member.role,
    }))
}

// ─── POST /workspaces/{slug}/members/leave/ ──────────────────────────────────

/// The authenticated user leaves the workspace.
///
/// Mirror of `WorkSpaceMemberViewSet.leave`
/// (`apps/api/plane/app/views/workspace/member.py:140-185`).
/// Reglas:
///   - Cannot leave if you are the only Admin of the workspace.
///   - Cannot leave if you are the only Admin of some project.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/members/leave/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 204, description = "Left workspace"),
        (status = 400, description = "Cannot leave — last admin or sole project admin"),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn leave_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;

    // If Admin, verify they are not the only one.
    if member.role >= ROLE_ADMIN {
        let admin_count = workspace_members::Entity::find()
            .active()
            .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
            .filter(workspace_members::Column::Role.gte(ROLE_ADMIN))
            .filter(workspace_members::Column::IsActive.eq(true))
            .count(&state.db)
            .await
            .map_err(AppError::Database)?;

        if admin_count <= 1 {
            return Err(AppError::BadRequest(
                "You cannot leave the workspace as you are the only admin. \
                 Please delete the workspace or promote another user to admin.".into(),
            ));
        }
    }

    // Verify they are not the only Admin in some project of the workspace.
    // Antipattern avoided: N+1 — we do a raw query COUNT with subquery.
    use sea_orm::Statement;
    use sea_orm::ConnectionTrait;
    let sole_admin_project: Option<bool> = state.db
        .query_one(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM projects p
                WHERE p.workspace_id = $1
                  AND p.deleted_at IS NULL
                  AND (
                    SELECT COUNT(*) FROM project_members pm
                    WHERE pm.project_id = p.id
                      AND pm.is_active = true
                      AND pm.deleted_at IS NULL
                  ) = 1
                  AND EXISTS (
                    SELECT 1 FROM project_members pm2
                    WHERE pm2.project_id = p.id
                      AND pm2.member_id = $2
                      AND pm2.role >= 20
                      AND pm2.is_active = true
                      AND pm2.deleted_at IS NULL
                  )
            )
            "#,
            vec![
                sea_orm::Value::Uuid(Some(Box::new(ws.id))),
                sea_orm::Value::Uuid(Some(Box::new(user.id))),
            ],
        ))
        .await
        .map_err(AppError::Database)?
        .map(|r| r.try_get::<bool>("", "exists").unwrap_or(false));

    if sole_admin_project.unwrap_or(false) {
        return Err(AppError::BadRequest(
            "You are the only admin in some projects. \
             Please leave those projects or promote another user to admin first.".into(),
        ));
    }

    // Deactivate workspace project memberships.
    project_members::Entity::update_many()
        .col_expr(
            project_members::Column::IsActive,
            sea_orm::sea_query::Expr::value(false),
        )
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Deactivate workspace membership.
    let mut am: workspace_members::ActiveModel = member.into();
    am.is_active = Set(false);
    am.updated_at = Set(chrono::Utc::now().fixed_offset());
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── GET /workspaces/{slug}/project-members/ ─────────────────────────────────

/// Returns a map `{project_id: [{member_id, role}]}` for all
/// workspace projects where the user is a member.
///
/// Mirror of `WorkspaceProjectMemberEndpoint.get`
/// (`apps/api/plane/app/views/workspace/member.py:187-220`).
/// Used by the frontend to load cross-project permissions.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/project-members/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Mapa project_id → [member roles]"),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn get_project_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Proyectos donde el usuario es miembro activo.
    let user_project_ids: Vec<Uuid> = project_members::Entity::find()
        .active()
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pm| pm.project_id)
        .collect();

    if user_project_ids.is_empty() {
        return Ok(Json(serde_json::json!({})));
    }

    // All active members of those projects — batch, no N+1.
    let all_members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::ProjectId.is_in(user_project_ids))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Construir mapa { project_id: [{member_id, role}] }.
    let mut map: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for pm in all_members {
        map.entry(pm.project_id.to_string())
            .or_default()
            .push(serde_json::json!({
                "member_id": pm.member_id,
                "role": pm.role,
            }));
    }

    Ok(Json(serde_json::to_value(map).unwrap_or(serde_json::json!({}))))
}

// ─── POST /workspaces/{slug}/workspace-views/ ────────────────────────────────

/// Persists the workspace member's view preferences.
///
/// Mirror of `WorkspaceMemberUserViewsEndpoint.post`
/// (`apps/api/plane/app/views/workspace/member.py:222-229`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-views/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 204, description = "View props updated"),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn update_workspace_views(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;

    let view_props = body
        .get("view_props")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let now = chrono::Utc::now().fixed_offset();
    let mut am: workspace_members::ActiveModel = member.into();
    am.view_props = Set(view_props);
    am.updated_at = Set(now);
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/workspaces/{slug}/workspace-views/`
///
/// Symmetric counterpart to `update_workspace_views`: returns the
/// `view_props` of the authenticated member for the workspace. The frontend
/// consumes it to pre-populate the saved filters UI.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-views/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "view_props del miembro"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn get_workspace_views(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    Ok(Json(serde_json::json!({
        "view_props": member.view_props,
    })))
}

// ─── GET + PATCH /workspaces/{slug}/invitations/{pk}/ ────────────────────────

/// Returns invitation detail.
///
/// Mirror of `WorkspaceInvitationsViewset.retrieve`
/// (`apps/api/plane/app/urls/workspace.py:29-33`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/invitations/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 200, description = "Invitation detail"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<InvitationResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    let invite = workspace_member_invites::Entity::find_by_id(pk)
        .active()
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(InvitationResponse::from(&invite)))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateInvitationRequest {
    pub role: Option<i16>,
}

/// Updates a pending invitation role.
///
/// Mirror of `WorkspaceInvitationsViewset.partial_update`
/// (`apps/api/plane/app/urls/workspace.py:29-33`).
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/invitations/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 200, description = "Updated invitation"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateInvitationRequest>,
) -> Result<Json<InvitationResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    let invite = workspace_member_invites::Entity::find_by_id(pk)
        .active()
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut am: workspace_member_invites::ActiveModel = invite.into();
    if let Some(role) = body.role {
        if ![ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&role) {
            return Err(AppError::BadRequest("Invalid role value".into()));
        }
        am.role = Set(role);
    }
    am.updated_at = Set(now);
    am.updated_by_id = Set(Some(user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(InvitationResponse::from(&updated)))
}

// ─── POST /workspaces/{slug}/invitations/{pk}/join/ ───────────────────────────

/// Responds to a workspace invitation (accept or decline).
///
/// Mirror of `WorkspaceJoinEndpoint.post`
/// (`apps/api/plane/app/views/workspace/invite.py`).
///
/// Body: `{ "token": "<invite_token>", "accepted": true|false }`
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct JoinWorkspaceRequest {
    pub token: String,
    pub accepted: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/invitations/{pk}/join/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 200, description = "Invitation response processed"),
        (status = 400, description = "Already responded"),
        (status = 403, description = "Invalid token"),
        (status = 404, description = "Invitation not found"),
    )
)]
pub async fn join_workspace_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<JoinWorkspaceRequest>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let invite = workspace_member_invites::Entity::find_by_id(pk)
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Validate token — constant-time to avoid timing attack
    let token_bytes = body.token.as_bytes();
    let invite_bytes = invite.token.as_bytes();
    let token_valid = token_bytes.len() == invite_bytes.len()
        && ct_memcmp::eq(token_bytes, invite_bytes);
    if !token_valid {
        return Err(AppError::Forbidden);
    }

    // Already responded
    if invite.responded_at.is_some() {
        return Err(AppError::BadRequest(
            "You have already responded to the invitation request".into(),
        ));
    }

    let accepted = body.accepted.unwrap_or(false);
    let now = chrono::Utc::now().fixed_offset();

    let mut am: workspace_member_invites::ActiveModel = invite.clone().into();
    am.accepted = Set(accepted);
    am.responded_at = Set(Some(now));
    am.updated_at = Set(now);
    am.updated_by_id = Set(Some(user.id));
    am.update(&state.db).await.map_err(AppError::Database)?;

    if accepted {
        // Verify if the invited user matches the authenticated one (by email)
        if user.email.as_deref() == Some(invite.email.as_str()) {
            // Find existing membership
            let existing = workspace_members::Entity::find()
                .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
                .filter(workspace_members::Column::MemberId.eq(user.id))
                .filter(workspace_members::Column::DeletedAt.is_null())
                .one(&state.db)
                .await
                .map_err(AppError::Database)?;

            if let Some(member) = existing {
                let mut mam: workspace_members::ActiveModel = member.into();
                mam.is_active = Set(true);
                mam.role = Set(invite.role);
                mam.updated_at = Set(now);
                mam.updated_by_id = Set(Some(user.id));
                mam.update(&state.db).await.map_err(AppError::Database)?;
            } else {
                let new_member = workspace_members::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    workspace_id: Set(ws.id),
                    member_id: Set(user.id),
                    role: Set(invite.role),
                    is_active: Set(true),
                    created_at: Set(now),
                    updated_at: Set(now),
                    created_by_id: Set(Some(user.id)),
                    updated_by_id: Set(Some(user.id)),
                    ..Default::default()
                };
                new_member.insert(&state.db).await.map_err(AppError::Database)?;
            }
        }

        // Delete accepted invitation
        let del_am: workspace_member_invites::ActiveModel = invite.into();
        del_am.delete(&state.db).await.map_err(AppError::Database)?;

        return Ok(axum::Json(serde_json::json!({
            "message": "Workspace Invitation Accepted"
        })));
    }

    Ok(axum::Json(serde_json::json!({
        "message": "Workspace Invitation was not accepted"
    })))
}

// ─── GET /workspaces/{slug}/invitations/{pk}/join — public ────────────────────
//
// Returns invitation details for the accept/decline page.
// This route is PUBLIC: the guest may not be authenticated yet.
// Mirror of WorkspaceJoinEndpoint.get in Django.

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/invitations/{pk}/join",
    tag = "Workspaces",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 200, description = "Invitation detail for join page"),
        (status = 404, description = "Invitation not found"),
    )
)]
pub async fn get_invitation_join(
    State(state): State<AppState>,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let invite = workspace_member_invites::Entity::find_by_id(pk)
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let logo_url = ws
        .logo_asset_id
        .map(|aid| format!("/api/assets/v2/static/{}/", aid))
        .or_else(|| ws.logo.clone())
        .unwrap_or_default();

    let app_base = state.config.app_base();
    let invite_link = format!(
        "{}/workspace-invitations/?invitation_id={}&email={}&slug={}",
        app_base.trim_end_matches('/'),
        invite.id,
        urlencoding::encode(invite.email.as_str()),
        ws.slug,
    );

    Ok(axum::Json(serde_json::json!({
        "id":           invite.id,
        "email":        invite.email,
        "message":      invite.message,
        "role":         invite.role,
        "token":        invite.token,
        "accepted":     invite.accepted,
        "responded_at": invite.responded_at,
        "invite_link":  invite_link,
        "workspace": {
            "id":       ws.id,
            "name":     ws.name,
            "slug":     ws.slug,
            "logo_url": logo_url,
        }
    })))
}

// ─── Workspace Themes ─────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct WorkspaceThemeResponse {
    pub id: Uuid,
    pub name: String,
    pub colors: serde_json::Value,
    pub workspace_id: Uuid,
    pub actor_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<crate::entities::workspace_themes::Model> for WorkspaceThemeResponse {
    fn from(m: crate::entities::workspace_themes::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            colors: m.colors,
            workspace_id: m.workspace_id,
            actor_id: m.actor_id,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateWorkspaceThemeRequest {
    pub name: String,
    pub colors: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkspaceThemeRequest {
    pub name: Option<String>,
    pub colors: Option<serde_json::Value>,
}

/// Lists all workspace themes.
///
/// Mirror of `WorkspaceThemeViewSet.list`
/// (`apps/api/plane/app/views/workspace/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-themes/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses((status = 200, description = "Theme list"))
)]
pub async fn list_workspace_themes(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<axum::Json<Vec<WorkspaceThemeResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let themes = crate::entities::workspace_themes::Entity::find()
        .filter(crate::entities::workspace_themes::Column::WorkspaceId.eq(ws.id))
        .filter(crate::entities::workspace_themes::Column::DeletedAt.is_null())
        .order_by_desc(crate::entities::workspace_themes::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(axum::Json(
        themes.into_iter().map(WorkspaceThemeResponse::from).collect(),
    ))
}

/// Creates a new workspace theme.
///
/// Mirror of `WorkspaceThemeViewSet.create`
/// (`apps/api/plane/app/views/workspace/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-themes/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Theme created"),
        (status = 400, description = "Duplicate name"),
    )
)]
pub async fn create_workspace_theme(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateWorkspaceThemeRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<WorkspaceThemeResponse>), AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let now = chrono::Utc::now().fixed_offset();
    let new_theme = crate::entities::workspace_themes::ActiveModel {
        id: Set(Uuid::new_v4()),
        workspace_id: Set(ws.id),
        actor_id: Set(user.id),
        name: Set(body.name),
        colors: Set(body.colors.unwrap_or_else(|| serde_json::json!({}))),
        created_at: Set(now),
        updated_at: Set(now),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        deleted_at: Set(None),
    };

    let theme = new_theme
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(WorkspaceThemeResponse::from(theme)),
    ))
}

/// Gets theme detail.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-themes/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Theme UUID"),
    ),
    responses(
        (status = 200, description = "Theme detail"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_workspace_theme(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<axum::Json<WorkspaceThemeResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let theme = crate::entities::workspace_themes::Entity::find_by_id(pk)
        .filter(crate::entities::workspace_themes::Column::WorkspaceId.eq(ws.id))
        .filter(crate::entities::workspace_themes::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(axum::Json(WorkspaceThemeResponse::from(theme)))
}

/// Updates a workspace theme.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/workspace-themes/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Theme UUID"),
    ),
    responses(
        (status = 200, description = "Theme updated"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_workspace_theme(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateWorkspaceThemeRequest>,
) -> Result<axum::Json<WorkspaceThemeResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let theme = crate::entities::workspace_themes::Entity::find_by_id(pk)
        .filter(crate::entities::workspace_themes::Column::WorkspaceId.eq(ws.id))
        .filter(crate::entities::workspace_themes::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut am: crate::entities::workspace_themes::ActiveModel = theme.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(colors) = body.colors {
        am.colors = Set(colors);
    }
    am.updated_at = Set(now);
    am.updated_by_id = Set(Some(user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(axum::Json(WorkspaceThemeResponse::from(updated)))
}

/// Deletes (soft-deletes) a workspace theme.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/workspace-themes/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Theme UUID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_workspace_theme(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let theme = crate::entities::workspace_themes::Entity::find_by_id(pk)
        .filter(crate::entities::workspace_themes::Column::WorkspaceId.eq(ws.id))
        .filter(crate::entities::workspace_themes::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut am: crate::entities::workspace_themes::ActiveModel = theme.into();
    am.deleted_at = Set(Some(now));
    am.updated_at = Set(now);
    am.updated_by_id = Set(Some(user.id));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
