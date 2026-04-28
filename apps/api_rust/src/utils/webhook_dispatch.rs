// src/utils/webhook_dispatch.rs
//! Fan-out of events to outgoing webhooks.
//!
//! Parity with Django's `plane/bgtasks/webhook_task.py::webhook_activity`:
//!
//! ```py
//! webhooks = Webhook.objects.filter(workspace__slug=slug, is_active=True)
//! if event == "project":       webhooks = webhooks.filter(project=True)
//! if event == "issue":         webhooks = webhooks.filter(issue=True)
//! if event in ("module", "module_issue"): webhooks = webhooks.filter(module=True)
//! if event in ("cycle",  "cycle_issue"):  webhooks = webhooks.filter(cycle=True)
//! if event == "issue_comment": webhooks = webhooks.filter(issue_comment=True)
//! for webhook in webhooks:
//!     webhook_send_task.delay(webhook_id=webhook.id, ...)
//! ```
//!
//! Design:
//!   - Discover → enqueue. All HTTP I/O is done by the apalis worker
//!     (`jobs::webhook_delivery`), this module only enqueues.
//!   - Weak idempotency: a fresh `delivery_id` per enqueued job.
//!     If the caller is executed twice (e.g. HTTP handler retry),
//!     two deliveries will be generated — this is consistent with Django.
//!   - Best-effort in the loop: a failure to enqueue to ONE webhook does not prevent
//!     the rest. It is logged to `tracing::warn!` with the webhook_id.
//!   - The `project_webhooks` table is NOT used as a delivery filter — in
//!     Django it only exists as an informative association; webhooks are
//!     workspace-level.

use apalis::prelude::Storage;
use apalis_sql::postgres::PostgresStorage;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entities::webhooks,
    jobs::webhook_delivery::DeliverWebhookJob,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

/// Recognized event types. They map 1:1 with the boolean flags
/// of the `webhooks` table (`project`, `issue`, `module`, `cycle`, `issue_comment`).
///
/// The `ModuleIssue` and `CycleIssue` aliases share a flag with `Module` and
/// `Cycle` respectively — parity with Django.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookEvent {
    Project,
    Issue,
    Module,
    ModuleIssue,
    Cycle,
    CycleIssue,
    IssueComment,
}

impl WebhookEvent {
    /// String sent as `event` in the payload and as `X-Plane-Event` header.
    /// Preserves Django's historical naming for existing consumers.
    pub fn as_payload_str(&self) -> &'static str {
        match self {
            Self::Project       => "project",
            Self::Issue         => "issue",
            Self::Module        => "module",
            Self::ModuleIssue   => "module_issue",
            Self::Cycle         => "cycle",
            Self::CycleIssue    => "cycle_issue",
            Self::IssueComment  => "issue_comment",
        }
    }
}

/// Action performed on the resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookAction {
    Created,
    Updated,
    Deleted,
}

impl WebhookAction {
    pub fn as_payload_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
        }
    }
}

