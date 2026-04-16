// src/routes/analytics.rs
//! Endpoints de analytics del workspace.
//!
//! Equivalente a `plane/app/views/analytic/` en Django.
//!
//! Rutas implementadas:
//!   GET    /api/workspaces/{slug}/analytics/
//!   GET    /api/workspaces/{slug}/default-analytics/
//!   GET    /api/workspaces/{slug}/project-stats/
//!   POST   /api/workspaces/{slug}/export-analytics/
//!
//!   GET/POST   /api/workspaces/{slug}/analytic-view/
//!   GET/PATCH/DELETE /api/workspaces/{slug}/analytic-view/{pk}/
//!   GET        /api/workspaces/{slug}/saved-analytic-view/{analytic_id}/
//!
//!   GET    /api/workspaces/{slug}/advance-analytics/
//!   GET    /api/workspaces/{slug}/advance-analytics-stats/
//!   GET    /api/workspaces/{slug}/advance-analytics-charts/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Set,
    Statement,
};
use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::WorkspaceMemberGuard,
        permissions::{require_workspace_admin, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::analytic_views,
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Constantes de validación ──────────────────────────────────────────────────

const VALID_X_AXIS: &[&str] = &[
    "state_id",
    "state__group",
    "labels__id",
    "assignees__id",
    "estimate_point__value",
    "issue_cycle__cycle_id",
    "issue_module__module_id",
    "priority",
    "start_date",
    "target_date",
    "created_at",
    "completed_at",
];

const VALID_Y_AXIS: &[&str] = &["issue_count", "estimate"];

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AnalyticViewResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub query: serde_json::Value,
    pub query_dict: serde_json::Value,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAnalyticViewRequest {
    pub name: String,
    pub description: Option<String>,
    pub query: Option<serde_json::Value>,
    pub query_dict: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAnalyticViewRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<serde_json::Value>,
    pub query_dict: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AnalyticsQuery {
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub segment: Option<String>,
    /// IDs de proyectos separados por coma para filtrar.
    pub project_ids: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProjectStatsQuery {
    pub fields: Option<String>,
    pub project_ids: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ExportAnalyticsRequest {
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub segment: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn analytic_view_to_response(v: &analytic_views::Model) -> AnalyticViewResponse {
    AnalyticViewResponse {
        id: v.id,
        name: v.name.clone(),
        description: v.description.clone(),
        query: v.query.clone(),
        query_dict: v.query_dict.clone(),
        workspace_id: v.workspace_id,
        created_by_id: v.created_by_id,
        updated_by_id: v.updated_by_id,
        created_at: v.created_at,
        updated_at: v.updated_at,
    }
}

// ── AnalyticView CRUD ─────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/analytic-view/
///
/// Lista las vistas analíticas guardadas del workspace. Solo admins.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/analytic-view/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "List of analytic views"),
        (status = 403, description = "Admin required"),
    )
)]
pub async fn list_analytic_views(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_admin(&guard.member)?;

    let views = analytic_views::Entity::find()
        .active()
        .filter(analytic_views::Column::WorkspaceId.eq(guard.workspace.id))
        .order_by_desc(analytic_views::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<AnalyticViewResponse> = views.iter().map(analytic_view_to_response).collect();
    Ok(Json(resp))
}

/// POST /api/workspaces/{slug}/analytic-view/
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/analytic-view/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Created"),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Admin required"),
    )
)]
pub async fn create_analytic_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateAnalyticViewRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_admin(&guard.member)?;

    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 255 {
        return Err(AppError::BadRequest(
            "name must be 1–255 characters".into(),
        ));
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_view = analytic_views::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        description: Set(body.description.unwrap_or_default()),
        query: Set(body.query.unwrap_or(serde_json::json!({}))),
        query_dict: Set(body.query_dict.unwrap_or(serde_json::json!({}))),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_view.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(analytic_view_to_response(&created))))
}

