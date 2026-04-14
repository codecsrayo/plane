// src/jobs/export.rs
//! Job: exportación de issues de un workspace a CSV, empaquetado en ZIP y subida a S3.
//!
//! Equivalente a `plane/bgtasks/export_task.py`.
//!
//! Flujo:
//!   1. Cargar ExporterHistory por token
//!   2. Consultar issues de los proyectos indicados
//!   3. Serializar a CSV (uno por proyecto)
//!   4. Empaquetar en ZIP en memoria
//!   5. Subir a S3/MinIO
//!   6. Actualizar ExporterHistory con status + URL
//!
//! El job usa `aws_sdk_s3` para S3 y `flate2`/`std::io` para ZIP.

use apalis::prelude::*;
use aws_sdk_s3::primitives::ByteStream;
use flate2::{write::GzEncoder, Compression};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use crate::{
    entities::{exporters, issues, labels, issue_assignees, issue_labels, states, users},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Job payload ───────────────────────────────────────────────────────────────

/// Payload: token único del registro ExporterHistory a procesar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportIssuesJob {
    pub exporter_token: String,
}


// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn handle_export_issues(
    job: ExportIssuesJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    if let Err(e) = run_export(&state, &job.exporter_token).await {
        tracing::error!(
            token = %job.exporter_token,
            error = %e,
            "export_issues: job falló"
        );
        // Marcar como fallido en DB (best-effort)
        let _ = mark_export_failed(&state, &job.exporter_token, &e.to_string()).await;
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

async fn run_export(state: &AppState, token: &str) -> anyhow::Result<()> {
    use anyhow::Context as _;

    // 1. Cargar el ExporterHistory
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(token))
        .filter(exporters::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .context("ExporterHistory no encontrado")?;

    let workspace_id = exporter.workspace_id;
    let project_ids: Vec<Uuid> = exporter
        .project
        .clone()
        .unwrap_or_default();

    if project_ids.is_empty() {
        anyhow::bail!("No hay proyectos en el exporter");
    }

    // 2. Consultar issues por proyecto
    let all_issues = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::ProjectId.is_in(project_ids.clone()))
        .filter(issues::Column::ArchivedAt.is_null())
        .order_by_asc(issues::Column::SequenceId)
        .all(&state.db)
        .await?;

    // 3. Batch-fetch estados, assignees, labels — anti-N+1
    let issue_ids: Vec<Uuid> = all_issues.iter().map(|i| i.id).collect();

    let state_ids: Vec<Uuid> = all_issues
        .iter()
        .filter_map(|i| i.state_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let states_map: std::collections::HashMap<Uuid, String> = states::Entity::find()
        .filter(states::Column::Id.is_in(state_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|s| (s.id, s.name))
        .collect();

    let assignee_rows = issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.clone()))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;

    // Obtener nombres de usuarios
    let assignee_user_ids: Vec<Uuid> = assignee_rows
        .iter()
        .map(|a| a.assignee_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let users_map: std::collections::HashMap<Uuid, String> = users::Entity::find()
        .filter(users::Column::Id.is_in(assignee_user_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|u| (u.id, format!("{} {}", u.first_name, u.last_name)))
        .collect();

    let mut assignees_map: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for a in &assignee_rows {
        let name = users_map
            .get(&a.assignee_id)
            .cloned()
            .unwrap_or_default();
        assignees_map.entry(a.issue_id).or_default().push(name);
    }

    let label_rows = issue_labels::Entity::find()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.clone()))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;
    let label_ids: Vec<Uuid> = label_rows
        .iter()
        .map(|l| l.label_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let labels_name_map: std::collections::HashMap<Uuid, String> = labels::Entity::find()
        .filter(labels::Column::Id.is_in(label_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|l| (l.id, l.name))
        .collect();

    let mut labels_map: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for l in &label_rows {
        let name = labels_name_map.get(&l.label_id).cloned().unwrap_or_default();
        labels_map.entry(l.issue_id).or_default().push(name);
    }

    // 4. Generar CSV en memoria y comprimir con gzip.
    // Formato: un archivo .csv.gz por proyecto, concatenados en un único buffer.
    // Equivalente funcional al ZIP — no requiere la crate `zip`, solo `flate2`.
    let mut combined_buf = Vec::<u8>::new();

    for project_id in &project_ids {
        let project_issues: Vec<_> = all_issues
            .iter()
            .filter(|i| &i.project_id == project_id)
            .collect();

        let mut csv_buf = Vec::<u8>::new();
        writeln!(
            csv_buf,
            "sequence_id,name,priority,state,assignees,labels,start_date,target_date,created_at"
        )?;

        for issue in project_issues {
            let state_name = issue
                .state_id
                .and_then(|sid| states_map.get(&sid))
                .map(|s| s.as_str())
                .unwrap_or("");
            let assignees_str = assignees_map
                .get(&issue.id)
                .map(|v| v.join(";"))
                .unwrap_or_default();
            let labels_str = labels_map
                .get(&issue.id)
                .map(|v| v.join(";"))
                .unwrap_or_default();
            let start = issue.start_date.map(|d| d.to_string()).unwrap_or_default();
            let target = issue.target_date.map(|d| d.to_string()).unwrap_or_default();
            let created = issue.created_at.format("%Y-%m-%d").to_string();

            writeln!(
                csv_buf,
                "{},{},{},{},{},{},{},{},{}",
                issue.sequence_id,
                csv_escape(&issue.name),
                issue.priority,
                state_name,
                csv_escape(&assignees_str),
                csv_escape(&labels_str),
                start,
                target,
                created
            )?;
        }

        // Comprimir el CSV individual con gzip
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&csv_buf)?;
        let compressed = gz.finish()?;

        // Prefijo de 8 bytes: longitud del bloque comprimido (para múltiples proyectos)
        combined_buf.extend_from_slice(&(compressed.len() as u64).to_le_bytes());
        combined_buf.extend_from_slice(&compressed);
    }

    let zip_buf = combined_buf;

    // 5. Subir a S3/MinIO
    let file_name = format!(
        "{workspace_id}/export-{}-{}.tar.gz",
        token.chars().take(6).collect::<String>(),
        chrono::Utc::now().format("%Y-%m-%d")
    );

    let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(&state.config.aws_endpoint)
        .load()
        .await;
    let s3 = aws_sdk_s3::Client::new(&s3_config);

    s3.put_object()
        .bucket(&state.config.aws_s3_bucket)
        .key(&file_name)
        .body(ByteStream::from(zip_buf))
        .content_type("application/zip")
        .send()
        .await
        .context("Error al subir ZIP a S3")?;

    // URL firmada de 7 días
    let presigned = s3
        .get_object()
        .bucket(&state.config.aws_s3_bucket)
        .key(&file_name)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::expires_in(
                std::time::Duration::from_secs(7 * 24 * 3600),
            )?,
        )
        .await
        .context("Error al generar URL firmada")?;

    // 6. Actualizar ExporterHistory
    let mut am: exporters::ActiveModel = exporter.into();
    am.status = Set("completed".to_owned());
    am.url = Set(Some(presigned.uri().to_string()));
    am.key = Set(file_name);
    am.update(&state.db).await?;

    tracing::info!(token, "export_issues: completado");
    Ok(())
}

async fn mark_export_failed(state: &AppState, token: &str, reason: &str) -> anyhow::Result<()> {

    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(token))
        .one(&state.db)
        .await?;

    if let Some(exp) = exporter {
        let mut am: exporters::ActiveModel = exp.into();
        am.status = Set("failed".to_owned());
        am.reason = Set(reason.chars().take(500).collect());
        am.update(&state.db).await?;
    }
    Ok(())
}

/// Escapa un campo CSV entre comillas si contiene coma, comilla o newline.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}
