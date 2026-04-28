// src/routes/issue_filters.rs
//! Issue filters shared between listing endpoints.
//!
//! Mirror of `apps/api/plane/utils/issue_filters.py`. Each frontend store
//! in Plane sends filters as query params; this module parses and
//! translates them to SeaORM conditions.
//!
//! # Coverage
//!
//! **Implemented** (high-traffic frontend filters):
//!   - state, state_group, priority, created_by, parent  (issue scalars)
//!   - name                                               (substring LIKE '%term%')
//!   - start_date, target_date                            (YYYY-MM-DD with ;after / ;before)
//!   - labels, assignees, module, cycle                   (m2m via issue_ids pre-query)
//!   - subscriber                                          (m2m via issue_subscribers)
//!   - type                                               (all/backlog/active → state_group)
//!   - start_target_date                                  (boolean toggle)
//!
//! **Not implemented** (marginal use or high complexity, postponed):
//!   - mentions, logged_by                                 (seldom used relationships)
//!   - estimate_point                                     (rare in default views)
//!   - relative date syntax (`2_weeks;after;fromnow`)
//!   - inbox_status, intake_status                        (intake specific)
//!
//! An unimplemented filter is silently ignored — behavioral parity
//! with Django for covered filters; doesn't break the request.
//!
//! # Strategy for m2m filters
//!
//! Pre-query `issue_id` via bridge table + `issues::Id IN (ids)` in the
//! listing query. In pure SQL `IN (subquery)` would be marginally better,
//! but pre-query is:
//!   - **Simpler**: uses only SeaORM APIs already tested in the codebase.
//!   - **More testable**: each step is an independent inspectable query.
//!   - **Equally secure**: no SQL injection risk (UUIDs validated
//!     before entering any query).
//!
//! The extra cost (one additional round-trip per m2m filter present) is
//! acceptable given that m2m filters are typically applied one at a time.
//!
//! # Anti-patterns avoided
//!
//! - **SQL injection**: `priority` and `state_group` validated against whitelist;
//!   UUIDs via `Uuid::parse_str` (invalid ones are silently discarded,
//!   mirroring `filter_valid_uuids` in issue_filters.py:16-25).
//! - **Soft-deleted linkage**: m2m filters add `deleted_at IS NULL`
//!   on bridge table (parity with issue_filters.py:158,173,331,346).
//! - **Cross-tenant leakage**: m2m subqueries filter by
//!   `workspace_id` when available.

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};
use sea_orm::sea_query::extension::postgres::PgExpr;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{cycle_issues, issue_assignees, issue_labels, issue_subscribers, issues, module_issues, states},
    error::AppError,
};

// ── Query params ──────────────────────────────────────────────────────────────

/// Filters accepted in query string. All are optional and combined with AND.
/// Comma-separated values are parsed inside each helper.
///
/// Note: NOT used as `#[serde(flatten)]` inside query structs because
/// axum's `Query<T>` uses `serde_urlencoded`, which doesn't support `flatten`.
/// Instead, each handler inlines the same fields in its own struct
/// and builds an `IssueFilterParams` via `FromRef`-like builder.
#[derive(Debug, Default, Deserialize)]
pub struct IssueFilterParams {
    pub state:             Option<String>,
    pub state_group:       Option<String>,
    pub priority:          Option<String>,
    pub created_by:        Option<String>,
    pub parent:             Option<String>,
    pub name:               Option<String>,
    pub start_date:         Option<String>,
    pub target_date:        Option<String>,
    pub labels:             Option<String>,
    pub assignees:          Option<String>,
    pub module:             Option<String>,
    pub cycle:              Option<String>,
    /// CSV of `user_ids` used as subscribers. Mirror of
    /// `filter_subscribed_issues` (issue_filters.py:392-403).
    /// `user-issues/{user_id}` endpoint uses this filter for the
    /// "Subscribed" tab in user profile.
    pub subscriber:         Option<String>,
    /// `all` | `backlog` | `active`. Mirror of
    /// `filter_issue_state_type` (issue_filters.py:296-305).
    #[serde(rename = "type")]
    pub type_filter:        Option<String>,
    pub start_target_date:  Option<String>,
}

