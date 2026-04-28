// src/jobs/export.rs
//! Job: exporting issues from a workspace to the requested format (csv/json/xlsx),
//! packaged in a real ZIP and uploaded to S3.
//!
//! Parity with `apps/api/plane/bgtasks/export_task.py::issue_export_task`:
//!   1. Load ExporterHistory by token → mark as "processing".
//!   2. Query issues from the indicated projects (filtered by active membership,
//!      unarchived project, non-soft-deleted and unarchived issue).
//!   3. Serialize each issue to a flat record.
//!   4. According to `multiple`:
//!        - `true`  → one file per project (`{slug}-{project_id}.{ext}`).
//!        - `false` → a single consolidated file (`{slug}-{workspace_id}.{ext}`).
//!          Format of each file according to `exporter.provider`:
//!        - `csv`  → CSV with prettified headers (Django's csv.DictWriter).
//!        - `json` → JSON indent=2.
//!        - `xlsx` → Excel via `rust_xlsxwriter` (mirroring `openpyxl`).
//!   5. Package all files in a real ZIP (deflate) — the same
//!      container that Django produces via `zipfile.ZipFile(..., ZIP_DEFLATED)`.
//!   6. Upload to S3/MinIO as `.zip` with `Content-Type: application/zip` and
//!      persist the signed URL (7 days) in ExporterHistory.
//!
//! HISTORICAL BUG (pre-fix): the worker generated a custom buffer with prefixes
//! of 8 bytes + concatenated gzip blocks, uploaded it as `.tar.gz`, and ignored
//! the provider — the user "downloaded an unreadable compressed file". See todo.md.
//!
//! TODO(full-parity): the current serializer exports a subset of fields
//! (9 columns). Django exports ~25 (parent, identifier, cycles, modules,
//! comments, relations, subscribers, estimate, sub_issues_count, etc.).
//! Expand when `IssueExportSerializer` is fully migrated.

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
        csv_sanitize::sanitize_csv_cell,
        s3::{build_s3_client, build_s3_presign_client},
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

// ── Job payload ───────────────────────────────────────────────────────────────

/// Payload: unique token of the ExporterHistory to process + `multiple` flag.
///
/// Django parity (apps/api/plane/app/views/exporter/base.py:49-56):
/// `issue_export_task.delay(..., multiple=multiple, ...)`.
/// - `multiple=true`  → one file per project (see `export_task.py:204-210`).
/// - `multiple=false` → a single consolidated file with all workspace issues
///   (see `export_task.py:211-215`).
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
        // `{:?}` exposes the complete `.context()` chain — `%e` hides
        // the root cause (e.g. real S3 SDK error).
        tracing::error!(
            token = %job.exporter_token,
            error = ?e,
            "export_issues: job failed"
        );
        // Mark as failed in DB (best-effort — if this UPDATE also
        // fails, at least it remains in the log).
        let _ = mark_export_failed(&state, &job.exporter_token, &e.to_string()).await;
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

// ── Flow principal ────────────────────────────────────────────────────────────

