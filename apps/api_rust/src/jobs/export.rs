// src/jobs/export.rs
//! Job: exportación de issues de un workspace al formato pedido (csv/json/xlsx),
//! empaquetado en ZIP real y subido a S3.
//!
//! Paridad con `apps/api/plane/bgtasks/export_task.py::issue_export_task`:
//!   1. Cargar ExporterHistory por token → marcar "processing".
//!   2. Consultar issues de los proyectos indicados (filtradas por membresía
//!      activa, proyecto no archivado, issue no soft-deleted ni archivada).
//!   3. Serializar cada issue a un registro plano.
//!   4. Según `multiple`:
//!        - `true`  → un archivo por proyecto (`{slug}-{project_id}.{ext}`).
//!        - `false` → un único archivo consolidado (`{slug}-{workspace_id}.{ext}`).
//!      Formato de cada archivo según `exporter.provider`:
//!        - `csv`  → CSV con headers prettificados (csv.DictWriter de Django).
//!        - `json` → JSON indent=2.
//!        - `xlsx` → Excel vía `rust_xlsxwriter` (espejo de `openpyxl`).
//!   5. Empaquetar todos los archivos en un ZIP real (deflate) — el mismo
//!      contenedor que Django produce vía `zipfile.ZipFile(..., ZIP_DEFLATED)`.
//!   6. Subir a S3/MinIO como `.zip` con `Content-Type: application/zip` y
//!      persistir la URL firmada (7 días) en ExporterHistory.
//!
//! BUG HISTÓRICO (pre-fix): el worker generaba un buffer custom con prefijos
//! de 8 bytes + bloques gzip concatenados, lo subía como `.tar.gz`, e ignoraba
//! el provider — el usuario "bajaba un comprimido" ilegible. Ver todo.md.
//!
//! TODO(paridad-full): el serializer actual exporta un subset de campos
//! (9 columnas). Django exporta ~25 (parent, identifier, cycles, modules,
//! comments, relations, subscribers, estimate, sub_issues_count, etc.).
//! Ampliar cuando se migre `IssueExportSerializer` completo.

use std::io::{Cursor, Write};

use apalis::prelude::*;
use aws_sdk_s3::primitives::ByteStream;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

