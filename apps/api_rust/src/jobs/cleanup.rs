// src/jobs/cleanup.rs
//! Periodic cleanup tasks.
//!
//! Equivalent to:
//!   - `plane/bgtasks/deletion_task.py`        → hard_delete
//!   - `plane/bgtasks/cleanup_task.py`          → delete_api_logs,
//!     delete_email_notification_logs, delete_page_versions,
//!     delete_issue_description_versions, delete_webhook_logs
//!   - `plane/bgtasks/exporter_expired_task.py` → delete_old_s3_links
//!   - `plane/bgtasks/file_asset_task.py`       → delete_unuploaded_file_assets
//!
//! These functions are called directly from the scheduler (`cron.rs`),
//! not through apalis — they are stateless tasks without retry.
//!
//! Note: the Django version attempted to archive records in MongoDB before
//! deleting them. In Rust, MongoDB is omitted (not in the stack) and
//! records are deleted directly.

use std::collections::HashSet;

use aws_sdk_s3::Client as S3Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Statement,
};

use crate::entities::{exporters};

// ── hard_delete ───────────────────────────────────────────────────────────────

/// Tables processed in leaf → root order during explicit pass.
/// Parity with the hardcoded block in `deletion_task.hard_delete()` in Django
/// (Workspace, Project, Cycle, Module, Issue, Page, IssueView, Label, State,
/// IssueActivity, IssueComment, IssueLink, IssueReaction, UserFavorite,
/// ModuleIssue, CycleIssue, Estimate, EstimatePoint). The order is inverted
/// with respect to Django because Rust does not cascade in the ORM: we have to delete
/// children before parents at the SQL level.
const HARD_DELETE_ORDERED_TABLES: &[&str] = &[
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

/// Permanently deletes records with `deleted_at` older than `days` days.
///
/// Equivalent to `plane/bgtasks/deletion_task.py::hard_delete()`.
///
/// Implements two passes, just like the Django version:
///
/// 1. **Ordered pass**: the 18 tables of the main hierarchy in order
///    leaf → root. Any failure here aborts (order matters and a failure
///    indicates state corruption).
/// 2. **Catch-all pass**: dynamically discovers all tables in the `public` schema
///    with a `deleted_at` column (via `information_schema`) and purges them.
///    Equivalent to the `apps.get_models()` loop at the end of
///    Django's `hard_delete`. Errors per table are logged as WARN
///    but DO NOT abort the sweep (more resilient than Django: Django aborts
///    the entire task if a single table fails, which is undesirable for a
///    daily GC task — better to purge those we can).
pub async fn hard_delete(db: &DatabaseConnection, days: i64) -> anyhow::Result<()> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let mut total_deleted = 0u64;

    // ── Pass 1: ordered tables (leaf → root) ──────────────────────────────
    for table in HARD_DELETE_ORDERED_TABLES {
        total_deleted += purge_soft_deleted(db, table, &cutoff_dt).await?;
    }

    // ── Pass 2: catch-all over all tables with `deleted_at` ──────────
    // Parity with:
    //     for model in apps.get_models():
    //         if hasattr(model, "deleted_at"):
    //             model.all_objects.filter(deleted_at__lt=cutoff).delete()
    let already_handled: HashSet<&str> = HARD_DELETE_ORDERED_TABLES.iter().copied().collect();
    let discovered = find_tables_with_deleted_at(db).await?;
    for table in discovered {
        if already_handled.contains(table.as_str()) {
            continue;
        }
        match purge_soft_deleted(db, &table, &cutoff_dt).await {
            Ok(n) => total_deleted += n,
            Err(e) => tracing::warn!(
                table = %table,
                error = %e,
                "hard_delete: failed to purge table in catch-all, continuing"
            ),
        }
    }

    tracing::info!(total_deleted, days, "hard_delete completed");
    Ok(())
}

