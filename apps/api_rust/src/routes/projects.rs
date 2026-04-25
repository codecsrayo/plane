// src/routes/projects.rs
//! Endpoints de Project ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Fase 2b.
//!
//! Equivalente a `plane/app/views/project/base.py` y `member.py` en Django.
//! AutenticaciÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³n: sesiÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³n cookie **o** API key (via `AnyAuth`).
//! AutorizaciÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³n: membresÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a activa en workspace; rol Admin para mutaciones.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        permissions::{ROLE_ADMIN, ROLE_GUEST, ROLE_MEMBER, ROLE_VIEWER},
    },
    entities::{
        intake_issues, intakes, issue_sequences, project_deploy_boards, project_identifiers,
        project_member_invites, project_members, project_user_properties, projects, states,
        user_favorites, users, workspace_members, workspaces,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Estados por defecto al crear un proyecto (espeja DEFAULT_STATES de Django) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

struct DefaultState {
    name: &'static str,
    color: &'static str,
    sequence: f64,
    group: &'static str,
    is_default: bool,
    is_triage: bool,
}

const DEFAULT_STATES: &[DefaultState] = &[
    DefaultState {
        name: "Backlog",
        color: "#60646C",
        sequence: 15000.0,
        group: "backlog",
        is_default: true,
        is_triage: false,
    },
    DefaultState {
        name: "Todo",
        color: "#60646C",
        sequence: 25000.0,
        group: "unstarted",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "In Progress",
        color: "#F59E0B",
        sequence: 35000.0,
        group: "started",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "Done",
        color: "#46A758",
        sequence: 45000.0,
        group: "completed",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "Cancelled",
        color: "#9AA4BC",
        sequence: 55000.0,
        group: "cancelled",
        is_default: false,
        is_triage: false,
    },
    DefaultState {
        name: "Triage",
        color: "#4E5355",
        sequence: 65000.0,
        group: "triage",
        is_default: false,
        is_triage: true,
    },
];

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Helpers ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Obtiene proyecto activo por id dentro del workspace.
async fn project_by_id(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    project_id: Uuid,
) -> Result<projects::Model, AppError> {
    projects::Entity::find_by_id(project_id)
        .active()
        .filter(projects::Column::WorkspaceId.eq(workspace_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

/// Obtiene membresÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a activa de proyecto (o Forbidden).
async fn project_member_for_user(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<project_members::Model>, AppError> {
    project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(user_id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)
}

/// Verifica Admin de proyecto O Admin de workspace.
fn require_project_admin(
    pm: &Option<project_members::Model>,
    wm: &workspace_members::Model,
) -> Result<(), AppError> {
    let project_role = pm.as_ref().map(|m| m.role).unwrap_or(0);
    if project_role >= ROLE_ADMIN || wm.role >= ROLE_ADMIN {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn validate_role(role: i16) -> Result<(), AppError> {
    if [ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&role) {
        Ok(())
    } else {
        Err(AppError::BadRequest("Invalid role value".into()))
    }
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ DTOs ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub identifier: String,
    pub description: String,
    pub network: i16,
    /// FK al workspace. En Django el serializer expone la FK como `workspace`
    /// (no `workspace_id`); el frontend filtra proyectos por `project.workspace`.
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    pub emoji: Option<String>,
    pub icon_prop: Option<serde_json::Value>,
    pub logo_props: serde_json::Value,
    pub cover_image: Option<String>,
    pub default_assignee_id: Option<Uuid>,
    pub project_lead_id: Option<Uuid>,
    pub default_state_id: Option<Uuid>,
    pub estimate_id: Option<Uuid>,
    pub cycle_view: bool,
    pub module_view: bool,
    pub issue_views_view: bool,
    pub page_view: bool,
    pub intake_view: bool,
    pub is_time_tracking_enabled: bool,
    pub is_issue_type_enabled: bool,
    pub guest_view_all_features: bool,
    pub timezone: String,
    pub archive_in: i32,
    pub close_in: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// NÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºmero de miembros activos en el proyecto.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_members: Option<i64>,
    /// Rol del usuario en este proyecto.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_role: Option<i16>,
}

impl ProjectResponse {
    fn from_model(
        p: &projects::Model,
        total_members: Option<i64>,
        member_role: Option<i16>,
    ) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            identifier: p.identifier.clone(),
            description: p.description.clone(),
            network: p.network,
            workspace_id: p.workspace_id,
            emoji: p.emoji.clone(),
            icon_prop: p.icon_prop.clone(),
            logo_props: p.logo_props.clone(),
            cover_image: p.cover_image.clone(),
            default_assignee_id: p.default_assignee_id,
            project_lead_id: p.project_lead_id,
            default_state_id: p.default_state_id,
            estimate_id: p.estimate_id,
            cycle_view: p.cycle_view,
            module_view: p.module_view,
            issue_views_view: p.issue_views_view,
            page_view: p.page_view,
            intake_view: p.intake_view,
            is_time_tracking_enabled: p.is_time_tracking_enabled,
            is_issue_type_enabled: p.is_issue_type_enabled,
            guest_view_all_features: p.guest_view_all_features,
            timezone: p.timezone.clone(),
            archive_in: p.archive_in,
            close_in: p.close_in,
            archived_at: p.archived_at.map(Into::into),
            created_at: p.created_at.into(),
            updated_at: p.updated_at.into(),
            total_members,
            member_role,
        }
    }
}

/// Respuesta plana de `GET /workspaces/{slug}/projects/` ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ espeja **exactamente**
/// los 22 campos que Django expone mediante `.values(...)` en
/// `ProjectViewSet.list` (`plane/app/views/project/base.py`).
///
/// Claves importantes:
/// - `workspace` (no `workspace_id`): el frontend filtra con `project.workspace`.
/// - `project_lead` (no `project_lead_id`): convenciÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³n DRF para FK.
/// - `inbox_view`: alias de `intake_view`.
/// - `sort_order`: viene de `project_user_properties` del usuario.
/// - `intake_count`: conteo de `intake_issues` con status=-2 (PENDING).
///
/// NO incluye `description`, `emoji`, `cover_image`, `timezone`, etc. ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Django
/// tampoco los manda en este endpoint; para eso existe `/projects/{id}/` y
/// `/projects/details/`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectListResponse {
    pub id: Uuid,
    pub name: String,
    pub identifier: String,
    pub sort_order: Option<f64>,
    pub logo_props: serde_json::Value,
    pub member_role: Option<i16>,
    pub intake_count: i64,
    pub archived_at: Option<DateTime<Utc>>,
    pub workspace: Uuid,
    pub cycle_view: bool,
    pub issue_views_view: bool,
    pub module_view: bool,
    pub page_view: bool,
    pub inbox_view: bool,
    pub guest_view_all_features: bool,
    pub project_lead: Option<Uuid>,
    pub network: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub network: Option<i16>,
    pub emoji: Option<String>,
    pub project_lead_id: Option<Uuid>,
    pub default_assignee_id: Option<Uuid>,
    pub timezone: Option<String>,
    // Paridad Django: ProjectSerializer usa `fields = "__all__"` con
    // `read_only_fields = ["workspace", "deleted_at"]`, por lo que acepta
    // logo_props / cover_image / cover_image_asset en el body de POST.
    // Referencia: apps/api/plane/app/serializers/project.py:30-37.
    pub logo_props: Option<serde_json::Value>,
    pub cover_image: Option<String>,
    pub cover_image_asset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub network: Option<i16>,
    pub emoji: Option<String>,
    pub project_lead_id: Option<Uuid>,
    pub default_assignee_id: Option<Uuid>,
    pub timezone: Option<String>,
    pub cycle_view: Option<bool>,
    pub module_view: Option<bool>,
    pub issue_views_view: Option<bool>,
    pub page_view: Option<bool>,
    pub intake_view: Option<bool>,
    pub is_time_tracking_enabled: Option<bool>,
    pub cover_image: Option<String>,
    // Paridad Django: ProjectSerializer (partial_update con partial=True)
    // acepta logo_props y cover_image_asset en PATCH ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ fields = "__all__"
    // cubre ambos y ninguno estÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡ en read_only_fields.
    // Referencia: apps/api/plane/app/views/project/base.py:344-349.
    pub logo_props: Option<serde_json::Value>,
    pub cover_image_asset_id: Option<Uuid>,
    pub archive_in: Option<i32>,
    pub close_in: Option<i32>,
    pub guest_view_all_features: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectMemberResponse {
    pub id: Uuid,
    pub member_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub role: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&project_members::Model> for ProjectMemberResponse {
    fn from(m: &project_members::Model) -> Self {
        Self {
            id: m.id,
            member_id: m.member_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            role: m.role,
            is_active: m.is_active,
            created_at: m.created_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectMemberRequest {
    pub role: i16,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectInvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub role: i16,
    pub accepted: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<&project_member_invites::Model> for ProjectInvitationResponse {
    fn from(i: &project_member_invites::Model) -> Self {
        Self {
            id: i.id,
            email: i.email.clone(),
            role: i.role,
            accepted: i.accepted,
            project_id: i.project_id,
            workspace_id: i.workspace_id,
            created_at: i.created_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProjectInvitationRequest {
    pub emails: Vec<ProjectInviteEmail>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProjectInviteEmail {
    pub email: String,
    pub role: i16,
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Handlers ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// `GET /api/workspaces/{slug}/projects/`
///
/// Lista proyectos visibles para el usuario en el workspace. Espeja
/// `ProjectViewSet.list` de Django:
/// - ADMIN: todos los proyectos del workspace.
/// - MEMBER: proyectos donde es miembro activo **o** `network == 2` (public).
/// - GUEST: solo proyectos donde es miembro activo.
///
/// Devuelve [`ProjectListResponse`] con el mismo shape que `.values(...)` de DRF.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Project list", body = Vec<ProjectListResponse>),
        (status = 403, description = "Not a workspace member"),
    )
)]
pub async fn list_projects(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<ProjectListResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 1. Filtrado por rol (espeja `def list` de Django) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let projects_list = if wm.role >= ROLE_ADMIN {
        projects::Entity::find()
            .active()
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        let member_project_ids: Vec<Uuid> = project_members::Entity::find()
            .active()
            .filter(project_members::Column::WorkspaceId.eq(ws.id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|pm| pm.project_id)
            .collect();

        if wm.role == ROLE_GUEST {
            // GUEST: estrictamente sus proyectos
            if member_project_ids.is_empty() {
                return Ok(Json(vec![]));
            }
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(projects::Column::Id.is_in(member_project_ids))
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        } else {
            // MEMBER (o VIEWER): sus proyectos + proyectos pÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºblicos (network=2)
            use sea_orm::Condition;
            let condition = if member_project_ids.is_empty() {
                Condition::all().add(projects::Column::Network.eq(2i16))
            } else {
                Condition::any()
                    .add(projects::Column::Id.is_in(member_project_ids))
                    .add(projects::Column::Network.eq(2i16))
            };
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(condition)
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        }
    };

    if projects_list.is_empty() {
        return Ok(Json(vec![]));
    }

    let project_ids: Vec<Uuid> = projects_list.iter().map(|p| p.id).collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 2. `member_role` del usuario por proyecto (solo memberships activas) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let pm_map: std::collections::HashMap<Uuid, i16> = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::ProjectId.is_in(project_ids.clone()))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|pm| (pm.project_id, pm.role))
        .collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 3. `sort_order` por proyecto, del usuario actual ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let sort_orders: std::collections::HashMap<Uuid, f64> =
        project_user_properties::Entity::find()
            .filter(project_user_properties::Column::UserId.eq(user.id))
            .filter(project_user_properties::Column::WorkspaceId.eq(ws.id))
            .filter(project_user_properties::Column::ProjectId.is_in(project_ids.clone()))
            .filter(project_user_properties::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.project_id, p.sort_order))
            .collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 4. `intake_count` por proyecto (status=-2 PENDING, no soft-deleted) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    //
    // Una sola query agrupada en lugar de N+1 (mÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡s eficiente que el loop en
    // `list_projects_detail`). Espeja el `Count(filter=Q(status=-2, ...))` de Django.
    let mut intake_counts: std::collections::HashMap<Uuid, i64> =
        std::collections::HashMap::new();
    let intake_rows: Vec<(Uuid, i64)> = intake_issues::Entity::find()
        .select_only()
        .column(intake_issues::Column::ProjectId)
        .column_as(
            sea_orm::sea_query::Expr::col(intake_issues::Column::Id).count(),
            "count",
        )
        .filter(intake_issues::Column::ProjectId.is_in(project_ids.clone()))
        .filter(intake_issues::Column::Status.eq(-2i32))
        .filter(intake_issues::Column::DeletedAt.is_null())
        .group_by(intake_issues::Column::ProjectId)
        .into_tuple()
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;
    for (pid, c) in intake_rows {
        intake_counts.insert(pid, c);
    }

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 5. Ensamblar + ordenar por (sort_order NULLS LAST, name) como Django ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let mut responses: Vec<ProjectListResponse> = projects_list
        .iter()
        .map(|p| ProjectListResponse {
            id: p.id,
            name: p.name.clone(),
            identifier: p.identifier.clone(),
            sort_order: sort_orders.get(&p.id).copied(),
            logo_props: p.logo_props.clone(),
            member_role: pm_map.get(&p.id).copied(),
            intake_count: *intake_counts.get(&p.id).unwrap_or(&0),
            archived_at: p.archived_at.map(Into::into),
            workspace: p.workspace_id,
            cycle_view: p.cycle_view,
            issue_views_view: p.issue_views_view,
            module_view: p.module_view,
            page_view: p.page_view,
            inbox_view: p.intake_view,
            guest_view_all_features: p.guest_view_all_features,
            project_lead: p.project_lead_id,
            network: p.network,
            created_at: p.created_at.into(),
            updated_at: p.updated_at.into(),
            created_by: p.created_by_id,
            updated_by: p.updated_by_id,
        })
        .collect();

    responses.sort_by(|a, b| {
        // NULLS LAST en sort_order, luego por name (case-sensitive como PG default).
        match (a.sort_order, b.sort_order) {
            (Some(x), Some(y)) => x
                .partial_cmp(&y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });

    Ok(Json(responses))
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ DTO extendido para /details ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Respuesta extendida de proyecto ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ espeja `ProjectListSerializer` de Django.
/// Incluye campos calculados: `is_favorite`, `sort_order`, `members`, `anchor`,
/// `inbox_view`, `intake_count`, `next_work_item_sequence`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectDetailResponse {
    #[serde(flatten)]
    pub base: ProjectResponse,
    pub is_favorite: bool,
    pub sort_order: Option<f64>,
    /// UUIDs de miembros activos del proyecto.
    pub members: Vec<Uuid>,
    /// Anchor del deploy-board pÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºblico, si existe.
    pub anchor: Option<String>,
    /// Alias de intake_view para compatibilidad con el frontend.
    pub inbox_view: bool,
    /// NÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºmero de intake-issues pendientes (status = -2).
    pub intake_count: i64,
    /// PrÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³ximo sequence_id disponible para un nuevo issue.
    pub next_work_item_sequence: i64,
}

/// `GET /api/workspaces/{slug}/projects/details/`
///
/// Lista completa de proyectos con todos los campos calculados que el
/// frontend necesita para renderizar el sidebar y la home de proyectos.
/// Espeja `ProjectViewSet.list_detail` de Django.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/details/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Project detail list", body = Vec<ProjectDetailResponse>),
        (status = 403, description = "Not a workspace member"),
    )
)]
pub async fn list_projects_detail(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<ProjectDetailResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 1. Proyectos visibles segÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºn rol ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let projects_list = if wm.role >= ROLE_ADMIN {
        projects::Entity::find()
            .active()
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .order_by_asc(projects::Column::Name)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
    } else {
        let member_project_ids: Vec<Uuid> = project_members::Entity::find()
            .active()
            .filter(project_members::Column::WorkspaceId.eq(ws.id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|pm| pm.project_id)
            .collect();

        if wm.role == ROLE_GUEST {
            if member_project_ids.is_empty() {
                return Ok(Json(vec![]));
            }
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(projects::Column::Id.is_in(member_project_ids))
                .order_by_asc(projects::Column::Name)
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        } else {
            // MEMBER: propios + proyectos pÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºblicos (network=2)
            use sea_orm::Condition;
            let condition = if member_project_ids.is_empty() {
                Condition::all().add(projects::Column::Network.eq(2i16))
            } else {
                Condition::any()
                    .add(projects::Column::Id.is_in(member_project_ids))
                    .add(projects::Column::Network.eq(2i16))
            };
            projects::Entity::find()
                .active()
                .filter(projects::Column::WorkspaceId.eq(ws.id))
                .filter(condition)
                .order_by_asc(projects::Column::Name)
                .all(&state.db)
                .await
                .map_err(AppError::Database)?
        }
    };

    if projects_list.is_empty() {
        return Ok(Json(vec![]));
    }

    let project_ids: Vec<Uuid> = projects_list.iter().map(|p| p.id).collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 2. MembresÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­as: role del usuario y lista completa de miembros ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let all_members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::ProjectId.is_in(project_ids.clone()))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let user_role_map: std::collections::HashMap<Uuid, i16> = all_members
        .iter()
        .filter(|m| m.member_id == Some(user.id))
        .map(|m| (m.project_id, m.role))
        .collect();

    let mut members_map: std::collections::HashMap<Uuid, Vec<Uuid>> =
        std::collections::HashMap::new();
    for m in &all_members {
        if let Some(mid) = m.member_id {
            members_map.entry(m.project_id).or_default().push(mid);
        }
    }

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 3. Favoritos del usuario ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let favorites: std::collections::HashSet<Uuid> = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.is_in(project_ids.clone()))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .filter_map(|f| f.entity_identifier)
        .collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 4. sort_order del usuario por proyecto ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let sort_orders: std::collections::HashMap<Uuid, f64> =
        project_user_properties::Entity::find()
            .filter(project_user_properties::Column::UserId.eq(user.id))
            .filter(project_user_properties::Column::WorkspaceId.eq(ws.id))
            .filter(project_user_properties::Column::ProjectId.is_in(project_ids.clone()))
            .filter(project_user_properties::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.project_id, p.sort_order))
            .collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 5. Deploy-board anchor ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let anchors: std::collections::HashMap<Uuid, String> =
        project_deploy_boards::Entity::find()
            .filter(project_deploy_boards::Column::WorkspaceId.eq(ws.id))
            .filter(project_deploy_boards::Column::ProjectId.is_in(project_ids.clone()))
            .filter(project_deploy_boards::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|d| (d.project_id, d.anchor))
            .collect();

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 6. intake_count (pending = -2) por proyecto ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let mut intake_counts: std::collections::HashMap<Uuid, i64> =
        std::collections::HashMap::new();
    for &pid in &project_ids {
        let count = intake_issues::Entity::find()
            .filter(intake_issues::Column::ProjectId.eq(pid))
            .filter(intake_issues::Column::Status.eq(-2i32))
            .filter(intake_issues::Column::DeletedAt.is_null())
            .count(&state.db)
            .await
            .map_err(AppError::Database)? as i64;
        intake_counts.insert(pid, count);
    }

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 7. next_work_item_sequence por proyecto ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let mut next_sequences: std::collections::HashMap<Uuid, i64> =
        std::collections::HashMap::new();
    for &pid in &project_ids {
        let max_seq = issue_sequences::Entity::find()
            .select_only()
            .column(issue_sequences::Column::Sequence)
            .filter(issue_sequences::Column::ProjectId.eq(pid))
            .filter(issue_sequences::Column::DeletedAt.is_null())
            .order_by_desc(issue_sequences::Column::Sequence)
            .limit(1)
            .into_tuple::<i64>()
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;
        next_sequences.insert(pid, max_seq.map(|s| s + 1).unwrap_or(1));
    }

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ 8. Ensamblar respuestas ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let responses = projects_list
        .iter()
        .map(|p| {
            let role = user_role_map.get(&p.id).copied();
            let base = ProjectResponse::from_model(p, None, role);
            ProjectDetailResponse {
                is_favorite: favorites.contains(&p.id),
                sort_order: sort_orders.get(&p.id).copied(),
                members: members_map.get(&p.id).cloned().unwrap_or_default(),
                anchor: anchors.get(&p.id).cloned(),
                inbox_view: p.intake_view,
                intake_count: *intake_counts.get(&p.id).unwrap_or(&0),
                next_work_item_sequence: *next_sequences.get(&p.id).unwrap_or(&1),
                base,
            }
        })
        .collect();

    Ok(Json(responses))
}

/// `POST /api/workspaces/{slug}/projects/`
///
/// Crea un proyecto, lo inicializa con estados por defecto y agrega al
/// usuario como Admin del proyecto.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = ProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Validaciones de forma (forbidden chars, longitud) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    // Espejan plane.db.models.project.Project.FORBIDDEN_IDENTIFIER_CHARS_PATTERN
    // y ProjectSerializer.validate_identifier de Django.
    if body.name.is_empty() || body.name.len() > 255 {
        return Err(AppError::Validation(serde_json::json!({
            "name": ["PROJECT_NAME_INVALID_LENGTH"]
        })));
    }
    if body.identifier.is_empty() || body.identifier.len() > 12 {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_INVALID_LENGTH"]
        })));
    }
    let identifier = body.identifier.trim().to_uppercase();
    // Django rechaza: & + , : ; $ ^ } { * = ? @ # | ' < > . ( ) % ! -
    const FORBIDDEN_CHARS: &[char] = &[
        '&', '+', ',', ':', ';', '$', '^', '}', '{', '*', '=', '?', '@', '#', '|', '\'', '<',
        '>', '.', '(', ')', '%', '!', '-',
    ];
    if identifier.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_CANNOT_CONTAIN_SPECIAL_CHARACTERS"]
        })));
    }

    let ws = workspace_by_slug(&state.db, &slug).await?;
    // Cualquier miembro activo puede crear proyectos
    require_workspace_member(&state.db, ws.id, user.id).await?;

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Unicidad de identifier (solo proyectos no soft-deleted) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let dup_identifier = projects::Entity::find()
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .filter(projects::Column::Identifier.eq(&identifier))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;
    if dup_identifier > 0 {
        return Err(AppError::Validation(serde_json::json!({
            "identifier": ["PROJECT_IDENTIFIER_ALREADY_EXIST"]
        })));
    }

    // ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Unicidad de name (espeja ProjectSerializer.validate_name de Django) ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
    let dup_name = projects::Entity::find()
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .filter(projects::Column::Name.eq(&body.name))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;
    if dup_name > 0 {
        return Err(AppError::Validation(serde_json::json!({
            "name": ["PROJECT_NAME_ALREADY_EXIST"]
        })));
    }

    let project_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();
    let network = body.network.unwrap_or(0);

    if ![0i16, 2].contains(&network) {
        return Err(AppError::BadRequest(
            "network must be 0 (secret) or 2 (public)".into(),
        ));
    }

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    // Crear proyecto
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
        // Paridad Django: campos opcionales en POST, default server-side
        // cuando el cliente no los envÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a (JSONField default=dict / null).
        logo_props: Set(body.logo_props.clone().unwrap_or_else(|| serde_json::json!({}))),
        cover_image: Set(body.cover_image.clone()),
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
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let project = new_project.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert project");
        AppError::Database(e)
    })?;

    // Crear membresÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a Admin para el creador
    let creator_member = project_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        project_id: Set(project_id),
        workspace_id: Set(ws.id),
        member_id: Set(Some(user.id)),
        role: Set(ROLE_ADMIN),
        is_active: Set(true),
        comment: Set(None),
        view_props: Set(serde_json::json!({})),
        default_props: Set(serde_json::json!({})),
        preferences: Set(serde_json::json!({})),
        sort_order: Set(65535.0),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };
    creator_member.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert project member");
        AppError::Database(e)
    })?;

    // Si project_lead es distinto del creador, agregarlo tambiÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ©n como Admin
    if let Some(lead_id) = body.project_lead_id {
        if lead_id != user.id {
            let lead_member = project_members::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                member_id: Set(Some(lead_id)),
                role: Set(ROLE_ADMIN),
                is_active: Set(true),
                comment: Set(None),
                view_props: Set(serde_json::json!({})),
                default_props: Set(serde_json::json!({})),
                preferences: Set(serde_json::json!({})),
                sort_order: Set(65535.0),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            };
            lead_member.insert(&txn).await.map_err(|e| {
                tracing::error!(error = %e, "Failed to insert project lead member");
                AppError::Database(e)
            })?;
        }
    }

    // Crear estados por defecto (espeja DEFAULT_STATES de Django)
    let mut default_state_id: Option<Uuid> = None;
    for ds in DEFAULT_STATES {
        let state_id = Uuid::new_v4();
        let slug_state = ds.name.to_lowercase().replace(' ', "-");
        let new_state = states::ActiveModel {
            id: Set(state_id),
            name: Set(ds.name.to_string()),
            description: Set(String::new()),
            color: Set(ds.color.to_string()),
            slug: Set(slug_state),
            sequence: Set(ds.sequence),
            group: Set(ds.group.to_string()),
            default: Set(ds.is_default),
            is_triage: Set(ds.is_triage),
            project_id: Set(project_id),
            workspace_id: Set(ws.id),
            external_id: Set(None),
            external_source: Set(None),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        new_state.insert(&txn).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to insert default state");
            AppError::Database(e)
        })?;
        if ds.is_default {
            default_state_id = Some(state_id);
        }
    }

    // Fijar default_state_id en el proyecto
    if let Some(ds_id) = default_state_id {
        let mut active: projects::ActiveModel = project.clone().into();
        active.default_state_id = Set(Some(ds_id));
        active.update(&txn).await.map_err(AppError::Database)?;
    }

    txn.commit().await.map_err(AppError::Database)?;

    let resp = ProjectResponse::from_model(&project, Some(1), Some(ROLE_ADMIN));
    Ok((StatusCode::CREATED, Json(resp)))
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/`
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Project detail", body = ProjectResponse),
        (status = 403, description = "Not a member"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    // Verificar acceso: workspace admin o miembro del proyecto
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    if pm.is_none() && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let total = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    let role = pm.as_ref().map(|m| m.role);
    Ok(Json(ProjectResponse::from_model(
        &project,
        Some(total as i64),
        role,
    )))
}

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Updated project", body = ProjectResponse),
        (status = 403, description = "Requires project Admin or workspace Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    if let Some(ref name) = body.name {
        if name.is_empty() || name.len() > 255 {
            return Err(AppError::BadRequest(
                "Project name must be between 1 and 255 characters".into(),
            ));
        }
    }
    if let Some(net) = body.network {
        if ![0i16, 2].contains(&net) {
            return Err(AppError::BadRequest(
                "network must be 0 (secret) or 2 (public)".into(),
            ));
        }
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: projects::ActiveModel = project.into();

    // Capturar si el cliente estÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡ activando intake_view para aplicar la
    // creaciÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³n idempotente del Intake row despuÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ©s del update ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ paridad con
    // Django (apps/api/plane/app/views/project/base.py:353-360).
    let enabled_intake = body.intake_view == Some(true);

    if let Some(v) = body.name { active.name = Set(v); }
    if let Some(v) = body.description { active.description = Set(v); }
    if let Some(v) = body.network { active.network = Set(v); }
    if let Some(v) = body.emoji { active.emoji = Set(Some(v)); }
    if let Some(v) = body.project_lead_id { active.project_lead_id = Set(Some(v)); }
    if let Some(v) = body.default_assignee_id { active.default_assignee_id = Set(Some(v)); }
    if let Some(v) = body.timezone { active.timezone = Set(v); }
    if let Some(v) = body.cycle_view { active.cycle_view = Set(v); }
    if let Some(v) = body.module_view { active.module_view = Set(v); }
    if let Some(v) = body.issue_views_view { active.issue_views_view = Set(v); }
    if let Some(v) = body.page_view { active.page_view = Set(v); }
    if let Some(v) = body.intake_view { active.intake_view = Set(v); }
    if let Some(v) = body.is_time_tracking_enabled { active.is_time_tracking_enabled = Set(v); }
    if let Some(v) = body.cover_image { active.cover_image = Set(Some(v)); }
    if let Some(v) = body.cover_image_asset_id { active.cover_image_asset_id = Set(Some(v)); }
    if let Some(v) = body.logo_props { active.logo_props = Set(v); }
    if let Some(v) = body.archive_in { active.archive_in = Set(v); }
    if let Some(v) = body.close_in { active.close_in = Set(v); }
    if let Some(v) = body.guest_view_all_features { active.guest_view_all_features = Set(v); }

    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;

    // Paridad Django: si intake_view acaba de activarse, hacer get-or-create
    // del Intake row para el proyecto. Sin esto, `projects.intake_view=true`
    // pero la tabla `intakes` vacÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ el frontend ve UI de intake pero el
    // handler POST /intake-issues/ falla (ver routes/intake.rs).
    //
    // NOTA: Django tampoco usa transaction.atomic() aquÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­, asÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­ que si este
    // INSERT falla, el cliente verÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡ intake_view=true pero el intake no
    // existirÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡. El handler de create_intake_issue tiene get-or-create
    // defensivo como safety net.
    if enabled_intake {
        let existing = intakes::Entity::find()
            .active()
            .filter(intakes::Column::ProjectId.eq(updated.id))
            .filter(intakes::Column::IsDefault.eq(true))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if existing.is_none() {
            intakes::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(format!("{} Intake", updated.name)),
                description: Set(String::new()),
                is_default: Set(true),
                view_props: Set(serde_json::json!({})),
                logo_props: Set(serde_json::json!({})),
                project_id: Set(updated.id),
                workspace_id: Set(updated.workspace_id),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?;
        }
    }

    let role = pm.as_ref().map(|m| m.role);
    Ok(Json(ProjectResponse::from_model(&updated, None, role)))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/`