use crate::{
    entities::{
        exporters, issue_assignees, issue_labels, issues, labels, projects, states, users,
        workspaces,
    },
    utils::{
        s3::{build_s3_client, build_s3_presign_client},
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

// ── Job payload ───────────────────────────────────────────────────────────────

/// Payload: token único del ExporterHistory a procesar + flag `multiple`.
///
/// Paridad Django (apps/api/plane/app/views/exporter/base.py:49-56):
/// `issue_export_task.delay(..., multiple=multiple, ...)`.
/// - `multiple=true`  → un archivo por proyecto (ver `export_task.py:204-210`).
/// - `multiple=false` → un único archivo consolidado con todas las issues del
///   workspace (ver `export_task.py:211-215`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportIssuesJob {
    pub exporter_token: String,
    #[serde(default)]
    pub multiple: bool,
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn handle_export_issues(job: ExportIssuesJob, ctx: Data<AppState>) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    if let Err(e) = run_export(&state, &job.exporter_token, job.multiple).await {
        // `{:?}` expone la cadena completa de `.context()` — `%e` oculta
        // la causa raíz (ej. error real del SDK de S3).
        tracing::error!(
            token = %job.exporter_token,
            error = ?e,
            "export_issues: job falló"
        );
        // Marcar como fallido en DB (best-effort — si este UPDATE también
        // falla, al menos queda registrado en el log).
        let _ = mark_export_failed(&state, &job.exporter_token, &e.to_string()).await;
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

// ── Flow principal ────────────────────────────────────────────────────────────

async fn run_export(state: &AppState, token: &str, multiple: bool) -> anyhow::Result<()> {
    use anyhow::Context as _;

    tracing::debug!(token, multiple, "export_issues: iniciando job");

    // 1. Cargar ExporterHistory + marcar "processing"
    // Paridad Django (export_task.py:143-145).
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(token))
        .filter(exporters::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .context("ExporterHistory no encontrado")?;

    {
        let mut am: exporters::ActiveModel = exporter.clone().into();
        am.status = Set("processing".to_owned());
        am.updated_at = Set(chrono::Utc::now().into());
        am.update(&state.db).await?;
    }

    let workspace_id = exporter.workspace_id;
    let provider = exporter.provider.clone();
    let project_ids: Vec<Uuid> = exporter.project.clone().unwrap_or_default();

    // Validar provider — defensa en profundidad; el endpoint ya lo valida.
    if !matches!(provider.as_str(), "csv" | "xlsx" | "json") {
        anyhow::bail!("Provider inválido: '{provider}' (esperado csv|xlsx|json)");
    }

    if project_ids.is_empty() {
        anyhow::bail!("No hay proyectos en el exporter");
    }

    // 2. Slug del workspace (se usa en nombres de archivo y S3 key — Django
    //    lo recibe como argumento del task, export_task.py:135).
    let workspace = workspaces::Entity::find_by_id(workspace_id)
        .one(&state.db)
        .await?
        .context("Workspace no encontrado")?;
    let slug = workspace.slug;

    // 3. Consultar issues filtradas (paridad con export_task.py:148-190).
    //    Excluimos archivadas y soft-deleted — ya hecho vía `.active()`.
    let all_issues = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::ProjectId.is_in(project_ids.clone()))
        .filter(issues::Column::ArchivedAt.is_null())
        .order_by_asc(issues::Column::SequenceId)
        .all(&state.db)
        .await?;

    // 4. Batch-fetch de relaciones — evita N+1 (paridad con `prefetch_related`).
    let maps = fetch_related_maps(state, &all_issues).await?;

    // 4b. Batch-fetch de proyectos (id → identifier + name) para armar
    //     filenames legibles. Divergimos acá de Django a propósito: el
    //     worker Python usa `{slug}-{project_id}` con el UUID crudo
    //     (export_task.py:208), lo que produce archivos indistinguibles
    //     a simple vista cuando se exportan varios proyectos de un mismo
    //     workspace. Mapeamos por identifier (ej. "FRONT", "API") que es
    //     el short-code único por workspace que ya se muestra en la UI.
    let project_info: std::collections::HashMap<Uuid, (String, String)> =
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids.clone()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|p| (p.id, (p.identifier, p.name)))
            .collect();

    // 5. Armar lista de (filename, bytes) según `multiple` + provider.
    //    Django en export_task.py:203-215 construye `files = [(name, content)]`
    //    y se lo pasa a `create_zip_file`.
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    if multiple {
        // Proteger contra colisiones de identifier (teóricamente imposible:
        // `identifier` es UNIQUE por workspace en el modelo de Plane) y
        // contra proyectos que no vengan en `project_info` (borrado en
        // carrera). Fallback: project_id truncado a 8 chars.
        let mut used_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for project_id in &project_ids {
            let project_issues: Vec<&issues::Model> = all_issues
                .iter()
                .filter(|i| &i.project_id == project_id)
                .collect();

            let label = project_info
                .get(project_id)
                .map(|(ident, name)| project_label(ident, name))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    // Proyecto borrado entre el enqueue del job y su
                    // ejecución, o identifier/name que quedan vacíos tras
                    // sanitizar (puros caracteres no-ASCII): degradamos al
                    // UUID truncado en vez de fallar todo el export.
                    project_id.simple().to_string().chars().take(8).collect()
                });
            // Filename per-project: `{label}-{random_uuid}`. El UUID v4
            // garantiza unicidad a nivel de export individual (dos exports
            // consecutivos del mismo proyecto producen nombres distintos,
            // útil si el usuario baja varios ZIPs en la misma sesión y los
            // extrae en la misma carpeta). `unique_base_name` sigue
            // operando por si acaso.
            let base_name = unique_base_name(
                &format!("{label}-{uuid}", uuid = Uuid::new_v4()),
                &mut used_names,
            );

            let (filename, content) = encode_issues(&base_name, &project_issues, &maps, &provider)?;
            files.push((filename, content));
        }
    } else {
        // `multiple=false` → un único archivo consolidado del workspace
        // (paridad con export_task.py:211-215). Construimos el nombre a
        // partir de la parte del slug posterior al primer `-` (ej. si el
        // slug es `tenant-workspace`, usamos `workspace`); si el slug no
        // contiene `-`, usamos el slug completo. Un UUID v4 al final da
        // unicidad entre exports consecutivos.
        let slug_tail = slug.split_once('-').map(|(_, r)| r).unwrap_or(&slug);
        let base_name = format!("{slug_tail}-{}", Uuid::new_v4());
        let refs: Vec<&issues::Model> = all_issues.iter().collect();
        let (filename, content) = encode_issues(&base_name, &refs, &maps, &provider)?;
        files.push((filename, content));
    }

    // 6. Empaquetar en ZIP real (deflate) — paridad con `create_zip_file`.
    let zip_buf = build_zip(&files).context("Error al generar ZIP")?;

    // 7. Subir a S3/MinIO
    // Formato de key espejo de Django (export_task.py:46):
    //   "{workspace_id}/export-{slug}-{token[:6]}-{YYYY-MM-DD}.zip"
    let file_name = format!(
        "{workspace_id}/export-{slug}-{}-{}.zip",
        token.chars().take(6).collect::<String>(),
        chrono::Utc::now().format("%Y-%m-%d")
    );

    // El builder canónico (utils/s3.rs) aplica credentials explícitas, región
    // con fallback a us-east-1, y `force_path_style=true` para MinIO.
    // `cron.rs::delete_old_s3_links` ya usa este helper — mismo patrón.
    let s3 = build_s3_client(&state.config);

    s3.put_object()
        .bucket(&state.config.aws_s3_bucket)
        .key(&file_name)
        .body(ByteStream::from(zip_buf))
        // application/zip — paridad con export_task.py:61,104.
        .content_type("application/zip")
        .send()
        .await
        .context("Error al subir ZIP a S3")?;

    // URL firmada de 7 días.
    // Paridad Django (export_task.py:65-79): con MinIO se usa un cliente
    // **distinto** con endpoint público (derivado de WEB_URL) para firmar —
    // de lo contrario la URL apunta al hostname Docker interno que el browser
    // no resuelve.
    let presign_s3 = build_s3_presign_client(&state.config);
    let presigned = presign_s3
        .get_object()
        .bucket(&state.config.aws_s3_bucket)
        .key(&file_name)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::expires_in(std::time::Duration::from_secs(
                7 * 24 * 3600,
            ))?,
        )
        .await
        .context("Error al generar URL firmada")?;

    // 8. Actualizar ExporterHistory — status "completed" + url + key.
    // SeaORM no hace auto_now; seteamos updated_at explícitamente.
    let mut am: exporters::ActiveModel = exporter.into();
    am.status = Set("completed".to_owned());
    am.url = Set(Some(presigned.uri().to_string()));
    am.key = Set(file_name);
    am.updated_at = Set(chrono::Utc::now().into());
    am.update(&state.db).await?;

    tracing::info!(token, provider = %provider, multiple, "export_issues: completado");
    Ok(())
}

