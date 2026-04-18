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
    entities::{module_issues, modules, issues},
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
