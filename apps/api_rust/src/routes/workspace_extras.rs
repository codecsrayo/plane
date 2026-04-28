// src/routes/workspace_extras.rs
//! Workspace Endpoints — additional sub-modules.
//!
//! Covers Django equivalents of:
//!   workspace/favorite.py        → user favorites
//!   workspace/home.py            → home preferences
//!   workspace/quick_link.py      → quick links
//!   workspace/recent_visit.py    → recent visits
//!   workspace/sticky.py          → stickies
//!   workspace/user_preference.py → user preferences
//!   workspace/draft.py           → draft issues
//!   workspace/cycle.py           → workspace-level cycles
//!   workspace/module.py          → workspace-level modules
//!   workspace/estimate.py        → workspace-level estimates
//!   workspace/label.py           → workspace-level labels
//!   workspace/state.py           → workspace-level states

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::{
        cycle_issues, cycles, draft_issues, estimates, estimate_points, file_assets,
        issue_assignees, issue_labels, issues, labels, module_issues, modules,
        projects, stickies, states, user_favorites, user_recent_visits,
        workspace_home_preferences, workspace_user_links,
        workspace_user_preferences, workspace_user_properties,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::pagination,
    AppState,
};

// ── Inline Permissions ──────────────────────────────────────────────────────

/// Ensures the member role is at least MEMBER (not GUEST).
fn require_member_or_admin(role: i16) -> Result<(), AppError> {
    // ADMIN=20, MEMBER=15, VIEWER=10, GUEST=5
    if role < 15 {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// FAVORITES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct FavoriteResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_identifier: Option<Uuid>,
    pub name: Option<String>,
    pub is_folder: bool,
    pub sequence: f64,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<user_favorites::Model> for FavoriteResponse {
    fn from(m: user_favorites::Model) -> Self {
        Self {
            id: m.id,
            entity_type: m.entity_type,
            entity_identifier: m.entity_identifier,
            name: m.name,
            is_folder: m.is_folder,
            sequence: m.sequence,
            parent_id: m.parent_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            user_id: m.user_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFavoriteRequest {
    pub entity_type: String,
    pub entity_identifier: Option<Uuid>,
    pub name: Option<String>,
    pub is_folder: Option<bool>,
    pub sequence: Option<f64>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFavoriteRequest {
    pub name: Option<String>,
    pub sequence: Option<f64>,
    pub parent_id: Option<Uuid>,
}

/// GET /workspaces/{slug}/user-favorites/
///
/// Lists user favorites in the workspace (no parent, no pages).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-favorites/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of favorites"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_favorites(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let favorites = user_favorites::Entity::find()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::ParentId.is_null())
        .filter(user_favorites::Column::DeletedAt.is_null())
        .order_by_asc(user_favorites::Column::Sequence)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<FavoriteResponse> = favorites.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// POST /workspaces/{slug}/user-favorites/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/user-favorites/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Favorite created or existing"),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create_favorite(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateFavoriteRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    require_member_or_admin(member.role)?;

    // Idempotency: if it already exists with same entity_identifier + entity_type, returns existing
    if let Some(eid) = body.entity_identifier {
        if let Some(existing) = user_favorites::Entity::find()
            .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
            .filter(user_favorites::Column::UserId.eq(user_id))
            .filter(user_favorites::Column::EntityType.eq(&body.entity_type))
            .filter(user_favorites::Column::EntityIdentifier.eq(eid))
            .filter(user_favorites::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
        {
            let resp: FavoriteResponse = existing.into();
            return Ok((StatusCode::OK, Json(resp)));
        }
    }

    let new_fav = user_favorites::ActiveModel {
        id: Set(Uuid::new_v4()),
        entity_type: Set(body.entity_type),
        entity_identifier: Set(body.entity_identifier),
        name: Set(body.name),
        is_folder: Set(body.is_folder.unwrap_or(false)),
        sequence: Set(body.sequence.unwrap_or(65535.0)),
        parent_id: Set(body.parent_id),
        project_id: Set(body.project_id),
        workspace_id: Set(ws.id),
        user_id: Set(user_id),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    };

    let saved = new_fav.insert(db).await.map_err(AppError::Database)?;
    let resp: FavoriteResponse = saved.into();
    // 201 when creating a new resource. The idempotent branch above returns 200.
    Ok((StatusCode::CREATED, Json(resp)))
}

/// PATCH /workspaces/{slug}/user-favorites/{favorite_id}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/user-favorites/{favorite_id}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("favorite_id" = Uuid, Path, description = "Favorite ID"),
    ),
    responses(
        (status = 200, description = "Favorite updated"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_favorite(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, favorite_id)): Path<(String, Uuid)>,
    Json(body): Json<UpdateFavoriteRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    require_member_or_admin(member.role)?;

    let fav = user_favorites::Entity::find_by_id(favorite_id)
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: user_favorites::ActiveModel = fav.into();
    if let Some(name) = body.name {
        active.name = Set(Some(name));
    }
    if let Some(seq) = body.sequence {
        active.sequence = Set(seq);
    }
    if let Some(pid) = body.parent_id {
        active.parent_id = Set(Some(pid));
    }
    active.updated_at = Set(chrono::Utc::now().into());
    active.updated_by_id = Set(Some(user_id));

    let saved = active.update(db).await.map_err(AppError::Database)?;
    let resp: FavoriteResponse = saved.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// DELETE /workspaces/{slug}/user-favorites/{favorite_id}/
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/user-favorites/{favorite_id}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("favorite_id" = Uuid, Path, description = "Favorite ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_favorite(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, favorite_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    require_member_or_admin(member.role)?;

    let fav = user_favorites::Entity::find_by_id(favorite_id)
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Hard delete (Django also does it with soft=False on this endpoint)
    user_favorites::Entity::delete_by_id(fav.id)
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /workspaces/{slug}/user-favorites/{favorite_id}/children/
///
/// Lists child favorites of a group/folder.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-favorites/{favorite_id}/children/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("favorite_id" = Uuid, Path, description = "Favorite parent ID"),
    ),
    responses(
        (status = 200, description = "List of child favorites"),
    )
)]
pub async fn list_favorite_children(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, favorite_id)): Path<(String, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    require_member_or_admin(member.role)?;

    let children = user_favorites::Entity::find()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::ParentId.eq(favorite_id))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .order_by_asc(user_favorites::Column::Sequence)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<FavoriteResponse> = children.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ═══════════════════════════════════════════════════════════════════════════
// HOME PREFERENCES
// ═══════════════════════════════════════════════════════════════════════════

/// GET response — mirror of Django `.values("key", "is_enabled", "config", "sort_order")`.
/// Does not include `id` because Django does not expose it in this endpoint either.
#[derive(Debug, Serialize)]
pub struct HomePreferenceResponse {
    pub key: String,
    pub is_enabled: bool,
    pub config: JsonValue,
    pub sort_order: f64,
}

impl From<workspace_home_preferences::Model> for HomePreferenceResponse {
    fn from(m: workspace_home_preferences::Model) -> Self {
        Self {
            key: m.key,
            is_enabled: m.is_enabled,
            config: m.config,
            sort_order: m.sort_order,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateHomePreferenceRequest {
    pub is_enabled: Option<bool>,
    pub config: Option<JsonValue>,
    pub sort_order: Option<f64>,
}

/// Keys that are auto-seeded in the GET.
///
/// Mirror of Django `WorkspaceHomePreference.HomeWidgetKeys.choices`
/// EXCLUDING `quick_tutorial` and `new_at_plane` (explicitly filtered
/// in `workspace/home.py:33-36`).
const HOME_PREFERENCE_KEYS: &[&str] = &["quick_links", "recents", "my_stickies"];

/// GET /workspaces/{slug}/home-preferences/
///
/// Mirror of Django `WorkspaceHomePreferenceViewSet.get`
/// (`plane/app/views/workspace/home.py:23`).
///
/// Auto-seed behavior: for each missing key in
/// `HOME_PREFERENCE_KEYS` a row is created with defaults (is_enabled=true,
/// config={}, sort_order = 1000 − position). This ensures the
/// frontend always receives a complete set of widgets without needing
/// a separate initialization flow.
///
/// `INSERT ... ON CONFLICT DO NOTHING` is used to avoid race conditions
/// between tabs of the same user, consistent with Django's `bulk_create(
/// ignore_conflicts=True)`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/home-preferences/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Home preferences"),
    )
)]
pub async fn get_home_preferences(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // ── 1. Read existing keys ──────────────────────────────────────────────
    let existing_keys: std::collections::HashSet<String> =
        workspace_home_preferences::Entity::find()
            .filter(workspace_home_preferences::Column::WorkspaceId.eq(ws.id))
            .filter(workspace_home_preferences::Column::UserId.eq(user_id))
            .filter(workspace_home_preferences::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| p.key)
            .collect();

    // ── 2. Auto-seed missing keys ──────────────────────────────────────────
    //
    // sort_order = 1000 − position (mirror Django: `sort_order = 1000 - sort_order_counter`).
    let now = chrono::Utc::now().fixed_offset();
    let missing: Vec<_> = HOME_PREFERENCE_KEYS
        .iter()
        .filter(|k| !existing_keys.contains(**k))
        .collect();

    if !missing.is_empty() {
        let to_insert: Vec<workspace_home_preferences::ActiveModel> = missing
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let sort_order = 1000.0 - (i as f64 + 1.0);
                workspace_home_preferences::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    key: Set(key.to_string()),
                    is_enabled: Set(true),
                    config: Set(serde_json::json!({})),
                    sort_order: Set(sort_order),
                    user_id: Set(user_id),
                    workspace_id: Set(ws.id),
                    created_by_id: Set(Some(user_id)),
                    updated_by_id: Set(Some(user_id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                }
            })
            .collect();

        // ON CONFLICT DO NOTHING — mirror of bulk_create(ignore_conflicts=True).
        // The partial unique constraint covers (workspace, user, key) WHERE deleted_at IS NULL.
        use sea_orm::sea_query::{Expr, OnConflict};
        workspace_home_preferences::Entity::insert_many(to_insert)
            .on_conflict(
                OnConflict::columns([
                    workspace_home_preferences::Column::WorkspaceId,
                    workspace_home_preferences::Column::UserId,
                    workspace_home_preferences::Column::Key,
                ])
                .target_and_where(
                    Expr::col(workspace_home_preferences::Column::DeletedAt).is_null(),
                )
                .do_nothing()
                .to_owned(),
            )
            .do_nothing()
            .exec(db)
            .await
            .map_err(AppError::Database)?;
    }

    // ── 3. Read all (including those just inserted) ────────────────────────
    let prefs = workspace_home_preferences::Entity::find()
        .filter(workspace_home_preferences::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_home_preferences::Column::UserId.eq(user_id))
        .filter(workspace_home_preferences::Column::DeletedAt.is_null())
        .order_by_asc(workspace_home_preferences::Column::SortOrder)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<HomePreferenceResponse> = prefs.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// PATCH /workspaces/{slug}/home-preferences/{key}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/home-preferences/{key}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("key" = String, Path, description = "Preference key"),
    ),
    responses(
        (status = 200, description = "Preference updated"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_home_preference(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, key)): Path<(String, String)>,
    Json(body): Json<UpdateHomePreferenceRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // Upsert: if the key doesn't exist, we create it with defaults + overrides.
    // The frontend assumes PATCH is idempotent (no explicit pre-seed
    // needed, avoiding a previous GET call).
    let existing = workspace_home_preferences::Entity::find()
        .filter(workspace_home_preferences::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_home_preferences::Column::UserId.eq(user_id))
        .filter(workspace_home_preferences::Column::Key.eq(&key))
        .filter(workspace_home_preferences::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?;

    let saved = if let Some(pref) = existing {
        let mut active: workspace_home_preferences::ActiveModel = pref.into();
        if let Some(v) = body.is_enabled {
            active.is_enabled = Set(v);
        }
        if let Some(v) = body.config {
            active.config = Set(v);
        }
        if let Some(v) = body.sort_order {
            active.sort_order = Set(v);
        }
        active.updated_at = Set(chrono::Utc::now().into());
        active.updated_by_id = Set(Some(user_id));
        active.update(db).await.map_err(AppError::Database)?
    } else {
        let now = chrono::Utc::now().fixed_offset();
        workspace_home_preferences::ActiveModel {
            id: Set(Uuid::new_v4()),
            key: Set(key.clone()),
            is_enabled: Set(body.is_enabled.unwrap_or(true)),
            config: Set(body.config.unwrap_or_else(|| serde_json::json!({}))),
            sort_order: Set(body.sort_order.unwrap_or(0.0)),
            user_id: Set(user_id),
            workspace_id: Set(ws.id),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(db)
        .await
        .map_err(AppError::Database)?
    };

    let resp: HomePreferenceResponse = saved.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// GET /workspaces/{slug}/home-preferences/{key}/
///
/// Returns the individual preference of the authenticated user for `key`.
/// 404 if the key has not been initialized yet (the frontend falls back to
/// PATCH upsert to create it on-demand).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/home-preferences/{key}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("key"  = String, Path, description = "Preference key"),
    ),
    responses(
        (status = 200, description = "Preference"),
        (status = 401, description = "Unauthenticated"),
        (status = 404, description = "Key not initialized"),
    )
)]
pub async fn get_home_preference_key(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, key)): Path<(String, String)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let pref = workspace_home_preferences::Entity::find()
        .filter(workspace_home_preferences::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_home_preferences::Column::UserId.eq(user_id))
        .filter(workspace_home_preferences::Column::Key.eq(&key))
        .filter(workspace_home_preferences::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let resp: HomePreferenceResponse = pref.into();
    Ok((StatusCode::OK, Json(resp)))
}

// ═══════════════════════════════════════════════════════════════════════════
// QUICK LINKS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct QuickLinkResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub url: String,
    pub metadata: JsonValue,
    pub workspace_id: Uuid,
    pub owner_id: Uuid,
    pub project_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<workspace_user_links::Model> for QuickLinkResponse {
    fn from(m: workspace_user_links::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            url: m.url,
            metadata: m.metadata,
            workspace_id: m.workspace_id,
            owner_id: m.owner_id,
            project_id: m.project_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateQuickLinkRequest {
    pub title: Option<String>,
    pub url: String,
    pub metadata: Option<JsonValue>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateQuickLinkRequest {
    pub title: Option<String>,
    pub url: Option<String>,
    pub metadata: Option<JsonValue>,
}

/// GET /workspaces/{slug}/quick-links/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/quick-links/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of quick links"),
    )
)]
pub async fn list_quick_links(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let links = workspace_user_links::Entity::find()
        .filter(workspace_user_links::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_user_links::Column::OwnerId.eq(user_id))
        .filter(workspace_user_links::Column::DeletedAt.is_null())
        .order_by_desc(workspace_user_links::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<QuickLinkResponse> = links.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// POST /workspaces/{slug}/quick-links/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/quick-links/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Quick link created"),
    )
)]
pub async fn create_quick_link(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateQuickLinkRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    if body.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".into()));
    }

    let new = workspace_user_links::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(body.title),
        url: Set(body.url),
        metadata: Set(body.metadata.unwrap_or(serde_json::json!({}))),
        workspace_id: Set(ws.id),
        owner_id: Set(user_id),
        project_id: Set(body.project_id),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    };

    let saved = new.insert(db).await.map_err(AppError::Database)?;
    let resp: QuickLinkResponse = saved.into();
    Ok((StatusCode::CREATED, Json(resp)))
}