///
/// Soft-delete. Requiere Admin del proyecto o Admin del workspace.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 403, description = "Requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: projects::ActiveModel = project.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Project Members ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// `GET /api/workspaces/{slug}/projects/{project_id}/members/`
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Member list", body = Vec<ProjectMemberResponse>),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_project_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<ProjectMemberResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    if pm.is_none() && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .order_by_asc(project_members::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(members.iter().map(ProjectMemberResponse::from).collect()))
}

/// `PATCH /api/workspaces/{slug}/projects/{project_id}/members/{pk}/`
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("pk"         = Uuid,   Path, description = "Member record UUID"),
    ),
    request_body = UpdateProjectMemberRequest,
    responses(
        (status = 200, description = "Updated member", body = ProjectMemberResponse),
        (status = 403, description = "Requires Admin"),
    )
)]
pub async fn update_project_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateProjectMemberRequest>,
) -> Result<Json<ProjectMemberResponse>, AppError> {
    validate_role(body.role)?;

    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let target = project_members::Entity::find_by_id(pk)
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: project_members::ActiveModel = target.into();
    active.role = Set(body.role);
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ProjectMemberResponse::from(&updated)))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/members/{pk}/`
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("pk"         = Uuid,   Path, description = "Member record UUID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 403, description = "Requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn remove_project_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let target = project_members::Entity::find_by_id(pk)
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: project_members::ActiveModel = target.into();
    active.is_active = Set(false);
    active.deleted_at = Set(Some(now));
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ Project Invitations ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// `GET /api/workspaces/{slug}/projects/{project_id}/invitations/`
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Invitation list", body = Vec<ProjectInvitationResponse>),
        (status = 403, description = "Requires Admin"),
    )
)]
pub async fn list_project_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<ProjectInvitationResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let invites = project_member_invites::Entity::find()
        .active()
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .filter(project_member_invites::Column::Accepted.eq(false))
        .order_by_desc(project_member_invites::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(invites.iter().map(ProjectInvitationResponse::from).collect()))
}

/// `POST /api/workspaces/{slug}/projects/{project_id}/invitations/`
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    request_body = CreateProjectInvitationRequest,
    responses(
        (status = 201, description = "Invitations created", body = Vec<ProjectInvitationResponse>),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Requires Admin"),
    )
)]
pub async fn create_project_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateProjectInvitationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.emails.is_empty() {
        return Err(AppError::BadRequest("emails list is required".into()));
    }

    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut created = Vec::with_capacity(body.emails.len());

    for invite_req in &body.emails {
        validate_role(invite_req.role)?;

        let existing = project_member_invites::Entity::find()
            .active()
            .filter(project_member_invites::Column::ProjectId.eq(project_id))
            .filter(project_member_invites::Column::Email.eq(&invite_req.email))
            .filter(project_member_invites::Column::Accepted.eq(false))
            .count(&state.db)
            .await
            .map_err(AppError::Database)?;

        if existing > 0 {
            tracing::warn!(
                email = %invite_req.email,
                project = %project_id,
                "Skipping duplicate pending project invitation"
            );
            continue;
        }

        let new_invite = project_member_invites::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            workspace_id: Set(ws.id),
            email: Set(invite_req.email.clone()),
            role: Set(invite_req.role),
            accepted: Set(false),
            token: Set(Uuid::new_v4().to_string()),
            message: Set(None),
            responded_at: Set(None),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        let saved = new_invite
            .insert(&state.db)
            .await
            .map_err(AppError::Database)?;
        created.push(saved);
    }

    let responses: Vec<ProjectInvitationResponse> =
        created.iter().map(ProjectInvitationResponse::from).collect();
    Ok((StatusCode::CREATED, Json(responses)))
}

/// `DELETE /api/workspaces/{slug}/projects/{project_id}/invitations/{pk}/`
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
        ("pk"         = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 204, description = "Invitation deleted"),
        (status = 403, description = "Requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_project_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let _project = project_by_id(&state.db, ws.id, project_id).await?;

    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let invite = project_member_invites::Entity::find_by_id(pk)
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .filter(project_member_invites::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: project_member_invites::ActiveModel = invite.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ GET /workspaces/{slug}/projects/{project_id}/project-members/me ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ
//
// Mirror de `plane/app/views/project/member.py::ProjectMemberUserEndpoint`:
// devuelve el ProjectMember del usuario autenticado serializado con
// `ProjectMemberSerializer` (workspace/project/member anidados como "lite").
//
// Nota: Django usa `ProjectMember.objects.get(...)` sobre el manager que ya
// filtra `deleted_at__isnull=True`; la ausencia de resultado levanta 404 vÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a
// DoesNotExist. AquÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­ lo traducimos a `AppError::NotFound` con los mismos
// filtros (is_active + deleted_at IS NULL via `.active()`).

/// Subconjunto de `WorkspaceLiteSerializer`
/// (`apps/api/plane/app/serializers/workspace.py:78-82`):
/// `["name", "slug", "id", "logo_url"]`. NO incluye `logo` crudo, igual que Django.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceLiteDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    /// Mirror de `Workspace.logo_url` (`apps/api/plane/db/models/workspace.py:146-154`):
    /// `logo_asset.asset_url` ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ `logo` crudo ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ `None`.
    pub logo_url: Option<String>,
}

impl From<&workspaces::Model> for WorkspaceLiteDto {
    fn from(w: &workspaces::Model) -> Self {
        // logo_asset.asset_url para WORKSPACE_LOGO es `/api/assets/v2/static/{id}/`
        // (ver `apps/api/plane/db/models/asset.py:79-87`).
        let logo_url = if let Some(asset_id) = w.logo_asset_id {
            Some(format!("/api/assets/v2/static/{}/", asset_id))
        } else {
            w.logo.clone()
        };
        Self {
            id: w.id,
            name: w.name.clone(),
            slug: w.slug.clone(),
            logo_url,
        }
    }
}

/// Subconjunto de `ProjectLiteSerializer`
/// (`apps/api/plane/app/serializers/project.py:97-108`):
/// `["id","identifier","name","cover_image","cover_image_url","logo_props","description"]`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectLiteDto {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub cover_image: Option<String>,
    /// Mirror de `Project.cover_image_url`
    /// (`apps/api/plane/db/models/project.py:127-137`).
    pub cover_image_url: Option<String>,
    pub logo_props: serde_json::Value,
    pub description: String,
}

impl From<&projects::Model> for ProjectLiteDto {
    fn from(p: &projects::Model) -> Self {
        // PROJECT_COVER.asset_url ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ `/api/assets/v2/static/{id}/`
        let cover_image_url = if let Some(asset_id) = p.cover_image_asset_id {
            Some(format!("/api/assets/v2/static/{}/", asset_id))
        } else {
            p.cover_image.clone()
        };
        Self {
            id: p.id,
            identifier: p.identifier.clone(),
            name: p.name.clone(),
            cover_image: p.cover_image.clone(),
            cover_image_url,
            logo_props: p.logo_props.clone(),
            description: p.description.clone(),
        }
    }
}

/// Mirror de `ProjectMemberSerializer` (fields="__all__") con
/// `workspace`, `project`, `member` anidados (Lite). Se listan explÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­citamente
/// los campos que el store del frontend consume para no filtrar de mÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡s; los
/// campos devueltos son los que persiste el modelo `ProjectMember`
/// (`apps/api/plane/db/models/project.py::ProjectMember`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectMemberMeResponse {
    pub id: Uuid,
    pub role: i16,
    pub is_active: bool,
    pub comment: Option<String>,
    pub view_props: serde_json::Value,
    pub default_props: serde_json::Value,
    pub preferences: serde_json::Value,
    pub sort_order: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub member: Option<crate::routes::workspaces::UserLiteDto>,
    pub project: ProjectLiteDto,
    pub workspace: WorkspaceLiteDto,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub member_id: Option<Uuid>,
}

/// `GET /api/workspaces/{slug}/projects/{project_id}/project-members/me/`
///
/// Devuelve el ProjectMember del usuario autenticado. 404 si no tiene
/// membresÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­a activa (paridad con `ProjectMember.objects.get(...)` de Django).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-members/me/",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"       = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid,   Path, description = "Project UUID"),
    ),
    responses(
        (status = 200, description = "Current user's project member", body = ProjectMemberMeResponse),
        (status = 404, description = "User is not a member of this project"),
    )
)]
pub async fn get_project_member_me(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectMemberMeResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    // Django NO exige workspace-member para este endpoint, pero la consulta
    // por (workspace_slug, project_id, member=request.user, is_active=true)
    // devuelve 404 si el usuario no pertenece. Replicamos la misma semÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡ntica:
    // buscamos directamente la fila y devolvemos 404 si no existe.

    let member = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Cargamos proyecto activo y usuario para los anidados.
    // `project_by_id` filtra `deleted_at IS NULL` ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ si el proyecto estÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¡ borrado
    // devolvemos 404, igual que Django (`ProjectMember.project` con manager
    // `active_objects` filtra deleted_at).
    let project = project_by_id(&state.db, ws.id, project_id).await?;

    let user_row = users::Entity::find_by_id(user.id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Admin-visibility para email/last_login_medium: mismos criterios que otros
    // serializadores (`is_admin = role >= ROLE_ADMIN` en el proyecto).
    let is_admin = member.role >= ROLE_ADMIN;
    let member_dto = user_row
        .as_ref()
        .map(|u| crate::routes::workspaces::user_to_lite(u, is_admin));

    Ok(Json(ProjectMemberMeResponse {
        id: member.id,
        role: member.role,
        is_active: member.is_active,
        comment: member.comment.clone(),
        view_props: member.view_props.clone(),
        default_props: member.default_props.clone(),
        preferences: member.preferences.clone(),
        sort_order: member.sort_order,
        created_at: member.created_at.into(),
        updated_at: member.updated_at.into(),
        created_by: member.created_by_id,
        updated_by: member.updated_by_id,
        member: member_dto,
        project: ProjectLiteDto::from(&project),
        workspace: WorkspaceLiteDto::from(&ws),
        project_id: member.project_id,
        workspace_id: member.workspace_id,
        member_id: member.member_id,
    }))
}


// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ GET /workspaces/{slug}/projects/{project_id}/members/{pk}/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Retorna un miembro especÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ­fico del proyecto.
///
/// Espejo de `ProjectMemberViewSet.retrieve`
/// (`apps/api/plane/app/views/project/member.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn get_project_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let caller_pm = project_member_for_user(&state.db, project_id, user.id).await?;
    let is_admin = caller_pm.as_ref().map(|m| m.role).unwrap_or(0) > ROLE_GUEST
        || wm.role > ROLE_GUEST;

    let member = project_members::Entity::find_by_id(pk)
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let user_model = users::Entity::find_by_id(member.member_id.ok_or(AppError::NotFound)?)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({
        "id": member.id,
        "member_id": member.member_id,
        "role": member.role,
        "is_active": member.is_active,
        "member": {
            "id": user_model.id,
            "display_name": user_model.display_name,
            "first_name": user_model.first_name,
            "last_name": user_model.last_name,
            "avatar": user_model.avatar,
            "email": if is_admin { user_model.email.clone() } else { None },
        },
    })))
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ POST /workspaces/{slug}/projects/{project_id}/members/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AddProjectMembersRequest {
    pub members: Vec<ProjectMemberEntry>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ProjectMemberEntry {
    pub member_id: Uuid,
    pub role: i16,
}

/// Agrega miembros al proyecto en bulk.
///
/// Espejo de `ProjectMemberViewSet.create`
/// (`apps/api/plane/app/views/project/member.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn create_project_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<AddProjectMembersRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let caller_pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&caller_pm, &wm)?;

    if body.members.is_empty() {
        return Err(AppError::BadRequest("At least one member is required".into()));
    }

    for entry in &body.members {
        validate_role(entry.role)?;
    }

    let _project = project_by_id(&state.db, ws.id, project_id).await?;
    let now = chrono::Utc::now().fixed_offset();

    // Validar workspace roles ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ batch fetch en lugar de N queries.
    let member_ids: Vec<Uuid> = body.members.iter().map(|m| m.member_id).collect();
    let ws_members: std::collections::HashMap<Uuid, i16> = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.is_in(member_ids.clone()))
        .filter(workspace_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|wm| (wm.member_id, wm.role))
        .collect();

    for entry in &body.members {
        let ws_role = *ws_members.get(&entry.member_id).unwrap_or(&0);
        // Workspace admin no puede tener rol bajo en proyecto.
        if ws_role >= ROLE_ADMIN && entry.role <= ROLE_MEMBER {
            return Err(AppError::BadRequest(
                "Cannot assign a role lower than workspace admin role".into(),
            ));
        }
        // Workspace guest no puede tener rol alto en proyecto.
        if ws_role <= ROLE_GUEST && entry.role >= ROLE_MEMBER {
            return Err(AppError::BadRequest(
                "Cannot assign a role higher than workspace guest role".into(),
            ));
        }
    }

    // Upsert: reactivar si ya existe, o insertar nuevo.
    let existing: std::collections::HashMap<Uuid, project_members::Model> =
        project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::MemberId.is_in(member_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .filter_map(|m| m.member_id.map(|id| (id, m)))
            .collect();

    let _role_map: std::collections::HashMap<Uuid, i16> =
        body.members.iter().map(|m| (m.member_id, m.role)).collect();

    for entry in &body.members {
        if let Some(pm) = existing.get(&entry.member_id) {
            // Reactivar + actualizar rol
            let mut am: project_members::ActiveModel = pm.clone().into();
            am.role = Set(entry.role);
            am.is_active = Set(true);
            am.updated_at = Set(now);
            am.update(&state.db).await.map_err(AppError::Database)?;
        } else {
            // Crear nuevo. NOTA: project_members tiene varias columnas
            // jsonb/double NOT NULL sin DEFAULT en BD (baseline.sql
            // project_members): view_props, default_props, preferences,
            // sort_order. Django las popula vía model defaults (Python),
            // SeaORM no replica eso → debemos setearlas explícitamente o
            // el INSERT falla con 23502 → 500.
            project_members::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                member_id: Set(Some(entry.member_id)),
                role: Set(entry.role),
                is_active: Set(true),
                comment: Set(None),
                view_props: Set(crate::utils::django_defaults::default_props()),
                default_props: Set(crate::utils::django_defaults::default_props()),
                preferences: Set(crate::utils::django_defaults::default_preferences()),
                sort_order: Set(65535.0),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&state.db)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, project_id = %project_id, member_id = %entry.member_id, "create_project_members: insert project_members falló");
                AppError::Database(e)
            })?;
        }

        // project_user_properties ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ ON CONFLICT DO NOTHING
        // project_user_properties: mismas columnas NOT NULL sin DEFAULT en
        // BD (display_properties, display_filters, filters, rich_filters,
        // preferences, sort_order). Django defaults en
        // apps/api/plane/db/models/project.py:ProjectUserProperty.
        //
        // Idempotencia: la versión anterior usaba
        //   ON CONFLICT (project_id, user_id) DO NOTHING
        // pero la BD NO tiene un unique constraint sobre esas 2 columnas;
        // sólo:
        //   1) UNIQUE (user_id, project_id, deleted_at)  — 3 columnas
        //   2) partial unique (user_id, project_id) WHERE deleted_at IS NULL
        // Postgres rechaza el ON CONFLICT con
        //   "there is no unique or exclusion constraint matching the
        //    ON CONFLICT specification"
        // que el handler convertía en AppError::Database → 500.
        // Fix: chequeo previo (mismo patrón que el INSERT de project_members
        // arriba), sin ON CONFLICT.
        let existing_pup = project_user_properties::Entity::find()
            .filter(project_user_properties::Column::ProjectId.eq(project_id))
            .filter(project_user_properties::Column::UserId.eq(entry.member_id))
            .filter(project_user_properties::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if existing_pup.is_none() {
            project_user_properties::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                workspace_id: Set(ws.id),
                user_id: Set(entry.member_id),
                display_properties: Set(crate::utils::django_defaults::default_display_properties()),
                display_filters: Set(crate::utils::django_defaults::default_display_filters()),
                filters: Set(crate::utils::django_defaults::default_filters()),
                rich_filters: Set(serde_json::json!({})),
                preferences: Set(crate::utils::django_defaults::default_preferences()),
                sort_order: Set(65535.0),
                created_by_id: Set(Some(user.id)),
                updated_by_id: Set(Some(user.id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
            .insert(&state.db)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, project_id = %project_id, user_id = %entry.member_id, "create_project_members: insert project_user_properties falló");
                AppError::Database(e)
            })?;
        }
    }

    // Retornar miembros actualizados
    let updated_members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.is_in(
            body.members.iter().map(|m| m.member_id).collect::<Vec<_>>()
        ))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<serde_json::Value> = updated_members.iter().map(|m| serde_json::json!({
        "id": m.id,
        "member_id": m.member_id,
        "role": m.role,
        "project_id": m.project_id,
    })).collect();

    Ok((StatusCode::CREATED, Json(resp)))
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ POST /workspaces/{slug}/projects/{project_id}/members/leave/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// El usuario autenticado abandona el proyecto.
///
/// Espejo de `ProjectMemberViewSet.leave`
/// (`apps/api/plane/app/views/project/member.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/members/leave/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn leave_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let pm = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verificar que no sea el ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂºnico Admin del proyecto.
    if pm.role >= ROLE_ADMIN {
        let admin_count = project_members::Entity::find()
            .active()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::Role.gte(ROLE_ADMIN))
            .filter(project_members::Column::IsActive.eq(true))
            .count(&state.db)
            .await
            .map_err(AppError::Database)?;

        if admin_count <= 1 {
            return Err(AppError::BadRequest(
                "You cannot leave the project as you are the only admin.                  Please delete the project or promote another user to admin.".into(),
            ));
        }
    }

    let mut am: project_members::ActiveModel = pm.into();
    am.is_active = Set(false);
    am.updated_at = Set(chrono::Utc::now().fixed_offset());
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ POST /workspaces/{slug}/projects/{project_id}/project-views/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Persiste view_props, default_props, preferences y sort_order del miembro.
///
/// Espejo de `ProjectUserViewsEndpoint.post`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-views/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn update_project_views(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let _ws = workspace_by_slug(&state.db, &slug).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id)
        .await?
        .ok_or(AppError::Forbidden)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut am: project_members::ActiveModel = pm.into();

    if let Some(v) = body.get("view_props") {
        am.view_props = Set(v.clone());
    }
    if let Some(v) = body.get("default_props") {
        am.default_props = Set(v.clone());
    }
    if let Some(v) = body.get("sort_order") {
        if let Some(n) = v.as_f64() {
            am.sort_order = Set(n);
        }
    }
    am.updated_at = Set(now);
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ GET + POST + DELETE /workspaces/{slug}/user-favorite-projects/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Lista proyectos favoritos del usuario en el workspace.
///
/// Espejo de `ProjectFavoritesViewSet.list`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-favorite-projects/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn list_project_favorites(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let favs = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .order_by_asc(user_favorites::Column::Sequence)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = favs.iter().map(|f| serde_json::json!({
        "id": f.id,
        "entity_identifier": f.entity_identifier,
        "entity_type": f.entity_type,
        "project_id": f.project_id,
    })).collect();

    Ok(Json(result))
}

