// src/jobs/instance_traces.rs
//! Periodic instance telemetry task.
//!
//! Equivalent to `plane/license/bgtasks/tracer.py → instance_traces`.
//!
//! Differences from Django:
//!   - Django uses OpenTelemetry with an external exporter (Jaeger/OTLP).
//!     Rust emits metrics via `tracing` (structured logging) so as not to
//!     add the opentelemetry dependency to the binary.
//!   - If the instance has `is_telemetry_enabled = false`, the task
//!     returns without emitting anything.

use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait};

use crate::{
    entities::{
        cycle_issues, cycles, instances, issues, module_issues, modules, pages, projects,
        users, workspaces,
    },
    utils::soft_delete::SoftDeleteExt,
};

/// Queries instance counts and emits them as a structured log.
///
/// Called every 6 hours by the scheduler (`cron.rs`).
pub async fn instance_traces(db: &DatabaseConnection) -> anyhow::Result<()> {
    // Get the first (and only) instance
    let instance = instances::Entity::find().one(db).await?;
    let instance = match instance {
        Some(i) => i,
        None => {
            tracing::debug!("instance_traces: instance not configured, skipping");
            return Ok(());
        }
    };

    if !instance.is_telemetry_enabled {
        tracing::debug!("instance_traces: telemetry disabled, skipping");
        return Ok(());
    }

    // Global counts
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
