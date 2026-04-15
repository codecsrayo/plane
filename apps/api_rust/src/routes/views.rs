// src/routes/views.rs
//! Endpoints de vistas de issues (project views y workspace views).
//!
//! Equivalente a `plane/app/views/view/base.py` en Django.
//!
//! Rutas implementadas:
//!   GET    /api/workspaces/{slug}/views/
//!   POST   /api/workspaces/{slug}/views/
//!   GET    /api/workspaces/{slug}/views/{pk}/
//!   PATCH  /api/workspaces/{slug}/views/{pk}/
//!   DELETE /api/workspaces/{slug}/views/{pk}/
//!
//!   GET    /api/workspaces/{slug}/projects/{project_id}/views/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/views/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/views/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/views/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/views/{pk}/
//!
//!   POST   /api/workspaces/{slug}/projects/{project_id}/user-favorite-views/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
        permissions::{require_role, ROLE_ADMIN, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{issue_views, user_favorites},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Constantes ────────────────────────────────────────────────────────────────

/// `access = 1` = public en Django IssueView.
const ACCESS_PUBLIC: i16 = 1;

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueViewResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub query: serde_json::Value,
    pub filters: serde_json::Value,
    pub display_filters: serde_json::Value,
    pub display_properties: serde_json::Value,
    pub logo_props: serde_json::Value,
    pub rich_filters: serde_json::Value,
    pub access: i16,
    pub is_locked: bool,
    pub sort_order: f64,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub owned_by_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    /// Solo presente en project views — indica si el usuario la marcó como favorita.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateViewRequest {
    pub name: String,
    pub description: Option<String>,
    pub query: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub display_filters: Option<serde_json::Value>,
    pub display_properties: Option<serde_json::Value>,
    pub logo_props: Option<serde_json::Value>,
    pub rich_filters: Option<serde_json::Value>,
    /// 0 = privado, 1 = público. Defecto: 0.
    pub access: Option<i16>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateViewRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub display_filters: Option<serde_json::Value>,
    pub display_properties: Option<serde_json::Value>,
    pub logo_props: Option<serde_json::Value>,
    pub rich_filters: Option<serde_json::Value>,
    pub access: Option<i16>,
    pub sort_order: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddFavoriteRequest {
    /// ID de la view a favoritar.
    pub view: Uuid,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn view_to_response(v: &issue_views::Model, is_favorite: Option<bool>) -> IssueViewResponse {
    IssueViewResponse {
        id: v.id,
        name: v.name.clone(),
        description: v.description.clone(),
        query: v.query.clone(),
        filters: v.filters.clone(),
        display_filters: v.display_filters.clone(),
        display_properties: v.display_properties.clone(),
        logo_props: v.logo_props.clone(),
        rich_filters: v.rich_filters.clone(),
        access: v.access,
        is_locked: v.is_locked,
        sort_order: v.sort_order,
        project_id: v.project_id,
        workspace_id: v.workspace_id,
        owned_by_id: v.owned_by_id,
        created_by_id: v.created_by_id,
        updated_by_id: v.updated_by_id,
        created_at: v.created_at,
        updated_at: v.updated_at,
        is_favorite,
    }
}

/// Aplica soft-delete a una view y limpia sus favoritos.
async fn soft_delete_view(
    db: &sea_orm::DatabaseConnection,
    view: issue_views::Model,
) -> Result<(), AppError> {
    let view_id = view.id;
    let txn = db.begin().await.map_err(AppError::Database)?;

    // Soft-delete la view
    let mut am: issue_views::ActiveModel = view.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&txn).await.map_err(AppError::Database)?;

    // Hard-delete favoritos asociados (user_favorites no tiene soft-delete relevante aquí)
    user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::EntityIdentifier.eq(view_id))
        .filter(user_favorites::Column::EntityType.eq("view"))
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;
    Ok(())
}

