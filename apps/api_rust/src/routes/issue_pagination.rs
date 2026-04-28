// src/routes/issue_pagination.rs
//! Shared helpers for Django-style issue pagination.
//!
//! Extracted from `workspace_view_issues.rs` to reuse in
//! `issues.rs::list_issues` (and future endpoints replicating the
//! `OffsetPaginator.paginate()` shape in `plane/utils/paginator.py:715-730`).
//!
//! Includes:
//!   - `parse_cursor`               → parses `{page_size}:{page}:{is_prev}`.
//!   - `EnrichmentMaps` +
//!     `load_enrichment`            → batch relationship loading (N+1 avoided)
//!     for the 8 enriched fields that
//!     mirror Django's `issue_on_results`
//!     (grouper.py:93-141).
//!   - `apply_issue_order`          → maps Django `order_by` to SeaORM.
//!   - `empty_paginated_response`   → exact Django paginator shape for
//!     empty responses (avoids building manually in early-returns).
//!
//! # Anti-patterns avoided
//! - **N+1**: all relationships loaded with `is_in()` in a single query.
//! - **Duplication**: this module replaces ~250 lines duplicated between
//!   `workspace_view_issues.rs` and `issues.rs`.
//! - **SQL injection**: `order_by` is mapped via `match` against a whitelist;
//!   unrecognized values fall back to safe default (`-created_at`).

use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    entities::{
        cycle_issues, file_assets, issue_assignees, issue_labels, issue_links, issues,
        module_issues, states,
    },
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
};

// ── Constants ────────────────────────────────────────────────────────────────

/// Upper bound for `page_size` (parallel to Django's `max_limit` paginator).
pub const PAGINATOR_MAX_LIMIT: u64 = 1000;

/// Default page size when neither `cursor` nor `per_page` is sent.
pub const DEFAULT_PER_PAGE: u64 = 100;

/// `file_assets` entity_type identifier for issue attachments
/// (mirrors `FileAsset.EntityTypeContext.ISSUE_ATTACHMENT`).
pub const ENTITY_TYPE_ISSUE_ATTACHMENT: &str = "issue_attachment";

/// `state.group` name that Django excludes in `IssueManager.get_queryset`
/// (db/models/issue.py:97). Issues with triage state MUST NOT
/// appear in general listings — they are exposed via intake.
pub const STATE_GROUP_TRIAGE: &str = "triage";

// ── Cursor ────────────────────────────────────────────────────────────────────

/// Parses Django cursor: `{page_size}:{current_page}:{is_prev}`.
///
/// If cursor is missing or malformed, falls back to `fallback_per_page` and
/// page 0. `page_size` is clamped to `[1, PAGINATOR_MAX_LIMIT]` — prevents
/// DoS via giant pages.
pub fn parse_cursor(cursor: Option<&str>, fallback_per_page: u64) -> (u64, u64) {
    if let Some(c) = cursor {
        let parts: Vec<&str> = c.splitn(3, ':').collect();
        if parts.len() == 3 {
            let page_size = parts[0].parse::<u64>().unwrap_or(fallback_per_page);
            let current_page = parts[1].parse::<u64>().unwrap_or(0);
            return (page_size.clamp(1, PAGINATOR_MAX_LIMIT), current_page);
        }
    }
    (fallback_per_page.clamp(1, PAGINATOR_MAX_LIMIT), 0)
}

// ── Batch enrichment ──────────────────────────────────────────────────────────

/// Relationships loaded in batch for page issues.
///
/// Each map is `issue_id → value(s)`. Absent keys = default value
/// (`vec![]` for lists, `0` for counters, `None` for optionals).
pub struct EnrichmentMaps {
    pub assignees:    HashMap<Uuid, Vec<Uuid>>,
    pub labels:       HashMap<Uuid, Vec<Uuid>>,
    pub modules:      HashMap<Uuid, Vec<Uuid>>,
    pub cycles:       HashMap<Uuid, Uuid>,
    pub sub_counts:   HashMap<Uuid, i64>,
    pub attachments:  HashMap<Uuid, i64>,
    pub links:        HashMap<Uuid, i64>,
    pub state_groups: HashMap<Uuid, String>,
}

impl EnrichmentMaps {
    /// Empty map — used when page has no issues.
    fn empty() -> Self {
        Self {
            assignees:    HashMap::new(),
            labels:       HashMap::new(),
            modules:      HashMap::new(),
            cycles:       HashMap::new(),
            sub_counts:   HashMap::new(),
            attachments:  HashMap::new(),
            links:        HashMap::new(),
            state_groups: HashMap::new(),
        }
    }
}

