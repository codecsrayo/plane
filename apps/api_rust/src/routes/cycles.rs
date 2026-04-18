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
//!   GET    /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder, Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{cycle_issues, cycle_user_properties, cycles, issues},
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

// ═════════════════════════════════════════════════════════════════════════════
// Cycle Analytics
// ═════════════════════════════════════════════════════════════════════════════
//
// Mirror de `CycleAnalyticsEndpoint` en
// apps/api/plane/app/views/cycle/base.py:786, incluyendo el helper
// `burndown_plot` en apps/api/plane/utils/analytics_plot.py.

#[derive(Debug, Deserialize)]
pub struct CycleAnalyticsQuery {
    /// "issues" (default) o "points".
    #[serde(rename = "type")]
    pub analytic_type: Option<String>,
}

/// GET /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics/
///
/// Analytics del ciclo: distribución de issues por assignee y por label, más
/// el burndown chart acumulado. Mirror exacto de `CycleAnalyticsEndpoint.get`.
///
/// Comportamiento:
/// - Si el ciclo no tiene `start_date` o `end_date` → 400.
/// - Si el ciclo tiene `progress_snapshot` no vacío → early return con los
///   datos snapshot (el ciclo está cerrado y los issues fueron transferidos).
/// - Si `type=points` y el proyecto no tiene estimate de tipo "points" →
///   distribuciones vacías, chart vacío (paridad Django).
/// - Si `type=issues` → counts de issues por assignee/label + burndown por issues.
/// - Si `type=points` + estimate points → sum de estimate_point.value por
///   assignee/label + burndown por puntos.
///
/// Permisos: ADMIN / MEMBER / GUEST (paridad con Django).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
        ("type" = Option<String>, Query, description = "'issues' (default) o 'points'"),
    ),
    responses(
        (status = 200, description = "Analytics del ciclo"),
        (status = 400, description = "Ciclo sin fechas"),
        (status = 403, description = "No autorizado"),
        (status = 404, description = "Ciclo no encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn cycle_analytics(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
    Query(params): Query<CycleAnalyticsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Permisos: GUEST+ a nivel proyecto o ADMIN a nivel workspace (Django
    // permite GUEST). require_role además del guard = defensa en profundidad.
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;
    let analytic_type = params.analytic_type.as_deref().unwrap_or("issues");

    // ── 1. Fetch del ciclo (con filtros workspace+project como defensa extra) ──
    let cycle = cycles::Entity::find_by_id(cycle_id)
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // ── 2. Validación de fechas (paridad Django: 400 si falta alguna) ─────────
    let (start_dt, end_dt) = match (cycle.start_date, cycle.end_date) {
        (Some(s), Some(e)) => (s, e),
        _ => {
            return Err(AppError::BadRequest(
                "Cycle has no start or end date".into(),
            ));
        }
    };
    let start_date = start_dt.date_naive();
    let end_date = end_dt.date_naive();

    // ── 3. Progress snapshot: si existe y es un objeto no-vacío, early return ─
    //
    // Django: `if cycle.progress_snapshot:` — truthy check sobre un dict.
    // En Postgres la columna es NOT NULL con default `{}`, así que puede
    // llegar como objeto vacío (falsy en Python).
    if let serde_json::Value::Object(snapshot) = &cycle.progress_snapshot {
        if !snapshot.is_empty() {
            let distribution = snapshot
                .get("distribution")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            let labels = distribution
                .get("labels")
                .cloned()
                .unwrap_or(serde_json::json!([]));
            let assignees = distribution
                .get("assignees")
                .cloned()
                .unwrap_or(serde_json::json!([]));
            let completion_chart = distribution
                .get("completion_chart")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            return Ok(Json(serde_json::json!({
                "labels": labels,
                "assignees": assignees,
                "completion_chart": completion_chart,
            })));
        }
    }

    // ── 4. ¿El proyecto usa estimate con type="points"? ───────────────────────
    let estimate_type_points = {
        let sql = format!(
            "SELECT EXISTS (
               SELECT 1 FROM estimates e
               JOIN projects p ON p.estimate_id = e.id
               WHERE p.id = '{project_id}'
                 AND p.workspace_id = '{ws_id}'
                 AND e.\"type\" = 'points'
                 AND e.deleted_at IS NULL
             ) AS ex"
        );
        db.query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .map_err(AppError::Database)?
        .and_then(|r| r.try_get::<bool>("", "ex").ok())
        .unwrap_or(false)
    };

    // ── 5. Fragmentos SQL reutilizables para las distribuciones ───────────────
    //
    // El scope_join garantiza que los issues considerados pertenecen al
    // ciclo, al workspace y al proyecto correctos. Reusado en todas las
    // queries de distribución + burndown.
    let scope_join = format!(
        "JOIN cycle_issues ci ON ci.issue_id = i.id
           AND ci.cycle_id = '{cycle_id}'
           AND ci.workspace_id = '{ws_id}'
           AND ci.project_id = '{project_id}'
           AND ci.deleted_at IS NULL"
    );
    // Paridad con `Issue.issue_objects` (manager): excluye draft y archived.
    // Aquí no excluimos triage porque requiere JOIN extra con states y es
    // extremadamente improbable que issues en triage estén en un ciclo.
    let issue_filters = format!(
        "i.deleted_at IS NULL
         AND i.archived_at IS NULL
         AND i.is_draft = false
         AND i.workspace_id = '{ws_id}'
         AND i.project_id = '{project_id}'"
    );

    // Default vacíos (usados cuando type=points sin estimate_type_points).
    let mut assignee_distribution: Vec<serde_json::Value> = Vec::new();
    let mut label_distribution: Vec<serde_json::Value> = Vec::new();
    let mut completion_chart = serde_json::json!({});

    // ── 6. type=points con estimate_type=points ───────────────────────────────
    if analytic_type == "points" && estimate_type_points {
        // Assignee distribution: SUM(ep.value::float) por assignee.
        let sql = format!(
            "SELECT
               u.display_name                                          AS display_name,
               u.id                                                    AS assignee_id,
               CASE
                 WHEN u.avatar_asset_id IS NOT NULL
                   THEN '/api/assets/v2/static/' || u.avatar_asset_id::text || '/'
                 ELSE u.avatar
               END                                                     AS avatar_url,
               COALESCE(SUM(ep.value::float), 0)                       AS total_estimates,
               COALESCE(SUM(ep.value::float)
                 FILTER (WHERE i.completed_at IS NOT NULL), 0)         AS completed_estimates,
               COALESCE(SUM(ep.value::float)
                 FILTER (WHERE i.completed_at IS NULL), 0)             AS pending_estimates
             FROM issues i
             {scope_join}
             LEFT JOIN issue_assignees ia
               ON ia.issue_id = i.id AND ia.deleted_at IS NULL
             LEFT JOIN users u ON u.id = ia.assignee_id
             LEFT JOIN estimate_points ep
               ON ep.id = i.estimate_point_id AND ep.deleted_at IS NULL
             WHERE {issue_filters}
             GROUP BY u.id, u.display_name, u.avatar_asset_id, u.avatar
             ORDER BY u.display_name NULLS LAST"
        );
        let rows = db
            .query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql,
            ))
            .await
            .map_err(AppError::Database)?;
        assignee_distribution = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "display_name":        r.try_get::<String>("", "display_name").ok(),
                    "assignee_id":         r.try_get::<Uuid>("", "assignee_id").ok(),
                    "avatar_url":          r.try_get::<String>("", "avatar_url").ok(),
                    "total_estimates":     r.try_get::<f64>("", "total_estimates").unwrap_or(0.0),
                    "completed_estimates": r.try_get::<f64>("", "completed_estimates").unwrap_or(0.0),
                    "pending_estimates":   r.try_get::<f64>("", "pending_estimates").unwrap_or(0.0),
                })
            })
            .collect();

        // Label distribution: SUM(ep.value::float) por label.
        let sql = format!(
            "SELECT
               l.name                                                  AS label_name,
               l.color                                                 AS color,
               l.id                                                    AS label_id,
               COALESCE(SUM(ep.value::float), 0)                       AS total_estimates,
               COALESCE(SUM(ep.value::float)
                 FILTER (WHERE i.completed_at IS NOT NULL), 0)         AS completed_estimates,
               COALESCE(SUM(ep.value::float)
                 FILTER (WHERE i.completed_at IS NULL), 0)             AS pending_estimates
             FROM issues i
             {scope_join}
             LEFT JOIN issue_labels il
               ON il.issue_id = i.id AND il.deleted_at IS NULL
             LEFT JOIN labels l ON l.id = il.label_id AND l.deleted_at IS NULL
             LEFT JOIN estimate_points ep
               ON ep.id = i.estimate_point_id AND ep.deleted_at IS NULL
             WHERE {issue_filters}
             GROUP BY l.id, l.name, l.color
             ORDER BY l.name NULLS LAST"
        );
        let rows = db
            .query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql,
            ))
            .await
            .map_err(AppError::Database)?;
        label_distribution = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "label_name":          r.try_get::<String>("", "label_name").ok(),
                    "color":               r.try_get::<String>("", "color").ok(),
                    "label_id":            r.try_get::<Uuid>("", "label_id").ok(),
                    "total_estimates":     r.try_get::<f64>("", "total_estimates").unwrap_or(0.0),
                    "completed_estimates": r.try_get::<f64>("", "completed_estimates").unwrap_or(0.0),
                    "pending_estimates":   r.try_get::<f64>("", "pending_estimates").unwrap_or(0.0),
                })
            })
            .collect();

        completion_chart = burndown_plot(
            db,
            ws_id,
            project_id,
            cycle_id,
            start_date,
            end_date,
            BurndownMode::Points,
        )
        .await?;
    }

    // ── 7. type=issues ────────────────────────────────────────────────────────
    if analytic_type == "issues" {
        // Assignee distribution: COUNT de issues por assignee.
        // Django usa Count("assignee_id", filter=...). COUNT(col) ignora NULLs,
        // por lo que el bucket de issues sin assignee tiene 0 counts (paridad).
        let sql = format!(
            "SELECT
               u.display_name                                          AS display_name,
               u.id                                                    AS assignee_id,
               CASE
                 WHEN u.avatar_asset_id IS NOT NULL
                   THEN '/api/assets/v2/static/' || u.avatar_asset_id::text || '/'
                 ELSE u.avatar
               END                                                     AS avatar_url,
               COUNT(u.id)                                             AS total_issues,
               COUNT(u.id) FILTER (WHERE i.completed_at IS NOT NULL)   AS completed_issues,
               COUNT(u.id) FILTER (WHERE i.completed_at IS NULL)       AS pending_issues
             FROM issues i
             {scope_join}
             LEFT JOIN issue_assignees ia
               ON ia.issue_id = i.id AND ia.deleted_at IS NULL
             LEFT JOIN users u ON u.id = ia.assignee_id
             WHERE {issue_filters}
             GROUP BY u.id, u.display_name, u.avatar_asset_id, u.avatar
             ORDER BY u.display_name NULLS LAST"
        );
        let rows = db
            .query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql,
            ))
            .await
            .map_err(AppError::Database)?;
        assignee_distribution = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "display_name":     r.try_get::<String>("", "display_name").ok(),
                    "assignee_id":      r.try_get::<Uuid>("", "assignee_id").ok(),
                    "avatar_url":       r.try_get::<String>("", "avatar_url").ok(),
                    "total_issues":     r.try_get::<i64>("", "total_issues").unwrap_or(0),
                    "completed_issues": r.try_get::<i64>("", "completed_issues").unwrap_or(0),
                    "pending_issues":   r.try_get::<i64>("", "pending_issues").unwrap_or(0),
                })
            })
            .collect();

        // Label distribution: COUNT por label.
        let sql = format!(
            "SELECT
               l.name                                                  AS label_name,
               l.color                                                 AS color,
               l.id                                                    AS label_id,
               COUNT(l.id)                                             AS total_issues,
               COUNT(l.id) FILTER (WHERE i.completed_at IS NOT NULL)   AS completed_issues,
               COUNT(l.id) FILTER (WHERE i.completed_at IS NULL)       AS pending_issues
             FROM issues i
             {scope_join}
             LEFT JOIN issue_labels il
               ON il.issue_id = i.id AND il.deleted_at IS NULL
             LEFT JOIN labels l ON l.id = il.label_id AND l.deleted_at IS NULL
             WHERE {issue_filters}
             GROUP BY l.id, l.name, l.color
             ORDER BY l.name NULLS LAST"
        );
        let rows = db
            .query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql,
            ))
            .await
            .map_err(AppError::Database)?;
        label_distribution = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "label_name":       r.try_get::<String>("", "label_name").ok(),
                    "color":            r.try_get::<String>("", "color").ok(),
                    "label_id":         r.try_get::<Uuid>("", "label_id").ok(),
                    "total_issues":     r.try_get::<i64>("", "total_issues").unwrap_or(0),
                    "completed_issues": r.try_get::<i64>("", "completed_issues").unwrap_or(0),
                    "pending_issues":   r.try_get::<i64>("", "pending_issues").unwrap_or(0),
                })
            })
            .collect();

        completion_chart = burndown_plot(
            db,
            ws_id,
            project_id,
            cycle_id,
            start_date,
            end_date,
            BurndownMode::Issues,
        )
        .await?;
    }

    Ok(Json(serde_json::json!({
        "assignees": assignee_distribution,
        "labels": label_distribution,
        "completion_chart": completion_chart,
    })))
}