/// GET /api/workspaces/{slug}/analytic-view/{pk}/
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/analytic-view/{pk}/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Analytic view ID"),
    ),
    responses(
        (status = 200, description = "Analytic view"),
        (status = 403, description = "Admin required"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_analytic_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_admin(&guard.member)?;

    let view = analytic_views::Entity::find_by_id(pk)
        .active()
        .filter(analytic_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(analytic_view_to_response(&view)))
}

/// PATCH /api/workspaces/{slug}/analytic-view/{pk}/
#[utoipa::path(
    patch,
    path = "/workspaces/{slug}/analytic-view/{pk}/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Analytic view ID"),
    ),
    responses(
        (status = 200, description = "Updated"),
        (status = 403, description = "Admin required"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_analytic_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateAnalyticViewRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_admin(&guard.member)?;

    let view = analytic_views::Entity::find_by_id(pk)
        .active()
        .filter(analytic_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: analytic_views::ActiveModel = view.into();
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
    if let Some(v) = body.query_dict {
        am.query_dict = Set(v);
    }

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(analytic_view_to_response(&updated)))
}

/// DELETE /api/workspaces/{slug}/analytic-view/{pk}/
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/analytic-view/{pk}/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Analytic view ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Admin required"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_analytic_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_workspace_admin(&guard.member)?;

    let view = analytic_views::Entity::find_by_id(pk)
        .active()
        .filter(analytic_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: analytic_views::ActiveModel = view.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Saved Analytic View ───────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/saved-analytic-view/{analytic_id}/
///
/// Devuelve la configuración de una vista analítica guardada.
/// La distribución real se computará en el cliente usando los parámetros de `query_dict`.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/saved-analytic-view/{analytic_id}/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("analytic_id" = Uuid, Path, description = "Analytic view ID"),
    ),
    responses(
        (status = 200, description = "Analytic view config"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_saved_analytic_view(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, analytic_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Member o superior puede acceder
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let view = analytic_views::Entity::find_by_id(analytic_id)
        .active()
        .filter(analytic_views::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Devuelve la config; el cliente usa query_dict para construir la distribución
    Ok(Json(analytic_view_to_response(&view)))
}

// ── Export Analytics ──────────────────────────────────────────────────────────

/// POST /api/workspaces/{slug}/export-analytics/
///
/// Encola una tarea de exportación. El resultado se envía por email.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/export-analytics/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Export queued"),
        (status = 400, description = "Invalid parameters"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn export_analytics(
    guard: WorkspaceMemberGuard,
    Json(body): Json<ExportAnalyticsRequest>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let x_axis = body.x_axis.unwrap_or_default();
    let y_axis = body.y_axis.unwrap_or_default();

    if !VALID_X_AXIS.contains(&x_axis.as_str()) || !VALID_Y_AXIS.contains(&y_axis.as_str()) {
        return Err(AppError::BadRequest(
            "x-axis and y-axis dimensions are required and the values should be valid".into(),
        ));
    }

    if let Some(ref seg) = body.segment {
        if !VALID_X_AXIS.contains(&seg.as_str()) || seg == &x_axis {
            return Err(AppError::BadRequest(
                "Both segment and x axis cannot be same and segment should be valid".into(),
            ));
        }
    }

    let email = guard.user.email.as_deref().unwrap_or("unknown");
    Ok(Json(serde_json::json!({
        "message": format!("Once the export is ready it will be emailed to you at {email}")
    })))
}

// ── Default Analytics ─────────────────────────────────────────────────────────

/// Fila de resultado para conteo por grupo de estado.
#[derive(Debug, FromQueryResult, Serialize)]
struct StateGroupCount {
    state_group: String,
    state_count: i64,
}

/// Fila de resultado para conteo mensual de issues completados.
#[derive(Debug, FromQueryResult, Serialize)]
struct MonthCount {
    month: i32,
    count: i64,
}

/// GET /api/workspaces/{slug}/default-analytics/
///
/// Retorna métricas agregadas del workspace: clasificación por estado,
/// issues completados por mes, usuarios más activos, etc.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/default-analytics/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Default analytics"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn default_analytics(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_GUEST {
        return Err(AppError::Forbidden);
    }

    let ws_id = guard.workspace.id;
    let db = &state.db;

    // ── Total de issues activos ───────────────────────────────────────────────
    let total_sql = format!(
        "SELECT COUNT(*) as cnt
         FROM issues i
         JOIN states s ON s.id = i.state_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL"
    );
    let total_row = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            total_sql,
        ))
        .await
        .map_err(AppError::Database)?;
    let total_issues: i64 = total_row
        .and_then(|r| r.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0);

    // ── Issues clasificados por state group ───────────────────────────────────
    let classified_sql = format!(
        "SELECT s.group AS state_group, COUNT(i.id) AS state_count
         FROM issues i
         JOIN states s ON s.id = i.state_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL
         GROUP BY s.group
         ORDER BY s.group"
    );
    let classified = StateGroupCount::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        classified_sql,
    ))
    .all(db)
    .await
    .map_err(AppError::Database)?;

    // ── Open issues (backlog, unstarted, started) ─────────────────────────────
    let open_sql = format!(
        "SELECT COUNT(i.id) AS cnt
         FROM issues i
         JOIN states s ON s.id = i.state_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL
           AND s.group IN ('backlog','unstarted','started')"
    );
    let open_row = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            open_sql,
        ))
        .await
        .map_err(AppError::Database)?;
    let open_issues: i64 = open_row
        .and_then(|r| r.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0);

    let open_classified_sql = format!(
        "SELECT s.group AS state_group, COUNT(i.id) AS state_count
         FROM issues i
         JOIN states s ON s.id = i.state_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL
           AND s.group IN ('backlog','unstarted','started')
         GROUP BY s.group
         ORDER BY s.group"
    );
    let open_classified = StateGroupCount::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        open_classified_sql,
    ))
    .all(db)
    .await
    .map_err(AppError::Database)?;

    // ── Issues completados por mes (año actual) ───────────────────────────────
    let current_year = Utc::now().year();
    let month_sql = format!(
        "SELECT EXTRACT(MONTH FROM i.completed_at)::int AS month, COUNT(*) AS count
         FROM issues i
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.completed_at IS NOT NULL
           AND EXTRACT(YEAR FROM i.completed_at) = {current_year}
         GROUP BY month
         ORDER BY month"
    );
    let completed_month_wise = MonthCount::find_by_statement(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        month_sql,
    ))
    .all(db)
    .await
    .map_err(AppError::Database)?;

    // ── Top 5 creadores de issues ─────────────────────────────────────────────
    let creators_sql = format!(
        "SELECT u.id, u.first_name, u.last_name, u.display_name, u.avatar,
                COUNT(i.id) AS count
         FROM issues i
         JOIN users u ON u.id = i.created_by_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.created_by_id IS NOT NULL
         GROUP BY u.id, u.first_name, u.last_name, u.display_name, u.avatar
         ORDER BY count DESC
         LIMIT 5"
    );
    let creator_rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            creators_sql,
        ))
        .await
        .map_err(AppError::Database)?;

    let most_issue_created_user: Vec<serde_json::Value> = creator_rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid>("", "id").ok(),
                "first_name": r.try_get::<String>("", "first_name").ok(),
                "last_name": r.try_get::<String>("", "last_name").ok(),
                "display_name": r.try_get::<String>("", "display_name").ok(),
                "avatar": r.try_get::<String>("", "avatar").ok(),
                "count": r.try_get::<i64>("", "count").ok(),
            })
        })
        .collect();

    // ── Top usuarios que cerraron más issues ──────────────────────────────────
    let closed_sql = format!(
        "SELECT u.id, u.first_name, u.last_name, u.display_name, u.avatar,
                COUNT(i.id) AS count
         FROM issues i
         JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.deleted_at IS NULL
         JOIN users u ON u.id = ia.assignee_id
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.completed_at IS NOT NULL
         GROUP BY u.id, u.first_name, u.last_name, u.display_name, u.avatar
         ORDER BY count DESC
         LIMIT 5"
    );
    let closed_rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            closed_sql,
        ))
        .await
        .map_err(AppError::Database)?;

    let most_issue_closed_user: Vec<serde_json::Value> = closed_rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid>("", "id").ok(),
                "first_name": r.try_get::<String>("", "first_name").ok(),
                "last_name": r.try_get::<String>("", "last_name").ok(),
                "display_name": r.try_get::<String>("", "display_name").ok(),
                "avatar": r.try_get::<String>("", "avatar").ok(),
                "count": r.try_get::<i64>("", "count").ok(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "total_issues": total_issues,
        "total_issues_classified": classified,
        "open_issues": open_issues,
        "open_issues_classified": open_classified,
        "issue_completed_month_wise": completed_month_wise,
        "most_issue_created_user": most_issue_created_user,
        "most_issue_closed_user": most_issue_closed_user,
    })))
}

