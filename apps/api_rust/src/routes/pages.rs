// src/routes/pages.rs
//! Pages endpoints (project documents).
//!
//!   GET    /api/workspaces/{slug}/projects/{project_id}/pages/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/pages/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}
//!   GET    /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/description/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/description/

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use regex::Regex;
use std::{collections::HashSet, sync::OnceLock};

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{page_labels, page_logs, page_versions, pages, project_pages, user_favorites},
    error::AppError,
    utils::{content_validator, soft_delete::SoftDeleteExt},
    AppState,
};

// ── Access constants (matches Django Page.ACCESS_CHOICES) ────────────────────
const ACCESS_PUBLIC: i16 = 0;
const ACCESS_PRIVATE: i16 = 1;

// ── Defaults (matches Django Page model defaults) ─────────────────────────────
// `Page.DEFAULT_SORT_ORDER = 65535` — ver plane/db/models/page.py
const DEFAULT_SORT_ORDER: f64 = 65535.0;

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PageResponse {
    pub id: Uuid,
    pub name: String,
    pub description_html: String,
    pub description_json: serde_json::Value,
    pub access: i16,
    #[serde(rename = "owned_by")]
    pub owned_by_id: Uuid,
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub is_locked: bool,
    pub is_favorite: bool,
    pub archived_at: Option<chrono::NaiveDate>,
    pub deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub parent_id: Option<Uuid>,
    pub color: String,
    pub logo_props: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    /// UUIDs of the projects the page belongs to (M2M via
    /// `project_pages`). Mirror of `PageSerializer.project_ids` in Django —
    /// the frontend uses it as `page.project_ids?.[0]` to build routes
    /// and make HTTP calls; if it's empty, the client-side guard throws
    /// "Missing required fields" before reaching the backend.
    pub project_ids: Vec<Uuid>,
    /// UUIDs of associated labels (M2M via `page_labels`).
    pub label_ids: Vec<Uuid>,
}