// ── Helpers: relaciones ──────────────────────────────────────────────────────

/// Maps precalculados para serialización — todos batch-loaded para evitar N+1.
struct RelationMaps {
    states: std::collections::HashMap<Uuid, String>,
    assignees: std::collections::HashMap<Uuid, Vec<String>>,
    labels: std::collections::HashMap<Uuid, Vec<String>>,
}

async fn fetch_related_maps(
    state: &AppState,
    all_issues: &[issues::Model],
) -> anyhow::Result<RelationMaps> {
    let issue_ids: Vec<Uuid> = all_issues.iter().map(|i| i.id).collect();

    // States
    let state_ids: Vec<Uuid> = all_issues
        .iter()
        .filter_map(|i| i.state_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let states_map: std::collections::HashMap<Uuid, String> = if state_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        states::Entity::find()
            .filter(states::Column::Id.is_in(state_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s.name))
            .collect()
    };

    // Assignees → users
    let assignee_rows = if issue_ids.is_empty() {
        Vec::new()
    } else {
        issue_assignees::Entity::find()
            .filter(issue_assignees::Column::IssueId.is_in(issue_ids.clone()))
            .filter(issue_assignees::Column::DeletedAt.is_null())
            .all(&state.db)
            .await?
    };
    let assignee_user_ids: Vec<Uuid> = assignee_rows
        .iter()
        .map(|a| a.assignee_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let users_map: std::collections::HashMap<Uuid, String> = if assignee_user_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(assignee_user_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| (u.id, format_user_name(&u.first_name, &u.last_name)))
            .collect()
    };
    let mut assignees_map: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for a in &assignee_rows {
        let name = users_map.get(&a.assignee_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            assignees_map.entry(a.issue_id).or_default().push(name);
        }
    }

    // Labels
    let label_rows = if issue_ids.is_empty() {
        Vec::new()
    } else {
        issue_labels::Entity::find()
            .filter(issue_labels::Column::IssueId.is_in(issue_ids.clone()))
            .filter(issue_labels::Column::DeletedAt.is_null())
            .all(&state.db)
            .await?
    };
    let label_ids: Vec<Uuid> = label_rows
        .iter()
        .map(|l| l.label_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let labels_name_map: std::collections::HashMap<Uuid, String> = if label_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        labels::Entity::find()
            .filter(labels::Column::Id.is_in(label_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|l| (l.id, l.name))
            .collect()
    };
    let mut labels_map: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for l in &label_rows {
        let name = labels_name_map.get(&l.label_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            labels_map.entry(l.issue_id).or_default().push(name);
        }
    }

    Ok(RelationMaps {
        states: states_map,
        assignees: assignees_map,
        labels: labels_map,
    })
}

/// Paridad con `User.full_name` en Django (usuario con nombre + apellido).
/// Si ambos están vacíos, devolvemos cadena vacía para no emitir " " suelto.
fn format_user_name(first: &str, last: &str) -> String {
    let f = first.trim();
    let l = last.trim();
    match (f.is_empty(), l.is_empty()) {
        (true, true) => String::new(),
        (true, false) => l.to_owned(),
        (false, true) => f.to_owned(),
        (false, false) => format!("{f} {l}"),
    }
}

/// Construye el segmento humano-legible del filename para un proyecto.
/// Preferimos `name` (ej. `test2`, `web-platform`) que es lo que los usuarios
/// reconocen en la UI. `identifier` (código corto UPPER tipo `TEST2`, `FRONT`)
/// queda solo como fallback para el caso raro en que el `name` quede vacío
/// tras sanitizar (puros caracteres no-ASCII o string vacío).
/// Si AMBOS están vacíos (teóricamente imposible: ambos son NOT NULL en el
/// esquema), el caller cae al fallback UUID-truncado.
fn project_label(identifier: &str, name: &str) -> String {
    let name_s = sanitize_filename_segment(name);
    if !name_s.is_empty() {
        return name_s;
    }
    sanitize_filename_segment(identifier)
}

/// Sanitiza un segmento de filename: colapsa espacios/caracteres no seguros
/// a `-`, limita longitud y evita los problemas clásicos de filenames en
/// Windows/macOS/Linux (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`).
/// No hace lowercasing porque los identifiers de Plane son UPPER por convención
/// y preservar el case original mejora la legibilidad.
fn sanitize_filename_segment(s: &str) -> String {
    const MAX_LEN: usize = 64; // Defensivo: algunos FS truncan a 255; dejamos margen.

    let mut out = String::with_capacity(s.len());
    let mut last_was_dash = false;
    for ch in s.chars() {
        let safe = match ch {
            // Permitidos tal cual: alfanuméricos ASCII + `_`.
            c if c.is_ascii_alphanumeric() || c == '_' => {
                out.push(c);
                last_was_dash = false;
                continue;
            }
            // Cualquier otra cosa (espacios, puntos, slashes, unicode, etc.)
            // colapsa a un solo `-`.
            _ => '-',
        };
        if !last_was_dash && !out.is_empty() {
            out.push(safe);
            last_was_dash = true;
        }
    }
    // Trim de dashes trailing + longitud máxima.
    let trimmed = out.trim_matches('-').to_owned();
    trimmed.chars().take(MAX_LEN).collect()
}

/// Garantiza unicidad de `base_name` dentro del ZIP. Si el nombre ya se usó
/// (caso degenerado: dos proyectos con el mismo identifier sanitizado),
/// apendea `-2`, `-3`, etc. hasta encontrar uno libre.
fn unique_base_name(
    candidate: &str,
    used: &mut std::collections::HashSet<String>,
) -> String {
    if used.insert(candidate.to_owned()) {
        return candidate.to_owned();
    }
    let mut n: u32 = 2;
    loop {
        let next = format!("{candidate}-{n}");
        if used.insert(next.clone()) {
            return next;
        }
        n += 1;
    }
}

// ── Helpers: serialización por provider ──────────────────────────────────────

/// Row aplanado listo para cualquier formatter. Ordenado como el serializer
/// de Django (IssueExportSerializer.Meta.fields — subset soportado).
struct IssueRow<'a> {
    sequence_id: i32,
    name: &'a str,
    state: String,
    priority: &'a str,
    assignees: Vec<String>,
    labels: Vec<String>,
    start_date: String,
    target_date: String,
    created_at: String,
}