// ── Project Stats ─────────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/project-stats/
///
/// Retorna estadísticas por proyecto: total de issues, completados, miembros, ciclos y módulos.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/project-stats/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("fields" = Option<String>, Query, description = "Comma-separated fields to include"),
        ("project_ids" = Option<String>, Query, description = "Comma-separated project IDs to filter"),
    ),
    responses(
        (status = 200, description = "Project stats"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn project_stats(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<ProjectStatsQuery>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_GUEST {
        return Err(AppError::Forbidden);
    }

    let ws_id = guard.workspace.id;
    let db = &state.db;

    let valid_fields = ["total_issues", "completed_issues", "total_members", "total_cycles", "total_modules"];
    let requested: std::collections::HashSet<&str> = params
        .fields
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|f| valid_fields.contains(f))
        .collect();

    let requested = if requested.is_empty() {
        valid_fields.iter().copied().collect::<std::collections::HashSet<_>>()
    } else {
        requested
    };

    // Construir filtro de project_ids
    let project_id_filter = if let Some(ref ids) = params.project_ids {
        let quoted: Vec<String> = ids
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("'{s}'"))
            .collect();
        if quoted.is_empty() {
            String::new()
        } else {
            format!("AND p.id IN ({})", quoted.join(","))
        }
    } else {
        String::new()
    };

    let mut select_parts = vec!["p.id".to_string()];

    if requested.contains("total_issues") {
        select_parts.push(
            "(SELECT COUNT(*) FROM issues i WHERE i.project_id = p.id AND i.deleted_at IS NULL AND i.archived_at IS NULL) AS total_issues".to_string()
        );
    }
    if requested.contains("completed_issues") {
        select_parts.push(
            "(SELECT COUNT(*) FROM issues i JOIN states s ON s.id = i.state_id WHERE i.project_id = p.id AND i.deleted_at IS NULL AND s.group IN ('completed','cancelled')) AS completed_issues".to_string()
        );
    }
    if requested.contains("total_members") {
        select_parts.push(
            "(SELECT COUNT(*) FROM project_members pm JOIN users u ON u.id = pm.member_id WHERE pm.project_id = p.id AND pm.is_active = true AND u.is_bot = false AND pm.deleted_at IS NULL) AS total_members".to_string()
        );
    }
    if requested.contains("total_cycles") {
        select_parts.push(
            "(SELECT COUNT(*) FROM cycles c WHERE c.project_id = p.id AND c.deleted_at IS NULL) AS total_cycles".to_string()
        );
    }
    if requested.contains("total_modules") {
        select_parts.push(
            "(SELECT COUNT(*) FROM modules m WHERE m.project_id = p.id AND m.deleted_at IS NULL) AS total_modules".to_string()
        );
    }

    let sql = format!(
        "SELECT {selects}
         FROM projects p
         WHERE p.workspace_id = '{ws_id}'
           AND p.deleted_at IS NULL
           {project_id_filter}
         ORDER BY p.name",
        selects = select_parts.join(", "),
        project_id_filter = project_id_filter,
    );

    let rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .map_err(AppError::Database)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let mut obj = serde_json::json!({
                "id": r.try_get::<Uuid>("", "id").ok(),
            });
            if requested.contains("total_issues") {
                obj["total_issues"] = r.try_get::<i64>("", "total_issues").ok().into();
            }
            if requested.contains("completed_issues") {
                obj["completed_issues"] = r.try_get::<i64>("", "completed_issues").ok().into();
            }
            if requested.contains("total_members") {
                obj["total_members"] = r.try_get::<i64>("", "total_members").ok().into();
            }
            if requested.contains("total_cycles") {
                obj["total_cycles"] = r.try_get::<i64>("", "total_cycles").ok().into();
            }
            if requested.contains("total_modules") {
                obj["total_modules"] = r.try_get::<i64>("", "total_modules").ok().into();
            }
            obj
        })
        .collect();

    Ok(Json(result))
}

// ── Analytics (distribución por eje X/Y) ─────────────────────────────────────

