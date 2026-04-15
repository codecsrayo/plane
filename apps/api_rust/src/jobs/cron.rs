// src/jobs/cron.rs
//! Scheduler periódico — reemplaza Celery beat.
//!
//! Equivalente a `plane/celery.py → app.conf.beat_schedule`.
//!
//! Cada tarea se lanza en un `tokio::spawn` independiente con su propio
//! loop de sleep → execute → sleep. Para tareas "a las HH:MM UTC" se
//! calcula el tiempo hasta la próxima ocurrencia antes del primer tick;
//! después corre cada 24 h.
//!
//! Tabla de equivalencias:
//!
//! | Celery beat task                          | Frecuencia       | Función Rust                                    |
//! |-------------------------------------------|------------------|-------------------------------------------------|
//! | stack_email_notification                  | cada 5 min       | email_notification::stack_email_notification    |
//! | instance_traces                           | cada 6 h         | instance_traces::instance_traces                |
//! | hard_delete                               | diario 00:00 UTC | cleanup::hard_delete                            |
//! | archive_and_close_old_issues              | diario 01:00 UTC | (enqueue RunIssueAutomationJob via apalis)       |
//! | delete_old_s3_link (exporter)             | diario 01:30 UTC | cleanup::delete_old_s3_links                    |
//! | delete_unuploaded_file_asset              | diario 02:00 UTC | cleanup::delete_unuploaded_file_assets          |
//! | delete_api_logs                           | diario 02:30 UTC | cleanup::delete_api_logs                        |
//! | delete_email_notification_logs            | diario 02:45 UTC | cleanup::delete_email_notification_logs         |
//! | delete_page_versions                      | diario 03:00 UTC | cleanup::delete_page_versions                   |
//! | delete_issue_description_versions         | diario 03:15 UTC | cleanup::delete_issue_description_versions      |
//! | delete_webhook_logs                       | diario 03:30 UTC | cleanup::delete_webhook_logs                    |

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

/// Inicia todos los workers del scheduler. Se llama una sola vez en `main`.
/// Cada tarea corre en un `tokio::spawn` independiente — un panic en una
/// no afecta a las demás.
pub async fn start_cron(state: AppState) {
    tracing::info!("🕐 Iniciando scheduler de tareas periódicas (reemplaza Celery beat)");

    // ── Cada 5 minutos ────────────────────────────────────────────────────────
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
                    tracing::error!(error = %e, "cron: stack_email_notification falló");
                }
            }
        });
    }

    // ── Cada 6 horas ─────────────────────────────────────────────────────────
    {
        let s = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(6 * 3600));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                if let Err(e) = instance_traces::instance_traces(&s.db).await {
                    tracing::error!(error = %e, "cron: instance_traces falló");
                }
            }
        });
    }

    // ── Tareas diarias a hora UTC fija ────────────────────────────────────────
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
                let s3 = build_s3_client(&s.config).await;
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

    tracing::info!("✅ Scheduler iniciado (11 tareas registradas)");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Ejecuta `f` todos los días a `hour:minute` UTC.
/// El primer tick espera hasta la próxima ocurrencia de esa hora;
/// los siguientes se disparan cada 24 h exactas.
async fn daily_at<F, Fut>(hour: u32, minute: u32, label: &'static str, f: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
{
    // Calcular tiempo hasta la próxima ocurrencia de HH:MM UTC
    let initial_delay = secs_until_utc(hour, minute);
    tracing::debug!(
        label,
        delay_secs = initial_delay,
        "cron: esperando primera ejecución"
    );
    sleep(Duration::from_secs(initial_delay)).await;

    loop {
        tracing::debug!(label, "cron: ejecutando tarea diaria");
        if let Err(e) = f().await {
            tracing::error!(label, error = %e, "cron: tarea diaria falló");
        }
        // Próxima ejecución en 24 horas
        sleep(Duration::from_secs(24 * 3600)).await;
    }
}

/// Segundos desde ahora hasta la próxima ocurrencia de `hour:minute` UTC.
/// Si la hora ya pasó hoy, devuelve el tiempo hasta mañana a esa hora.
fn secs_until_utc(hour: u32, minute: u32) -> u64 {
    let now = Utc::now();
    let today_secs = now.num_seconds_from_midnight() as u64;
    let target_secs = (hour as u64) * 3600 + (minute as u64) * 60;

    if target_secs > today_secs {
        target_secs - today_secs
    } else {
        // La hora ya pasó hoy → esperar hasta mañana
        24 * 3600 - today_secs + target_secs
    }
}

/// Encola un `RunIssueAutomationJob` en apalis para que el worker lo procese.
async fn enqueue_issue_automation(state: &AppState) -> anyhow::Result<()> {
    let pg_pool = sqlx::PgPool::connect(&state.config.database_url).await?;
    let mut storage: PostgresStorage<RunIssueAutomationJob> =
        PostgresStorage::new(pg_pool);
    use apalis::prelude::Storage;
    storage.push(RunIssueAutomationJob).await?;
    tracing::info!("cron: RunIssueAutomationJob encolado");
    Ok(())
}
