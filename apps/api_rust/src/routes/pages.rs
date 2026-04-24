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
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{page_labels, page_versions, pages, project_pages, user_favorites},
    error::AppError,
    utils::{content_validator, soft_delete::SoftDeleteExt},
    AppState,
};

// ââ Access constants (matches Django Page.ACCESS_CHOICES) ââââââââââââââââââââ
const ACCESS_PUBLIC: i16 = 0;
const ACCESS_PRIVATE: i16 = 1;

// ââ Defaults (matches Django Page model defaults) âââââââââââââââââââââââââââââ
// `Page.DEFAULT_SORT_ORDER = 65535` â ver plane/db/models/page.py
const DEFAULT_SORT_ORDER: f64 = 65535.0;

// ââ DTOs âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

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
    /// UUIDs de los proyectos a los que pertenece la pÃ¡gina (M2M via
    /// `project_pages`). Mirror de `PageSerializer.project_ids` en Django â
    /// el frontend lo usa como `page.project_ids?.[0]` para construir rutas
    /// y hacer llamadas HTTP; si viene vacÃ­o, el guard client-side lanza
    /// "Missing required fields" antes de llegar al backend.
    pub project_ids: Vec<Uuid>,
    /// UUIDs de los labels asociados (M2M via `page_labels`).
    pub label_ids: Vec<Uuid>,
}

impl PageResponse {
    fn from_model(m: pages::Model, project_ids: Vec<Uuid>, label_ids: Vec<Uuid>) -> Self {
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
    // `name` es opcional y por defecto vacÃ­o para reflejar Django:
    // `Page.name = TextField(blank=True)` + PageSerializer sin `required=True`.
    // El frontend crea pÃ¡ginas desde el botÃ³n "Create your first Page"
    // enviando Ãºnicamente `{ access }`, sin `name`.
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

/// Body para `PATCH /pages/{id}/description/` â mirror de
/// `PageBinaryUpdateSerializer` en Django. Todos los campos son opcionales: el
/// cliente puede enviar solo el que necesita actualizar (e.g. el editor Y.js
/// envÃ­a Ãºnicamente `description_binary` al autosave, mientras que al cerrar
/// la pÃ¡gina envÃ­a tambiÃ©n `description_html` y `description_json`).
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePageDescriptionRequest {
    /// Base64 del documento Y.js serializado. Decode + validaciÃ³n de tamaÃ±o
    /// (10 MB) + heurÃ­stica de patrones sospechosos aplican en el handler.
    pub description_binary: Option<String>,
    /// HTML del editor. SerÃ¡ sanitizado con `ammonia` antes de guardar,
    /// preservando los tags custom de Plane (`mention-component`, etc.).
    pub description_html: Option<String>,
    /// RepresentaciÃ³n JSON (Tiptap ProseMirror doc). Se guarda tal cual.
    pub description_json: Option<serde_json::Value>,
}

// ââ Helper: resolve page through project_pages ââââââââââââââââââââââââââââââââ

async fn find_project_page(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    page_id: Uuid,
) -> Result<pages::Model, AppError> {
    // Verificar que la pÃ¡gina pertenece a este proyecto vÃ­a project_pages
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

// ââ M2M helpers ââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ
//
// Django espeja `project_ids`/`label_ids` sobre cada fila de Page vÃ­a
// `ArrayAgg` en el queryset (ver `page/base.py:120-123`). En SeaORM no
// tenemos agregaciÃ³n nativa en la query principal sin romper el mapeo a la
// entidad, asÃ­ que resolvemos los M2M con queries auxiliares. Se respeta el
// soft-delete en `project_pages` (`.active()`) y en `page_labels`.

/// M2M de una sola pÃ¡gina â usado por handlers que devuelven una pÃ¡gina
/// despuÃ©s de un write (create/update/archive/lock/duplicate/get).
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

/// M2M de mÃºltiples pÃ¡ginas â batched para `list_pages` y evitar N+1.
/// Devuelve un par de HashMaps: `(page_id -> project_ids, page_id -> label_ids)`.
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

// ââ GET /pages/ âââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Lista de pÃ¡ginas")),
    security(("TokenAuth" = []))
)]
pub async fn list_pages(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<PageResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Obtener IDs de pÃ¡ginas del proyecto a travÃ©s de project_pages
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
        // Usuarios ven pÃ¡ginas pÃºblicas o propias
        .filter(
            pages::Column::Access
                .eq(ACCESS_PUBLIC)
                .or(pages::Column::OwnedById.eq(guard.user.id)),
        )
        .order_by_desc(pages::Column::UpdatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batched M2M para evitar N+1 en proyectos con muchas pÃ¡ginas.
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

    Ok(Json(responses))
}

// ââ POST /pages/ ââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââââ

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "PÃ¡gina creada"),
        (status = 400, description = "Error de validaciÃ³n"),
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
    // Django rechazarÃ­a cualquier otro valor en el serializer; replicamos esa
    // validaciÃ³n aquÃ­ para no guardar basura en DB.
    let access = body.access.unwrap_or(ACCESS_PUBLIC);
    if access != ACCESS_PUBLIC && access != ACCESS_PRIVATE {
        return Err(AppError::BadRequest(
            "access debe ser 0 (Public) o 1 (Private)".into(),
        ));
    }

