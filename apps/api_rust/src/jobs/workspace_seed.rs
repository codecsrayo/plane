// src/jobs/workspace_seed.rs
//! Job: siembra de datos iniciales al crear un workspace.
//!
//! Equivalente a `plane/bgtasks/workspace_seed_task.py::workspace_seed`.
//!
//! Flujo:
//!   1. Crear bot user `WORKSPACE_SEED`
//!   2. Agregar bot como miembro Admin del workspace
//!   3. Crear proyecto demo con nombre del workspace
//!   4. Crear states, labels, cycles, modules
//!   5. Crear issues con asignaciones a cycles y modules
//!   6. Crear views y pages
//!
//! El job es idempotente: si el workspace ya tiene proyectos, no hace nada.

use apalis::prelude::*;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{
        cycle_issues, cycles, issue_labels, issue_sequences, issue_views,
        issues, labels, module_issues, modules, pages, project_members,
        project_pages, project_user_properties, projects, states,
        users, workspace_members,
    },
    AppState,
};

// ── Payload ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSeedJob {
    pub workspace_id: Uuid,
    /// ID del owner (el usuario que creó el workspace).
    pub owner_id: Uuid,
    /// Nombre del workspace, para usarlo en el proyecto demo.
    pub workspace_name: String,
}

// ── Constantes de roles ───────────────────────────────────────────────────────

const ROLE_ADMIN: i16 = 20;

// ── Handler apalis ────────────────────────────────────────────────────────────