/// Agrega un proyecto a favoritos.
///
/// Espejo de `ProjectFavoritesViewSet.create`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/user-favorite-projects/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn create_project_favorite(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let project_id: Uuid = body.get("project")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("project is required".into()))?;

    let now = chrono::Utc::now().fixed_offset();

    // Idempotente: no duplicar si ya existe.
    let existing = user_favorites::Entity::find()
        .active()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_none() {
        user_favorites::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set("project".into()),
            entity_identifier: Set(Some(project_id)),
            project_id: Set(Some(project_id)),
            workspace_id: Set(ws.id),
            user_id: Set(user.id),
            is_folder: Set(false),
            sequence: Set(65535.0),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Elimina un proyecto de favoritos.
///
/// Espejo de `ProjectFavoritesViewSet.destroy`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/user-favorite-projects/{project_id}/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn delete_project_favorite(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Hard delete (Django: soft=False)
    user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::UserId.eq(user.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ POST + DELETE /workspaces/{slug}/projects/{project_id}/archive/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Archiva un proyecto.
///
/// Espejo de `ProjectArchiveUnarchiveEndpoint.post`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/archive/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn archive_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;

    // ADMIN o MEMBER pueden archivar
    let role = pm.as_ref().map(|m| m.role).unwrap_or(0);
    if role < ROLE_MEMBER && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let project = project_by_id(&state.db, ws.id, project_id).await?;
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let mut am: projects::ActiveModel = project.into();
    am.archived_at = Set(Some(now));
    am.updated_at = Set(now);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    // Eliminar favoritos del proyecto (Django behavior)
    user_favorites::Entity::delete_many()
        .filter(user_favorites::Column::WorkspaceId.eq(ws.id))
        .filter(user_favorites::Column::EntityType.eq("project"))
        .filter(user_favorites::Column::EntityIdentifier.eq(project_id))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "archived_at": updated.archived_at })))
}