/// "Empty result" marker: when a filter implies no possible matches
/// (e.g. `type=backlog` but workspace has no states in `backlog` group),
/// we return this enum instead of continuing query mutation.
/// Handler calling `apply_issue_filters` can then early-return with
/// `empty_paginated_response`.
//
// `Active` is the hot path (~100% of real requests) and `Empty` variant
// is a marker only returned in edge cases (filter that nulls query).
// Boxing `Select<issues::Entity>` to balance sizes adds one alloc per
// request in happy-path without real benefit; we prefer accepting size-skew.
#[allow(clippy::large_enum_variant)]
pub enum FilteredQuery {
    /// Query with applied filters — still `Select<issues::Entity>`.
    Active(Select<issues::Entity>),
    /// At least one filter guarantees 0 results — not worth executing.
    Empty,
}

// ── Parsing helpers ────────────────────────────────────────────────────────

/// Splits a comma-separated string, discarding empty ones and "null" literals.
fn split_csv(raw: &str) -> Vec<&str> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "null")
        .collect()
}

/// Mirror of `filter_valid_uuids` (issue_filters.py:16-25).
/// Invalid UUIDs are silently DISCARDED (Django aligned).
fn parse_uuids_csv(raw: &str) -> Vec<Uuid> {
    split_csv(raw)
        .into_iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect()
}

/// Django uses `"None"` literal for "no value" (→ `isnull=True`).
fn csv_contains_none(raw: &str) -> bool {
    raw.split(',').map(str::trim).any(|s| s == "None")
}

// ── Rich filters parsing (Django-parity subset) ──────────────────────

/// Parses `?filters=<JSON>` and merges recognized fields into `params`.
///
/// Frontend sends this blob when using layout=spreadsheet or a saved view,
/// e.g. `?filters={"state_group__in":"backlog"}`. Django processes it with
/// `ComplexFilterBackend` + `IssueFilterSet`; Rust API ports the subset of
/// keys 1:1 mappable to flat params this module already implements.
///
/// Recognized keys (mirroring `IssueFilterSet` in
/// `plane/utils/filters/filterset.py`):
///
/// | JSON key                   | `IssueFilterParams` field |
/// |----------------------------|--------------------------|
/// | `state_group` / `...__in`  | `state_group`            |
/// | `state_id`    / `...__in`  | `state`                  |
/// | `priority`    / `...__in`  | `priority`               |
/// | `label_id`    / `...__in`  | `labels`                 |
/// | `assignee_id` / `...__in`  | `assignees`              |
/// | `created_by_id`/ `...__in` | `created_by`             |
/// | `module_id`   / `...__in`  | `module`                 |
/// | `cycle_id`    / `...__in`  | `cycle`                  |
///
/// # Priority
///
/// If flat query-param is already set (`Some`), JSON value is ignored —
/// flat params take precedence (user explicitly set them in URL besides
/// `filters` blob).
///
/// # Security
///
/// **Unknown** keys return 400. Silently ignoring them would return
/// more results than expected (semantic data leak); better to fail
/// explicitly and track the gap.
///
/// # Safe no-ops
///
/// - `filters` absent, empty, `null`, `{}`, `[]` → no changes to `params`.
pub fn merge_json_filters(
    raw: Option<&str>,
    params: &mut IssueFilterParams,
) -> Result<(), AppError> {
    let Some(trimmed) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(());
    };

    let obj = match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(serde_json::Value::Null) => return Ok(()),
        Ok(serde_json::Value::Object(m)) if m.is_empty() => return Ok(()),
        Ok(serde_json::Value::Array(a)) if a.is_empty() => return Ok(()),
        Ok(serde_json::Value::Object(m)) => m,
        Ok(_) => {
            return Err(AppError::BadRequest(
                "?filters must be a JSON object (received: array, string, or number)".into(),
            ))
        }
        Err(_) => {
            return Err(AppError::BadRequest(
                "?filters contains malformed JSON".into(),
            ))
        }
    };

    for (key, val) in &obj {
        let csv = json_filter_value_to_csv(val).map_err(|_| {
            AppError::BadRequest(format!(
                "?filters: invalid value for key '{}' (expected string or string array)",
                key
            ))
        })?;

        // Only write if flat param is not already set (flat params take precedence).
        match key.as_str() {
            "state_group" | "state_group__in" => {
                params.state_group.get_or_insert(csv);
            }
            "state_id" | "state_id__in" => {
                params.state.get_or_insert(csv);
            }
            "priority" | "priority__in" => {
                params.priority.get_or_insert(csv);
            }
            "label_id" | "label_id__in" => {
                params.labels.get_or_insert(csv);
            }
            "assignee_id" | "assignee_id__in" => {
                params.assignees.get_or_insert(csv);
            }
            "created_by_id" | "created_by_id__in" => {
                params.created_by.get_or_insert(csv);
            }
            "module_id" | "module_id__in" => {
                params.module.get_or_insert(csv);
            }
            "cycle_id" | "cycle_id__in" => {
                params.cycle.get_or_insert(csv);
            }
            unknown => {
                return Err(AppError::BadRequest(format!(
                    "?filters: key '{}' is not supported by Rust API. \
                     Use equivalent flat query params (priority, state_group, \
                     labels, assignees, cycle, module, etc.) or point to Django backend. \
                     Tracking: rich-filters Django-parity follow-up.",
                    unknown
                )));
            }
        }
    }

    Ok(())
}

