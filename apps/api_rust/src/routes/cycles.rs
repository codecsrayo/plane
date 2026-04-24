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
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{cycle_issues, cycle_user_properties, cycles, issues, user_favorites},
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
    #[serde(default, deserialize_with = "crate::utils::serde_date::deserialize_date_or_datetime")]
    pub start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    #[serde(default, deserialize_with = "crate::utils::serde_date::deserialize_date_or_datetime")]
    pub end_date: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCycleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_date::deserialize_date_or_datetime")]
    pub start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    #[serde(default, deserialize_with = "crate::utils::serde_date::deserialize_date_or_datetime")]
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

    // Paridad con `CycleWriteSerializer.validate` (Django): cuando AMBAS
    // fechas están presentes, se convierten a UTC usando la zona horaria
    // del proyecto (start → 00:00:01 local, end → 23:59:00 local). Si solo
    // viene una, Django NO aplica la conversión — replicamos esa quirk.
    let (start_date, end_date) = match (body.start_date, body.end_date) {
        (Some(s), Some(e)) => {
            let now_utc = chrono::Utc::now();
            let start = crate::utils::serde_date::project_tz_to_utc(
                s.date_naive(),
                &guard.project.timezone,
                true,
                now_utc,
            )
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            let end = crate::utils::serde_date::project_tz_to_utc(
                e.date_naive(),
                &guard.project.timezone,
                false,
                now_utc,
            )
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            (Some(start), Some(end))
        }
        other => other,
    };

    // created_at / updated_at explícitos: cycles::ActiveModelBehavior está
    // vacío y la columna es NOT NULL sin DEFAULT. Mismo patrón que
    // labels.rs / issues.rs.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let cycle = cycles::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or_default()),
        start_date: Set(start_date),
        end_date: Set(end_date),
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

    // Paridad con `CycleWriteSerializer.validate` (Django): la conversión a
    // UTC tz-aware solo aplica cuando AMBOS dates están en el payload de
    // este PATCH. Si viene solo uno, se persiste tal cual — mismo quirk
    // que Django, donde `validate()` opera sobre el `data` parcial y no
    // sobre la instancia mergeada. Sirve para que un PATCH que toque solo
    // `name`/`description` no recalcule las fechas existentes.
    match (body.start_date, body.end_date) {
        (Some(s), Some(e)) => {
            let now_utc = chrono::Utc::now();
            let start = crate::utils::serde_date::project_tz_to_utc(
                s.date_naive(),
                &guard.project.timezone,
                true,
                now_utc,
            )
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            let end = crate::utils::serde_date::project_tz_to_utc(
                e.date_naive(),
                &guard.project.timezone,
                false,
                now_utc,
            )
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            am.start_date = Set(Some(start));
            am.end_date = Set(Some(end));
        }
        (Some(s), None) => {
            am.start_date = Set(Some(s));
        }
        (None, Some(e)) => {
            am.end_date = Set(Some(e));
        }
        (None, None) => {}
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

// ═════════════════════════════════════════════════════════════════════════════
// Cycle Progress
// ═════════════════════════════════════════════════════════════════════════════
//
// Mirror de `CycleProgressEndpoint` en
// apps/api/plane/app/views/cycle/base.py:658-783.
//
// Devuelve counts de issues y sumas de estimate_points agrupados por
// state.group (backlog/unstarted/started/cancelled/completed) + totales.
//
// Semántica clave (paridad Django):
//   - Si el ciclo tiene `progress_snapshot` no vacío → los counts de issues
//     vienen del snapshot (ciclo cerrado con issues transferidos).
//     Las sumas de estimate_points SIEMPRE se calculan live — el snapshot
//     no las contiene (línea 664 en base.py: `aggregate_estimates` se
//     computa antes del branch).
//   - Sin snapshot → counts live sobre `issues` JOIN `cycle_issues`.
//   - Estimate sums: sólo suman puntos de issues cuyo estimate.type='points'.
//   - Permisos: ADMIN / MEMBER / GUEST.
//
// Optimización sobre Django: una sola query agregada por cada bloque (counts
// y estimates), en vez de 6 y 6 queries separadas como hace el ORM.

/// Estructura interna para deserializar los counts del SQL agregado.
#[derive(Debug, Default)]
struct ProgressIssueCounts {
    backlog: i64,
    unstarted: i64,
    started: i64,
    cancelled: i64,
    completed: i64,
    total: i64,
}

/// Estructura interna para deserializar las sumas de estimate_points.
#[derive(Debug, Default)]
struct ProgressEstimatePoints {
    backlog: f64,
    unstarted: f64,
    started: f64,
    cancelled: f64,
    completed: f64,
    total: f64,
}

/// Suma de `CAST(estimate_points.value AS DOUBLE PRECISION)` agrupada por
/// `state.group`, filtrando issues del ciclo cuyo `estimate_point` pertenece
/// a un `estimate` con `type='points'`.
///
/// Mirror de la query compuesta en cycle/base.py:664-711. Django hace esto
/// con 6 `Sum(Case(When(...), default=0))` en un único `.aggregate(...)`,
/// que se compila a exactamente esta forma en SQL.
///
/// Todos los resultados se COALESCE a 0 — Django usa `default=Value(0)` en
/// cada Sum y además `or 0` en la mayoría de los campos de respuesta.
async fn compute_estimate_points(
    db: &sea_orm::DatabaseConnection,
    ws_id: Uuid,
    project_id: Uuid,
    cycle_id: Uuid,
) -> Result<ProgressEstimatePoints, AppError> {
    // UUIDs van interpolados: son type-safe (Uuid::Display solo produce
    // hex+dashes), no hay superficie de SQL injection. Mismo patrón que
    // `cycle_analytics` más arriba en este archivo.
    let sql = format!(
        "SELECT
            COALESCE(SUM(CAST(ep.value AS DOUBLE PRECISION)) FILTER (WHERE s.\"group\" = 'backlog'), 0)::float8   AS backlog,
            COALESCE(SUM(CAST(ep.value AS DOUBLE PRECISION)) FILTER (WHERE s.\"group\" = 'unstarted'), 0)::float8 AS unstarted,
            COALESCE(SUM(CAST(ep.value AS DOUBLE PRECISION)) FILTER (WHERE s.\"group\" = 'started'), 0)::float8   AS started,
            COALESCE(SUM(CAST(ep.value AS DOUBLE PRECISION)) FILTER (WHERE s.\"group\" = 'cancelled'), 0)::float8 AS cancelled,
            COALESCE(SUM(CAST(ep.value AS DOUBLE PRECISION)) FILTER (WHERE s.\"group\" = 'completed'), 0)::float8 AS completed,
            COALESCE(SUM(CAST(ep.value AS DOUBLE PRECISION)), 0)::float8                                          AS total
         FROM issues i
         JOIN cycle_issues ci
           ON ci.issue_id = i.id
          AND ci.cycle_id = '{cycle_id}'
          AND ci.workspace_id = '{ws_id}'
          AND ci.project_id = '{project_id}'
          AND ci.deleted_at IS NULL
         JOIN estimate_points ep
           ON ep.id = i.estimate_point_id
          AND ep.deleted_at IS NULL
         JOIN estimates e
           ON e.id = ep.estimate_id
          AND e.\"type\" = 'points'
          AND e.deleted_at IS NULL
         LEFT JOIN states s ON s.id = i.state_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.project_id = '{project_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL
           AND i.is_draft = FALSE
           AND (s.\"group\" IS NULL OR s.\"group\" <> 'triage')"
    );

    let row = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .map_err(AppError::Database)?;

    let Some(row) = row else {
        return Ok(ProgressEstimatePoints::default());
    };

    Ok(ProgressEstimatePoints {
        backlog: row.try_get::<f64>("", "backlog").unwrap_or(0.0),
        unstarted: row.try_get::<f64>("", "unstarted").unwrap_or(0.0),
        started: row.try_get::<f64>("", "started").unwrap_or(0.0),
        cancelled: row.try_get::<f64>("", "cancelled").unwrap_or(0.0),
        completed: row.try_get::<f64>("", "completed").unwrap_or(0.0),
        total: row.try_get::<f64>("", "total").unwrap_or(0.0),
    })
}

/// Counts live de issues por `state.group` para el ciclo. Una query en vez
/// de 6. Mirror de cycle/base.py:720-765 (que Django resuelve con 6 queries
/// separadas). Respeta el manager `issue_objects` (excluye triage, archived,
/// draft, soft-deleted).
async fn compute_issue_counts(
    db: &sea_orm::DatabaseConnection,
    ws_id: Uuid,
    project_id: Uuid,
    cycle_id: Uuid,
) -> Result<ProgressIssueCounts, AppError> {
    let sql = format!(
        "SELECT
            COUNT(*) FILTER (WHERE s.\"group\" = 'backlog')   AS backlog,
            COUNT(*) FILTER (WHERE s.\"group\" = 'unstarted') AS unstarted,
            COUNT(*) FILTER (WHERE s.\"group\" = 'started')   AS started,
            COUNT(*) FILTER (WHERE s.\"group\" = 'cancelled') AS cancelled,
            COUNT(*) FILTER (WHERE s.\"group\" = 'completed') AS completed,
            COUNT(*)                                          AS total
         FROM issues i
         JOIN cycle_issues ci
           ON ci.issue_id = i.id
          AND ci.cycle_id = '{cycle_id}'
          AND ci.workspace_id = '{ws_id}'
          AND ci.project_id = '{project_id}'
          AND ci.deleted_at IS NULL
         LEFT JOIN states s ON s.id = i.state_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.project_id = '{project_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL
           AND i.is_draft = FALSE
           AND (s.\"group\" IS NULL OR s.\"group\" <> 'triage')"
    );

    let row = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .map_err(AppError::Database)?;

    let Some(row) = row else {
        return Ok(ProgressIssueCounts::default());
    };

    Ok(ProgressIssueCounts {
        backlog: row.try_get::<i64>("", "backlog").unwrap_or(0),
        unstarted: row.try_get::<i64>("", "unstarted").unwrap_or(0),
        started: row.try_get::<i64>("", "started").unwrap_or(0),
        cancelled: row.try_get::<i64>("", "cancelled").unwrap_or(0),
        completed: row.try_get::<i64>("", "completed").unwrap_or(0),
        total: row.try_get::<i64>("", "total").unwrap_or(0),
    })
}

/// Extrae un count entero de `progress_snapshot[key]`. El snapshot guarda
/// los valores como números JSON; si falta la clave o no es número, 0.
fn snapshot_i64(snapshot: &serde_json::Map<String, serde_json::Value>, key: &str) -> i64 {
    snapshot
        .get(key)
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
        .unwrap_or(0)
}

/// GET /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress/
///
/// Devuelve 12 métricas: counts de issues por estado + sumas de estimate_points
/// por estado, más totales. Mirror exacto de `CycleProgressEndpoint.get`
/// (cycle/base.py:658-783).
///
/// Comportamiento:
/// - Si el ciclo no existe → 404 (paridad Django: `{"error": "Cycle not found"}`).
/// - Si `progress_snapshot` es un objeto no vacío → los counts de issues se
///   leen del snapshot (ciclo cerrado, issues transferidos).
/// - Las sumas de estimate_points SIEMPRE se calculan live (Django también
///   lo hace: `aggregate_estimates` se computa antes del branch del snapshot).
///
/// Permisos: ADMIN / MEMBER / GUEST (paridad Django).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress/",
    tag = "Cycles",
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("cycle_id"   = Uuid,   Path, description = "Cycle UUID"),
    ),
    responses(
        (status = 200, description = "Cycle progress counts + estimate points sums"),
        (status = 403, description = "Not authorized"),
        (status = 404, description = "Cycle not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn cycle_progress(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_GUEST,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    // ── 1. Validar que el ciclo existe y pertenece al proyecto ────────────────
    let cycle =
        ensure_cycle_belongs_to_project(db, ws_id, project_id, cycle_id).await?;

    // ── 2. Sumas de estimate_points — SIEMPRE live, como en Django ────────────
    //
    // Django computa `aggregate_estimates` en cycle/base.py:664, ANTES del
    // branch del snapshot. El snapshot no contiene estas sumas.
    let estimates = compute_estimate_points(db, ws_id, project_id, cycle_id).await?;

    // ── 3. Counts de issues: desde snapshot si existe y no está vacío ────────
    //
    // Django: `if cycle.progress_snapshot:` — truthy sobre dict. En Postgres
    // la columna es NOT NULL con default `{}` (falsy en Python) así que
    // tenemos que distinguir objeto vacío de objeto con datos.
    let counts = if let serde_json::Value::Object(snapshot) = &cycle.progress_snapshot {
        if !snapshot.is_empty() {
            // Leer del snapshot — mismas claves que usa Django al persistirlo.
            ProgressIssueCounts {
                backlog: snapshot_i64(snapshot, "backlog_issues"),
                unstarted: snapshot_i64(snapshot, "unstarted_issues"),
                started: snapshot_i64(snapshot, "started_issues"),
                cancelled: snapshot_i64(snapshot, "cancelled_issues"),
                completed: snapshot_i64(snapshot, "completed_issues"),
                total: snapshot_i64(snapshot, "total_issues"),
            }
        } else {
            compute_issue_counts(db, ws_id, project_id, cycle_id).await?
        }
    } else {
        compute_issue_counts(db, ws_id, project_id, cycle_id).await?
    };

    // ── 4. Respuesta — mismas claves que Django (cycle/base.py:768-781) ───────
    Ok(Json(serde_json::json!({
        "backlog_estimate_points":   estimates.backlog,
        "unstarted_estimate_points": estimates.unstarted,
        "started_estimate_points":   estimates.started,
        "cancelled_estimate_points": estimates.cancelled,
        "completed_estimate_points": estimates.completed,
        "total_estimate_points":     estimates.total,
        "backlog_issues":   counts.backlog,
        "unstarted_issues": counts.unstarted,
        "started_issues":   counts.started,
        "cancelled_issues": counts.cancelled,
        "completed_issues": counts.completed,
        "total_issues":     counts.total,
    })))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS PENDIENTES — implementados a continuación
// ═══════════════════════════════════════════════════════════════════════════════

// ── date-check ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DateCheckRequest {
    pub start_date: String,
    pub end_date: String,
    pub cycle_id: Option<Uuid>,
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/cycles/date-check/`
///
/// Verifica si un rango de fechas se solapa con algún ciclo existente en el proyecto.
/// Paridad con `CycleDateCheckEndpoint.post` (cycle/base.py).
/// Retorna `{"status": true}` si hay solapamiento, `{"status": false}` si no.
///
/// Permisos: ADMIN / MEMBER.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/date-check/",
    tag = "Cycles",
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Date overlap check result"),
        (status = 400, description = "Missing start_date or end_date"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn cycle_date_check(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<DateCheckRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    // Parsear fechas — acepta "YYYY-MM-DD" o ISO 8601
    let start: chrono::NaiveDate = body
        .start_date
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid start_date format".into()))?;
    let end: chrono::NaiveDate = body
        .end_date
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid end_date format".into()))?;

    // Convertir a DateTimeWithTimeZone para comparar con la columna
    let start_dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::NaiveDateTime::new(start, chrono::NaiveTime::MIN)
            .and_utc()
            .fixed_offset();
    let end_dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::NaiveDateTime::new(end, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap())
            .and_utc()
            .fixed_offset();

    // Buscar ciclos que se solapan con el rango dado
    // Solapamiento: start_cycle <= end_input AND end_cycle >= start_input
    let mut query = cycles::Entity::find()
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .filter(cycles::Column::ArchivedAt.is_null())
        .filter(cycles::Column::StartDate.is_not_null())
        .filter(cycles::Column::EndDate.is_not_null())
        .filter(cycles::Column::StartDate.lte(end_dt))
        .filter(cycles::Column::EndDate.gte(start_dt));

    if let Some(cid) = body.cycle_id {
        query = query.filter(cycles::Column::Id.ne(cid));
    }

    let overlapping = query.count(db).await.map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "status": overlapping > 0 })))
}

// ── user-favorite-cycles ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct FavoriteCycleRequest {
    pub cycle: Uuid,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-favorite-cycles/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of favorite cycles"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_favorite_cycles(
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
        .filter(user_favorites::Column::EntityType.eq("cycle"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<serde_json::Value> = favorites
        .into_iter()
        .map(|f| {
            serde_json::json!({
                "id": f.id,
                "entity_type": f.entity_type,
                "entity_identifier": f.entity_identifier,
                "project_id": f.project_id,
                "workspace_id": f.workspace_id,
            })
        })
        .collect();

    Ok(Json(resp))
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/user-favorite-cycles/`
///
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-favorite-cycles/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 204, description = "Cycle added to favorites"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_favorite_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<FavoriteCycleRequest>,
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

    // Idempotente: si ya existe no duplicamos
    let existing = user_favorites::Entity::find()
        .filter(user_favorites::Column::WorkspaceId.eq(ws_id))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .filter(user_favorites::Column::UserId.eq(user_id))
        .filter(user_favorites::Column::EntityType.eq("cycle"))
        .filter(user_favorites::Column::EntityIdentifier.eq(body.cycle))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_none() {
        let new_fav = user_favorites::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set("cycle".to_string()),
            entity_identifier: Set(Some(body.cycle)),
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

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/user-favorite-cycles/{cycle_id}/`
///
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/user-favorite-cycles/{cycle_id}/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 204, description = "Cycle removed from favorites"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_favorite_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
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
        .filter(user_favorites::Column::EntityType.eq("cycle"))
        .filter(user_favorites::Column::EntityIdentifier.eq(cycle_id))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let active: user_favorites::ActiveModel = fav.into();
    active.delete(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── transfer-issues ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TransferCycleIssuesRequest {
    pub new_cycle_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Issues transferred"),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn transfer_cycle_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<TransferCycleIssuesRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let new_cycle_id = body.new_cycle_id;
    let db = &state.db;

    // Validar que el ciclo destino existe y no está completado
    let new_cycle = cycles::Entity::find_by_id(new_cycle_id)
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now();
    if let Some(end) = new_cycle.end_date {
        if end.with_timezone(&chrono::Utc) < now {
            return Err(AppError::BadRequest(
                "The cycle where the issues are transferred is already completed".into(),
            ));
        }
    }

    // Validar que el ciclo origen existe
    let old_cycle = ensure_cycle_belongs_to_project(db, ws_id, project_id, cycle_id).await?;

    // Computar counts de issues por estado para el snapshot
    let counts = compute_issue_counts(db, ws_id, project_id, cycle_id).await?;

    // Guardar progress snapshot en el ciclo origen
    let snapshot = serde_json::json!({
        "total_issues":     counts.total,
        "completed_issues": counts.completed,
        "cancelled_issues": counts.cancelled,
        "started_issues":   counts.started,
        "unstarted_issues": counts.unstarted,
        "backlog_issues":   counts.backlog,
        "distribution": {
            "labels": [],
            "assignees": [],
            "completion_chart": {}
        },
        "estimate_distribution": {}
    });

    let mut old_active: cycles::ActiveModel = old_cycle.into();
    old_active.progress_snapshot = Set(snapshot);
    old_active.update(db).await.map_err(AppError::Database)?;

    // Transferir las issues incompletas al nuevo ciclo
    // Solo issues con estado: backlog, unstarted, started
    let sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        UPDATE cycle_issues ci
        SET cycle_id = $1, updated_at = NOW()
        FROM issues i
        JOIN states s ON s.id = i.state_id
        WHERE ci.cycle_id = $2
          AND ci.deleted_at IS NULL
          AND ci.issue_id = i.id
          AND i.archived_at IS NULL
          AND i.is_draft = FALSE
          AND i.deleted_at IS NULL
          AND s.group IN ('backlog', 'unstarted', 'started')
        "#,
        vec![
            new_cycle_id.into(),
            cycle_id.into(),
        ],
    );
    db.execute(sql).await.map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "message": "Success" })))
}

// ── archive / unarchive cycle ─────────────────────────────────────────────────

/// `POST /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive/`
///
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Cycle archived"),
        (status = 400, description = "Only completed cycles can be archived"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn archive_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let cycle = ensure_cycle_belongs_to_project(db, ws_id, project_id, cycle_id).await?;

    // Solo ciclos completados (end_date en el pasado) pueden archivarse
    let now = chrono::Utc::now();
    match cycle.end_date {
        Some(end) if end.with_timezone(&chrono::Utc) >= now => {
            return Err(AppError::BadRequest(
                "Only completed cycles can be archived".into(),
            ));
        }
        None => {
            return Err(AppError::BadRequest(
                "Only completed cycles can be archived".into(),
            ));
        }
        _ => {}
    }

    let archived_at = chrono::Utc::now().fixed_offset();
    let mut active: cycles::ActiveModel = cycle.into();
    active.archived_at = Set(Some(archived_at));
    active.update(db).await.map_err(AppError::Database)?;

    // Eliminar de favoritos (paridad Django)
    let _ = user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::EntityType.eq("cycle"))
        .filter(user_favorites::Column::EntityIdentifier.eq(cycle_id))
        .filter(user_favorites::Column::ProjectId.eq(project_id))
        .filter(user_favorites::Column::WorkspaceId.eq(ws_id))
        .exec(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "archived_at": archived_at.to_rfc3339() })))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive/`
///
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Cycle unarchived"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn unarchive_cycle(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;

    let cycle = cycles::Entity::find_by_id(cycle_id)
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut active: cycles::ActiveModel = cycle.into();
    active.archived_at = Set(None);
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── archived-cycles ───────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/projects/{project_id}/archived-cycles/`
///
/// Lista los ciclos archivados del proyecto.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/archived-cycles/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of archived cycles"),
        (status = 403, description = "Not authorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_archived_cycles(
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

    let cycles_list = cycles::Entity::find()
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .filter(cycles::Column::ArchivedAt.is_not_null())
        .order_by_desc(cycles::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<serde_json::Value> = cycles_list
        .into_iter()
        .map(|c| {
            let now = chrono::Utc::now();
            let status = match (c.start_date, c.end_date) {
                (None, _) | (_, None) => "draft",
                (Some(s), Some(e)) => {
                    let s_utc = s.with_timezone(&chrono::Utc);
                    let e_utc = e.with_timezone(&chrono::Utc);
                    if now < s_utc { "upcoming" } else if now > e_utc { "completed" } else { "started" }
                }
            };
            serde_json::json!({
                "id": c.id,
                "workspace_id": c.workspace_id,
                "project_id": c.project_id,
                "name": c.name,
                "description": c.description,
                "start_date": c.start_date,
                "end_date": c.end_date,
                "owned_by_id": c.owned_by_id,
                "view_props": c.view_props,
                "sort_order": c.sort_order,
                "external_source": c.external_source,
                "external_id": c.external_id,
                "progress_snapshot": c.progress_snapshot,
                "status": status,
                "archived_at": c.archived_at,
                "created_at": c.created_at,
                "updated_at": c.updated_at,
            })
        })
        .collect();

    Ok(Json(resp))
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}/`
///
/// Devuelve un ciclo archivado por su ID.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Cycle ID"),
    ),
    responses(
        (status = 200, description = "Archived cycle"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_archived_cycle(
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

    let c = cycles::Entity::find_by_id(pk)
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .filter(cycles::Column::ArchivedAt.is_not_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now();
    let status = match (c.start_date, c.end_date) {
        (None, _) | (_, None) => "draft",
        (Some(s), Some(e)) => {
            if now < s.with_timezone(&chrono::Utc) { "upcoming" }
            else if now > e.with_timezone(&chrono::Utc) { "completed" }
            else { "started" }
        }
    };

    Ok(Json(serde_json::json!({
        "id": c.id,
        "workspace_id": c.workspace_id,
        "project_id": c.project_id,
        "name": c.name,
        "description": c.description,
        "start_date": c.start_date,
        "end_date": c.end_date,
        "owned_by_id": c.owned_by_id,
        "view_props": c.view_props,
        "sort_order": c.sort_order,
        "external_source": c.external_source,
        "external_id": c.external_id,
        "progress_snapshot": c.progress_snapshot,
        "logo_props": c.logo_props,
        "status": status,
        "archived_at": c.archived_at,
        "created_at": c.created_at,
        "updated_at": c.updated_at,
        "created_by_id": c.created_by_id,
    })))
}