    // Django permite nombres vacÃ­os (`TextField(blank=True)`), no rechazamos.
    let name = body.name.unwrap_or_default();

    // Django usa `request.data.get("description_html", "<p></p>")` al crear.
    let raw_html = body.description_html.unwrap_or_else(|| "<p></p>".into());
    let description_html = content_validator::sanitize_description(&raw_html)
        .map_err(AppError::BadRequest)?;

    // Django rellena created_at/updated_at vÃ­a `BaseModel.save()`
    // (`auto_now_add=True` / `auto_now=True`). Las columnas en DB son NOT NULL;
    // SeaORM no las auto-popula, hay que setearlas explÃ­citamente. Se comparte
    // el mismo instante para la pÃ¡gina y su fila en project_pages.
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
        // NOT NULL sin default en la entidad generada â hay que setearlos
        // explÃ­citamente con los defaults de Django:
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

    // Asociar pÃ¡gina al proyecto en project_pages
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

    // La pÃ¡gina reciÃ©n creada estÃ¡ asociada a exactamente un proyecto (el
    // project_pages se acaba de insertar arriba) y no tiene labels todavÃ­a.
    // Evitamos un round-trip a DB devolviendo los IDs conocidos inline.
    Ok((
        StatusCode::CREATED,
        Json(PageResponse::from_model(
            page,
            vec![guard.project.id],
            vec![],
        )),
    ))
}

// ââ GET /pages/{page_id}/ âââââââââââââââââââââââââââââââââââââââââââââââââââââ

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
        (status = 200, description = "Detalle de la pÃ¡gina"),
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

    // Verificar acceso: pÃºblico o propietario
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    let (pids, lids) = fetch_page_m2m(&state.db, page.id).await?;
    Ok(Json(PageResponse::from_model(page, pids, lids)))
}

// ââ PATCH /pages/{page_id}/ âââââââââââââââââââââââââââââââââââââââââââââââââââ

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
        (status = 200, description = "PÃ¡gina actualizada"),
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
        return Err(AppError::BadRequest("La pÃ¡gina estÃ¡ bloqueada".into()));
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
    Ok(Json(PageResponse::from_model(updated, pids, lids)))
}

// ââ DELETE /pages/{page_id}/ ââââââââââââââââââââââââââââââââââââââââââââââââââ

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

// ââ POST /pages/{page_id}/archive/ âââââââââââââââââââââââââââââââââââââââââââ

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "PÃ¡gina archivada")),
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
    Ok(Json(PageResponse::from_model(updated, pids, lids)))
}

// ââ DELETE /pages/{page_id}/archive/ (unarchive) âââââââââââââââââââââââââââââ

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "PÃ¡gina desarchivada")),
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
    Ok(Json(PageResponse::from_model(updated, pids, lids)))
}

// ââ POST /pages/{page_id}/lock/ âââââââââââââââââââââââââââââââââââââââââââââââ

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "PÃ¡gina bloqueada")),
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
    Ok(Json(PageResponse::from_model(updated, pids, lids)))
}

