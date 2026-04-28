// src/routes/cycles.rs
//! Cycles endpoints.
//!
//! Implemented endpoints:
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
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait,
    FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, Statement,
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
    // Frontend ICycle uses "created_by" not "created_by_id"
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    // Frontend ICycle uses "updated_by" not "updated_by_id"
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub is_favorite: bool,
    pub view_props: serde_json::Value,
    pub progress_snapshot: serde_json::Value,
    pub version: i32,
    // TProgressSnapshot fields — annotated by Django, computed here per cycle
    pub total_issues: i64,
    pub completed_issues: i64,
    pub cancelled_issues: i64,
    pub started_issues: i64,
    pub unstarted_issues: i64,
    pub backlog_issues: i64,
    pub total_estimate_points: i64,
    pub completed_estimate_points: i64,
    pub cancelled_estimate_points: i64,
    pub started_estimate_points: i64,
    pub unstarted_estimate_points: i64,
    pub backlog_estimate_points: i64,
}

impl CycleResponse {
    pub fn from_model(m: cycles::Model) -> Self {
        // Status derived from dates — reflects Django CycleViewSet logic
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
            is_favorite: false,
            view_props: m.view_props,
            progress_snapshot: m.progress_snapshot,
            version: m.version,
            // Populated by enrich_cycle_counts after batch query
            total_issues: 0,
            completed_issues: 0,
            cancelled_issues: 0,
            started_issues: 0,
            unstarted_issues: 0,
            backlog_issues: 0,
            total_estimate_points: 0,
            completed_estimate_points: 0,
            cancelled_estimate_points: 0,
            started_estimate_points: 0,
            unstarted_estimate_points: 0,
            backlog_estimate_points: 0,
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

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Rows returned by the issue-counts SQL.
#[derive(sea_orm::FromQueryResult)]
struct CycleCountRow {
    cycle_id: Uuid,
    group_name: String,
    cnt: i64,
}

/// Batch-load TProgressSnapshot counts (total/completed/cancelled/started/
/// unstarted/backlog) for a set of cycles, using a single SQL query.
/// Mirror of Django `CycleIssueGroupedCount` annotation.
async fn enrich_cycle_counts(
    db: &sea_orm::DatabaseConnection,
    mut cycles: Vec<CycleResponse>,
) -> Result<Vec<CycleResponse>, AppError> {
    if cycles.is_empty() {
        return Ok(cycles);
    }

    let cycle_ids: Vec<Uuid> = cycles.iter().map(|c| c.id).collect();
    let placeholders = cycle_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        r#"SELECT
             ci.cycle_id,
             COALESCE(s.group, 'backlog') AS group_name,
             COUNT(ci.issue_id)::BIGINT   AS cnt
           FROM cycle_issues ci
           JOIN issues        i  ON i.id = ci.issue_id   AND i.deleted_at IS NULL
           LEFT JOIN states   s  ON s.id = i.state_id
           WHERE ci.cycle_id IN ({})
             AND ci.deleted_at IS NULL
             AND i.archived_at  IS NULL
             AND i.is_draft     = FALSE
           GROUP BY ci.cycle_id, COALESCE(s.group, 'backlog')"#,
        placeholders
    );

    let values: Vec<sea_orm::Value> = cycle_ids
        .iter()
        .map(|id| sea_orm::Value::Uuid(Some(Box::new(*id))))
        .collect();

    let rows = CycleCountRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        &sql,
        values,
    ))
    .all(db)
    .await
    .map_err(AppError::Database)?;

    // Build a map: cycle_id → (total, completed, cancelled, started, unstarted, backlog)
    let mut counts: std::collections::HashMap<Uuid, [i64; 6]> = std::collections::HashMap::new();
    for row in rows {
        let entry = counts.entry(row.cycle_id).or_insert([0i64; 6]);
        let n = row.cnt;
        // indices: 0=total,1=completed,2=cancelled,3=started,4=unstarted,5=backlog
        entry[0] += n;
        match row.group_name.as_str() {
            "done"      => entry[1] += n,
            "cancelled" => entry[2] += n,
            "started"   => entry[3] += n,
            "unstarted" => entry[4] += n,
            _           => entry[5] += n, // backlog + triage + unknown
        }
    }

    for c in &mut cycles {
        if let Some(cnt) = counts.get(&c.id) {
            c.total_issues     = cnt[0];
            c.completed_issues = cnt[1];
            c.cancelled_issues = cnt[2];
            c.started_issues   = cnt[3];
            c.unstarted_issues = cnt[4];
            c.backlog_issues   = cnt[5];
        }
    }

    Ok(cycles)
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
        (status = 200, description = "Cycles list"),
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

    // Batch-load user favorites for cycles in this project
    let fav_ids: std::collections::HashSet<Uuid> = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::ProjectId.eq(guard.project.id))
        .filter(user_favorites::Column::EntityType.eq("cycle"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    let result = rows.into_iter().map(|m| {
        let is_fav = fav_ids.contains(&m.id);
        let mut r = CycleResponse::from_model(m);
        r.is_favorite = is_fav;
        r
    }).collect();

    let result = enrich_cycle_counts(&state.db, result).await?;
    Ok(Json(result))
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
        (status = 201, description = "Cycle created"),
        (status = 400, description = "Validation error"),
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
        return Err(AppError::BadRequest("name is required".into()));
    }

    // Validate start_date < end_date if both are present
    if let (Some(start), Some(end)) = (body.start_date, body.end_date) {
        if start >= end {
            return Err(AppError::BadRequest(
                "start_date must be before end_date".into(),
            ));
        }
    }

    // Parity with Django `CycleWriteSerializer.validate`: when BOTH dates are
    // present, they are converted to UTC using the project's timezone
    // (start → 00:00:01 local, end → 23:59:00 local). If only one is
    // provided, Django DOES NOT apply conversion — we replicate that quirk.
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

    // Explicit created_at / updated_at: cycles::ActiveModelBehavior is
    // empty and the column is NOT NULL without DEFAULT. Same pattern as
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
        (status = 200, description = "Cycle details"),
        (status = 404, description = "Not found"),
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

    let is_favorite = user_favorites::Entity::find()
        .filter(user_favorites::Column::UserId.eq(guard.user.id))
        .filter(user_favorites::Column::EntityIdentifier.eq(pk))
        .filter(user_favorites::Column::EntityType.eq("cycle"))
        .filter(user_favorites::Column::DeletedAt.is_null())
        .count(&state.db)
        .await
        .map_err(AppError::Database)? > 0;

    let mut resp = CycleResponse::from_model(cycle);
    resp.is_favorite = is_favorite;
    let mut enriched = enrich_cycle_counts(&state.db, vec![resp]).await?;
    Ok(Json(enriched.remove(0)))
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
        (status = 200, description = "Cycle updated"),
        (status = 404, description = "Not found"),
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

    // Parity with Django `CycleWriteSerializer.validate`: UTC tz-aware conversion
    // only applies when BOTH dates are in the PATCH payload. If only one is
    // provided, it's persisted as-is — same quirk as Django, where
    // `validate()` operates on partial `data` and not on the merged instance.
    // This allows a PATCH touching only `name`/`description` not to recalculate
    // existing dates.
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
    let resp = CycleResponse::from_model(updated);
    let mut enriched = enrich_cycle_counts(&state.db, vec![resp]).await?;
    Ok(Json(enriched.remove(0)))
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
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
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
        (status = 200, description = "Cycle issues"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_cycle_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<Vec<CycleIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Verify cycle belongs to project
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
        (status = 200, description = "Issues added to cycle"),
        (status = 400, description = "Validation error"),
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
        // Verify issue exists and belongs to project
        let issue_exists = issues::Entity::find_by_id(issue_id)
            .active()
            .filter(issues::Column::ProjectId.eq(project_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if issue_exists.is_none() {
            continue;
        }

        // Idempotent: if already exists (even soft-deleted), resurrect or ignore
        let existing = cycle_issues::Entity::find()
            .filter(cycle_issues::Column::CycleId.eq(cycle_id))
            .filter(cycle_issues::Column::IssueId.eq(issue_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

        let ci = match existing {
            Some(ci) if ci.deleted_at.is_none() => ci, // already exists active
            Some(ci) => {
                // Resurrect soft-deleted
                let mut am: cycle_issues::ActiveModel = ci.into();
                am.deleted_at = Set(None);
                am.updated_by_id = Set(Some(user_id));
                // Django: TimeAuditModel auto_now=True.
                am.updated_at = Set(now);
                am.update(&state.db).await.map_err(AppError::Database)?
            }
            None => {
                // Explicit created_at/updated_at (cycle_issues NOT NULL without
                // DEFAULT; ActiveModelBehavior empty).
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
        (status = 204, description = "Issue removed from cycle"),
        (status = 404, description = "Not found"),
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
// Mirror of `CycleAnalyticsEndpoint` in
// apps/api/plane/app/views/cycle/base.py:786, including `burndown_plot` helper
// in apps/api/plane/utils/analytics_plot.py.

#[derive(Debug, Deserialize)]
pub struct CycleAnalyticsQuery {
    /// "issues" (default) or "points".
    #[serde(rename = "type")]
    pub analytic_type: Option<String>,
}

/// GET /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics/
///
/// Cycle analytics: issue distribution by assignee and by label, plus
/// cumulative burndown chart. Exact mirror of `CycleAnalyticsEndpoint.get`.
///
/// Behavior:
/// - If cycle lacks `start_date` or `end_date` → 400.
/// - If cycle has non-empty `progress_snapshot` → early return with snapshot
///   data (cycle is closed and issues were transferred).
/// - If `type=points` and project does not have "points" type estimate →
///   empty distributions, empty chart (Django parity).
/// - If `type=issues` → issue counts by assignee/label + burndown by issues.
/// - If `type=points` + estimate points → sum of estimate_point.value by
///   assignee/label + burndown by points.
///
/// Permissions: ADMIN / MEMBER / GUEST (Django parity).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics/",
    tag = "Cycles",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cycle_id" = Uuid, Path, description = "Cycle ID"),
        ("type" = Option<String>, Query, description = "'issues' (default) or 'points'"),
    ),
    responses(
        (status = 200, description = "Cycle analytics"),
        (status = 400, description = "Cycle without dates"),
        (status = 403, description = "Unauthorized"),
        (status = 404, description = "Cycle not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn cycle_analytics(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
    Query(params): Query<CycleAnalyticsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Permissions: GUEST+ at project level or ADMIN at workspace level (Django
    // allows GUEST). require_role besides guard = defense in depth.
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let ws_id = guard.workspace.id;
    let project_id = guard.project.id;
    let db = &state.db;
    let analytic_type = params.analytic_type.as_deref().unwrap_or("issues");

    // ── 1. Cycle fetch (with workspace+project filters as extra defense) ──
    let cycle = cycles::Entity::find_by_id(cycle_id)
        .filter(cycles::Column::WorkspaceId.eq(ws_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // ── 2. Date validation (Django parity: 400 if any is missing) ─────────
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

    // ── 3. Progress snapshot: if exists and non-empty object, early return ─
    //
    // Django: `if cycle.progress_snapshot:` — truthy check on a dict.
    // In Postgres column is NOT NULL with default `{}`, so it can
    // arrive as an empty object (falsy in Python).
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

    // ── 4. Does the project use estimate with type="points"? ───────────────────────
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

    // ── 5. Reusable SQL fragments for distributions ───────────────
    //
    // scope_join ensures issues considered belong to the correct
    // cycle, workspace, and project. Reused in all
    // distribution + burndown queries.
    let scope_join = format!(
        "JOIN cycle_issues ci ON ci.issue_id = i.id
           AND ci.cycle_id = '{cycle_id}'
           AND ci.workspace_id = '{ws_id}'
           AND ci.project_id = '{project_id}'
           AND ci.deleted_at IS NULL"
    );
    // Parity with `Issue.issue_objects` (manager): excludes draft and archived.
    // We don't exclude triage here as it requires an extra JOIN with states and
    // it's extremely unlikely that triage issues are in a cycle.
    let issue_filters = format!(
        "i.deleted_at IS NULL
         AND i.archived_at IS NULL
         AND i.is_draft = false
         AND i.workspace_id = '{ws_id}'
         AND i.project_id = '{project_id}'"
    );

    // Default empty (used when type=points without estimate_type_points).
    let mut assignee_distribution: Vec<serde_json::Value> = Vec::new();
    let mut label_distribution: Vec<serde_json::Value> = Vec::new();
    let mut completion_chart = serde_json::json!({});

    // ── 6. type=points with estimate_type=points ───────────────────────────────
    if analytic_type == "points" && estimate_type_points {
        // Assignee distribution: SUM(ep.value::float) by assignee.
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

        // Label distribution: SUM(ep.value::float) by label.
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
        // Assignee distribution: issue COUNT by assignee.
        // Django uses Count("assignee_id", filter=...). COUNT(col) ignores NULLs,
        // so bucket of issues without assignee has 0 counts (parity).
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

        // Label distribution: COUNT by label.
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
// Mirror of `burndown_plot` (apps/api/plane/utils/analytics_plot.py:97).
// Generates a `{date_str: cumulative_pending}` dict for each day in
// [start_date, end_date]. Future dates remain `null` (parity).

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
    // ── Cycle Total ──────────────────────────────────────────────────────
    //
    // Django parity: for issues, total = `cycle.total_issues`
    // (active non-draft issues in cycle). For points, total = sum of
    // estimate_point.value of all issues (with estimate) in cycle.
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

    // ── Completed by date ────────────────────────────────────────────────
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

    // Map: date → total completed that day.
    use std::collections::BTreeMap;
    let daily_completed: BTreeMap<chrono::NaiveDate, f64> = rows
        .iter()
        .filter_map(|r| {
            let day = r.try_get::<chrono::NaiveDate>("", "day").ok()?;
            let completed = r.try_get::<f64>("", "completed").unwrap_or(0.0);
            Some((day, completed))
        })
        .collect();

    // ── Build chart_data: iterate day by day and accumulate ──────────────────
    let today = chrono::Utc::now().date_naive();
    let mut chart_data = serde_json::Map::new();
    let mut current = start_date;
    while current <= end_date {
        let key = current.to_string(); // "YYYY-MM-DD"
        if current > today {
            // Future dates: null (Django parity).
            chart_data.insert(key, serde_json::Value::Null);
        } else {
            // Sum of everything completed up to (inclusive) `current`.
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
// Mirror of `CycleUserPropertiesEndpoint` in
// apps/api/plane/app/views/cycle/base.py:625-655.
//
// Key semantics (Django parity):
//   - GET performs `get_or_create` → NEVER returns 404 due to missing row.
//     If it doesn't exist, it's created with defaults and 200 returned.
//   - PATCH assumes existence (Django uses raw `.get(...)`, which would 500
//     if missing). To avoid that crash and be more helpful to frontend, we
//     also perform `get_or_create` before applying patch — doesn't
//     degrade any valid use case.
//   - Permissions: ADMIN / MEMBER / GUEST (same as Django).
//
// Note on model: `CycleUserProperties` (cycle.py:130-153) DOES NOT have
// `preferences` or `sort_order` fields like `ProjectUserProperty`.
// Thus the request/response DTO is simpler than project's.

// ─── Defaults — mirror of `plane/db/models/issue.py:47-88` ───────────────────

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

/// Mirror of `CycleUserPropertiesSerializer` (fields="__all__", read_only:
/// workspace/project/cycle/user). Includes all fields from
/// `CycleUserProperties` model in `apps/api/plane/db/models/cycle.py:130-153`.
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

/// Allowed body in PATCH. All fields optional — Django serializer
/// `partial=True` semantics. Read-only fields (workspace/project/cycle/user)
/// are ignored if provided in body.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCycleUserPropertiesRequest {
    pub filters: Option<serde_json::Value>,
    pub display_filters: Option<serde_json::Value>,
    pub display_properties: Option<serde_json::Value>,
    pub rich_filters: Option<serde_json::Value>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Look up or create `cycle_user_properties` row for `(cycle, user)`.
/// Mirror of `CycleUserProperties.objects.get_or_create(...)`.
///
/// Unique constraint in Django: `(cycle, user)` WHERE `deleted_at IS NULL`
/// (`cycle.py:144-149`). We filter by `.active()`. In case of concurrent
/// INSERT with unique index violation, error propagates as
/// `AppError::Database` and client retry will resolve — same as Django.
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

/// Ensures cycle exists and belongs to workspace/project of guard.
/// 404 if not exists or soft-deleted. Defense in depth — guard only validates
/// workspace + project, not `cycle_id` membership.
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
/// Parity with `CycleUserPropertiesEndpoint.get` (cycle/base.py:647-655).
/// `get_or_create` ensures we never return 404 due to missing
/// properties row — this resolves the frontend 404.
///
/// Permissions: ROLE_GUEST+ (ADMIN/MEMBER/GUEST, Django parity).
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

    // Validation: cycle must exist and belong to project. If not,
    // 404 — prevents creating an orphaned user-properties row.
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
/// Parity with `CycleUserPropertiesEndpoint.patch` (cycle/base.py:627-644),
/// with one improvement: Django assumes row exists (`.objects.get(...)`) and
/// would throw 500 if missing; here we perform `get_or_create` before patching,
/// which is strictly more robust.
///
/// Django returns 201 in PATCH (inherited non-idiomatic behavior).
/// We maintain 200 here as (a) it's an update, not a create, and (b) Plane
/// frontend doesn't depend on the exact code — it checks `>=200 <300`.
///
/// Permissions: ROLE_GUEST+ (Django parity).
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
// Mirror of `CycleProgressEndpoint` in
// apps/api/plane/app/views/cycle/base.py:658-783.
//
// Returns issue counts and estimate_points sums grouped by
// state.group (backlog/unstarted/started/cancelled/completed) + totals.
//
// Key semantics (Django parity):
//   - If cycle has non-empty `progress_snapshot` → issue counts
//     come from snapshot (closed cycle with issues transferred).
//     estimate_points sums are ALWAYS calculated live — snapshot
//     does not contain them (line 664 in base.py: `aggregate_estimates` is
//     computed before branching).
//   - Without snapshot → live counts on `issues` JOIN `cycle_issues`.
//   - Estimate sums: only sums points of issues whose estimate.type='points'.
//   - Permissions: ADMIN / MEMBER / GUEST.
//
// Optimization over Django: single aggregate query for each block (counts
// and estimates), instead of 6 and 6 separate queries as ORM does.

/// Internal structure to deserialize counts from aggregated SQL.
#[derive(Debug, Default)]
struct ProgressIssueCounts {
    backlog: i64,
    unstarted: i64,
    started: i64,
    cancelled: i64,
    completed: i64,
    total: i64,
}

/// Internal structure to deserialize estimate_points sums.
#[derive(Debug, Default)]
struct ProgressEstimatePoints {
    backlog: f64,
    unstarted: f64,
    started: f64,
    cancelled: f64,
    completed: f64,
    total: f64,
}

/// Sum of `CAST(estimate_points.value AS DOUBLE PRECISION)` grouped by
/// `state.group`, filtering cycle issues whose `estimate_point` belongs
/// to an `estimate` with `type='points'`.
///
/// Mirror of compound query in cycle/base.py:664-711. Django does this
/// with 6 `Sum(Case(When(...), default=0))` in a single `.aggregate(...)`,
/// which compiles to exactly this form in SQL.
///
/// All results are COALESCE to 0 — Django uses `default=Value(0)` in
/// each Sum and also `or 0` in most response fields.
async fn compute_estimate_points(
    db: &sea_orm::DatabaseConnection,
    ws_id: Uuid,
    project_id: Uuid,
    cycle_id: Uuid,
) -> Result<ProgressEstimatePoints, AppError> {
    // UUIDs are interpolated: they are type-safe (Uuid::Display only produces
    // hex+dashes), no SQL injection surface. Same pattern as
    // `cycle_analytics` earlier in this file.
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

/// Live issue counts by `state.group` for cycle. One query instead of 6.
/// Mirror of cycle/base.py:720-765 (which Django resolves with 6 separate queries).
/// Respects `issue_objects` manager (excludes triage, archived, draft, soft-deleted).
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

/// Extracts integer count from `progress_snapshot[key]`. Snapshot saves
/// values as JSON numbers; if key is missing or not a number, 0.
fn snapshot_i64(snapshot: &serde_json::Map<String, serde_json::Value>, key: &str) -> i64 {
    snapshot
        .get(key)
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
        .unwrap_or(0)
}

/// GET /api/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress/
///
/// Returns 12 metrics: issue counts by state + estimate_points sums by state,
/// plus totals. Exact mirror of `CycleProgressEndpoint.get`
/// (cycle/base.py:658-783).
///
/// Behavior:
/// - If cycle doesn't exist → 404 (Django parity: `{"error": "Cycle not found"}`).
/// - If `progress_snapshot` is non-empty object → issue counts
///   are read from snapshot (closed cycle, issues transferred).
/// - estimate_points sums are ALWAYS calculated live (Django does too:
///   `aggregate_estimates` is computed before snapshot branch).
///
/// Permissions: ADMIN / MEMBER / GUEST (Django parity).
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

    // ── 1. Validate cycle exists and belongs to project ────────────────
    let cycle =
        ensure_cycle_belongs_to_project(db, ws_id, project_id, cycle_id).await?;

    // ── 2. estimate_points sums — ALWAYS live, as in Django ────────────
    //
    // Django computes `aggregate_estimates` in cycle/base.py:664, BEFORE
    // snapshot branch. Snapshot does not contain these sums.
    let estimates = compute_estimate_points(db, ws_id, project_id, cycle_id).await?;

    // ── 3. Issue counts: from snapshot if exists and not empty ────────
    //
    // Django: `if cycle.progress_snapshot:` — truthy on dict. In Postgres
    // column is NOT NULL with default `{}` (falsy in Python) so we
    // must distinguish empty object from object with data.
    let counts = if let serde_json::Value::Object(snapshot) = &cycle.progress_snapshot {
        if !snapshot.is_empty() {
            // Read from snapshot — same keys used by Django when persisting it.
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

    // ── 4. Response — same keys as Django (cycle/base.py:768-781) ───────
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
// PENDING ENDPOINTS — implemented below
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
/// Verifies if a date range overlaps with any existing cycle in the project.
/// Parity with `CycleDateCheckEndpoint.post` (cycle/base.py).
/// Returns `{"status": true}` if there is overlap, `{"status": false}` otherwise.
///
/// Permissions: ADMIN / MEMBER.
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

    // Parse dates — accepts "YYYY-MM-DD" or ISO 8601
    let start: chrono::NaiveDate = body
        .start_date
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid start_date format".into()))?;
    let end: chrono::NaiveDate = body
        .end_date
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid end_date format".into()))?;

    // Convert to DateTimeWithTimeZone to compare with column
    let start_dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::NaiveDateTime::new(start, chrono::NaiveTime::MIN)
            .and_utc()
            .fixed_offset();
    let end_dt: chrono::DateTime<chrono::FixedOffset> =
        chrono::NaiveDateTime::new(end, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap())
            .and_utc()
            .fixed_offset();

    // Look for cycles overlapping given range
    // Overlap: start_cycle <= end_input AND end_cycle >= start_input
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

    // Idempotent: don't duplicate if already exists
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
        let now = chrono::Utc::now();
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
            // `created_at` and `updated_at` are NOT NULL without DEFAULT in
            // baseline SQL. Django fills them via `auto_now_add` / `auto_now`,
            // here we must set them explicitly or INSERT fails with
            // 23502 → AppError::Database → 500.
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
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

    // Validate target cycle exists and is not completed
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

    // Validate source cycle exists
    let old_cycle = ensure_cycle_belongs_to_project(db, ws_id, project_id, cycle_id).await?;

    // Compute issue counts by state for snapshot
    let counts = compute_issue_counts(db, ws_id, project_id, cycle_id).await?;

    // Save progress snapshot in source cycle
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

    // Transfer incomplete issues to new cycle
    // Only issues with state: backlog, unstarted, started
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

    // Only completed cycles (end_date in past) can be archived
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

    // Remove from favorites (Django parity)
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
/// Lists archived cycles of the project.
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
/// Returns an archived cycle by its ID.
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
