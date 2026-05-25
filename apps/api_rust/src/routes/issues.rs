// src/routes/issues.rs
//! Issues endpoints (work items).
//!
//! Implemented endpoints:
//!   GET    /api/workspaces/{slug}/projects/{project_id}/issues/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/issues/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/issues/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/issues/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/issues/{pk}/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{intake_issues, issue_assignees, issue_labels, issue_subscribers, issues, labels},
    error::AppError,
    routes::issue_pagination::{
        apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
        load_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
    },
    routes::issue_filters::{apply_issue_filters, merge_json_filters, FilteredQuery},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Shape of `GET /workspaces/{slug}/projects/{project_id}/issues/{pk}/`.
///
/// EXACT mirror of `IssueDetailSerializer` in
/// `apps/api/plane/app/serializers/issue.py:924-935`, which extends
/// `IssueSerializer` (line 760) with `description_html`, `is_subscribed`,
/// `is_intake`. The frontend (`packages/types/src/issues/issue.ts:TIssue`)
/// consumes exactly this shape.
///
/// # Differences with previous DTO (`IssueResponse`)
/// - No `workspace_id` — Django does not include it in the detail serializer.
/// - No `type_id` — also not in `IssueSerializer.Meta.fields`.
/// - Renames: `created_by_id → created_by`, `updated_by_id → updated_by`,
///   `estimate_point_id → estimate_point` (Django convention when the field
///   is declared as a FK in the serializer, not as a raw UUIDField).
/// - Added: `cycle_id`, `module_ids`, `sub_issues_count`, `attachment_count`,
///   `link_count`, `is_subscribed`, `is_intake`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub state_id: Option<Uuid>,
    pub sort_order: f64,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    // Django exposes the estimate FK as `estimate_point` (source of the FK),
    // not as raw `estimate_point_id`. The frontend reads `estimate_point`.
    #[serde(rename = "estimate_point")]
    pub estimate_point_id: Option<Uuid>,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    // Enriched — first active cycle associated with the issue.
    pub cycle_id: Option<Uuid>,
    // Enriched — arrays of M2M relationship IDs.
    pub module_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    // Enriched — aggregated counters.
    pub sub_issues_count: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    // Django renders audit FKs as `created_by`/`updated_by`
    // (not `_id`) because the serializer declares them as ForeignKey fields.
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub attachment_count: i64,
    pub link_count: i64,
    pub is_draft: bool,
    pub archived_at: Option<chrono::NaiveDate>,
    pub type_id: Option<Uuid>,
    // Extras specific to `IssueDetailSerializer` (not in the list shape).
    pub description_html: String,
    pub is_subscribed: bool,
    pub is_intake: bool,
}

/// Shape of `POST /workspaces/{slug}/projects/{project_id}/issues/`.
///
/// EXACT mirror of the `.values(...)` projection that Django uses in
/// `apps/api/plane/app/views/issue/base.py:427-454` after creating an issue.
///
/// # Differences with `IssueDetailResponse`
/// - Includes `deleted_at` (always `null` immediately after create, but
///   Django projects it — the frontend can read it without breaking).
/// - Omits `description_html`, `is_subscribed`, `is_intake` — the create view
///   does not expose them.
///
/// Keeping both shapes separate avoids the "union DTO with all optional fields"
/// antipattern, which loses information and confuses the consumer about
/// which endpoint is being called.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueCreateResponse {
    pub id: Uuid,
    pub name: String,
    pub state_id: Option<Uuid>,
    pub sort_order: f64,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    #[serde(rename = "estimate_point")]
    pub estimate_point_id: Option<Uuid>,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub cycle_id: Option<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    pub sub_issues_count: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub attachment_count: i64,
    pub link_count: i64,
    pub is_draft: bool,
    pub archived_at: Option<chrono::NaiveDate>,
    pub type_id: Option<Uuid>,
    pub deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

/// Shape of `POST /workspaces/{slug}/projects/{project_id}/issues/`.
///
/// Exact parity with `IssueCreateSerializer` (apps/api/plane/app/serializers/
/// issue.py:82) and with the payload the frontend builds from
/// `DEFAULT_WORK_ITEM_FORM_VALUES` (packages/constants/src/issue/modal.ts).
///
/// # Critical conventions
/// - `label_ids` / `assignee_ids` (plural + suffix) are the names used by
///   DRF and the frontend; renaming them silently broke deserialization
///   (fields remained `None` and the issue was created without assignees/labels).
/// - `estimate_point` without `_id` — DRF exposes the FK with that name when
///   declared as `PrimaryKeyRelatedField(source="estimate_point", ...)`.
/// - All `Option<Uuid>` / `Option<NaiveDate>` use deserializers that
///   convert `""` to `None`, because the frontend sends `state_id: ""` and
///   empty dates by default — native serde fails with 422.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIssueRequest {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub state_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub target_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub estimate_point: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub type_id: Option<Uuid>,
    // `deserialize_uuid_list_filter_nulls` — the frontend (react-hook-form)
    // sometimes initializes these arrays with `[null]` when the assignee/label
    // select starts at "unassigned". Native serde would fail with 422
    // upon encountering `null` inside `Vec<Uuid>`. Django tolerates it because
    // Postgres discards NULLs from the `IN (...)` at the end.
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub state_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub target_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub estimate_point: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub type_id: Option<Uuid>,
    // `deserialize_uuid_list_filter_nulls` — same reason as in
    // `CreateIssueRequest`: the frontend sends `[null]` from react-hook-form
    // when deselecting assignees or labels (regression observed with real
    // `PATCH /issues/{id}/` payload returning 422 with the message
    // `assignee_ids[0]: invalid type: null, expected a formatted UUID string`).
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub label_ids: Option<Vec<Uuid>>,
    pub is_draft: Option<bool>,
}

