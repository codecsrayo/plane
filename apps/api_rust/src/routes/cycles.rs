// src/routes/cycles.rs
//! Endpoints de Cycles.
//!
//! Endpoints implementados:
//!   GET    /api/workspaces/{slug}/projects/{project_id}/cycles/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/cycles/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/cycles/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/cycles/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/cycles/{pk}/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}/

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
    entities::{cycle_issues, cycles, issues},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CycleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub end_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub status: String,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub owned_by_id: Uuid,
    pub archived_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub sort_order: f64,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl CycleResponse {
    fn from_model(m: cycles::Model) -> Self {
        // Status derivado de fechas — refleja la lógica de Django CycleViewSet
        let now = chrono::Utc::now();
        let status = match (m.start_date, m.end_date) {
            (None, _) | (_, None) => "draft",
            (Some(start), Some(end)) => {
                let start_utc = start.with_timezone(&chrono::Utc);
                let end_utc = end.with_timezone(&chrono::Utc);
                if now < start_utc {
                    "upcoming"
                } else if now > end_utc {
                    "completed"
                } else {
                    "started"
                }
            }
        }
        .to_owned();

        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            start_date: m.start_date,
            end_date: m.end_date,
            status,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            owned_by_id: m.owned_by_id,
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
pub struct CycleIssueResponse {
    pub id: Uuid,
    pub cycle_id: Uuid,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCycleRequest {
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub end_date: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCycleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub end_date: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddCycleIssuesRequest {
    pub issues: Vec<Uuid>,
}

// ── GET /workspaces/{slug}/projects/{project_id}/cycles/ ─────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Lista de ciclos"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_cycles(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<CycleResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = cycles::Entity::find()
        .active()
        .filter(cycles::Column::ProjectId.eq(guard.project.id))
        .filter(cycles::Column::ArchivedAt.is_null())
        .order_by_asc(cycles::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(CycleResponse::from_model).collect()))
}

// ── POST /workspaces/{slug}/projects/{project_id}/cycles/ ────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Ciclo creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateCycleRequest>,
) -> Result<(StatusCode, Json<CycleResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    // Validar que start_date < end_date si ambos están presentes
    if let (Some(start), Some(end)) = (body.start_date, body.end_date) {
        if start >= end {
            return Err(AppError::BadRequest(
                "start_date debe ser anterior a end_date".into(),
            ));
        }
    }

    // created_at / updated_at explícitos: cycles::ActiveModelBehavior está
    // vacío y la columna es NOT NULL sin DEFAULT. Mismo patrón que
    // labels.rs / issues.rs.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let cycle = cycles::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or_default()),
        start_date: Set(body.start_date),
        end_date: Set(body.end_date),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        owned_by_id: Set(guard.user.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        sort_order: Set(65535.0),
        view_props: Set(serde_json::json!({})),
        progress_snapshot: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        timezone: Set("UTC".to_owned()),
        version: Set(2),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(CycleResponse::from_model(cycle))))
}

// ── GET /workspaces/{slug}/projects/{project_id}/cycles/{pk}/ ────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{pk}/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Detalle del ciclo"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<CycleResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let cycle = cycles::Entity::find_by_id(pk)
        .active()
        .filter(cycles::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(CycleResponse::from_model(cycle)))
}

// ── PATCH /workspaces/{slug}/projects/{project_id}/cycles/{pk}/ ──────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{pk}/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Ciclo actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateCycleRequest>,
) -> Result<Json<CycleResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let cycle = cycles::Entity::find_by_id(pk)
        .active()
        .filter(cycles::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: cycles::ActiveModel = cycle.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    if body.start_date.is_some() {
        am.start_date = Set(body.start_date);
    }
    if body.end_date.is_some() {
        am.end_date = Set(body.end_date);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(CycleResponse::from_model(updated)))
}

// ── DELETE /workspaces/{slug}/projects/{project_id}/cycles/{pk}/ ─────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{pk}/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let cycle = cycles::Entity::find_by_id(pk)
        .active()
        .filter(cycles::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: cycles::ActiveModel = cycle.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /cycles/{cycle_id}/cycle-issues/ ─────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Issues del ciclo"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_cycle_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<Vec<CycleIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Verificar que el ciclo pertenece al proyecto
    let _ = cycles::Entity::find_by_id(cycle_id)
        .active()
        .filter(cycles::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let rows = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::CycleId.eq(cycle_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|ci| CycleIssueResponse {
                id: ci.id,
                cycle_id: ci.cycle_id,
                issue_id: ci.issue_id,
                project_id: ci.project_id,
                workspace_id: ci.workspace_id,
                created_by_id: ci.created_by_id,
            })
            .collect(),
    ))
}

// ── POST /cycles/{cycle_id}/cycle-issues/ ────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Issues agregados al ciclo"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn add_issues_to_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<AddCycleIssuesRequest>,
) -> Result<Json<Vec<CycleIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let _ = cycles::Entity::find_by_id(cycle_id)
        .active()
        .filter(cycles::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let mut created = Vec::new();

    for issue_id in body.issues {
        // Verificar que el issue existe y pertenece al proyecto
        let issue_exists = issues::Entity::find_by_id(issue_id)
            .active()
            .filter(issues::Column::ProjectId.eq(project_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if issue_exists.is_none() {
            continue;
        }

        // Idempotente: si ya existe (incluso soft-deleted), resucitar o ignorar
        let existing = cycle_issues::Entity::find()
            .filter(cycle_issues::Column::CycleId.eq(cycle_id))
            .filter(cycle_issues::Column::IssueId.eq(issue_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

        let ci = match existing {
            Some(ci) if ci.deleted_at.is_none() => ci, // ya existe activo
            Some(ci) => {
                // Resucitar soft-deleted
                let mut am: cycle_issues::ActiveModel = ci.into();
                am.deleted_at = Set(None);
                am.updated_by_id = Set(Some(user_id));
                // Django: TimeAuditModel auto_now=True.
                am.updated_at = Set(now);
                am.update(&state.db).await.map_err(AppError::Database)?
            }
            None => {
                // created_at/updated_at explícitos (cycle_issues NOT NULL sin
                // DEFAULT; ActiveModelBehavior vacío).
                cycle_issues::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    cycle_id: Set(cycle_id),
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

        created.push(CycleIssueResponse {
            id: ci.id,
            cycle_id: ci.cycle_id,
            issue_id: ci.issue_id,
            project_id: ci.project_id,
            workspace_id: ci.workspace_id,
            created_by_id: ci.created_by_id,
        });
    }

    Ok(Json(created))
}

// ── DELETE /cycles/{cycle_id}/cycle-issues/{issue_id}/ ───────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
        ("issue_id" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 204, description = "Issue removido del ciclo"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn remove_issue_from_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id, issue_id)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let ci = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::CycleId.eq(cycle_id))
        .filter(cycle_issues::Column::IssueId.eq(issue_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: cycle_issues::ActiveModel = ci.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