/// Headers en el mismo orden que se escriben los valores.
/// Nota: prettificados (`snake_case → Title Case`) para paridad con
/// `CSVFormatter.prettify_headers=True` y `XLSXFormatter.prettify_headers=True`.
const HEADERS: &[&str] = &[
    "Sequence Id",
    "Name",
    "State",
    "Priority",
    "Assignees",
    "Labels",
    "Start Date",
    "Target Date",
    "Created At",
];

fn build_rows<'a>(
    issues_ref: &[&'a issues::Model],
    maps: &RelationMaps,
) -> Vec<IssueRow<'a>> {
    issues_ref
        .iter()
        .map(|issue| {
            let state = issue
                .state_id
                .and_then(|sid| maps.states.get(&sid))
                .cloned()
                .unwrap_or_default();
            IssueRow {
                sequence_id: issue.sequence_id,
                name: &issue.name,
                state,
                priority: &issue.priority,
                assignees: maps.assignees.get(&issue.id).cloned().unwrap_or_default(),
                labels: maps.labels.get(&issue.id).cloned().unwrap_or_default(),
                start_date: issue.start_date.map(|d| d.to_string()).unwrap_or_default(),
                target_date: issue.target_date.map(|d| d.to_string()).unwrap_or_default(),
                created_at: issue.created_at.format("%Y-%m-%d").to_string(),
            }
        })
        .collect()
}

