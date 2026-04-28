// src/jobs/notifications.rs
//! Job: in-app notification creation for issue activities.
//!
//! Equivalent to `plane/bgtasks/notification_task.py`.
//!
//! Flow:
//!   1. Receive IssueActivityNotificationJob with activity ID
//!   2. Determine recipients: assignees + subscribers + creator + mentioned
//!   3. Insert records into `notifications` table (without duplicates)
//!
//! This job does not send emails — email delivery is the responsibility
//! of a separate job that reads `email_notification_logs`.

use apalis::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{
        issue_activities, issue_assignees, issue_subscribers, issues, notifications,
        project_members,
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Job payload ───────────────────────────────────────────────────────────────

/// Job payload: ID of the activity that triggered the notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueActivityNotificationJob {
    pub activity_id: Uuid,
}


// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn handle_issue_activity_notification(
    job: IssueActivityNotificationJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    if let Err(e) = run_notification(&state, job.activity_id).await {
        tracing::error!(
            activity_id = %job.activity_id,
            error = %e,
            "issue_activity_notification: job failed"
        );
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

async fn run_notification(state: &AppState, activity_id: Uuid) -> anyhow::Result<()> {
    use anyhow::Context as _;

    // 1. Load the activity
    let activity = issue_activities::Entity::find_by_id(activity_id)
        .one(&state.db)
        .await?
        .context("IssueActivity not found")?;

    let issue_id = match activity.issue_id {
        Some(id) => id,
        None => return Ok(()), // activity without associated issue — ignore
    };

    let actor_id = activity.actor_id;

    // 2. Load issue for context
    let issue = issues::Entity::find_by_id(issue_id)
        .active()
        .one(&state.db)
        .await?
        .context("Issue not found")?;

    // 3. Collect unique recipients (excluding the actor)
    let mut receiver_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

    // Issue creator
    if let Some(creator) = issue.created_by_id {
        receiver_ids.insert(creator);
    }

    // Active assignees
    let assignees = issue_assignees::Entity::find()
        .active()
        .filter(issue_assignees::Column::IssueId.eq(issue_id))
        .all(&state.db)
        .await?;
    for a in assignees {
        receiver_ids.insert(a.assignee_id);
    }

    // Active subscribers
    let subscribers = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .all(&state.db)
        .await?;
    for s in subscribers {
        receiver_ids.insert(s.subscriber_id);
    }

    // Exclude the actor — don't notify oneself
    if let Some(actor) = actor_id {
        receiver_ids.remove(&actor);
    }

    if receiver_ids.is_empty() {
        return Ok(());
    }

    // 4. Verify recipients are active project members
    let active_members: std::collections::HashSet<Uuid> = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(issue.project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await?
        .into_iter()
        .filter_map(|m| m.member_id)
        .collect();

    let valid_receivers: Vec<Uuid> = receiver_ids
        .into_iter()
        .filter(|id| active_members.contains(id))
        .collect();

    // 5. Build notification title
    let title = build_notification_title(&activity, &issue.name);
    let message_html = format!(
        "<p>{}</p>",
        ammonia::clean(&title)
    );

    // 6. Insert notifications (ignore duplicates: same activity + receiver)
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let triggered_by = activity.actor_id;

    for receiver_id in valid_receivers {
        // Check if a notification for this activity + receptor already exists
        // to ensure idempotency in case of job re-execution.
        let already_exists = notifications::Entity::find()
            .filter(notifications::Column::ReceiverId.eq(receiver_id))
            .filter(notifications::Column::EntityIdentifier.eq(activity_id))
            .filter(notifications::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?;

        if already_exists.is_some() {
            continue;
        }

        notifications::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title.clone()),
            message_html: Set(message_html.clone()),
            message_stripped: Set(Some(title.clone())),
            entity_identifier: Set(Some(activity_id)),
            entity_name: Set("issue_activity".to_owned()),
            sender: Set("in_app".to_owned()),
            receiver_id: Set(receiver_id),
            triggered_by_id: Set(triggered_by),
            project_id: Set(Some(issue.project_id)),
            workspace_id: Set(issue.workspace_id),
            created_by_id: Set(triggered_by),
            updated_by_id: Set(triggered_by),
            data: Set(Some(serde_json::json!({
                "issue_id": issue_id,
                "issue_name": issue.name,
                "activity_id": activity_id,
                "field": activity.field,
                "old_value": activity.old_value,
                "new_value": activity.new_value,
            }))),
            message: Set(None),
            read_at: Set(None),
            snoozed_till: Set(None),
            archived_at: Set(None),
            deleted_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&state.db)
        .await?;
    }

    tracing::debug!(
        activity_id = %activity_id,
        issue_id = %issue_id,
        "issue_activity_notification: notifications created"
    );

    Ok(())
}

/// Builds the readable notification title based on the modified field.
fn build_notification_title(activity: &issue_activities::Model, issue_name: &str) -> String {
    let field = activity.field.as_deref().unwrap_or("unknown");
    let new_val = activity.new_value.as_deref().unwrap_or("");

    match field {
        "state" => format!("State updated to «{new_val}» in «{issue_name}»"),
        "assignees" => format!("Assignees changed in «{issue_name}»"),
        "priority" => format!("Priority changed to «{new_val}» in «{issue_name}»"),
        "comment" => format!("New comment in «{issue_name}»"),
        "name" => format!("Title updated to «{new_val}»"),
        "description" => format!("Description updated in «{issue_name}»"),
        "target_date" => format!("Due date updated in «{issue_name}»"),
        "cycle" => format!("Issue moved to cycle «{new_val}»"),
        "module" => format!("Issue moved to module «{new_val}»"),
        _ => format!("Update in «{issue_name}»"),
    }
}