/// Desarchiva un proyecto.
///
/// Espejo de `ProjectArchiveUnarchiveEndpoint.delete`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/archive/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn unarchive_project(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;

    let role = pm.as_ref().map(|m| m.role).unwrap_or(0);
    if role < ROLE_MEMBER && wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let project = projects::Entity::find_by_id(project_id)
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let mut am: projects::ActiveModel = project.into();
    am.archived_at = Set(None);
    am.updated_at = Set(now);
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ GET + DELETE /workspaces/{slug}/project-identifiers/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

#[derive(Debug, serde::Deserialize)]
pub struct IdentifierQuery {
    pub name: Option<String>,
}

/// Verifica si un identificador de proyecto ya existe en el workspace.
///
/// Espejo de `ProjectIdentifierEndpoint.get`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/project-identifiers/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn check_project_identifier(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Query(q): Query<IdentifierQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let _ = require_workspace_member(&state.db, ws.id, user.id).await?;

    let name = q.name
        .map(|n: String| n.trim().to_uppercase())
        .filter(|n: &String| !n.is_empty())
        .ok_or_else(|| AppError::BadRequest("name is required".into()))?;

    let identifiers = project_identifiers::Entity::find()
        .filter(project_identifiers::Column::Name.eq(&name))
        .filter(project_identifiers::Column::WorkspaceId.eq(ws.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let exists_count = identifiers.len();
    let identifiers_data: Vec<serde_json::Value> = identifiers.iter().map(|i| serde_json::json!({
        "id": i.id,
        "name": i.name,
        "project": i.project_id,
    })).collect();

    Ok(Json(serde_json::json!({
        "exists": exists_count,
        "identifiers": identifiers_data,
    })))
}

/// Elimina un identificador de proyecto sin proyecto asociado.
///
/// Espejo de `ProjectIdentifierEndpoint.delete`
/// (`apps/api/plane/app/views/project/base.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/project-identifiers/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn delete_project_identifier(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let name = body.get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("name is required".into()))?;

    // No eliminar si hay un proyecto activo con ese identifier
    let project_exists = projects::Entity::find()
        .active()
        .filter(projects::Column::Identifier.eq(&name))
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    if project_exists > 0 {
        return Err(AppError::BadRequest(
            "Cannot delete an identifier of an existing project".into(),
        ));
    }

    project_identifiers::Entity::delete_many()
        .filter(project_identifiers::Column::Name.eq(&name))
        .filter(project_identifiers::Column::WorkspaceId.eq(ws.id))
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ GET /workspaces/{slug}/projects/{project_id}/invitations/{pk}/ ÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ¢ÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ

/// Retorna el detalle de una invitaciÃÂÃÂÃÂÃÂÃÂÃÂÃÂÃÂ³n de proyecto.
///
/// Espejo de `ProjectInvitationsViewset.retrieve`
/// (`apps/api/plane/app/urls/project.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/invitations/{pk}/",
    tag = "Projects",
    security(("TokenAuth" = []))
)]
pub async fn get_project_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<ProjectInvitationResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    let pm = project_member_for_user(&state.db, project_id, user.id).await?;
    require_project_admin(&pm, &wm)?;

    let invite = project_member_invites::Entity::find_by_id(pk)
        .active()
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ProjectInvitationResponse::from(&invite)))
}

