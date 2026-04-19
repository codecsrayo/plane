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
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
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
        workspace_user_preferences, workspace_user_properties,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::pagination,
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

/// Respuesta del GET — mirror de Django `.values("key", "is_enabled", "config", "sort_order")`.
/// No incluye `id` porque Django tampoco lo expone en este endpoint.
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

/// Keys que se auto-siembran en el GET.
///
/// Mirror de Django `WorkspaceHomePreference.HomeWidgetKeys.choices`
/// EXCLUYENDO `quick_tutorial` y `new_at_plane` (filtradas explícitamente
/// en `workspace/home.py:33-36`).
const HOME_PREFERENCE_KEYS: &[&str] = &["quick_links", "recents", "my_stickies"];

/// GET /workspaces/{slug}/home-preferences/
///
/// Mirror de Django `WorkspaceHomePreferenceViewSet.get`
/// (`plane/app/views/workspace/home.py:23`).
///
/// Comportamiento de auto-seed: para cada key faltante en
/// `HOME_PREFERENCE_KEYS` se crea una fila con defaults (is_enabled=true,
/// config={}, sort_order = 1000 − position). Esto garantiza que el
/// frontend siempre reciba un set completo de widgets sin necesidad de
/// un flujo de inicialización separado.
///
/// Se usa `INSERT ... ON CONFLICT DO NOTHING` para evitar race conditions
/// entre tabs del mismo usuario, coherente con el `bulk_create(
/// ignore_conflicts=True)` de Django.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/home-preferences/",
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

    // ── 1. Leer keys existentes ────────────────────────────────────────────
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

    // ── 2. Auto-seed keys faltantes ────────────────────────────────────────
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

        // ON CONFLICT DO NOTHING — mirror de bulk_create(ignore_conflicts=True).
        // La unique constraint parcial cubre (workspace, user, key) WHERE deleted_at IS NULL.
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

    // ── 3. Leer todas (incluidas las recién insertadas) ────────────────────
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

/// GET /workspaces/{slug}/recent-visits/
///
/// Paridad con Django (`plane/app/views/workspace/recent_visit.py::UserRecentVisitViewSet`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/recent-visits/",
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
    /// Cursor en formato Django: `"per_page:offset:is_prev"` (e.g. `"20:0:0"`).
    pub cursor: Option<String>,
    /// Override opcional del per_page (Django lo toma antes que el cursor).
    pub per_page: Option<u64>,
}