/// Converts a filter JSON value to a CSV string compatible with
/// `IssueFilterParams` (fields are comma-separated `Option<String>`).
///
/// Accepts:
/// - `"backlog"` → `"backlog"`
/// - `["high","medium"]` → `"high,medium"`
/// - `["uuid1","uuid2"]` → `"uuid1,uuid2"`
fn json_filter_value_to_csv(val: &serde_json::Value) -> Result<String, ()> {
    match val {
        serde_json::Value::String(s) => Ok(s.clone()),
        serde_json::Value::Array(arr) => {
            let parts: Result<Vec<String>, ()> = arr
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => Ok(s.clone()),
                    serde_json::Value::Number(n) => Ok(n.to_string()),
                    _ => Err(()),
                })
                .collect();
            Ok(parts?.join(","))
        }
        _ => Err(()),
    }
}

// ── Whitelists ────────────────────────────────────────────────────────────────

/// Valid priorities (mirror of CHOICES in db/models/issue.py:105-111).
const VALID_PRIORITIES: &[&str] = &["urgent", "high", "medium", "low", "none"];

/// Valid state groups (mirror of `StateGroup` in db/models/state.py).
const VALID_STATE_GROUPS: &[&str] = &[
    "backlog",
    "unstarted",
    "started",
    "completed",
    "cancelled",
    "triage",
];

// ── Main Application ──────────────────────────────────────────────────────