impl PageResponse {
    fn from_model(m: pages::Model, project_ids: Vec<Uuid>, label_ids: Vec<Uuid>) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description_html: m.description_html,
            description_json: m.description_json,
            access: m.access,
            owned_by_id: m.owned_by_id,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
            is_locked: m.is_locked,
            is_favorite: false,
            archived_at: m.archived_at,
            deleted_at: m.deleted_at.map(Into::into),
            parent_id: m.parent_id,
            color: m.color,
            logo_props: m.logo_props,
            created_at: m.created_at,
            updated_at: m.updated_at,
            project_ids,
            label_ids,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PageVersionResponse {
    pub id: Uuid,
    pub page_id: Uuid,
    pub description_html: String,
    pub last_saved_at: chrono::DateTime<chrono::FixedOffset>,
    pub owned_by_id: Uuid,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePageRequest {
    // `name` is optional and empty by default to reflect Django:
    // `Page.name = TextField(blank=True)` + PageSerializer without `required=True`.
    // The frontend creates pages from the "Create your first Page" button
    // sending only `{ access }`, without `name`.
    #[serde(default)]
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub color: Option<String>,
    pub access: Option<i16>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePageRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub color: Option<String>,
    pub access: Option<i16>,
    pub parent_id: Option<Uuid>,
}

/// Body for `PATCH /pages/{id}/description/` — mirror of
/// `PageBinaryUpdateSerializer` in Django. All fields are optional: the
/// client can send only the one it needs to update (e.g. the Y.js editor
/// sends only `description_binary` on autosave, while on closing
/// the page it also sends `description_html` and `description_json`).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePageDescriptionRequest {
    /// Base64 of the serialized Y.js document. Decoding + size validation
    /// (10 MB) + suspicious pattern heuristics apply in the handler.
    pub description_binary: Option<String>,
    /// Editor HTML. It will be sanitized with `ammonia` before saving,
    /// preserving Plane custom tags (`mention-component`, etc.).
    pub description_html: Option<String>,
    /// JSON representation (Tiptap ProseMirror doc). It is saved as is.
    pub description_json: Option<serde_json::Value>,
}

// ── Helper: resolve page through project_pages ────────────────────────────────

async fn find_project_page(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    page_id: Uuid,
) -> Result<pages::Model, AppError> {
    // Verify that the page belongs to this project via project_pages
    let pp = project_pages::Entity::find()
        .active()
        .filter(project_pages::Column::ProjectId.eq(project_id))
        .filter(project_pages::Column::PageId.eq(page_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    pages::Entity::find_by_id(pp.page_id)
        .active()
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

// ── M2M helpers ──────────────────────────────────────────────────────────────
//
// Django mirrors `project_ids`/`label_ids` on each Page row via
// `ArrayAgg` in the queryset (see `page/base.py:120-123`). In SeaORM we
// don't have native aggregation in the main query without breaking the mapping to the
// entity, so we resolve M2M with auxiliary queries. Soft-delete is respected
// in `project_pages` (`.active()`) and in `page_labels`.

/// M2M of a single page — used by handlers that return a page
/// after a write (create/update/archive/lock/duplicate/get).
async fn fetch_page_m2m(
    db: &sea_orm::DatabaseConnection,
    page_id: Uuid,
) -> Result<(Vec<Uuid>, Vec<Uuid>), AppError> {
    let project_ids: Vec<Uuid> = project_pages::Entity::find()
        .active()
        .filter(project_pages::Column::PageId.eq(page_id))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pp| pp.project_id)
        .collect();

    let label_ids: Vec<Uuid> = page_labels::Entity::find()
        .active()
        .filter(page_labels::Column::PageId.eq(page_id))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pl| pl.label_id)
        .collect();

    Ok((project_ids, label_ids))
}

/// M2M of multiple pages — batched for `list_pages` to avoid N+1.
/// Returns a pair of HashMaps: `(page_id -> project_ids, page_id -> label_ids)`.
async fn fetch_pages_m2m(
    db: &sea_orm::DatabaseConnection,
    page_ids: &[Uuid],
) -> Result<
    (
        std::collections::HashMap<Uuid, Vec<Uuid>>,
        std::collections::HashMap<Uuid, Vec<Uuid>>,
    ),
    AppError,
> {
    use std::collections::HashMap;
    if page_ids.is_empty() {
        return Ok((HashMap::new(), HashMap::new()));
    }

    let mut projects_by_page: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for pp in project_pages::Entity::find()
        .active()
        .filter(project_pages::Column::PageId.is_in(page_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        projects_by_page.entry(pp.page_id).or_default().push(pp.project_id);
    }

    let mut labels_by_page: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for pl in page_labels::Entity::find()
        .active()
        .filter(page_labels::Column::PageId.is_in(page_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?
    {
        labels_by_page.entry(pl.page_id).or_default().push(pl.label_id);
    }

    Ok((projects_by_page, labels_by_page))
}

/// Batch-load is_favorite for a list of PageResponse (single query, no N+1).
async fn enrich_page_favorites(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
    workspace_id: Uuid,
    mut pages: Vec<PageResponse>,
) -> Result<Vec<PageResponse>, AppError> {
    if pages.is_empty() {
        return Ok(pages);
    }
    let fav_ids: std::collections::HashSet<Uuid> = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::WorkspaceId.eq(workspace_id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    for p in &mut pages {
        p.is_favorite = fav_ids.contains(&p.id);
    }
    Ok(pages)
}

// ── GET /pages/ ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "List of pages")),
    security(("TokenAuth" = []))
)]
pub async fn list_pages(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<PageResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Get page IDs of the project through project_pages
    let page_ids: Vec<Uuid> = project_pages::Entity::find()
        .active()
        .filter(project_pages::Column::ProjectId.eq(guard.project.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pp| pp.page_id)
        .collect();

    if page_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let rows = pages::Entity::find()
        .active()
        .filter(pages::Column::Id.is_in(page_ids))
        .filter(pages::Column::ArchivedAt.is_null())
        // Users see public or their own pages
        .filter(
            pages::Column::Access
                .eq(ACCESS_PUBLIC)
                .or(pages::Column::OwnedById.eq(guard.user.id)),
        )
        .order_by_desc(pages::Column::UpdatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batched M2M to avoid N+1 in projects with many pages.
    let ids: Vec<Uuid> = rows.iter().map(|p| p.id).collect();
    let (mut projects_by_page, mut labels_by_page) = fetch_pages_m2m(&state.db, &ids).await?;

    let responses: Vec<PageResponse> = rows
        .into_iter()
        .map(|m| {
            let pids = projects_by_page.remove(&m.id).unwrap_or_default();
            let lids = labels_by_page.remove(&m.id).unwrap_or_default();
            PageResponse::from_model(m, pids, lids)
        })
        .collect();

    let responses = enrich_page_favorites(&state.db, guard.user.id, guard.workspace.id, responses).await?;
    Ok(Json(responses))
}

// ── POST /pages/ ──────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Page created"),
        (status = 400, description = "Validation error"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreatePageRequest>,
) -> Result<(StatusCode, Json<PageResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // Validate `access` against Django's choice set
    // `Page.access = PositiveSmallIntegerField(choices=((0, "Public"), (1, "Private")), default=0)`.
    // Django would reject any other value in the serializer; we replicate that
    // validation here to avoid saving garbage in DB.
    let access = body.access.unwrap_or(ACCESS_PUBLIC);
    if access != ACCESS_PUBLIC && access != ACCESS_PRIVATE {
        return Err(AppError::BadRequest(
            "access must be 0 (Public) or 1 (Private)".into(),
        ));
    }

    // Django allows empty names (`TextField(blank=True)`), we don't reject.
    let name = body.name.unwrap_or_default();

    // Django uses `request.data.get("description_html", "<p></p>")` on create.
    let raw_html = body.description_html.unwrap_or_else(|| "<p></p>".into());
    let description_html = content_validator::sanitize_description(&raw_html)
        .map_err(AppError::BadRequest)?;

    // Django fills created_at/updated_at via `BaseModel.save()`
    // (`auto_now_add=True` / `auto_now=True`). Columns in DB are NOT NULL;
    // SeaORM does not auto-populate them, they must be explicitly set. The
    // same instant is shared for the page and its project_pages row.
    let now: DateTime<FixedOffset> = Utc::now().into();

    let page = pages::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        description_html: Set(description_html),
        description_json: Set(serde_json::json!({})),
        description_stripped: Set(None),
        description_binary: Set(None),
        access: Set(access),
        color: Set(body.color.unwrap_or_default()),
        parent_id: Set(body.parent_id),
        owned_by_id: Set(guard.user.id),
        workspace_id: Set(guard.workspace.id),
        is_locked: Set(false),
        view_props: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        // NOT NULL without default in the generated entity — they must be
        // explicitly set with Django defaults:
        // `is_global = BooleanField(default=False)`
        // `sort_order = FloatField(default=DEFAULT_SORT_ORDER)`
        is_global: Set(false),
        sort_order: Set(DEFAULT_SORT_ORDER),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Associate page with the project in project_pages
    project_pages::ActiveModel {
        id: Set(Uuid::new_v4()),
        page_id: Set(page.id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // The newly created page is associated with exactly one project (the
    // project_pages just inserted above) and has no labels yet.
    // We avoid a DB round-trip by returning the known IDs inline.
    Ok((
        StatusCode::CREATED,
        Json(PageResponse::from_model(
            page,
            vec![guard.project.id],
            vec![],
        )),
    ))
}

// ── GET /pages/{page_id}/ ─────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses(
        (status = 200, description = "Page detail"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    // Verify access: public or owner
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    let (pids, lids) = fetch_page_m2m(&state.db, page.id).await?;
    let mut _single_resp = PageResponse::from_model(page, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}

// ── PATCH /pages/{page_id}/ ───────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses(
        (status = 200, description = "Page updated"),
        (status = 403, description = "Only the owner can edit"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdatePageRequest>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    // Only the owner or an admin can edit
    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    if page.is_locked {
        return Err(AppError::BadRequest("Page is locked".into()));
    }

    let mut am: pages::ActiveModel = page.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(html) = body.description_html {
        let clean = content_validator::sanitize_description(&html)
            .map_err(AppError::BadRequest)?;
        am.description_html = Set(clean);
    }
    if let Some(color) = body.color {
        am.color = Set(color);
    }
    if let Some(access) = body.access {
        am.access = Set(access);
    }
    if body.parent_id.is_some() {
        am.parent_id = Set(body.parent_id);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    let (pids, lids) = fetch_page_m2m(&state.db, updated.id).await?;
    let mut _single_resp = PageResponse::from_model(updated, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}

// ── DELETE /pages/{page_id}/ ──────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 204, description = "Deleted")),
    security(("TokenAuth" = []))
)]
pub async fn delete_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    let mut am: pages::ActiveModel = page.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── POST /pages/{page_id}/archive/ ───────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "Page archived")),
    security(("TokenAuth" = []))
)]
pub async fn archive_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    let mut am: pages::ActiveModel = page.into();
    am.archived_at = Set(Some(chrono::Utc::now().date_naive()));
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    let (pids, lids) = fetch_page_m2m(&state.db, updated.id).await?;
    let mut _single_resp = PageResponse::from_model(updated, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}

// ── DELETE /pages/{page_id}/archive/ (unarchive) ─────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "Page unarchived")),
    security(("TokenAuth" = []))
)]
pub async fn unarchive_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    let mut am: pages::ActiveModel = page.into();
    am.archived_at = Set(None);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    let (pids, lids) = fetch_page_m2m(&state.db, updated.id).await?;
    let mut _single_resp = PageResponse::from_model(updated, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}

// ── POST /pages/{page_id}/lock/ ───────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "Page locked")),
    security(("TokenAuth" = []))
)]
pub async fn lock_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    let mut am: pages::ActiveModel = page.into();
    am.is_locked = Set(true);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    let (pids, lids) = fetch_page_m2m(&state.db, updated.id).await?;
    let mut _single_resp = PageResponse::from_model(updated, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}

// ── DELETE /pages/{page_id}/lock/ (unlock) ────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "Page unlocked")),
    security(("TokenAuth" = []))
)]
pub async fn unlock_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    let mut am: pages::ActiveModel = page.into();
    am.is_locked = Set(false);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    let (pids, lids) = fetch_page_m2m(&state.db, updated.id).await?;
    let mut _single_resp = PageResponse::from_model(updated, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}