// ── Query params for list_issues ────────────────────────────────────────────

/// Query params for `GET /workspaces/{slug}/projects/{project_id}/issues/`.
/// Partial mirror of `IssueViewSet.list()` (apps/api/plane/app/views/issue/base.py:251).
///
/// Fields postponed to future iterations (not critical to unlock integration panel rendering):
///   - `group_by` / `sub_group_by`    → require dedicated paginators.
///   - filters from `issue_filters(...)` → labels, assignees, priority, etc.
#[derive(Debug, Deserialize)]
pub struct ListIssuesQuery {
    // ── Pagination / order ────────────────────────────────────────────────────
    pub cursor:   Option<String>,
    pub per_page: Option<u64>,
    pub order_by: Option<String>,

    // ── Simple toggles ───────────────────────────────────────────────────────
    /// `false` excludes sub-issues (mirror of `filter_sub_issue_toggle`
    /// in plane/utils/issue_filters.py:380).
    pub sub_issue: Option<String>,
    /// Incremental filter — only issues updated after this timestamp.
    /// Mirror of `updated_at__gt` in base.py:256.
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,

    // ── Filters delegated to `issue_filters` module ──────────────────────────
    //
    // Inlined here because axum's `Query<T>` uses `serde_urlencoded`, which
    // does not support `#[serde(flatten)]`. Duplication between this struct and
    // `WorkspaceIssuesQuery` is the cost — parsing and application logic
    // lives in one place (`issue_filters::apply_issue_filters`).
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
    /// (issue_filters.py:392-403). Shared with the rest of issue listings.
    pub subscriber:        Option<String>,
    #[serde(rename = "type")]
    pub type_filter:       Option<String>,
    pub start_target_date: Option<String>,

    // ── Rich filters (Django-parity subset) ─────────────────────────────
    //
    // JSON blob that the frontend sends in spreadsheet layout and saved views.
    // Parsed with `issue_filters::merge_json_filters` and merged into
    // flat params; unknown keys → 400.
    pub filters:           Option<String>,
}

impl ListIssuesQuery {
    /// Builds filter parameters to pass to
    /// `issue_filters::apply_issue_filters`. Fields are `Option<String>`,
    /// so the move is cheap (no extra allocations).
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

// ── DTO serialized in the paginated response ─────────────────────────────────

/// Exact mirror of `issue_on_results` in
/// `apps/api/plane/utils/grouper.py:93-141`.
///
/// These are the 23 fields Django exposes via `.values(*required_fields)` in the
/// paginated listing, plus the three enriched arrays (`assignee_ids`,
/// `label_ids`, `module_ids`).
///
/// Names are serialized exactly as in Django so the frontend
/// (`base-issues.store.ts` + `packages/types/src/issues/issue.ts`) requires
/// no changes. `state__group` uses explicit rename to respect Django's
/// double-underscore.
#[derive(Debug, Serialize)]
pub struct ProjectIssueItem {
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
    pub type_id:          Option<Uuid>,
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

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Builds the `IssueDetailResponse` shape for an individual issue,
/// reusing the shared helper `load_enrichment` + two detail-specific queries.
///
/// Mirror of annotations in `retrieve()` in
/// `apps/api/plane/app/views/issue/base.py:480-575`.
///
/// # Populated fields (vs commit 1, which left stubs)
/// - `cycle_id`, `module_ids`, `sub_issues_count`, `attachment_count`,
///   `link_count` — via `load_enrichment`.
/// - `is_subscribed` — specific query to `issue_subscribers`.
/// - `is_intake`    — specific query to `intake_issues` with status ∈ (-2, 0).
///
/// # Strategy
/// `load_enrichment` is designed for batching N issues; here we use it
/// with N=1. Overhead is one query per relationship instead of one query
/// per issue × relationship — still O(1) roundtrips. Maintaining a single
/// enrichment helper (instead of "single" and "batch" versions)
/// avoids drift between both code paths.
///
/// # `state_ids = &[]`
/// `IssueDetailResponse` does not expose `state__group` (only the listing uses it
/// via `ProjectIssueItem`). We pass an empty slice → `load_enrichment` does an
/// early-return for the states query.
async fn build_detail_response(
    db: &sea_orm::DatabaseConnection,
    issue_model: issues::Model,
    user_id: Uuid,
) -> Result<IssueDetailResponse, AppError> {
    let id = issue_model.id;
    let issue_ids = [id];
    let mut maps = load_enrichment(db, &issue_ids, &[]).await?;

    // is_subscribed — mirror of `Exists(IssueSubscriber.objects.filter(...))`
    // in base.py:566-574. We use `count > 0` as equivalent to EXISTS; the
    // composite index (issue_id, subscriber_id) makes the query O(log n).
    let is_subscribed = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(id))
        .filter(issue_subscribers::Column::SubscriberId.eq(user_id))
        .count(db)
        .await
        .map_err(AppError::Database)?
        > 0;

