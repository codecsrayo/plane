// src/routes/project_templates.rs
//! Project Templates endpoints.
//!
//! Mirrors Django's ProjectTemplateViewSet, ProjectSaveAsTemplateEndpoint,
//! and ProjectTemplateInstantiateEndpoint from
//! apps/api/plane/app/views/project/template.py.
//!
//!   GET    /api/workspaces/{slug}/project-templates/
//!   POST   /api/workspaces/{slug}/project-templates/
//!   GET    /api/workspaces/{slug}/project-templates/{pk}/
//!   PATCH  /api/workspaces/{slug}/project-templates/{pk}/
//!   DELETE /api/workspaces/{slug}/project-templates/{pk}/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/save-as-template/
//!   POST   /api/workspaces/{slug}/project-templates/{template_id}/instantiate/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
        permissions::{require_workspace_admin, ROLE_ADMIN, ROLE_MEMBER},
    },
    entities::{
        cycles, labels, modules, project_members, project_templates, projects, states,
    },
    error::AppError,
    routes::projects::ProjectResponse,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectTemplateResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: String,
    pub template_data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
}

impl ProjectTemplateResponse {
    fn from_model(m: project_templates::Model) -> Self {
        Self {
            id: m.id,
            workspace_id: m.workspace_id,
            name: m.name,
            description: m.description,
            template_data: m.template_data,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub template_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SaveAsTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

// ── GET /workspaces/{slug}/project-templates/ ─────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/project-templates/",
    tag = "Projects",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses((status = 200, description = "Template list")),
    security(("TokenAuth" = []))
)]
pub async fn list_templates(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<Vec<ProjectTemplateResponse>>, AppError> {
    let rows = project_templates::Entity::find()
        .active()
        .filter(project_templates::Column::WorkspaceId.eq(guard.workspace.id))
        .order_by_desc(project_templates::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(ProjectTemplateResponse::from_model)
            .collect(),
    ))
}

// ── POST /workspaces/{slug}/project-templates/ ────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/project-templates/",
    tag = "Projects",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Template created"),
        (status = 400, description = "Validation error"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_template(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<ProjectTemplateResponse>), AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    let now = chrono::Utc::now().fixed_offset();
    let template = project_templates::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or_default()),
        template_data: Set(body.template_data.unwrap_or_else(|| serde_json::json!({}))),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectTemplateResponse::from_model(template)),
    ))
}

