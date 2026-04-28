// src/routes/exporter.rs
//! Issue export endpoints — parity with
//! `apps/api/plane/app/views/exporter/base.py::ExportIssuesEndpoint`.
//!
//! - `GET  /api/workspaces/{slug}/export-issues/` — paginated list of
//!   `ExporterHistory` filtered by `type="issue_exports"` (requires
//!   `per_page` + `cursor`).
//! - `POST /api/workspaces/{slug}/export-issues/` — enqueues an export
//!   job and returns the token for polling.
//! - `GET  /api/workspaces/{slug}/export-issues/{token}/` — queries the
//!   job status.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{extractors::WorkspaceMemberGuard, permissions::ROLE_MEMBER},
    entities::{exporters, project_members, projects, users},
    error::AppError,
    routes::workspaces::{user_to_lite, UserLiteDto},
    utils::{pagination, soft_delete::SoftDeleteExt},
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ExportIssuesRequest {
    /// IDs of projects to export. Django parity: the body field is
    /// `project` (not `project_ids`) — see
    /// apps/api/plane/app/views/exporter/base.py:29. If absent or empty,
    /// all projects in the workspace where the user is an active member
    /// and the project is not archived are used.
    pub project: Option<Vec<Uuid>>,
    /// Export format: "csv", "xlsx" or "json".
    pub provider: Option<String>,
    /// Accepted for parity with Django (apps/api/plane/app/views/exporter/
    /// base.py:28) although not used at this point in the flow — consumed by the
    /// worker via the persisted row.
    #[serde(default)]
    pub multiple: Option<bool>,
}

/// Django response shape (apps/api/plane/app/views/exporter/base.py:57-60):
/// `200 {"message": "Once the export is ready you will be able to download it"}`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExportIssuesEnqueuedResponse {
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExportIssuesResponse {
    pub token: String,
    pub status: String,
    pub url: Option<String>,
}

// ── GET /export-issues/ ──────────────────────────────────────────────────────

/// Query params for `GET /workspaces/{slug}/export-issues/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListExportIssuesQuery {
    /// Django cursor: `"{per_page}:{offset}:{is_prev}"` (e.g., `"10:0:0"`).
    pub cursor: Option<String>,
    /// per_page override. Django takes it before the cursor value.
    pub per_page: Option<u64>,
    /// Order field, Django format: `"-created_at"` (default) or `"created_at"`.
    /// Only `created_at` is supported for parity with actual frontend usage.
    pub order_by: Option<String>,
}

/// Mirror of `ExporterHistorySerializer`
/// (apps/api/plane/app/serializers/exporter.py:11-30).
///
/// Emitted fields: id, created_at, updated_at, project, provider, status, url,
/// initiated_by, initiated_by_detail (UserLiteSerializer no-admin), token,
/// created_by, updated_by.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExporterHistoryResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project: Option<Vec<Uuid>>,
    pub provider: String,
    pub status: String,
    pub url: Option<String>,
    pub initiated_by: Uuid,
    pub initiated_by_detail: Option<UserLiteDto>,
    pub token: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// GET /api/workspaces/{slug}/export-issues/ — paginated list of exports.