    // is_intake — mirror of pattern in base.py:1305-1313.
    //
    // # About divergence with main `retrieve()`
    // Django's `retrieve()` (base.py:480-575) DOES NOT annotate this field —
    // seems like an oversight, as `IssueDetailSerializer` declares it as
    // `BooleanField(read_only=True)`. The parallel endpoint that queries
    // issues by `sequence_id` (`base.py:1225-1314`) does annotate it.
    //
    // The Rust port populates it correctly in both cases. Statuses
    // `-2` (PENDING) and `0` (SNOOZED) are the ones that count as "in intake"
    // according to inbox logic.
    let is_intake = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::IssueId.eq(id))
        .filter(intake_issues::Column::Status.is_in(vec![-2_i32, 0]))
        .count(db)
        .await
        .map_err(AppError::Database)?
        > 0;

    Ok(IssueDetailResponse {
        id,
        name: issue_model.name,
        state_id: issue_model.state_id,
        sort_order: issue_model.sort_order,
        completed_at: issue_model.completed_at,
        estimate_point_id: issue_model.estimate_point_id,
        priority: issue_model.priority,
        start_date: issue_model.start_date,
        target_date: issue_model.target_date,
        sequence_id: issue_model.sequence_id,
        project_id: issue_model.project_id,
        parent_id: issue_model.parent_id,
        cycle_id: maps.cycles.remove(&id),
        module_ids: maps.modules.remove(&id).unwrap_or_default(),
        label_ids: maps.labels.remove(&id).unwrap_or_default(),
        assignee_ids: maps.assignees.remove(&id).unwrap_or_default(),
        sub_issues_count: maps.sub_counts.get(&id).copied().unwrap_or(0),
        attachment_count: maps.attachments.get(&id).copied().unwrap_or(0),
        link_count: maps.links.get(&id).copied().unwrap_or(0),
        created_at: issue_model.created_at,
        updated_at: issue_model.updated_at,
        created_by_id: issue_model.created_by_id,
        updated_by_id: issue_model.updated_by_id,
        is_draft: issue_model.is_draft,
        archived_at: issue_model.archived_at,
        type_id: issue_model.type_id,
        description_html: issue_model.description_html,
        is_subscribed,
        is_intake,
    })
}

/// Builds `IssueCreateResponse` shape — mirror of `.values(...)` projection
/// in `base.py:427-454`.
///
/// Reuses `load_enrichment` for cycle/modules/labels/assignees/counts.
/// Does not query `is_subscribed` or `is_intake` (Django does not project them in
/// create). Includes `deleted_at` directly from the model (always `None`
/// immediately after create, but we emit it for strict parity).
async fn build_create_response(
    db: &sea_orm::DatabaseConnection,
    issue_model: issues::Model,
) -> Result<IssueCreateResponse, AppError> {
    let id = issue_model.id;
    let issue_ids = [id];
    let mut maps = load_enrichment(db, &issue_ids, &[]).await?;

    Ok(IssueCreateResponse {
        id,
        name: issue_model.name,
        state_id: issue_model.state_id,
        sort_order: issue_model.sort_order,
        completed_at: issue_model.completed_at,
        estimate_point_id: issue_model.estimate_point_id,
        priority: issue_model.priority,
        start_date: issue_model.start_date,
        target_date: issue_model.target_date,
        sequence_id: issue_model.sequence_id,
        project_id: issue_model.project_id,
        parent_id: issue_model.parent_id,
        cycle_id: maps.cycles.remove(&id),
        module_ids: maps.modules.remove(&id).unwrap_or_default(),
        label_ids: maps.labels.remove(&id).unwrap_or_default(),
        assignee_ids: maps.assignees.remove(&id).unwrap_or_default(),
        sub_issues_count: maps.sub_counts.get(&id).copied().unwrap_or(0),
        attachment_count: maps.attachments.get(&id).copied().unwrap_or(0),
        link_count: maps.links.get(&id).copied().unwrap_or(0),
        created_at: issue_model.created_at,
        updated_at: issue_model.updated_at,
        created_by_id: issue_model.created_by_id,
        updated_by_id: issue_model.updated_by_id,
        is_draft: issue_model.is_draft,
        archived_at: issue_model.archived_at,
        type_id: issue_model.type_id,
        deleted_at: issue_model.deleted_at,
    })
}