async fn run_export(state: &AppState, token: &str, multiple: bool) -> anyhow::Result<()> {
    use anyhow::Context as _;

    tracing::debug!(token, multiple, "export_issues: iniciando job");

    // 1. Load ExporterHistory + mark as "processing"
    // Django parity (export_task.py:143-145).
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(token))
        .filter(exporters::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .context("ExporterHistory not found")?;

    {
        let mut am: exporters::ActiveModel = exporter.clone().into();
        am.status = Set("processing".to_owned());
        am.updated_at = Set(chrono::Utc::now().into());
        am.update(&state.db).await?;
    }

    let workspace_id = exporter.workspace_id;
    let provider = exporter.provider.clone();
    let project_ids: Vec<Uuid> = exporter.project.clone().unwrap_or_default();

    // Validate provider — defense in depth; the endpoint already validates it.
    if !matches!(provider.as_str(), "csv" | "xlsx" | "json") {
        anyhow::bail!("Invalid provider: '{provider}' (expected csv|xlsx|json)");
    }

    if project_ids.is_empty() {
        anyhow::bail!("No projects in the exporter");
    }

    // 2. Workspace slug (used in filenames and S3 key — Django
    //    receives it as a task argument, export_task.py:135).
    let workspace = workspaces::Entity::find_by_id(workspace_id)
        .one(&state.db)
        .await?
        .context("Workspace not found")?;
    let slug = workspace.slug;

    // 3. Query filtered issues (parity with export_task.py:148-190).
    //    Excluded archived and soft-deleted — already done via `.active()`.
    let all_issues = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::ProjectId.is_in(project_ids.clone()))
        .filter(issues::Column::ArchivedAt.is_null())
        .order_by_asc(issues::Column::SequenceId)
        .all(&state.db)
        .await?;

    // 4. Batch-fetch relations — avoids N+1 (parity with `prefetch_related`).
    let maps = fetch_related_maps(state, &all_issues).await?;

    // 4b. Batch-fetch projects (id → identifier + name) to build
    //     readable filenames. We diverge here from Django on purpose: the
    //     Python worker uses `{slug}-{project_id}` with the raw UUID
    //     (export_task.py:208), which produces indistinguishable files
    //     at first glance when exporting multiple projects from the same
    //     workspace. We map by identifier (e.g. "FRONT", "API") which is
    //     the unique short-code per workspace already shown in the UI.
    let project_info: std::collections::HashMap<Uuid, (String, String)> =
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids.clone()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|p| (p.id, (p.identifier, p.name)))
            .collect();

    // 5. Build list of (filename, bytes) according to `multiple` + provider.
    //    Django in export_task.py:203-215 constructs `files = [(name, content)]`
    //    and passes it to `create_zip_file`.
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    if multiple {
        // Protect against identifier collisions (theoretically impossible:
        // `identifier` is UNIQUE per workspace in the Plane model) and
        // against projects not present in `project_info` (race condition delete).
        // Fallback: project_id truncated to 8 chars.
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
                    // Project deleted between job enqueue and execution,
                    // or identifier/name that end up empty after
                    // sanitizing (purely non-ASCII characters): fallback to
                    // truncated UUID instead of failing the whole export.
                    project_id.simple().to_string().chars().take(8).collect()
                });
            // Filename per-project: `{label}-{random_uuid}`. UUID v4
            // guarantees uniqueness at individual export level (two consecutive
            // exports of the same project produce different names,
            // useful if the user downloads several ZIPs in the same session and
            // extracts them in the same folder). `unique_base_name` still
            // operates just in case.
            let base_name = unique_base_name(
                &format!("{label}-{uuid}", uuid = Uuid::new_v4()),
                &mut used_names,
            );

            let (filename, content) = encode_issues(&base_name, &project_issues, &maps, &provider)?;
            files.push((filename, content));
        }
    } else {
        // `multiple=false` → a single consolidated workspace file
        // (parity with export_task.py:211-215). We build the name from
        // the part of the slug after the first `-` (e.g. if slug is
        // `tenant-workspace`, we use `workspace`); if the slug has no
        // `-`, we use the full slug. A UUID v4 at the end gives
        // uniqueness between consecutive exports.
        let slug_tail = slug.split_once('-').map(|(_, r)| r).unwrap_or(&slug);
        let base_name = format!("{slug_tail}-{}", Uuid::new_v4());
        let refs: Vec<&issues::Model> = all_issues.iter().collect();
        let (filename, content) = encode_issues(&base_name, &refs, &maps, &provider)?;
        files.push((filename, content));
    }

    // 6. Package in real ZIP (deflate) — parity with `create_zip_file`.
    let zip_buf = build_zip(&files).context("Error generating ZIP")?;

    // 7. Upload to S3/MinIO
    // Key format mirroring Django (export_task.py:46):
    //   "{workspace_id}/export-{slug}-{token[:6]}-{YYYY-MM-DD}.zip"
    let file_name = format!(
        "{workspace_id}/export-{slug}-{}-{}.zip",
        token.chars().take(6).collect::<String>(),
        chrono::Utc::now().format("%Y-%m-%d")
    );

    // The canonical builder (utils/s3.rs) applies explicit credentials, region
    // with fallback to us-east-1, and `force_path_style=true` for MinIO.
    // `cron.rs::delete_old_s3_links` already uses this helper — same pattern.
    let s3 = build_s3_client(&state.config);

    s3.put_object()
        .bucket(&state.config.aws_s3_bucket)
        .key(&file_name)
        .body(ByteStream::from(zip_buf))
        // application/zip — parity with export_task.py:61,104.
        .content_type("application/zip")
        .send()
        .await
        .context("Error uploading ZIP to S3")?;

    // 7-day signed URL.
    // Django parity (export_task.py:65-79): with MinIO a **different**
    // client with public endpoint (derived from WEB_URL) is used for signing —
    // otherwise the URL points to the internal Docker hostname that the browser
    // does not resolve.
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
        .context("Error generating signed URL")?;

    // 8. Update ExporterHistory — status "completed" + url + key.
    // SeaORM does not do auto_now; we set updated_at explicitly.
    let mut am: exporters::ActiveModel = exporter.into();
    am.status = Set("completed".to_owned());
    am.url = Set(Some(presigned.uri().to_string()));
    am.key = Set(file_name);
    am.updated_at = Set(chrono::Utc::now().into());
    am.update(&state.db).await?;

    tracing::info!(token, provider = %provider, multiple, "export_issues: completed");
    Ok(())
}

// ── Helpers: relaciones ──────────────────────────────────────────────────────

/// Precalculated maps for serialization — all batch-loaded to avoid N+1.
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

/// Parity with `User.full_name` in Django (user with first name + last name).
/// If both are empty, return an empty string to avoid emitting a loose " ".
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

/// Builds the human-readable segment of the filename for a project.
/// We prefer `name` (e.g. `test2`, `web-platform`) which is what users
/// recognize in the UI. `identifier` (UPPER short-code like `TEST2`, `FRONT`)
/// remains only as a fallback for the rare case where `name` remains empty
/// after sanitizing (purely non-ASCII characters or empty string).
/// If BOTH are empty (theoretically impossible: both are NOT NULL in the
/// schema), the caller falls back to truncated UUID.
fn project_label(identifier: &str, name: &str) -> String {
    let name_s = sanitize_filename_segment(name);
    if !name_s.is_empty() {
        return name_s;
    }
    sanitize_filename_segment(identifier)
}