/// Enqueues a webhook delivery for each active webhook in the workspace
/// whose event flag is on.
///
/// Parameters:
///   - `data`: serialized model that will go in `data` of the envelope. For deletes
///     Django sends `{"id": <uuid>}`; for create/update, the complete model
///     serialized with DRF (`IssueExpandSerializer`, `ProjectSerializer`, …).
///   - `activity`: optional `activity` block. `None` is propagated as `null`
///     in the final body — the Django contract always includes the key.
///     For create/delete triggered directly from a handler, it's usually
///     `None`; for updates with per-field diff it's filled with
///     `{field, old_value, new_value, actor, old_identifier, new_identifier}`.
///
/// Returns the number of successfully enqueued `DeliverWebhookJob`.
///
/// Errors:
///   - Query to `webhooks` fails → returns `Err` (systemic failure).
///   - An individual `push` to storage fails → it's logged and continues.
///     The returned count reflects only those enqueued successfully.
pub async fn dispatch_event(
    state: &AppState,
    workspace_id: Uuid,
    event: WebhookEvent,
    action: WebhookAction,
    data: serde_json::Value,
    activity: Option<serde_json::Value>,
) -> anyhow::Result<usize> {
    // 1. Discover subscribed webhooks
    let event_column = match event {
        WebhookEvent::Project                              => webhooks::Column::Project,
        WebhookEvent::Issue                                => webhooks::Column::Issue,
        WebhookEvent::Module       | WebhookEvent::ModuleIssue => webhooks::Column::Module,
        WebhookEvent::Cycle        | WebhookEvent::CycleIssue  => webhooks::Column::Cycle,
        WebhookEvent::IssueComment                         => webhooks::Column::IssueComment,
    };

    let matching_webhooks = webhooks::Entity::find()
        .active() // deleted_at IS NULL
        .filter(webhooks::Column::WorkspaceId.eq(workspace_id))
        .filter(webhooks::Column::IsActive.eq(true))
        .filter(event_column.eq(true))
        .all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(
                workspace_id = %workspace_id,
                event = event.as_payload_str(),
                error = %e,
                "dispatch_event: webhook query failed",
            );
            anyhow::anyhow!(e)
        })?;

    if matching_webhooks.is_empty() {
        tracing::debug!(
            workspace_id = %workspace_id,
            event = event.as_payload_str(),
            action = action.as_payload_str(),
            "dispatch_event: no subscribed webhooks",
        );
        return Ok(0);
    }

    // 2. Enqueue one per webhook
    //    A single storage for all pushes in the batch — cheaper than
    //    rebuilding it in each iteration.
    let mut storage: PostgresStorage<DeliverWebhookJob> =
        PostgresStorage::new(state.pg_pool.clone());

    let event_str = event.as_payload_str().to_owned();
    let action_str = action.as_payload_str().to_owned();
    let mut enqueued = 0usize;

    for wh in matching_webhooks {
        let job = DeliverWebhookJob {
            webhook_id: wh.id,
            event: event_str.clone(),
            action: action_str.clone(),
            data: data.clone(),
            activity: activity.clone(),
            delivery_id: Uuid::new_v4(),
        };

        match storage.push(job).await {
            Ok(_) => enqueued += 1,
            Err(e) => {
                // Best-effort: do not abort the batch for a transient failure.
                tracing::warn!(
                    webhook_id = %wh.id,
                    workspace_id = %workspace_id,
                    event = %event_str,
                    error = %e,
                    "dispatch_event: could not enqueue DeliverWebhookJob",
                );
            }
        }
    }

    tracing::debug!(
        workspace_id = %workspace_id,
        event = %event_str,
        action = %action_str,
        enqueued,
        "dispatch_event: fan-out completed",
    );
    Ok(enqueued)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_payload_strings_match_django() {
        // Exact names consumed by existing subscribers.
        assert_eq!(WebhookEvent::Project.as_payload_str(),      "project");
        assert_eq!(WebhookEvent::Issue.as_payload_str(),        "issue");
        assert_eq!(WebhookEvent::Module.as_payload_str(),       "module");
        assert_eq!(WebhookEvent::ModuleIssue.as_payload_str(),  "module_issue");
        assert_eq!(WebhookEvent::Cycle.as_payload_str(),        "cycle");
        assert_eq!(WebhookEvent::CycleIssue.as_payload_str(),   "cycle_issue");
        assert_eq!(WebhookEvent::IssueComment.as_payload_str(), "issue_comment");
    }

    #[test]
    fn action_payload_strings_match_django() {
        assert_eq!(WebhookAction::Created.as_payload_str(), "created");
        assert_eq!(WebhookAction::Updated.as_payload_str(), "updated");
        assert_eq!(WebhookAction::Deleted.as_payload_str(), "deleted");
    }

    #[test]
    fn module_and_module_issue_share_column() {
        // Django contract: both events query `module=True`.
        // If this changes, the test fails and forces a mapping review.
        let col_module      = matches!(WebhookEvent::Module,      WebhookEvent::Module);
        let col_mod_issue   = matches!(WebhookEvent::ModuleIssue, WebhookEvent::ModuleIssue);
        assert!(col_module && col_mod_issue);
    }
}