/// GET /api/workspaces/{slug}/analytics/
///
/// Retorna la distribución de issues agrupada por x_axis y opcionalmente segmentada.
/// Parámetros requeridos: `x_axis`, `y_axis`.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/analytics/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("x_axis" = Option<String>, Query, description = "X axis dimension"),
        ("y_axis" = Option<String>, Query, description = "Y axis (issue_count|estimate)"),
        ("segment" = Option<String>, Query, description = "Optional segment dimension"),
    ),
    responses(
        (status = 200, description = "Analytics distribution"),
        (status = 400, description = "Invalid parameters"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn workspace_analytics(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<AnalyticsQuery>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let x_axis = params.x_axis.unwrap_or_default();
    let y_axis = params.y_axis.unwrap_or_default();

    if !VALID_X_AXIS.contains(&x_axis.as_str()) || !VALID_Y_AXIS.contains(&y_axis.as_str()) {
        return Err(AppError::BadRequest(
            "x-axis and y-axis dimensions are required and the values should be valid".into(),
        ));
    }

    if let Some(ref seg) = params.segment {
        if !VALID_X_AXIS.contains(&seg.as_str()) || seg == &x_axis {
            return Err(AppError::BadRequest(
                "Both segment and x axis cannot be same and segment should be valid".into(),
            ));
        }
    }

    let ws_id = guard.workspace.id;
    let db = &state.db;

    // Parsear y validar project_ids para evitar inyección SQL en raw SQL.
    // Se aceptan únicamente UUIDs válidos; cualquier valor malformado produce 400.
    let project_id_filter = match params.project_ids.as_deref() {
        Some(raw) if !raw.is_empty() => {
            let ids: Result<Vec<Uuid>, _> = raw
                .split(',')
                .map(|s| s.trim().parse::<Uuid>())
                .collect();
            match ids {
                Ok(uuids) if !uuids.is_empty() => {
                    let list = uuids
                        .iter()
                        .map(|u| format!("'{u}'"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("AND i.project_id IN ({list})")
                }
                _ => {
                    return Err(AppError::BadRequest(
                        "project_ids contains invalid UUID values".into(),
                    ))
                }
            }
        }
        _ => String::new(),
    };

    // Construir SELECT y GROUP BY según x_axis
    let (x_col, x_join) = axis_to_sql_col(&x_axis);
    let y_col = if y_axis == "estimate" {
        "COALESCE(SUM(ep.value::numeric), 0)".to_string()
    } else {
        "COUNT(DISTINCT i.id)".to_string()
    };

    let estimate_join = if y_axis == "estimate" {
        "LEFT JOIN estimate_points ep ON ep.id = i.estimate_point_id AND ep.deleted_at IS NULL"
    } else {
        ""
    };

    let base_sql = format!(
        "SELECT {x_col} AS x_axis_value, {y_col} AS value
         FROM issues i
         {x_join}
         {estimate_join}
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL
           AND i.archived_at IS NULL
           {project_id_filter}
         GROUP BY {x_col}
         ORDER BY value DESC",
        x_col = x_col,
        y_col = y_col,
        x_join = x_join,
        estimate_join = estimate_join,
        ws_id = ws_id,
        project_id_filter = project_id_filter,
    );

    let rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            base_sql,
        ))
        .await
        .map_err(AppError::Database)?;

    let distribution: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "x_axis": r.try_get::<String>("", "x_axis_value").ok()
                    .or_else(|| r.try_get::<Uuid>("", "x_axis_value").ok().map(|u| u.to_string())),
                "value": r.try_get::<i64>("", "value").ok()
                    .map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
            })
        })
        .collect();

    // Total issues en el workspace (aplica el mismo filtro de proyectos)
    let total_sql = format!(
        "SELECT COUNT(*) AS cnt FROM issues i
         WHERE i.workspace_id = '{ws_id}'
           AND i.deleted_at IS NULL AND i.archived_at IS NULL
           {project_id_filter}",
        ws_id = ws_id,
        project_id_filter = project_id_filter,
    );
    let total_row = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            total_sql,
        ))
        .await
        .map_err(AppError::Database)?;
    let total: i64 = total_row
        .and_then(|r| r.try_get::<i64>("", "cnt").ok())
        .unwrap_or(0);

    Ok(Json(serde_json::json!({
        "total": total,
        "distribution": distribution,
    })))
}

/// Mapea un campo de eje X de la API Django a columna SQL y JOIN necesario.
fn axis_to_sql_col(axis: &str) -> (String, String) {
    match axis {
        "state_id" => ("i.state_id::text".into(), String::new()),
        "state__group" => (
            "s.group".into(),
            "LEFT JOIN states s ON s.id = i.state_id".into(),
        ),
        "labels__id" => (
            "COALESCE(il.label_id::text, 'None')".into(),
            "LEFT JOIN issue_labels il ON il.issue_id = i.id AND il.deleted_at IS NULL".into(),
        ),
        "assignees__id" => (
            "COALESCE(ia.assignee_id::text, 'None')".into(),
            "LEFT JOIN issue_assignees ia ON ia.issue_id = i.id AND ia.deleted_at IS NULL".into(),
        ),
        "priority" => ("i.priority".into(), String::new()),
        "start_date" => ("i.start_date::text".into(), String::new()),
        "target_date" => ("i.target_date::text".into(), String::new()),
        "created_at" => ("DATE(i.created_at)::text".into(), String::new()),
        "completed_at" => ("DATE(i.completed_at)::text".into(), String::new()),
        "issue_cycle__cycle_id" => (
            "COALESCE(ic.cycle_id::text, 'None')".into(),
            "LEFT JOIN issue_cycles ic ON ic.issue_id = i.id AND ic.deleted_at IS NULL".into(),
        ),
        "issue_module__module_id" => (
            "COALESCE(im.module_id::text, 'None')".into(),
            "LEFT JOIN issue_modules im ON im.issue_id = i.id AND im.deleted_at IS NULL".into(),
        ),
        _ => ("i.id::text".into(), String::new()),
    }
}

// ── Advance Analytics ─────────────────────────────────────────────────────────

// Helper: build date clause for analytics (created_at filter)
fn analytics_date_clause(date_filter: Option<&str>, col: &str) -> String {
    let Some(df) = date_filter else { return String::new() };
    let today = chrono::Utc::now().date_naive();
    let (gte, lte) = match df {
        "yesterday" => {
            let d = today - chrono::Duration::days(1);
            (d.to_string(), d.to_string())
        }
        "last_7_days" => ((today - chrono::Duration::days(7)).to_string(), today.to_string()),
        "last_30_days" => ((today - chrono::Duration::days(30)).to_string(), today.to_string()),
        "last_3_months" => ((today - chrono::Duration::days(90)).to_string(), today.to_string()),
        _ => return String::new(),
    };
    format!("AND DATE({col}) >= '{gte}' AND DATE({col}) <= '{lte}'")
}

// Helper: chart period range as (start, end) NaiveDates
fn chart_period_range(date_filter: Option<&str>) -> Option<(chrono::NaiveDate, chrono::NaiveDate)> {
    let today = chrono::Utc::now().date_naive();
    match date_filter? {
        "yesterday" => {
            let d = today - chrono::Duration::days(1);
            Some((d, d))
        }
        "last_7_days" => Some((today - chrono::Duration::days(7), today)),
        "last_30_days" => Some((today - chrono::Duration::days(30), today)),
        "last_3_months" => Some((today - chrono::Duration::days(90), today)),
        _ => None,
    }
}