// ─── Burndown plot ───────────────────────────────────────────────────────────
//
// Mirror de `burndown_plot` (apps/api/plane/utils/analytics_plot.py:97).
// Genera un dict `{date_str: cumulative_pending}` para cada día en
// [start_date, end_date]. Las fechas futuras quedan en `null` (paridad).

#[derive(Debug, Clone, Copy)]
enum BurndownMode {
    Issues,
    Points,
}

async fn burndown_plot(
    db: &sea_orm::DatabaseConnection,
    ws_id: Uuid,
    project_id: Uuid,
    cycle_id: Uuid,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    mode: BurndownMode,
) -> Result<serde_json::Value, AppError> {
    // ── Total del ciclo ──────────────────────────────────────────────────────
    //
    // Paridad con Django: para issues, total = `cycle.total_issues`
    // (issues activos no-draft en el ciclo). Para points, total = suma de
    // estimate_point.value de todos los issues (con estimate) del ciclo.
    let total: f64 = match mode {
        BurndownMode::Issues => {
            let sql = format!(
                "SELECT COUNT(DISTINCT i.id) AS total
                 FROM issues i
                 JOIN cycle_issues ci ON ci.issue_id = i.id
                   AND ci.cycle_id = '{cycle_id}'
                   AND ci.workspace_id = '{ws_id}'
                   AND ci.project_id = '{project_id}'
                   AND ci.deleted_at IS NULL
                 WHERE i.deleted_at IS NULL
                   AND i.archived_at IS NULL
                   AND i.is_draft = false
                   AND i.workspace_id = '{ws_id}'
                   AND i.project_id = '{project_id}'"
            );
            db.query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql,
            ))
            .await
            .map_err(AppError::Database)?
            .and_then(|r| r.try_get::<i64>("", "total").ok())
            .unwrap_or(0) as f64
        }
        BurndownMode::Points => {
            let sql = format!(
                "SELECT COALESCE(SUM(ep.value::float), 0) AS total
                 FROM issues i
                 JOIN cycle_issues ci ON ci.issue_id = i.id
                   AND ci.cycle_id = '{cycle_id}'
                   AND ci.workspace_id = '{ws_id}'
                   AND ci.project_id = '{project_id}'
                   AND ci.deleted_at IS NULL
                 JOIN estimate_points ep
                   ON ep.id = i.estimate_point_id AND ep.deleted_at IS NULL
                 WHERE i.deleted_at IS NULL
                   AND i.archived_at IS NULL
                   AND i.is_draft = false
                   AND i.workspace_id = '{ws_id}'
                   AND i.project_id = '{project_id}'
                   AND i.estimate_point_id IS NOT NULL"
            );
            db.query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                sql,
            ))
            .await
            .map_err(AppError::Database)?
            .and_then(|r| r.try_get::<f64>("", "total").ok())
            .unwrap_or(0.0)
        }
    };

    // ── Completados por fecha ────────────────────────────────────────────────
    let completed_sql = match mode {
        BurndownMode::Issues => format!(
            "SELECT DATE(i.completed_at) AS day, COUNT(*)::float AS completed
             FROM issues i
             JOIN cycle_issues ci ON ci.issue_id = i.id
               AND ci.cycle_id = '{cycle_id}'
               AND ci.workspace_id = '{ws_id}'
               AND ci.project_id = '{project_id}'
               AND ci.deleted_at IS NULL
             WHERE i.deleted_at IS NULL
               AND i.archived_at IS NULL
               AND i.is_draft = false
               AND i.workspace_id = '{ws_id}'
               AND i.project_id = '{project_id}'
               AND i.completed_at IS NOT NULL
             GROUP BY DATE(i.completed_at)
             ORDER BY DATE(i.completed_at)"
        ),
        BurndownMode::Points => format!(
            "SELECT DATE(i.completed_at) AS day, SUM(ep.value::float) AS completed
             FROM issues i
             JOIN cycle_issues ci ON ci.issue_id = i.id
               AND ci.cycle_id = '{cycle_id}'
               AND ci.workspace_id = '{ws_id}'
               AND ci.project_id = '{project_id}'
               AND ci.deleted_at IS NULL
             JOIN estimate_points ep
               ON ep.id = i.estimate_point_id AND ep.deleted_at IS NULL
             WHERE i.deleted_at IS NULL
               AND i.archived_at IS NULL
               AND i.is_draft = false
               AND i.workspace_id = '{ws_id}'
               AND i.project_id = '{project_id}'
               AND i.completed_at IS NOT NULL
               AND i.estimate_point_id IS NOT NULL
             GROUP BY DATE(i.completed_at)
             ORDER BY DATE(i.completed_at)"
        ),
    };
    let rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            completed_sql,
        ))
        .await
        .map_err(AppError::Database)?;

    // Map: fecha → total completado ese día.
    use std::collections::BTreeMap;
    let daily_completed: BTreeMap<chrono::NaiveDate, f64> = rows
        .iter()
        .filter_map(|r| {
            let day = r.try_get::<chrono::NaiveDate>("", "day").ok()?;
            let completed = r.try_get::<f64>("", "completed").unwrap_or(0.0);
            Some((day, completed))
        })
        .collect();

    // ── Construir chart_data: iterar día por día y acumular ──────────────────
    let today = chrono::Utc::now().date_naive();
    let mut chart_data = serde_json::Map::new();
    let mut current = start_date;
    while current <= end_date {
        let key = current.to_string(); // "YYYY-MM-DD"
        if current > today {
            // Fechas futuras: null (paridad Django).
            chart_data.insert(key, serde_json::Value::Null);
        } else {
            // Suma de todo lo completado hasta (inclusive) `current`.
            let completed_to_date: f64 = daily_completed
                .range(..=current)
                .map(|(_, v)| v)
                .sum();
            let pending = total - completed_to_date;
            chart_data.insert(
                key,
                serde_json::Number::from_f64(pending)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        current += chrono::Duration::days(1);
    }

    Ok(serde_json::Value::Object(chart_data))
}

// ═════════════════════════════════════════════════════════════════════════════
// Cycle User Properties
// ═════════════════════════════════════════════════════════════════════════════
//
// Mirror de `CycleUserPropertiesEndpoint` en
// apps/api/plane/app/views/cycle/base.py:625-655.
//
// Semántica clave (paridad Django):
//   - GET hace `get_or_create` → NUNCA devuelve 404 por ausencia de fila.
//     Si no existe, se crea con defaults y se devuelve 200.
//   - PATCH asume que existe (Django usa `.get(...)` crudo, que lanzaría 500
//     si faltara). Para evitar ese fallo y ser más útil al frontend, aquí
//     también hacemos `get_or_create` y aplicamos el patch encima — no
//     degrada ningún caso de uso válido.
//   - Permisos: ADMIN / MEMBER / GUEST (igual que Django).
//
// Nota sobre el modelo: `CycleUserProperties` (cycle.py:130-153) NO tiene
// los campos `preferences` ni `sort_order` que sí tiene `ProjectUserProperty`.
// Por eso el request/response DTO es más simple que el de project.

// ─── Defaults — mirror de `plane/db/models/issue.py:47-88` ───────────────────

fn cycle_default_filters() -> serde_json::Value {
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

fn cycle_default_display_filters() -> serde_json::Value {
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

fn cycle_default_display_properties() -> serde_json::Value {
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

/// Mirror de `CycleUserPropertiesSerializer` (fields="__all__", read_only:
/// workspace/project/cycle/user). Incluye todos los campos del modelo
/// `CycleUserProperties` de `apps/api/plane/db/models/cycle.py:130-153`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CycleUserPropertiesResponse {
    pub id: Uuid,
    pub user: Uuid,
    pub cycle: Uuid,
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

impl From<&cycle_user_properties::Model> for CycleUserPropertiesResponse {
    fn from(m: &cycle_user_properties::Model) -> Self {
        Self {
            id: m.id,
            user: m.user_id,
            cycle: m.cycle_id,
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
/// (workspace/project/cycle/user) se ignoran si vienen en el body.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCycleUserPropertiesRequest {
    pub filters: Option<serde_json::Value>,
    pub display_filters: Option<serde_json::Value>,
    pub display_properties: Option<serde_json::Value>,
    pub rich_filters: Option<serde_json::Value>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Busca o crea la fila `cycle_user_properties` para `(cycle, user)`.
/// Mirror de `CycleUserProperties.objects.get_or_create(...)`.
///
/// Constraint único en Django: `(cycle, user)` WHERE `deleted_at IS NULL`
/// (`cycle.py:144-149`). Filtramos por `.active()`. En caso de INSERT
/// concurrente con violación del índice único, el error se propagaría como
/// `AppError::Database` y un reintento del cliente resolvería el caso —
/// mismo comportamiento que Django.
async fn get_or_create_cycle_user_properties(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
    cycle_id: Uuid,
    user_id: Uuid,
) -> Result<cycle_user_properties::Model, AppError> {
    if let Some(existing) = cycle_user_properties::Entity::find()
        .active()
        .filter(cycle_user_properties::Column::UserId.eq(user_id))
        .filter(cycle_user_properties::Column::CycleId.eq(cycle_id))
        .filter(cycle_user_properties::Column::ProjectId.eq(project_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
    {
        return Ok(existing);
    }

    let now = chrono::Utc::now().fixed_offset();
    let created = cycle_user_properties::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        cycle_id: Set(cycle_id),
        project_id: Set(project_id),
        workspace_id: Set(workspace_id),
        filters: Set(cycle_default_filters()),
        display_filters: Set(cycle_default_display_filters()),
        display_properties: Set(cycle_default_display_properties()),
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

/// Asegura que el ciclo existe y pertenece al workspace/proyecto del guard.
/// 404 si no existe o fue soft-deleted. Defensa en profundidad — el guard
/// sólo valida workspace + proyecto, no la pertenencia del `cycle_id`.
async fn ensure_cycle_belongs_to_project(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
    cycle_id: Uuid,
) -> Result<cycles::Model, AppError> {
    cycles::Entity::find_by_id(cycle_id)
        .filter(cycles::Column::WorkspaceId.eq(workspace_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

// ─── GET ──────────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties/`
///
/// Paridad con `CycleUserPropertiesEndpoint.get` (cycle/base.py:647-655).
/// `get_or_create` garantiza que nunca devolvemos 404 por ausencia de la
/// fila de propiedades — esto es lo que resuelve el 404 del frontend.
///
/// Permisos: ROLE_GUEST+ (ADMIN/MEMBER/GUEST, paridad Django).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties/",
    tag = "Cycles",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("cycle_id"   = Uuid,   Path, description = "Cycle UUID"),
    ),
    responses(
        (status = 200, description = "Cycle user properties", body = CycleUserPropertiesResponse),
        (status = 403, description = "User is not a member of the project"),
        (status = 404, description = "Workspace, project or cycle not found"),
    )
)]
pub async fn get_cycle_user_properties(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<CycleUserPropertiesResponse>, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    // Validación: el ciclo debe existir y pertenecer al proyecto. Si no,
    // 404 — evita crear una fila user-properties huérfana.
    let _cycle = ensure_cycle_belongs_to_project(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        cycle_id,
    )
    .await?;

    let row = get_or_create_cycle_user_properties(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        cycle_id,
        guard.user.id,
    )
    .await?;

    Ok(Json(CycleUserPropertiesResponse::from(&row)))
}

// ─── PATCH ────────────────────────────────────────────────────────────────────

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties/`
///
/// Paridad con `CycleUserPropertiesEndpoint.patch` (cycle/base.py:627-644),
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
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties/",
    tag = "Cycles",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("cycle_id"   = Uuid,   Path, description = "Cycle UUID"),
    ),
    request_body = UpdateCycleUserPropertiesRequest,
    responses(
        (status = 200, description = "Updated cycle user properties", body = CycleUserPropertiesResponse),
        (status = 403, description = "User is not a member of the project"),
        (status = 404, description = "Workspace, project or cycle not found"),
    )
)]
pub async fn update_cycle_user_properties(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateCycleUserPropertiesRequest>,
) -> Result<Json<CycleUserPropertiesResponse>, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    let _cycle = ensure_cycle_belongs_to_project(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        cycle_id,
    )
    .await?;

    let row = get_or_create_cycle_user_properties(
        &state.db,
        guard.workspace.id,
        guard.project.id,
        cycle_id,
        guard.user.id,
    )
    .await?;

    let mut am: cycle_user_properties::ActiveModel = row.into();
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
    Ok(Json(CycleUserPropertiesResponse::from(&updated)))
}
