// src/routes/workspace_extras.rs
//! Endpoints de Workspace — sub-módulos adicionales.
//!
//! Cubre los equivalentes Django de:
//!   workspace/favorite.py        → favoritos del usuario
//!   workspace/home.py            → preferencias de home
//!   workspace/quick_link.py      → quick links
//!   workspace/recent_visit.py    → visitas recientes
//!   workspace/sticky.py          → stickies
//!   workspace/user_preference.py → preferencias de usuario
//!   workspace/draft.py           → draft issues
//!   workspace/cycle.py           → cycles a nivel workspace
//!   workspace/module.py          → modules a nivel workspace
//!   workspace/estimate.py        → estimates a nivel workspace
//!   workspace/label.py           → labels a nivel workspace
//!   workspace/state.py           → states a nivel workspace

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::{
        cycles, draft_issues, estimates, estimate_points, labels, modules,
        stickies, states, user_favorites, user_recent_visits,
        workspace_home_preferences, workspace_user_links,
        workspace_user_preferences,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    AppState,
};

// ── Permisos inline ────────────────────────────────────────────────────────

/// Asegura que el rol del miembro es al menos MEMBER (no GUEST).
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
/// Lista favoritos del usuario en el workspace (sin padre, sin páginas).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-favorites/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Lista de favoritos"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin acceso"),
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
        (status = 200, description = "Favorito creado o existente"),
        (status = 400, description = "Error de validación"),
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

    // Idempotencia: si ya existe con mismo entity_identifier + entity_type, retorna el existente
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
    Ok((StatusCode::OK, Json(resp)))
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
        (status = 200, description = "Favorito actualizado"),
        (status = 404, description = "No encontrado"),
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
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
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

    // Hard delete (Django también lo hace con soft=False en este endpoint)
    user_favorites::Entity::delete_by_id(fav.id)
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /workspaces/{slug}/user-favorites/{favorite_id}/children/
///
/// Lista favoritos hijos de un grupo/folder.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-favorites/{favorite_id}/children/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("favorite_id" = Uuid, Path, description = "Favorite parent ID"),
    ),
    responses(
        (status = 200, description = "Lista de favoritos hijos"),
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

#[derive(Debug, Serialize)]
pub struct HomePreferenceResponse {
    pub id: Uuid,
    pub key: String,
    pub is_enabled: bool,
    pub config: JsonValue,
    pub sort_order: f64,
}

impl From<workspace_home_preferences::Model> for HomePreferenceResponse {
    fn from(m: workspace_home_preferences::Model) -> Self {
        Self {
            id: m.id,
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

/// GET /workspaces/{slug}/home-preference/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/home-preference/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Preferencias de home"),
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

/// PATCH /workspaces/{slug}/home-preference/{key}/
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/home-preference/{key}/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("key" = String, Path, description = "Preference key"),
    ),
    responses(
        (status = 200, description = "Preferencia actualizada"),
        (status = 404, description = "No encontrada"),
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

    let pref = workspace_home_preferences::Entity::find()
        .filter(workspace_home_preferences::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_home_preferences::Column::UserId.eq(user_id))
        .filter(workspace_home_preferences::Column::Key.eq(&key))
        .filter(workspace_home_preferences::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

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

    let saved = active.update(db).await.map_err(AppError::Database)?;
    let resp: HomePreferenceResponse = saved.into();
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
        (status = 200, description = "Lista de quick links"),
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
        (status = 201, description = "Quick link creado"),
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
        (status = 200, description = "Actualizado"),
        (status = 404, description = "No encontrado"),
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
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
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

/// GET /workspaces/{slug}/user-recent-visit/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-recent-visit/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("entity_name" = Option<String>, Query, description = "Filtrar por tipo de entidad"),
    ),
    responses(
        (status = 200, description = "Lista de visitas recientes (máx 20)"),
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

    // Sólo entidades válidas: issue, page, project
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
        // Filtrar sólo entidades permitidas usando sea_orm OR condition
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
}

/// GET /workspaces/{slug}/stickies/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/stickies/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("query" = Option<String>, Query, description = "Buscar en descripción"),
    ),
    responses(
        (status = 200, description = "Lista de stickies"),
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

    let mut query = stickies::Entity::find()
        .filter(stickies::Column::WorkspaceId.eq(ws.id))
        .filter(stickies::Column::OwnerId.eq(user_id))
        .filter(stickies::Column::DeletedAt.is_null());

    if let Some(search) = &q.query {
        if !search.is_empty() {
            query = query.filter(
                stickies::Column::DescriptionStripped.contains(search.as_str()),
            );
        }
    }

    let items = query
        .order_by_desc(stickies::Column::SortOrder)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<StickyResponse> = items.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// POST /workspaces/{slug}/stickies/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/stickies/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Sticky creado"),
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
        (status = 200, description = "Sticky actualizado"),
        (status = 403, description = "Sin permisos (solo el creador)"),
        (status = 404, description = "No encontrado"),
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
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
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

/// GET /workspaces/{slug}/user-preference/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-preference/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Preferencias de usuario (mapa key → {is_pinned, sort_order})"),
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

    let prefs = workspace_user_preferences::Entity::find()
        .filter(workspace_user_preferences::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_user_preferences::Column::UserId.eq(user_id))
        .filter(workspace_user_preferences::Column::DeletedAt.is_null())
        .order_by_asc(workspace_user_preferences::Column::SortOrder)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Devuelve mapa { key: {is_pinned, sort_order} } igual que Django
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

/// PATCH /workspaces/{slug}/user-preference/
///
/// Body: array de objetos `{key, is_pinned?, sort_order?}`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/user-preference/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Actualizado"),
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

#[derive(Debug, Serialize)]
pub struct DraftIssueResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub description_html: String,
    pub priority: String,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub type_id: Option<Uuid>,
    pub estimate_point_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sort_order: f64,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<draft_issues::Model> for DraftIssueResponse {
    fn from(m: draft_issues::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description_html: m.description_html,
            priority: m.priority,
            state_id: m.state_id,
            parent_id: m.parent_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            type_id: m.type_id,
            estimate_point_id: m.estimate_point_id,
            start_date: m.start_date,
            target_date: m.target_date,
            completed_at: m.completed_at.map(Into::into),
            sort_order: m.sort_order,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateDraftIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub estimate_point_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sort_order: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateDraftIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub estimate_point_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sort_order: Option<f64>,
}

/// GET /workspaces/{slug}/draft-issues/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/draft-issues/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Lista de draft issues del usuario"),
    )
)]
pub async fn list_draft_issues(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let user_id = auth_user.id;
    let ws = workspace_by_slug(db, &slug).await?;
    let _member = require_workspace_member(db, ws.id, user_id).await?;

    let issues = draft_issues::Entity::find()
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .filter(draft_issues::Column::CreatedById.eq(user_id))
        .filter(draft_issues::Column::DeletedAt.is_null())
        .order_by_desc(draft_issues::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<DraftIssueResponse> = issues.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

/// POST /workspaces/{slug}/draft-issues/
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/draft-issues/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Draft issue creado"),
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
    let _member = require_workspace_member(db, ws.id, user_id).await?;

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
        project_id: Set(body.project_id),
        workspace_id: Set(ws.id),
        type_id: Set(body.type_id),
        estimate_point_id: Set(body.estimate_point_id),
        start_date: Set(body.start_date),
        target_date: Set(body.target_date),
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

    let saved = new.insert(db).await.map_err(AppError::Database)?;
    let resp: DraftIssueResponse = saved.into();
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
        (status = 200, description = "Draft issue detalle"),
        (status = 404, description = "No encontrado"),
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

    let resp: DraftIssueResponse = issue.into();
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
        (status = 204, description = "Actualizado"),
        (status = 404, description = "No encontrado"),
    )
)]
pub async fn update_draft_issue(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateDraftIssueRequest>,
) -> Result<StatusCode, AppError> {
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

    let mut active: draft_issues::ActiveModel = issue.into();
    if let Some(v) = body.name {
        active.name = Set(Some(v));
    }
    if let Some(v) = body.description_html {
        active.description_html = Set(v);
    }
    if let Some(v) = body.priority {
        active.priority = Set(v);
    }
    if let Some(v) = body.state_id {
        active.state_id = Set(Some(v));
    }
    if let Some(v) = body.parent_id {
        active.parent_id = Set(Some(v));
    }
    if let Some(v) = body.project_id {
        active.project_id = Set(Some(v));
    }
    if let Some(v) = body.type_id {
        active.type_id = Set(Some(v));
    }
    if let Some(v) = body.estimate_point_id {
        active.estimate_point_id = Set(Some(v));
    }
    if let Some(v) = body.start_date {
        active.start_date = Set(Some(v));
    }
    if let Some(v) = body.target_date {
        active.target_date = Set(Some(v));
    }
    if let Some(v) = body.sort_order {
        active.sort_order = Set(v);
    }
    active.updated_at = Set(chrono::Utc::now().into());
    active.updated_by_id = Set(Some(user_id));

    active.update(db).await.map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
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
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
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
// Equivalentes a: workspace/cycle.py, module.py, estimate.py, label.py, state.py
// Son vistas de sólo lectura que listan entidades a través de todos los proyectos
// del workspace a los que el usuario pertenece.

// ── Cycles ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorkspaceCycleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub owned_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<cycles::Model> for WorkspaceCycleResponse {
    fn from(m: cycles::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            start_date: m.start_date.map(Into::into),
            end_date: m.end_date.map(Into::into),
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            owned_by_id: m.owned_by_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

/// GET /workspaces/{slug}/cycles/
///
/// Lista todos los cycles no archivados del workspace.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/cycles/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Lista de cycles del workspace"),
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

    let resp: Vec<WorkspaceCycleResponse> = items.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(resp)))
}

// ── Modules ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorkspaceModuleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub lead_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<modules::Model> for WorkspaceModuleResponse {
    fn from(m: modules::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            status: m.status,
            start_date: m.start_date,
            target_date: m.target_date,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            lead_id: m.lead_id,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

/// GET /workspaces/{slug}/modules/
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/modules/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Lista de modules del workspace"),
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

    let resp: Vec<WorkspaceModuleResponse> = items.into_iter().map(Into::into).collect();
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
        (status = 200, description = "Lista de estimates del workspace"),
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

    // Cargar estimates + sus puntos con dos queries (no ORM join con SeaORM)
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
        (status = 200, description = "Lista de labels del workspace"),
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
        (status = 200, description = "Lista de states del workspace"),
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