/// Executes `DELETE FROM <table> WHERE deleted_at IS NOT NULL AND deleted_at < $1`.
///
/// Validates the table identifier before interpolating it to prevent
/// SQL injection (defense in depth: names come from `information_schema`
/// and are always safe, but we validate anyway).
async fn purge_soft_deleted(
    db: &DatabaseConnection,
    table: &str,
    cutoff_dt: &chrono::DateTime<chrono::FixedOffset>,
) -> anyhow::Result<u64> {
    if !is_safe_identifier(table) {
        anyhow::bail!("hard_delete: invalid table identifier: {table:?}");
    }
    // Identifier is quoted with double quotes (standard SQL identifier quoting);
    // the cutoff value is parameterized.
    let sql = format!(
        r#"DELETE FROM "{table}" WHERE deleted_at IS NOT NULL AND deleted_at < $1"#
    );
    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        &sql,
        vec![(*cutoff_dt).into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    if n > 0 {
        tracing::info!(table, deleted = n, "hard_delete: purged rows");
    }
    Ok(n)
}

/// Returns the names of `BASE TABLE` tables in the `public` schema that
/// have a `deleted_at` column.
///
/// Mirror of Django's `hasattr(model, "deleted_at")`, but at the
/// DB metadata level (does not depend on entities registered in SeaORM).
async fn find_tables_with_deleted_at(
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<String>> {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT c.table_name
        FROM   information_schema.columns c
        JOIN   information_schema.tables  t
          ON   t.table_schema = c.table_schema
         AND   t.table_name   = c.table_name
        WHERE  c.table_schema = 'public'
          AND  c.column_name  = 'deleted_at'
          AND  t.table_type   = 'BASE TABLE'
        ORDER BY c.table_name
        "#
        .to_owned(),
    );
    let rows = db.query_all(stmt).await?;
    let tables = rows
        .iter()
        .filter_map(|r| r.try_get::<String>("", "table_name").ok())
        .collect();
    Ok(tables)
}

/// Validates that an identifier is `[a-zA-Z_][a-zA-Z0-9_]*` (standard Postgres
/// snake_case). Rejects anything with quotes, spaces, semicolons, etc.
/// Defense against SQL injection in the `format!()` path of the DELETE.
fn is_safe_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 63 {
        // 63 is the identifier limit for Postgres (NAMEDATALEN-1).
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ── delete_api_logs ───────────────────────────────────────────────────────────

/// Deletes `api_activity_logs` records older than `days` days.
pub async fn delete_api_logs(db: &DatabaseConnection, days: i64) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "DELETE FROM api_activity_logs WHERE created_at <= $1",
        vec![cutoff_dt.into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, days, "delete_api_logs completed");
    Ok(n)
}

// ── delete_email_notification_logs ───────────────────────────────────────────

/// Deletes `email_notification_logs` sent more than `days` days ago.
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
    tracing::info!(deleted = n, days, "delete_email_notification_logs completed");
    Ok(n)
}

// ── delete_page_versions ─────────────────────────────────────────────────────

/// Deletes page versions exceeding the 20 most recent per page.
///
/// Uses a window function (`ROW_NUMBER`) to identify the oldest records.
/// Equivalent to Django's subquery with `annotate(row_num=Window(...))`.
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
    tracing::info!(deleted = n, "delete_page_versions completed");
    Ok(n)
}

// ── delete_issue_description_versions ────────────────────────────────────────

/// Deletes issue description versions exceeding the 20 most recent per issue.
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
    tracing::info!(deleted = n, "delete_issue_description_versions completed");
    Ok(n)
}

// ── delete_webhook_logs ───────────────────────────────────────────────────────

/// Deletes `webhook_logs` older than `days` days.
pub async fn delete_webhook_logs(db: &DatabaseConnection, days: i64) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "DELETE FROM webhook_logs WHERE created_at <= $1",
        vec![cutoff_dt.into()],
    );
    let n = db.execute(stmt).await?.rows_affected();
    tracing::info!(deleted = n, days, "delete_webhook_logs completed");
    Ok(n)
}

// ── delete_old_s3_links ───────────────────────────────────────────────────────

/// Deletes S3 objects and clears the URL in `exporter_history` for records
/// more than 8 days old.
///
/// Equivalent to `exporter_expired_task.delete_old_s3_link()`.
pub async fn delete_old_s3_links(
    db: &DatabaseConnection,
    s3: &S3Client,
    bucket: &str,
) -> anyhow::Result<u64> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(8);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    // Records with active URL created more than 8 days ago
    let expired = exporters::Entity::find()
        .filter(exporters::Column::Url.is_not_null())
        .filter(exporters::Column::CreatedAt.lte(cutoff_dt))
        .all(db)
        .await?;

    let count = expired.len() as u64;
    for record in expired {
        // Delete object from S3 (best-effort — error is not fatal)
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
                    "delete_old_s3_links: failed to delete S3 object"
                );
            }
        }

        // Set url to NULL in the DB
        let mut am: exporters::ActiveModel = record.into();
        am.url = Set(None);
        am.update(db).await?;
    }

    tracing::info!(deleted = count, "delete_old_s3_links completed");
    Ok(count)
}

// ── delete_unuploaded_file_assets ─────────────────────────────────────────────

/// Deletes `file_assets` that did not complete upload and are older than `days` days.
///
/// Equivalent to `file_asset_task.delete_unuploaded_file_asset()`.
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
    tracing::info!(deleted = n, days, "delete_unuploaded_file_assets completed");
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::is_safe_identifier;

    #[test]
    fn safe_identifier_accepts_snake_case() {
        assert!(is_safe_identifier("issues"));
        assert!(is_safe_identifier("issue_description_versions"));
        assert!(is_safe_identifier("_internal"));
        assert!(is_safe_identifier("t1"));
    }

    #[test]
    fn safe_identifier_rejects_injection_attempts() {
        assert!(!is_safe_identifier(""));
        assert!(!is_safe_identifier("1issues")); // cannot start with digit
        assert!(!is_safe_identifier("issues; DROP TABLE x"));
        assert!(!is_safe_identifier("issues--"));
        assert!(!is_safe_identifier("\"issues\""));
        assert!(!is_safe_identifier("is sues"));
        assert!(!is_safe_identifier("issues.users"));
        assert!(!is_safe_identifier(&"a".repeat(64))); // > 63 chars
    }
}