/// PATCH /workspaces/{slug}/quick-links/{pk}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/quick-links/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Quick link ID"),
    ),
    responses(
        (status = 200, description = "Updated"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_quick_link(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateQuickLinkRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let link = workspace_user_links::Entity::find_by_id(pk)
        .filter(workspace_user_links::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_user_links::Column::OwnerId.eq(user_id))
        .filter(workspace_user_links::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: workspace_user_links::ActiveModel = link.into();
    if let Some(t) = body.title {
        active.title = Set(Some(t));
    }
    if let Some(u) = body.url {
        active.url = Set(u);
    }
    if let Some(m) = body.metadata {
        active.metadata = Set(m);
    }
    active.updated_at = Set(chrono::Utc::now().into());
    active.updated_by_id = Set(Some(user_id));

    let saved = active.update(db).await.map_err(AppError::Database)?;
    let resp: QuickLinkResponse = saved.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// DELETE /workspaces/{slug}/quick-links/{pk}/
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/quick-links/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Quick link ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_quick_link(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let link = workspace_user_links::Entity::find_by_id(pk)
        .filter(workspace_user_links::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_user_links::Column::OwnerId.eq(user_id))
        .filter(workspace_user_links::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    workspace_user_links::Entity::delete_by_id(link.id)
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// RECENT VISITS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct RecentVisitResponse {
    pub id: Uuid,
    pub entity_name: String,
    pub entity_identifier: Option<Uuid>,
    pub visited_at: DateTime<Utc>,
    pub workspace_id: Uuid,
    pub project_id: Option<Uuid>,
    pub user_id: Uuid,
}

impl From<user_recent_visits::Model> for RecentVisitResponse {
    fn from(m: user_recent_visits::Model) -> Self {
        Self {
            id: m.id,
            entity_name: m.entity_name,
            entity_identifier: m.entity_identifier,
            visited_at: m.visited_at.into(),
            workspace_id: m.workspace_id,
            project_id: m.project_id,
            user_id: m.user_id,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RecentVisitQuery {
    pub entity_name: Option<String>,
}

/// GET /workspaces/{slug}/recent-visits/
///
/// Parity with Django (`plane/app/views/workspace/recent_visit.py::UserRecentVisitViewSet`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/recent-visits/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("entity_name" = Option<String>, Query, description = "Filter by entity type"),
    ),
    responses(
        (status = 200, description = "List of recent visits (max 20)"),
    )
)]
pub async fn list_recent_visits(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Query(q): Query<RecentVisitQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // Valid entities only: issue, page, project
    let allowed = ["issue", "page", "project"];

    let mut query = user_recent_visits::Entity::find()
        .filter(user_recent_visits::Column::WorkspaceId.eq(ws.id))
        .filter(user_recent_visits::Column::UserId.eq(user_id))
        .filter(user_recent_visits::Column::DeletedAt.is_null());

    if let Some(entity_name) = &q.entity_name {
        if allowed.contains(&entity_name.as_str()) {
            query = query.filter(user_recent_visits::Column::EntityName.eq(entity_name));
        }
    } else {
        // Filter only allowed entities using sea_orm OR condition
        use sea_orm::Condition;
        let mut cond = Condition::any();
        for name in &allowed {
            cond = cond.add(user_recent_visits::Column::EntityName.eq(*name));
        }
        query = query.filter(cond);
    }

    let visits = query
        .order_by_desc(user_recent_visits::Column::VisitedAt)
        .limit(20)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<RecentVisitResponse> = visits.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ═══════════════════════════════════════════════════════════════════════════
// STICKIES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct StickyResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: JsonValue,
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub logo_props: JsonValue,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub sort_order: f64,
    pub workspace_id: Uuid,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<stickies::Model> for StickyResponse {
    fn from(m: stickies::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            description_html: m.description_html,
            description_stripped: m.description_stripped,
            logo_props: m.logo_props,
            color: m.color,
            background_color: m.background_color,
            sort_order: m.sort_order,
            workspace_id: m.workspace_id,
            owner_id: m.owner_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateStickyRequest {
    pub name: Option<String>,
    pub description: Option<JsonValue>,
    pub description_html: Option<String>,
    pub description_stripped: Option<String>,
    pub logo_props: Option<JsonValue>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub sort_order: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStickyRequest {
    pub name: Option<String>,
    pub description: Option<JsonValue>,
    pub description_html: Option<String>,
    pub description_stripped: Option<String>,
    pub logo_props: Option<JsonValue>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub sort_order: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StickyListQuery {
    pub query: Option<String>,
    /// Cursor en formato Django: `"per_page:offset:is_prev"` (e.g. `"20:0:0"`).
    pub cursor: Option<String>,
    /// Optional per_page override (Django takes it before the cursor).
    pub per_page: Option<u64>,
}

/// GET /workspaces/{slug}/stickies/
///
/// Parity with Django (`plane/app/views/workspace/sticky.py::WorkspaceStickyViewSet.list`).
/// Returns paged shape: `{ results, total_count, next_cursor, prev_cursor,
/// next_page_results, prev_page_results, count, total_pages, total_results,
/// grouped_by, sub_grouped_by, extra_stats }`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/stickies/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("query" = Option<String>, Query, description = "Search in description"),
        ("cursor" = Option<String>, Query, description = "Django Cursor: per_page:offset:is_prev"),
        ("per_page" = Option<u64>, Query, description = "Override per_page"),
    ),
    responses(
        (status = 200, description = "Paged list of stickies"),
    )
)]
pub async fn list_stickies(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Query(q): Query<StickyListQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // Parity with Django `WorkspaceStickyViewSet.list` + `paginate(default_per_page=20)`.
    const DEFAULT_PER_PAGE: u64 = 20;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    let mut query = stickies::Entity::find()
        .filter(stickies::Column::WorkspaceId.eq(ws.id))
        .filter(stickies::Column::OwnerId.eq(user_id))
        .filter(stickies::Column::DeletedAt.is_null());

    if let Some(search) = q.query.as_deref().filter(|s| !s.is_empty()) {
        query = query.filter(stickies::Column::DescriptionStripped.contains(search));
    }

    // Count and page must use the SAME query (same filters) — we clone before
    // applying the order to avoid divergence.
    let total_count = query
        .clone()
        .count(db)
        .await
        .map_err(AppError::Database)?;

    let items = query
        .order_by_desc(stickies::Column::SortOrder)
        .paginate(db, limit)
        .fetch_page(cursor.offset)
        .await
        .map_err(AppError::Database)?;

    let results: Vec<StickyResponse> = items.into_iter().map(Into::into).collect();
    let body = pagination::build_response(results, total_count, limit, cursor.offset);

    Ok((StatusCode::OK, Json(body)))
}

/// POST /workspaces/{slug}/stickies/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/stickies/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Sticky created"),
    )
)]
pub async fn create_sticky(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateStickyRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let new = stickies::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or(serde_json::json!({}))),
        description_html: Set(body.description_html.unwrap_or_default()),
        description_stripped: Set(body.description_stripped),
        description_binary: Set(None),
        logo_props: Set(body.logo_props.unwrap_or(serde_json::json!({}))),
        color: Set(body.color),
        background_color: Set(body.background_color),
        sort_order: Set(body.sort_order.unwrap_or(65535.0)),
        workspace_id: Set(ws.id),
        owner_id: Set(user_id),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        deleted_at: Set(None),
    };

    let saved = new.insert(db).await.map_err(AppError::Database)?;
    let resp: StickyResponse = saved.into();
    Ok((StatusCode::CREATED, Json(resp)))
}

/// PATCH /workspaces/{slug}/stickies/{pk}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/stickies/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Sticky ID"),
    ),
    responses(
        (status = 200, description = "Sticky updated"),
        (status = 403, description = "Forbidden (creator only)"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_sticky(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateStickyRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let sticky = stickies::Entity::find_by_id(pk)
        .filter(stickies::Column::WorkspaceId.eq(ws.id))
        .filter(stickies::Column::OwnerId.eq(user_id))
        .filter(stickies::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: stickies::ActiveModel = sticky.into();
    if let Some(v) = body.name {
        active.name = Set(Some(v));
    }
    if let Some(v) = body.description {
        active.description = Set(v);
    }
    if let Some(v) = body.description_html {
        active.description_html = Set(v);
    }
    if let Some(v) = body.description_stripped {
        active.description_stripped = Set(Some(v));
    }
    if let Some(v) = body.logo_props {
        active.logo_props = Set(v);
    }
    if let Some(v) = body.color {
        active.color = Set(Some(v));
    }
    if let Some(v) = body.background_color {
        active.background_color = Set(Some(v));
    }
    if let Some(v) = body.sort_order {
        active.sort_order = Set(v);
    }
    active.updated_at = Set(chrono::Utc::now().into());
    active.updated_by_id = Set(Some(user_id));

    let saved = active.update(db).await.map_err(AppError::Database)?;
    let resp: StickyResponse = saved.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// DELETE /workspaces/{slug}/stickies/{pk}/
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/stickies/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Sticky ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_sticky(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let sticky = stickies::Entity::find_by_id(pk)
        .filter(stickies::Column::WorkspaceId.eq(ws.id))
        .filter(stickies::Column::OwnerId.eq(user_id))
        .filter(stickies::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Soft delete
    let mut active: stickies::ActiveModel = sticky.into();
    active.deleted_at = Set(Some(chrono::Utc::now().into()));
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// USER PREFERENCES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct UserPreferenceEntry {
    pub is_pinned: bool,
    pub sort_order: f64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserPreferenceItem {
    pub key: String,
    pub is_pinned: Option<bool>,
    pub sort_order: Option<f64>,
}

/// GET /workspaces/{slug}/sidebar-preferences/
///
/// Mirror of Django `WorkspaceUserPreferenceViewSet.get` in
/// `plane/app/views/workspace/user_preference.py:26`. Returns the map
/// `{key: {is_pinned, sort_order}}` and SEEDS the missing keys with
/// defaults (sort_order = 65535 + i*10000, pinned for drafts/your_work/
/// stickies) so that the frontend has a consistent state from the
/// first render.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/sidebar-preferences/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "User preferences (key map → {is_pinned, sort_order})"),
    )
)]
pub async fn get_user_preferences(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // ── 1. Seed missing keys ────────────────────────────────────────────────
    //
    // Exact mirror of Django: GET is responsible for ensuring that
    // rows exist for each UserPreferenceKeys.choices. If they don't exist,
    // they are created with defaults; if they already exist (race condition
    // between tabs) they are ignored via ON CONFLICT DO NOTHING.
    //
    // Order and defaults are normative — the frontend assumes these values.
    const PREFERENCE_KEYS: &[&str] = &[
        "views",
        "active_cycles",
        "analytics",
        "drafts",
        "your_work",
        "archives",
        "stickies",
    ];
    const PINNED_BY_DEFAULT: &[&str] = &["drafts", "your_work", "stickies"];

    // Keys already present for this (workspace, user).
    let existing_keys: std::collections::HashSet<String> =
        workspace_user_preferences::Entity::find()
            .filter(workspace_user_preferences::Column::WorkspaceId.eq(ws.id))
            .filter(workspace_user_preferences::Column::UserId.eq(user_id))
            .filter(workspace_user_preferences::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| p.key)
            .collect();

    // Build ActiveModel only for the missing keys, maintaining the
    // same sort_order calculation as Django: 65535 + i*10000 where `i` is
    // the position within the subset of missing keys (not the total).
    let now = chrono::Utc::now().fixed_offset();
    let mut to_insert = Vec::new();
    for (i, key) in PREFERENCE_KEYS
        .iter()
        .filter(|k| !existing_keys.contains(**k))
        .enumerate()
    {
        let sort_order = 65535.0_f64 + (i as f64) * 10000.0;
        let is_pinned = PINNED_BY_DEFAULT.contains(key);
        to_insert.push(workspace_user_preferences::ActiveModel {
            id: Set(Uuid::new_v4()),
            key: Set(key.to_string()),
            is_pinned: Set(is_pinned),
            sort_order: Set(sort_order),
            user_id: Set(user_id),
            workspace_id: Set(ws.id),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        });
    }

    if !to_insert.is_empty() {
        // ON CONFLICT DO NOTHING — mirror of bulk_create(ignore_conflicts=True).
        // The partial unique constraint covers (workspace_id, user_id, key) when
        // deleted_at IS NULL, so re-applying the GET concurrently from another
        // tab does not break.
        //
        // IMPORTANT: PostgreSQL REQUIRES repeating the `WHERE deleted_at IS NULL`
        // predicate in the conflict target to infer a partial arbiter index — without
        // `.target_and_where(...)` the INSERT blows up with "there is no unique or
        // exclusion constraint matching the ON CONFLICT specification" even if the
        // columns match the constraint exactly. See
        // plane/db/models/workspace.py:443-451 (Django constraint) and
        // https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT
        // ("index_predicate … must satisfy arbiter indexes").
        use sea_orm::sea_query::{Expr, OnConflict};
        workspace_user_preferences::Entity::insert_many(to_insert)
            .on_conflict(
                OnConflict::columns([
                    workspace_user_preferences::Column::WorkspaceId,
                    workspace_user_preferences::Column::UserId,
                    workspace_user_preferences::Column::Key,
                ])
                .target_and_where(
                    Expr::col(workspace_user_preferences::Column::DeletedAt).is_null(),
                )
                .do_nothing()
                .to_owned(),
            )
            .do_nothing()
            .exec(db)
            .await
            .map_err(AppError::Database)?;
    }

    // ── 2. Read all preferences (including those just inserted) ────────────
    let prefs = workspace_user_preferences::Entity::find()
        .filter(workspace_user_preferences::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_user_preferences::Column::UserId.eq(user_id))
        .filter(workspace_user_preferences::Column::DeletedAt.is_null())
        .order_by_asc(workspace_user_preferences::Column::SortOrder)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Returns map { key: {is_pinned, sort_order} } same as Django
    let map: std::collections::HashMap<String, UserPreferenceEntry> = prefs
        .into_iter()
        .map(|p| {
            (
                p.key,
                UserPreferenceEntry {
                    is_pinned: p.is_pinned,
                    sort_order: p.sort_order,
                },
            )
        })
        .collect();

    Ok((StatusCode::OK, Json(map)))
}

/// PATCH /workspaces/{slug}/sidebar-preferences/
///
/// Body: array of objects `{key, is_pinned?, sort_order?}`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/sidebar-preferences/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Updated"),
    )
)]
pub async fn update_user_preferences(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<Vec<UpdateUserPreferenceItem>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    for item in body {
        if let Some(pref) = workspace_user_preferences::Entity::find()
            .filter(workspace_user_preferences::Column::WorkspaceId.eq(ws.id))
            .filter(workspace_user_preferences::Column::UserId.eq(user_id))
            .filter(workspace_user_preferences::Column::Key.eq(&item.key))
            .filter(workspace_user_preferences::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
        {
            let mut active: workspace_user_preferences::ActiveModel = pref.into();
            if let Some(v) = item.is_pinned {
                active.is_pinned = Set(v);
            }
            if let Some(v) = item.sort_order {
                active.sort_order = Set(v);
            }
            active.updated_at = Set(chrono::Utc::now().into());
            active.update(db).await.map_err(AppError::Database)?;
        }
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Successfully updated"}))))
}

// ═══════════════════════════════════════════════════════════════════════════
// DRAFT ISSUES
// ═══════════════════════════════════════════════════════════════════════════

// ── N-to-M sync helpers for Draft Issues ───────────────────────────────────
//
// Mirror the logic of `DraftIssueCreateSerializer.create` / `.update` in
// `apps/api/plane/app/serializers/draft.py:142-297`. Django does a hard-delete
// (`.delete()`) on existing assignees/labels/cycles/modules and then
// `bulk_create` with the new ones. We replicate this behavior with
// hard-delete + insert in the same transaction.
//
// NOTE: draft_issue_* does NOT have logical soft-delete in Django (the
// `.delete()` on `BaseManager` is a hard-delete because the tables are
// declared without soft-delete). For parity, we do hard-delete here.

/// Replaces draft assignees with `new_ids` (hard-delete existing + insert).
async fn sync_draft_assignees(
    txn: &sea_orm::DatabaseTransaction,
    draft_id: Uuid,
    project_id: Option<Uuid>,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    use crate::entities::draft_issue_assignees;
    draft_issue_assignees::Entity::delete_many()
        .filter(draft_issue_assignees::Column::DraftIssueId.eq(draft_id))
        .exec(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for assignee_id in new_ids {
        draft_issue_assignees::ActiveModel {
            id: Set(Uuid::new_v4()),
            draft_issue_id: Set(draft_id),
            assignee_id: Set(*assignee_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(txn)
        .await
        .map_err(AppError::Database)?;
    }
    Ok(())
}

/// Replaces draft labels with `new_ids`.
async fn sync_draft_labels(
    txn: &sea_orm::DatabaseTransaction,
    draft_id: Uuid,
    project_id: Option<Uuid>,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    use crate::entities::draft_issue_labels;
    draft_issue_labels::Entity::delete_many()
        .filter(draft_issue_labels::Column::DraftIssueId.eq(draft_id))
        .exec(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for label_id in new_ids {
        draft_issue_labels::ActiveModel {
            id: Set(Uuid::new_v4()),
            draft_issue_id: Set(draft_id),
            label_id: Set(*label_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(txn)
        .await
        .map_err(AppError::Database)?;
    }
    Ok(())
}

/// Replaces the (only) draft cycle. `new_id = None` only deletes.
///
/// Parity with draft.py:266-276: always deletes existing; only creates a
/// new one if `cycle_id` is truthy (in Django: not None, not ""; here not
/// `Option<Option<Uuid>>::Some(None)`).
async fn sync_draft_cycle(
    txn: &sea_orm::DatabaseTransaction,
    draft_id: Uuid,
    project_id: Option<Uuid>,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_id: Option<Uuid>,
) -> Result<(), AppError> {
    use crate::entities::draft_issue_cycles;
    draft_issue_cycles::Entity::delete_many()
        .filter(draft_issue_cycles::Column::DraftIssueId.eq(draft_id))
        .exec(txn)
        .await
        .map_err(AppError::Database)?;

    if let Some(cid) = new_id {
        let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
        draft_issue_cycles::ActiveModel {
            id: Set(Uuid::new_v4()),
            draft_issue_id: Set(draft_id),
            cycle_id: Set(cid),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(txn)
        .await
        .map_err(AppError::Database)?;
    }
    Ok(())
}

/// Replaces draft modules with `new_ids`.
async fn sync_draft_modules(
    txn: &sea_orm::DatabaseTransaction,
    draft_id: Uuid,
    project_id: Option<Uuid>,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    use crate::entities::draft_issue_modules;
    draft_issue_modules::Entity::delete_many()
        .filter(draft_issue_modules::Column::DraftIssueId.eq(draft_id))
        .exec(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for module_id in new_ids {
        draft_issue_modules::ActiveModel {
            id: Set(Uuid::new_v4()),
            draft_issue_id: Set(draft_id),
            module_id: Set(*module_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(txn)
        .await
        .map_err(AppError::Database)?;
    }
    Ok(())
}

/// Response shape for draft issues. Parity with Django
/// `DraftIssueSerializer` (apps/api/plane/app/serializers/draft.py:300-334)
/// and with the frontend type `TWorkspaceDraftIssue`
/// (packages/types/src/workspace-draft-issues/base.ts:9).
///
/// # Names (differences vs DB columns)
/// - `estimate_point` (not `_id`) — DRF exposes the FK with the name declared
///   in `Meta.fields`. The DB column is `estimate_point_id`; the mapping is
///   done by the hydrator.
/// - `created_by` / `updated_by` (not `_id`) — same reason; `BaseSerializer`
///   exposes these FKs with the field name.
/// - `workspace_id` is NOT exposed — Django does not include it in `Meta.fields`.
///
/// # Annotated fields (ArrayAgg/Subquery in Django `draft.py:54-95`)
/// - `cycle_id`: first active `DraftIssueCycle` (deleted_at NULL).
/// - `label_ids` / `assignee_ids` / `module_ids`: list of active IDs,
///   filtering `deleted_at IS NULL` on each intermediate table (same
///   pragmatic criterion as `load_enrichment` in `issue_pagination.rs:122`
///   for regular issues, which already omits the `member_project__is_active`
///   check from Django for simplicity of parity between endpoints).
#[derive(Debug, Serialize)]
pub struct DraftIssueResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub description_html: String,
    pub priority: String,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub estimate_point: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sort_order: f64,
    pub cycle_id: Option<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Always `true` — discriminator used by the frontend to distinguish
    /// drafts of regular issues (`TWorkspaceDraftIssue.is_draft`).
    pub is_draft: bool,
}

/// Hydrates `DraftIssueResponse` with M2M annotations from the DB.
///
/// One query per relation type (O(1) roundtrips), filtering
/// `deleted_at IS NULL` on each intermediate table. Parity with the
/// annotations of the `WorkspaceDraftIssueViewSet.get_queryset` queryset
/// (draft.py:49-95).
///
/// Returns `Vec<DraftIssueResponse>` in the same order as `models`.
async fn hydrate_draft_issue_responses(
    db: &sea_orm::DatabaseConnection,
    models: Vec<draft_issues::Model>,
) -> Result<Vec<DraftIssueResponse>, AppError> {
    if models.is_empty() {
        return Ok(Vec::new());
    }

    use crate::entities::{
        draft_issue_assignees, draft_issue_cycles, draft_issue_labels, draft_issue_modules,
    };
    use std::collections::HashMap;

    let ids: Vec<Uuid> = models.iter().map(|m| m.id).collect();

    // cycle_id (first active cycle — Subquery `[:1]` in Django draft.py:55-58).
    let cycles_rows = draft_issue_cycles::Entity::find()
        .filter(draft_issue_cycles::Column::DraftIssueId.is_in(ids.clone()))
        .filter(draft_issue_cycles::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut cycle_by_draft: HashMap<Uuid, Uuid> = HashMap::new();
    for r in cycles_rows {
        // `entry().or_insert()` preserves the first value seen — mirror
        // of Django's Subquery `[:1]`.
        cycle_by_draft.entry(r.draft_issue_id).or_insert(r.cycle_id);
    }

    // label_ids
    let labels_rows = draft_issue_labels::Entity::find()
        .filter(draft_issue_labels::Column::DraftIssueId.is_in(ids.clone()))
        .filter(draft_issue_labels::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut labels_by_draft: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for r in labels_rows {
        labels_by_draft
            .entry(r.draft_issue_id)
            .or_default()
            .push(r.label_id);
    }

    // assignee_ids
    let assignees_rows = draft_issue_assignees::Entity::find()
        .filter(draft_issue_assignees::Column::DraftIssueId.is_in(ids.clone()))
        .filter(draft_issue_assignees::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut assignees_by_draft: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for r in assignees_rows {
        assignees_by_draft
            .entry(r.draft_issue_id)
            .or_default()
            .push(r.assignee_id);
    }

    // module_ids
    let modules_rows = draft_issue_modules::Entity::find()
        .filter(draft_issue_modules::Column::DraftIssueId.is_in(ids.clone()))
        .filter(draft_issue_modules::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut modules_by_draft: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for r in modules_rows {
        modules_by_draft
            .entry(r.draft_issue_id)
            .or_default()
            .push(r.module_id);
    }

    let result = models
        .into_iter()
        .map(|m| {
            let id = m.id;
            DraftIssueResponse {
                id,
                name: m.name,
                description_html: m.description_html,
                priority: m.priority,
                state_id: m.state_id,
                parent_id: m.parent_id,
                project_id: m.project_id,
                type_id: m.type_id,
                estimate_point: m.estimate_point_id,
                start_date: m.start_date,
                target_date: m.target_date,
                completed_at: m.completed_at.map(Into::into),
                sort_order: m.sort_order,
                cycle_id: cycle_by_draft.get(&id).copied(),
                label_ids: labels_by_draft.remove(&id).unwrap_or_default(),
                assignee_ids: assignees_by_draft.remove(&id).unwrap_or_default(),
                module_ids: modules_by_draft.remove(&id).unwrap_or_default(),
                created_by: m.created_by_id,
                updated_by: m.updated_by_id,
                created_at: m.created_at.into(),
                updated_at: m.updated_at.into(),
                is_draft: true,
            }
        })
        .collect();

    Ok(result)
}

/// Convenience to hydrate a single draft (used by `create`/`get`).
async fn hydrate_draft_issue_response(
    db: &sea_orm::DatabaseConnection,
    model: draft_issues::Model,
) -> Result<DraftIssueResponse, AppError> {
    let mut responses = hydrate_draft_issue_responses(db, vec![model]).await?;
    // `hydrate_draft_issue_responses` preserves order and never empties the
    // vec when input has elements — `pop` is safe.
    Ok(responses
        .pop()
        .expect("hydrate_draft_issue_responses preserves the input vec"))
}

/// `POST /workspaces/{slug}/draft-issues/` shape.
///
/// Exact parity with `DraftIssueCreateSerializer` (apps/api/plane/app/
/// serializers/draft.py:33) and with the payload the frontend builds
/// from `DEFAULT_WORK_ITEM_FORM_VALUES` (packages/constants/src/issue/
/// modal.ts).
///
/// # Critical Conventions
/// - All `Option<Uuid>` / `Option<NaiveDate>` use the helpers in
///   `crate::utils::serde_empty` to convert `""` → `None`. The frontend
///   sends `project_id: ""`, `state_id: ""`, empty dates by default; native
///   serde fails with 422 (HTTP Unprocessable Entity) when encountering `""`
///   where it expects a `Uuid`.
/// - `estimate_point` without `_id` — DRF exposes the FK with that name when
///   declared as `PrimaryKeyRelatedField(source="estimate_point", ...)`.
///   The DB column is `estimate_point_id`; the mapping is done by the
///   handler (mirror of `CreateIssueRequest` in `issues.rs:163`).
/// - `assignee_ids` / `label_ids` (plural + suffix) are the names used by
///   DRF and the frontend; they are `ListField(required=False)` in Django.
/// - `cycle_id` and `module_ids` exist in the frontend payload even though
///   `DraftIssueCreateSerializer` reads them from `initial_data`
///   (draft.py:146-147) — here we declare them explicitly.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateDraftIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub state_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub project_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub type_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub estimate_point: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub target_date: Option<chrono::NaiveDate>,
    pub sort_order: Option<f64>,
    // Tolerant to `[null]` / `[""]` — same pattern as in `CreateIssueRequest`,
    // the frontend (react-hook-form) sometimes initializes these arrays with
    // null placeholders when mounting controlled selects in "unassigned" state.
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub label_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub cycle_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub module_ids: Option<Vec<Uuid>>,
}

/// `PATCH /workspaces/{slug}/draft-issues/{pk}/` shape.
///
/// Parity with `DraftIssueCreateSerializer(partial=True)` (draft.py:170-178).
/// Same conventions as `CreateDraftIssueRequest`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateDraftIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub state_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub project_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub type_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub estimate_point: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub target_date: Option<chrono::NaiveDate>,
    pub sort_order: Option<f64>,
    // `deserialize_uuid_list_filter_nulls` — tolerates `[null]` and `[""]` from
    // the frontend (see `serde_empty::deserialize_uuid_list_filter_nulls`).
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub label_ids: Option<Vec<Uuid>>,
    /// `cycle_id` in PATCH uses Option<Option<Uuid>> to distinguish:
    /// - field absent                          → do not touch cycle (`"not_provided"` in Django)
    /// - `cycle_id: null` or `cycle_id: ""`    → unassign cycle
    /// - `cycle_id: "<uuid>"`                  → assign cycle
    ///
    /// Django reads `request.data.get("cycle_id", "not_provided")` (draft.py:176)
    /// and in the serializer `if cycle_id != "not_provided"` decides whether to touch it
    /// (draft.py:266). We replicate that behavior using the
    /// `Option<Option<T>>` + `#[serde(default, with = "::serde_with::rust::double_option")]`
    /// trick — but without depending on `serde_with`, we implement the same double-wrap
    /// manually: `#[serde(default, deserialize_with = ...)]`.
    #[serde(default, deserialize_with = "deserialize_double_option_uuid")]
    pub cycle_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub module_ids: Option<Vec<Uuid>>,
}

/// Deserializer for `Option<Option<Uuid>>` with empty-string-as-none.
///
/// Semantics:
/// - field absent                 → `None`            (do not touch)
/// - `null` or `""`               → `Some(None)`      (unassign)
/// - `"<uuid>"`                  → `Some(Some(uuid))` (assign)
///
/// Needed for PATCH where we want to distinguish "field not sent" from
/// "field sent as null/empty". Native Serde collapses both to `None`
/// with `Option<Uuid>`.
fn deserialize_double_option_uuid<'de, D>(
    deserializer: D,
) -> Result<Option<Option<Uuid>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(Some(None)),
        Some(s) if s.trim().is_empty() => Ok(Some(None)),
        Some(s) => Uuid::parse_str(&s)
            .map(|u| Some(Some(u)))
            .map_err(serde::de::Error::custom),
    }
}

/// Query params for `GET /workspaces/{slug}/draft-issues/`.
///
/// Parity with Django `self.paginate(...)` in
/// `WorkspaceDraftIssueViewSet.list` (draft.py:99-109), which accepts `cursor`
/// and `per_page` via querystring. The frontend destructures the response as
/// `{ results, ...paginationInfo }` in `issue.store.ts:231` — if we don't wrap
/// the array the UI cannot add anything to `issuesMap` and the panel remains empty.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DraftIssueListQuery {
    /// Django Cursor: `"per_page:offset:is_prev"` (e.g. `"20:0:0"`).
    pub cursor: Option<String>,
    /// Explicit per_page override (priority over cursor-derived,
    /// same order as Django in `BasePaginator.paginate`).
    pub per_page: Option<u64>,
}

/// GET /workspaces/{slug}/draft-issues/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/draft-issues/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "User draft issues page"),
    )
)]
pub async fn list_draft_issues(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Query(q): Query<DraftIssueListQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // Parity with Django: `paginate(default_per_page=100)` inherited from
    // `BasePaginator`. Maximum is respected from `pagination::resolve_per_page`.
    const DEFAULT_PER_PAGE: u64 = 100;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // Filters identical to Django queryset draft.py:49-101: workspace by
    // slug, `created_by=request.user`, active soft-delete.
    let base = draft_issues::Entity::find()
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::CreatedById.eq(user_id))
        .filter(draft_issues::Column::DeletedAt.is_null());

    // `count` and `page` use the SAME filter — we clone before adding order
    // to not diverge (same pattern as `list_stickies`).
    let total_count = base.clone().count(db).await.map_err(AppError::Database)?;

    let page = base
        .order_by_desc(draft_issues::Column::CreatedAt)
        .paginate(db, limit)
        .fetch_page(cursor.offset)
        .await
        .map_err(AppError::Database)?;

    let results = hydrate_draft_issue_responses(db, page).await?;
    let body = pagination::build_response(results, total_count, limit, cursor.offset);

    Ok((StatusCode::OK, Json(body)))
}

/// POST /workspaces/{slug}/draft-issues/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/draft-issues/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Draft issue created"),
    )
)]
pub async fn create_draft_issue(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateDraftIssueRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    // Draft issues: GUEST is allowed in Django (draft.py:111); we don't use
    // `require_member_or_admin` here to maintain parity.
    let _ = member;

    // Parity validation with DraftIssueCreateSerializer.validate (draft.py:72-77):
    // start_date > target_date → error.
    if let (Some(start), Some(target)) = (body.start_date, body.target_date) {
        if start > target {
            return Err(AppError::BadRequest(
                "Start date cannot exceed target date".into(),
            ));
        }
    }

    let workspace_id = ws.id;
    let project_id = body.project_id;
    let assignees = body.assignee_ids.clone().unwrap_or_default();
    let labels = body.label_ids.clone().unwrap_or_default();
    let cycle_id = body.cycle_id;
    let modules = body.module_ids.clone().unwrap_or_default();

    // Execute in transaction so draft + its N-to-M remain atomic,
    // same as `DraftIssueCreateSerializer.create` which does bulk_create
    // within the same request and rollback if something fails.
    use sea_orm::TransactionTrait;
    let saved = db
        .transaction::<_, draft_issues::Model, AppError>(|txn| {
            Box::pin(async move {
                let new = draft_issues::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    name: Set(body.name),
                    description_json: Set(serde_json::json!({})),
                    description_html: Set(body.description_html.unwrap_or_default()),
                    description_stripped: Set(None),
                    description_binary: Set(None),
                    priority: Set(body.priority.unwrap_or_else(|| "none".to_string())),
                    state_id: Set(body.state_id),
                    parent_id: Set(body.parent_id),
                    project_id: Set(project_id),
                    workspace_id: Set(workspace_id),
                    type_id: Set(body.type_id),
                    // `estimate_point` from DRF payload → `estimate_point_id` column.
                    estimate_point_id: Set(body.estimate_point),
                    start_date: Set(body.start_date),
                    target_date: Set(body.target_date),
                    // Django default (draft.py DraftIssue model): sort_order=65535.0
                    // if not provided; exact parity.
                    sort_order: Set(body.sort_order.unwrap_or(65535.0)),
                    completed_at: Set(None),
                    external_source: Set(None),
                    external_id: Set(None),
                    created_by_id: Set(Some(user_id)),
                    updated_by_id: Set(Some(user_id)),
                    created_at: Set(chrono::Utc::now().into()),
                    updated_at: Set(chrono::Utc::now().into()),
                    deleted_at: Set(None),
                };
                let saved = new.insert(txn).await.map_err(AppError::Database)?;

                sync_draft_assignees(
                    txn,
                    saved.id,
                    project_id,
                    workspace_id,
                    user_id,
                    &assignees,
                )
                .await?;
                sync_draft_labels(
                    txn,
                    saved.id,
                    project_id,
                    workspace_id,
                    user_id,
                    &labels,
                )
                .await?;
                if let Some(cid) = cycle_id {
                    sync_draft_cycle(
                        txn,
                        saved.id,
                        project_id,
                        workspace_id,
                        user_id,
                        Some(cid),
                    )
                    .await?;
                }
                sync_draft_modules(
                    txn,
                    saved.id,
                    project_id,
                    workspace_id,
                    user_id,
                    &modules,
                )
                .await?;

                Ok(saved)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    // Hydrate with M2M annotations. Parity with Django draft.py:124-151,
    // which after `serializer.save()` re-queries the annotated queryset and
    // returns the full shape (with cycle_id/label_ids/assignee_ids/
    // module_ids), not just the raw model.
    let resp = hydrate_draft_issue_response(db, saved).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /workspaces/{slug}/draft-issues/{pk}/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/draft-issues/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Draft Issue ID"),
    ),
    responses(
        (status = 200, description = "Draft issue detail"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_draft_issue(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let issue = draft_issues::Entity::find_by_id(pk)
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::CreatedById.eq(user_id))
        .filter(draft_issues::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let resp = hydrate_draft_issue_response(db, issue).await?;
    Ok((StatusCode::OK, Json(resp)))
}

/// PATCH /workspaces/{slug}/draft-issues/{pk}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/draft-issues/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Draft Issue ID"),
    ),
    responses(
        (status = 200, description = "Draft updated"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_draft_issue(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateDraftIssueRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    require_member_or_admin(member.role)?;

    let issue = draft_issues::Entity::find_by_id(pk)
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::CreatedById.eq(user_id))
        .filter(draft_issues::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Parity validation with DraftIssueCreateSerializer.validate (draft.py:72-77).
    // In PATCH we compare the effective pair: field sent if present, otherwise
    // the current draft value.
    let effective_start = body.start_date.or(issue.start_date);
    let effective_target = body.target_date.or(issue.target_date);
    if let (Some(start), Some(target)) = (effective_start, effective_target) {
        if start > target {
            return Err(AppError::BadRequest(
                "Start date cannot exceed target date".into(),
            ));
        }
    }

    // Effective `project_id` to sync N-to-Ms (Django takes it from
    // the request if present, otherwise from the draft — draft.py:168).
    let effective_project_id = body.project_id.or(issue.project_id);
    let workspace_id = ws.id;
    let issue_id = issue.id;

    use sea_orm::TransactionTrait;
    db.transaction::<_, (), AppError>(|txn| {
        // Build ActiveModel sync — mirror of issues.rs:update_issue pattern
        // (avoids having to move full `body` to async and deal with partial
        // borrows).
        let mut active: draft_issues::ActiveModel = issue.into();
        if let Some(v) = body.name.clone() {
            active.name = Set(Some(v));
        }
        if let Some(v) = body.description_html.clone() {
            active.description_html = Set(v);
        }
        if let Some(v) = body.priority.clone() {
            active.priority = Set(v);
        }
        // For simple UUID fields: `deserialize_empty_as_none_uuid`
        // already collapses `""` → `None`, so `is_some()` detects only the
        // case of valid UUID present. Parity with issues.rs:update_issue.
        if body.state_id.is_some() {
            active.state_id = Set(body.state_id);
        }
        if body.parent_id.is_some() {
            active.parent_id = Set(body.parent_id);
        }
        if body.project_id.is_some() {
            active.project_id = Set(body.project_id);
        }
        if body.type_id.is_some() {
            active.type_id = Set(body.type_id);
        }
        if body.estimate_point.is_some() {
            active.estimate_point_id = Set(body.estimate_point);
        }
        if body.start_date.is_some() {
            active.start_date = Set(body.start_date);
        }
        if body.target_date.is_some() {
            active.target_date = Set(body.target_date);
        }
        if let Some(v) = body.sort_order {
            active.sort_order = Set(v);
        }
        active.updated_at = Set(chrono::Utc::now().into());
        active.updated_by_id = Set(Some(user_id));

        // Extract N-to-Ms for the async block.
        let assignees = body.assignee_ids.clone();
        let labels = body.label_ids.clone();
        let cycle_id = body.cycle_id;
        let modules = body.module_ids.clone();

        Box::pin(async move {
            active.update(txn).await.map_err(AppError::Database)?;

            // Sync N-to-M only if field was in payload (parity with
            // `if assignees is not None` in draft.py:232,249).
            if let Some(ref a) = assignees {
                sync_draft_assignees(
                    txn,
                    issue_id,
                    effective_project_id,
                    workspace_id,
                    user_id,
                    a,
                )
                .await?;
            }
            if let Some(ref l) = labels {
                sync_draft_labels(
                    txn,
                    issue_id,
                    effective_project_id,
                    workspace_id,
                    user_id,
                    l,
                )
                .await?;
            }
            // cycle_id: Some(Some(uuid))=assign, Some(None)=unassign,
            // None=do not touch. Parity with `if cycle_id != "not_provided"` in
            // draft.py:266.
            if let Some(cid) = cycle_id {
                sync_draft_cycle(
                    txn,
                    issue_id,
                    effective_project_id,
                    workspace_id,
                    user_id,
                    cid,
                )
                .await?;
            }
            if let Some(ref m) = modules {
                sync_draft_modules(
                    txn,
                    issue_id,
                    effective_project_id,
                    workspace_id,
                    user_id,
                    m,
                )
                .await?;
            }

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        sea_orm::TransactionError::Transaction(app_err) => app_err,
        sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
    })?;

    // Re-read updated draft to return canonical shape
    // (M2M relations synced in transaction are already in DB).
    let updated = draft_issues::Entity::find_by_id(pk)
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;
    let resp = hydrate_draft_issue_response(db, updated).await?;
    Ok((StatusCode::OK, Json(resp)))
}

/// DELETE /workspaces/{slug}/draft-issues/{pk}/
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/draft-issues/{pk}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Draft Issue ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_draft_issue(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let issue = draft_issues::Entity::find_by_id(pk)
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: draft_issues::ActiveModel = issue.into();
    active.deleted_at = Set(Some(chrono::Utc::now().into()));
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE-LEVEL AGGREGATE VIEWS
// ═══════════════════════════════════════════════════════════════════════════
//
// Equivalents to: workspace/cycle.py, module.py, estimate.py, label.py, state.py
// These are read-only views listing entities across all projects in the
// workspace that the user belongs to.

// ── Cycles ──────────────────────────────────────────────────────────────────

// WorkspaceCycleResponse removed — reuse cycles::CycleResponse for full ICycle contract.

/// GET /workspaces/{slug}/cycles/
///
/// Lists all non-archived cycles in the workspace.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/cycles/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of workspace cycles"),
    )
)]
pub async fn list_workspace_cycles(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let items = cycles::Entity::find()
        .filter(cycles::Column::WorkspaceId.eq(ws.id))
        .filter(cycles::Column::ArchivedAt.is_null())
        .filter(cycles::Column::DeletedAt.is_null())
        .order_by_desc(cycles::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<super::cycles::CycleResponse> =
        items.into_iter().map(super::cycles::CycleResponse::from_model).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ── Modules ──────────────────────────────────────────────────────────────────

// WorkspaceModuleResponse removed — reuse modules::ModuleResponse for full IModule contract.

/// GET /workspaces/{slug}/modules/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/modules/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of workspace modules"),
    )
)]
pub async fn list_workspace_modules(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let items = modules::Entity::find()
        .filter(modules::Column::WorkspaceId.eq(ws.id))
        .filter(modules::Column::ArchivedAt.is_null())
        .filter(modules::Column::DeletedAt.is_null())
        .order_by_desc(modules::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Batch-load favorites across the entire workspace for this user
    use crate::entities::user_favorites;
    let fav_ids: std::collections::HashSet<uuid::Uuid> = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::EntityType.eq("module"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    use crate::entities::module_members;
    use sea_orm::{ColumnTrait as _, EntityTrait as _, QueryFilter as _, QuerySelect as _};
    let member_rows = module_members::Entity::find()
        .select_only()
        .column(module_members::Column::ModuleId)
        .column(module_members::Column::MemberId)
        .filter(module_members::Column::WorkspaceId.eq(ws.id))
        .filter(module_members::Column::DeletedAt.is_null())
        .into_tuple::<(uuid::Uuid, uuid::Uuid)>()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut members_map: std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>> =
        std::collections::HashMap::new();
    for (mod_id, member_id) in member_rows {
        members_map.entry(mod_id).or_default().push(member_id);
    }

    let resp: Vec<super::modules::ModuleResponse> = items.into_iter().map(|m| {
        let is_favorite = fav_ids.contains(&m.id);
        let member_ids = members_map.remove(&m.id).unwrap_or_default();
        let mut r = super::modules::ModuleResponse::from_model(m);
        r.is_favorite = is_favorite;
        r.member_ids = member_ids;
        r
    }).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ── Estimates ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorkspaceEstimatePointResponse {
    pub id: Uuid,
    pub key: i32,
    pub value: String,
    pub description: String,
}

impl From<estimate_points::Model> for WorkspaceEstimatePointResponse {
    fn from(m: estimate_points::Model) -> Self {
        Self {
            id: m.id,
            key: m.key,
            value: m.value,
            description: m.description,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceEstimateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub points: Vec<WorkspaceEstimatePointResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GET /workspaces/{slug}/estimates/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/estimates/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of workspace estimates"),
    )
)]
pub async fn list_workspace_estimates(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // Load estimates + their points with two queries (no ORM join with SeaORM)
    let ests = estimates::Entity::find()
        .filter(estimates::Column::WorkspaceId.eq(ws.id))
        .filter(estimates::Column::DeletedAt.is_null())
        .order_by_desc(estimates::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut resp = Vec::with_capacity(ests.len());
    for est in ests {
        let points = estimate_points::Entity::find()
            .filter(estimate_points::Column::EstimateId.eq(est.id))
            .filter(estimate_points::Column::DeletedAt.is_null())
            .order_by_asc(estimate_points::Column::Key)
            .all(db)
            .await
            .map_err(AppError::Database)?;

        resp.push(WorkspaceEstimateResponse {
            id: est.id,
            name: est.name,
            description: est.description,
            r#type: est.r#type,
            project_id: est.project_id,
            workspace_id: est.workspace_id,
            points: points.into_iter().map(Into::into).collect(),
            created_at: est.created_at.into(),
            updated_at: est.updated_at.into(),
        });
    }

    Ok((StatusCode::OK, Json(resp)))
}

// ── Labels ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorkspaceLabelResponse {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub description: String,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<labels::Model> for WorkspaceLabelResponse {
    fn from(m: labels::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            color: m.color,
            description: m.description,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            parent_id: m.parent_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

/// GET /workspaces/{slug}/labels/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/labels/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of workspace labels"),
    )
)]
pub async fn list_workspace_labels(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let items = labels::Entity::find()
        .filter(labels::Column::WorkspaceId.eq(ws.id))
        .filter(labels::Column::DeletedAt.is_null())
        .order_by_asc(labels::Column::Name)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<WorkspaceLabelResponse> = items.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ── States ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorkspaceStateResponse {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub group: String,
    pub description: String,
    pub sequence: f64,
    pub is_default: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<states::Model> for WorkspaceStateResponse {
    fn from(m: states::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            color: m.color,
            group: m.group,
            description: m.description,
            sequence: m.sequence,
            is_default: m.default,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

/// GET /workspaces/{slug}/states/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/states/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of workspace states"),
    )
)]
pub async fn list_workspace_states(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let items = states::Entity::find()
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .filter(states::Column::DeletedAt.is_null())
        .order_by_asc(states::Column::Sequence)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<WorkspaceStateResponse> = items.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE USER PROPERTIES
// ═══════════════════════════════════════════════════════════════════════════
//
// Mirror of Django `WorkspaceUserPropertiesEndpoint`
// (plane/app/views/workspace/user.py:252).
//
// GET   /workspaces/{slug}/user-properties/
// PATCH /workspaces/{slug}/user-properties/
//
// Both operate under `get_or_create(user, workspace)` semantics: if the row
// doesn't exist it's created with defaults. In Rust it's implemented
// atomically with `INSERT ... ON CONFLICT DO NOTHING` to avoid race
// conditions between tabs of the same user, followed by a SELECT.
//
// Permission: `WorkspaceViewerPermission` (any active workspace member).
//
// Security and intentional differences from Django:
// - PATCH body uses a strict whitelist; `id`, `user_id`, `workspace_id`,
//   timestamps and `deleted_at` cannot be overridden from the request.
// - `navigation_control_preference` is validated against the Django enum
//   (`ACCORDION` | `TABBED`). Django delegates validation to the CharField
//   + choices at serializer level; here it's made explicit.
// - `navigation_project_limit` is validated in range `0..=1000` to avoid
//   absurd values / `i32` overflow. Django does not validate; this
//   hardening is consistent with the policy of not introducing unsafe
//   patterns.

/// Allowed values for `navigation_control_preference`.
///
/// Mirror of Django enum
/// `WorkspaceUserProperties.NavigationControlPreference.choices`.
const NAVIGATION_CONTROL_PREFERENCES: &[&str] = &["ACCORDION", "TABBED"];

/// Reasonable limit for `navigation_project_limit`. Django does not validate;
/// here it's bounded to avoid hostile or absurd values.
const MAX_NAVIGATION_PROJECT_LIMIT: i32 = 1000;

/// Defaults identical to Django `get_default_filters` in
/// `plane/db/models/workspace.py:62`.
fn default_filters() -> JsonValue {
    serde_json::json!({
        "priority": null,
        "state": null,
        "state_group": null,
        "assignees": null,
        "created_by": null,
        "labels": null,
        "start_date": null,
        "target_date": null,
        "subscriber": null,
    })
}

/// Mirror of `get_default_display_filters`
/// (plane/db/models/workspace.py:76).
fn default_display_filters() -> JsonValue {
    serde_json::json!({
        "display_filters": {
            "group_by": null,
            "order_by": "-created_at",
            "type": null,
            "sub_issue": true,
            "show_empty_groups": true,
            "layout": "list",
            "calendar_date_range": "",
        }
    })
}

/// Mirror of `get_default_display_properties`
/// (plane/db/models/workspace.py:90).
fn default_display_properties() -> JsonValue {
    serde_json::json!({
        "display_properties": {
            "assignee": true,
            "attachment_count": true,
            "created_on": true,
            "due_date": true,
            "estimate": true,
            "key": true,
            "labels": true,
            "link": true,
            "priority": true,
            "start_date": true,
            "state": true,
            "sub_issue_count": true,
            "updated_on": true,
        }
    })
}

#[derive(Debug, Serialize)]
pub struct WorkspaceUserPropertiesResponse {
    pub id: Uuid,
    pub workspace: Uuid,
    pub user: Uuid,
    pub filters: JsonValue,
    pub display_filters: JsonValue,
    pub display_properties: JsonValue,
    pub rich_filters: JsonValue,
    pub navigation_project_limit: i32,
    pub navigation_control_preference: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<workspace_user_properties::Model> for WorkspaceUserPropertiesResponse {
    fn from(m: workspace_user_properties::Model) -> Self {
        Self {
            id: m.id,
            // Django `WorkspaceUserPropertiesSerializer` uses `fields = "__all__"`
            // with `read_only_fields = ["workspace", "user"]`. In the JSON
            // response these FKs appear as `workspace` and `user` (not as
            // `workspace_id`/`user_id`). We preserve this contract for the
            // frontend.
            workspace: m.workspace_id,
            user: m.user_id,
            filters: m.filters,
            display_filters: m.display_filters,
            display_properties: m.display_properties,
            rich_filters: m.rich_filters,
            navigation_project_limit: m.navigation_project_limit,
            navigation_control_preference: m.navigation_control_preference,
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
            deleted_at: m.deleted_at.map(Into::into),
        }
    }
}

/// Body accepted by PATCH. All fields are optional (partial update, same
/// as Django `partial=True`). FKs and timestamps are left out of the
/// whitelist by design.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkspaceUserPropertiesRequest {
    pub filters: Option<JsonValue>,
    pub display_filters: Option<JsonValue>,
    pub display_properties: Option<JsonValue>,
    pub rich_filters: Option<JsonValue>,
    pub navigation_project_limit: Option<i32>,
    pub navigation_control_preference: Option<String>,
}

/// Returns existing row or creates it with defaults.
///
/// Implements `get_or_create(user=..., workspace=...)` atomically:
/// 1. `INSERT ... ON CONFLICT DO NOTHING` with Django defaults.
/// 2. `SELECT` filtering by active `(workspace_id, user_id)`.
///
/// The partial unique constraint
/// `workspace_user_properties_unique_workspace_user_when_deleted_at_null`
/// (see `plane/db/models/workspace.py:338-344`) ensures there aren't
/// two active rows for the same (workspace, user) pair.
async fn get_or_create_workspace_user_properties(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<workspace_user_properties::Model, AppError> {
    use sea_orm::sea_query::OnConflict;

    let now = chrono::Utc::now().fixed_offset();

    // Insert attempt with ON CONFLICT DO NOTHING.
    // If an active row already exists for (workspace_id, user_id) the
    // partial unique constraint makes the insert fail silently.
    let to_insert = workspace_user_properties::ActiveModel {
        id: Set(Uuid::new_v4()),
        created_at: Set(now),
        updated_at: Set(now),
        filters: Set(default_filters()),
        display_filters: Set(default_display_filters()),
        display_properties: Set(default_display_properties()),
        rich_filters: Set(serde_json::json!({})),
        navigation_project_limit: Set(10),
        navigation_control_preference: Set("ACCORDION".to_string()),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        user_id: Set(user_id),
        workspace_id: Set(workspace_id),
        deleted_at: Set(None),
    };

    workspace_user_properties::Entity::insert(to_insert)
        .on_conflict(
            OnConflict::columns([
                workspace_user_properties::Column::WorkspaceId,
                workspace_user_properties::Column::UserId,
            ])
            // PostgreSQL REQUIREMENT, not decoration: when the arbiter index is partial
            // (`UNIQUE (workspace_id, user_id) WHERE deleted_at IS NULL`, see
            // WorkspaceUserProperties.Meta.constraints in plane/db/models/workspace.py:338-344),
            // the `ON CONFLICT` MUST repeat the same predicate or PostgreSQL rejects with
            // `there is no unique or exclusion constraint matching the ON CONFLICT specification`
            // (see https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT
            // → "If an index_predicate is specified, it must, as a further requirement for
            //    inference, satisfy arbiter indexes"). The other existing unique
            // (`unique_together = [workspace, user, deleted_at]`) also doesn't match because
            // it includes `deleted_at` as a third column and we only target two.
            .target_and_where(
                sea_orm::sea_query::Expr::col(
                    workspace_user_properties::Column::DeletedAt,
                )
                .is_null(),
            )
            .do_nothing()
            .to_owned(),
        )
        .do_nothing()
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    // Guaranteed SELECT: either the newly inserted row or the existing one.
    workspace_user_properties::Entity::find()
        .filter(workspace_user_properties::Column::WorkspaceId.eq(workspace_id))
        .filter(workspace_user_properties::Column::UserId.eq(user_id))
        .filter(workspace_user_properties::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

/// GET /workspaces/{slug}/user-properties/
///
/// Mirror Django: `WorkspaceUserPropertiesEndpoint.get`
/// (plane/app/views/workspace/user.py:269).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-properties/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "User properties in the workspace"),
        (status = 403, description = "Not an active workspace member"),
        (status = 404, description = "Workspace does not exist"),
    )
)]
pub async fn get_workspace_user_properties(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let props = get_or_create_workspace_user_properties(db, ws.id, user_id).await?;
    let resp: WorkspaceUserPropertiesResponse = props.into();
    Ok((StatusCode::OK, Json(resp)))
}

/// PATCH /workspaces/{slug}/user-properties/
///
/// Mirror Django: `WorkspaceUserPropertiesEndpoint.patch`
/// (plane/app/views/workspace/user.py:255).
///
/// Update policy:
/// - Strict whitelist (FKs/timestamps/id never touched from body).
/// - `navigation_control_preference` ∈ {"ACCORDION", "TABBED"}.
/// - `navigation_project_limit` ∈ [0, MAX_NAVIGATION_PROJECT_LIMIT].
/// - JSON fields (`filters`, `display_filters`, `display_properties`,
///   `rich_filters`) are replaced by value, respecting the `partial=True`
///   contract of the Django serializer.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/user-properties/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Properties updated"),
        (status = 400, description = "Invalid body"),
        (status = 403, description = "Not an active workspace member"),
        (status = 404, description = "Workspace does not exist"),
    )
)]
pub async fn update_workspace_user_properties(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<UpdateWorkspaceUserPropertiesRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    // Validations before touching DB: fail fast and don't consume writes.
    if let Some(ref pref) = body.navigation_control_preference {
        if !NAVIGATION_CONTROL_PREFERENCES.contains(&pref.as_str()) {
            return Err(AppError::BadRequest(format!(
                "navigation_control_preference must be one of {:?}",
                NAVIGATION_CONTROL_PREFERENCES
            )));
        }
    }
    if let Some(limit) = body.navigation_project_limit {
        if !(0..=MAX_NAVIGATION_PROJECT_LIMIT).contains(&limit) {
            return Err(AppError::BadRequest(format!(
                "navigation_project_limit must be between 0 and {}",
                MAX_NAVIGATION_PROJECT_LIMIT
            )));
        }
    }

    let existing = get_or_create_workspace_user_properties(db, ws.id, user_id).await?;

    let mut active: workspace_user_properties::ActiveModel = existing.into();
    if let Some(v) = body.filters {
        active.filters = Set(v);
    }
    if let Some(v) = body.display_filters {
        active.display_filters = Set(v);
    }
    if let Some(v) = body.display_properties {
        active.display_properties = Set(v);
    }
    if let Some(v) = body.rich_filters {
        active.rich_filters = Set(v);
    }
    if let Some(v) = body.navigation_project_limit {
        active.navigation_project_limit = Set(v);
    }
    if let Some(v) = body.navigation_control_preference {
        active.navigation_control_preference = Set(v);
    }
    active.updated_at = Set(chrono::Utc::now().fixed_offset());
    active.updated_by_id = Set(Some(user_id));

    let saved = active.update(db).await.map_err(AppError::Database)?;
    let resp: WorkspaceUserPropertiesResponse = saved.into();
    Ok((StatusCode::OK, Json(resp)))
}


// ═══════════════════════════════════════════════════════════════════════════
// DRAFT TO ISSUE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DraftToIssueRequest {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    pub state_id: Option<uuid::Uuid>,
    pub parent_id: Option<uuid::Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub estimate_point: Option<uuid::Uuid>,
    pub assignee_ids: Option<Vec<uuid::Uuid>>,
    pub label_ids: Option<Vec<uuid::Uuid>>,
    pub cycle_id: Option<uuid::Uuid>,
    pub module_ids: Option<Vec<uuid::Uuid>>,
    pub type_id: Option<uuid::Uuid>,
}

/// POST /workspaces/{slug}/draft-to-issue/{draft_id}/
///
/// Converts a DraftIssue into a real Issue and deletes the draft.
///
/// Mirror of `WorkspaceDraftIssueViewSet.create_draft_to_issue`
/// (`apps/api/plane/app/views/workspace/draft.py:196`).
///
/// Implements:
/// - project_id validation in the draft
/// - Issue creation with atomic sequence_id (SERIALIZABLE)
/// - CycleIssue creation if cycle_id is in body
/// - ModuleIssue(s) creation for each module_id in body
/// - Reassigning FileAssets from draft to the new issue
/// - Draft soft-delete
///
/// Note: activity job (issue_activity) is Phase 3 and is omitted here.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/draft-to-issue/{draft_id}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("draft_id" = uuid::Uuid, Path, description = "Draft issue ID"),
    ),
    responses(
        (status = 201, description = "Issue created"),
        (status = 400, description = "Draft without assigned project"),
        (status = 404, description = "Draft not found"),
    )
)]
pub async fn draft_to_issue(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, draft_id)): Path<(String, uuid::Uuid)>,
    Json(body): Json<DraftToIssueRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let member = require_workspace_member(db, ws.id, user_id).await?;
    require_member_or_admin(member.role)?;

    // 1. Get draft
    let draft = draft_issues::Entity::find_by_id(draft_id)
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::CreatedById.eq(user_id))
        .filter(draft_issues::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // 2. Validate that draft has a project
    let project_id = draft.project_id.ok_or_else(|| {
        AppError::BadRequest("Project is required to create an issue.".into())
    })?;

    // 3. Verify project membership
    // Verify user is an active project member before creating the issue.
    // Returns 403 if no access — reflects Django ProjectViewSet permission check.
    crate::routes::helpers::project_member_for_user(db, project_id, user_id)
        .await
        .map_err(|_| AppError::Forbidden)?
        .ok_or(AppError::Forbidden)?;

    let project = projects::Entity::find_by_id(project_id)
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let assignee_ids = body.assignee_ids.clone().unwrap_or_default();
    let label_ids = body.label_ids.clone().unwrap_or_default();
    let cycle_id = body.cycle_id;
    let module_ids = body.module_ids.clone().unwrap_or_default();

    // 4. Create Issue in SERIALIZABLE transaction (atomic sequence_id)
    let issue = db
        .transaction_with_config::<_, issues::Model, AppError>(
            |txn| {
                let name = body.name.clone();
                let description_html = body.description_html.clone()
                    .unwrap_or_else(|| draft.description_html.clone());
                let priority = body.priority.clone()
                    .unwrap_or_else(|| draft.priority.clone());
                let state_id = body.state_id.or(draft.state_id);
                let parent_id = body.parent_id.or(draft.parent_id);
                let start_date = body.start_date.or(draft.start_date);
                let target_date = body.target_date.or(draft.target_date);
                let estimate_point_id = body.estimate_point.or(draft.estimate_point_id);
                let type_id = body.type_id.or(draft.type_id);
                let assignee_ids = assignee_ids.clone();
                let label_ids = label_ids.clone();
                let _default_assignee_id = project.default_assignee_id;
                Box::pin(async move {
                    use sea_orm::QuerySelect;
                    let max_seq: Option<i32> = issues::Entity::find()
                        .filter(issues::Column::ProjectId.eq(project_id))
                        .select_only()
                        .column_as(
                            sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
                            "max_seq",
                        )
                        .into_tuple::<Option<i32>>()
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?
                        .flatten();
                    let sequence_id = max_seq.unwrap_or(0) + 1;
                    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

                    let issue_id = uuid::Uuid::new_v4();
                    let new_issue = issues::ActiveModel {
                        id: Set(issue_id),
                        name: Set(name),
                        description_html: Set(description_html),
                        priority: Set(priority),
                        state_id: Set(state_id),
                        parent_id: Set(parent_id),
                        start_date: Set(start_date),
                        target_date: Set(target_date),
                        estimate_point_id: Set(estimate_point_id),
                        type_id: Set(type_id),
                        sequence_id: Set(sequence_id),
                        project_id: Set(project_id),
                        workspace_id: Set(ws.id),
                        created_by_id: Set(Some(user_id)),
                        updated_by_id: Set(Some(user_id)),
                        sort_order: Set(65535.0),
                        created_at: Set(now),
                        updated_at: Set(now),
                        deleted_at: Set(None),
                        ..Default::default()
                    };
                    let saved_issue = new_issue.insert(txn).await.map_err(AppError::Database)?;

                    // Assignees
                    for assignee_id in &assignee_ids {
                        issue_assignees::ActiveModel {
                            id: Set(uuid::Uuid::new_v4()),
                            issue_id: Set(issue_id),
                            assignee_id: Set(*assignee_id),
                            project_id: Set(project_id),
                            workspace_id: Set(ws.id),
                            created_by_id: Set(Some(user_id)),
                            updated_by_id: Set(Some(user_id)),
                            created_at: Set(now),
                            updated_at: Set(now),
                            deleted_at: Set(None),
                        }
                        .insert(txn)
                        .await
                        .map_err(AppError::Database)?;
                    }

                    // Labels
                    for label_id in &label_ids {
                        issue_labels::ActiveModel {
                            id: Set(uuid::Uuid::new_v4()),
                            issue_id: Set(issue_id),
                            label_id: Set(*label_id),
                            project_id: Set(project_id),
                            workspace_id: Set(ws.id),
                            created_by_id: Set(Some(user_id)),
                            updated_by_id: Set(Some(user_id)),
                            created_at: Set(now),
                            updated_at: Set(now),
                            deleted_at: Set(None),
                        }
                        .insert(txn)
                        .await
                        .map_err(AppError::Database)?;
                    }

                    Ok(saved_issue)
                })
            },
            Some(sea_orm::IsolationLevel::Serializable),
            None,
        )
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(ae) => ae,
            sea_orm::TransactionError::Connection(de) => AppError::Database(de),
        })?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let issue_id = issue.id;

    // 5. CycleIssue (best-effort, does not block if it fails)
    if let Some(cid) = cycle_id {
        let _ = cycle_issues::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            cycle_id: Set(cid),
            issue_id: Set(issue_id),
            project_id: Set(project_id),
            workspace_id: Set(ws.id),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(db)
        .await
        .map_err(|e| {
            tracing::warn!("Failed to create CycleIssue during draft_to_issue: {e}");
            AppError::Database(e)
        });
    }

    // 6. ModuleIssues
    for module_id in &module_ids {
        let _ = module_issues::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            issue_id: Set(issue_id),
            module_id: Set(*module_id),
            project_id: Set(project_id),
            workspace_id: Set(ws.id),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(db)
        .await
        .map_err(|e| {
            tracing::warn!("Failed to create ModuleIssue: {e}");
            AppError::Database(e)
        });
    }

    // 7. Reassign FileAssets from draft to real issue
    // Django: file_assets.update(issue_id=..., entity_type=ISSUE_DESCRIPTION, draft_issue_id=None)
    file_assets::Entity::update_many()
        .col_expr(file_assets::Column::IssueId, sea_orm::sea_query::Expr::value(Some(issue_id)))
        .col_expr(file_assets::Column::EntityType, sea_orm::sea_query::Expr::value(Some("issue_description")))
        .col_expr(file_assets::Column::DraftIssueId, sea_orm::sea_query::Expr::value(Option::<uuid::Uuid>::None))
        .filter(file_assets::Column::DraftIssueId.eq(draft_id))
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    // 8. Draft soft-delete
    let mut draft_am: draft_issues::ActiveModel = draft.into();
    draft_am.deleted_at = Set(Some(now));
    draft_am.update(db).await.map_err(AppError::Database)?;

    // Response mirror of Django shape: basic fields of created issue
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": issue_id,
        "name": issue.name,
        "sequence_id": issue.sequence_id,
        "priority": issue.priority,
        "state_id": issue.state_id,
        "project_id": issue.project_id,
        "workspace_id": issue.workspace_id,
        "created_at": issue.created_at,
        "created_by": issue.created_by_id,
    }))))
}