/// GET /workspaces/{slug}/stickies/
///
/// Paridad con Django (`plane/app/views/workspace/sticky.py::WorkspaceStickyViewSet.list`).
/// Devuelve shape paginado: `{ results, total_count, next_cursor, prev_cursor,
/// next_page_results, prev_page_results, count, total_pages, total_results,
/// grouped_by, sub_grouped_by, extra_stats }`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/stickies/",
    tag = "Workspace Extras",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("query" = Option<String>, Query, description = "Buscar en descripción"),
        ("cursor" = Option<String>, Query, description = "Cursor Django: per_page:offset:is_prev"),
        ("per_page" = Option<u64>, Query, description = "Override per_page"),
    ),
    responses(
        (status = 200, description = "Lista paginada de stickies"),
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

    // Paridad con Django `WorkspaceStickyViewSet.list` + `paginate(default_per_page=20)`.
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

    // Count y page deben usar el MISMO query (mismos filtros) — clonamos antes
    // de aplicar el orden para evitar divergencia.
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

/// GET /workspaces/{slug}/sidebar-preferences/
///
/// Espejo de Django `WorkspaceUserPreferenceViewSet.get` en
/// `plane/app/views/workspace/user_preference.py:26`. Devuelve el mapa
/// `{key: {is_pinned, sort_order}}` y SIEMBRA los keys faltantes con
/// defaults (sort_order = 65535 + i*10000, pinned para drafts/your_work/
/// stickies) para que el frontend tenga estado consistente desde el
/// primer render.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/sidebar-preferences/",
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

    // ── 1. Seed de keys faltantes ───────────────────────────────────────────
    //
    // Mirror exacto de Django: el GET es responsable de garantizar que
    // existan filas para cada UserPreferenceKeys.choices. Si no existen,
    // se crean con defaults; si ya existen (race condition entre tabs)
    // se ignoran via ON CONFLICT DO NOTHING.
    //
    // Orden y defaults son normativos — el frontend asume estos valores.
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

    // Keys ya presentes para este (workspace, user).
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

    // Construir ActiveModel solo para los keys que faltan, manteniendo el
    // mismo cálculo de sort_order que Django: 65535 + i*10000 donde `i` es
    // la posición dentro del subset de keys faltantes (no del total).
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
        // ON CONFLICT DO NOTHING — espejo de bulk_create(ignore_conflicts=True).
        // La unique constraint parcial cubre (workspace_id, user_id, key) cuando
        // deleted_at IS NULL, así que reaplicar el GET concurrentemente desde otra
        // pestaña no rompe.
        //
        // IMPORTANTE: PostgreSQL EXIGE repetir el predicado `WHERE deleted_at IS NULL`
        // en el conflict target para poder inferir un arbiter index parcial — sin
        // `.target_and_where(...)` el INSERT revienta con "there is no unique or
        // exclusion constraint matching the ON CONFLICT specification" aunque las
        // columnas coincidan exactamente con la constraint. Ver
        // plane/db/models/workspace.py:443-451 (constraint Django) y
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

    // ── 2. Leer todas las preferencias (incluye las recién insertadas) ──────
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

/// PATCH /workspaces/{slug}/sidebar-preferences/
///
/// Body: array de objetos `{key, is_pinned?, sort_order?}`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/sidebar-preferences/",
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

// ── Helpers de sync N-a-M para Draft Issues ────────────────────────────────
//
// Espejan la lógica de `DraftIssueCreateSerializer.create` / `.update` en
// `apps/api/plane/app/serializers/draft.py:142-297`. Django hace hard-delete
// (`.delete()`) sobre los assignees/labels/cycles/modules existentes y luego
// `bulk_create` con los nuevos. Nosotros replicamos el comportamiento con
// hard-delete + insert en la misma transacción.
//
// NOTA: draft_issue_* NO tiene soft-delete lógico en Django (el
// `.delete()` sobre `BaseManager` es hard-delete porque las tablas están
// declaradas sin soft-delete). Por paridad hacemos hard-delete aquí.

/// Reemplaza los assignees del draft con `new_ids` (hard-delete existentes + insert).
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

/// Reemplaza los labels del draft con `new_ids`.
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

/// Reemplaza el (único) cycle del draft. `new_id = None` solo borra.
///
/// Paridad con draft.py:266-276: siempre borra existentes; solo crea uno
/// nuevo si `cycle_id` es truthy (en Django: no None, no ""; aquí no es
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

/// Reemplaza los modules del draft con `new_ids`.
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

/// Shape de `POST /workspaces/{slug}/draft-issues/`.
///
/// Paridad exacta con `DraftIssueCreateSerializer` (apps/api/plane/app/
/// serializers/draft.py:33) y con el payload que construye el frontend
/// desde `DEFAULT_WORK_ITEM_FORM_VALUES` (packages/constants/src/issue/
/// modal.ts).
///
/// # Convenciones críticas
/// - Todos los `Option<Uuid>` / `Option<NaiveDate>` usan los helpers de
///   `crate::utils::serde_empty` para convertir `""` → `None`. El frontend
///   envía `project_id: ""`, `state_id: ""`, fechas vacías por default; serde
///   nativo falla con 422 (HTTP Unprocessable Entity) al toparse con `""`
///   donde espera un `Uuid`.
/// - `estimate_point` sin `_id` — DRF expone la FK con ese nombre cuando se
///   declara como `PrimaryKeyRelatedField(source="estimate_point", ...)`.
///   La columna en la DB sí es `estimate_point_id`; el mapeo lo hace el
///   handler (mirror de `CreateIssueRequest` en `issues.rs:163`).
/// - `assignee_ids` / `label_ids` (plural + sufijo) son los nombres que usan
///   DRF y el frontend; son `ListField(required=False)` en Django.
/// - `cycle_id` y `module_ids` existen en el payload del frontend aunque
///   el `DraftIssueCreateSerializer` los lee desde `initial_data`
///   (draft.py:146-147) — aquí los declaramos explícitos.
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
    #[serde(default)]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub label_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub cycle_id: Option<Uuid>,
    #[serde(default)]
    pub module_ids: Option<Vec<Uuid>>,
}

/// Shape de `PATCH /workspaces/{slug}/draft-issues/{pk}/`.
///
/// Paridad con `DraftIssueCreateSerializer(partial=True)` (draft.py:170-178).
/// Mismas convenciones que `CreateDraftIssueRequest`.
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
    #[serde(default)]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub label_ids: Option<Vec<Uuid>>,
    /// `cycle_id` en PATCH usa Option<Option<Uuid>> para distinguir:
    /// - campo ausente                         → no tocar cycle (`"not_provided"` en Django)
    /// - `cycle_id: null` o `cycle_id: ""`     → desasignar cycle
    /// - `cycle_id: "<uuid>"`                  → asignar cycle
    ///
    /// Django lee `request.data.get("cycle_id", "not_provided")` (draft.py:176)
    /// y en el serializer `if cycle_id != "not_provided"` decide si tocarlo
    /// (draft.py:266). Replicamos ese comportamiento usando el truco de
    /// `Option<Option<T>>` + `#[serde(default, with = "::serde_with::rust::double_option")]`
    /// — pero sin depender de `serde_with`, implementamos el mismo doble-wrap
    /// manualmente: `#[serde(default, deserialize_with = ...)]`.
    #[serde(default, deserialize_with = "deserialize_double_option_uuid")]
    pub cycle_id: Option<Option<Uuid>>,
    #[serde(default)]
    pub module_ids: Option<Vec<Uuid>>,
}