/// Synchronizes issue assignees within a transaction.
/// Soft-deletes existing ones not in `new_ids`, inserts new ones.
pub(crate) async fn sync_assignees(
    txn: &sea_orm::DatabaseTransaction,
    issue_id: Uuid,
    project_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    // Soft-delete all existing ones
    let existing = issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.eq(issue_id))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for row in existing {
        let mut am: issue_assignees::ActiveModel = row.into();
        am.deleted_at = Set(Some(now));
        // Django uses auto_now=True for updated_at (TimeAuditModel).
        am.updated_at = Set(now);
        am.update(txn).await.map_err(AppError::Database)?;
    }

    // Insert new ones. Explicit created_at / updated_at because
    // issue_assignees::ActiveModelBehavior is empty and columns are
    // NOT NULL (same pattern as labels.rs / issues).
    for assignee_id in new_ids {
        issue_assignees::ActiveModel {
            id: Set(Uuid::new_v4()),
            issue_id: Set(issue_id),
            assignee_id: Set(*assignee_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(txn)
        .await
        .map_err(AppError::Database)?;
    }
    Ok(())
}

/// Synchronizes issue labels within a transaction.
pub(crate) async fn sync_labels(
    txn: &sea_orm::DatabaseTransaction,
    issue_id: Uuid,
    project_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    let existing = issue_labels::Entity::find()
        .filter(issue_labels::Column::IssueId.eq(issue_id))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .all(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for row in existing {
        let mut am: issue_labels::ActiveModel = row.into();
        am.deleted_at = Set(Some(now));
        // Django uses auto_now=True for updated_at (TimeAuditModel).
        am.updated_at = Set(now);
        am.update(txn).await.map_err(AppError::Database)?;
    }

    for label_id in new_ids {
        // Verify label belongs to project before inserting
        let exists = labels::Entity::find_by_id(*label_id)
            .filter(labels::Column::ProjectId.eq(project_id))
            .filter(labels::Column::DeletedAt.is_null())
            .one(txn)
            .await
            .map_err(AppError::Database)?;

        if exists.is_some() {
            // Explicit created_at / updated_at (same reason as
            // sync_assignees / labels.rs: ActiveModelBehavior empty,
            // NOT NULL columns).
            issue_labels::ActiveModel {
                id: Set(Uuid::new_v4()),
                issue_id: Set(issue_id),
                label_id: Set(*label_id),
                project_id: Set(project_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(actor_id)),
                updated_by_id: Set(Some(actor_id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(txn)
            .await
            .map_err(AppError::Database)?;
        }
    }
    Ok(())
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues/ ──────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/",
    tag = "Issues",
    params(
        ("slug"       = String, Path,  description = "Workspace slug"),
        ("project_id" = Uuid,   Path,  description = "Project ID"),
        ("cursor"     = Option<String>, Query, description = "Django Cursor: {per_page}:{page}:{is_prev}"),
        ("per_page"   = Option<u64>,    Query, description = "Page size (ignored if coming from cursor)"),
        ("order_by"   = Option<String>, Query, description = "Sort field, e.g.: -created_at"),
    ),
    responses(
        (status = 200, description = "Paginated list of project issues"),
        (status = 403, description = "No access"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<ListIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Parse `?filters=<JSON>` and merge recognized fields into
    // filter_params (state_group__in, priority__in, label_id__in, etc.).
    // Unknown keys return 400 to prevent silent data leaks.
    // See `issue_filters::merge_json_filters` for complete mapping.
    let mut filter_params = params.to_filter_params();
    merge_json_filters(params.filters.as_deref(), &mut filter_params)?;

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    // ── 1. Pagination ─────────────────────────────────────────────────────────
    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // ── 2. Mirror of `IssueManager.get_queryset` (db/models/issue.py:92-101) ──
    //
    // The Django manager applies 4 implicit exclusions to ALL listings
    // starting from `Issue.issue_objects`. The list also inherits
    // `SoftDeletionManager.active()` → `deleted_at IS NULL`.
    //
    //   1. deleted_at IS NULL                    ← `.active()`
    //   2. archived_at IS NULL                   ← explicit filter
    //   3. project.archived_at IS NULL           ← early-return if the project
    //                                               loaded by guard is already
    //                                               archived. Cannot change
    //                                               within this request.
    //   4. is_draft = false                      ← explicit filter
    //   5. state.group != 'triage'               ← pre-query of triage
    //                                               state_ids + NOT IN.
    //
    // Antipattern avoided: We DO NOT JOIN with `states` in every query;
    // triage state IDs are resolved in a single additional query.

    if guard.project.archived_at.is_some() {
        // Archived project → no visible issues (mirror of manager exclusion).
        // Return empty paginated shape.
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let triage_state_ids = load_triage_state_ids(db, project_id).await?;

    // ── 3. Project issues base query ──────────────────────────────────
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::IsDraft.eq(false));

    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    // ── 4. Guest restriction ──────────────────────────────────────────────────
    //
    // Mirror of base.py:297-308: if user is role=5 in this project AND
    // project has `guest_view_all_features=false`, they only see their own
    // issues (`created_by = user`).
    //
    // The guard already validated membership; `project_member.role` is the role in
    // this specific project.
    let is_restricted_guest = guard.project_member.role == 5 && !guard.project.guest_view_all_features;
    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    // Sub-issues toggle (mirror of `filter_sub_issue_toggle` in
    // plane/utils/issue_filters.py:380). `sub_issue=false` hides issues with
    // a parent; any other value (or absence) shows all.
    if matches!(params.sub_issue.as_deref(), Some("false")) {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // Sync delta (`updated_at__gt` in base.py:256). The frontend uses this
    // filter to refresh only what changed since the last poll.
    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // ── 4.5. Shared module filters ────────────────────────────────────
    //
    // `state`, `state_group`, `priority`, `created_by`, `parent`, `name`,
    // `start_date`, `target_date`, `labels`, `assignees`, `module`, `cycle`,
    // `type`, `start_target_date`. See `routes::issue_filters` for the
    // complete list and detailed mapping.
    //
    // `filter_params` already includes the merge of `?filters=<JSON>` blob made
    // at the beginning of the handler — use that version (DO NOT call `to_filter_params`
    // again, it would overshadow the merge and discard the JSON).
    //
    // `FilteredQuery::Empty` → some filter implies 0 guaranteed matches
    // (e.g. `labels=<uuid>` with no issue having that label). Early-return
    // with empty paginated shape.
    let workspace_id = guard.workspace.id;
    let filtered = apply_issue_filters(db, base_query, &filter_params, workspace_id).await?;
    let base_query = match filtered {
        FilteredQuery::Active(q) => q,
        FilteredQuery::Empty => return Ok(Json(empty_paginated_response(page_size))),
    };

    // ── 5. Total count ────────────────────────────────────────────────────────
    let total_results = base_query.clone().count(db).await.map_err(AppError::Database)?;

    // ── 6. Sorting ───────────────────────────────────────────────────────
    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    // ── 7. Offset pagination ──────────────────────────────────────────────────
    let start_index = current_page * page_size;
    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── 8. Batch enrichment (no N+1) ────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);

    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    // ── 9. Serialize to ProjectIssueItem (mirror `issue_on_results`) ──────────
    let results: Vec<ProjectIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());

            ProjectIssueItem {
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

    // ── 10. Paginated response ────────────────────────────────────────────────
    //
    // Shape mirror of `OffsetPaginator.paginate()` (plane/utils/paginator.py:715-730).
    // Frontend reads `total_count` and `results` in base-issues.store.ts:1272-1291.
    Ok(Json(paginated_response(
        results,
        page_size,
        current_page,
        total_results,
    )))
}

// ── POST /workspaces/{slug}/projects/{project_id}/issues/ ─────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Issue created"),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Permission denied"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateIssueRequest>,
) -> Result<(StatusCode, Json<IssueCreateResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    // Atomic next sequence_id retrieval within SERIALIZABLE.
    // Antipattern fixed: MAX() outside transaction is TOCTOU under concurrency.
    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let assignees = body.assignee_ids.clone().unwrap_or_default();
    let label_ids = body.label_ids.clone().unwrap_or_default();

    let raw_html = body.description_html.clone().unwrap_or_default();
    let description_html = crate::utils::content_validator::sanitize_description(&raw_html)
        .map_err(AppError::BadRequest)?;

    let issue = state
        .db
        .transaction_with_config::<_, issues::Model, AppError>(
            |txn| {
                let name = body.name.clone();
                let description_html = description_html.clone();
                let priority = body.priority.clone().unwrap_or_else(|| "none".to_owned());
                let state_id = body.state_id;
                let parent_id = body.parent_id;
                let start_date = body.start_date;
                let target_date = body.target_date;
                // Django exposes FK as `estimate_point` (no `_id`) over the wire,
                // but SeaORM model keeps `estimate_point_id` as column name.
                // Bridged here.
                let estimate_point_id = body.estimate_point;
                let type_id = body.type_id;
                let assignees = assignees.clone();
                let label_ids = label_ids.clone();
                Box::pin(async move {
                    // sequence_id = MAX(sequence_id) + 1 within project.
                    //
                    // NOTE 1: MAX() without GROUP BY always returns a row
                    // (even if table is empty, with NULL value). That's why
                    // we decode to Option<i32> and flatten over the
                    // Option<Option<i32>> returned by .one(); without the
                    // outermost Option, SeaORM would treat NULL as decode error
                    // ("A null value was encountered while decoding 0").
                    //
                    // NOTE 2: target is i32 (NOT i64). In Postgres
                    // MAX(INT4) → INT4; not promoted to BIGINT like in MySQL.
                    // Using i64 produces
                    // "mismatched types; Rust type Option<i64> (as SQL type
                    // INT8) is not compatible with SQL type INT4".
                    use sea_orm::QuerySelect;
                    let max_seq: Option<i32> = issues::Entity::find()
                        .filter(issues::Column::ProjectId.eq(project_id))
                        .select_only()
                        .column_as(
                            sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
                            "max_seq",
                        )
                        .into_tuple::<Option<i32>>()
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?
                        .flatten();

                    let sequence_id = max_seq.unwrap_or(0) + 1;

                    // created_at / updated_at explicitly set because
                    // issues::ActiveModelBehavior is empty (no before_save hook)
                    // and columns are NOT NULL. Leaving them as
                    // Default::default() made SeaORM send NULL and DB
                    // reject with 23502 ("violates not-null constraint").
                    // Same pattern as labels.rs / issue_extras.rs / pages.rs.
                    let now: chrono::DateTime<chrono::FixedOffset> =
                        chrono::Utc::now().into();

                    let new_issue = issues::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        name: Set(name),
                        description_html: Set(description_html),
                        description_json: Set(serde_json::json!({})),
                        priority: Set(priority),
                        state_id: Set(state_id),
                        parent_id: Set(parent_id),
                        start_date: Set(start_date),
                        target_date: Set(target_date),
                        sequence_id: Set(sequence_id),
                        sort_order: Set(65535.0),
                        project_id: Set(project_id),
                        workspace_id: Set(workspace_id),
                        created_by_id: Set(Some(user_id)),
                        updated_by_id: Set(Some(user_id)),
                        estimate_point_id: Set(estimate_point_id),
                        type_id: Set(type_id),
                        is_draft: Set(false),
                        description_stripped: Set(None),
                        created_at: Set(now),
                        updated_at: Set(now),
                        deleted_at: Set(None),
                        ..Default::default()
                    };

                    let issue = new_issue.insert(txn).await.map_err(AppError::Database)?;

                    // Sync assignees and labels within same transaction
                    if !assignees.is_empty() {
                        sync_assignees(txn, issue.id, project_id, workspace_id, user_id, &assignees).await?;
                    }
                    if !label_ids.is_empty() {
                        sync_labels(txn, issue.id, project_id, workspace_id, user_id, &label_ids).await?;
                    }

                    Ok(issue)
                })
            },
            Some(IsolationLevel::Serializable),
            None,
        )
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    let response = build_create_response(&state.db, issue).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues/{pk}/ ─────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 200, description = "Issue detail"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<IssueDetailResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let issue = issues::Entity::find_by_id(pk)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let response = build_detail_response(&state.db, issue, guard.user.id).await?;
    Ok(Json(response))
}

// ── PATCH /workspaces/{slug}/projects/{project_id}/issues/{pk}/ ───────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 200, description = "Updated issue (body: IssueDetailResponse)"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
/// Partially updates an issue.
///
/// # Response contract: 200 + IssueDetailResponse
/// Django responds with 204 No Content (`base.py:700`), but this forces
/// frontend to perform an extra GET or apply optimistic updates with
/// risk of divergence (race conditions with other writers, derivation
/// of fields like `updated_at`/`updated_by`/`label_ids`, etc.).
///
/// Returning the updated issue:
///   - eliminates the GET round-trip after each PATCH,
///   - is the source of truth for backend-calculated fields
///     (timestamps, M2M ids after label/assignee sync),
///   - maintains compatibility for clients that only check
///     `2xx` (common with fetch/axios — `response.ok` is `true`
///     for both 200 and 204).
///
/// Deliberate DIVERGENCE decision from Django; also documented in
/// the `responses` field of the `#[utoipa::path]` above.
pub async fn update_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueDetailResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let issue = issues::Entity::find_by_id(pk)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let assignees = body.assignee_ids.clone();
    let label_ids = body.label_ids.clone();

    let clean_html = if let Some(ref html) = body.description_html {
        Some(crate::utils::content_validator::sanitize_description(html)
            .map_err(AppError::BadRequest)?)
    } else {
        None
    };

    // Tx value is no longer discarded — we use it as base to build
    // response. `build_detail_response` re-reads with annotations (cycle_id,
    // assignee_ids, label_ids, etc.) to return full shape.
    let updated = state
        .db
        .transaction::<_, issues::Model, AppError>(|txn| {
            let mut am: issues::ActiveModel = issue.into();
            if let Some(name) = body.name.clone() {
                am.name = Set(name);
            }
            if let Some(html) = clean_html.clone() {
                am.description_html = Set(html);
            }
            if let Some(priority) = body.priority.clone() {
                am.priority = Set(priority);
            }
            // Use Option<Option<>> to differentiate "not sent" from "explicit null"
            // In PATCH, if field comes in body it's applied; if not, preserved.
            if body.state_id.is_some() {
                am.state_id = Set(body.state_id);
            }
            if body.parent_id.is_some() {
                am.parent_id = Set(body.parent_id);
            }
            if body.start_date.is_some() {
                am.start_date = Set(body.start_date);
            }
            if body.target_date.is_some() {
                am.target_date = Set(body.target_date);
            }
            if let Some(draft) = body.is_draft {
                am.is_draft = Set(draft);
            }
            // Wire: `estimate_point` (DRF parity) → column: `estimate_point_id`.
            if body.estimate_point.is_some() {
                am.estimate_point_id = Set(body.estimate_point);
            }
            if body.type_id.is_some() {
                am.type_id = Set(body.type_id);
            }
            am.updated_by_id = Set(Some(user_id));
            // Django uses auto_now=True for updated_at (TimeAuditModel). SeaORM would
            // leave it Unchanged and UPDATE wouldn't touch it; must set it.
            am.updated_at = Set(chrono::Utc::now().into());

            let assignees = assignees.clone();
            let label_ids = label_ids.clone();
            Box::pin(async move {
                let updated = am.update(txn).await.map_err(AppError::Database)?;

                if let Some(ref ids) = assignees {
                    sync_assignees(txn, updated.id, project_id, workspace_id, user_id, ids).await?;
                }
                if let Some(ref ids) = label_ids {
                    sync_labels(txn, updated.id, project_id, workspace_id, user_id, ids).await?;
                }
                Ok(updated)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    let response = build_detail_response(&state.db, updated, user_id).await?;
    Ok(Json(response))
}

// ── DELETE /workspaces/{slug}/projects/{project_id}/issues/{pk}/ ──────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let issue = issues::Entity::find_by_id(pk)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issues::ActiveModel = issue.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues/list/ ─────────────────
//
// Mirror of `IssueListEndpoint.get()` in `apps/api/plane/app/views/issue/base.py:80-133`.
// Receives `?issues=uuid1,uuid2,...` and returns the list of issues with those IDs
// (without pagination — client already knows the IDs it wants).
//
// Response shape: Vec<ProjectIssueItem> (same as list_issues shape).

#[derive(Debug, Deserialize)]
pub struct IssueListByIdsQuery {
    /// Comma-separated issue UUIDs.
    pub issues: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/list/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issues" = String, Query, description = "Comma-separated issue UUIDs"),
    ),
    responses(
        (status = 200, description = "List of issues by IDs"),
        (status = 400, description = "issues parameter is required"),
        (status = 403, description = "Permission denied"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues_by_ids(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<IssueListByIdsQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let raw_ids = params.issues.as_deref().unwrap_or("").trim().to_string();
    if raw_ids.is_empty() {
        return Err(AppError::BadRequest(
            "issues query parameter is required".to_string(),
        ));
    }

    // Parse comma-separated UUIDs; ignore empty/malformed strings
    // (mirror of Django list-comprehension: `[id for id in ids.split(",") if id != ""]`).
    let issue_ids: Vec<Uuid> = raw_ids
        .split(',')
        .filter_map(|s| s.trim().parse::<Uuid>().ok())
        .collect();

    if issue_ids.is_empty() {
        return Ok(Json(serde_json::Value::Array(vec![])).into_response());
    }

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    // Guest restriction: if role=5 and guest_view_all_features=false, only their issues.
    let is_restricted_guest =
        guard.project_member.role == 5 && !guard.project.guest_view_all_features;

    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::Id.is_in(issue_ids.clone()));

    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    let issue_models = base_query.all(db).await.map_err(AppError::Database)?;

    if issue_models.is_empty() {
        return Ok(Json(serde_json::Value::Array(vec![])).into_response());
    }

    let fetched_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &fetched_ids, &state_ids).await?;

    let results: Vec<ProjectIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());
            ProjectIssueItem {
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

    Ok(Json(results).into_response())
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues-detail/ ───────────────
//
// Mirror of `IssueDetailEndpoint.get()` in `apps/api/plane/app/views/issue/base.py:960-1090`.
// Returns a paginated page (Django OffsetPaginator shape) with full
// IssueListDetailSerializer shape (same as ProjectIssueItem).
//
// Key differences vs list_issues:
// - Includes archived and draft issues (doesn't apply IssueManager filters).
// - Filters by guest permissions (owner || guest_view_all_features).

#[derive(Debug, Deserialize)]
pub struct IssueDetailQuery {
    pub cursor: Option<String>,
    pub per_page: Option<u64>,
    pub order_by: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues-detail/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor"),
        ("order_by" = Option<String>, Query, description = "Sort field"),
    ),
    responses(
        (status = 200, description = "Paginated list of issues with detail"),
        (status = 403, description = "Permission denied"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues_detail(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<IssueDetailQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // Guest restriction
    let is_restricted_guest =
        guard.project_member.role == 5 && !guard.project.guest_view_all_features;

    // Unlike list_issues, IssueDetailEndpoint DOES NOT apply IssueManager
    // filters (archived_at, is_draft, triage) — exposes all issues.
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id));

    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    let total_results = base_query.clone().count(db).await.map_err(AppError::Database)?;

    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    let start_index = current_page * page_size;
    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if issue_models.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    let results: Vec<ProjectIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());
            ProjectIssueItem {
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

    Ok(Json(paginated_response(results, page_size, current_page, total_results)))
}

// ── GET /workspaces/{slug}/projects/{project_id}/v2/issues/ ──────────────────
//
// Mirror of `IssuePaginatedViewSet.list()` in
// `apps/api/plane/app/views/issue/base.py:803-958`.
//
// Differences vs list_issues (v1):
// - Sorted by `updated_at` ASC (delta sync for client).
// - Supports `?updated_at__gt=<datetime>` for incremental sync.
// - Optional `description_html` field when `?description=true`.
// - Applies same IssueManager exclusions (archived, draft, triage).
// - Guest restriction also applies.

#[derive(Debug, Serialize)]
pub struct V2IssueItem {
    pub id:               Uuid,
    pub name:             String,
    pub state_id:         Option<Uuid>,
    #[serde(rename = "state__group")]
    pub state_group:      Option<String>,
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
    pub type_id:          Option<Uuid>,
    pub created_at:       chrono::DateTime<chrono::FixedOffset>,
    pub updated_at:       chrono::DateTime<chrono::FixedOffset>,
    pub created_by:       Option<Uuid>,
    pub updated_by:       Option<Uuid>,
    pub is_draft:         bool,
    pub archived_at:      Option<chrono::NaiveDate>,
    pub module_ids:       Vec<Uuid>,
    pub label_ids:        Vec<Uuid>,
    pub assignee_ids:     Vec<Uuid>,
    pub link_count:       i64,
    pub attachment_count: i64,
    pub sub_issues_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_html: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct V2IssuesQuery {
    pub cursor:        Option<String>,
    pub per_page:      Option<u64>,
    /// ISO-8601 Date; only returns issues updated after this date.
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,
    /// If `"true"`, includes `description_html` in response.
    pub description:   Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/v2/issues/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor"),
        ("updated_at__gt" = Option<String>, Query, description = "Sync delta filter"),
        ("description" = Option<String>, Query, description = "Include description_html"),
    ),
    responses(
        (status = 200, description = "Paginated lightweight issues list (v2)"),
        (status = 403, description = "Permission denied"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues_v2(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<V2IssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // Archived project → empty (mirror IssueManager)
    if guard.project.archived_at.is_some() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let triage_state_ids = load_triage_state_ids(db, project_id).await?;

    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::IsDraft.eq(false));

    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    // Guest restriction
    let is_restricted_guest =
        guard.project_member.role == 5 && !guard.project.guest_view_all_features;
    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    // Sync delta filter
    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // v2 always sorts by updated_at ASC (for sequential delta sync)
    let ordered_query = base_query.clone().order_by(issues::Column::UpdatedAt, Order::Asc);

    let total_results = base_query.count(db).await.map_err(AppError::Database)?;

    let start_index = current_page * page_size;
    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if issue_models.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let include_description = params.description.as_deref() == Some("true");

    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    let results: Vec<V2IssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());
            let description_html = if include_description {
                Some(m.description_html.clone())
            } else {
                None
            };
            V2IssueItem {
                id,
                name: m.name,
                state_id: m.state_id,
                state_group,
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
                created_at: m.created_at,
                updated_at: m.updated_at,
                created_by: m.created_by_id,
                updated_by: m.updated_by_id,
                is_draft: m.is_draft,
                archived_at: m.archived_at,
                type_id: m.type_id,
                module_ids: enrich.modules.remove(&id).unwrap_or_default(),
                label_ids: enrich.labels.remove(&id).unwrap_or_default(),
                assignee_ids: enrich.assignees.remove(&id).unwrap_or_default(),
                link_count: enrich.links.get(&id).copied().unwrap_or(0),
                attachment_count: enrich.attachments.get(&id).copied().unwrap_or(0),
                sub_issues_count: enrich.sub_counts.get(&id).copied().unwrap_or(0),
                description_html,
            }
        })
        .collect();

    Ok(Json(paginated_response(results, page_size, current_page, total_results)))
}