/// Sanitizes a filename segment: collapses spaces/unsafe characters
/// to `-`, limits length, and avoids classic filename issues on
/// Windows/macOS/Linux (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`).
/// No lowercasing is done because Plane identifiers are UPPER by convention
/// and preserving the original case improves readability.
fn sanitize_filename_segment(s: &str) -> String {
    const MAX_LEN: usize = 64; // Defensive: some FS truncate at 255; we leave a margin.

    let mut out = String::with_capacity(s.len());
    let mut last_was_dash = false;
    for ch in s.chars() {
        let safe = match ch {
            // Allowed as is: ASCII alphanumeric + `_`.
            c if c.is_ascii_alphanumeric() || c == '_' => {
                out.push(c);
                last_was_dash = false;
                continue;
            }
            // Anything else (spaces, dots, slashes, unicode, etc.)
            // collapses to a single `-`.
            _ => '-',
        };
        if !last_was_dash && !out.is_empty() {
            out.push(safe);
            last_was_dash = true;
        }
    }
    // Trim trailing dashes + max length.
    let trimmed = out.trim_matches('-').to_owned();
    trimmed.chars().take(MAX_LEN).collect()
}

/// Guarantees uniqueness of `base_name` within the ZIP. If the name was already used
/// (degenerate case: two projects with the same sanitized identifier),
/// appends `-2`, `-3`, etc. until a free one is found.
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

// ── Helpers: serialization by provider ──────────────────────────────────────

/// Flattened row ready for any formatter. Ordered like the Django
/// serializer (IssueExportSerializer.Meta.fields — supported subset).
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

/// Headers in the same order as values are written.
/// Note: prettified (`snake_case → Title Case`) for parity with
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

/// Encode: QuerySet → (filename.ext, bytes), routed by provider.
/// Parity with `DataExporter.export(filename, queryset)` — filename includes
/// the extension and content are bytes ready for the ZIP.
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
        other => anyhow::bail!("Invalid provider: '{other}'"),
    }
}

/// CSV with RFC 4180 escaping via the `csv` crate + CSV injection sanitization.
/// Parity with `CSVFormatter.encode` using `csv.writer` + `sanitize_csv_row`.
fn encode_csv(rows: &[IssueRow<'_>]) -> anyhow::Result<Vec<u8>> {
    let mut wtr = csv::Writer::from_writer(Vec::<u8>::new());
    wtr.write_record(HEADERS)?;
    for r in rows {
        wtr.write_record([
            &r.sequence_id.to_string(),
            &sanitize_csv_cell(r.name),
            &sanitize_csv_cell(&r.state),
            &sanitize_csv_cell(r.priority),
            // `XLSXFormatter` uses list_joiner=", " and `CSVFormatter` flattens lists
            // with json.dumps — we diverge slightly here and use "; " as in
            // the previous worker for frontend stability. See full-parity TODO
            // at the top of the module.
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

/// JSON indent=2. Parity with `JSONFormatter.encode(data, indent=2)`.
/// We keep snake_case (JSONFormatter does not prettify headers) — same
/// keys as `JSON_KEYS` above; repeated here because the `json!` macro only
/// admits literals or identifiers in scope as keys.
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

/// XLSX via `rust_xlsxwriter`. Parity with `XLSXFormatter.encode` (openpyxl):
/// prettified headers, lists with join=", ".
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
            .write_string(row, 4, r.assignees.join(", "))
            .map_err(|e| anyhow::anyhow!("xlsx write assignees: {e}"))?;
        sheet
            .write_string(row, 5, r.labels.join(", "))
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

// ── Helpers: real ZIP ────────────────────────────────────────────────────────

/// Packages `(filename, bytes)` into a standard ZIP (deflate).
/// Parity with `create_zip_file` (export_task.py:28-38).
fn build_zip(files: &[(String, Vec<u8>)]) -> anyhow::Result<Vec<u8>> {
    let buf = Vec::<u8>::new();
    let cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(cursor);

    // `SimpleFileOptions` avoids having to type the `FileOptions` generic
    // (API change in zip 2.x to support extended attributes).
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (name, bytes) in files {
        zip.start_file(name, opts)?;
        zip.write_all(bytes)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

// ── Mark job as failed ──────────────────────────────────────────────────

async fn mark_export_failed(state: &AppState, token: &str, reason: &str) -> anyhow::Result<()> {
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(token))
        .one(&state.db)
        .await?;

    if let Some(exp) = exporter {
        let mut am: exporters::ActiveModel = exp.into();
        am.status = Set("failed".to_owned());
        // `reason` is TEXT but we truncate in case the upstream explodes with a
        // giant error (e.g. stack traces in S3 errors). 500 chars cover
        // 99% of useful cases.
        am.reason = Set(reason.chars().take(500).collect());
        am.updated_at = Set(chrono::Utc::now().into());
        am.update(&state.db).await?;
    }
    Ok(())
}
