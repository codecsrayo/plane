// src/jobs/scheduled.rs
//! Tareas programadas periódicas.
//!
//! Equivalente a `plane/bgtasks/issue_automation_task.py`.
//!
//! Tareas:
//!   - `archive_old_issues` — archiva issues completados/cancelados que no
//!     han tenido actividad en N meses (según `project.archive_in`)
//!   - `close_old_issues`   — cierra issues vencidos (target_date < hoy)
//!     que aún están en estado "started" o "unstarted"
//!
//! Estas tareas se lanzan como jobs de apalis con scheduler externo (cron),
//! o bien desde un loop de Tokio con `tokio::time::interval`.

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

/// Job disparado periódicamente para archivar + cerrar issues viejos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIssueAutomationJob;


// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn handle_run_issue_automation(
    _job: RunIssueAutomationJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    let archived = archive_old_issues(&state).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "scheduled: archive_old_issues falló");
        0
    });

    let closed = close_old_issues(&state).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "scheduled: close_old_issues falló");
        0
    });

    tracing::info!(archived, closed, "scheduled: issue_automation completado");
    Ok(())
}

// ── archive_old_issues ────────────────────────────────────────────────────────

/// Archiva issues de proyectos con `archive_in > 0` que:
/// - Están en estado grupo "completed" o "cancelled"
/// - No han sido actualizados en `archive_in` meses
/// - No tienen `archived_at` ya establecido
///
/// Retorna el número de issues archivados.
///
/// Visibilidad `pub` para permitir tests de integración (ver
/// `tests/jobs_scheduler.rs`). Los tests de integración están en un crate
/// separado, por lo que `pub(crate)` no los alcanza. Permite invocar la
/// lógica del job sin tener que montar el storage de apalis
/// (`handle_run_issue_automation` requiere `apalis::Data<AppState>`).
pub async fn archive_old_issues(state: &AppState) -> anyhow::Result<u64> {
    // Proyectos con archivado automático activado
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

        // Estados de grupos "completed" y "cancelled" del proyecto
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

        // Issues elegibles: en estado terminal, sin archivar, sin actividad reciente
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

/// Cierra issues cuya `target_date` ya pasó y están en estados "started"
/// o "unstarted". Mueve al primer estado "completed" del proyecto.
///
/// Retorna el número de issues cerrados.
///
/// Visibilidad `pub` por la misma razón que `archive_old_issues` — permitir
/// tests de integración (crate separado del binario) sin scaffolding de apalis.
pub async fn close_old_issues(state: &AppState) -> anyhow::Result<u64> {
    // Proyectos con close_in > 0 (cierre automático)
    let close_projects = projects::Entity::find()
        .filter(projects::Column::CloseIn.gt(0))
        .filter(projects::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;

    let mut total_closed = 0u64;
    let today = chrono::Utc::now().date_naive();

    for project in close_projects {
        // Buscar estado "completed" destino
        let completed_state = states::Entity::find()
            .active()
            .filter(states::Column::ProjectId.eq(project.id))
            .filter(states::Column::Group.eq("completed"))
            .one(&state.db)
            .await?;

        let target_state = match completed_state {
            Some(s) => s,
            None => continue, // no hay estado completado — no cerrar
        };

        // Estados activos "started" / "unstarted"
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
            // target_date < today — issue vencido
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