// ── Workspace Views ───────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/views/
///
/// Lista las views del workspace visibles para el usuario:
/// propias + públicas (access = 1).
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/views/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of workspace views"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_workspace_views(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    let user_id = guard.user.id;
    let workspace_id = guard.workspace.id;

    // Guests solo ven sus propias views
    let views = if guard.member.role <= ROLE_GUEST {
        issue_views::Entity::find()
            .active()
            .filter(issue_views::Column::WorkspaceId.eq(workspace_id))
            .filter(issue_views::Column::ProjectId.is_null())
            .filter(issue_views::Column::OwnedById.eq(user_id))
            .order_by_desc(issue_views::Column::CreatedAt)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        issue_views::Entity::find()
            .active()
            .filter(issue_views::Column::WorkspaceId.eq(workspace_id))
            .filter(issue_views::Column::ProjectId.is_null())
            .filter(
                Condition::any()
                    .add(issue_views::Column::OwnedById.eq(user_id))
                    .add(issue_views::Column::Access.eq(ACCESS_PUBLIC)),
            )
            .order_by_desc(issue_views::Column::CreatedAt)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    let resp: Vec<IssueViewResponse> = views.iter().map(|v| view_to_response(v, None)).collect();
    Ok(Json(resp))
}

/// POST /api/workspaces/{slug}/views/
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/views/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "View created"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn create_workspace_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateViewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 255 {
        return Err(AppError::BadRequest(
            "name must be between 1 and 255 characters".into(),
        ));
    }

    let access = body.access.unwrap_or(0).clamp(0, 1);

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_view = issue_views::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        description: Set(body.description.unwrap_or_default()),
        query: Set(body.query.unwrap_or(serde_json::json!({}))),
        filters: Set(body.filters.unwrap_or(serde_json::json!({}))),
        display_filters: Set(body.display_filters.unwrap_or(serde_json::json!({}))),
        display_properties: Set(body.display_properties.unwrap_or(serde_json::json!({}))),
        logo_props: Set(body.logo_props.unwrap_or(serde_json::json!({}))),
        rich_filters: Set(body.rich_filters.unwrap_or(serde_json::json!({}))),
        access: Set(access),
        is_locked: Set(false),
        sort_order: Set(65535.0),
        project_id: Set(None),
        workspace_id: Set(guard.workspace.id),
        owned_by_id: Set(guard.user.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        archived_at: Set(None),
    };

    let created = new_view.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(view_to_response(&created, None))))
}

/// GET /api/workspaces/{slug}/views/{pk}/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/views/{pk}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 200, description = "View"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_workspace_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let view = issue_views::Entity::find_by_id(pk)
        .active()
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issue_views::Column::ProjectId.is_null())
        .filter(
            Condition::any()
                .add(issue_views::Column::OwnedById.eq(guard.user.id))
                .add(issue_views::Column::Access.eq(ACCESS_PUBLIC)),
        )
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(view_to_response(&view, None)))
}

/// PATCH /api/workspaces/{slug}/views/{pk}/
#[utoipa::path(
    patch,
    path = "/workspaces/{slug}/views/{pk}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 200, description = "Updated view"),
        (status = 400, description = "Locked or not owner"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_workspace_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateViewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let view = issue_views::Entity::find_by_id(pk)
        .active()
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issue_views::Column::ProjectId.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if view.is_locked {
        return Err(AppError::BadRequest("view is locked".into()));
    }

    if view.owned_by_id != guard.user.id {
        return Err(AppError::BadRequest(
            "Only the owner of the view can update the view".into(),
        ));
    }

    let mut am: issue_views::ActiveModel = view.into();
    am.updated_at = Set(Utc::now().into());
    am.updated_by_id = Set(Some(guard.user.id));

    if let Some(v) = body.name {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() || trimmed.len() > 255 {
            return Err(AppError::BadRequest("name must be 1–255 characters".into()));
        }
        am.name = Set(trimmed);
    }
    if let Some(v) = body.description {
        am.description = Set(v);
    }
    if let Some(v) = body.query {
        am.query = Set(v);
    }
    if let Some(v) = body.filters {
        am.filters = Set(v);
    }
    if let Some(v) = body.display_filters {
        am.display_filters = Set(v);
    }
    if let Some(v) = body.display_properties {
        am.display_properties = Set(v);
    }
    if let Some(v) = body.logo_props {
        am.logo_props = Set(v);
    }
    if let Some(v) = body.rich_filters {
        am.rich_filters = Set(v);
    }
    if let Some(v) = body.access {
        am.access = Set(v.clamp(0, 1));
    }
    if let Some(v) = body.sort_order {
        am.sort_order = Set(v);
    }

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(view_to_response(&updated, None)))
}

