// src/routes/user_profile_issues.rs
//! Endpoint `GET /api/workspaces/{slug}/user-issues/{user_id}/`.
//!
//! Mirror of `WorkspaceUserProfileIssuesEndpoint` in
//! `plane/app/views/workspace/user.py:98-249`.
//!
//! Lists issues associated with `user_id` (target) within the workspace,
//! filtered by projects to which the **requester** has access.
//!
//! # Target user scope (mirror Django)
//!
//! ```python
//! id__in=Issue.issue_objects.filter(
//!     Q(assignees__in=[user_id])
//!     | Q(created_by_id=user_id)
//!     | Q(issue_subscribers__subscriber_id=user_id),
//!     workspace__slug=slug,
//! ).values_list("id", flat=True)
//! ```
//!
//! That OR is ALWAYS applied, regardless of whether the frontend also
//! sends `?assignees=user_id`, `?created_by=user_id`, or `?subscriber=user_id`
//! for the Assigned / Created / Subscribed tabs. The absence of the OR
//! would allow reading all workspace issues if the frontend omitted the filter
//! — a silent privilege escalation compared to Django.
//!
//! # Permissions
//!
//! Mirror of `WorkspaceViewerPermission`: the requester must be an active
//! workspace member (any role). Issues are scoped to projects
//! where the requester is an active member + guest logic:
//! `guest_view_all_features = false` restricts to issues created by the
//! requester themselves. Identical to `list_workspace_view_issues`.
//!
//! # Avoided Anti-patterns
//!
//! - **N+1**: single pre-query of target `id__in`; batch enrichment
//!   (8 relations) via `load_enrichment`.
//! - **Cross-tenant leakage**: all subqueries filter by `workspace_id`,
//!   including target user scope subqueries.
//! - **Privilege escalation**: the target user OR-scope is not optional
//!   — it's applied even if the frontend sends filters; additionally, the
//!   standard `permission_condition` is applied (full_access vs restricted guest).
//! - **SQL injection**: UUIDs are parsed/validated before entering the query;
//!   SeaORM binds the parameters. No `format!`.
//! - **Paging DoS**: `parse_cursor` clamps `page_size` to `PAGINATOR_MAX_LIMIT`.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    auth::{any_auth::AnyAuth, permissions::ROLE_GUEST},
    entities::{issue_assignees, issue_subscribers, issues, project_members, projects},
    error::AppError,
    routes::{
        helpers::{require_workspace_member, workspace_by_slug},
        issue_filters::{apply_issue_filters, merge_json_filters, FilteredQuery, IssueFilterParams},
        issue_pagination::{
            apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
            load_workspace_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
        },
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Query params ──────────────────────────────────────────────────────────────
//
// `serde_urlencoded` does not support `#[serde(flatten)]`, which is why the
// `IssueFilterParams` fields are inlined in the query struct. Following the
// same pattern as `WorkspaceIssuesQuery` in `workspace_view_issues.rs`.

#[derive(Debug, Deserialize)]
pub struct UserProfileIssuesQuery {
    // Paging / ordering
    pub cursor:            Option<String>,
    pub per_page:          Option<u64>,
    pub order_by:          Option<String>,

    // Toggles
    pub sub_issue:         Option<String>,
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt:     Option<chrono::DateTime<chrono::FixedOffset>>,

    // Filters (delegated to issue_filters)
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
    pub subscriber:        Option<String>,
    #[serde(rename = "type")]
    pub type_filter:       Option<String>,
    pub start_target_date: Option<String>,

    // Rich filters (JSON blob for saved views / spreadsheet layout)
    pub filters:           Option<String>,
}

impl UserProfileIssuesQuery {
    fn to_filter_params(&self) -> IssueFilterParams {
        IssueFilterParams {
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

// ── DTO ───────────────────────────────────────────────────────────────────────
//
// Shape identical to `WorkspaceIssueItem` from `workspace_view_issues.rs`. The
// frontend consumes both with `TIssuesResponse` (packages/types), so fields
// must remain in parity.

#[derive(Debug, Serialize)]
pub struct UserProfileIssueItem {
    pub id:               Uuid,
    pub name:             String,
    pub state_id:         Option<Uuid>,
    pub sort_order:       f64,
    pub completed_at:     Option<chrono::DateTime<chrono::FixedOffset>>,
    pub estimate_point:   Option<Uuid>,
    pub priority:         String,
    pub start_date:       Option<chrono::NaiveDate>,
    pub target_date:      Option<chrono::NaiveDate>,
    pub sequence_id:      i32,
    pub project_id:       Uuid,
    pub parent_id:        Option<Uuid>,
    pub cycle_id:         Option<Uuid>,
    pub sub_issues_count: i64,
    pub created_at:       chrono::DateTime<chrono::FixedOffset>,
    pub updated_at:       chrono::DateTime<chrono::FixedOffset>,
    pub created_by:       Option<Uuid>,
    pub updated_by:       Option<Uuid>,
    pub attachment_count: i64,
    pub link_count:       i64,
    pub is_draft:         bool,
    pub archived_at:      Option<chrono::NaiveDate>,
    #[serde(rename = "state__group")]
    pub state_group:      Option<String>,
    pub assignee_ids:     Vec<Uuid>,
    pub label_ids:        Vec<Uuid>,
    pub module_ids:       Vec<Uuid>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/user-issues/{user_id}/`
///
/// Paged list of issues associated with `user_id` (target) within the
/// projects to which the **requester** has access in the workspace.
///
/// Mirror of `WorkspaceUserProfileIssuesEndpoint`
/// (`plane/app/views/workspace/user.py:98-249`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-issues/{user_id}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"     = String, Path,  description = "Workspace slug"),
        ("user_id"  = Uuid,   Path,  description = "Target user UUID"),
        ("cursor"   = Option<String>, Query, description = "Django Cursor: {per_page}:{page}:{is_prev}"),
        ("per_page" = Option<u64>,    Query, description = "Page size (ignored if in cursor)"),
        ("order_by" = Option<String>, Query, description = "Sort field, default `-created_at`"),
    ),
    responses(
        (status = 200, description = "Paged list of issues from the user profile"),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "Not an active workspace member"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn list_user_profile_issues(
    State(state): State<AppState>,
    AnyAuth(requester): AnyAuth,
    Path((slug, target_user_id)): Path<(String, Uuid)>,
    Query(params): Query<UserProfileIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;

    // ── 1. Auth + workspace permissions ──────────────────────────────────────
    let ws = workspace_by_slug(db, &slug).await?;
    let workspace_id = ws.id;
    let requester_id = requester.id;
    let workspace_member = require_workspace_member(db, workspace_id, requester_id).await?;
    let workspace_member_role = workspace_member.role;

    // ── 2. Paging ────────────────────────────────────────────────────────────
    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // ── 3. Projects accessible by the requester (mirror view_issues) ─────────
    //
    // Same logic as Django's `_get_project_permission_filters`:
    //   - Guest (role = 5) with `guest_view_all_features = false` → only sees
    //     their own issues (created_by_id = requester).
    //   - Guest with `guest_view_all_features = true` or roles > 5 → sees all
    //     issues in the project.
    let memberships = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(workspace_id))
        .filter(project_members::Column::MemberId.eq(requester_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if memberships.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let project_ids: Vec<Uuid> = memberships.iter().map(|m| m.project_id).collect();

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

    let project_meta: HashMap<Uuid, (bool, bool)> = project_rows
        .into_iter()
        .map(|(id, gvaf, archived)| (id, (gvaf, archived.is_some())))
        .collect();

    let mut full_access_ids: HashSet<Uuid> = HashSet::new();
    let mut restricted_ids: HashSet<Uuid> = HashSet::new();

    for membership in &memberships {
        let pid = membership.project_id;
        let role = membership.role;

        let (guest_view_all_features, is_archived) = match project_meta.get(&pid) {
            Some(meta) => *meta,
            None => continue,
        };
        if is_archived {
            continue;
        }

        if workspace_member_role >= 20 || role > ROLE_GUEST || guest_view_all_features {
            full_access_ids.insert(pid);
        } else {
            restricted_ids.insert(pid);
        }
    }

    let full_ids: Vec<Uuid> = full_access_ids.into_iter().collect();
    let rest_ids: Vec<Uuid> = restricted_ids.into_iter().collect();

    if full_ids.is_empty() && rest_ids.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    // Permission condition: full_access OR (restricted AND created_by = requester)
    let permission_condition = {
        let mut cond = Condition::any();
        if !full_ids.is_empty() {
            cond = cond.add(issues::Column::ProjectId.is_in(full_ids.clone()));
        }
        if !rest_ids.is_empty() {
            cond = cond.add(
                Condition::all()
                    .add(issues::Column::ProjectId.is_in(rest_ids.clone()))
                    .add(issues::Column::CreatedById.eq(requester_id)),
            );
        }
        cond
    };

    // ── 4. Target user OR scope (assignee ∪ created_by ∪ subscriber) ─────────
    //
    // EXACT mirror of Django's `id__in=...`. ALWAYS executed — it's the
    // invariant of this endpoint: it never returns issues not linked to the target.
    // Frontend query string filters narrow down this OR.
    //
    // Implementation: three batch pre-queries, then union in memory.
    // SQL `UNION` alternative would be marginally faster, but this
    // pattern is consistent with `issue_filters::load_issues_with_*` and avoids
    // an additional custom statement.
    let target_issue_ids = load_target_user_issue_ids(db, workspace_id, target_user_id).await?;
    if target_issue_ids.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    // ── 5. Base Query ────────────────────────────────────────────────────────
    let exclude_sub_issues = params
        .sub_issue
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::IsDraft.eq(false))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::Id.is_in(target_issue_ids))
        .filter(permission_condition);

    if exclude_sub_issues {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // Triage exclusion (mirror `IssueManager.get_queryset` in
    // db/models/issue.py:97).
    let triage_state_ids = load_workspace_triage_state_ids(db, workspace_id).await?;
    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // ── 6. Query string filters + JSON blob ──────────────────────────────────
    let mut filter_params = params.to_filter_params();
    merge_json_filters(params.filters.as_deref(), &mut filter_params)?;
    let filtered = apply_issue_filters(db, base_query, &filter_params, workspace_id).await?;
    let base_query = match filtered {
        FilteredQuery::Active(q) => q,
        FilteredQuery::Empty => return Ok(Json(empty_paginated_response(page_size))),
    };

    // ── 7. Total count + order + offset paging ───────────────────────────────
    let total_results = base_query
        .clone()
        .count(db)
        .await
        .map_err(AppError::Database)?;

    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    let start_index = current_page * page_size;

    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── 8. Batch enrichment (no N+1) ─────────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    // ── 9. Serialize ─────────────────────────────────────────────────────────
    let results: Vec<UserProfileIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());

            UserProfileIssueItem {
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
                state_group,
                assignee_ids: enrich.assignees.remove(&id).unwrap_or_default(),
                label_ids: enrich.labels.remove(&id).unwrap_or_default(),
                module_ids: enrich.modules.remove(&id).unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(paginated_response(
        results,
        page_size,
        current_page,
        total_results,
    )))
}

// ── Internal Helpers ──────────────────────────────────────────────────────────

/// Pre-query: issue IDs where `target_user_id` is assignee, created_by
/// or subscriber within the workspace. Mirror of Django subquery:
///
/// ```python
/// Issue.issue_objects.filter(
///     Q(assignees__in=[user_id])
///     | Q(created_by_id=user_id)
///     | Q(issue_subscribers__subscriber_id=user_id),
///     workspace__slug=slug,
/// ).values_list("id", flat=True)
/// ```
///
/// 3 batch queries are made and joined in memory (via `HashSet`). Simpler
/// and more testable than a custom SQL `UNION`, with negligible overhead for
/// expected volumes (issues per user typically < 10k).
async fn load_target_user_issue_ids(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    target_user_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let mut ids: HashSet<Uuid> = HashSet::new();

    // Assignee
    let assigned: Vec<Uuid> = issue_assignees::Entity::find()
        .select_only()
        .column(issue_assignees::Column::IssueId)
        .filter(issue_assignees::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_assignees::Column::AssigneeId.eq(target_user_id))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    ids.extend(assigned);

    // Created by
    let created: Vec<Uuid> = issues::Entity::find()
        .select_only()
        .column(issues::Column::Id)
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::CreatedById.eq(target_user_id))
        .filter(issues::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    ids.extend(created);

    // Subscriber
    let subscribed: Vec<Uuid> = issue_subscribers::Entity::find()
        .select_only()
        .column(issue_subscribers::Column::IssueId)
        .filter(issue_subscribers::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_subscribers::Column::SubscriberId.eq(target_user_id))
        .filter(issue_subscribers::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    ids.extend(subscribed);

    Ok(ids.into_iter().collect())
}