pub async fn handle_workspace_seed(
    job: WorkspaceSeedJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    if let Err(e) = run_seed(&state, job).await {
        tracing::error!(error = %e, "workspace_seed: job falló");
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

// ── Lógica principal ──────────────────────────────────────────────────────────

async fn run_seed(state: &AppState, job: WorkspaceSeedJob) -> anyhow::Result<()> {
    use anyhow::Context as _;

    let db = &state.db;
    let workspace_id = job.workspace_id;
    let owner_id = job.owner_id;
    let now = Utc::now().fixed_offset();

    // Idempotencia: si el workspace ya tiene proyectos, salir
    let existing = projects::Entity::find()
        .filter(projects::Column::WorkspaceId.eq(workspace_id))
        .filter(projects::Column::DeletedAt.is_null())
        .one(db)
        .await?;

    if existing.is_some() {
        tracing::info!(%workspace_id, "workspace_seed: ya tiene proyectos, saltando");
        return Ok(());
    }

    // ── 1. Bot user ───────────────────────────────────────────────────────────
    let bot_id = Uuid::new_v4();
    let bot_email = format!("bot_seed_{}@plane.so", workspace_id.simple());
    let bot_username = format!("bot_seed_{}", workspace_id.simple());

    let bot = users::ActiveModel {
        id: Set(bot_id),
        username: Set(bot_username),
        email: Set(Some(bot_email)),
        password: Set(format!("pbkdf2_sha256$600000${}$unusable", Uuid::new_v4().simple())),
        display_name: Set("Plane".to_string()),
        first_name: Set("Plane".to_string()),
        last_name: Set(String::new()),
        is_bot: Set(true),
        bot_type: Set(Some("WORKSPACE_SEED".to_string())),
        is_active: Set(true),
        is_password_autoset: Set(true),
        is_superuser: Set(false),
        is_staff: Set(false),
        is_managed: Set(false),
        is_email_verified: Set(false),
        is_password_expired: Set(false),
        is_email_valid: Set(true),
        is_password_reset_required: Set(false),
        avatar: Set(String::new()),
        token: Set(Uuid::new_v4().simple().to_string()),
        user_timezone: Set("UTC".to_string()),
        last_location: Set(String::new()),
        created_location: Set(String::new()),
        last_login_ip: Set(String::new()),
        last_logout_ip: Set(String::new()),
        last_login_medium: Set("email".to_string()),
        last_login_uagent: Set(String::new()),
        date_joined: Set(now),
        created_at: Set(now),
        updated_at: Set(now),
        last_login: Set(None),
        last_active: Set(None),
        last_login_time: Set(None),
        last_logout_time: Set(None),
        token_updated_at: Set(None),
        mobile_number: Set(None),
        cover_image: Set(None),
        avatar_asset_id: Set(None),
        cover_image_asset_id: Set(None),
        masked_at: Set(None),
    };
    bot.insert(db).await.context("crear bot user")?;

    // ── 2. Bot como miembro Admin ─────────────────────────────────────────────
    workspace_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        workspace_id: Set(workspace_id),
        member_id: Set(bot_id),
        role: Set(ROLE_ADMIN),
        company_role: Set(None),
        is_active: Set(true),
        created_by_id: Set(Some(bot_id)),
        updated_by_id: Set(Some(bot_id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        view_props: Set(serde_json::json!({})),
        default_props: Set(serde_json::json!({})),
        issue_props: Set(serde_json::json!({})),
        explored_features: Set(serde_json::json!({})),
        getting_started_checklist: Set(serde_json::json!({})),
        tips: Set(serde_json::json!({})),
    }
    .insert(db)
    .await
    .context("agregar bot al workspace")?;

    // ── 3. Proyecto ───────────────────────────────────────────────────────────
    let project_id = Uuid::new_v4();
    let identifier: String = job
        .workspace_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(5)
        .collect::<String>()
        .to_uppercase();
    let identifier = if identifier.is_empty() { "DEMO".to_string() } else { identifier };

    projects::ActiveModel {
        id: Set(project_id),
        name: Set(job.workspace_name.clone()),
        identifier: Set(identifier),
        description: Set("Welcome to the Plane Demo Project!".to_string()),
        description_text: Set(None),
        description_html: Set(None),
        network: Set(2),
        workspace_id: Set(workspace_id),
        created_by_id: Set(Some(bot_id)),
        updated_by_id: Set(Some(bot_id)),
        default_assignee_id: Set(None),
        project_lead_id: Set(None),
        emoji: Set(None),
        icon_prop: Set(None),
        logo_props: Set(serde_json::json!({"emoji":{"url":"https://cdn.jsdelivr.net/npm/emoji-datasource-apple/img/apple/64/1f447.png","value":"128071"},"in_use":"emoji"})),
        cover_image: Set(Some("https://images.unsplash.com/photo-1691230995681-480d86cbc135?auto=format&fit=crop&q=80&w=870".to_string())),
        cycle_view: Set(true),
        module_view: Set(true),
        issue_views_view: Set(true),
        page_view: Set(true),
        intake_view: Set(false),
        estimate_id: Set(None),
        archive_in: Set(0),
        close_in: Set(0),
        default_state_id: Set(None),
        archived_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        is_time_tracking_enabled: Set(false),
        is_issue_type_enabled: Set(false),
        guest_view_all_features: Set(false),
        timezone: Set("UTC".to_string()),
        cover_image_asset_id: Set(None),
        external_id: Set(None),
        external_source: Set(None),
    }
    .insert(db)
    .await
    .context("crear proyecto")?;

    // Miembros del proyecto: todos los miembros actuales del workspace
    let ws_members = workspace_members::Entity::find()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .all(db)
        .await?;

    for wm in &ws_members {
        project_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            member_id: Set(Some(wm.member_id)),
            role: Set(wm.role),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            comment: Set(None),
            view_props: Set(serde_json::json!({})),
            default_props: Set(serde_json::json!({})),
            sort_order: Set(65535.0),
            preferences: Set(serde_json::json!({})),
            is_active: Set(true),
        }
        .insert(db)
        .await
        .context("agregar miembro al proyecto")?;

        // ProjectUserProperty por cada miembro real (no bot)
        if wm.member_id != bot_id {
            project_user_properties::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                user_id: Set(wm.member_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(bot_id)),
                updated_by_id: Set(Some(bot_id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
                display_filters: Set(serde_json::json!({"layout":"list","group_by":"state","order_by":"sort_order","sub_issue":true,"show_empty_groups":true})),
                display_properties: Set(serde_json::json!({"key":true,"state":true,"priority":true,"assignee":true,"estimate":true,"created_on":true,"updated_on":true})),
                filters: Set(serde_json::json!({})),
                rich_filters: Set(serde_json::json!({})),
                preferences: Set(serde_json::json!({})),
                sort_order: Set(65535.0),
            }
            .insert(db)
            .await
            .context("crear project user properties")?;
        }
    }

    // ── 4. States ─────────────────────────────────────────────────────────────
    let states_data = vec![
        (1u32, "Backlog",     "#A3A3A3", "backlog",    15000.0f64, true),
        (2,    "Todo",        "#3A3A3A", "unstarted",  25000.0,    false),
        (3,    "In Progress", "#F59E0B", "started",    35000.0,    false),
        (4,    "Done",        "#16A34A", "completed",  45000.0,    false),
        (5,    "Cancelled",   "#EF4444", "cancelled",  55000.0,    false),
    ];

    let mut state_map: std::collections::HashMap<u32, Uuid> = std::collections::HashMap::new();

    for (seed_id, name, color, group, seq, is_default) in &states_data {
        let state_id = Uuid::new_v4();
        states::ActiveModel {
            id: Set(state_id),
            name: Set(name.to_string()),
            color: Set(color.to_string()),
            group: Set(group.to_string()),
            description: Set(String::new()),
            slug: Set(name.to_lowercase().replace(' ', "-")),
            sequence: Set(*seq),
            default: Set(*is_default),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            external_id: Set(None),
            external_source: Set(None),
            sort_order: Set(65535.0),
            is_triage: Set(false),
        }
        .insert(db)
        .await
        .context("crear state")?;

        state_map.insert(*seed_id, state_id);
    }

    // ── 5. Labels ─────────────────────────────────────────────────────────────
    let labels_data = vec![
        (1u32, "admin",    "#0693e3", 85535.0f64),
        (2,    "concepts", "#9900ef", 95535.0),
    ];
    let mut label_map: std::collections::HashMap<u32, Uuid> = std::collections::HashMap::new();

    for (seed_id, name, color, sort_order) in &labels_data {
        let label_id = Uuid::new_v4();
        labels::ActiveModel {
            id: Set(label_id),
            name: Set(name.to_string()),
            color: Set(color.to_string()),
            description: Set(String::new()),
            project_id: Set(Some(project_id)),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            parent_id: Set(None),
            sort_order: Set(*sort_order),
            external_id: Set(None),
            external_source: Set(None),
            sort_order: Set(65535.0),
        }
        .insert(db)
        .await
        .context("crear label")?;

        label_map.insert(*seed_id, label_id);
    }

    // ── 6. Cycles ─────────────────────────────────────────────────────────────
    let cycle1_id = Uuid::new_v4();
    let cycle2_id = Uuid::new_v4();

    let current_start = now;
    let current_end = now + Duration::days(14);
    let upcoming_start = current_end + Duration::days(1);
    let upcoming_end = upcoming_start + Duration::days(14);

    cycles::ActiveModel {
        id: Set(cycle1_id),
        name: Set("Cycle 1: Getting Started with Plane".to_string()),
        description: Set(String::new()),
        start_date: Set(Some(current_start)),
        end_date: Set(Some(current_end)),
        project_id: Set(project_id),
        workspace_id: Set(workspace_id),
        owned_by_id: Set(bot_id),
        created_by_id: Set(Some(bot_id)),
        updated_by_id: Set(Some(bot_id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        archived_at: Set(None),
        sort_order: Set(1.0),
        view_props: Set(serde_json::json!({})),
        external_id: Set(None),
        external_source: Set(None),
            sort_order: Set(65535.0),
        progress_snapshot: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        timezone: Set("UTC".to_string()),
        version: Set(1),
    }
    .insert(db)
    .await
    .context("crear cycle 1")?;

    cycles::ActiveModel {
        id: Set(cycle2_id),
        name: Set("Cycle 2: Collaboration & Customization".to_string()),
        description: Set(String::new()),
        start_date: Set(Some(upcoming_start)),
        end_date: Set(Some(upcoming_end)),
        project_id: Set(project_id),
        workspace_id: Set(workspace_id),
        owned_by_id: Set(bot_id),
        created_by_id: Set(Some(bot_id)),
        updated_by_id: Set(Some(bot_id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        archived_at: Set(None),
        sort_order: Set(2.0),
        view_props: Set(serde_json::json!({})),
        external_id: Set(None),
        external_source: Set(None),
            sort_order: Set(65535.0),
        progress_snapshot: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        timezone: Set("UTC".to_string()),
        version: Set(1),
    }
    .insert(db)
    .await
    .context("crear cycle 2")?;

    let cycle_map: std::collections::HashMap<u32, Uuid> = [(1, cycle1_id), (2, cycle2_id)]
        .into_iter()
        .collect();

    // ── 7. Modules ────────────────────────────────────────────────────────────
    let modules_data = vec![
        (1u32, "Core Workflow (System)",    "planned",     "Manage, visualize, and track your work items across views."),
        (2,    "Onboarding Flow (Feature)", "backlog",     "Everything about getting started - creating a project, inviting teammates."),
        (3,    "Workspace Setup (Area)",    "in-progress", "The personalization layer - settings, labels, automations."),
    ];

    let mut module_map: std::collections::HashMap<u32, Uuid> = std::collections::HashMap::new();

    for (i, (seed_id, name, status, desc)) in modules_data.iter().enumerate() {
        let module_id = Uuid::new_v4();
        let mod_start = now + Duration::days(i as i64 * 2);
        let mod_end = mod_start + Duration::days(14);

        modules::ActiveModel {
            id: Set(module_id),
            name: Set(name.to_string()),
            description: Set(desc.to_string()),
            description_text: Set(None),
            description_html: Set(None),
            status: Set(status.to_string()),
            start_date: Set(Some(mod_start.date_naive())),
            target_date: Set(Some(mod_end.date_naive())),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            lead_id: Set(None),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            archived_at: Set(None),
            sort_order: Set(*seed_id as f64),
            view_props: Set(serde_json::json!({})),
            external_id: Set(None),
            external_source: Set(None),
            sort_order: Set(65535.0),
            logo_props: Set(serde_json::json!({})),
        }
        .insert(db)
        .await
        .context("crear module")?;

        module_map.insert(*seed_id, module_id);
    }

    // ── 8. Issues ─────────────────────────────────────────────────────────────
    // (seed_id, name, seq, state_seed, priority, sort_order, label_seeds, cycle_seed, module_seeds)
    let issues_data: Vec<(u32, &str, i32, u32, &str, f64, Vec<u32>, Option<u32>, Vec<u32>)> = vec![
        (1, "Welcome to Plane 👋",          1, 4, "urgent", 1000.0, vec![],  Some(1), vec![1]),
        (2, "1. Create Projects 🎯",         2, 2, "high",   2000.0, vec![2], Some(1), vec![1]),
        (3, "2. Invite Teammates 🤝",        3, 2, "high",   3000.0, vec![2], Some(1), vec![2]),
        (4, "3. Create Work Items ✅",       4, 2, "medium", 4000.0, vec![1], Some(1), vec![1]),
        (5, "4. Organize with Cycles 🔄",    5, 2, "medium", 5000.0, vec![1], Some(2), vec![1]),
        (6, "5. Group with Modules 📦",      6, 2, "low",    6000.0, vec![1], Some(2), vec![3]),
        (7, "6. Explore Views & Filters 🔍", 7, 2, "low",    7000.0, vec![], Some(2), vec![3]),
    ];

    for (_, name, seq, state_seed, priority, sort_order, label_seeds, cycle_seed, module_seeds) in &issues_data {
        let issue_id = Uuid::new_v4();

        issues::ActiveModel {
            id: Set(issue_id),
            name: Set(name.to_string()),
            description_html: Set(String::new()),
            description_stripped: Set(None),
            description_binary: Set(None),
            priority: Set(priority.to_string()),
            state_id: Set(state_map.get(state_seed).copied()),
            parent_id: Set(None),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            sequence_id: Set(*seq),
            sort_order: Set(*sort_order),
            type_id: Set(None),
            estimate_point_id: Set(None),
            start_date: Set(None),
            target_date: Set(None),
            completed_at: Set(None),
            archived_at: Set(None),
            is_draft: Set(false),
            description_json: Set(serde_json::json!({})),
            point: Set(None),
            external_id: Set(None),
            external_source: Set(None),
            sort_order: Set(65535.0),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(db)
        .await
        .context("crear issue")?;

        // IssueSequence
        issue_sequences::ActiveModel {
            id: Set(Uuid::new_v4()),
            sequence: Set(*seq as i64),
            deleted: Set(false),
            issue_id: Set(Some(issue_id)),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(db)
        .await
        .context("crear issue sequence")?;

        // Labels
        for label_seed in label_seeds {
            if let Some(&lid) = label_map.get(label_seed) {
                issue_labels::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    issue_id: Set(issue_id),
                    label_id: Set(lid),
                    project_id: Set(project_id),
                    workspace_id: Set(workspace_id),
                    created_by_id: Set(Some(bot_id)),
                    updated_by_id: Set(Some(bot_id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                }
                .insert(db)
                .await
                .context("crear issue label")?;
            }
        }

        // Cycle
        if let Some(cseed) = cycle_seed {
            if let Some(&cid) = cycle_map.get(cseed) {
                cycle_issues::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    issue_id: Set(issue_id),
                    cycle_id: Set(cid),
                    project_id: Set(project_id),
                    workspace_id: Set(workspace_id),
                    created_by_id: Set(Some(bot_id)),
                    updated_by_id: Set(Some(bot_id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                }
                .insert(db)
                .await
                .context("crear cycle issue")?;
            }
        }

        // Modules
        for mseed in module_seeds {
            if let Some(&mid) = module_map.get(mseed) {
                module_issues::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    issue_id: Set(issue_id),
                    module_id: Set(mid),
                    project_id: Set(project_id),
                    workspace_id: Set(workspace_id),
                    created_by_id: Set(Some(bot_id)),
                    updated_by_id: Set(Some(bot_id)),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: Set(None),
                }
                .insert(db)
                .await
                .context("crear module issue")?;
            }
        }
    }

    // ── 9. View ───────────────────────────────────────────────────────────────
    issue_views::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Project Urgent Tasks".to_string()),
        description: Set("Project Urgent Tasks".to_string()),
        query: Set(serde_json::json!({})),
        access: Set(1),
        filters: Set(serde_json::json!({})),
        project_id: Set(Some(project_id)),
        workspace_id: Set(workspace_id),
        owned_by_id: Set(owner_id),
        created_by_id: Set(Some(bot_id)),
        updated_by_id: Set(Some(bot_id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        display_filters: Set(serde_json::json!({"layout":"list","group_by":"state","order_by":"sort_order","sub_issue":false,"show_empty_groups":false})),
        display_properties: Set(serde_json::json!({"key":true,"link":true,"cycle":true,"state":true,"labels":true,"modules":true,"assignee":true,"due_date":true,"estimate":true,"priority":true,"created_on":true,"updated_on":true})),
        sort_order: Set(75535.0),
        logo_props: Set(serde_json::json!({})),
        is_locked: Set(false),
        rich_filters: Set(serde_json::json!({"priority__in": "urgent"})),
        archived_at: Set(None),
    }
    .insert(db)
    .await
    .context("crear view")?;

    // ── 10. Pages ─────────────────────────────────────────────────────────────
    let pages_data = vec![
        ("Project Design Spec",    0i16, serde_json::json!({"emoji":{"url":"https://cdn.jsdelivr.net/npm/emoji-datasource-apple/img/apple/64/1f680.png","value":"128640"},"in_use":"emoji"})),
        ("Project Draft Proposal", 1i16, serde_json::json!({"emoji":{"url":"https://cdn.jsdelivr.net/npm/emoji-datasource-apple/img/apple/64/1f9f1.png","value":"129521"},"in_use":"emoji"})),
    ];

    for (name, access, logo) in &pages_data {
        let page_id = Uuid::new_v4();

        pages::ActiveModel {
            id: Set(page_id),
            name: Set(name.to_string()),
            description_json: Set(serde_json::json!({})),
            description_html: Set(String::new()),
            description_stripped: Set(None),
            description_binary: Set(None),
            access: Set(*access),
            workspace_id: Set(workspace_id),
            owned_by_id: Set(owner_id),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            archived_at: Set(None),
            is_locked: Set(false),
            parent_id: Set(None),
            view_props: Set(serde_json::json!({})),
            logo_props: Set(logo.clone()),
            color: Set(String::new()),
            is_global: Set(false),
            moved_to_page: Set(None),
            moved_to_project: Set(None),
            external_id: Set(None),
            external_source: Set(None),
            sort_order: Set(65535.0),
        }
        .insert(db)
        .await
        .context("crear page")?;

        project_pages::ActiveModel {
            id: Set(Uuid::new_v4()),
            page_id: Set(page_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(bot_id)),
            updated_by_id: Set(Some(bot_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(db)
        .await
        .context("crear project page")?;
    }

    tracing::info!(
        %workspace_id,
        %project_id,
        "workspace_seed: siembra completada — 1 proyecto, 5 estados, 2 labels, 7 issues, 2 cycles, 3 modules, 1 view, 2 pages"
    );

    Ok(())
}