// ── GET /workspaces/{slug}/project-templates/{pk}/ ────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/project-templates/{pk}/",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Template ID"),
    ),
    responses(
        (status = 200, description = "Template detail"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_template(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<ProjectTemplateResponse>, AppError> {
    let template = project_templates::Entity::find_by_id(pk)
        .active()
        .filter(project_templates::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ProjectTemplateResponse::from_model(template)))
}

// ── PATCH /workspaces/{slug}/project-templates/{pk}/ ─────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/project-templates/{pk}/",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Template ID"),
    ),
    responses(
        (status = 200, description = "Template updated"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_template(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateTemplateRequest>,
) -> Result<Json<ProjectTemplateResponse>, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let template = project_templates::Entity::find_by_id(pk)
        .active()
        .filter(project_templates::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: project_templates::ActiveModel = template.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    if let Some(data) = body.template_data {
        am.template_data = Set(data);
    }
    am.updated_by_id = Set(Some(guard.user.id));
    am.updated_at = Set(chrono::Utc::now().fixed_offset());

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ProjectTemplateResponse::from_model(updated)))
}

// ── DELETE /workspaces/{slug}/project-templates/{pk}/ ────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/project-templates/{pk}/",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Template ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_template(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let template = project_templates::Entity::find_by_id(pk)
        .active()
        .filter(project_templates::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: project_templates::ActiveModel = template.into();
    am.deleted_at = Set(Some(chrono::Utc::now().fixed_offset()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── POST /workspaces/{slug}/projects/{project_id}/save-as-template/ ───────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/save-as-template/",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Template created from project"),
        (status = 404, description = "Project not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn save_as_template(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<SaveAsTemplateRequest>,
) -> Result<(StatusCode, Json<ProjectTemplateResponse>), AppError> {
    if guard.project_member.role < ROLE_MEMBER && guard.workspace_member.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let project_id = guard.project.id;

    // Snapshot states (non-soft-deleted)
    let project_states: Vec<serde_json::Value> = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "color": s.color,
                "sequence": s.sequence,
                "group": s.group,
                "default": s.default,
                "description": s.description,
            })
        })
        .collect();

    let project_labels: Vec<serde_json::Value> = labels::Entity::find()
        .active()
        .filter(labels::Column::ProjectId.eq(Some(project_id)))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|l| {
            serde_json::json!({
                "name": l.name,
                "color": l.color,
                "description": l.description,
                "sort_order": l.sort_order,
            })
        })
        .collect();

    let project_modules: Vec<serde_json::Value> = modules::Entity::find()
        .active()
        .filter(modules::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "description": m.description,
                "status": m.status,
            })
        })
        .collect();

    let project_cycles: Vec<serde_json::Value> = cycles::Entity::find()
        .active()
        .filter(cycles::Column::ProjectId.eq(project_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "description": c.description,
            })
        })
        .collect();

    let project_member_roles: Vec<serde_json::Value> = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|m| serde_json::json!({"role": m.role}))
        .collect();

    let template_data = serde_json::json!({
        "states": project_states,
        "labels": project_labels,
        "modules": project_modules,
        "cycles": project_cycles,
        "member_roles": project_member_roles,
    });

    let template_name = body
        .name
        .unwrap_or_else(|| format!("{} Template", guard.project.name));
    let template_desc = body.description.unwrap_or_default();

    let now = chrono::Utc::now().fixed_offset();
    let template = project_templates::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(template_name),
        description: Set(template_desc),
        template_data: Set(template_data),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectTemplateResponse::from_model(template)),
    ))
}

// ── POST /workspaces/{slug}/project-templates/{template_id}/instantiate/ ──────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InstantiateTemplateRequest {
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub network: Option<i16>,
    pub emoji: Option<String>,
    pub project_lead_id: Option<Uuid>,
    pub default_assignee_id: Option<Uuid>,
    pub timezone: Option<String>,
    pub logo_props: Option<serde_json::Value>,
    pub cover_image: Option<String>,
    pub cover_image_asset_id: Option<Uuid>,
}

/// Default states used when a template doesn't define any.
struct DefaultStateSpec {
    name: &'static str,
    color: &'static str,
    sequence: f64,
    group: &'static str,
    is_default: bool,
}