// ── Active Cycles ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ActiveCyclesQuery {
    /// Django-style cursor: "per_page:offset:is_prev"
    pub cursor: Option<String>,
    /// Per-page override
    pub per_page: Option<u64>,
}

/// GET /api/workspaces/{slug}/active-cycles/
///
/// Returns a cursor-paginated list of currently active cycles (status = "started")
/// across all projects in the workspace.
/// Mirrors Django `WorkspaceActiveCycleEndpoint` behaviour.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/active-cycles/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("cursor" = Option<String>, Query, description = "Cursor: per_page:offset:is_prev"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "Paginated active cycles"),
        (status = 403, description = "Not a workspace member"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_workspace_active_cycles(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
    Query(q): Query<ActiveCyclesQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    const DEFAULT_PER_PAGE: u64 = 10;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(Some(cursor.per_page), q.per_page, DEFAULT_PER_PAGE, MAX_PER_PAGE);
    let offset = cursor.offset * limit;

    let now = chrono::Utc::now();

    // Active = start_date <= now AND end_date >= now, not archived, not deleted
    let now_fixed: chrono::DateTime<chrono::FixedOffset> = now.into();

    let total_count = cycles::Entity::find()
        .filter(cycles::Column::WorkspaceId.eq(ws.id))
        .filter(cycles::Column::DeletedAt.is_null())
        .filter(cycles::Column::ArchivedAt.is_null())
        .filter(cycles::Column::StartDate.lte(now_fixed))
        .filter(cycles::Column::EndDate.gte(now_fixed))
        .count(db)
        .await
        .map_err(AppError::Database)?;

    let items = cycles::Entity::find()
        .filter(cycles::Column::WorkspaceId.eq(ws.id))
        .filter(cycles::Column::DeletedAt.is_null())
        .filter(cycles::Column::ArchivedAt.is_null())
        .filter(cycles::Column::StartDate.lte(now_fixed))
        .filter(cycles::Column::EndDate.gte(now_fixed))
        .order_by_desc(cycles::Column::UpdatedAt)
        .limit(limit)
        .offset(offset)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let cycle_responses: Vec<super::cycles::CycleResponse> = items
        .into_iter()
        .map(super::cycles::CycleResponse::from_model)
        .collect();

    let body = pagination::build_response(cycle_responses, total_count, limit, cursor.offset);
    Ok((StatusCode::OK, Json(body)))
}
