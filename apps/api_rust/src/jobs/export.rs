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
//! Full parity with Django `IssueExportSerializer` (27 columns).

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
        cycle_issues, cycles, estimate_points, exporters, issue_assignees, issue_comments,
        issue_labels, issue_links, issue_relations, issue_subscribers, issues, labels, module_issues,
        modules, projects, states, users, workspaces,
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

    // 4. Batch-fetch all relations — avoids N+1 (parity with `prefetch_related`).
    //    Projects are now included inside RelationMaps.maps.projects to avoid
    //    a second query. Filename labels are read from maps.projects.
    let maps = fetch_related_maps(state, &all_issues).await?;

    // 5. Build list of (filename, bytes) according to `multiple` + provider.
    //    Django in export_task.py:203-215 constructs `files = [(name, content)]`
    //    and passes it to `create_zip_file`.
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    if multiple {
        // Protect against identifier collisions (theoretically impossible:
        // `identifier` is UNIQUE per workspace in the Plane model) and
        // against projects not present in `maps.projects` (race condition delete).
        // Fallback: project_id truncated to 8 chars.
        let mut used_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for project_id in &project_ids {
            let project_issues: Vec<&issues::Model> = all_issues
                .iter()
                .filter(|i| &i.project_id == project_id)
                .collect();

            let label = maps.projects
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

/// Link entry for export: {url, title}.
#[derive(Clone)]
struct ExportLink {
    url: String,
    title: String,
}

/// Relation entry for export: {relation_type, issue_identifier, direction}.
#[derive(Clone)]
struct ExportRelation {
    relation_type: String,
    issue_identifier: String,
    direction: &'static str,
}

/// Comment entry for export: {comment_stripped, created_by, created_at}.
#[derive(Clone)]
struct ExportComment {
    comment: String,
    created_by: String,
    created_at: String,
}

/// Precalculated maps for serialization — all batch-loaded to avoid N+1.
/// Parity with Django `IssueExportSerializer` (27 columns).
struct RelationMaps {
    // project_id → (identifier, name)
    projects: std::collections::HashMap<Uuid, (String, String)>,
    // state_id → state name
    states: std::collections::HashMap<Uuid, String>,
    // issue_id → list of assignee full names
    assignees: std::collections::HashMap<Uuid, Vec<String>>,
    // issue_id → list of subscriber full names
    subscribers: std::collections::HashMap<Uuid, Vec<String>>,
    // user_id → full name (covers created_by and comment actors)
    users: std::collections::HashMap<Uuid, String>,
    // issue_id → list of label names
    labels: std::collections::HashMap<Uuid, Vec<String>>,
    // issue_id → list of cycle names
    cycles: std::collections::HashMap<Uuid, Vec<String>>,
    // issue_id → list of module names
    modules: std::collections::HashMap<Uuid, Vec<String>>,
    // estimate_point_id → value string
    estimates: std::collections::HashMap<Uuid, String>,
    // issue_id → list of ExportLink
    links: std::collections::HashMap<Uuid, Vec<ExportLink>>,
    // issue_id → list of ExportRelation
    relations: std::collections::HashMap<Uuid, Vec<ExportRelation>>,
    // issue_id → list of ExportComment
    comments: std::collections::HashMap<Uuid, Vec<ExportComment>>,
    // issue_id → sub_issues_count
    sub_issues_count: std::collections::HashMap<Uuid, i64>,
    // issue_id → attachment_count (approximated: 0 without dedicated entity)
    attachment_count: std::collections::HashMap<Uuid, i64>,
    // issue_id → link_count
    link_count: std::collections::HashMap<Uuid, i64>,
    // issue_id → "PROJ-123" parent identifier
    parent_identifier: std::collections::HashMap<Uuid, String>,
}

async fn fetch_related_maps(
    state: &AppState,
    all_issues: &[issues::Model],
) -> anyhow::Result<RelationMaps> {
    use std::collections::HashMap;

    let issue_ids: Vec<Uuid> = all_issues.iter().map(|i| i.id).collect();

    // Projects
    let project_ids: Vec<Uuid> = all_issues
        .iter()
        .map(|i| i.project_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let projects_map: HashMap<Uuid, (String, String)> = projects::Entity::find()
        .filter(projects::Column::Id.is_in(project_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|p| (p.id, (p.identifier, p.name)))
        .collect();

    // States
    let state_ids: Vec<Uuid> = all_issues
        .iter()
        .filter_map(|i| i.state_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let states_map: HashMap<Uuid, String> = if state_ids.is_empty() {
        HashMap::new()
    } else {
        states::Entity::find()
            .filter(states::Column::Id.is_in(state_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s.name))
            .collect()
    };

    if issue_ids.is_empty() {
        return Ok(RelationMaps {
            projects: projects_map,
            states: states_map,
            assignees: HashMap::new(),
            subscribers: HashMap::new(),
            users: HashMap::new(),
            labels: HashMap::new(),
            cycles: HashMap::new(),
            modules: HashMap::new(),
            estimates: HashMap::new(),
            links: HashMap::new(),
            relations: HashMap::new(),
            comments: HashMap::new(),
            sub_issues_count: HashMap::new(),
            attachment_count: HashMap::new(),
            link_count: HashMap::new(),
            parent_identifier: HashMap::new(),
        });
    }

    // Collect all user IDs we need (assignees, subscribers, comment actors, created_by)
    let mut all_user_ids: std::collections::HashSet<Uuid> = all_issues
        .iter()
        .filter_map(|i| i.created_by_id)
        .collect();

    // Assignees
    let assignee_rows = issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.clone()))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;
    for a in &assignee_rows {
        all_user_ids.insert(a.assignee_id);
    }

    // Subscribers
    let subscriber_rows = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.is_in(issue_ids.clone()))
        .all(&state.db)
        .await?;
    for s in &subscriber_rows {
        all_user_ids.insert(s.subscriber_id);
    }

    // Labels
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
    let labels_name_map: HashMap<Uuid, String> = if label_ids.is_empty() {
        HashMap::new()
    } else {
        labels::Entity::find()
            .filter(labels::Column::Id.is_in(label_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|l| (l.id, l.name))
            .collect()
    };
    let mut labels_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for l in &label_rows {
        let name = labels_name_map.get(&l.label_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            labels_map.entry(l.issue_id).or_default().push(name);
        }
    }

    // Cycles: cycle_issues → cycles
    let cycle_issue_rows = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.clone()))
        .all(&state.db)
        .await?;
    let cycle_ids: Vec<Uuid> = cycle_issue_rows
        .iter()
        .map(|ci| ci.cycle_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let cycles_name_map: HashMap<Uuid, String> = if cycle_ids.is_empty() {
        HashMap::new()
    } else {
        cycles::Entity::find()
            .active()
            .filter(cycles::Column::Id.is_in(cycle_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect()
    };
    let mut cycles_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for ci in &cycle_issue_rows {
        let name = cycles_name_map.get(&ci.cycle_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            cycles_map.entry(ci.issue_id).or_default().push(name);
        }
    }

    // Modules: module_issues → modules
    let module_issue_rows = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::IssueId.is_in(issue_ids.clone()))
        .all(&state.db)
        .await?;
    let module_ids: Vec<Uuid> = module_issue_rows
        .iter()
        .map(|mi| mi.module_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let modules_name_map: HashMap<Uuid, String> = if module_ids.is_empty() {
        HashMap::new()
    } else {
        modules::Entity::find()
            .active()
            .filter(modules::Column::Id.is_in(module_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|m| (m.id, m.name))
            .collect()
    };
    let mut modules_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for mi in &module_issue_rows {
        let name = modules_name_map.get(&mi.module_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            modules_map.entry(mi.issue_id).or_default().push(name);
        }
    }

    // Estimate points
    let ep_ids: Vec<Uuid> = all_issues
        .iter()
        .filter_map(|i| i.estimate_point_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let estimates_map: HashMap<Uuid, String> = if ep_ids.is_empty() {
        HashMap::new()
    } else {
        estimate_points::Entity::find()
            .active()
            .filter(estimate_points::Column::Id.is_in(ep_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|ep| (ep.id, ep.value))
            .collect()
    };

    // Links
    let link_rows = issue_links::Entity::find()
        .active()
        .filter(issue_links::Column::IssueId.is_in(issue_ids.clone()))
        .all(&state.db)
        .await?;
    let mut links_map: HashMap<Uuid, Vec<ExportLink>> = HashMap::new();
    let mut link_count_map: HashMap<Uuid, i64> = HashMap::new();
    for lnk in &link_rows {
        let title = lnk.title.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| lnk.url.clone());
        links_map.entry(lnk.issue_id).or_default().push(ExportLink {
            url: lnk.url.clone(),
            title,
        });
        *link_count_map.entry(lnk.issue_id).or_insert(0) += 1;
    }

    // Relations (outgoing): issue_id → related_issue_id
    let relation_rows = issue_relations::Entity::find()
        .active()
        .filter(issue_relations::Column::IssueId.is_in(issue_ids.clone()))
        .all(&state.db)
        .await?;
    // Also incoming: related_issue_id → issue_id
    let incoming_relation_rows = issue_relations::Entity::find()
        .active()
        .filter(issue_relations::Column::RelatedIssueId.is_in(issue_ids.clone()))
        .all(&state.db)
        .await?;

    // Collect all issue IDs we need identifiers for (for relations)
    let mut related_issue_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    for r in &relation_rows {
        related_issue_ids.insert(r.related_issue_id);
    }
    for r in &incoming_relation_rows {
        related_issue_ids.insert(r.issue_id);
    }
    // Fetch those issues (for project.identifier + sequence_id)
    let related_issues: HashMap<Uuid, (String, i32)> = if related_issue_ids.is_empty() {
        HashMap::new()
    } else {
        issues::Entity::find()
            .filter(issues::Column::Id.is_in(related_issue_ids.into_iter().collect::<Vec<_>>()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|i| {
                let proj_ident = projects_map
                    .get(&i.project_id)
                    .map(|(ident, _)| ident.clone())
                    .unwrap_or_default();
                (i.id, (proj_ident, i.sequence_id))
            })
            .collect()
    };

    let mut relations_map: HashMap<Uuid, Vec<ExportRelation>> = HashMap::new();
    for r in &relation_rows {
        let (proj_ident, seq_id) = related_issues
            .get(&r.related_issue_id)
            .cloned()
            .unwrap_or_default();
        relations_map.entry(r.issue_id).or_default().push(ExportRelation {
            relation_type: r.relation_type.clone(),
            issue_identifier: format!("{proj_ident}-{seq_id}"),
            direction: "outgoing",
        });
    }
    for r in &incoming_relation_rows {
        let (proj_ident, seq_id) = related_issues
            .get(&r.issue_id)
            .cloned()
            .unwrap_or_default();
        relations_map.entry(r.related_issue_id).or_default().push(ExportRelation {
            relation_type: r.relation_type.clone(),
            issue_identifier: format!("{proj_ident}-{seq_id}"),
            direction: "incoming",
        });
    }

    // Comments
    let comment_rows = issue_comments::Entity::find()
        .active()
        .filter(issue_comments::Column::IssueId.is_in(issue_ids.clone()))
        .order_by_asc(issue_comments::Column::CreatedAt)
        .all(&state.db)
        .await?;
    for c in &comment_rows {
        if let Some(actor_id) = c.actor_id {
            all_user_ids.insert(actor_id);
        }
    }

    // Sub-issues count
    let sub_issue_rows = issues::Entity::find()
        .filter(issues::Column::ParentId.is_in(issue_ids.clone()))
        .filter(issues::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;
    let mut sub_issues_count_map: HashMap<Uuid, i64> = HashMap::new();
    for si in &sub_issue_rows {
        if let Some(pid) = si.parent_id {
            *sub_issues_count_map.entry(pid).or_insert(0) += 1;
        }
    }

    // Parent identifier: for issues with parent_id, resolve "PROJ-123" format
    let parent_ids: Vec<Uuid> = all_issues.iter().filter_map(|i| i.parent_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let mut parent_identifier_map: HashMap<Uuid, String> = HashMap::new();
    if !parent_ids.is_empty() {
        let parent_issues = issues::Entity::find()
            .filter(issues::Column::Id.is_in(parent_ids))
            .all(&state.db)
            .await?;
        for pi in &parent_issues {
            let proj_ident = projects_map
                .get(&pi.project_id)
                .map(|(ident, _)| ident.clone())
                .unwrap_or_default();
            parent_identifier_map.insert(pi.id, format!("{proj_ident}-{}", pi.sequence_id));
        }
    }

    // Attachment count: using issue_attachments (no entity available here, use 0)
    // Note: issue_attachments entity is in scope via entities mod but not imported in export.rs.
    // Batch it separately without adding a full entity import path.
    let mut attachment_count_map: HashMap<Uuid, i64> = HashMap::new();
    {
        use crate::entities::issue_attachments;
        let att_rows = crate::entities::issue_attachments::Entity::find()
            .filter(issue_attachments::Column::IssueId.is_in(issue_ids.clone()))
            .filter(issue_attachments::Column::DeletedAt.is_null())
            .all(&state.db)
            .await?;
        for att in &att_rows {
            *attachment_count_map.entry(att.issue_id).or_insert(0) += 1;
        }
    }

    // Build users_map after collecting all user IDs
    let users_map: HashMap<Uuid, String> = if all_user_ids.is_empty() {
        HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(all_user_ids.into_iter().collect::<Vec<_>>()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| (u.id, format_user_name(&u.first_name, &u.last_name)))
            .collect()
    };

    // Build assignees_map using now-complete users_map
    let mut assignees_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for a in &assignee_rows {
        let name = users_map.get(&a.assignee_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            assignees_map.entry(a.issue_id).or_default().push(name);
        }
    }

    // Build subscribers_map
    let mut subscribers_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for s in &subscriber_rows {
        let name = users_map.get(&s.subscriber_id).cloned().unwrap_or_default();
        if !name.is_empty() {
            subscribers_map.entry(s.issue_id).or_default().push(name);
        }
    }

    // Build comments_map
    let mut comments_map: HashMap<Uuid, Vec<ExportComment>> = HashMap::new();
    for c in &comment_rows {
        let created_by = c.actor_id
            .and_then(|id| users_map.get(&id))
            .cloned()
            .unwrap_or_default();
        comments_map.entry(c.issue_id).or_default().push(ExportComment {
            comment: c.comment_stripped.clone(),
            created_by,
            created_at: c.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    Ok(RelationMaps {
        projects: projects_map,
        states: states_map,
        assignees: assignees_map,
        subscribers: subscribers_map,
        users: users_map,
        labels: labels_map,
        cycles: cycles_map,
        modules: modules_map,
        estimates: estimates_map,
        links: links_map,
        relations: relations_map,
        comments: comments_map,
        sub_issues_count: sub_issues_count_map,
        attachment_count: attachment_count_map,
        link_count: link_count_map,
        parent_identifier: parent_identifier_map,
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
/// `IssueExportSerializer.Meta.fields` (27 columns, full parity).
struct IssueRow<'a> {
    project_name: String,
    project_identifier: String,
    parent: String,
    identifier: String,
    sequence_id: i32,
    name: &'a str,
    state: String,
    priority: &'a str,
    assignees: Vec<String>,
    subscribers: Vec<String>,
    created_by_name: String,
    start_date: String,
    target_date: String,
    completed_at: String,
    created_at: String,
    updated_at: String,
    archived_at: String,
    estimate: String,
    labels: Vec<String>,
    cycles: Vec<String>,
    modules: Vec<String>,
    links: Vec<ExportLink>,
    relations: Vec<ExportRelation>,
    comments: Vec<ExportComment>,
    sub_issues_count: i64,
    link_count: i64,
    attachment_count: i64,
    is_draft: bool,
}

/// Headers in the same order as values are written.
/// Prettified (`snake_case → Title Case`) for parity with
/// `CSVFormatter.prettify_headers=True` and `XLSXFormatter.prettify_headers=True`.
const HEADERS: &[&str] = &[
    "Project Name",
    "Project Identifier",
    "Parent",
    "Identifier",
    "Sequence Id",
    "Name",
    "State",
    "Priority",
    "Assignees",
    "Subscribers",
    "Created By",
    "Start Date",
    "Target Date",
    "Completed At",
    "Created At",
    "Updated At",
    "Archived At",
    "Estimate",
    "Labels",
    "Cycles",
    "Modules",
    "Links",
    "Relations",
    "Comments",
    "Sub Issues Count",
    "Link Count",
    "Attachment Count",
    "Is Draft",
];

fn build_rows<'a>(
    issues_ref: &[&'a issues::Model],
    maps: &'a RelationMaps,
) -> Vec<IssueRow<'a>> {
    issues_ref
        .iter()
        .map(|issue| {
            let (project_name, project_identifier) = maps
                .projects
                .get(&issue.project_id)
                .map(|(ident, name)| (name.clone(), ident.clone()))
                .unwrap_or_default();
            let identifier = format!("{project_identifier}-{}", issue.sequence_id);
            let parent = issue
                .parent_id
                .and_then(|pid| maps.parent_identifier.get(&pid))
                .cloned()
                .unwrap_or_default();
            let state = issue
                .state_id
                .and_then(|sid| maps.states.get(&sid))
                .cloned()
                .unwrap_or_default();
            let estimate = issue
                .estimate_point_id
                .and_then(|ep_id| maps.estimates.get(&ep_id))
                .cloned()
                .unwrap_or_default();
            let created_by_name = issue
                .created_by_id
                .and_then(|uid| maps.users.get(&uid))
                .cloned()
                .unwrap_or_default();
            IssueRow {
                project_name,
                project_identifier,
                parent,
                identifier,
                sequence_id: issue.sequence_id,
                name: &issue.name,
                state,
                priority: &issue.priority,
                assignees: maps.assignees.get(&issue.id).cloned().unwrap_or_default(),
                subscribers: maps.subscribers.get(&issue.id).cloned().unwrap_or_default(),
                created_by_name,
                start_date: issue.start_date.map(|d| d.to_string()).unwrap_or_default(),
                target_date: issue.target_date.map(|d| d.to_string()).unwrap_or_default(),
                completed_at: issue
                    .completed_at
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
                created_at: issue.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: issue.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                archived_at: issue
                    .archived_at
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                estimate,
                labels: maps.labels.get(&issue.id).cloned().unwrap_or_default(),
                cycles: maps.cycles.get(&issue.id).cloned().unwrap_or_default(),
                modules: maps.modules.get(&issue.id).cloned().unwrap_or_default(),
                links: maps.links.get(&issue.id).cloned().unwrap_or_default(),
                relations: maps.relations.get(&issue.id).cloned().unwrap_or_default(),
                comments: maps.comments.get(&issue.id).cloned().unwrap_or_default(),
                sub_issues_count: maps.sub_issues_count.get(&issue.id).copied().unwrap_or(0),
                link_count: maps.link_count.get(&issue.id).copied().unwrap_or(0),
                attachment_count: maps.attachment_count.get(&issue.id).copied().unwrap_or(0),
                is_draft: issue.is_draft,
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
        // Complex fields serialized as JSON for CSV (parity with Django CSVFormatter
        // which uses json.dumps for list values).
        let links_json = serde_json::to_string(
            &r.links.iter().map(|l| serde_json::json!({"url": l.url, "title": l.title})).collect::<Vec<_>>()
        ).unwrap_or_default();
        let relations_json = serde_json::to_string(
            &r.relations.iter().map(|rel| serde_json::json!({"type": rel.relation_type, "issue": rel.issue_identifier, "direction": rel.direction})).collect::<Vec<_>>()
        ).unwrap_or_default();
        let comments_json = serde_json::to_string(
            &r.comments.iter().map(|c| serde_json::json!({"comment": c.comment, "created_by": c.created_by, "created_at": c.created_at})).collect::<Vec<_>>()
        ).unwrap_or_default();

        wtr.write_record([
            &sanitize_csv_cell(&r.project_name),
            &sanitize_csv_cell(&r.project_identifier),
            &sanitize_csv_cell(&r.parent),
            &sanitize_csv_cell(&r.identifier),
            &r.sequence_id.to_string(),
            &sanitize_csv_cell(r.name),
            &sanitize_csv_cell(&r.state),
            &sanitize_csv_cell(r.priority),
            &sanitize_csv_cell(&r.assignees.join("; ")),
            &sanitize_csv_cell(&r.subscribers.join("; ")),
            &sanitize_csv_cell(&r.created_by_name),
            &r.start_date,
            &r.target_date,
            &r.completed_at,
            &r.created_at,
            &r.updated_at,
            &r.archived_at,
            &sanitize_csv_cell(&r.estimate),
            &sanitize_csv_cell(&r.labels.join("; ")),
            &sanitize_csv_cell(&r.cycles.join("; ")),
            &sanitize_csv_cell(&r.modules.join("; ")),
            &links_json,
            &relations_json,
            &comments_json,
            &r.sub_issues_count.to_string(),
            &r.link_count.to_string(),
            &r.attachment_count.to_string(),
            &r.is_draft.to_string(),
        ])?;
    }
    let buf = wtr.into_inner()?;
    Ok(buf)
}

/// JSON indent=2. Parity with `JSONFormatter.encode(data, indent=2)`.
/// Keys in snake_case (JSONFormatter does not prettify headers).
fn encode_json(rows: &[IssueRow<'_>]) -> anyhow::Result<Vec<u8>> {
    use serde_json::{json, Value};

    let arr: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "project_name": r.project_name,
                "project_identifier": r.project_identifier,
                "parent": r.parent,
                "identifier": r.identifier,
                "sequence_id": r.sequence_id,
                "name": r.name,
                "state_name": r.state,
                "priority": r.priority,
                "assignees": r.assignees,
                "subscribers": r.subscribers,
                "created_by_name": r.created_by_name,
                "start_date": r.start_date,
                "target_date": r.target_date,
                "completed_at": r.completed_at,
                "created_at": r.created_at,
                "updated_at": r.updated_at,
                "archived_at": r.archived_at,
                "estimate": r.estimate,
                "labels": r.labels,
                "cycles": r.cycles,
                "modules": r.modules,
                "links": r.links.iter().map(|l| json!({"url": l.url, "title": l.title})).collect::<Vec<_>>(),
                "relations": r.relations.iter().map(|rel| json!({"type": rel.relation_type, "issue": rel.issue_identifier, "direction": rel.direction})).collect::<Vec<_>>(),
                "comments": r.comments.iter().map(|c| json!({"comment": c.comment, "created_by": c.created_by, "created_at": c.created_at})).collect::<Vec<_>>(),
                "sub_issues_count": r.sub_issues_count,
                "link_count": r.link_count,
                "attachment_count": r.attachment_count,
                "is_draft": r.is_draft,
            })
        })
        .collect();
    let s = serde_json::to_string_pretty(&arr)?;
    Ok(s.into_bytes())
}

/// XLSX via `rust_xlsxwriter`. Parity with `XLSXFormatter.encode` (openpyxl):
/// prettified headers, lists joined with ", ".
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
        let links_str = r.links.iter().map(|l| format!("{} ({})", l.title, l.url)).collect::<Vec<_>>().join(", ");
        let relations_str = r.relations.iter().map(|rel| format!("{}: {} ({})", rel.relation_type, rel.issue_identifier, rel.direction)).collect::<Vec<_>>().join(", ");
        let comments_str = r.comments.iter().map(|c| format!("[{}] {}", c.created_by, c.comment)).collect::<Vec<_>>().join("; ");

        let cells: Vec<String> = vec![
            r.project_name.clone(),
            r.project_identifier.clone(),
            r.parent.clone(),
            r.identifier.clone(),
            r.sequence_id.to_string(),
            r.name.to_string(),
            r.state.clone(),
            r.priority.to_string(),
            r.assignees.join(", "),
            r.subscribers.join(", "),
            r.created_by_name.clone(),
            r.start_date.clone(),
            r.target_date.clone(),
            r.completed_at.clone(),
            r.created_at.clone(),
            r.updated_at.clone(),
            r.archived_at.clone(),
            r.estimate.clone(),
            r.labels.join(", "),
            r.cycles.join(", "),
            r.modules.join(", "),
            links_str,
            relations_str,
            comments_str,
            r.sub_issues_count.to_string(),
            r.link_count.to_string(),
            r.attachment_count.to_string(),
            r.is_draft.to_string(),
        ];

        for (col, val) in cells.iter().enumerate() {
            sheet
                .write_string(row, col as u16, val)
                .map_err(|e| anyhow::anyhow!("xlsx write col {col}: {e}"))?;
        }
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
