// src/routes/workspace_view_issues.rs
//! Issue endpoint at workspace level (global view / spreadsheet).
//!
//! Equivalent to `WorkspaceViewIssuesViewSet` in
//! `plane/app/views/view/base.py` → `plane/app/urls/views.py:52`.
//!
//! Implemented route:
//!   GET /api/workspaces/{slug}/issues/
//!
//! Query parameters:
//!   - cursor      : Django pagination (`{page_size}:{page}:{is_prev}`), default `100:0:0`
//!   - per_page    : ignored if provided in cursor; default 100
//!   - order_by    : sort field, default `-created_at`
//!   - sub_issue   : `false` = exclude sub-issues (parent_id IS NOT NULL), default shows all
//!
//! Permission logic (mirror Django `_get_project_permission_filters`):
//!   For guests (role = 5):
//!     - if project.guest_view_all_features = true  → sees all project issues
//!     - if project.guest_view_all_features = false → sees only their own issues (created_by)
//!   For member / admin (role > 5):
//!     → sees all issues of the projects where they are an active member

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    auth::extractors::WorkspaceMemberGuard,
    auth::permissions::ROLE_GUEST,
    entities::{issues, project_members, projects},
    error::AppError,
    routes::issue_pagination::{
        apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
        load_workspace_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
    },
    routes::issue_filters::{apply_issue_filters, merge_json_filters, FilteredQuery},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WorkspaceIssuesQuery {
    // ── Paging / ordering ─────────────────────────────────────────────────────
    pub cursor:        Option<String>,
    pub per_page:      Option<u64>,
    pub order_by:      Option<String>,

    // ── Simple Toggles ────────────────────────────────────────────────────────
    pub sub_issue:     Option<String>,
    /// Incremental filter — only issues updated after this timestamp.
    /// Mirror of `updated_at__gt` in base.py:256.
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,

    // ── Filters delegated to the `issue_filters` module (inlined due to
    //    limitations of serde_urlencoded with `flatten`). The parsing/application
    //    logic lives in a single place.
    pub state:             Option<String>,
    pub state_group:       Option<String>,
    pub priority:          Option<String>,
    pub created_by:        Option<String>,
    pub parent:            Option<String>,
    pub name:              Option<String>,
    pub start_date:        Option<String>,
    pub target_date:       Option<String>,
    pub labels:            Option<String>,
    pub assignees:         Option<String>,
    pub module:            Option<String>,
    pub cycle:             Option<String>,
    /// CSV of `user_ids` — mirror of `filter_subscribed_issues`
    /// (issue_filters.py:392-403). Shared with other issue listings.
    pub subscriber:        Option<String>,
    #[serde(rename = "type")]
    pub type_filter:       Option<String>,
    pub start_target_date: Option<String>,

    // ── Rich filters (Django-parity subset) ───────────────────────────────
    //
    // JSON blob that the frontend sends in spreadsheet layout and saved views.
    // Parsed with `issue_filters::merge_json_filters`; unknown keys → 400.
    pub filters:           Option<String>,
}

impl WorkspaceIssuesQuery {
    fn to_filter_params(&self) -> crate::routes::issue_filters::IssueFilterParams {
        crate::routes::issue_filters::IssueFilterParams {
            state:             self.state.clone(),
            state_group:       self.state_group.clone(),
            priority:          self.priority.clone(),
            created_by:        self.created_by.clone(),
            parent:            self.parent.clone(),
            name:              self.name.clone(),
            start_date:        self.start_date.clone(),
            target_date:       self.target_date.clone(),
            labels:            self.labels.clone(),
            assignees:         self.assignees.clone(),
            module:            self.module.clone(),
            cycle:             self.cycle.clone(),
            subscriber:        self.subscriber.clone(),
            type_filter:       self.type_filter.clone(),
            start_target_date: self.start_target_date.clone(),
        }
    }
}

// ── Response DTOs ─────────────────────────────────────────────────────────────

