// src/jobs/scheduled.rs
//! Scheduled periodic tasks.
//!
//! Equivalent to `plane/bgtasks/issue_automation_task.py`.
//!
//! Tasks:
//!   - `archive_old_issues` — archives completed/cancelled issues that
//!     haven't had activity in N months (according to `project.archive_in`)
//!   - `close_old_issues`   — closes overdue issues (target_date < today)
//!     that are still in "started" or "unstarted" state
//!
//! These tasks are launched as apalis jobs with an external scheduler (cron),
//! or from a Tokio loop with `tokio::time::interval`.

use apalis::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::{issues, projects, states},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Job payloads ──────────────────────────────────────────────────────────────

/// Job triggered periodically to archive + close old issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIssueAutomationJob;


// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn handle_run_issue_automation(
    _job: RunIssueAutomationJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    let archived = archive_old_issues(&state).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "scheduled: archive_old_issues failed");
        0
    });

    let closed = close_old_issues(&state).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "scheduled: close_old_issues failed");
        0
    });

    tracing::info!(archived, closed, "scheduled: issue_automation completed");
    Ok(())
}

// ── archive_old_issues ────────────────────────────────────────────────────────

/// Archives issues from projects with `archive_in > 0` that:
/// - Are in "completed" or "cancelled" group state
/// - Haven't been updated in `archive_in` months
/// - Don't have `archived_at` already set
///
/// Returns the number of archived issues.
///
/// Visibility `pub` to allow integration tests (see
/// `tests/jobs_scheduler.rs`). Integration tests are in a separate crate,
/// so `pub(crate)` doesn't reach them. Allows invoking job logic without
/// setting up apalis storage (`handle_run_issue_automation` requires `apalis::Data<AppState>`).
pub async fn archive_old_issues(state: &AppState) -> anyhow::Result<u64> {
    // Projects with automatic archiving enabled
    let archive_projects = projects::Entity::find()
        .filter(projects::Column::ArchiveIn.gt(0))
        .filter(projects::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;

    let mut total_archived = 0u64;
    let now = chrono::Utc::now();

    for project in archive_projects {
        let archive_months = project.archive_in as i64;
        let cutoff = now - chrono::Duration::days(archive_months * 30);

        // Terminal states ("completed" and "cancelled" groups) of the project
        let terminal_states: Vec<uuid::Uuid> = states::Entity::find()
            .active()
            .filter(states::Column::ProjectId.eq(project.id))
            .filter(
                states::Column::Group
                    .eq("completed")
                    .or(states::Column::Group.eq("cancelled")),
            )
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| s.id)
            .collect();

        if terminal_states.is_empty() {
            continue;
        }

        // Eligible issues: in terminal state, not archived, no recent activity
        let candidates = issues::Entity::find()
            .active()
            .filter(issues::Column::ProjectId.eq(project.id))
            .filter(issues::Column::ArchivedAt.is_null())
            .filter(issues::Column::IsDraft.eq(false))
            .filter(issues::Column::StateId.is_in(terminal_states))
            .filter(
                issues::Column::UpdatedAt
                    .lt(chrono::DateTime::<chrono::FixedOffset>::from(cutoff)),
            )
            .all(&state.db)
            .await?;

        let archive_date = now.date_naive();
        for issue in candidates {
            let mut am: issues::ActiveModel = issue.into();
            am.archived_at = Set(Some(archive_date));
            am.update(&state.db).await?;
            total_archived += 1;
        }
    }

    Ok(total_archived)
}

// ── close_old_issues ─────────────────────────────────────────────────────────

/// Closes issues whose `target_date` has passed and are in "started" or "unstarted" states.
/// Moves them to the first "completed" state of the project.
///
/// Returns the number of closed issues.
///
/// Visibility `pub` for the same reason as `archive_old_issues` — allow
/// integration tests (separate crate from binary) without apalis scaffolding.
pub async fn close_old_issues(state: &AppState) -> anyhow::Result<u64> {
    // Projects with close_in > 0 (automatic closing)
    let close_projects = projects::Entity::find()
        .filter(projects::Column::CloseIn.gt(0))
        .filter(projects::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;

    let mut total_closed = 0u64;
    let today = chrono::Utc::now().date_naive();

    for project in close_projects {
        // Find target "completed" state
        let completed_state = states::Entity::find()
            .active()
            .filter(states::Column::ProjectId.eq(project.id))
            .filter(states::Column::Group.eq("completed"))
            .one(&state.db)
            .await?;

        let target_state = match completed_state {
            Some(s) => s,
            None => continue, // no completed state — don't close
        };

        // Active "started" / "unstarted" states
        let open_states: Vec<uuid::Uuid> = states::Entity::find()
            .active()
            .filter(states::Column::ProjectId.eq(project.id))
            .filter(
                states::Column::Group
                    .eq("started")
                    .or(states::Column::Group.eq("unstarted")),
            )
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| s.id)
            .collect();

        if open_states.is_empty() {
            continue;
        }

        let candidates = issues::Entity::find()
            .active()
            .filter(issues::Column::ProjectId.eq(project.id))
            .filter(issues::Column::StateId.is_in(open_states))
            .filter(issues::Column::ArchivedAt.is_null())
            .filter(issues::Column::IsDraft.eq(false))
            // target_date < today — overdue issue
            .filter(issues::Column::TargetDate.lt(today))
            .all(&state.db)
            .await?;

        for issue in candidates {
            let mut am: issues::ActiveModel = issue.into();
            am.state_id = Set(Some(target_state.id));
            am.completed_at = Set(Some(chrono::Utc::now().into()));
            am.update(&state.db).await?;
            total_closed += 1;
        }
    }

    Ok(total_closed)
}