/// DELETE /api/workspaces/{slug}/views/{pk}/
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/views/{pk}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, description = "Not owner or not admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_workspace_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let view = issue_views::Entity::find_by_id(pk)
        .active()
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issue_views::Column::ProjectId.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Solo admin del workspace o dueño puede eliminar
    let is_admin = guard.member.role >= ROLE_ADMIN;
    let is_owner = view.owned_by_id == guard.user.id;

    if !is_admin && !is_owner {
        return Err(AppError::BadRequest(
            "Only admin or owner can delete the view".into(),
        ));
    }

    soft_delete_view(&state.db, view).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Project Views ─────────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/projects/{project_id}/views/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/views/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of project views"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_project_views(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    let user_id = guard.user.id;
    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;

    // Guests con guest_view_all_features=false ven solo sus propias views
    let guest_restricted = guard.project_member.role <= ROLE_GUEST
        && !guard.project.guest_view_all_features;

    let views = if guest_restricted {
        issue_views::Entity::find()
            .active()
            .filter(issue_views::Column::WorkspaceId.eq(workspace_id))
            .filter(issue_views::Column::ProjectId.eq(project_id))
            .filter(issue_views::Column::OwnedById.eq(user_id))
            .order_by_asc(issue_views::Column::Name)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        issue_views::Entity::find()
            .active()
            .filter(issue_views::Column::WorkspaceId.eq(workspace_id))
            .filter(issue_views::Column::ProjectId.eq(project_id))
            .filter(
                Condition::any()
                    .add(issue_views::Column::OwnedById.eq(user_id))
                    .add(issue_views::Column::Access.eq(ACCESS_PUBLIC)),
            )
            .order_by_asc(issue_views::Column::Name)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    // Cargar favoritos del usuario para este proyecto
    let view_ids: Vec<Uuid> = views.iter().map(|v| v.id).collect();
    let favorites = if view_ids.is_empty() {
        vec![]
    } else {
        user_favorites::Entity::find()
            .active()
            .filter(user_favorites::Column::UserId.eq(user_id))
            .filter(user_favorites::Column::EntityType.eq("view"))
            .filter(user_favorites::Column::ProjectId.eq(project_id))
            .filter(user_favorites::Column::WorkspaceId.eq(workspace_id))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    };

    let fav_ids: std::collections::HashSet<Uuid> = favorites
        .iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    let resp: Vec<IssueViewResponse> = views
        .iter()
        .map(|v| view_to_response(v, Some(fav_ids.contains(&v.id))))
        .collect();

    Ok(Json(resp))
}

/// POST /api/workspaces/{slug}/projects/{project_id}/views/
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/views/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "View created"),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_project_view(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateViewRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 255 {
        return Err(AppError::BadRequest(
            "name must be between 1 and 255 characters".into(),
        ));
    }

    let access = body.access.unwrap_or(0).clamp(0, 1);
    let now: DateTime<FixedOffset> = Utc::now().into();

    let new_view = issue_views::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        description: Set(body.description.unwrap_or_default()),
        query: Set(body.query.unwrap_or(serde_json::json!({}))),
        filters: Set(body.filters.unwrap_or(serde_json::json!({}))),
        display_filters: Set(body.display_filters.unwrap_or(serde_json::json!({}))),
        display_properties: Set(body.display_properties.unwrap_or(serde_json::json!({}))),
        logo_props: Set(body.logo_props.unwrap_or(serde_json::json!({}))),
        rich_filters: Set(body.rich_filters.unwrap_or(serde_json::json!({}))),
        access: Set(access),
        is_locked: Set(false),
        sort_order: Set(65535.0),
        project_id: Set(Some(guard.project.id)),
        workspace_id: Set(guard.workspace.id),
        owned_by_id: Set(guard.user.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        archived_at: Set(None),
    };

    let created = new_view.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((
        StatusCode::CREATED,
        Json(view_to_response(&created, Some(false))),
    ))
}

/// GET /api/workspaces/{slug}/projects/{project_id}/views/{pk}/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/views/{pk}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 200, description = "View"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_project_view(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    let view = issue_views::Entity::find_by_id(pk)
        .active()
        .filter(issue_views::Column::ProjectId.eq(project_id))
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Guest sin guest_view_all_features solo puede ver sus propias views
    let guest_restricted = guard.project_member.role <= ROLE_GUEST
        && !guard.project.guest_view_all_features;

    if guest_restricted && view.owned_by_id != user_id {
        return Err(AppError::Forbidden);
    }

    let is_fav = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::EntityType.eq("view"))
        .filter(user_favorites::Column::EntityIdentifier.eq(pk))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    Ok(Json(view_to_response(&view, Some(is_fav))))
}