/// Loads all relationships needed to serialize an issue page.
///
/// Mirrors Django annotations in
/// `apps/api/plane/app/views/issue/base.py:213-247` +
/// `apps/api/plane/utils/grouper.py:70-81`.
///
/// # Batching
/// One query per relationship type → O(1) roundtrip cost,
/// independent of issue count. Queries are `SELECT ... WHERE issue_id IN (...)`.
pub async fn load_enrichment(
    db: &DatabaseConnection,
    issue_ids: &[Uuid],
    state_ids: &[Uuid],
) -> Result<EnrichmentMaps, AppError> {
    if issue_ids.is_empty() {
        return Ok(EnrichmentMaps::empty());
    }

    // Assignees
    let raw_assignees = issue_assignees::Entity::find()
        .active()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut assignees: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for a in raw_assignees {
        assignees.entry(a.issue_id).or_default().push(a.assignee_id);
    }

    // Labels
    let raw_labels = issue_labels::Entity::find()
        .active()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut labels: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for l in raw_labels {
        labels.entry(l.issue_id).or_default().push(l.label_id);
    }

    // Modules
    let raw_modules = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut modules: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for m in raw_modules {
        modules.entry(m.issue_id).or_default().push(m.module_id);
    }

    // Cycle IDs — first active cycle per issue only.
    // Mirror of Django Subquery: `CycleIssue.objects.filter(...).values("cycle_id")[:1]`
    // `entry(...).or_insert(...)` preserves first value seen.
    let raw_cycles = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut cycles: HashMap<Uuid, Uuid> = HashMap::new();
    for ci in raw_cycles {
        cycles.entry(ci.issue_id).or_insert(ci.cycle_id);
    }

    // Sub-issues count — grouped aggregation to avoid N+1.
    let raw_sub: Vec<(Uuid, i64)> = issues::Entity::find()
        .select_only()
        .column(issues::Column::ParentId)
        .column_as(
            sea_orm::sea_query::Expr::col(issues::Column::Id).count(),
            "cnt",
        )
        .filter(issues::Column::ParentId.is_in(issue_ids.to_vec()))
        .filter(issues::Column::DeletedAt.is_null())
        .group_by(issues::Column::ParentId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut sub_counts: HashMap<Uuid, i64> = HashMap::new();
    for (parent_id, cnt) in raw_sub {
        sub_counts.insert(parent_id, cnt);
    }

    // Attachment counts — grouped by issue_id (Option<Uuid> in model,
    // because `file_assets` also stores attachments for other entities
    // like pages, comments, etc.). `map(Some).collect()` wraps UUIDs
    // in `Option<Uuid>` to match column type.
    let raw_attachments: Vec<(Option<Uuid>, i64)> = file_assets::Entity::find()
        .select_only()
        .column(file_assets::Column::IssueId)
        .column_as(
            sea_orm::sea_query::Expr::col(file_assets::Column::Id).count(),
            "cnt",
        )
        .filter(
            file_assets::Column::IssueId
                .is_in(issue_ids.iter().cloned().map(Some).collect::<Vec<_>>()),
        )
        .filter(file_assets::Column::EntityType.eq(ENTITY_TYPE_ISSUE_ATTACHMENT))
        .filter(file_assets::Column::DeletedAt.is_null())
        .group_by(file_assets::Column::IssueId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut attachments: HashMap<Uuid, i64> = HashMap::new();
    for (issue_id, cnt) in raw_attachments {
        if let Some(id) = issue_id {
            attachments.insert(id, cnt);
        }
    }

    // Link counts — grouped.
    let raw_links: Vec<(Uuid, i64)> = issue_links::Entity::find()
        .select_only()
        .column(issue_links::Column::IssueId)
        .column_as(
            sea_orm::sea_query::Expr::col(issue_links::Column::Id).count(),
            "cnt",
        )
        .filter(issue_links::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_links::Column::DeletedAt.is_null())
        .group_by(issue_links::Column::IssueId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut links: HashMap<Uuid, i64> = HashMap::new();
    for (issue_id, cnt) in raw_links {
        links.insert(issue_id, cnt);
    }

    // State groups — only for states referenced by issues.
    let state_groups: HashMap<Uuid, String> = if state_ids.is_empty() {
        HashMap::new()
    } else {
        let rows = states::Entity::find()
            .select_only()
            .column(states::Column::Id)
            .column(states::Column::Group)
            .filter(states::Column::Id.is_in(state_ids.to_vec()))
            .into_tuple::<(Uuid, String)>()
            .all(db)
            .await
            .map_err(AppError::Database)?;
        rows.into_iter().collect()
    };

    Ok(EnrichmentMaps {
        assignees,
        labels,
        modules,
        cycles,
        sub_counts,
        attachments,
        links,
        state_groups,
    })
}

// ── Triage exclusion ──────────────────────────────────────────────────────────

/// Returns `states` IDs where `group = 'triage'` in a given workspace or
/// project. Mirror of `.exclude(state__group=StateGroup.TRIAGE.value)`
/// in `IssueManager.get_queryset` (db/models/issue.py:97).
///
/// We prefer pre-querying these IDs and filtering `issue.state_id NOT IN (...)`
/// instead of JOINing with `states` in every query → keeps SeaORM query
/// builders simple and typed.
pub async fn load_triage_state_ids(
    db: &DatabaseConnection,
    project_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::Group.eq(STATE_GROUP_TRIAGE))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}

/// Workspace-scoped version of `load_triage_state_ids` — loads triage state
/// IDs for ALL projects in workspace.
///
/// Used in `list_workspace_view_issues` to apply same exclusion
/// as Django at manager level, without requiring JOIN with `states` in
/// each listing query.
pub async fn load_workspace_triage_state_ids(
    db: &DatabaseConnection,
    workspace_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::WorkspaceId.eq(workspace_id))
        .filter(states::Column::Group.eq(STATE_GROUP_TRIAGE))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}

// ── Sorting ──────────────────────────────────────────────────────────────

/// Applies Django `order_by` to SeaORM SelectModel.
/// `-` prefix = descending. Strict whitelist — unrecognized values
/// fall back to safe default `-created_at` (no SQL injection risk).
pub fn apply_issue_order(
    query: sea_orm::Select<issues::Entity>,
    order_by: &str,
) -> sea_orm::Select<issues::Entity> {
    use sea_orm::Order::{Asc, Desc};

    let (col, dir): (issues::Column, _) = match order_by {
        "-created_at"   => (issues::Column::CreatedAt,   Desc),
        "created_at"    => (issues::Column::CreatedAt,   Asc),
        "-updated_at"   => (issues::Column::UpdatedAt,   Desc),
        "updated_at"    => (issues::Column::UpdatedAt,   Asc),
        "-priority"     => (issues::Column::Priority,    Desc),
        "priority"      => (issues::Column::Priority,    Asc),
        "-sort_order"   => (issues::Column::SortOrder,   Desc),
        "sort_order"    => (issues::Column::SortOrder,   Asc),
        "-sequence_id"  => (issues::Column::SequenceId,  Desc),
        "sequence_id"   => (issues::Column::SequenceId,  Asc),
        "-target_date"  => (issues::Column::TargetDate,  Desc),
        "target_date"   => (issues::Column::TargetDate,  Asc),
        "-start_date"   => (issues::Column::StartDate,   Desc),
        "start_date"    => (issues::Column::StartDate,   Asc),
        "-completed_at" => (issues::Column::CompletedAt, Desc),
        "completed_at"  => (issues::Column::CompletedAt, Asc),
        // Safe default
        _ => (issues::Column::CreatedAt, Desc),
    };

    query.order_by(col, dir)
}

// ── Paginated response ────────────────────────────────────────────────────────

/// Builds exact shape of empty `OffsetPaginator.paginate()`.
/// (`plane/utils/paginator.py:715-730`). Used in early-returns when
/// no valid results are found for user.
pub fn empty_paginated_response(page_size: u64) -> serde_json::Value {
    json!({
        "grouped_by":        null,
        "sub_grouped_by":    null,
        "total_count":       0,
        "next_cursor":       format!("{page_size}:1:0"),
        "prev_cursor":       format!("{page_size}:-1:1"),
        "next_page_results": false,
        "prev_page_results": false,
        "count":             0,
        "total_pages":       0,
        "total_results":     0,
        "extra_stats":       null,
        "results":           [],
    })
}

/// Builds paginated shape with results. `results` must be serializable.
///
/// Calculates `next_cursor` / `prev_cursor` and `*_page_results` flags
/// from `current_page` and `total_results` — same logic as Django.
pub fn paginated_response<T: serde::Serialize>(
    results: Vec<T>,
    page_size: u64,
    current_page: u64,
    total_results: u64,
) -> serde_json::Value {
    let total_pages = if total_results == 0 {
        0
    } else {
        total_results.div_ceil(page_size)
    };
    let start_index = current_page * page_size;
    let end_index = (start_index + page_size).min(total_results);
    let has_next = end_index < total_results;
    let has_prev = current_page > 0;

    // In Django, next/prev cursor are ALWAYS strings — page availability
    // is communicated via `*_page_results`.
    let prev_cursor = if current_page == 0 {
        format!("{page_size}:-1:1")
    } else {
        format!("{page_size}:{}:1", current_page - 1)
    };
    let next_cursor = format!("{page_size}:{}:0", current_page + 1);
    let page_count = results.len() as u64;

    json!({
        "grouped_by":        null,
        "sub_grouped_by":    null,
        "total_count":       total_results,
        "next_cursor":       next_cursor,
        "prev_cursor":       prev_cursor,
        "next_page_results": has_next,
        "prev_page_results": has_prev,
        "count":             page_count,
        "total_pages":       total_pages,
        "total_results":     total_results,
        "extra_stats":       null,
        "results":           results,
    })
}

/// Extracts unique `state_ids` (discarding `None`) from a list of issues.
/// Small helper — avoids repeating `filter_map + HashSet` pattern in each handler.
pub fn collect_state_ids(models: &[issues::Model]) -> Vec<Uuid> {
    models
        .iter()
        .filter_map(|i| i.state_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}
