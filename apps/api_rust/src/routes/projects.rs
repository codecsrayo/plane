// src/routes/projects.rs
//! Endpoints de Project — Fase 2b.
//!
//! Equivalente a `plane/app/views/project/base.py` y `member.py` en Django.
//! Autenticación: sesión cookie **o** API key (via `AnyAuth`).
//! Autorización: membresía activa en workspace; rol Admin para mutaciones.

use axum::{
    extract::{Path, State},
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
        intake_issues, issue_sequences, project_deploy_boards, project_member_invites,
        project_members, project_user_properties, projects, states, user_favorites,
        workspace_members,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ─── Estados por defecto al crear un proyecto (espeja DEFAULT_STATES de Django) ─

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

// ─── Helpers ──────────────────────────────────────────────────────────────────

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

/// Obtiene membresía activa de proyecto (o Forbidden).
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

// ─── DTOs ─────────────────────────────────────────────────────────────────────

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
    /// Número de miembros activos en el proyecto.
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

/// Respuesta plana de `GET /workspaces/{slug}/projects/` — espeja **exactamente**
/// los 22 campos que Django expone mediante `.values(...)` en
/// `ProjectViewSet.list` (`plane/app/views/project/base.py`).
///
/// Claves importantes:
/// - `workspace` (no `workspace_id`): el frontend filtra con `project.workspace`.
/// - `project_lead` (no `project_lead_id`): convención DRF para FK.
/// - `inbox_view`: alias de `intake_view`.
/// - `sort_order`: viene de `project_user_properties` del usuario.
/// - `intake_count`: conteo de `intake_issues` con status=-2 (PENDING).
///
/// NO incluye `description`, `emoji`, `cover_image`, `timezone`, etc. — Django
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

// ─── Handlers ─────────────────────────────────────────────────────────────────

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

    // ── 1. Filtrado por rol (espeja `def list` de Django) ───────────────────
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
            // MEMBER (o VIEWER): sus proyectos + proyectos públicos (network=2)
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

    // ── 2. `member_role` del usuario por proyecto (solo memberships activas) ─
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

    // ── 3. `sort_order` por proyecto, del usuario actual ────────────────────
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

    // ── 4. `intake_count` por proyecto (status=-2 PENDING, no soft-deleted) ─
    //
    // Una sola query agrupada en lugar de N+1 (más eficiente que el loop en
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

    // ── 5. Ensamblar + ordenar por (sort_order NULLS LAST, name) como Django ─
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

// ─── DTO extendido para /details ──────────────────────────────────────────────

/// Respuesta extendida de proyecto — espeja `ProjectListSerializer` de Django.
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
    /// Anchor del deploy-board público, si existe.
    pub anchor: Option<String>,
    /// Alias de intake_view para compatibilidad con el frontend.
    pub inbox_view: bool,
    /// Número de intake-issues pendientes (status = -2).
    pub intake_count: i64,
    /// Próximo sequence_id disponible para un nuevo issue.
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

    // ── 1. Proyectos visibles según rol ──────────────────────────────────────
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
            // MEMBER: propios + proyectos públicos (network=2)
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

    // ── 2. Membresías: role del usuario y lista completa de miembros ─────────
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

    // ── 3. Favoritos del usuario ──────────────────────────────────────────────
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

    // ── 4. sort_order del usuario por proyecto ────────────────────────────────
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

    // ── 5. Deploy-board anchor ────────────────────────────────────────────────
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

    // ── 6. intake_count (pending = -2) por proyecto ───────────────────────────
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

    // ── 7. next_work_item_sequence por proyecto ───────────────────────────────
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

    // ── 8. Ensamblar respuestas ───────────────────────────────────────────────
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
    // ── Validaciones de forma (forbidden chars, longitud) ────────────────────
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

    // ── Unicidad de identifier (solo proyectos no soft-deleted) ──────────────
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

    // ── Unicidad de name (espeja ProjectSerializer.validate_name de Django) ──
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
        logo_props: Set(serde_json::json!({})),
        cover_image: Set(None),
        cover_image_asset_id: Set(None),
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

    // Crear membresía Admin para el creador
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

    // Si project_lead es distinto del creador, agregarlo también como Admin
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
    if let Some(v) = body.archive_in { active.archive_in = Set(v); }
    if let Some(v) = body.close_in { active.close_in = Set(v); }
    if let Some(v) = body.guest_view_all_features { active.guest_view_all_features = Set(v); }

    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
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

// ─── Project Members ──────────────────────────────────────────────────────────

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

// ─── Project Invitations ──────────────────────────────────────────────────────

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