// ── POST /pages/{page_id}/duplicate/ ─────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 201, description = "Copy created")),
    security(("TokenAuth" = []))
)]
pub async fn duplicate_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<PageResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let source = find_project_page(&state.db, guard.project.id, page_id).await?;

    // Django auto-popula timestamps en save(); SeaORM no lo hace.
    let now: DateTime<FixedOffset> = Utc::now().into();

    let new_page = pages::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(format!("Copy of {}", source.name)),
        description_html: Set(source.description_html.clone()),
        description_json: Set(source.description_json.clone()),
        description_stripped: Set(source.description_stripped.clone()),
        description_binary: Set(source.description_binary.clone()),
        access: Set(ACCESS_PRIVATE), // copia privada por defecto
        color: Set(source.color.clone()),
        parent_id: Set(None),
        owned_by_id: Set(guard.user.id),
        workspace_id: Set(guard.workspace.id),
        is_locked: Set(false),
        view_props: Set(source.view_props.clone()),
        logo_props: Set(source.logo_props.clone()),
        // NOT NULL — preserve the original value when duplicating (Django shares
        // the same logical row when doing .save() of a new model with the
        // copied attributes). is_global is inherited; sort_order as well.
        is_global: Set(source.is_global),
        sort_order: Set(source.sort_order),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    project_pages::ActiveModel {
        id: Set(Uuid::new_v4()),
        page_id: Set(new_page.id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // We use fetch_page_m2m instead of `vec![guard.project.id]` so that
    // the response reflects what was actually inserted if someone fixes the
    // pre-existing TODO in this function: Django duplicates `project_pages` to all
    // projects where the source page was (see
    // `apps/api/plane/app/views/page/base.py:594-611`), while
    // Rust only inserts a row for the current project.
    let (pids, lids) = fetch_page_m2m(&state.db, new_page.id).await?;
    Ok((
        StatusCode::CREATED,
        Json(PageResponse::from_model(new_page, pids, lids)),
    ))
}

// ── GET /pages/{page_id}/versions/ ───────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "Version history")),
    security(("TokenAuth" = []))
)]
pub async fn list_page_versions(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<Vec<PageVersionResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let _ = find_project_page(&state.db, guard.project.id, page_id).await?;

    let versions = page_versions::Entity::find()
        .filter(page_versions::Column::PageId.eq(page_id))
        .order_by_desc(page_versions::Column::LastSavedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        versions
            .into_iter()
            .map(|v| PageVersionResponse {
                id: v.id,
                page_id: v.page_id,
                description_html: v.description_html,
                last_saved_at: v.last_saved_at,
                owned_by_id: v.owned_by_id,
                created_at: v.created_at,
            })
            .collect(),
    ))
}