// ── Project Join & User Invitations ─────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProjectJoinRequest {
    /// Email del usuario que acepta la invitación.
    pub email: String,
    /// Si acepta o rechaza la invitación.
    #[serde(default)]
    pub accepted: bool,
}

/// Acepta o rechaza una invitación a un proyecto (endpoint público).
///
/// `POST /workspaces/{slug}/projects/{project_id}/join/{pk}`
///
/// Mirror de `ProjectJoinEndpoint.post` en Django.
/// No requiere autenticación (AllowAny), solo el email correcto.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/join/{pk}",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Invitation ID"),
    ),
    request_body = ProjectJoinRequest,
    responses(
        (status = 200, description = "Invitation accepted or declined"),
        (status = 400, description = "Already responded"),
        (status = 403, description = "Email mismatch"),
        (status = 404, description = "Invitation not found"),
    )
)]
pub async fn join_project_invitation(
    State(state): State<AppState>,
    Path((_slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<ProjectJoinRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = body.email.trim().to_lowercase();

    let invite = project_member_invites::Entity::find_by_id(pk)
        .filter(project_member_invites::Column::ProjectId.eq(project_id))
        .filter(project_member_invites::Column::WorkspaceId.is_not_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Verificar que el email coincide
    if email.is_empty() || invite.email.to_lowercase() != email {
        return Err(AppError::Forbidden);
    }

    // Ya respondió
    if invite.responded_at.is_some() {
        return Err(AppError::BadRequest(
            "You have already responded to the invitation request".into(),
        ));
    }

    // Registrar respuesta
    let mut active: project_member_invites::ActiveModel = invite.clone().into();
    active.accepted = Set(body.accepted);
    active.responded_at = Set(Some(chrono::Utc::now().into()));
    active.update(&state.db).await.map_err(AppError::Database)?;

    if !body.accepted {
        return Ok(Json(serde_json::json!({
            "message": "Project Invitation was not accepted"
        })));
    }

    // Aceptó — incorporar al workspace y proyecto
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&email))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if let Some(user) = user {
        // Asegurar membresía en workspace
        let ws_member = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(invite.workspace_id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if ws_member.is_none() {
            let role = if invite.role >= 15 { 15i16 } else { invite.role };
            let new_wm = workspace_members::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                workspace_id: Set(invite.workspace_id),
                member_id: Set(user.id),
                role: Set(role),
                is_active: Set(true),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let _ = new_wm.insert(&state.db).await; // ignorar conflicto
        } else if let Some(wm) = ws_member {
            let mut wm_active: workspace_members::ActiveModel = wm.into();
            wm_active.is_active = Set(true);
            let _ = wm_active.update(&state.db).await;
        }

        // Asegurar membresía en proyecto
        let pm = project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if pm.is_none() {
            let new_pm = project_members::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                project_id: Set(project_id),
                member_id: Set(Some(user.id)),
                role: Set(invite.role),
                workspace_id: Set(invite.workspace_id),
                is_active: Set(true),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let _ = new_pm.insert(&state.db).await; // ignorar conflicto
        } else if let Some(pm_model) = pm {
            let mut pm_active: project_members::ActiveModel = pm_model.into();
            pm_active.is_active = Set(true);
            let _ = pm_active.update(&state.db).await;
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Project Invitation Accepted"
    })))
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProjectInvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub accepted: bool,
    pub role: i16,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
}

