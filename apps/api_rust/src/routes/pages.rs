// src/routes/pages.rs
//! Endpoints de Pages (documentos de proyecto).
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
//!   GET    /api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{page_versions, pages, project_pages},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Access constants (matches Django Page.ACCESS_CHOICES) ────────────────────
const ACCESS_PUBLIC: i16 = 0;
const ACCESS_PRIVATE: i16 = 1;

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PageResponse {
    pub id: Uuid,
    pub name: String,
    pub description_html: String,
    pub access: i16,
    pub owned_by_id: Uuid,
    pub workspace_id: Uuid,
    pub is_locked: bool,
    pub archived_at: Option<chrono::NaiveDate>,
    pub parent_id: Option<Uuid>,
    pub color: String,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl PageResponse {
    fn from_model(m: pages::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description_html: m.description_html,
            access: m.access,
            owned_by_id: m.owned_by_id,
            workspace_id: m.workspace_id,
            is_locked: m.is_locked,
            archived_at: m.archived_at,
            parent_id: m.parent_id,
            color: m.color,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
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
    // `name` es opcional y por defecto vacío para reflejar Django:
    // `Page.name = TextField(blank=True)` + PageSerializer sin `required=True`.
    // El frontend crea páginas desde el botón "Create your first Page"
    // enviando únicamente `{ access }`, sin `name`.
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

// ── Helper: resolve page through project_pages ────────────────────────────────

async fn find_project_page(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    page_id: Uuid,
) -> Result<pages::Model, AppError> {
    // Verificar que la página pertenece a este proyecto vía project_pages
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

// ── GET /pages/ ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Lista de páginas")),
    security(("TokenAuth" = []))
)]
pub async fn list_pages(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<PageResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Obtener IDs de páginas del proyecto a través de project_pages
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
        // Usuarios ven páginas públicas o propias
        .filter(
            pages::Column::Access
                .eq(ACCESS_PUBLIC)
                .or(pages::Column::OwnedById.eq(guard.user.id)),
        )
        .order_by_desc(pages::Column::UpdatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(PageResponse::from_model).collect()))
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
        (status = 201, description = "Página creada"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreatePageRequest>,
) -> Result<(StatusCode, Json<PageResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // Validar `access` contra el choice set de Django
    // `Page.access = PositiveSmallIntegerField(choices=((0, "Public"), (1, "Private")), default=0)`.
    // Django rechazaría cualquier otro valor en el serializer; replicamos esa
    // validación aquí para no guardar basura en DB.
    let access = body.access.unwrap_or(ACCESS_PUBLIC);
    if access != ACCESS_PUBLIC && access != ACCESS_PRIVATE {
        return Err(AppError::BadRequest(
            "access debe ser 0 (Public) o 1 (Private)".into(),
        ));
    }

    // Django permite nombres vacíos (`TextField(blank=True)`), no rechazamos.
    let name = body.name.unwrap_or_default();

    // Django usa `request.data.get("description_html", "<p></p>")` al crear.
    let description_html = body.description_html.unwrap_or_else(|| "<p></p>".into());

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
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Asociar página al proyecto en project_pages
    project_pages::ActiveModel {
        id: Set(Uuid::new_v4()),
        page_id: Set(page.id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(PageResponse::from_model(page))))
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
        (status = 200, description = "Detalle de la página"),
        (status = 404, description = "No encontrada"),
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

    // Verificar acceso: público o propietario
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    Ok(Json(PageResponse::from_model(page)))
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
        (status = 200, description = "Página actualizada"),
        (status = 403, description = "Solo el propietario puede editar"),
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

    // Solo el propietario o un admin puede editar
    if page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    if page.is_locked {
        return Err(AppError::BadRequest("La página está bloqueada".into()));
    }

    let mut am: pages::ActiveModel = page.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(html) = body.description_html {
        am.description_html = Set(html);
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
    Ok(Json(PageResponse::from_model(updated)))
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
    responses((status = 204, description = "Eliminada")),
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
    responses((status = 200, description = "Página archivada")),
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
    Ok(Json(PageResponse::from_model(updated)))
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
    responses((status = 200, description = "Página desarchivada")),
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
    Ok(Json(PageResponse::from_model(updated)))
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
    responses((status = 200, description = "Página bloqueada")),
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
    Ok(Json(PageResponse::from_model(updated)))
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
    responses((status = 200, description = "Página desbloqueada")),
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
    Ok(Json(PageResponse::from_model(updated)))
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
    responses((status = 201, description = "Copia creada")),
    security(("TokenAuth" = []))
)]
pub async fn duplicate_page(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<PageResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let source = find_project_page(&state.db, guard.project.id, page_id).await?;

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
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
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
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(PageResponse::from_model(new_page))))
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
    responses((status = 200, description = "Historial de versiones")),
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
        (status = 200, description = "Versión específica"),
        (status = 404, description = "No encontrada"),
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
