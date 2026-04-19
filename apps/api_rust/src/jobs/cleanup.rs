// src/jobs/cleanup.rs
//! Tareas de limpieza periódica.
//!
//! Equivalentes a:
//!   - `plane/bgtasks/deletion_task.py`        → hard_delete
//!   - `plane/bgtasks/cleanup_task.py`          → delete_api_logs,
//!     delete_email_notification_logs, delete_page_versions,
//!     delete_issue_description_versions, delete_webhook_logs
//!   - `plane/bgtasks/exporter_expired_task.py` → delete_old_s3_links
//!   - `plane/bgtasks/file_asset_task.py`       → delete_unuploaded_file_assets
//!
//! Estas funciones se invocan directamente desde el scheduler (`cron.rs`),
//! no a través de apalis — son tareas sin estado de reintento.
//!
//! Nota: la versión Django intentaba archivar registros en MongoDB antes de
//! eliminarlos. En Rust se omite MongoDB (no está en el stack) y se elimina
//! directamente.

use std::collections::HashSet;

use aws_sdk_s3::Client as S3Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Statement,
};

use crate::entities::{exporters};

// ── hard_delete ───────────────────────────────────────────────────────────────

/// Tablas procesadas en orden hoja → raíz durante la pasada explícita.
/// Paridad con el bloque hardcodeado de `deletion_task.hard_delete()` en Django
/// (Workspace, Project, Cycle, Module, Issue, Page, IssueView, Label, State,
/// IssueActivity, IssueComment, IssueLink, IssueReaction, UserFavorite,
/// ModuleIssue, CycleIssue, Estimate, EstimatePoint). El orden está invertido
/// respecto a Django porque Rust no cascada en el ORM: tenemos que borrar
/// hijos antes que padres al nivel SQL.
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

/// Elimina definitivamente registros con `deleted_at` anterior a `days` días.
///
/// Equivalente a `plane/bgtasks/deletion_task.py::hard_delete()`.
///
/// Implementa dos pasadas, igual que la versión Django:
///
/// 1. **Pasada ordenada**: las 18 tablas de la jerarquía principal en orden
///    hoja → raíz. Cualquier fallo aquí aborta (el orden importa y un fallo
///    indica corrupción de estado).
/// 2. **Pasada catch-all**: descubre dinámicamente toda tabla en el schema
///    `public` con columna `deleted_at` (vía `information_schema`) y las
///    purga. Equivalente al loop `apps.get_models()` al final del
///    `hard_delete` de Django. Los errores por tabla se loguean como WARN
///    pero NO abortan el barrido (más resiliente que Django: Django aborta
///    toda la task si una sola tabla falla, lo cual es indeseable para una
///    tarea diaria de GC — mejor purgar las que podemos).
pub async fn hard_delete(db: &DatabaseConnection, days: i64) -> anyhow::Result<()> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_dt: chrono::DateTime<chrono::FixedOffset> = cutoff.into();

    let mut total_deleted = 0u64;

    // ── Pasada 1: tablas ordenadas (hoja → raíz) ──────────────────────────────
    for table in HARD_DELETE_ORDERED_TABLES {
        total_deleted += purge_soft_deleted(db, table, &cutoff_dt).await?;
    }

    // ── Pasada 2: catch-all sobre todas las tablas con `deleted_at` ──────────
    // Paridad con:
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
                "hard_delete: fallo al purgar tabla en catch-all, continuando"
            ),
        }
    }

    tracing::info!(total_deleted, days, "hard_delete completado");
    Ok(())
}

/// Ejecuta `DELETE FROM <table> WHERE deleted_at IS NOT NULL AND deleted_at < $1`.
///
/// Valida el identificador de tabla antes de interpolarlo para evitar
/// inyección SQL (defensa en profundidad: los nombres vienen de
/// `information_schema` y son siempre seguros, pero validamos igual).
async fn purge_soft_deleted(
    db: &DatabaseConnection,
    table: &str,
    cutoff_dt: &chrono::DateTime<chrono::FixedOffset>,
) -> anyhow::Result<u64> {
    if !is_safe_identifier(table) {
        anyhow::bail!("hard_delete: identificador de tabla inválido: {table:?}");
    }
    // El identificador se cita con comillas dobles (identifier quoting de SQL
    // estándar); el valor de cutoff va parametrizado.
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

/// Devuelve los nombres de las tablas `BASE TABLE` del schema `public` que
/// tienen una columna `deleted_at`.
///
/// Mirror del `hasattr(model, "deleted_at")` de Django, pero a nivel de
/// metadata de la BD (no depende de entities registrados en SeaORM).
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

/// Valida que un identificador sea `[a-zA-Z_][a-zA-Z0-9_]*` (snake_case
/// estándar de Postgres). Rechaza cualquier cosa con comillas, espacios,
/// punto y coma, etc. Defensa ante inyección SQL en el path del
/// `format!()` del DELETE.
fn is_safe_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 63 {
        // 63 es el límite de identificador de Postgres (NAMEDATALEN-1).
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
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
        assert!(!is_safe_identifier("1issues")); // no puede empezar con dígito
        assert!(!is_safe_identifier("issues; DROP TABLE x"));
        assert!(!is_safe_identifier("issues--"));
        assert!(!is_safe_identifier("\"issues\""));
        assert!(!is_safe_identifier("is sues"));
        assert!(!is_safe_identifier("issues.users"));
        assert!(!is_safe_identifier(&"a".repeat(64))); // > 63 chars
    }
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