// ââ DELETE /pages/{page_id}/lock/ (unlock) ââââââââââââââââââââââââââââââââââââ

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock/",
    tag = "Pages",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("page_id" = Uuid, Path, description = "Page ID"),
    ),
    responses((status = 200, description = "PÃ¡gina desbloqueada")),
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
    Ok(Json(PageResponse::from_model(updated, pids, lids)))
}

// ââ POST /pages/{page_id}/duplicate/ âââââââââââââââââââââââââââââââââââââââââ

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
        // NOT NULL â conservar el valor original al duplicar (Django comparte
        // la misma fila lÃ³gica al hacer .save() de un model nuevo con los
        // atributos copiados). is_global se hereda; sort_order tambiÃ©n.
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

    // Usamos fetch_page_m2m en lugar de `vec![guard.project.id]` para que
    // el response refleje lo realmente insertado si alguien arregla el TODO
    // preexistente de esta funciÃ³n: Django duplica `project_pages` a todos
    // los proyectos donde estaba la pÃ¡gina origen (ver
    // `apps/api/plane/app/views/page/base.py:594-611`), mientras que el
    // Rust solo inserta una fila para el proyecto actual.
    let (pids, lids) = fetch_page_m2m(&state.db, new_page.id).await?;
    Ok((
        StatusCode::CREATED,
        Json(PageResponse::from_model(new_page, pids, lids)),
    ))
}

// ââ GET /pages/{page_id}/versions/ âââââââââââââââââââââââââââââââââââââââââââ

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

// ââ GET /pages/{page_id}/versions/{pk}/ ââââââââââââââââââââââââââââââââââââââ

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
// ââ GET /pages/{page_id}/description/ ââââââââââââââââââââââââââââââââââââââââ
//
// Mirror de `PagesDescriptionViewSet.retrieve` en
// `apps/api/plane/app/views/page/base.py`. Sirve el documento Y.js binario
// (`description_binary`) como `application/octet-stream` para que el editor
// colaborativo lo cargue al abrir la pÃ¡gina.
//
// Control de acceso: Django usa `Q(owned_by=user) | Q(access=0)`. En Rust
// replicamos ese filtro con el mismo patrÃ³n que `get_page`: un 404/403 segÃºn
// visibilidad â nunca revelamos que la pÃ¡gina existe si el usuario no puede
// verla. `find_project_page` ya garantiza que la fila pertenece al proyecto
// y no estÃ¡ soft-deleted (`project_pages.deleted_at IS NULL`).

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
        (status = 403, description = "PÃ¡gina privada de otro usuario"),
        (status = 404, description = "No encontrada"),
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

    // Acceso: pÃºblica o propietario. Private de otro usuario â 403.
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    // Django envÃ­a `b""` cuando `description_binary` es NULL (stream_data yield
    // b""), con status 200. Replicamos esa semÃ¡ntica devolviendo un body vacÃ­o
    // en ese caso para que el editor sepa que debe inicializar un doc nuevo.
    let bytes = page.description_binary.unwrap_or_default();

    // Headers explÃ­citos. Los arrays de `(HeaderName, &str)` implementan
    // `IntoResponseParts` en Axum y se aplican antes del body, por lo que el
    // Content-Type aquÃ­ gana frente al default de `Vec<u8>`.
    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            r#"attachment; filename="page_description.bin""#,
        ),
    ];
    Ok((headers, bytes))
}