/// Deserializador para `Option<Option<Uuid>>` con empty-string-as-none.
///
/// Semántica:
/// - campo ausente                → `None`            (no tocar)
/// - `null` o `""`                → `Some(None)`      (desasignar)
/// - `"<uuid>"`                   → `Some(Some(uuid))` (asignar)
///
/// Necesario para PATCH donde queremos distinguir "campo no enviado" de
/// "campo enviado como null/empty". Serde nativo colapsa ambos a `None`
/// con `Option<Uuid>`.
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
    let member = require_workspace_member(db, ws.id, user_id).await?;
    // Draft issues: GUEST es permitido en Django (draft.py:111); no usamos
    // `require_member_or_admin` aquí para mantener paridad.
    let _ = member;

    // Validación paridad con DraftIssueCreateSerializer.validate (draft.py:72-77):
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

    // Ejecutar en transacción para que el draft + sus N-a-M queden atómicos,
    // igual que `DraftIssueCreateSerializer.create` que hace bulk_create
    // dentro del mismo request y rollback si algo falla.
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
                    // `estimate_point` del payload DRF → columna `estimate_point_id`.
                    estimate_point_id: Set(body.estimate_point),
                    start_date: Set(body.start_date),
                    target_date: Set(body.target_date),
                    // Django default (draft.py DraftIssue model): sort_order=65535.0
                    // si no viene; paridad exacta.
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

    // Validación paridad con DraftIssueCreateSerializer.validate (draft.py:72-77).
    // En PATCH comparamos el par efectivo: campo enviado si presente, si no
    // el valor actual del draft.
    let effective_start = body.start_date.or(issue.start_date);
    let effective_target = body.target_date.or(issue.target_date);
    if let (Some(start), Some(target)) = (effective_start, effective_target) {
        if start > target {
            return Err(AppError::BadRequest(
                "Start date cannot exceed target date".into(),
            ));
        }
    }

    // `project_id` efectivo para sincronizar las N-a-M (Django toma el
    // del request si viene, si no el del draft — draft.py:168).
    let effective_project_id = body.project_id.or(issue.project_id);
    let workspace_id = ws.id;
    let issue_id = issue.id;

    use sea_orm::TransactionTrait;
    db.transaction::<_, (), AppError>(|txn| {
        // Build ActiveModel sync — espejo del patrón de issues.rs:update_issue
        // (evita tener que mover `body` completo al async y lidiar con borrows
        // parciales).
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
        // Para los campos UUID simples: `deserialize_empty_as_none_uuid`
        // ya colapsa `""` → `None`, así que `is_some()` detecta solo el
        // caso de UUID válido presente. Paridad con issues.rs:update_issue.
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

        // Extraer las N-a-M para el async block.
        let assignees = body.assignee_ids.clone();
        let labels = body.label_ids.clone();
        let cycle_id = body.cycle_id;
        let modules = body.module_ids.clone();

        Box::pin(async move {
            active.update(txn).await.map_err(AppError::Database)?;

            // Sync N-a-M solo si el campo vino en el payload (paridad con
            // `if assignees is not None` en draft.py:232,249).
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
            // cycle_id: Some(Some(uuid))=asignar, Some(None)=desasignar,
            // None=no tocar. Paridad con `if cycle_id != "not_provided"` en
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

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE USER PROPERTIES
// ═══════════════════════════════════════════════════════════════════════════
//
// Mirror de Django `WorkspaceUserPropertiesEndpoint`
// (plane/app/views/workspace/user.py:252).
//
// GET   /workspaces/{slug}/user-properties/
// PATCH /workspaces/{slug}/user-properties/
//
// Ambos operan bajo la semántica `get_or_create(user, workspace)`: si no
// existe la fila la crean con defaults. En Rust se implementa de forma
// atómica con `INSERT ... ON CONFLICT DO NOTHING` para evitar race
// conditions entre tabs del mismo usuario, seguido de un SELECT.
//
// Permiso: `WorkspaceViewerPermission` (cualquier miembro activo del
// workspace).
//
// Seguridad y diferencias intencionales respecto a Django:
// - El body del PATCH usa una whitelist estricta; `id`, `user_id`,
//   `workspace_id`, timestamps y `deleted_at` no pueden ser pisados desde
//   el request.
// - `navigation_control_preference` se valida contra el enum Django
//   (`ACCORDION` | `TABBED`). Django delega la validación al CharField
//   + choices a nivel de serializer; aquí se hace explícito.
// - `navigation_project_limit` se valida en rango `0..=1000` para evitar
//   valores absurdos / overflow de `i32`. Django no valida; este
//   endurecimiento es consistente con la política de no introducir
//   patrones inseguros.

/// Valores permitidos para `navigation_control_preference`.
///
/// Mirror del enum Django
/// `WorkspaceUserProperties.NavigationControlPreference.choices`.
const NAVIGATION_CONTROL_PREFERENCES: &[&str] = &["ACCORDION", "TABBED"];

/// Límite razonable para `navigation_project_limit`. Django no valida;
/// aquí se acota para evitar valores hostiles o absurdos.
const MAX_NAVIGATION_PROJECT_LIMIT: i32 = 1000;

/// Defaults idénticos a Django `get_default_filters` en
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

/// Mirror de `get_default_display_filters`
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

/// Mirror de `get_default_display_properties`
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
            // Django `WorkspaceUserPropertiesSerializer` usa `fields = "__all__"`
            // con `read_only_fields = ["workspace", "user"]`. En la respuesta
            // JSON esos FK aparecen como `workspace` y `user` (no como
            // `workspace_id`/`user_id`). Preservamos ese contrato para el
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

/// Body aceptado por el PATCH. Todos los campos son opcionales
/// (partial update, igual que Django `partial=True`). Los FKs y timestamps
/// quedan fuera de la whitelist por diseño.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkspaceUserPropertiesRequest {
    pub filters: Option<JsonValue>,
    pub display_filters: Option<JsonValue>,
    pub display_properties: Option<JsonValue>,
    pub rich_filters: Option<JsonValue>,
    pub navigation_project_limit: Option<i32>,
    pub navigation_control_preference: Option<String>,
}

/// Devuelve la fila existente o la crea con defaults.
///
/// Implementa `get_or_create(user=..., workspace=...)` de forma atómica:
/// 1. `INSERT ... ON CONFLICT DO NOTHING` con defaults Django.
/// 2. `SELECT` filtrando por `(workspace_id, user_id)` activos.
///
/// La unique constraint parcial
/// `workspace_user_properties_unique_workspace_user_when_deleted_at_null`
/// (ver `plane/db/models/workspace.py:338-344`) garantiza que no existan
/// dos filas activas para el mismo par (workspace, user).
async fn get_or_create_workspace_user_properties(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<workspace_user_properties::Model, AppError> {
    use sea_orm::sea_query::OnConflict;

    let now = chrono::Utc::now().fixed_offset();

    // Intento de insert con ON CONFLICT DO NOTHING.
    // Si ya existe una fila activa para (workspace_id, user_id) la unique
    // constraint parcial hace fallar el insert silenciosamente.
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
            // REQUISITO de PostgreSQL, no decoración: cuando el arbiter index es parcial
            // (`UNIQUE (workspace_id, user_id) WHERE deleted_at IS NULL`, ver
            // WorkspaceUserProperties.Meta.constraints en plane/db/models/workspace.py:338-344),
            // el `ON CONFLICT` DEBE repetir el mismo predicado o PostgreSQL rechaza con
            // `there is no unique or exclusion constraint matching the ON CONFLICT specification`
            // (ver https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT
            // → "If an index_predicate is specified, it must, as a further requirement for
            //    inference, satisfy arbiter indexes"). El otro unique existente
            // (`unique_together = [workspace, user, deleted_at]`) tampoco matchea porque
            // incluye `deleted_at` como tercera columna y nosotros solo apuntamos a dos.
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

    // SELECT garantizado: ya sea la fila recién insertada o la existente.
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
        (status = 200, description = "Propiedades del usuario en el workspace"),
        (status = 403, description = "No es miembro activo del workspace"),
        (status = 404, description = "Workspace no existe"),
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
/// Política de actualización:
/// - Whitelist estricta (FKs/timestamps/id nunca se tocan desde el body).
/// - `navigation_control_preference` ∈ {"ACCORDION", "TABBED"}.
/// - `navigation_project_limit` ∈ [0, MAX_NAVIGATION_PROJECT_LIMIT].
/// - Campos JSON (`filters`, `display_filters`, `display_properties`,
///   `rich_filters`) se reemplazan por valor, respetando el contrato
///   `partial=True` del serializer Django.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/user-properties/",
    tag = "Workspace Extras",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Propiedades actualizadas"),
        (status = 400, description = "Body inválido"),
        (status = 403, description = "No es miembro activo del workspace"),
        (status = 404, description = "Workspace no existe"),
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

    // Validaciones antes de tocar DB: fallan rápido y no consumen escritura.
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
