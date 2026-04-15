// src/jobs/instance_traces.rs
//! Tarea periódica de telemetría de instancia.
//!
//! Equivalente a `plane/license/bgtasks/tracer.py → instance_traces`.
//!
//! Diferencias respecto a Django:
//!   - Django usa OpenTelemetry con un exporter externo (Jaeger/OTLP).
//!     Rust emite las métricas via `tracing` (structured logging) para no
//!     añadir la dependencia de opentelemetry al binario.
//!   - Si la instancia tiene `is_telemetry_enabled = false`, la tarea
//!     retorna sin emitir nada.

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};

use crate::{
    entities::{
        cycle_issues, cycles, instances, issues, module_issues, modules, pages, projects,
        users, workspaces,
    },
    utils::soft_delete::SoftDeleteExt,
};

/// Consulta conteos de la instancia y los emite como structured log.
///
/// Llamado cada 6 horas por el scheduler (`cron.rs`).
pub async fn instance_traces(db: &DatabaseConnection) -> anyhow::Result<()> {
    // Obtener la primera (y única) instancia
    let instance = instances::Entity::find().one(db).await?;
    let instance = match instance {
        Some(i) => i,
        None => {
            tracing::debug!("instance_traces: instancia no configurada, omitiendo");
            return Ok(());
        }
    };

    if !instance.is_telemetry_enabled {
        tracing::debug!("instance_traces: telemetría desactivada, omitiendo");
        return Ok(());
    }

    // Conteos globales
    let workspace_count = workspaces::Entity::find().active().count(db).await?;
    let user_count = users::Entity::find().count(db).await?;
    let project_count = projects::Entity::find().active().count(db).await?;
    let issue_count = issues::Entity::find().active().count(db).await?;
    let module_count = modules::Entity::find().active().count(db).await?;
    let cycle_count = cycles::Entity::find().active().count(db).await?;
    let cycle_issue_count = cycle_issues::Entity::find().active().count(db).await?;
    let module_issue_count = module_issues::Entity::find().active().count(db).await?;
    let page_count = pages::Entity::find().active().count(db).await?;

    tracing::info!(
        instance_id   = %instance.instance_id,
        instance_name = %instance.instance_name,
        version       = %instance.current_version,
        is_setup_done = instance.is_setup_done,
        users         = user_count,
        workspaces    = workspace_count,
        projects      = project_count,
        issues        = issue_count,
        modules       = module_count,
        cycles        = cycle_count,
        cycle_issues  = cycle_issue_count,
        module_issues = module_issue_count,
        pages         = page_count,
        "instance_traces"
    );

    Ok(())
}