/// Lista las invitaciones de proyectos del usuario autenticado.
///
/// `GET /users/me/workspaces/{slug}/projects/invitations`
///
/// Mirror de `UserProjectInvitationsViewset.list` en Django.
#[utoipa::path(
    get,
    path = "/users/me/workspaces/{slug}/projects/invitations",
    tag = "Projects",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
    ),
    responses(
        (status = 200, description = "List of project invitations"),
    ),
    security(("sessionAuth" = []))
)]
pub async fn list_user_project_invitations(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<UserProjectInvitationResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let invites = project_member_invites::Entity::find()
        .filter(project_member_invites::Column::Email.eq(
            user.email.as_deref().unwrap_or(""),
        ))
        .filter(project_member_invites::Column::WorkspaceId.eq(ws.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = invites
        .iter()
        .map(|i| UserProjectInvitationResponse {
            id: i.id,
            email: i.email.clone(),
            accepted: i.accepted,
            role: i.role,
            project_id: i.project_id,
            workspace_id: i.workspace_id,
        })
        .collect();

    Ok(Json(result))
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECT DEPLOY BOARDS
// ═══════════════════════════════════════════════════════════════════════════

use crate::entities::deploy_boards;

#[derive(Debug, Serialize)]
pub struct DeployBoardResponse {
    pub id: Uuid,
    pub anchor: String,
    pub is_comments_enabled: bool,
    pub is_reactions_enabled: bool,
    pub is_votes_enabled: bool,
    pub view_props: serde_json::Value,
    pub intake_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub entity_name: Option<String>,
    pub entity_identifier: Option<Uuid>,
    pub is_activity_enabled: bool,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<deploy_boards::Model> for DeployBoardResponse {
    fn from(m: deploy_boards::Model) -> Self {
        Self {
            id: m.id,
            anchor: m.anchor,
            is_comments_enabled: m.is_comments_enabled,
            is_reactions_enabled: m.is_reactions_enabled,
            is_votes_enabled: m.is_votes_enabled,
            view_props: m.view_props,
            intake_id: m.intake_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            entity_name: m.entity_name,
            entity_identifier: m.entity_identifier,
            is_activity_enabled: m.is_activity_enabled,
            is_disabled: m.is_disabled,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Deploy board or null"),
        (status = 403, description = "Forbidden"),
    )
)]
/// GET /workspaces/{slug}/projects/{project_id}/project-deploy-boards
/// Devuelve el deploy board de un proyecto (o null si no existe).
pub async fn get_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Option<DeployBoardResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    let board = deploy_boards::Entity::find()
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::EntityIdentifier.eq(project_id))
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(board.map(DeployBoardResponse::from)))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertDeployBoardRequest {
    pub is_comments_enabled: Option<bool>,
    pub is_reactions_enabled: Option<bool>,
    pub is_votes_enabled: Option<bool>,
    pub view_props: Option<serde_json::Value>,
    pub intake_id: Option<Uuid>,
}

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Deploy board created/updated"),
        (status = 403, description = "Forbidden"),
    )
)]
/// POST /workspaces/{slug}/projects/{project_id}/project-deploy-boards
/// Crea o actualiza el deploy board del proyecto (upsert como Django).
pub async fn upsert_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Json(body): Json<UpsertDeployBoardRequest>,
) -> Result<Json<DeployBoardResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    // Busca deploy board existente
    let existing = deploy_boards::Entity::find()
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::EntityIdentifier.eq(project_id))
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let default_view_props = serde_json::json!({
        "list": true, "kanban": true, "calendar": true, "gantt": true, "spreadsheet": true
    });

    let board = if let Some(existing) = existing {
        let mut am: deploy_boards::ActiveModel = existing.into();
        if let Some(v) = body.is_comments_enabled {
            am.is_comments_enabled = Set(v);
        }
        if let Some(v) = body.is_reactions_enabled {
            am.is_reactions_enabled = Set(v);
        }
        if let Some(v) = body.is_votes_enabled {
            am.is_votes_enabled = Set(v);
        }
        if let Some(v) = body.view_props {
            am.view_props = Set(v);
        }
        am.intake_id = Set(body.intake_id);
        am.updated_by_id = Set(Some(user.id));
        am.update(&state.db).await.map_err(AppError::Database)?
    } else {
        let anchor = uuid::Uuid::new_v4().to_string().replace('-', "");
        let am = deploy_boards::ActiveModel {
            id: Set(Uuid::new_v4()),
            anchor: Set(anchor),
            entity_name: Set(Some("project".to_string())),
            entity_identifier: Set(Some(project_id)),
            project_id: Set(Some(project_id)),
            workspace_id: Set(ws.id),
            is_comments_enabled: Set(body.is_comments_enabled.unwrap_or(false)),
            is_reactions_enabled: Set(body.is_reactions_enabled.unwrap_or(false)),
            is_votes_enabled: Set(body.is_votes_enabled.unwrap_or(false)),
            view_props: Set(body.view_props.unwrap_or(default_view_props)),
            intake_id: Set(body.intake_id),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            is_activity_enabled: Set(true),
            is_disabled: Set(false),
            deleted_at: Set(None),
            ..Default::default()
        };
        am.insert(&state.db).await.map_err(AppError::Database)?
    };

    Ok(Json(DeployBoardResponse::from(board)))
}

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Deploy board ID"),
    ),
    responses(
        (status = 200, description = "Updated deploy board"),
        (status = 404, description = "Not found"),
    )
)]
/// PATCH /workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}
pub async fn update_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpsertDeployBoardRequest>,
) -> Result<Json<DeployBoardResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    let existing = deploy_boards::Entity::find_by_id(pk)
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: deploy_boards::ActiveModel = existing.into();
    if let Some(v) = body.is_comments_enabled {
        am.is_comments_enabled = Set(v);
    }
    if let Some(v) = body.is_reactions_enabled {
        am.is_reactions_enabled = Set(v);
    }
    if let Some(v) = body.is_votes_enabled {
        am.is_votes_enabled = Set(v);
    }
    if let Some(v) = body.view_props {
        am.view_props = Set(v);
    }
    am.intake_id = Set(body.intake_id);
    am.updated_by_id = Set(Some(user.id));
    let board = am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(DeployBoardResponse::from(board)))
}

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Deploy board ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 403, description = "Forbidden"),
    )
)]
/// DELETE /workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}
pub async fn delete_project_deploy_board(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let wm = require_workspace_member(&state.db, ws.id, user.id).await?;
    if wm.role < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }
    let _ = project_by_id(&state.db, ws.id, project_id).await?;

    let board = deploy_boards::Entity::find_by_id(pk)
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: deploy_boards::ActiveModel = board.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECT MEMBER PREFERENCES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct MemberPreferencesResponse {
    pub preferences: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("member_id" = Uuid, Path, description = "Member user ID"),
    ),
    responses(
        (status = 200, description = "Member preferences JSON"),
        (status = 404, description = "Member not found"),
    )
)]
/// GET /workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}
pub async fn get_project_member_preferences(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, member_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<MemberPreferencesResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let pm = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(member_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(MemberPreferencesResponse {
        preferences: pm.preferences,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}",
    tag = "Projects",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("member_id" = Uuid, Path, description = "Member user ID"),
    ),
    responses(
        (status = 200, description = "Updated preferences JSON"),
        (status = 404, description = "Member not found"),
    )
)]
/// PATCH /workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}
pub async fn update_project_member_preferences(
    AnyAuth(user): AnyAuth,
    State(state): State<AppState>,
    Path((slug, project_id, member_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<MemberPreferencesResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let pm = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(member_id))
        .filter(project_members::Column::WorkspaceId.eq(ws.id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: project_members::ActiveModel = pm.into();
    am.preferences = Set(body);
    am.updated_by_id = Set(Some(user.id));
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(MemberPreferencesResponse {
        preferences: updated.preferences,
    }))
}