/// Reflects the fields of `ViewIssueListSerializer` in Django.
/// Field names are snake_case -> serde serializes them as is.
#[derive(Debug, Serialize)]
pub struct WorkspaceIssueItem {
    pub id:              Uuid,
    pub name:            String,
    pub state_id:        Option<Uuid>,
    pub sort_order:      f64,
    pub completed_at:    Option<chrono::DateTime<chrono::FixedOffset>>,
    pub estimate_point:  Option<Uuid>,
    pub priority:        String,
    pub start_date:      Option<chrono::NaiveDate>,
    pub target_date:     Option<chrono::NaiveDate>,
    pub sequence_id:     i32,
    pub project_id:      Uuid,
    pub parent_id:       Option<Uuid>,
    pub cycle_id:        Option<Uuid>,
    pub sub_issues_count: i64,
    pub created_at:      chrono::DateTime<chrono::FixedOffset>,
    pub updated_at:      chrono::DateTime<chrono::FixedOffset>,
    pub created_by:      Option<Uuid>,
    pub updated_by:      Option<Uuid>,
    pub attachment_count: i64,
    pub link_count:      i64,
    pub is_draft:        bool,
    pub archived_at:     Option<chrono::NaiveDate>,
    pub type_id:         Option<Uuid>,
    #[serde(rename = "state__group")]
    pub state_group:     Option<String>,
    pub assignee_ids:    Vec<Uuid>,
    pub label_ids:       Vec<Uuid>,
    pub module_ids:      Vec<Uuid>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/issues/
///
/// Lists issues from all workspace projects the user has access to.
/// Equivalent to `WorkspaceViewIssuesViewSet.list()` in Django.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/issues/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("cursor" = Option<String>, Query, description = "Django pagination cursor: {per_page}:{page}:{is_prev}"),
        ("per_page" = Option<u64>, Query, description = "Results per page (ignored if in cursor)"),
        ("order_by" = Option<String>, Query, description = "Sort field, e.g.: -created_at"),
        ("sub_issue" = Option<String>, Query, description = "false = exclude sub-issues"),
    ),
    responses(
        (status = 200, description = "Paged list of workspace issues"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "No access to workspace"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_workspace_view_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<WorkspaceIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let workspace_member_role = guard.member.role;

    // ── 1. Paging ─────────────────────────────────────────────────────────────
    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) =
        parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // ── 2. Base Query Filters ────────────────────────────────────────────────
    let exclude_sub_issues = params
        .sub_issue
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    // ── 3. Calculate projects the user has access to ─────────────────────────
    //
    // Mirror of `_get_project_permission_filters` in Django:
    //   - We fetch active project_members for the user in this workspace.
    //   - For guests (role = 5): if guest_view_all_features = false,
    //     the project goes to "restricted" (they only see their own issues).
    //   - For roles > 5: full access to the project.
    //
    // Optimization: if the user is a workspace admin (role >= 20),
    // we assume full access to all active workspace projects.

    // Get user project memberships in this workspace
    let memberships = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(workspace_id))
        .filter(project_members::Column::MemberId.eq(user_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if memberships.is_empty() {
        // The user does not belong to any project in the workspace.
        // Shape mirror of Django `OffsetPaginator.paginate()`
        // (plane/utils/paginator.py:715-730).
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let project_ids: Vec<Uuid> = memberships.iter().map(|m| m.project_id).collect();

    // Cargar datos de proyectos para verificar guest_view_all_features y archived_at
    let project_rows = projects::Entity::find()
        .select_only()
        .column(projects::Column::Id)
        .column(projects::Column::GuestViewAllFeatures)
        .column(projects::Column::ArchivedAt)
        .filter(projects::Column::Id.is_in(project_ids.clone()))
        .filter(projects::Column::DeletedAt.is_null())
        .into_tuple::<(Uuid, bool, Option<chrono::DateTime<chrono::FixedOffset>>)>()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Fast index project_id → (guest_view_all_features, is_archived)
    let project_meta: HashMap<Uuid, (bool, bool)> = project_rows
        .into_iter()
        .map(|(id, gvaf, archived)| (id, (gvaf, archived.is_some())))
        .collect();

    // Classify projects into: full_access vs own_issues_only
    // (exclude archived projects)
    let mut full_access_ids: HashSet<Uuid> = HashSet::new();
    let mut restricted_ids: HashSet<Uuid> = HashSet::new();

    for membership in &memberships {
        let pid = membership.project_id;
        let role = membership.role;

        let (guest_view_all_features, is_archived) = match project_meta.get(&pid) {
            Some(meta) => *meta,
            None => continue, // project not found or deleted
        };

        if is_archived {
            continue; // exclude archived projects
        }

        // Workspace admin can see everything regardless of project role
        if workspace_member_role >= 20 || role > ROLE_GUEST {
            full_access_ids.insert(pid);
        } else {
            // Guest (role = 5)
            if guest_view_all_features {
                full_access_ids.insert(pid);
            } else {
                restricted_ids.insert(pid);
            }
        }
    }

    // ── 4. Build Base Query ──────────────────────────────────────────────────
    //
    // Avoided Anti-pattern: we don't use OR without an index — we separate into two
    // distinct conditions and join them with `sea_orm::Condition::any()`.

    let full_ids: Vec<Uuid> = full_access_ids.into_iter().collect();
    let rest_ids: Vec<Uuid> = restricted_ids.into_iter().collect();

    // Permission condition: full_access OR (restricted AND created_by = user)
    let permission_condition = {
        use sea_orm::Condition;
        let mut cond = Condition::any();

        if !full_ids.is_empty() {
            cond = cond.add(issues::Column::ProjectId.is_in(full_ids.clone()));
        }

        if !rest_ids.is_empty() {
            cond = cond.add(
                Condition::all()
                    .add(issues::Column::ProjectId.is_in(rest_ids.clone()))
                    .add(issues::Column::CreatedById.eq(user_id)),
            );
        }

        cond
    };

    // If no valid condition exists, the user sees nothing.
    if full_ids.is_empty() && rest_ids.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    // Base query: active issues (not soft-deleted), not archived.
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::IsDraft.eq(false))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(permission_condition.clone());

    if exclude_sub_issues {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // Mirror of `IssueManager.exclude(state__group='triage')` in
    // db/models/issue.py:97. This endpoint previously did not apply this exclusion
    // — it is now in parity with Django. We use a state IDs pre-query
    // instead of JOIN to keep the query builder simple.
    let triage_state_ids = load_workspace_triage_state_ids(db, workspace_id).await?;
    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    // Incremental filter `updated_at__gt` (base.py:256). Useful for delta
    // sync of the frontend without re-downloading the entire list.
    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // ── 4.5. Shared Module Filters ───────────────────────────────────────────
    //
    // state, state_group, priority, created_by, parent, name, start_date,
    // target_date, labels, assignees, module, cycle, type, start_target_date.
    // See `routes::issue_filters` for detailed mapping.
    //
    // `merge_json_filters` merges the `?filters=<JSON>` blob (spreadsheet
    // layout, saved views) over flat params. Unknown keys → 400.
    let mut filter_params = params.to_filter_params();
    merge_json_filters(params.filters.as_deref(), &mut filter_params)?;
    let filtered = apply_issue_filters(db, base_query, &filter_params, workspace_id).await?;
    let base_query = match filtered {
        FilteredQuery::Active(q) => q,
        FilteredQuery::Empty => return Ok(Json(empty_paginated_response(page_size))),
    };

    // ── 5. Total count (for paging) ───────────────────────────────────────────
    let total_results = base_query.clone().count(db).await.map_err(AppError::Database)?;

    // ── 6. Ordering ──────────────────────────────────────────────────────────
    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    // ── 7. Offset Paging ──────────────────────────────────────────────────────
    let start_index = current_page * page_size;

    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── 8. Batch Enrichment ──────────────────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);

    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    // ── 9. Serialize Results ──────────────────────────────────────────────────
    let results: Vec<WorkspaceIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());

            WorkspaceIssueItem {
                id,
                name: m.name,
                state_id: m.state_id,
                sort_order: m.sort_order,
                completed_at: m.completed_at,
                estimate_point: m.estimate_point_id,
                priority: m.priority,
                start_date: m.start_date,
                target_date: m.target_date,
                sequence_id: m.sequence_id,
                project_id: m.project_id,
                parent_id: m.parent_id,
                cycle_id: enrich.cycles.remove(&id),
                sub_issues_count: enrich.sub_counts.get(&id).copied().unwrap_or(0),
                created_at: m.created_at,
                updated_at: m.updated_at,
                created_by: m.created_by_id,
                updated_by: m.updated_by_id,
                attachment_count: enrich.attachments.get(&id).copied().unwrap_or(0),
                link_count: enrich.links.get(&id).copied().unwrap_or(0),
                is_draft: m.is_draft,
                archived_at: m.archived_at,
                type_id: m.type_id,
                state_group,
                assignee_ids: enrich.assignees.remove(&id).unwrap_or_default(),
                label_ids: enrich.labels.remove(&id).unwrap_or_default(),
                module_ids: enrich.modules.remove(&id).unwrap_or_default(),
            }
        })
        .collect();

    // ── 10. Paged Response ───────────────────────────────────────────────────
    //
    // Exact mirror shape of `OffsetPaginator.paginate()` in
    // `plane/utils/paginator.py:715-730`. The frontend reads `total_count`
    // in `base-issues.store.ts:1290`, and `TIssuesResponse`
    // (packages/types/src/issues/issue.ts:126) declara
    // `grouped_by`, `count`, `extra_stats` como requeridos.
    Ok(Json(paginated_response(
        results,
        page_size,
        current_page,
        total_results,
    )))
}