// ââ PATCH /pages/{page_id}/description/ ââââââââââââââââââââââââââââââââââââââ
//
// Mirror de `PagesDescriptionViewSet.partial_update` en
// `apps/api/plane/app/views/page/base.py:520`. Acepta cualquier combinaciÃ³n
// de `description_binary` (base64), `description_html` (sanitizado) y
// `description_json` (JSON). Los tres se guardan atÃ³micamente en una misma
// fila de `pages` con el timestamp de `updated_at` refrescado por SeaORM.
//
// Validaciones previas al write (cÃ³digo de error y mensaje idÃ©nticos a Django
// para que el frontend compartido siga funcionando):
// * `page.is_locked`   â 400 {"error_code": 4701, "error_message": "PAGE_LOCKED"}
// * `page.archived_at` â 400 {"error_code": 4702, "error_message": "PAGE_ARCHIVED"}
//
// NOTA: Django dispara dos tareas Celery en background tras guardar:
//   * `page_transaction(new_html, old_html, page_id)` â track de cambios.
//   * `track_page_version(page_id, existing_instance, user_id)` â snapshot en
//     `page_versions`.
// El API Rust aÃºn no tiene el job system para estas tareas (el worker de jobs
// solo cubre notificaciones hoy). Las dejamos como TODO visibles para no
// perder trazabilidad; el editor sigue funcionando sin versionado histÃ³rico
// en el Rust path â cuando haya cliente, el Django path sigue disponible.

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
        (status = 400, description = "PÃ¡gina bloqueada, archivada o contenido invÃ¡lido"),
        (status = 403, description = "PÃ¡gina privada de otro usuario"),
        (status = 404, description = "No encontrada"),
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

    // Mismo filtro de acceso que el GET â Django usa `Q(owned_by=user) | Q(access=0)`.
    if page.access == ACCESS_PRIVATE && page.owned_by_id != guard.user.id {
        return Err(AppError::Forbidden);
    }

    // Validaciones de estado de la pÃ¡gina. Mantener el mismo shape JSON que
    // Django (`error_code` int + `error_message` str) porque el cliente
    // compartido hace `response.data.error_code === 4701` para toast-specific.
    // CÃ³digos definidos en `apps/api/plane/utils/error_codes.py:12-13`.
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

    // Sanitizar/validar contenido ANTES de abrir la mutaciÃ³n. Cualquier
    // fallo de validaciÃ³n devuelve 400 sin tocar la DB. Los mensajes espejean
    // los de `PageBinaryUpdateSerializer` en Django para que el frontend
    // pueda mostrarlos sin traducciones adicionales.
    let decoded_binary: Option<Vec<u8>> = if let Some(ref b64) = body.description_binary {
        if b64.is_empty() {
            // DRF trata `""` como "no cambiar" pero el serializer usa
            // `allow_blank=True` y guarda el binario decodificado tal cual.
            // Un base64 vacÃ­o decodifica a `vec![]`, que el validador acepta.
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

    // Aplicar solo los campos presentes. `updated_at` lo refresca SeaORM al
    // llamar `update()` si el modelo tiene `auto_now`; en este proyecto no es
    // asÃ­, asÃ­ que lo setamos manualmente igual que en `update_page`.
    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut am: pages::ActiveModel = page.into();

    if let Some(bytes) = decoded_binary {
        // Option<Vec<u8>> en la entidad: guardamos `Some(bytes)`; un buffer
        // vacÃ­o se guarda como `Some(vec![])` y no como `None` (Django tambiÃ©n
        // persiste b"" sin convertirlo a NULL).
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

    am.update(&state.db).await.map_err(AppError::Database)?;

    // TODO(api_rust): encolar `page_transaction` y `track_page_version` cuando
    // el job worker soporte tareas de page versioning. Por ahora el Django
    // path sigue disponible como fallback para clientes que necesiten el
    // historial. Ver `apps/api/plane/app/views/page/base.py:556-571`.

    Ok(Json(serde_json::json!({ "message": "Updated successfully" })))
}


// âââ POST /workspaces/{slug}/projects/{project_id}/pages/{page_id}/access/ âââ
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

// âââ POST + DELETE /workspaces/{slug}/projects/{project_id}/favorite-pages/{page_id}/ ââ
pub async fn add_page_favorite(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, project_id, page_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    use sea_orm::ActiveValue::Set;
    if guard.project_member.role < ROLE_MEMBER && guard.workspace_member.role < 20 {
        return Err(AppError::Forbidden);
    }
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
    user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityType.eq("page"))
        .filter(user_favorites::Column::EntityIdentifier.eq(page_id))
        .exec(&state.db).await.map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

// âââ GET /workspaces/{slug}/projects/{project_id}/pages-summary/ âââââââââââââ
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