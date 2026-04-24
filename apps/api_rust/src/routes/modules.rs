// src/routes/modules.rs
//! Endpoints de Modules.
//!
//! Endpoints implementados:
//!   GET    /api/workspaces/{slug}/projects/{project_id}/modules/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/modules/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/modules/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/modules/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/modules/{pk}/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
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
    entities::{issues, module_issues, module_links, module_user_properties, modules, user_favorites},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub lead_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub archived_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub sort_order: f64,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl ModuleResponse {
    fn from_model(m: modules::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            status: m.status,
            start_date: m.start_date,
            target_date: m.target_date,
            lead_id: m.lead_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            archived_at: m.archived_at,
            sort_order: m.sort_order,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleIssueResponse {
    pub id: Uuid,
    pub module_id: Uuid,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateModuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub lead_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateModuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub lead_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddModuleIssuesRequest {
    pub issues: Vec<Uuid>,
}

/// Valores válidos de status para un módulo.
const VALID_MODULE_STATUSES: &[&str] =
    &["backlog", "in-progress", "paused", "completed", "cancelled"];

// ── GET /workspaces/{slug}/projects/{project_id}/modules/ ────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Lista de módulos"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_modules(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<ModuleResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = modules::Entity::find()
        .active()
        .filter(modules::Column::ProjectId.eq(guard.project.id))
        .filter(modules::Column::ArchivedAt.is_null())
        .order_by_asc(modules::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(ModuleResponse::from_model).collect()))
}

// ── POST /workspaces/{slug}/projects/{project_id}/modules/ ───────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Módulo creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateModuleRequest>,
) -> Result<(StatusCode, Json<ModuleResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    let status = body
        .status
        .as_deref()
        .unwrap_or("backlog")
        .to_owned();

    if !VALID_MODULE_STATUSES.contains(&status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "status inválido '{}'. Valores permitidos: {}",
            status,
            VALID_MODULE_STATUSES.join(", ")
        )));
    }

    // created_at/updated_at explícitos: modules::ActiveModelBehavior vacío,
    // columnas NOT NULL sin DEFAULT (ver baseline.sql:1724-1726). Mismo
    // patrón que labels.rs / issues.rs / cycles.rs.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let module = modules::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or_default()),
        status: Set(status),
        start_date: Set(body.start_date),
        target_date: Set(body.target_date),
        lead_id: Set(body.lead_id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        sort_order: Set(65535.0),
        view_props: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(ModuleResponse::from_model(module))))
}

// ── GET /workspaces/{slug}/projects/{project_id}/modules/{pk}/ ───────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Detalle del módulo"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<ModuleResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let module = modules::Entity::find_by_id(pk)
        .active()
        .filter(modules::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ModuleResponse::from_model(module)))
}

// ── PATCH /workspaces/{slug}/projects/{project_id}/modules/{pk}/ ─────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Módulo actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateModuleRequest>,
) -> Result<Json<ModuleResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let module = modules::Entity::find_by_id(pk)
        .active()
        .filter(modules::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: modules::ActiveModel = module.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    if let Some(status) = body.status {
        if !VALID_MODULE_STATUSES.contains(&status.as_str()) {
            return Err(AppError::BadRequest(format!(
                "status inválido '{status}'. Valores permitidos: {}",
                VALID_MODULE_STATUSES.join(", ")
            )));
        }
        am.status = Set(status);
    }
    if body.start_date.is_some() {
        am.start_date = Set(body.start_date);
    }
    if body.target_date.is_some() {
        am.target_date = Set(body.target_date);
    }
    if body.lead_id.is_some() {
        am.lead_id = Set(body.lead_id);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ModuleResponse::from_model(updated)))
}

// ── DELETE /workspaces/{slug}/projects/{project_id}/modules/{pk}/ ────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let module = modules::Entity::find_by_id(pk)
        .active()
        .filter(modules::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: modules::ActiveModel = module.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /modules/{module_id}/issues/ ────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Issues del módulo"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_module_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<Vec<ModuleIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let _ = modules::Entity::find_by_id(module_id)
        .active()
        .filter(modules::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let rows = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::ModuleId.eq(module_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|mi| ModuleIssueResponse {
                id: mi.id,
                module_id: mi.module_id,
                issue_id: mi.issue_id,
                project_id: mi.project_id,
                workspace_id: mi.workspace_id,
                created_by_id: mi.created_by_id,
            })
            .collect(),
    ))
}