/// Applies all recognized filters to a `Select<issues::Entity>`.
///
/// Returns `FilteredQuery::Empty` if any filter results in 0 guaranteed matches
/// (e.g. `labels=<uuid>` but no issues have that label in entire workspace)
/// — handler can then early-return without executing the main query.
///
/// `workspace_id` is used to scope m2m subqueries (prevents accidental
/// matching with labels/modules/etc. from another workspace).
pub async fn apply_issue_filters(
    db: &sea_orm::DatabaseConnection,
    query: Select<issues::Entity>,
    params: &IssueFilterParams,
    workspace_id: Uuid,
) -> Result<FilteredQuery, AppError> {
    let mut query = query;

    // ── Issue Scalars ────────────────────────────────────────────────

    if let Some(raw) = params.state.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            query = query.filter(issues::Column::StateId.is_in(ids));
        }
    }

    if let Some(raw) = params.priority.as_deref() {
        let values: Vec<String> = split_csv(raw)
            .into_iter()
            .filter(|v| VALID_PRIORITIES.contains(v))
            .map(String::from)
            .collect();
        if !values.is_empty() {
            query = query.filter(issues::Column::Priority.is_in(values));
        }
    }

    query = apply_nullable_uuid_filter(
        query,
        params.created_by.as_deref(),
        issues::Column::CreatedById,
    );
    query = apply_nullable_uuid_filter(
        query,
        params.parent.as_deref(),
        issues::Column::ParentId,
    );

    // ── Text (search) ────────────────────────────────────────────────────────

    if let Some(name) = params.name.as_deref() {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            // Mirror of `name__icontains` in filter_name (issue_filters.py:~240).
            // Postgres `ILIKE` is case-insensitive; exact parity with Django
            // (and current use in `search.rs:104`).
            //
            // `%` and `_` in input act as SQL wildcards, same behavior
            // as Django — not escaped to maintain UX parity.
            let pattern = format!("%{}%", trimmed);
            query = query.filter(
                sea_orm::sea_query::Expr::col(issues::Column::Name).ilike(pattern),
            );
        }
    }

    // ── Simple dates ────────────────────────────────────────────────────────

    if let Some(raw) = params.start_date.as_deref() {
        query = apply_date_filter(query, raw, issues::Column::StartDate);
    }
    if let Some(raw) = params.target_date.as_deref() {
        query = apply_date_filter(query, raw, issues::Column::TargetDate);
    }

    // ── type (all/backlog/active) → state IDs by group ──────────────────────

    if let Some(type_filter) = params.type_filter.as_deref() {
        let groups: &[&str] = match type_filter {
            "backlog" => &["backlog"],
            "active" => &["unstarted", "started"],
            // `all` (and unknown values) DO NOT apply filter here. Django
            // (issue_filters.py:298,304) explicitly puts `group IN
            // [backlog, unstarted, started, completed, cancelled]` — excludes
            // `triage`. In our port, `triage` is already excluded before
            // by `load_triage_state_ids + is_not_in` in handler, so
            // omitting IN here is functionally equivalent and saves
            // a state IDs pre-query.
            _ => &[],
        };
        if !groups.is_empty() {
            let state_ids = load_state_ids_by_groups(db, workspace_id, groups).await?;
            if state_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::StateId.is_in(state_ids));
        }
    }

    // ── state_group (explicit, distinct from `type`) ──────────────────────────

    if let Some(raw) = params.state_group.as_deref() {
        let groups: Vec<&str> = split_csv(raw)
            .into_iter()
            .filter(|g| VALID_STATE_GROUPS.contains(g))
            .collect();
        if !groups.is_empty() {
            let state_ids = load_state_ids_by_groups(db, workspace_id, &groups).await?;
            if state_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::StateId.is_in(state_ids));
        }
    }

    // ── start_target_date (toggle) ────────────────────────────────────────────

    if matches!(params.start_target_date.as_deref(), Some("true")) {
        query = query
            .filter(issues::Column::StartDate.is_not_null())
            .filter(issues::Column::TargetDate.is_not_null());
    }

    // ── m2m via pre-query ─────────────────────────────────────────────────────
    //
    // Each m2m filter present adds 1 extra round-trip and an
    // `IN (issue_ids)` to the main query. If pre-query returns
    // empty → FilteredQuery::Empty (0 results guaranteed).

    if let Some(raw) = params.labels.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            let issue_ids = load_issues_with_labels(db, workspace_id, ids).await?;
            if issue_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::Id.is_in(issue_ids));
        }
    }

    if let Some(raw) = params.assignees.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            let issue_ids = load_issues_with_assignees(db, workspace_id, ids).await?;
            if issue_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::Id.is_in(issue_ids));
        }
    }

    if let Some(raw) = params.subscriber.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            let issue_ids = load_issues_with_subscribers(db, workspace_id, ids).await?;
            if issue_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::Id.is_in(issue_ids));
        }
    }

    if let Some(raw) = params.module.as_deref() {
        let ids = parse_uuids_csv(raw);
        let has_none = csv_contains_none(raw);
        match apply_module_membership(db, query, workspace_id, ids, has_none).await? {
            FilteredQuery::Active(q) => query = q,
            FilteredQuery::Empty => return Ok(FilteredQuery::Empty),
        }
    }

    if let Some(raw) = params.cycle.as_deref() {
        let ids = parse_uuids_csv(raw);
        let has_none = csv_contains_none(raw);
        match apply_cycle_membership(db, query, workspace_id, ids, has_none).await? {
            FilteredQuery::Active(q) => query = q,
            FilteredQuery::Empty => return Ok(FilteredQuery::Empty),
        }
    }

    Ok(FilteredQuery::Active(query))
}