/// PATCH /api/workspaces/{slug}/projects/{project_id}/views/{pk}/
#[utoipa::path(
    patch,
    path = "/workspaces/{slug}/projects/{project_id}/views/{pk}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 200, description = "Updated view"),
        (status = 400, description = "Locked or not owner"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_project_view(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateViewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let view = issue_views::Entity::find_by_id(pk)
        .active()
        .filter(issue_views::Column::ProjectId.eq(guard.project.id))
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if view.is_locked {
        return Err(AppError::BadRequest("view is locked".into()));
    }

    if view.owned_by_id != guard.user.id {
        return Err(AppError::BadRequest(
            "Only the owner of the view can update the view".into(),
        ));
    }

    let mut am: issue_views::ActiveModel = view.into();
    am.updated_at = Set(Utc::now().into());
    am.updated_by_id = Set(Some(guard.user.id));

    if let Some(v) = body.name {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() || trimmed.len() > 255 {
            return Err(AppError::BadRequest("name must be 1–255 characters".into()));
        }
        am.name = Set(trimmed);
    }
    if let Some(v) = body.description {
        am.description = Set(v);
    }
    if let Some(v) = body.query {
        am.query = Set(v);
    }
    if let Some(v) = body.filters {
        am.filters = Set(v);
    }
    if let Some(v) = body.display_filters {
        am.display_filters = Set(v);
    }
    if let Some(v) = body.display_properties {
        am.display_properties = Set(v);
    }
    if let Some(v) = body.logo_props {
        am.logo_props = Set(v);
    }
    if let Some(v) = body.rich_filters {
        am.rich_filters = Set(v);
    }
    if let Some(v) = body.access {
        am.access = Set(v.clamp(0, 1));
    }
    if let Some(v) = body.sort_order {
        am.sort_order = Set(v);
    }

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(view_to_response(&updated, None)))
}

/// DELETE /api/workspaces/{slug}/projects/{project_id}/views/{pk}/
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/projects/{project_id}/views/{pk}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_project_view(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let view = issue_views::Entity::find_by_id(pk)
        .active()
        .filter(issue_views::Column::ProjectId.eq(guard.project.id))
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let is_admin = guard.project_member.role >= ROLE_ADMIN
        || guard.workspace_member.role >= ROLE_ADMIN;
    let is_owner = view.owned_by_id == guard.user.id;

    if !is_admin && !is_owner {
        return Err(AppError::BadRequest(
            "Only admin or owner can delete the view".into(),
        ));
    }

    soft_delete_view(&state.db, view).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── View Favorites ────────────────────────────────────────────────────────────

/// POST /api/workspaces/{slug}/projects/{project_id}/user-favorite-views/
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/user-favorite-views/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 204, description = "Added to favorites"),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn add_favorite_view(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<AddFavoriteRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    // Verificar que la view existe y es accesible
    let view_exists = issue_views::Entity::find_by_id(body.view)
        .active()
        .filter(issue_views::Column::ProjectId.eq(guard.project.id))
        .filter(issue_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if view_exists.is_none() {
        return Err(AppError::NotFound);
    }

    // Evitar duplicados — idempotente
    let already_fav = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityType.eq("view"))
        .filter(user_favorites::Column::EntityIdentifier.eq(body.view))
        .filter(user_favorites::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if already_fav.is_none() {
        let now: DateTime<FixedOffset> = Utc::now().into();
        let fav = user_favorites::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set("view".to_string()),
            entity_identifier: Set(Some(body.view)),
            name: Set(None),
            is_folder: Set(false),
            sequence: Set(65535.0),
            project_id: Set(Some(guard.project.id)),
            workspace_id: Set(guard.workspace.id),
            user_id: Set(guard.user.id),
            parent_id: Set(None),
            created_by_id: Set(Some(guard.user.id)),
            updated_by_id: Set(Some(guard.user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        fav.insert(&state.db).await.map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}/
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}/",
    tag = "Views",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("view_id" = Uuid, Path, description = "View ID"),
    ),
    responses(
        (status = 204, description = "Removed from favorites"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn remove_favorite_view(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, view_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let fav = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityType.eq("view"))
        .filter(user_favorites::Column::EntityIdentifier.eq(view_id))
        .filter(user_favorites::Column::ProjectId.eq(guard.project.id))
        .filter(user_favorites::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Hard-delete del favorito (no tiene soft-delete semántico aquí)
    let fav_id = fav.id;
    user_favorites::Entity::delete_by_id(fav_id)
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