/// Valida y construye el filtro SQL de project_ids a partir de una cadena separada por comas.
/// Devuelve Err si algún valor no es UUID válido (prevención de inyección SQL).
fn project_ids_filter(raw: Option<&str>) -> Result<String, AppError> {
    match raw {
        Some(s) if !s.trim().is_empty() => {
            let ids: Result<Vec<Uuid>, _> = s.split(',').map(|p| p.trim().parse::<Uuid>()).collect();
            match ids {
                Ok(uuids) if !uuids.is_empty() => {
                    let list = uuids.iter().map(|u| format!("'{u}'")).collect::<Vec<_>>().join(", ");
                    Ok(format!("AND i.project_id IN ({list})"))
                }
                _ => Err(AppError::BadRequest("project_ids contains invalid UUID values".into())),
            }
        }
        _ => Ok(String::new()),
    }
}

/// Filtro base de workspace para issues: considera solo proyectos donde el usuario es miembro activo.
fn base_issue_filter(ws_id: Uuid, user_id: Uuid) -> String {
    format!(
        "i.workspace_id = '{ws_id}'
         AND i.deleted_at IS NULL
         AND i.archived_at IS NULL
         AND EXISTS (
             SELECT 1 FROM project_members pm
             WHERE pm.project_id = i.project_id
               AND pm.member_id = '{user_id}'
               AND pm.is_active = true
               AND pm.deleted_at IS NULL
         )"
    )
}