// ── Helper: created_by / parent ───────────────────────────────────────────────

/// Applies a filter of type `(col IN (ids)) OR (col IS NULL if "None" present)`.
/// Handles the repeated pattern for `created_by` and `parent` (could extend
/// to other nullable UUIDs).
fn apply_nullable_uuid_filter(
    query: Select<issues::Entity>,
    raw_opt: Option<&str>,
    column: issues::Column,
) -> Select<issues::Entity> {
    let Some(raw) = raw_opt else {
        return query;
    };
    let ids = parse_uuids_csv(raw);
    let has_none = csv_contains_none(raw);

    let mut cond = Condition::any();
    let mut touched = false;
    if has_none {
        cond = cond.add(column.is_null());
        touched = true;
    }
    if !ids.is_empty() {
        cond = cond.add(column.is_in(ids));
        touched = true;
    }
    if touched {
        query.filter(cond)
    } else {
        query
    }
}

// ── Helper: dates ────────────────────────────────────────────────────────────

fn apply_date_filter(
    mut query: Select<issues::Entity>,
    raw: &str,
    column: issues::Column,
) -> Select<issues::Entity> {
    // Supported per-value format:
    //   "YYYY-MM-DD"          → column = date
    //   "YYYY-MM-DD;after"    → column >= date
    //   "YYYY-MM-DD;before"   → column <= date
    //
    // Multiple comma-separated values are combined with AND (Django
    // `date_filter` mirror when receiving multiple tokens).
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = part.split(';').collect();
        let Ok(date) = chrono::NaiveDate::parse_from_str(tokens[0], "%Y-%m-%d") else {
            continue;
        };
        let direction = tokens.get(1).copied().unwrap_or("");
        query = match direction {
            "after" => query.filter(column.gte(date)),
            "before" => query.filter(column.lte(date)),
            _ => query.filter(column.eq(date)),
        };
    }
    query
}

// ── Helpers: m2m pre-queries ──────────────────────────────────────────────────