const DEFAULT_STATES: &[DefaultStateSpec] = &[
    DefaultStateSpec { name: "Backlog",     color: "#60646C", sequence: 15000.0, group: "backlog",    is_default: true  },
    DefaultStateSpec { name: "Todo",        color: "#60646C", sequence: 25000.0, group: "unstarted",  is_default: false },
    DefaultStateSpec { name: "In Progress", color: "#F59E0B", sequence: 35000.0, group: "started",    is_default: false },
    DefaultStateSpec { name: "Done",        color: "#46A758", sequence: 45000.0, group: "completed",  is_default: false },
    DefaultStateSpec { name: "Cancelled",   color: "#9AA4BC", sequence: 55000.0, group: "cancelled",  is_default: false },
];

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/project-templates/{template_id}/instantiate/",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("template_id" = Uuid, Path, description = "Template ID"),
    ),
    responses(
        (status = 201, description = "Project created from template"),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Template not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn instantiate_template(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, template_id)): Path<(String, Uuid)>,
    Json(body): Json<InstantiateTemplateRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let template = project_templates::Entity::find_by_id(template_id)
        .active()
        .filter(project_templates::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let identifier = body.identifier.trim().to_uppercase();
    const FORBIDDEN_CHARS: &[char] = &[
        '&', '+', ',', ':', ';', '$', '^', '}', '{', '*', '=', '?', '@', '#', '|', '\'', '<',
        '>', '.', '(', ')', '%', '!', '-',
    ];
    if identifier.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_CANNOT_CONTAIN_SPECIAL_CHARACTERS"]
        })));
    }

    // Uniqueness checks
    let dup_id = projects::Entity::find()
        .active()
        .filter(projects::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(projects::Column::Identifier.eq(&identifier))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;
    if !dup_id.is_empty() {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_ALREADY_EXIST"]
        })));
    }

    let dup_name = projects::Entity::find()
        .active()
        .filter(projects::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(projects::Column::Name.eq(&body.name))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;
    if !dup_name.is_empty() {
        return Err(AppError::Validation(serde_json::json!({
            "name": ["PROJECT_NAME_ALREADY_EXIST"]
        })));
    }

    let network = body.network.unwrap_or(0);
    if ![0i16, 2].contains(&network) {
        return Err(AppError::BadRequest(
            "network must be 0 (secret) or 2 (public)".into(),
        ));
    }

    let tdata = &template.template_data;
    let project_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();
    let ws = &guard.workspace;

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    // Create project
    let new_project = projects::ActiveModel {
        id: Set(project_id),
        name: Set(body.name.clone()),
        identifier: Set(identifier),
        description: Set(body.description.unwrap_or_default()),
        description_text: Set(None),
        description_html: Set(None),
        network: Set(network),
        workspace_id: Set(ws.id),
        emoji: Set(body.emoji),
        icon_prop: Set(None),
        logo_props: Set(body.logo_props.unwrap_or_else(|| serde_json::json!({}))),
        cover_image: Set(body.cover_image),
        cover_image_asset_id: Set(body.cover_image_asset_id),
        default_assignee_id: Set(body.default_assignee_id),
        project_lead_id: Set(body.project_lead_id),
        default_state_id: Set(None),
        estimate_id: Set(None),
        cycle_view: Set(true),
        module_view: Set(true),
        issue_views_view: Set(true),
        page_view: Set(true),
        intake_view: Set(true),
        is_time_tracking_enabled: Set(false),
        is_issue_type_enabled: Set(false),
        guest_view_all_features: Set(false),
        timezone: Set(body.timezone.unwrap_or_else(|| ws.timezone.clone())),
        archive_in: Set(0),
        close_in: Set(0),
        archived_at: Set(None),
        deleted_at: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let project = new_project.insert(&txn).await.map_err(AppError::Database)?;

    // Add creator as Admin
    project_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        project_id: Set(project_id),
        workspace_id: Set(ws.id),
        member_id: Set(Some(guard.user.id)),
        role: Set(ROLE_ADMIN),
        is_active: Set(true),
        comment: Set(None),
        view_props: Set(serde_json::json!({})),
        default_props: Set(serde_json::json!({})),
        preferences: Set(serde_json::json!({})),
        sort_order: Set(65535.0),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    // Create states from template or fall back to defaults
    let template_states = tdata.get("states").and_then(|v| v.as_array()).cloned();
    let mut default_state_id: Option<Uuid> = None;

    if let Some(t_states) = template_states.filter(|s| !s.is_empty()) {
        for s in &t_states {
            let state_id = Uuid::new_v4();
            let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("State");
            let color = s.get("color").and_then(|v| v.as_str()).unwrap_or("#60646C");
            let sequence = s
                .get("sequence")
                .and_then(|v| v.as_f64())
                .unwrap_or(65535.0);
            let group = s
                .get("group")
                .and_then(|v| v.as_str())
                .unwrap_or("backlog");
            let is_default = s
                .get("default")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let description = s
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let slug = name.to_lowercase().replace(' ', "-");

            states::ActiveModel {
                id: Set(state_id),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
                color: Set(color.to_string()),
                slug: Set(slug),
                sequence: Set(sequence),
                group: Set(group.to_string()),
                default: Set(is_default),
                is_triage: Set(false),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                external_id: Set(None),
                external_source: Set(None),
                created_by_id: Set(Some(guard.user.id)),
                updated_by_id: Set(Some(guard.user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&txn)
            .await
            .map_err(AppError::Database)?;

            if is_default {
                default_state_id = Some(state_id);
            }
        }
    } else {
        // Fall back to default states
        for ds in DEFAULT_STATES {
            let state_id = Uuid::new_v4();
            let slug = ds.name.to_lowercase().replace(' ', "-");
            states::ActiveModel {
                id: Set(state_id),
                name: Set(ds.name.to_string()),
                description: Set(String::new()),
                color: Set(ds.color.to_string()),
                slug: Set(slug),
                sequence: Set(ds.sequence),
                group: Set(ds.group.to_string()),
                default: Set(ds.is_default),
                is_triage: Set(false),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                external_id: Set(None),
                external_source: Set(None),
                created_by_id: Set(Some(guard.user.id)),
                updated_by_id: Set(Some(guard.user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&txn)
            .await
            .map_err(AppError::Database)?;

            if ds.is_default {
                default_state_id = Some(state_id);
            }
        }
    }

    // Set default_state_id on project
    if let Some(ds_id) = default_state_id {
        let mut ap: projects::ActiveModel = project.clone().into();
        ap.default_state_id = Set(Some(ds_id));
        ap.update(&txn).await.map_err(AppError::Database)?;
    }

    // Create labels from template
    if let Some(t_labels) = tdata.get("labels").and_then(|v| v.as_array()) {
        for l in t_labels {
            let name = l.get("name").and_then(|v| v.as_str()).unwrap_or("Label");
            let color = l
                .get("color")
                .and_then(|v| v.as_str())
                .unwrap_or("#6b7280");
            let description = l
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sort_order = l
                .get("sort_order")
                .and_then(|v| v.as_f64())
                .unwrap_or(65535.0);

            labels::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
                color: Set(color.to_string()),
                parent_id: Set(None),
                project_id: Set(Some(project_id)),
                workspace_id: Set(ws.id),
                sort_order: Set(sort_order),
                created_by_id: Set(Some(guard.user.id)),
                updated_by_id: Set(Some(guard.user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                external_id: Set(None),
                external_source: Set(None),
                deleted_at: Set(None),
            }
            .insert(&txn)
            .await
            .map_err(AppError::Database)?;
        }
    }

    // Create modules from template
    if let Some(t_modules) = tdata.get("modules").and_then(|v| v.as_array()) {
        for m in t_modules {
            let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("Module");
            let description = m
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let status = m
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("backlog");

            modules::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
                description_text: Set(None),
                description_html: Set(None),
                start_date: Set(None),
                target_date: Set(None),
                status: Set(status.to_string()),
                lead_id: Set(None),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                view_props: Set(serde_json::json!({})),
                sort_order: Set(65535.0),
                external_id: Set(None),
                external_source: Set(None),
                archived_at: Set(None),
                logo_props: Set(serde_json::json!({})),
                created_by_id: Set(Some(guard.user.id)),
                updated_by_id: Set(Some(guard.user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&txn)
            .await
            .map_err(AppError::Database)?;
        }
    }

    // Create cycles from template
    if let Some(t_cycles) = tdata.get("cycles").and_then(|v| v.as_array()) {
        for c in t_cycles {
            let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("Cycle");
            let description = c
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            cycles::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
                start_date: Set(None),
                end_date: Set(None),
                owned_by_id: Set(guard.user.id),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                view_props: Set(serde_json::json!({})),
                sort_order: Set(65535.0),
                external_id: Set(None),
                external_source: Set(None),
                progress_snapshot: Set(serde_json::json!({})),
                archived_at: Set(None),
                logo_props: Set(serde_json::json!({})),
                timezone: Set(ws.timezone.clone()),
                version: Set(2),
                created_by_id: Set(Some(guard.user.id)),
                updated_by_id: Set(Some(guard.user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&txn)
            .await
            .map_err(AppError::Database)?;
        }
    }

    txn.commit().await.map_err(AppError::Database)?;

    let resp = ProjectResponse::from_model(&project, Some(1), Some(ROLE_ADMIN));
    Ok((StatusCode::CREATED, Json(resp)))
}