// ── GET /pages/{page_id}/versions/{pk}/ ──────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
        ("pk" = Uuid, Path, description = "Version ID"),
    ),
    responses(
        (status = 200, description = "Specific version"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_page_version(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<Json<PageVersionResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let _ = find_project_page(&state.db, guard.project.id, page_id).await?;

    let v = page_versions::Entity::find_by_id(pk)
        .filter(page_versions::Column::PageId.eq(page_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(PageVersionResponse {
        id: v.id,
        page_id: v.page_id,
        description_html: v.description_html,
        last_saved_at: v.last_saved_at,
        owned_by_id: v.owned_by_id,
        created_at: v.created_at,
    }))
}
// ── GET /pages/{page_id}/description/ ────────────────────────────────────────
//
// Mirror of `PagesDescriptionViewSet.retrieve` in
// `apps/api/plane/app/views/page/base.py`. Serves the binary Y.js document
// (`description_binary`) as `application/octet-stream` so that the
// collaborative editor loads it when opening the page.
//
// Access control: Django uses `Q(owned_by=user) | Q(access=0)`. In Rust
// we replicate that filter with the same pattern as `get_page`: a 404/403 according to
// visibility — we never reveal that the page exists if the user cannot
// see it. `find_project_page` already guarantees that the row belongs to the project
// and is not soft-deleted (`project_pages.deleted_at IS NULL`).

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/description/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses(
        (status = 200, description = "Binary Y.js document", content_type = "application/octet-stream"),
        (status = 403, description = "Private page of another user"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_page_description(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    // Access: public or owner. Private of another user ⇒ 403.
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    // Django sends `b""` when `description_binary` is NULL (stream_data yield
    // b""), with status 200. We replicate that semantics returning an empty body
    // in that case so the editor knows it must initialize a new doc.
    let bytes = page.description_binary.unwrap_or_default();

    // Explicit headers. Arrays of `(HeaderName, &str)` implement
    // `IntoResponseParts` in Axum and are applied before the body, so the
    // Content-Type here wins against the default of `Vec<u8>`.
    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            r#"attachment; filename="page_description.bin""#,
        ),
    ];
    Ok((headers, bytes))
}

// ── Page versioning & transaction helpers ────────────────────────────────────
//
// Parity with Django Celery tasks:
//   page_version_task.track_page_version  (bgtasks/page_version_task.py)
//   page_transaction_task.page_transaction (bgtasks/page_transaction_task.py)
// Implemented inline (synchronous) — no dedicated job type required.

const PAGE_VERSION_TIMEOUT_SECS: i64 = 600;
const PAGE_VERSION_MAX_COUNT: u64 = 20;

struct PageComponent {
    /// Component's own `id` attribute — used as PageLog.transaction.
    id: Uuid,
    /// Referenced entity (e.g. user UUID for mention, None for image).
    entity_identifier: Option<Uuid>,
    entity_name: String,
}

/// Extracts mention-component and image-component elements from HTML.
/// Returns (mention_components, image_components).
fn extract_page_components(html: &str) -> (Vec<PageComponent>, Vec<PageComponent>) {
    static MENTION_RE: OnceLock<Regex> = OnceLock::new();
    static IMAGE_RE: OnceLock<Regex> = OnceLock::new();
    static ATTR_ID_RE: OnceLock<Regex> = OnceLock::new();
    static ATTR_EI_RE: OnceLock<Regex> = OnceLock::new();
    static ATTR_EN_RE: OnceLock<Regex> = OnceLock::new();
    static ATTR_SRC_RE: OnceLock<Regex> = OnceLock::new();

    let mention_re = MENTION_RE
        .get_or_init(|| Regex::new(r"(?i)<mention-component\s[^>]*/?>").unwrap());
    let image_re = IMAGE_RE
        .get_or_init(|| Regex::new(r"(?i)<image-component\s[^>]*/?>").unwrap());
    let attr_id_re = ATTR_ID_RE
        .get_or_init(|| Regex::new(r#"(?i)\bid\s*=\s*"([^"]+)""#).unwrap());
    let attr_ei_re = ATTR_EI_RE
        .get_or_init(|| Regex::new(r#"(?i)\bentity_identifier\s*=\s*"([^"]+)""#).unwrap());
    let attr_en_re = ATTR_EN_RE
        .get_or_init(|| Regex::new(r#"(?i)\bentity_name\s*=\s*"([^"]+)""#).unwrap());
    let attr_src_re = ATTR_SRC_RE
        .get_or_init(|| Regex::new(r#"(?i)\bsrc\s*=\s*"([^"]+)""#).unwrap());

    let extract_attr = |tag: &str, re: &Regex| -> Option<String> {
        re.captures(tag)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    };

    let mentions: Vec<PageComponent> = mention_re
        .find_iter(html)
        .filter_map(|m| {
            let tag = m.as_str();
            let id_str = extract_attr(tag, attr_id_re)?;
            let id = id_str.parse::<Uuid>().ok()?;
            let entity_identifier = extract_attr(tag, attr_ei_re)
                .and_then(|s| s.parse::<Uuid>().ok());
            let entity_name = extract_attr(tag, attr_en_re).unwrap_or_default();
            Some(PageComponent { id, entity_identifier, entity_name })
        })
        .collect();

    let images: Vec<PageComponent> = image_re
        .find_iter(html)
        .filter_map(|m| {
            let tag = m.as_str();
            let id_str = extract_attr(tag, attr_id_re)?;
            let id = id_str.parse::<Uuid>().ok()?;
            // `src` is a URL — not storable as UUID; entity_identifier is None.
            let _ = attr_src_re;
            Some(PageComponent { id, entity_identifier: None, entity_name: "image".to_string() })
        })
        .collect();

    (mentions, images)
}

/// Parity with `track_page_version` Celery task.
/// Upserts the current page snapshot into `page_versions` with a 600-second
/// rolling window per user.
async fn track_page_version_inline(
    db: &sea_orm::DatabaseConnection,
    page: &pages::Model,
    old_html: &str,
    user_id: Uuid,
) -> Result<(), AppError> {
    if old_html == page.description_html {
        return Ok(());
    }

    let now: DateTime<FixedOffset> = Utc::now().into();

    // Latest version for this page, ordered by last_saved_at DESC.
    let latest = page_versions::Entity::find()
        .filter(page_versions::Column::PageId.eq(page.id))
        .filter(page_versions::Column::DeletedAt.is_null())
        .order_by_desc(page_versions::Column::LastSavedAt)
        .one(db)
        .await
        .map_err(AppError::Database)?;

    let reuse = latest.as_ref().is_some_and(|v| {
        v.owned_by_id == user_id
            && (now.timestamp() - v.last_saved_at.timestamp()) <= PAGE_VERSION_TIMEOUT_SECS
    });

    if reuse {
        let version = latest.unwrap();
        let mut am: page_versions::ActiveModel = version.into();
        am.description_html = Set(page.description_html.clone());
        am.description_binary = Set(page.description_binary.clone());
        am.description_json = Set(page.description_json.clone());
        am.description_stripped = Set(page.description_stripped.clone());
        am.sub_pages_data = Set(serde_json::json!({}));
        am.updated_at = Set(now);
        am.update(db).await.map_err(AppError::Database)?;
    } else {
        page_versions::ActiveModel {
            id: Set(Uuid::new_v4()),
            page_id: Set(page.id),
            workspace_id: Set(page.workspace_id),
            description_html: Set(page.description_html.clone()),
            description_binary: Set(page.description_binary.clone()),
            description_json: Set(page.description_json.clone()),
            description_stripped: Set(page.description_stripped.clone()),
            owned_by_id: Set(user_id),
            last_saved_at: Set(now),
            sub_pages_data: Set(serde_json::json!({})),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(AppError::Database)?;

        // Cap at 20 versions: delete oldest if exceeded.
        let count = page_versions::Entity::find()
            .filter(page_versions::Column::PageId.eq(page.id))
            .filter(page_versions::Column::DeletedAt.is_null())
            .count(db)
            .await
            .map_err(AppError::Database)?;

        if count > PAGE_VERSION_MAX_COUNT {
            if let Some(oldest) = page_versions::Entity::find()
                .filter(page_versions::Column::PageId.eq(page.id))
                .filter(page_versions::Column::DeletedAt.is_null())
                .order_by_asc(page_versions::Column::LastSavedAt)
                .one(db)
                .await
                .map_err(AppError::Database)?
            {
                let mut am: page_versions::ActiveModel = oldest.into();
                am.deleted_at = Set(Some(now));
                am.update(db).await.map_err(AppError::Database)?;
            }
        }
    }

    Ok(())
}

/// Parity with `page_transaction` Celery task.
/// Diffs old vs new HTML for mention/image components and upserts PageLog rows.
async fn page_transaction_inline(
    db: &sea_orm::DatabaseConnection,
    new_html: &str,
    old_html: &str,
    page_id: Uuid,
    workspace_id: Uuid,
) -> Result<(), AppError> {
    let has_existing = page_logs::Entity::find()
        .filter(page_logs::Column::PageId.eq(page_id))
        .filter(page_logs::Column::DeletedAt.is_null())
        .count(db)
        .await
        .map_err(AppError::Database)?
        > 0;

    let (old_mentions, old_images) = extract_page_components(old_html);
    let (new_mentions, new_images) = extract_page_components(new_html);

    let old_ids: HashSet<Uuid> = old_mentions.iter().chain(old_images.iter()).map(|c| c.id).collect();
    let new_ids: HashSet<Uuid> = new_mentions.iter().chain(new_images.iter()).map(|c| c.id).collect();

    let deleted_ids: Vec<Uuid> = old_ids.difference(&new_ids).copied().collect();
    if !deleted_ids.is_empty() {
        let now: DateTime<FixedOffset> = Utc::now().into();
        let to_delete = page_logs::Entity::find()
            .filter(page_logs::Column::Transaction.is_in(deleted_ids))
            .filter(page_logs::Column::PageId.eq(page_id))
            .all(db)
            .await
            .map_err(AppError::Database)?;
        for log in to_delete {
            let mut am: page_logs::ActiveModel = log.into();
            am.deleted_at = Set(Some(now));
            am.update(db).await.map_err(AppError::Database)?;
        }
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    for component in new_mentions.iter().chain(new_images.iter()) {
        if old_ids.contains(&component.id) && has_existing {
            continue;
        }
        page_logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction: Set(component.id),
            page_id: Set(page_id),
            entity_identifier: Set(component.entity_identifier),
            entity_name: Set(component.entity_name.clone()),
            entity_type: Set(None),
            workspace_id: Set(workspace_id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(())
}

// ── PATCH /pages/{page_id}/description/ ──────────────────────────────────────
//
// Mirror of `PagesDescriptionViewSet.partial_update` in
// `apps/api/plane/app/views/page/base.py:520`. Accepts any combination
// of `description_binary` (base64), `description_html` (sanitized) and
// `description_json` (JSON). All three are saved atomically in the same
// `pages` row with the `updated_at` timestamp refreshed by SeaORM.
//
// Pre-write validations (error code and message identical to Django
// so the shared frontend continues to work):
// * `page.is_locked`   ⇒ 400 {"error_code": 4701, "error_message": "PAGE_LOCKED"}
// * `page.archived_at` ⇒ 400 {"error_code": 4702, "error_message": "PAGE_ARCHIVED"}
//
// After saving, two inline tasks run for parity with Django Celery tasks:
//   * `page_transaction_inline` — diffs old/new HTML for component changes.
//   * `track_page_version_inline` — snapshots the page into `page_versions`.
// Both errors are swallowed (non-fatal) so a failed snapshot never blocks the save.

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/description/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses(
        (status = 200, description = "Updated successfully"),
        (status = 400, description = "Page locked, archived or invalid content"),
        (status = 403, description = "Private page of another user"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_page_description(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdatePageDescriptionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let page = find_project_page(&state.db, guard.project.id, page_id).await?;

    // Same access filter as GET — Django uses `Q(owned_by=user) | Q(access=0)`.
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    // Page status validations. Keep the same JSON shape as
    // Django (`error_code` int + `error_message` str) because the shared
    // client does `response.data.error_code === 4701` for toast-specifics.
    // Codes defined in `apps/api/plane/utils/error_codes.py:12-13`.
    if page.is_locked {
        return Err(AppError::Validation(serde_json::json!({
            "error_code": 4701,
            "error_message": "PAGE_LOCKED",
        })));
    }
    if page.archived_at.is_some() {
        return Err(AppError::Validation(serde_json::json!({
            "error_code": 4702,
            "error_message": "PAGE_ARCHIVED",
        })));
    }

    // Sanitize/validate content BEFORE opening the mutation. Any
    // validation failure returns 400 without touching the DB. Messages mirror
    // those of `PageBinaryUpdateSerializer` in Django so the frontend
    // can show them without additional translations.
    let decoded_binary: Option<Vec<u8>> = if let Some(ref b64) = body.description_binary {
        if b64.is_empty() {
            // DRF treats `""` as "do not change" but the serializer uses
            // `allow_blank=True` and saves the decoded binary as is.
            // An empty base64 decodes to `vec![]`, which the validator accepts.
            Some(Vec::new())
        } else {
            let decoded = BASE64_STANDARD.decode(b64).map_err(|_| {
                AppError::BadRequest("Failed to decode base64 data".into())
            })?;
            content_validator::validate_binary_data(&decoded).map_err(|e| {
                AppError::BadRequest(format!("Invalid binary data: {e}"))
            })?;
            Some(decoded)
        }
    } else {
        None
    };

    let sanitized_html: Option<String> = match body.description_html {
        Some(ref raw) => {
            let clean = content_validator::sanitize_description(raw)
                .map_err(AppError::BadRequest)?;
            Some(clean)
        }
        None => None,
    };

    // Apply only the fields present. `updated_at` is refreshed by SeaORM when
    // calling `update()` if the model has `auto_now`; in this project it is
    // not so, so we set it manually as in `update_page`.
    let now: DateTime<FixedOffset> = Utc::now().into();
    let old_html = page.description_html.clone();
    let mut am: pages::ActiveModel = page.into();

    if let Some(bytes) = decoded_binary {
        // Option<Vec<u8>> in the entity: we save `Some(bytes)`; an empty
        // buffer is saved as `Some(vec![])` and not as `None` (Django also
        // persists b"" without converting it to NULL).
        am.description_binary = Set(Some(bytes));
    }
    if let Some(html) = sanitized_html {
        am.description_html = Set(html);
    }
    if let Some(json) = body.description_json {
        am.description_json = Set(json);
    }
    am.updated_by_id = Set(Some(guard.user.id));
    am.updated_at = Set(now);

    let updated_page = am.update(&state.db).await.map_err(AppError::Database)?;

    // Inline parity with Django post-save Celery tasks (base.py:556-571).
    // Errors are non-fatal: the description was already saved successfully.
    let _ = track_page_version_inline(&state.db, &updated_page, &old_html, guard.user.id).await;
    let _ = page_transaction_inline(
        &state.db,
        &updated_page.description_html,
        &old_html,
        updated_page.id,
        updated_page.workspace_id,
    )
    .await;

    Ok(Json(serde_json::json!({ "message": "Updated successfully" })))
}


// ─── POST /workspaces/{slug}/projects/{project_id}/pages/{page_id}/access/ ───
pub async fn update_page_access(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    use sea_orm::ActiveValue::Set;
    let access: i16 = body.get("access")
        .and_then(|v| v.as_i64())
        .map(|v| v as i16)
        .unwrap_or(0);
    if ![0i16, 1].contains(&access) {
        return Err(AppError::BadRequest("access must be 0 (public) or 1 (private)".into()));
    }
    let page = pages::Entity::find_by_id(page_id).active()
        .filter(pages::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db).await.map_err(AppError::Database)?.ok_or(AppError::NotFound)?;
    if page.access != access && page.owned_by_id != guard.user.id {
        return Err(AppError::BadRequest("Access cannot be updated since this page is owned by someone else".into()));
    }
    let now = chrono::Utc::now().fixed_offset();
    let mut am: pages::ActiveModel = page.into();
    am.access = Set(access);
    am.updated_at = Set(now);
    am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── POST + DELETE /workspaces/{slug}/projects/{project_id}/favorite-pages/{page_id}/ ──
pub async fn add_page_favorite(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    use sea_orm::ActiveValue::Set;
    if guard.project_member.role < ROLE_MEMBER && guard.workspace_member.role < 20 {
        return Err(AppError::Forbidden);
    }

    // Validate that the page exists and belongs to the project.
    // Django (apps/api/plane/app/views/page/base.py:476-483) creates the favorite
    // without validating existence, which produces orphan rows in
    // user_favorites pointing to ghost UUIDs (UserFavorite has no FK to
    // pages because entity_identifier is polymorphic). Deliberate reinforcement
    // to preserve referential integrity. Same helper as the rest of
    // pages.rs (find_project_page).
    find_project_page(&state.db, project_id, page_id).await?;

    let now = chrono::Utc::now().fixed_offset();
    let existing = user_favorites::Entity::find().active()
        .filter(user_favorites::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::EntityIdentifier.eq(page_id))
        .one(&state.db).await.map_err(AppError::Database)?;
    if existing.is_none() {
        user_favorites::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set("page".into()),
            entity_identifier: Set(Some(page_id)),
            project_id: Set(Some(project_id)),
            workspace_id: Set(guard.workspace.id),
            user_id: Set(guard.user.id),
            is_folder: Set(false),
            sequence: Set(65535.0),
            created_by_id: Set(Some(guard.user.id)),
            updated_by_id: Set(Some(guard.user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        }.insert(&state.db).await.map_err(AppError::Database)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_page_favorite(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Django parity (apps/api/plane/app/views/page/base.py:486-495):
    //   `UserFavorite.objects.get(...)` throws DoesNotExist if no row exists
    //   → DRF translates it to 404. The handler previously used delete_many() which
    //   silently returned 204 with 0 rows affected, diverging from
    //   Django and masking client bugs.
    let fav = user_favorites::Entity::find().active()
        .filter(user_favorites::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::EntityIdentifier.eq(page_id))
        .one(&state.db).await.map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let am: user_favorites::ActiveModel = fav.into();
    am.delete(&state.db).await.map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── GET /workspaces/{slug}/projects/{project_id}/pages-summary/ ─────────────
pub async fn pages_summary(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<serde_json::Value>, AppError> {
    use sea_orm::{FromQueryResult, Statement};

    #[derive(FromQueryResult)]
    struct SummaryRow {
        public_pages: i64,
        private_pages: i64,
        total_pages: i64,
        archived_pages: i64,
    }

    let row = SummaryRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT
            COUNT(CASE WHEN p.access = 0 AND p.archived_at IS NULL THEN 1 END) AS public_pages,
            COUNT(CASE WHEN p.access = 1 AND p.archived_at IS NULL AND p.owned_by_id = $1 THEN 1 END) AS private_pages,
            COUNT(CASE WHEN p.archived_at IS NULL THEN 1 END) AS total_pages,
            COUNT(CASE WHEN p.archived_at IS NOT NULL THEN 1 END) AS archived_pages
        FROM pages p
        INNER JOIN project_pages pp ON pp.page_id = p.id AND pp.project_id = $2 AND pp.deleted_at IS NULL
        WHERE p.workspace_id = $3 AND p.deleted_at IS NULL AND p.parent_id IS NULL
          AND (p.owned_by_id = $1 OR p.access = 0)
        "#,
        vec![
            sea_orm::Value::Uuid(Some(Box::new(guard.user.id))),
            sea_orm::Value::Uuid(Some(Box::new(guard.project.id))),
            sea_orm::Value::Uuid(Some(Box::new(guard.workspace.id))),
        ],
    )).one(&state.db).await.map_err(AppError::Database)?
      .unwrap_or(SummaryRow { public_pages: 0, private_pages: 0, total_pages: 0, archived_pages: 0 });

    Ok(Json(serde_json::json!({
        "public_pages": row.public_pages,
        "private_pages": row.private_pages,
        "total_pages": row.total_pages,
        "archived_pages": row.archived_pages,
    })))
}
// ── GET /workspaces/{slug}/projects/{project_id}/favorite-pages/ ─────────────
/// Lists the pages marked as favorites by the user in the project.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/favorite-pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "List of favorite pages")),
    security(("TokenAuth" = []))
)]
pub async fn list_favorite_pages(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<PageResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let db = &state.db;

    // Load favorite page_ids of the user in this project
    let fav_page_ids: Vec<Uuid> = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::ProjectId.eq(guard.project.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    if fav_page_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let rows = pages::Entity::find()
        .active()
        .filter(pages::Column::Id.is_in(fav_page_ids))
        .filter(pages::Column::ArchivedAt.is_null())
        .order_by_desc(pages::Column::UpdatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let ids: Vec<Uuid> = rows.iter().map(|p| p.id).collect();
    let (mut projects_by_page, mut labels_by_page) = fetch_pages_m2m(db, &ids).await?;
    let responses = rows.into_iter().map(|m| {
        let pids = projects_by_page.remove(&m.id).unwrap_or_default();
        let lids = labels_by_page.remove(&m.id).unwrap_or_default();
        PageResponse::from_model(m, pids, lids)
    }).collect();

    let responses = enrich_page_favorites(&state.db, guard.user.id, guard.workspace.id, responses).await?;
    Ok(Json(responses))
}

// ── GET /workspaces/{slug}/projects/{project_id}/archived-pages/ ─────────────
/// Lists the archived pages of the project.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/archived-pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "List of archived pages")),
    security(("TokenAuth" = []))
)]
pub async fn list_archived_pages(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<PageResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let db = &state.db;

    let page_ids: Vec<Uuid> = project_pages::Entity::find()
        .active()
        .filter(project_pages::Column::ProjectId.eq(guard.project.id))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pp| pp.page_id)
        .collect();

    if page_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let rows = pages::Entity::find()
        .active()
        .filter(pages::Column::Id.is_in(page_ids))
        .filter(pages::Column::ArchivedAt.is_not_null())
        .filter(
            pages::Column::Access
                .eq(ACCESS_PUBLIC)
                .or(pages::Column::OwnedById.eq(guard.user.id)),
        )
        .order_by_desc(pages::Column::ArchivedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let ids: Vec<Uuid> = rows.iter().map(|p| p.id).collect();
    let (mut projects_by_page, mut labels_by_page) = fetch_pages_m2m(db, &ids).await?;
    let responses = rows.into_iter().map(|m| {
        let pids = projects_by_page.remove(&m.id).unwrap_or_default();
        let lids = labels_by_page.remove(&m.id).unwrap_or_default();
        PageResponse::from_model(m, pids, lids)
    }).collect();

    let responses = enrich_page_favorites(&state.db, guard.user.id, guard.workspace.id, responses).await?;
    Ok(Json(responses))
}

// ── POST /workspaces/{slug}/projects/{project_id}/pages/{page_id}/move/ ──────
/// Moves a page to another project in the same workspace.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/move/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "Page moved")),
    security(("TokenAuth" = []))
)]
pub async fn move_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<PageResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let db = &state.db;

    let new_project_id: Uuid = body
        .get("new_project_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("new_project_id required".into()))?;

    // Verify that the user is a member of the destination project
    use crate::entities::project_members;
    let _dest_member = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(new_project_id))
        .filter(project_members::Column::MemberId.eq(guard.user.id))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Forbidden)?;

    // Verify that the page exists and belongs to the source project
    let pp = project_pages::Entity::find()
        .active()
        .filter(project_pages::Column::ProjectId.eq(guard.project.id))
        .filter(project_pages::Column::PageId.eq(page_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Only the owner can move
    let page = pages::Entity::find_by_id(page_id)
        .active()
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    // Update the project_pages relation to the new project
    let mut am: project_pages::ActiveModel = pp.into();
    am.project_id = sea_orm::Set(new_project_id);
    am.update(db).await.map_err(AppError::Database)?;

    let ids = vec![page.id];
    let (mut projects_by_page, mut labels_by_page) = fetch_pages_m2m(db, &ids).await?;
    let pids = projects_by_page.remove(&page.id).unwrap_or_default();
    let lids = labels_by_page.remove(&page.id).unwrap_or_default();
    let mut _single_resp = PageResponse::from_model(page, pids, lids);
    _single_resp.is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(_single_resp.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db).await.map_err(AppError::Database)? > 0;
    Ok(Json(_single_resp))
}