// ── POST /modules/{module_id}/issues/ ────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Issues agregados al módulo"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn add_issues_to_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<AddModuleIssuesRequest>,
) -> Result<Json<Vec<ModuleIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let _ = modules::Entity::find_by_id(module_id)
        .active()
        .filter(modules::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let mut created = Vec::new();

    for issue_id in body.issues {
        let issue_exists = issues::Entity::find_by_id(issue_id)
            .active()
            .filter(issues::Column::ProjectId.eq(project_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if issue_exists.is_none() {
            continue;
        }

        let existing = module_issues::Entity::find()
            .filter(module_issues::Column::ModuleId.eq(module_id))
            .filter(module_issues::Column::IssueId.eq(issue_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

        let mi = match existing {
            Some(mi) if mi.deleted_at.is_none() => mi,
            Some(mi) => {
                let mut am: module_issues::ActiveModel = mi.into();
                am.deleted_at = Set(None);
                am.updated_by_id = Set(Some(user_id));
                // Django: TimeAuditModel auto_now=True.
                am.updated_at = Set(now);
                am.update(&state.db).await.map_err(AppError::Database)?
            }
            None => {
                // created_at/updated_at explícitos (NOT NULL sin DEFAULT).
                module_issues::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    module_id: Set(module_id),
                    issue_id: Set(issue_id),
                    project_id: Set(project_id),
                    workspace_id: Set(workspace_id),
                    created_by_id: Set(Some(user_id)),
                    updated_by_id: Set(Some(user_id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                    ..Default::default()
                }
                .insert(&state.db)
                .await
                .map_err(AppError::Database)?
            }
        };

        created.push(ModuleIssueResponse {
            id: mi.id,
            module_id: mi.module_id,
            issue_id: mi.issue_id,
            project_id: mi.project_id,
            workspace_id: mi.workspace_id,
            created_by_id: mi.created_by_id,
        });
    }

    Ok(Json(created))
}

// ── DELETE /modules/{module_id}/issues/{issue_id}/ ───────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 204, description = "Issue removido del módulo"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn remove_issue_from_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id, issue_id)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let mi = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::ModuleId.eq(module_id))
        .filter(module_issues::Column::IssueId.eq(issue_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: module_issues::ActiveModel = mi.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═════════════════════════════════════════════════════════════════════════════
// Module User Properties
// ═════════════════════════════════════════════════════════════════════════════
//
// Mirror de `ModuleUserPropertiesEndpoint` en
// apps/api/plane/app/views/module/base.py:825-855.
//
// Semántica clave (paridad Django):
//   - GET hace `get_or_create` → NUNCA devuelve 404 por ausencia de fila.
//     Si no existe, se crea con defaults y se devuelve 200.
//   - PATCH: Django usa `.get(...)` crudo, que lanzaría 500 si faltara.
//     Para evitar ese fallo y ser más útil al frontend, aquí hacemos
//     `get_or_create` y aplicamos el patch encima — no degrada ningún
//     caso de uso válido. Mismo trade-off ya adoptado en cycles.rs.
//   - Permisos: ADMIN / MEMBER / GUEST (igual que Django,
//     `@allow_permission([ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST])`).
//
// Nota sobre el modelo: `ModuleUserProperties` (module.py:190-217) tiene los
// mismos campos que `CycleUserProperties`: filters, display_filters,
// display_properties, rich_filters. Sin `preferences` ni `sort_order`.

// ─── Defaults — mirror de `plane/db/models/module.py:14-55` ──────────────────

fn module_default_filters() -> serde_json::Value {
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

fn module_default_display_filters() -> serde_json::Value {
    serde_json::json!({
        "group_by": null,
        "order_by": "-created_at",
        "type": null,
        "sub_issue": true,
        "show_empty_groups": true,
        "layout": "list",
        "calendar_date_range": "",
    })
}

fn module_default_display_properties() -> serde_json::Value {
    serde_json::json!({
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
    })
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

/// Mirror de `ModuleUserPropertiesSerializer` (fields="__all__", read_only:
/// workspace/project/module/user). Incluye todos los campos del modelo
/// `ModuleUserProperties` de `apps/api/plane/db/models/module.py:190-217`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleUserPropertiesResponse {
    pub id: Uuid,
    pub user: Uuid,
    pub module: Uuid,
    pub project: Uuid,
    pub workspace: Uuid,
    pub filters: serde_json::Value,
    pub display_filters: serde_json::Value,
    pub display_properties: serde_json::Value,
    pub rich_filters: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl From<&module_user_properties::Model> for ModuleUserPropertiesResponse {
    fn from(m: &module_user_properties::Model) -> Self {
        Self {
            id: m.id,
            user: m.user_id,
            module: m.module_id,
            project: m.project_id,
            workspace: m.workspace_id,
            filters: m.filters.clone(),
            display_filters: m.display_filters.clone(),
            display_properties: m.display_properties.clone(),
            rich_filters: m.rich_filters.clone(),
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
        }
    }
}

/// Body admitido en PATCH. Todos los campos son opcionales — semántica
/// `partial=True` del serializer Django. Los campos read-only
/// (workspace/project/module/user) se ignoran si vienen en el body.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateModuleUserPropertiesRequest {
    pub filters: Option<serde_json::Value>,
    pub display_filters: Option<serde_json::Value>,
    pub display_properties: Option<serde_json::Value>,
    pub rich_filters: Option<serde_json::Value>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Busca o crea la fila `module_user_properties` para `(module, user)`.
/// Mirror de `ModuleUserProperties.objects.get_or_create(...)`.
///
/// Constraint único en Django: `(module, user)` WHERE `deleted_at IS NULL`
/// (`module.py:204-209`). Filtramos por `.active()`. En caso de INSERT
/// concurrente con violación del índice único, el error se propagaría como
/// `AppError::Database` y un reintento del cliente resolvería el caso —
/// mismo comportamiento que Django.
async fn get_or_create_module_user_properties(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
    module_id: Uuid,
    user_id: Uuid,
) -> Result<module_user_properties::Model, AppError> {
    if let Some(existing) = module_user_properties::Entity::find()
        .active()
        .filter(module_user_properties::Column::UserId.eq(user_id))
        .filter(module_user_properties::Column::ModuleId.eq(module_id))
        .filter(module_user_properties::Column::ProjectId.eq(project_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
    {
        return Ok(existing);
    }

    let now = chrono::Utc::now().fixed_offset();
    let created = module_user_properties::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        module_id: Set(module_id),
        project_id: Set(project_id),
        workspace_id: Set(workspace_id),
        filters: Set(module_default_filters()),
        display_filters: Set(module_default_display_filters()),
        display_properties: Set(module_default_display_properties()),
        rich_filters: Set(serde_json::json!({})),
        created_at: Set(now),
        updated_at: Set(now),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        deleted_at: Set(None),
    }
    .insert(db)
    .await
    .map_err(AppError::Database)?;

    Ok(created)
}

/// Asegura que el módulo existe y pertenece al workspace/proyecto del guard.
/// 404 si no existe o fue soft-deleted. Defensa en profundidad — el guard
/// sólo valida workspace + proyecto, no la pertenencia del `module_id`.
///
/// Sin esta validación, un cliente podría crear una fila
/// `module_user_properties` apuntando a un módulo de OTRO proyecto/workspace
/// y leer/modificar filtros privados cruzando límites de tenant.
async fn ensure_module_belongs_to_project(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
    module_id: Uuid,
) -> Result<modules::Model, AppError> {
    modules::Entity::find_by_id(module_id)
        .filter(modules::Column::WorkspaceId.eq(workspace_id))
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

// ─── GET ──────────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties/`
///
/// Paridad con `ModuleUserPropertiesEndpoint.get` (module/base.py:846-855).
/// `get_or_create` garantiza que nunca devolvemos 404 por ausencia de la
/// fila de propiedades — esto es lo que resuelve el 404 del frontend.
///
/// Permisos: ROLE_GUEST+ (ADMIN/MEMBER/GUEST, paridad Django).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties/",
    tag = "Modules",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("module_id"  = Uuid,   Path, description = "Module UUID"),
    ),
    responses(
        (status = 200, description = "Module user properties", body = ModuleUserPropertiesResponse),
        (status = 403, description = "User is not a member of the project"),
        (status = 404, description = "Workspace, project or module not found"),
    )
)]
pub async fn get_module_user_properties(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<ModuleUserPropertiesResponse>, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    // Validación: el módulo debe existir y pertenecer al proyecto. Si no,
    // 404 — evita crear una fila user-properties huérfana o cross-tenant.
    let _module = ensure_module_belongs_to_project(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        module_id,
    )
    .await?;

    let row = get_or_create_module_user_properties(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        module_id,
        guard.user.id,
    )
    .await?;

    Ok(Json(ModuleUserPropertiesResponse::from(&row)))
}

// ─── PATCH ────────────────────────────────────────────────────────────────────

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties/`
///
/// Paridad con `ModuleUserPropertiesEndpoint.patch` (module/base.py:826-844),
/// con una mejora: Django asume que la fila existe (`.objects.get(...)`) y
/// lanzaría 500 si faltara; aquí hacemos `get_or_create` antes del patch,
/// lo que es estrictamente más robusto.
///
/// Django devuelve 201 en PATCH (comportamiento no-idiomático heredado).
/// Mantenemos 200 aquí porque (a) no es un create, es un update, y (b) el
/// frontend de Plane no depende del código exacto — comprueba `>=200 <300`.
///
/// Permisos: ROLE_GUEST+ (paridad Django).
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties/",
    tag = "Modules",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("module_id"  = Uuid,   Path, description = "Module UUID"),
    ),
    request_body = UpdateModuleUserPropertiesRequest,
    responses(
        (status = 200, description = "Updated module user properties", body = ModuleUserPropertiesResponse),
        (status = 403, description = "User is not a member of the project"),
        (status = 404, description = "Workspace, project or module not found"),
    )
)]
pub async fn update_module_user_properties(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateModuleUserPropertiesRequest>,
) -> Result<Json<ModuleUserPropertiesResponse>, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    let _module = ensure_module_belongs_to_project(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        module_id,
    )
    .await?;

    let row = get_or_create_module_user_properties(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        module_id,
        guard.user.id,
    )
    .await?;

    let mut am: module_user_properties::ActiveModel = row.into();
    if let Some(v) = body.filters {
        am.filters = Set(v);
    }
    if let Some(v) = body.display_filters {
        am.display_filters = Set(v);
    }
    if let Some(v) = body.display_properties {
        am.display_properties = Set(v);
    }
    if let Some(v) = body.rich_filters {
        am.rich_filters = Set(v);
    }
    am.updated_at = Set(chrono::Utc::now().fixed_offset());
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ModuleUserPropertiesResponse::from(&updated)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS PENDIENTES
// ═══════════════════════════════════════════════════════════════════════════════

// ── POST /issues/{issue_id}/modules ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct IssueModulesRequest {
    pub modules: Option<Vec<Uuid>>,
    pub removed_modules: Option<Vec<Uuid>>,
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/modules/`
///
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/modules/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 200, description = "Modules updated for issue"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn set_issue_modules(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<IssueModulesRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let user_id = guard.user.id;
    let db = &state.db;

    // Agregar módulos
    if let Some(mods) = body.modules {
        for module_id in mods {
            // Idempotente: verificar si ya existe
            let existing = module_issues::Entity::find()
                .filter(module_issues::Column::IssueId.eq(issue_id))
                .filter(module_issues::Column::ModuleId.eq(module_id))
                .filter(module_issues::Column::ProjectId.eq(project_id))
                .filter(module_issues::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(AppError::Database)?;

            if existing.is_none() {
                let mi = module_issues::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    issue_id: Set(issue_id),
                    module_id: Set(module_id),
                    project_id: Set(project_id),
                    workspace_id: Set(ws_id),
                    created_by_id: Set(Some(user_id)),
                    updated_by_id: Set(Some(user_id)),
                    ..Default::default()
                };
                mi.insert(db).await.map_err(AppError::Database)?;
            }
        }
    }

    // Eliminar módulos (soft-delete)
    if let Some(removed) = body.removed_modules {
        for module_id in removed {
            if let Some(mi) = module_issues::Entity::find()
                .filter(module_issues::Column::IssueId.eq(issue_id))
                .filter(module_issues::Column::ModuleId.eq(module_id))
                .filter(module_issues::Column::ProjectId.eq(project_id))
                .filter(module_issues::Column::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(AppError::Database)?
            {
                let mut am: module_issues::ActiveModel = mi.into();
                am.deleted_at = Set(Some(chrono::Utc::now().into()));
                am.update(db).await.map_err(AppError::Database)?;
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── module-links ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ModuleLinkResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub url: String,
    pub module_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<module_links::Model> for ModuleLinkResponse {
    fn from(m: module_links::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            url: m.url,
            module_id: m.module_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateModuleLinkRequest {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModuleLinkRequest {
    pub url: Option<String>,
    pub title: Option<String>,
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/`
///
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "List of module links"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_module_links(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let links = module_links::Entity::find()
        .filter(module_links::Column::WorkspaceId.eq(ws_id))
        .filter(module_links::Column::ProjectId.eq(project_id))
        .filter(module_links::Column::ModuleId.eq(module_id))
        .filter(module_links::Column::DeletedAt.is_null())
        .order_by_desc(module_links::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<ModuleLinkResponse> = links.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/`
///
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 201, description = "Module link created"),
        (status = 400, description = "Invalid request"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_module_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateModuleLinkRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let user_id = guard.user.id;
    let db = &state.db;

    let link = module_links::ActiveModel {
        id: Set(Uuid::new_v4()),
        url: Set(body.url),
        title: Set(body.title),
        module_id: Set(module_id),
        project_id: Set(project_id),
        workspace_id: Set(ws_id),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        metadata: Set(serde_json::json!({})),
        ..Default::default()
    };

    let created = link.insert(db).await.map_err(AppError::Database)?;
    let resp: ModuleLinkResponse = created.into();
    Ok((StatusCode::CREATED, Json(resp)))
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}/`
///
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("pk" = Uuid, Path, description = "Link ID"),
    ),
    responses(
        (status = 200, description = "Module link"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_module_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    let project_id = guard.project.id;
    let db = &state.db;

    let link = module_links::Entity::find_by_id(pk)
        .filter(module_links::Column::ProjectId.eq(project_id))
        .filter(module_links::Column::ModuleId.eq(module_id))
        .filter(module_links::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let resp: ModuleLinkResponse = link.into();
    Ok(Json(resp))
}

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}/`
///
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("pk" = Uuid, Path, description = "Link ID"),
    ),
    responses(
        (status = 200, description = "Updated module link"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_module_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
    Json(body): Json<UpdateModuleLinkRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let project_id = guard.project.id;
    let user_id = guard.user.id;
    let db = &state.db;

    let link = module_links::Entity::find_by_id(pk)
        .filter(module_links::Column::ProjectId.eq(project_id))
        .filter(module_links::Column::ModuleId.eq(module_id))
        .filter(module_links::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: module_links::ActiveModel = link.into();
    if let Some(url) = body.url {
        active.url = Set(url);
    }
    if let Some(title) = body.title {
        active.title = Set(Some(title));
    }
    active.updated_by_id = Set(Some(user_id));
    let updated = active.update(db).await.map_err(AppError::Database)?;

    let resp: ModuleLinkResponse = updated.into();
    Ok(Json(resp))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}/`
///
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
        ("pk" = Uuid, Path, description = "Link ID"),
    ),
    responses(
        (status = 204, description = "Module link deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_module_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let project_id = guard.project.id;
    let db = &state.db;

    let link = module_links::Entity::find_by_id(pk)
        .filter(module_links::Column::ProjectId.eq(project_id))
        .filter(module_links::Column::ModuleId.eq(module_id))
        .filter(module_links::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: module_links::ActiveModel = link.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── user-favorite-modules ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FavoriteModuleRequest {
    pub module: Uuid,
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/user-favorite-modules/`
///
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-favorite-modules/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of favorite modules"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_favorite_modules(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let user_id = guard.user.id;
    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let favorites = user_favorites::Entity::find()
        .filter(user_favorites::Column::WorkspaceId.eq(ws_id))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::EntityType.eq("module"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<serde_json::Value> = favorites
        .into_iter()
        .map(|f| serde_json::json!({
            "id": f.id,
            "entity_type": f.entity_type,
            "entity_identifier": f.entity_identifier,
            "project_id": f.project_id,
            "workspace_id": f.workspace_id,
        }))
        .collect();

    Ok(Json(resp))
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/user-favorite-modules/`
///
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-favorite-modules/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 204, description = "Module added to favorites"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_favorite_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<FavoriteModuleRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let user_id = guard.user.id;
    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let existing = user_favorites::Entity::find()
        .filter(user_favorites::Column::WorkspaceId.eq(ws_id))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::EntityType.eq("module"))
        .filter(user_favorites::Column::EntityIdentifier.eq(body.module))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_none() {
        let new_fav = user_favorites::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set("module".to_string()),
            entity_identifier: Set(Some(body.module)),
            project_id: Set(Some(project_id)),
            workspace_id: Set(ws_id),
            user_id: Set(user_id),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            sequence: Set(65535.0_f64),
            is_folder: Set(false),
            ..Default::default()
        };
        new_fav.insert(db).await.map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/user-favorite-modules/{module_id}/`
///
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-favorite-modules/{module_id}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 204, description = "Module removed from favorites"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_favorite_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let user_id = guard.user.id;
    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let fav = user_favorites::Entity::find()
        .filter(user_favorites::Column::WorkspaceId.eq(ws_id))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::EntityType.eq("module"))
        .filter(user_favorites::Column::EntityIdentifier.eq(module_id))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let active: user_favorites::ActiveModel = fav.into();
    active.delete(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── archive / unarchive module ────────────────────────────────────────────────

/// `POST /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive/`
///
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Module archived"),
        (status = 400, description = "Invalid state for archiving"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn archive_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let module = modules::Entity::find_by_id(module_id)
        .filter(modules::Column::WorkspaceId.eq(ws_id))
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Solo módulos completados o cancelados pueden archivarse
    if !["completed", "cancelled"].contains(&module.status.as_str()) {
        return Err(AppError::BadRequest(
            "Only completed or cancelled modules can be archived".into(),
        ));
    }

    let archived_at = chrono::Utc::now().fixed_offset();
    let mut active: modules::ActiveModel = module.into();
    active.archived_at = Set(Some(archived_at));
    active.update(db).await.map_err(AppError::Database)?;

    // Eliminar de favoritos
    let _ = user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::EntityType.eq("module"))
        .filter(user_favorites::Column::EntityIdentifier.eq(module_id))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .filter(user_favorites::Column::WorkspaceId.eq(ws_id))
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "archived_at": archived_at.to_rfc3339() })))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive/`
///
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("module_id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Module unarchived"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn unarchive_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, module_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let module = modules::Entity::find_by_id(module_id)
        .filter(modules::Column::WorkspaceId.eq(ws_id))
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: modules::ActiveModel = module.into();
    active.archived_at = Set(None);
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── archived-modules ──────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/archived-modules/`
///
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/archived-modules/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of archived modules"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_archived_modules(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let mods = modules::Entity::find()
        .filter(modules::Column::WorkspaceId.eq(ws_id))
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::DeletedAt.is_null())
        .filter(modules::Column::ArchivedAt.is_not_null())
        .order_by_desc(modules::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<ModuleResponse> = mods.into_iter().map(ModuleResponse::from_model).collect();
    Ok(Json(resp))
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/archived-modules/{pk}/`
///
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/archived-modules/{pk}/",
    tag = "Modules",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Archived module"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_archived_module(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let module = modules::Entity::find_by_id(pk)
        .filter(modules::Column::WorkspaceId.eq(ws_id))
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::DeletedAt.is_null())
        .filter(modules::Column::ArchivedAt.is_not_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ModuleResponse::from_model(module)))
}