async fn load_issues_with_labels(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    label_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = issue_labels::Entity::find()
        .select_only()
        .column(issue_labels::Column::IssueId)
        .filter(issue_labels::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_labels::Column::LabelId.is_in(label_ids))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

async fn load_issues_with_assignees(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    assignee_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = issue_assignees::Entity::find()
        .select_only()
        .column(issue_assignees::Column::IssueId)
        .filter(issue_assignees::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_assignees::Column::AssigneeId.is_in(assignee_ids))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

/// Mirror of `filter_subscribed_issues` in `issue_filters.py:392-403`.
///
/// Scoped by `workspace_id` (cross-tenant defense) + `deleted_at IS NULL`
/// on bridge table (Django parity). Invalid UUIDs were already
/// discarded by `parse_uuids_csv`, so `is_in` receives only valid values
/// — no injection risk.
async fn load_issues_with_subscribers(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    subscriber_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = issue_subscribers::Entity::find()
        .select_only()
        .column(issue_subscribers::Column::IssueId)
        .filter(issue_subscribers::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_subscribers::Column::SubscriberId.is_in(subscriber_ids))
        .filter(issue_subscribers::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

async fn load_issues_in_modules(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    module_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = module_issues::Entity::find()
        .select_only()
        .column(module_issues::Column::IssueId)
        .filter(module_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(module_issues::Column::ModuleId.is_in(module_ids))
        .filter(module_issues::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

async fn apply_module_membership(
    db: &sea_orm::DatabaseConnection,
    query: Select<issues::Entity>,
    workspace_id: Uuid,
    module_ids: Vec<Uuid>,
    has_none: bool,
) -> Result<FilteredQuery, AppError> {
    if !has_none && module_ids.is_empty() {
        return Ok(FilteredQuery::Active(query));
    }

    // Subquery: issue_ids with at least one active module_issue in this workspace.
    // Used for the `None` branch (issues WITHOUT module) — `issue.id NOT IN (subq)`.
    // We prefer subquery over `load_all_ids` not to fetch potentially millions
    // of UUIDs into process memory.
    let without_link_subq = module_issues::Entity::find()
        .select_only()
        .column(module_issues::Column::IssueId)
        .filter(module_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(module_issues::Column::DeletedAt.is_null())
        .into_query();

    let filtered = match (has_none, module_ids.is_empty()) {
        (true, true) => {
            // Only "None" → issues without any module.
            query.filter(issues::Column::Id.not_in_subquery(without_link_subq))
        }
        (false, false) => {
            // UUIDs only → existing behavior with pre-query.
            let matching = load_issues_in_modules(db, workspace_id, module_ids).await?;
            if matching.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query.filter(issues::Column::Id.is_in(matching))
        }
        (true, false) => {
            // None + UUIDs → union: (id IN matching) OR (id NOT IN any_link).
            // Mirror of Django `filter(**{"issue_module__module_id__in": [...],
            // "issue_module__module_id__isnull": True})` but with correct
            // semantics (Django would combine with AND, which is always empty
            // — here we interpret real user intent: "with these modules OR
            // without none").
            let matching = load_issues_in_modules(db, workspace_id, module_ids).await?;
            let cond = Condition::any()
                .add(issues::Column::Id.is_in(matching))
                .add(issues::Column::Id.not_in_subquery(without_link_subq));
            query.filter(cond)
        }
        (false, true) => unreachable!("covered by early-return"),
    };

    Ok(FilteredQuery::Active(filtered))
}

async fn apply_cycle_membership(
    db: &sea_orm::DatabaseConnection,
    query: Select<issues::Entity>,
    workspace_id: Uuid,
    cycle_ids: Vec<Uuid>,
    has_none: bool,
) -> Result<FilteredQuery, AppError> {
    if !has_none && cycle_ids.is_empty() {
        return Ok(FilteredQuery::Active(query));
    }

    let without_link_subq = cycle_issues::Entity::find()
        .select_only()
        .column(cycle_issues::Column::IssueId)
        .filter(cycle_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .into_query();

    let filtered = match (has_none, cycle_ids.is_empty()) {
        (true, true) => query.filter(issues::Column::Id.not_in_subquery(without_link_subq)),
        (false, false) => {
            let matching = load_issues_in_cycles(db, workspace_id, cycle_ids).await?;
            if matching.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query.filter(issues::Column::Id.is_in(matching))
        }
        (true, false) => {
            let matching = load_issues_in_cycles(db, workspace_id, cycle_ids).await?;
            let cond = Condition::any()
                .add(issues::Column::Id.is_in(matching))
                .add(issues::Column::Id.not_in_subquery(without_link_subq));
            query.filter(cond)
        }
        (false, true) => unreachable!("covered by early-return"),
    };

    Ok(FilteredQuery::Active(filtered))
}

async fn load_issues_in_cycles(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    cycle_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = cycle_issues::Entity::find()
        .select_only()
        .column(cycle_issues::Column::IssueId)
        .filter(cycle_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(cycle_issues::Column::CycleId.is_in(cycle_ids))
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

fn dedup(mut v: Vec<Uuid>) -> Vec<Uuid> {
    v.sort();
    v.dedup();
    v
}

// ── Helper: state groups → state IDs ──────────────────────────────────────────

async fn load_state_ids_by_groups(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    groups: &[&str],
) -> Result<Vec<Uuid>, AppError> {
    let groups_vec: Vec<String> = groups.iter().map(|s| s.to_string()).collect();
    let rows: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::WorkspaceId.eq(workspace_id))
        .filter(states::Column::Group.is_in(groups_vec))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}
