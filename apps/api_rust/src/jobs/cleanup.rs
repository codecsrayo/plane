// src/jobs/cleanup.rs
//! Tareas de limpieza periódica.
//!
//! Equivalentes a:
//!   - `plane/bgtasks/deletion_task.py`         → hard_delete
//!   - `plane/bgtasks/cleanup_task.py`           → delete_api_logs, delete_email_notification_logs,
//!                                                  delete_page_versions, delete_issue_description_versions,
//!                                                  delete_webhook_logs
//!   - `plane/bgtasks/exporter_expired_task.py`  → delete_old_s3_links
//!   - `plane/bgtasks/file_asset_task.py`        → delete_unuploaded_file_assets
//!
//! Estas funciones se invocan directamente desde el scheduler (`cron.rs`),
//! no a través de apalis — son tareas sin estado de reintento.
//!
//! Nota: la versión Django intentaba archivar registros en MongoDB antes de
//! eliminarlos. En Rust se omite MongoDB (no está en el stack) y se elimina
//! directamente.

use aws_sdk_s3::Client as S3Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Statement,
};

use crate::entities::{exporters};

// ── hard_delete ───────────────────────────────────────────────────────────────

/// Elimina definitivamente registros con `deleted_at` mayor a `days` días.
///
/// Equivalente a `deletion_task.hard_delete()`.
/// Procesa las entidades en orden de dependencia (hijos antes que padres)
/// para evitar violaciones de FK con ON DELETE RESTRICT.
pub async fn hard_delete(db: &DatabaseConnection, days: i64) -> anyhow::Result<()> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    // Hoja → raíz para respetar FKs
    let tables: &[&str] = &[
        "estimate_points",
        "estimates",
        "issue_reactions",
        "issue_links",
        "issue_comments",
        "issue_activities",
        "user_favorites",
        "module_issues",
        "cycle_issues",
        "issues",
        "states",
        "labels",
        "issue_views",
        "pages",
        "modules",
        "cycles",
        "projects",
        "workspaces",
    ];

    let mut total_deleted = 0u64;
    for table in tables {
        let sql = format!(
            "DELETE FROM {table} WHERE deleted_at IS NOT NULL AND deleted_at < $1"
        );
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            &sql,
            vec![cutoff_dt.into()],
        );
        let result: sea_orm::ExecResult = db.execute(stmt).await?;
        let n = result.rows_affected();
        if n > 0 {
            tracing::info!(table, deleted = n, "hard_delete: purged rows");
        }
        total_deleted += n;
    }

    tracing::info!(total_deleted, days, "hard_delete completado");
    Ok(())
}

// ── delete_api_logs ───────────────────────────────────────────────────────────

/// Elimina registros de `api_activity_logs` más antiguos que `days` días.
pub async fn delete_api_logs(db: &DatabaseConnection, days: i64) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "DELETE FROM api_activity_logs WHERE created_at <= $1",
        vec![cutoff_dt.into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, days, "delete_api_logs completado");
    Ok(n)
}

// ── delete_email_notification_logs ───────────────────────────────────────────

/// Elimina `email_notification_logs` enviados hace más de `days` días.
pub async fn delete_email_notification_logs(
    db: &DatabaseConnection,
    days: i64,
) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "DELETE FROM email_notification_logs WHERE sent_at IS NOT NULL AND sent_at <= $1",
        vec![cutoff_dt.into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, days, "delete_email_notification_logs completado");
    Ok(n)
}

// ── delete_page_versions ─────────────────────────────────────────────────────

/// Elimina versiones de página que excedan las 20 más recientes por página.
///
/// Usa una window function (`ROW_NUMBER`) para identificar los registros
/// más antiguos. Equivalente al subquery de Django con `annotate(row_num=Window(...))`.
pub async fn delete_page_versions(db: &DatabaseConnection) -> anyhow::Result<u64> {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        DELETE FROM page_versions
        WHERE id IN (
            SELECT id FROM (
                SELECT id,
                       ROW_NUMBER() OVER (
                           PARTITION BY page_id
                           ORDER BY created_at DESC
                       ) AS rn
                FROM page_versions
            ) ranked
            WHERE rn > 20
        )
        "#
        .to_owned(),
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, "delete_page_versions completado");
    Ok(n)
}

// ── delete_issue_description_versions ────────────────────────────────────────

/// Elimina versiones de descripción de issue que excedan las 20 más recientes por issue.
pub async fn delete_issue_description_versions(
    db: &DatabaseConnection,
) -> anyhow::Result<u64> {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        DELETE FROM issue_description_versions
        WHERE id IN (
            SELECT id FROM (
                SELECT id,
                       ROW_NUMBER() OVER (
                           PARTITION BY issue_id
                           ORDER BY created_at DESC
                       ) AS rn
                FROM issue_description_versions
            ) ranked
            WHERE rn > 20
        )
        "#
        .to_owned(),
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, "delete_issue_description_versions completado");
    Ok(n)
}

// ── delete_webhook_logs ───────────────────────────────────────────────────────

/// Elimina `webhook_logs` más antiguos que `days` días.
pub async fn delete_webhook_logs(db: &DatabaseConnection, days: i64) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "DELETE FROM webhook_logs WHERE created_at <= $1",
        vec![cutoff_dt.into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, days, "delete_webhook_logs completado");
    Ok(n)
}

// ── delete_old_s3_links ───────────────────────────────────────────────────────

/// Elimina objetos de S3 y limpia la URL en `exporter_history` para registros
/// con más de 8 días de antigüedad.
///
/// Equivalente a `exporter_expired_task.delete_old_s3_link()`.
pub async fn delete_old_s3_links(
    db: &DatabaseConnection,
    s3: &S3Client,
    bucket: &str,
) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(8);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    // Registros con URL activa creados hace más de 8 días
    let expired = exporters::Entity::find()
        .filter(exporters::Column::Url.is_not_null())
        .filter(exporters::Column::CreatedAt.lte(cutoff_dt))
        .all(db)
        .await?;

    let count = expired.len() as u64;
    for record in expired {
        // Eliminar objeto de S3 (best-effort — error no es fatal)
        if !record.key.is_empty() {
            if let Err(e) = s3
                .delete_object()
                .bucket(bucket)
                .key(&record.key)
                .send()
                .await
            {
                tracing::warn!(
                    key = %record.key,
                    error = %e,
                    "delete_old_s3_links: fallo al eliminar objeto S3"
                );
            }
        }

        // Poner url a NULL en la BD
        let mut am: exporters::ActiveModel = record.into();
        am.url = Set(None);
        am.update(db).await?;
    }

    tracing::info!(deleted = count, "delete_old_s3_links completado");
    Ok(count)
}

// ── delete_unuploaded_file_assets ─────────────────────────────────────────────

/// Elimina `file_assets` que no completaron la subida y tienen más de `days` días.
///
/// Equivalente a `file_asset_task.delete_unuploaded_file_asset()`.
pub async fn delete_unuploaded_file_assets(
    db: &DatabaseConnection,
    days: i64,
) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "DELETE FROM file_assets WHERE is_uploaded = false AND created_at < $1",
        vec![cutoff_dt.into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, days, "delete_unuploaded_file_assets completado");
    Ok(n)
}