/// Encode: QuerySet → (filename.ext, bytes), ruteado por provider.
/// Paridad con `DataExporter.export(filename, queryset)` — el filename incluye
/// la extensión y el content son bytes listos para el ZIP.
fn encode_issues(
    base_name: &str,
    issues_ref: &[&issues::Model],
    maps: &RelationMaps,
    provider: &str,
) -> anyhow::Result<(String, Vec<u8>)> {
    let rows = build_rows(issues_ref, maps);
    match provider {
        "csv" => Ok((format!("{base_name}.csv"), encode_csv(&rows)?)),
        "json" => Ok((format!("{base_name}.json"), encode_json(&rows)?)),
        "xlsx" => Ok((format!("{base_name}.xlsx"), encode_xlsx(&rows)?)),
        other => anyhow::bail!("Provider inválido: '{other}'"),
    }
}

/// CSV con escape RFC 4180 vía la crate `csv` + sanitización de CSV injection.
/// Paridad con `CSVFormatter.encode` usando `csv.writer` + `sanitize_csv_row`.
fn encode_csv(rows: &[IssueRow<'_>]) -> anyhow::Result<Vec<u8>> {
    let mut wtr = csv::Writer::from_writer(Vec::<u8>::new());
    wtr.write_record(HEADERS)?;
    for r in rows {
        wtr.write_record([
            &r.sequence_id.to_string(),
            &sanitize_csv_cell(r.name),
            &sanitize_csv_cell(&r.state),
            &sanitize_csv_cell(r.priority),
            // `XLSXFormatter` usa list_joiner=", " y `CSVFormatter` aplana listas
            // con json.dumps — divergimos levemente acá y usamos "; " como en
            // el worker previo para estabilidad hacia el frontend. Ver TODO
            // de paridad-full al tope del módulo.
            &sanitize_csv_cell(&r.assignees.join("; ")),
            &sanitize_csv_cell(&r.labels.join("; ")),
            &r.start_date,
            &r.target_date,
            &r.created_at,
        ])?;
    }
    let buf = wtr.into_inner()?;
    Ok(buf)
}

/// Sanitización de CSV injection (paridad con `apps/api/plane/utils/csv_utils.py
/// ::sanitize_csv_value`): si el valor empieza por `=`, `+`, `-`, `@`, `\t`, o
/// `\r`, se prefija con comilla simple para que Excel/LibreOffice no lo evalúen
/// como fórmula.
fn sanitize_csv_cell(value: &str) -> String {
    // Django sanitiza considerando el primer char — replicamos el mismo check.
    if let Some(first) = value.chars().next() {
        if matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r') {
            return format!("'{value}");
        }
    }
    value.to_owned()
}