///
/// Parity with `ExportIssuesEndpoint.get`
/// (apps/api/plane/app/views/exporter/base.py:67-84):
///  - `@allow_permission([ADMIN, MEMBER], level="WORKSPACE")` → role >= MEMBER.
///  - Filter by `workspace.slug == slug` and `type == "issue_exports"`.
///  - Requires `per_page` AND `cursor` in query — if either is missing, 400.
///  - `order_by` default = `"-created_at"`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/export-issues/",
    tag = "Exporter",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("cursor" = Option<String>, Query, description = "Django cursor: per_page:offset:is_prev"),
        ("per_page" = Option<u64>, Query, description = "Override per_page"),
        ("order_by" = Option<String>, Query, description = "Order, default '-created_at'"),
    ),
    responses(
        (status = 200, description = "Paginated list of ExporterHistory"),
        (status = 400, description = "Missing per_page or cursor"),
        (status = 403, description = "Insufficient role (GUEST not allowed)"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_export_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(q): Query<ListExportIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Django parity: `@allow_permission([ADMIN, MEMBER], level="WORKSPACE")`.
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    // Django parity (apps/api/plane/app/views/exporter/base.py:73-84): this
    // endpoint requires **both** `per_page` AND `cursor` to be present. Without either
    // it responds with 400 `{"error": "per_page and cursor are required"}`. No
    // fallback to defaults — we replicate the error shape.
    if q.per_page.is_none() || q.cursor.as_deref().map(str::trim).is_none_or(str::is_empty) {
        return Err(AppError::BadRequest(
            "per_page and cursor are required".into(),
        ));
    }

    const DEFAULT_PER_PAGE: u64 = 10;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // Django parity: `order_by=request.GET.get("order_by", "-created_at")`.
    // We only support created_at (asc/desc) because no other column is used
    // in the frontend callsites. Any other value falls back to the default to
    // avoid expanding the SQL injection surface via dynamic column.
    let order_desc = match q.order_by.as_deref().unwrap_or("-created_at") {
        "created_at" => false,
        _ => true, // "-created_at" o cualquier otro → desc
    };

    let base = exporters::Entity::find()
        .filter(exporters::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(exporters::Column::Type.eq("issue_exports"))
        .filter(exporters::Column::DeletedAt.is_null());

    let total_count = base
        .clone()
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    let ordered = if order_desc {
        base.order_by_desc(exporters::Column::CreatedAt)
    } else {
        base.order_by_asc(exporters::Column::CreatedAt)
    };

    let rows = ordered
        .paginate(&state.db, limit)
        .fetch_page(cursor.offset)
        .await
        .map_err(AppError::Database)?;

    // Batch-load of `initiated_by` to avoid N+1 (parity with
    // `select_related("initiated_by")` in Django, apps/api/plane/app/views/
    // exporter/base.py:69).
    let initiator_ids: Vec<Uuid> = {
        let mut seen = std::collections::HashSet::new();
        rows.iter()
            .map(|r| r.initiated_by_id)
            .filter(|id| seen.insert(*id))
            .collect()
    };

    let users_by_id: HashMap<Uuid, users::Model> = if initiator_ids.is_empty() {
        HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(initiator_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    let results: Vec<ExporterHistoryResponse> = rows
        .into_iter()
        .map(|row| ExporterHistoryResponse {
            // `initiated_by_detail` uses `UserLiteSerializer` (no-admin), so
            // `is_admin=false` → no email or last_login_medium.
            initiated_by_detail: users_by_id.get(&row.initiated_by_id).map(|u| user_to_lite(u, false)),
            id: row.id,
            created_at: row.created_at.into(),
            updated_at: row.updated_at.into(),
            project: row.project,
            provider: row.provider,
            status: row.status,
            url: row.url,
            initiated_by: row.initiated_by_id,
            token: row.token,
            created_by: row.created_by_id,
            updated_by: row.updated_by_id,
        })
        .collect();

    let body = pagination::build_response(results, total_count, limit, cursor.offset);
    Ok((StatusCode::OK, Json(body)))
}

// ── POST /export-issues/ ──────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/export-issues/",
    tag = "Exporter",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Export enqueued"),
        (status = 400, description = "Invalid provider"),
        (status = 403, description = "Insufficient role (GUEST not allowed)"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn export_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<ExportIssuesRequest>,
) -> Result<(StatusCode, Json<ExportIssuesEnqueuedResponse>), AppError> {
    // Django parity: `@allow_permission([ADMIN, MEMBER], level="WORKSPACE")`
    // (apps/api/plane/app/views/exporter/base.py:22). GUEST (role=5) cannot
    // enqueue exports. `WorkspaceMemberGuard` only validates membership, so
    // we must force the role floor here.
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    // Django parity: only csv/xlsx/json are valid providers
    // (apps/api/plane/app/views/exporter/base.py:31,62-65). Any other
    // value → 400 with the same error shape.
    let provider = body.provider.as_deref().unwrap_or("csv").to_owned();
    if !matches!(provider.as_str(), "csv" | "xlsx" | "json") {
        return Err(AppError::BadRequest(format!(
            "Provider '{provider}' not found."
        )));
    }

    // Django parity (apps/api/plane/app/views/exporter/base.py:29,32-39):
    //   project_ids = request.data.get("project", [])
    //   if not project_ids:
    //       project_ids = Project.objects.filter(
    //           workspace__slug=slug,
    //           project_projectmember__member=request.user,
    //           project_projectmember__is_active=True,
    //           archived_at__isnull=True,
    //       ).values_list("id", flat=True)
    //
    // Frontend listing (column.tsx) does `project.length`, and the worker
    // fails with "No projects in the exporter" if `project` is NULL, so
    // this fallback is **required** — never persist NULL in `project`.
    let project_ids: Vec<Uuid> = match body.project.as_ref() {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => projects::Entity::find()
            .active()
            .inner_join(project_members::Entity)
            .filter(projects::Column::WorkspaceId.eq(guard.workspace.id))
            .filter(projects::Column::ArchivedAt.is_null())
            .filter(project_members::Column::MemberId.eq(guard.user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .filter(project_members::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| p.id)
            .collect(),
    };

    // Generate unique token for this job
    let token = format!("{}", Uuid::new_v4().as_simple());

    // Django parity: `TimeAuditModel` (apps/api/plane/db/mixins.py:19-20)
    // fills `created_at`/`updated_at` via `auto_now_add`/`auto_now`. In the
    // SeaORM entity gen (src/entities/exporters.rs:8-10) both columns are
    // `NOT NULL` without `ActiveModelBehavior::before_save`, so the call site
    // must set them explicitly — omitting them produces a 500:
    //   `null value in column "created_at" of relation "exporters" violates
    //    not-null constraint`.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let _exporter = exporters::ActiveModel {
        id: Set(Uuid::new_v4()),
        created_at: Set(now),
        updated_at: Set(now),
        token: Set(token.clone()),
        provider: Set(provider),
        status: Set("queued".to_owned()),
        reason: Set(String::new()),
        key: Set(String::new()),
        url: Set(None),
        // Never NULL: if the user didn't send `project` or sent `[]`, we
        // complete it with the fallback above. Django persists the exact
        // same resolved array (line 43 of the view).
        project: Set(Some(project_ids)),
        initiated_by_id: Set(guard.user.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        workspace_id: Set(guard.workspace.id),
        // Django parity: `type="issue_exports"` is the model default
        // (apps/api/plane/db/models/exporter.py:26-33) and is the filter used
        // by the listing GET. If we write `"issues"`, records
        // created by Rust remain invisible when listing.
        r#type: Set("issue_exports".to_owned()),
        name: Set(None),
        filters: Set(None),
        rich_filters: Set(None),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Enqueue the export job in apalis
    use crate::jobs::export::ExportIssuesJob;
    use apalis::prelude::Storage;
    use apalis_sql::postgres::PostgresStorage;

    // Reuse shared AppState PgPool instead of opening a new
    // connection per request (antipattern: paying for TCP+TLS+auth on each export
    // and discarding the pool upon exiting the function).
    let mut storage: PostgresStorage<ExportIssuesJob> =
        PostgresStorage::new(state.pg_pool.clone());
    storage
        .push(ExportIssuesJob {
            exporter_token: token.clone(),
            // Django parity (apps/api/plane/app/views/exporter/base.py:28,54):
            // default = false (export single consolidated file). The worker
            // does not yet respect this flag — see TODO in jobs/export.rs.
            multiple: body.multiple.unwrap_or(false),
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Job queue error: {e}")))?;

    // Exact shape parity with Django (apps/api/plane/app/views/exporter/
    // base.py:57-60): 200 OK with `{"message": "..."}`. The frontend
    // (export-modal.tsx:82, export-form.tsx:110) doesn't read the response
    // body, it only refreshes the listing via SWR; returning token/status/url
    // here is unnecessary divergence.
    Ok((
        StatusCode::OK,
        Json(ExportIssuesEnqueuedResponse {
            message: "Once the export is ready you will be able to download it".to_owned(),
        }),
    ))
}

/// GET /api/workspaces/{slug}/export-issues/{token}/ — status polling
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/export-issues/{token}/",
    tag = "Exporter",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("token" = String, Path, description = "Export token"),
    ),
    responses(
        (status = 200, description = "Export status"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_export_status(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, token)): Path<(String, String)>,
) -> Result<Json<ExportIssuesResponse>, AppError> {
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(&token))
        .filter(exporters::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(exporters::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ExportIssuesResponse {
        token: exporter.token,
        status: exporter.status,
        url: exporter.url,
    }))
}
