// src/jobs/cron.rs
//! Periodic scheduler — replaces Celery beat.
//!
//! Equivalent to `plane/celery.py → app.conf.beat_schedule`.
//!
//! Each task is launched in an independent `tokio::spawn` with its own
//! sleep → execute → sleep loop. For "at HH:MM UTC" tasks, the time until
//! the next occurrence is calculated before the first tick;
//! then it runs every 24 hours.
//!
//! Equivalency table:
//!
//! | Celery beat task                          | Frequency        | Rust function                                   |
//! |-------------------------------------------|------------------|-------------------------------------------------|
//! | stack_email_notification                  | every 5 min      | email_notification::stack_email_notification    |
//! | instance_traces                           | every 6 h         | instance_traces::instance_traces                |
//! | hard_delete                               | daily 00:00 UTC | cleanup::hard_delete                            |
//! | archive_and_close_old_issues              | daily 01:00 UTC | (enqueue RunIssueAutomationJob via apalis)       |
//! | delete_old_s3_link (exporter)             | daily 01:30 UTC | cleanup::delete_old_s3_links                    |
//! | delete_unuploaded_file_asset              | daily 02:00 UTC | cleanup::delete_unuploaded_file_assets          |
//! | delete_api_logs                           | daily 02:30 UTC | cleanup::delete_api_logs                        |
//! | delete_email_notification_logs            | daily 02:45 UTC | cleanup::delete_email_notification_logs         |
//! | delete_page_versions                      | daily 03:00 UTC | cleanup::delete_page_versions                   |
//! | delete_issue_description_versions         | daily 03:15 UTC | cleanup::delete_issue_description_versions      |
//! | delete_webhook_logs                       | daily 03:30 UTC | cleanup::delete_webhook_logs                    |

use std::time::Duration;

use apalis_sql::postgres::PostgresStorage;
use chrono::{Timelike, Utc};
use tokio::time::sleep;

use crate::{
    AppState,
    jobs::{
        cleanup,
        email_notification,
        instance_traces,
        scheduled::RunIssueAutomationJob,
    },
    utils::s3::build_s3_client,
};

/// Starts all scheduler workers. Called once in `main`.
/// Each task runs in an independent `tokio::spawn` — a panic in one
/// does not affect the others.
pub async fn start_cron(state: AppState) {
    tracing::info!("Starting periodic task scheduler (replaces Celery beat)");

    // ── Every 5 minutes ────────────────────────────────────────────────────────
    {
        let s = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if let Err(e) = email_notification::stack_email_notification(
                    &s.db,
                    &s.redis,
                    &s.config,
                )
                .await
                {
                    tracing::error!(error = %e, "cron: stack_email_notification failed");
                }
            }
        });
    }

    // ── Every 6 hours ─────────────────────────────────────────────────────────
    {
        let s = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(6 * 3600));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if let Err(e) = instance_traces::instance_traces(&s.db).await {
                    tracing::error!(error = %e, "cron: instance_traces failed");
                }
            }
        });
    }

    // ── Daily tasks at fixed UTC time ────────────────────────────────────────
    // 00:00 — hard_delete
    {
        let s = state.clone();
        tokio::spawn(daily_at(0, 0, "hard_delete", move || {
            let s = s.clone();
            async move {
                let days = s.config.hard_delete_after_days;
                cleanup::hard_delete(&s.db, days).await
            }
        }));
    }

    // 01:00 — archive_and_close_old_issues (enqueue apalis job)
    {
        let s = state.clone();
        tokio::spawn(daily_at(1, 0, "RunIssueAutomationJob enqueue", move || {
            let s = s.clone();
            async move { enqueue_issue_automation(&s).await }
        }));
    }

    // 01:30 — delete_old_s3_links
    {
        let s = state.clone();
        tokio::spawn(daily_at(1, 30, "delete_old_s3_links", move || {
            let s = s.clone();
            async move {
                let s3 = build_s3_client(&s.config);
                cleanup::delete_old_s3_links(&s.db, &s3, &s.config.aws_s3_bucket).await?;
                Ok(())
            }
        }));
    }

    // 02:00 — delete_unuploaded_file_assets
    {
        let s = state.clone();
        tokio::spawn(daily_at(2, 0, "delete_unuploaded_file_assets", move || {
            let s = s.clone();
            async move {
                let days = s.config.unuploaded_asset_delete_days;
                cleanup::delete_unuploaded_file_assets(&s.db, days).await?;
                Ok(())
            }
        }));
    }

    // 02:30 — delete_api_logs
    {
        let s = state.clone();
        tokio::spawn(daily_at(2, 30, "delete_api_logs", move || {
            let s = s.clone();
            async move {
                let days = s.config.hard_delete_after_days;
                cleanup::delete_api_logs(&s.db, days).await?;
                Ok(())
            }
        }));
    }

    // 02:45 — delete_email_notification_logs
    {
        let s = state.clone();
        tokio::spawn(daily_at(2, 45, "delete_email_notification_logs", move || {
            let s = s.clone();
            async move {
                let days = s.config.hard_delete_after_days;
                cleanup::delete_email_notification_logs(&s.db, days).await?;
                Ok(())
            }
        }));
    }

    // 03:00 — delete_page_versions
    {
        let s = state.clone();
        tokio::spawn(daily_at(3, 0, "delete_page_versions", move || {
            let s = s.clone();
            async move { cleanup::delete_page_versions(&s.db).await.map(|_| ()) }
        }));
    }

    // 03:15 — delete_issue_description_versions
    {
        let s = state.clone();
        tokio::spawn(daily_at(3, 15, "delete_issue_description_versions", move || {
            let s = s.clone();
            async move {
                cleanup::delete_issue_description_versions(&s.db)
                    .await
                    .map(|_| ())
            }
        }));
    }

    // 03:30 — delete_webhook_logs
    {
        let s = state.clone();
        tokio::spawn(daily_at(3, 30, "delete_webhook_logs", move || {
            let s = s.clone();
            async move {
                let days = s.config.hard_delete_after_days;
                cleanup::delete_webhook_logs(&s.db, days).await?;
                Ok(())
            }
        }));
    }

    tracing::info!("✅ Scheduler started (11 tasks registered)");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Executes `f` every day at `hour:minute` UTC.