/// JSON indent=2. Paridad con `JSONFormatter.encode(data, indent=2)`.
/// Mantenemos snake_case (JSONFormatter no prettifica headers) — las mismas
/// keys que `JSON_KEYS` arriba; se repiten acá porque el macro `json!` sólo
/// admite literales o identificadores en scope como keys.
fn encode_json(rows: &[IssueRow<'_>]) -> anyhow::Result<Vec<u8>> {
    use serde_json::{json, Value};

    let arr: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "sequence_id": r.sequence_id,
                "name": r.name,
                "state_name": r.state,
                "priority": r.priority,
                "assignees": r.assignees,
                "labels": r.labels,
                "start_date": r.start_date,
                "target_date": r.target_date,
                "created_at": r.created_at,
            })
        })
        .collect();
    let s = serde_json::to_string_pretty(&arr)?;
    Ok(s.into_bytes())
}

/// XLSX vía `rust_xlsxwriter`. Paridad con `XLSXFormatter.encode` (openpyxl):
/// headers prettificados, listas con join=", ".
fn encode_xlsx(rows: &[IssueRow<'_>]) -> anyhow::Result<Vec<u8>> {
    use rust_xlsxwriter::Workbook;

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    // Header row
    for (col, h) in HEADERS.iter().enumerate() {
        sheet
            .write_string(0, col as u16, *h)
            .map_err(|e| anyhow::anyhow!("xlsx write header: {e}"))?;
    }

    // Data rows
    for (i, r) in rows.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet
            .write_number(row, 0, r.sequence_id as f64)
            .map_err(|e| anyhow::anyhow!("xlsx write sequence_id: {e}"))?;
        sheet
            .write_string(row, 1, r.name)
            .map_err(|e| anyhow::anyhow!("xlsx write name: {e}"))?;
        sheet
            .write_string(row, 2, &r.state)
            .map_err(|e| anyhow::anyhow!("xlsx write state: {e}"))?;
        sheet
            .write_string(row, 3, r.priority)
            .map_err(|e| anyhow::anyhow!("xlsx write priority: {e}"))?;
        sheet
            .write_string(row, 4, &r.assignees.join(", "))
            .map_err(|e| anyhow::anyhow!("xlsx write assignees: {e}"))?;
        sheet
            .write_string(row, 5, &r.labels.join(", "))
            .map_err(|e| anyhow::anyhow!("xlsx write labels: {e}"))?;
        sheet
            .write_string(row, 6, &r.start_date)
            .map_err(|e| anyhow::anyhow!("xlsx write start_date: {e}"))?;
        sheet
            .write_string(row, 7, &r.target_date)
            .map_err(|e| anyhow::anyhow!("xlsx write target_date: {e}"))?;
        sheet
            .write_string(row, 8, &r.created_at)
            .map_err(|e| anyhow::anyhow!("xlsx write created_at: {e}"))?;
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| anyhow::anyhow!("xlsx save_to_buffer: {e}"))?;
    Ok(bytes)
}

// ── Helpers: ZIP real ────────────────────────────────────────────────────────

/// Empaqueta `(filename, bytes)` en un ZIP estándar (deflate).
/// Paridad con `create_zip_file` (export_task.py:28-38).
fn build_zip(files: &[(String, Vec<u8>)]) -> anyhow::Result<Vec<u8>> {
    let buf = Vec::<u8>::new();
    let cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(cursor);

    // `SimpleFileOptions` evita tener que tipar el generic de `FileOptions`
    // (cambio de la API en zip 2.x para soportar extended attributes).
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (name, bytes) in files {
        zip.start_file(name, opts)?;
        zip.write_all(bytes)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

// ── Marcar job como fallido ──────────────────────────────────────────────────

async fn mark_export_failed(state: &AppState, token: &str, reason: &str) -> anyhow::Result<()> {
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(token))
        .one(&state.db)
        .await?;

    if let Some(exp) = exporter {
        let mut am: exporters::ActiveModel = exp.into();
        am.status = Set("failed".to_owned());
        // `reason` es TEXT pero truncamos por si el upstream explota con un
        // error gigante (ej. stack traces en errores de S3). 500 chars cubren
        // el 99% de casos útiles.
        am.reason = Set(reason.chars().take(500).collect());
        am.updated_at = Set(chrono::Utc::now().into());
        am.update(&state.db).await?;
    }
    Ok(())
}