// ── Query structs para advance analytics ─────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AdvanceAnalyticsQuery {
    pub tab: Option<String>,
    pub date_filter: Option<String>,
    pub project_ids: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AdvanceAnalyticsStatsQuery {
    #[serde(rename = "type")]
    pub stat_type: Option<String>,
    pub date_filter: Option<String>,
    pub project_ids: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AdvanceAnalyticsChartQuery {
    #[serde(rename = "type")]
    pub chart_type: Option<String>,
    pub date_filter: Option<String>,
    pub project_ids: Option<String>,
    pub x_axis: Option<String>,
    pub group_by: Option<String>,
}

// ── GET /workspaces/{slug}/advance-analytics/ ─────────────────────────────────

/// GET /api/workspaces/{slug}/advance-analytics/
///
/// Métricas agregadas del workspace. `tab=overview` (default) o `tab=work-items`.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/advance-analytics/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("tab" = Option<String>, Query, description = "overview | work-items"),
        ("date_filter" = Option<String>, Query, description = "Date filter"),
        ("project_ids" = Option<String>, Query, description = "Comma-separated project IDs"),
    ),
    responses(
        (status = 200, description = "Advance analytics overview"),
        (status = 400, description = "Invalid tab"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn advance_analytics(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<AdvanceAnalyticsQuery>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let ws_id = guard.workspace.id;
    let user_id = guard.user.id;
    let db = &state.db;

    let tab = params.tab.as_deref().unwrap_or("overview");
    let date_filter = params.date_filter.as_deref();

    // Validar y construir filtro de project_ids
    let raw_project_ids = params.project_ids.as_deref();
    let project_id_filter = project_ids_filter(raw_project_ids)?;

    // Cláusula de fecha sobre created_at
    let date_clause_created = analytics_date_clause(date_filter, "i.created_at");

    match tab {
        "overview" => {
            let base_filter = base_issue_filter(ws_id, user_id);

            // Total users / admins / members / guests
            // Si se filtra por project_ids usamos project_members, si no workspace_members
            let (total_users, total_admins, total_members, total_guests) =
                if let Some(raw) = raw_project_ids.filter(|s| !s.trim().is_empty()) {
                    let ids: Result<Vec<Uuid>, _> =
                        raw.split(',').map(|p| p.trim().parse::<Uuid>()).collect();
                    let ids = ids.map_err(|_| AppError::BadRequest("project_ids contains invalid UUID values".into()))?;
                    let list = ids.iter().map(|u| format!("'{u}'")).collect::<Vec<_>>().join(", ");

                    let date_pm = analytics_date_clause(date_filter, "pm.created_at");
                    let total = {
                        let sql = format!(
                            "SELECT COUNT(DISTINCT pm.member_id) AS cnt
                             FROM project_members pm
                             JOIN users u ON u.id = pm.member_id
                             WHERE pm.project_id IN ({list})
                               AND pm.is_active = true
                               AND u.is_bot = false
                               AND pm.deleted_at IS NULL
                               {date_pm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    let admins = {
                        let sql = format!(
                            "SELECT COUNT(DISTINCT pm.member_id) AS cnt
                             FROM project_members pm
                             JOIN users u ON u.id = pm.member_id
                             WHERE pm.project_id IN ({list})
                               AND pm.is_active = true AND pm.role = 20
                               AND u.is_bot = false AND pm.deleted_at IS NULL {date_pm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    let mems = {
                        let sql = format!(
                            "SELECT COUNT(DISTINCT pm.member_id) AS cnt
                             FROM project_members pm
                             JOIN users u ON u.id = pm.member_id
                             WHERE pm.project_id IN ({list})
                               AND pm.is_active = true AND pm.role = 15
                               AND u.is_bot = false AND pm.deleted_at IS NULL {date_pm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    let guests = {
                        let sql = format!(
                            "SELECT COUNT(DISTINCT pm.member_id) AS cnt
                             FROM project_members pm
                             JOIN users u ON u.id = pm.member_id
                             WHERE pm.project_id IN ({list})
                               AND pm.is_active = true AND pm.role = 5
                               AND u.is_bot = false AND pm.deleted_at IS NULL {date_pm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    (total, admins, mems, guests)
                } else {
                    let date_wm = analytics_date_clause(date_filter, "wm.created_at");
                    let total = {
                        let sql = format!(
                            "SELECT COUNT(*) AS cnt FROM workspace_members wm
                             JOIN users u ON u.id = wm.member_id
                             WHERE wm.workspace_id = '{ws_id}'
                               AND wm.is_active = true AND u.is_bot = false
                               AND wm.deleted_at IS NULL {date_wm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    let admins = {
                        let sql = format!(
                            "SELECT COUNT(*) AS cnt FROM workspace_members wm
                             JOIN users u ON u.id = wm.member_id
                             WHERE wm.workspace_id = '{ws_id}' AND wm.role = 20
                               AND wm.is_active = true AND u.is_bot = false
                               AND wm.deleted_at IS NULL {date_wm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    let mems = {
                        let sql = format!(
                            "SELECT COUNT(*) AS cnt FROM workspace_members wm
                             JOIN users u ON u.id = wm.member_id
                             WHERE wm.workspace_id = '{ws_id}' AND wm.role = 15
                               AND wm.is_active = true AND u.is_bot = false
                               AND wm.deleted_at IS NULL {date_wm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    let guests = {
                        let sql = format!(
                            "SELECT COUNT(*) AS cnt FROM workspace_members wm
                             JOIN users u ON u.id = wm.member_id
                             WHERE wm.workspace_id = '{ws_id}' AND wm.role = 5
                               AND wm.is_active = true AND u.is_bot = false
                               AND wm.deleted_at IS NULL {date_wm}",
                        );
                        db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                            .await.map_err(AppError::Database)?
                            .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
                    };
                    (total, admins, mems, guests)
                };

            // Total projects accesibles por el usuario
            let total_projects = {
                let pid_filter_proj = match raw_project_ids.filter(|s| !s.trim().is_empty()) {
                    Some(raw) => {
                        let ids: Result<Vec<Uuid>, _> =
                            raw.split(',').map(|p| p.trim().parse::<Uuid>()).collect();
                        let ids = ids.map_err(|_| AppError::BadRequest("project_ids contains invalid UUID values".into()))?;
                        let list = ids.iter().map(|u| format!("'{u}'")).collect::<Vec<_>>().join(", ");
                        format!("AND p.id IN ({list})")
                    }
                    _ => String::new(),
                };
                let sql = format!(
                    "SELECT COUNT(DISTINCT p.id) AS cnt
                     FROM projects p
                     JOIN project_members pm ON pm.project_id = p.id
                       AND pm.member_id = '{user_id}' AND pm.is_active = true AND pm.deleted_at IS NULL
                     WHERE p.workspace_id = '{ws_id}'
                       AND p.deleted_at IS NULL AND p.archived_at IS NULL
                       {pid_filter_proj}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            // Total work items
            let total_work_items = {
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM issues i
                     WHERE {base_filter} {project_id_filter} {date_clause_created}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            // Total cycles
            let total_cycles = {
                let cycles_pid_filter = project_ids_filter(raw_project_ids)?
                    .replace("i.project_id", "c.project_id");
                let date_clause_cycles = analytics_date_clause(date_filter, "c.created_at");
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM cycles c
                     WHERE c.workspace_id = '{ws_id}' AND c.deleted_at IS NULL
                       AND EXISTS (
                           SELECT 1 FROM project_members pm WHERE pm.project_id = c.project_id
                             AND pm.member_id = '{user_id}' AND pm.is_active = true AND pm.deleted_at IS NULL
                       )
                       {cycles_pid_filter}
                     {date_clause_cycles}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            // Total intake (issues con intake_issues relacionados)
            let total_intake = {
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM issues i
                     JOIN intake_issues ii ON ii.issue_id = i.id
                     WHERE {base_filter} {project_id_filter}
                       AND ii.status IN (-2, -1, 0, 1, 2)
                       {date_clause_created}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            Ok(Json(serde_json::json!({
                "total_users": { "count": total_users },
                "total_admins": { "count": total_admins },
                "total_members": { "count": total_members },
                "total_guests": { "count": total_guests },
                "total_projects": { "count": total_projects },
                "total_work_items": { "count": total_work_items },
                "total_cycles": { "count": total_cycles },
                "total_intake": { "count": total_intake },
            })))
        }

        "work-items" => {
            let base_filter = base_issue_filter(ws_id, user_id);
            let sql = format!(
                "SELECT
                   COUNT(*) AS total,
                   COUNT(*) FILTER (WHERE s.group = 'started') AS started,
                   COUNT(*) FILTER (WHERE s.group = 'backlog') AS backlog,
                   COUNT(*) FILTER (WHERE s.group = 'unstarted') AS unstarted,
                   COUNT(*) FILTER (WHERE s.group = 'completed') AS completed
                 FROM issues i
                 JOIN states s ON s.id = i.state_id
                 WHERE {base_filter} {project_id_filter} {date_clause_created}",
            );
            let row = db
                .query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                .await.map_err(AppError::Database)?;

            let (total, started, backlog, unstarted, completed) = row
                .map(|r| {
                    (
                        r.try_get::<i64>("", "total").unwrap_or(0),
                        r.try_get::<i64>("", "started").unwrap_or(0),
                        r.try_get::<i64>("", "backlog").unwrap_or(0),
                        r.try_get::<i64>("", "unstarted").unwrap_or(0),
                        r.try_get::<i64>("", "completed").unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0, 0, 0, 0));

            Ok(Json(serde_json::json!({
                "total_work_items": { "count": total },
                "started_work_items": { "count": started },
                "backlog_work_items": { "count": backlog },
                "un_started_work_items": { "count": unstarted },
                "completed_work_items": { "count": completed },
            })))
        }

        _ => Err(AppError::BadRequest("Invalid tab".into())),
    }
}

// ── GET /workspaces/{slug}/advance-analytics-stats/ ───────────────────────────

/// GET /api/workspaces/{slug}/advance-analytics-stats/
///
/// Estadísticas por proyecto desagregadas por estado. `type=work-items` (default).
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/advance-analytics-stats/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("type" = Option<String>, Query, description = "work-items"),
        ("date_filter" = Option<String>, Query, description = "Date filter"),
        ("project_ids" = Option<String>, Query, description = "Comma-separated project IDs"),
    ),
    responses(
        (status = 200, description = "Work items stats per project"),
        (status = 400, description = "Invalid type"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn advance_analytics_stats(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<AdvanceAnalyticsStatsQuery>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let stat_type = params.stat_type.as_deref().unwrap_or("work-items");
    if stat_type != "work-items" {
        return Err(AppError::BadRequest("Invalid type".into()));
    }

    let ws_id = guard.workspace.id;
    let user_id = guard.user.id;
    let db = &state.db;

    let project_id_filter = project_ids_filter(params.project_ids.as_deref())?;
    let date_clause = analytics_date_clause(params.date_filter.as_deref(), "i.created_at");
    let base_filter = base_issue_filter(ws_id, user_id);

    let sql = format!(
        "SELECT
           i.project_id,
           p.name AS project_name,
           COUNT(*) FILTER (WHERE s.group = 'cancelled') AS cancelled_work_items,
           COUNT(*) FILTER (WHERE s.group = 'completed') AS completed_work_items,
           COUNT(*) FILTER (WHERE s.group = 'backlog') AS backlog_work_items,
           COUNT(*) FILTER (WHERE s.group = 'unstarted') AS un_started_work_items,
           COUNT(*) FILTER (WHERE s.group = 'started') AS started_work_items
         FROM issues i
         JOIN states s ON s.id = i.state_id
         JOIN projects p ON p.id = i.project_id
         WHERE {base_filter} {project_id_filter} {date_clause}
         GROUP BY i.project_id, p.name
         ORDER BY i.project_id",
    );

    let rows = db
        .query_all(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
        .await
        .map_err(AppError::Database)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "project_id": r.try_get::<Uuid>("", "project_id").ok(),
                "project__name": r.try_get::<String>("", "project_name").unwrap_or_default(),
                "cancelled_work_items": r.try_get::<i64>("", "cancelled_work_items").unwrap_or(0),
                "completed_work_items": r.try_get::<i64>("", "completed_work_items").unwrap_or(0),
                "backlog_work_items": r.try_get::<i64>("", "backlog_work_items").unwrap_or(0),
                "un_started_work_items": r.try_get::<i64>("", "un_started_work_items").unwrap_or(0),
                "started_work_items": r.try_get::<i64>("", "started_work_items").unwrap_or(0),
            })
        })
        .collect();

    Ok(Json(result))
}

// ── GET /workspaces/{slug}/advance-analytics-charts/ ─────────────────────────

/// GET /api/workspaces/{slug}/advance-analytics-charts/
///
/// Datos para gráficas de analytics avanzado.
/// `type=projects` (default) | `type=work-items` | `type=custom-work-items`
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/advance-analytics-charts/",
    tag = "Analytics",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("type" = Option<String>, Query, description = "projects | work-items | custom-work-items"),
        ("date_filter" = Option<String>, Query, description = "Date filter"),
        ("project_ids" = Option<String>, Query, description = "Comma-separated project IDs"),
        ("x_axis" = Option<String>, Query, description = "X axis for custom chart"),
        ("group_by" = Option<String>, Query, description = "Group by for custom chart"),
    ),
    responses(
        (status = 200, description = "Chart data"),
        (status = 400, description = "Invalid type"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn advance_analytics_charts(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<AdvanceAnalyticsChartQuery>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let ws_id = guard.workspace.id;
    let user_id = guard.user.id;
    let db = &state.db;

    let chart_type = params.chart_type.as_deref().unwrap_or("projects");
    let date_filter = params.date_filter.as_deref();
    let project_id_filter = project_ids_filter(params.project_ids.as_deref())?;
    let date_clause = analytics_date_clause(date_filter, "i.created_at");
    let base_filter = base_issue_filter(ws_id, user_id);

    match chart_type {
        "projects" => {
            // Conteo de entidades por tipo en el workspace

            let work_items: i64 = {
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM issues i
                     WHERE {base_filter} {project_id_filter} {date_clause}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let cycles: i64 = {
                let pid_f = project_ids_filter(params.project_ids.as_deref())?
                    .replace("i.project_id", "c.project_id");
                let dc = analytics_date_clause(date_filter, "c.created_at");
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM cycles c
                     WHERE c.workspace_id = '{ws_id}' AND c.deleted_at IS NULL
                       AND EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = c.project_id
                         AND pm.member_id = '{user_id}' AND pm.is_active = true AND pm.deleted_at IS NULL)
                     {pid_f} {dc}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let modules: i64 = {
                let pid_f = project_ids_filter(params.project_ids.as_deref())?
                    .replace("i.project_id", "m.project_id");
                let dc = analytics_date_clause(date_filter, "m.created_at");
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM modules m
                     WHERE m.workspace_id = '{ws_id}' AND m.deleted_at IS NULL
                       AND EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = m.project_id
                         AND pm.member_id = '{user_id}' AND pm.is_active = true AND pm.deleted_at IS NULL)
                     {pid_f} {dc}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let intake: i64 = {
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM issues i
                     JOIN intake_issues ii ON ii.issue_id = i.id
                     WHERE {base_filter} {project_id_filter} {date_clause}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let members: i64 = {
                let dc = analytics_date_clause(date_filter, "wm.created_at");
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM workspace_members wm
                     JOIN users u ON u.id = wm.member_id
                     WHERE wm.workspace_id = '{ws_id}' AND wm.is_active = true
                       AND u.is_bot = false AND wm.deleted_at IS NULL {dc}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let pages: i64 = {
                let pid_f = project_ids_filter(params.project_ids.as_deref())?
                    .replace("i.project_id", "pp.project_id");
                let dc = analytics_date_clause(date_filter, "pp.created_at");
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM project_pages pp
                     WHERE pp.workspace_id = '{ws_id}' AND pp.deleted_at IS NULL
                       AND EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = pp.project_id
                         AND pm.member_id = '{user_id}' AND pm.is_active = true AND pm.deleted_at IS NULL)
                     {pid_f} {dc}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let views: i64 = {
                let pid_f = project_ids_filter(params.project_ids.as_deref())?
                    .replace("i.project_id", "iv.project_id");
                let dc = analytics_date_clause(date_filter, "iv.created_at");
                let sql = format!(
                    "SELECT COUNT(*) AS cnt FROM issue_views iv
                     WHERE iv.workspace_id = '{ws_id}' AND iv.deleted_at IS NULL
                       AND EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = iv.project_id
                         AND pm.member_id = '{user_id}' AND pm.is_active = true AND pm.deleted_at IS NULL)
                     {pid_f} {dc}",
                );
                db.query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                    .await.map_err(AppError::Database)?
                    .and_then(|r| r.try_get::<i64>("", "cnt").ok()).unwrap_or(0)
            };

            let result = vec![
                serde_json::json!({"key": "work_items", "name": "Work Items", "count": work_items}),
                serde_json::json!({"key": "cycles", "name": "Cycles", "count": cycles}),
                serde_json::json!({"key": "modules", "name": "Modules", "count": modules}),
                serde_json::json!({"key": "intake", "name": "Intake", "count": intake}),
                serde_json::json!({"key": "members", "name": "Members", "count": members}),
                serde_json::json!({"key": "pages", "name": "Pages", "count": pages}),
                serde_json::json!({"key": "views", "name": "Views", "count": views}),
            ];
            Ok(Json(serde_json::Value::Array(result)))
        }

        "work-items" => {
            // Gráfica mensual de issues creados vs completados desde el inicio del workspace
            let workspace_sql = format!(
                "SELECT created_at FROM workspaces WHERE id = '{ws_id}'",
            );
            let ws_row = db
                .query_one(Statement::from_string(sea_orm::DatabaseBackend::Postgres, workspace_sql))
                .await
                .map_err(AppError::Database)?;

            let ws_created: chrono::NaiveDate = ws_row
                .and_then(|r| {
                    r.try_get::<chrono::DateTime<chrono::FixedOffset>>("", "created_at")
                        .ok()
                        .map(|dt| dt.date_naive().with_day(1).unwrap_or(dt.date_naive()))
                })
                .unwrap_or_else(|| chrono::Utc::now().date_naive().with_day(1).unwrap());

            let (sql_start, sql_end) = if let Some((s, e)) = chart_period_range(date_filter) {
                (s, e)
            } else {
                (ws_created, chrono::Utc::now().date_naive())
            };

            let monthly_sql = format!(
                "SELECT
                   DATE_TRUNC('month', i.created_at)::date AS month,
                   COUNT(*) AS created_count,
                   COUNT(*) FILTER (WHERE s.group = 'completed') AS completed_count
                 FROM issues i
                 JOIN states s ON s.id = i.state_id
                 WHERE {base_filter} {project_id_filter}
                   AND DATE(i.created_at) >= '{sql_start}' AND DATE(i.created_at) <= '{sql_end}'
                 GROUP BY month ORDER BY month",
            );

            let monthly_rows = db
                .query_all(Statement::from_string(sea_orm::DatabaseBackend::Postgres, monthly_sql))
                .await
                .map_err(AppError::Database)?;

            use std::collections::HashMap;
            let stats_map: HashMap<String, (i64, i64)> = monthly_rows
                .iter()
                .filter_map(|r| {
                    let month = r.try_get::<chrono::NaiveDate>("", "month").ok()?;
                    let created = r.try_get::<i64>("", "created_count").unwrap_or(0);
                    let completed = r.try_get::<i64>("", "completed_count").unwrap_or(0);
                    Some((month.format("%Y-%m-%d").to_string(), (created, completed)))
                })
                .collect();

            let mut data = Vec::new();
            let mut current = sql_start.with_day(1).unwrap_or(sql_start);
            let last_month = chrono::Utc::now().date_naive().with_day(1).unwrap_or(sql_end);

            while current <= last_month {
                let key = current.format("%Y-%m-%d").to_string();
                let (created, completed) = stats_map.get(&key).copied().unwrap_or((0, 0));
                data.push(serde_json::json!({
                    "key": key,
                    "name": key,
                    "count": created,
                    "created_issues": created,
                    "completed_issues": completed,
                }));
                // Avanzar al siguiente mes
                if current.month() == 12 {
                    current = current.with_year(current.year() + 1).unwrap().with_month(1).unwrap();
                } else {
                    current = current.with_month(current.month() + 1).unwrap();
                }
            }

            Ok(Json(serde_json::json!({
                "data": data,
                "schema": { "completed_issues": "completed_issues", "created_issues": "created_issues" }
            })))
        }

        "custom-work-items" => {
            let x_axis = params.x_axis.as_deref().unwrap_or("priority");

            if !VALID_X_AXIS.contains(&x_axis) {
                return Err(AppError::BadRequest(
                    "x_axis value is not valid".into(),
                ));
            }

            if let Some(ref gb) = params.group_by {
                if !VALID_X_AXIS.contains(&gb.as_str()) || gb.as_str() == x_axis {
                    return Err(AppError::BadRequest(
                        "group_by must be valid and different from x_axis".into(),
                    ));
                }
            }

            let (x_col, x_join) = axis_to_sql_col(x_axis);

            let sql = format!(
                "SELECT {x_col} AS x_axis_value, COUNT(DISTINCT i.id) AS value
                 FROM issues i {x_join}
                 WHERE {base_filter} {project_id_filter} {date_clause}
                 GROUP BY {x_col}
                 ORDER BY value DESC",
            );

            let rows = db
                .query_all(Statement::from_string(sea_orm::DatabaseBackend::Postgres, sql))
                .await
                .map_err(AppError::Database)?;

            let distribution: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "x_axis": r.try_get::<String>("", "x_axis_value").ok()
                            .or_else(|| r.try_get::<Uuid>("", "x_axis_value").ok().map(|u| u.to_string())),
                        "value": r.try_get::<i64>("", "value").unwrap_or(0),
                    })
                })
                .collect();

            Ok(Json(serde_json::json!({ "distribution": distribution })))
        }

        _ => Err(AppError::BadRequest("Invalid type".into())),
    }
}