/// The first tick waits until the next occurrence of that time;
/// subsequent ticks fire every exactly 24 hours.
async fn daily_at<F, Fut>(hour: u32, minute: u32, label: &'static str, f: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
{
    // Calculate time until next HH:MM UTC occurrence
    let initial_delay = secs_until_utc(hour, minute);
    tracing::debug!(
        label,
        delay_secs = initial_delay,
        "cron: waiting for first execution"
    );
    sleep(Duration::from_secs(initial_delay)).await;

    loop {
        tracing::debug!(label, "cron: executing daily task");
        if let Err(e) = f().await {
            tracing::error!(label, error = %e, "cron: daily task failed");
        }
        // Next execution in 24 hours
        sleep(Duration::from_secs(24 * 3600)).await;
    }
}

/// Seconds from now until the next occurrence of `hour:minute` UTC.
/// If the time has already passed today, returns the time until tomorrow at that time.
///
/// Visibility `pub` to allow integration tests (see
/// `tests/jobs_scheduler.rs`). Integration tests live in a separate crate
/// from the binary, so `pub(crate)` doesn't reach them. The function
/// is pure — all input is received as argument or via `Utc::now()` — so
/// it can be exercised directly without DB scaffolding.
pub fn secs_until_utc(hour: u32, minute: u32) -> u64 {
    let now = Utc::now();
    let today_secs = now.num_seconds_from_midnight() as u64;
    let target_secs = (hour as u64) * 3600 + (minute as u64) * 60;

    if target_secs > today_secs {
        target_secs - today_secs
    } else {
        // Time already passed today → wait until tomorrow
        24 * 3600 - today_secs + target_secs
    }
}

/// Enqueues a `RunIssueAutomationJob` in apalis for worker processing.
async fn enqueue_issue_automation(state: &AppState) -> anyhow::Result<()> {
    // Reuse shared PgPool from AppState instead of opening a new connection
    // on each cron tick (antipattern).
    let mut storage: PostgresStorage<RunIssueAutomationJob> =
        PostgresStorage::new(state.pg_pool.clone());
    use apalis::prelude::Storage;
    storage.push(RunIssueAutomationJob).await?;
    tracing::info!("cron: RunIssueAutomationJob enqueued");
    Ok(())
}
