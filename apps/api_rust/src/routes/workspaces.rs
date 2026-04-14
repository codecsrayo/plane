// src/routes/workspaces.rs
//! Endpoints de Workspace — Fase 2.
//!
//! Equivalente a `plane/app/views/workspace/base.py` y `member.py` en Django.
//! Autenticación: sesión cookie **o** API key (via `AnyAuth`).
//! Autorización: membresía activa en el workspace; rol Admin requerido en mutaciones.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        permissions::{require_workspace_admin, ROLE_ADMIN, ROLE_GUEST, ROLE_MEMBER, ROLE_VIEWER},
    },
    entities::{workspace_member_invites, workspace_members, workspaces},
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::{instance_config::get_config_value, soft_delete::SoftDeleteExt},
    AppState,
};

// ─── Alias de permisos ───────────────────────────────────────────────────────

/// Alias local de [`require_workspace_admin`].
///
/// Permite usar `require_admin(&member)?` de forma concisa en todos los
/// handlers de este módulo sin importar un símbolo adicional en cada llamada.
#[inline(always)]
fn require_admin(member: &workspace_members::Model) -> Result<(), AppError> {
    require_workspace_admin(member)
}

// ─── Slugs reservados ────────────────────────────────────────────────────────

const RESTRICTED_SLUGS: &[&str] = &[
    "404", "500", "about", "account", "admin", "api", "auth", "billing", "blog",
    "careers", "changelog", "contact", "docs", "enterprise", "explore", "faq",
    "features", "god-mode", "help", "home", "integrations", "legal", "login",
    "logout", "me", "onboarding", "open", "plane", "pricing", "privacy",
    "profile", "register", "reset-password", "security", "settings", "signup",
    "space", "spaces", "static", "terms", "updates", "workspaces",
];

// ─── Helpers de slug ─────────────────────────────────────────────────────────
//
// `workspace_by_slug` y `require_workspace_member` viven en `helpers.rs`
// y se importan arriba. Solo queda la validación local del formato del slug.

fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.is_empty() || slug.len() > 48 {
        return Err(AppError::BadRequest(
            "Slug must be between 1 and 48 characters".into(),
        ));
    }
    if !slug.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest(
            "Slug can only contain letters, numbers, hyphens and underscores".into(),
        ));
    }
    if RESTRICTED_SLUGS.contains(&slug) {
        return Err(AppError::BadRequest("Slug is reserved".into()));
    }
    Ok(())
}

// ─── DTOs ────────────────────────────────────────────────────────────────────

/// Representación pública de un workspace.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub organization_size: Option<String>,
    pub owner_id: Uuid,
    pub timezone: String,
    pub background_color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Número de miembros activos (excluye bots). `None` en contextos sin join.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_members: Option<i64>,
    /// Rol del usuario autenticado en este workspace (solo en list/detail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i16>,
}

impl WorkspaceResponse {
    fn from_model(
        ws: &workspaces::Model,
        total_members: Option<i64>,
        role: Option<i16>,
    ) -> Self {
        Self {
            id: ws.id,
            name: ws.name.clone(),
            slug: ws.slug.clone(),
            logo: ws.logo.clone(),
            organization_size: ws.organization_size.clone(),
            owner_id: ws.owner_id,
            timezone: ws.timezone.clone(),
            background_color: ws.background_color.clone(),
            created_at: ws.created_at.into(),
            updated_at: ws.updated_at.into(),
            total_members,
            role,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub slug: String,
    pub organization_size: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub organization_size: Option<String>,
    pub timezone: Option<String>,
    pub background_color: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMemberResponse {
    pub id: Uuid,
    pub member_id: Uuid,
    pub role: i16,
    pub company_role: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&workspace_members::Model> for WorkspaceMemberResponse {
    fn from(m: &workspace_members::Model) -> Self {
        Self {
            id: m.id,
            member_id: m.member_id,
            role: m.role,
            company_role: m.company_role.clone(),
            is_active: m.is_active,
            created_at: m.created_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role: i16,
    pub company_role: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub email: String,
    pub role: i16,
    pub accepted: bool,
    pub message: Option<String>,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<&workspace_member_invites::Model> for InvitationResponse {
    fn from(i: &workspace_member_invites::Model) -> Self {
        Self {
            id: i.id,
            email: i.email.clone(),
            role: i.role,
            accepted: i.accepted,
            message: i.message.clone(),
            workspace_id: i.workspace_id,
            created_at: i.created_at.into(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateInvitationRequest {
    pub emails: Vec<InviteEmail>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct InviteEmail {
    pub email: String,
    pub role: i16,
}

#[derive(Debug, Deserialize)]
pub struct SlugCheckQuery {
    pub slug: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SlugCheckResponse {
    pub status: bool,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// `GET /api/workspace-slug-check/?slug={slug}`
///
/// Verifica disponibilidad de un slug. No requiere autenticación.
#[utoipa::path(
    get,
    path = "/api/workspace-slug-check/",
    tag = "Workspaces",
    params(("slug" = String, Query, description = "Slug to check")),
    responses(
        (status = 200, description = "Availability result", body = SlugCheckResponse),
    )
)]
pub async fn slug_check(
    State(state): State<AppState>,
    Query(q): Query<SlugCheckQuery>,
) -> Result<Json<SlugCheckResponse>, AppError> {
    let slug = q.slug.to_lowercase();
    if RESTRICTED_SLUGS.contains(&slug.as_str()) {
        return Ok(Json(SlugCheckResponse { status: false }));
    }
    let exists = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(SlugCheckResponse { status: exists == 0 }))
}

/// `GET /api/workspaces/`
///
/// Lista los workspaces donde el usuario autenticado es miembro activo.
#[utoipa::path(
    get,
    path = "/api/workspaces/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    responses(
        (status = 200, description = "List of workspaces", body = Vec<WorkspaceResponse>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_workspaces(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    // Membresías activas del usuario
    let memberships = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    if memberships.is_empty() {
        return Ok(Json(vec![]));
    }

    let workspace_ids: Vec<Uuid> = memberships.iter().map(|m| m.workspace_id).collect();
    let member_role_map: std::collections::HashMap<Uuid, i16> = memberships
        .iter()
        .map(|m| (m.workspace_id, m.role))
        .collect();

    let workspaces_list = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Id.is_in(workspace_ids))
        .order_by_asc(workspaces::Column::Name)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let responses: Vec<WorkspaceResponse> = workspaces_list
        .iter()
        .map(|ws| {
            let role = member_role_map.get(&ws.id).copied();
            WorkspaceResponse::from_model(ws, None, role)
        })
        .collect();

    Ok(Json(responses))
}

/// `POST /api/workspaces/`
///
/// Crea un nuevo workspace y registra al usuario como Admin + Owner.
#[utoipa::path(
    post,
    path = "/api/workspaces/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Workspace creation disabled"),
    )
)]
pub async fn create_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verificar flag de instancia
    let disabled = get_config_value(&state, "DISABLE_WORKSPACE_CREATION", Some("0"))
        .await?
        .unwrap_or_default();
    if disabled == "1" {
        return Err(AppError::Forbidden);
    }

    if body.name.is_empty() || body.name.len() > 80 {
        return Err(AppError::BadRequest(
            "Name must be between 1 and 80 characters".into(),
        ));
    }
    let slug = body.slug.to_lowercase();
    validate_slug(&slug)?;

    // Verificar slug disponible
    let exists = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;
    if exists > 0 {
        return Err(AppError::BadRequest("Slug already taken".into()));
    }

    let ws_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    // Crear workspace
    let new_ws = workspaces::ActiveModel {
        id: Set(ws_id),
        name: Set(body.name.clone()),
        slug: Set(slug),
        logo: Set(None),
        organization_size: Set(body.organization_size.clone()),
        owner_id: Set(user.id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        timezone: Set(body.timezone.unwrap_or_else(|| "UTC".into())),
        background_color: Set(String::new()),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        logo_asset_id: Set(None),
    };
    let ws = new_ws.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert workspace");
        AppError::Database(e)
    })?;

    // Crear membresía Admin
    let new_member = workspace_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        workspace_id: Set(ws_id),
        member_id: Set(user.id),
        role: Set(ROLE_ADMIN),
        company_role: Set(None),
        is_active: Set(true),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        view_props: Set(serde_json::json!({})),
        default_props: Set(serde_json::json!({})),
        issue_props: Set(serde_json::json!({})),
        explored_features: Set(serde_json::json!({})),
        getting_started_checklist: Set(serde_json::json!({})),
        tips: Set(serde_json::json!({})),
    };
    new_member.insert(&txn).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to insert workspace member");
        AppError::Database(e)
    })?;

    txn.commit().await.map_err(AppError::Database)?;

    let resp = WorkspaceResponse::from_model(&ws, Some(1), Some(ROLE_ADMIN));
    Ok((StatusCode::CREATED, Json(resp)))
}

/// `GET /api/workspaces/{slug}/`
///
/// Retorna el detalle del workspace. Requiere membresía activa.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Workspace detail", body = WorkspaceResponse),
        (status = 403, description = "Not a member"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;

    let total = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(WorkspaceResponse::from_model(
        &ws,
        Some(total as i64),
        Some(member.role),
    )))
}

/// `PATCH /api/workspaces/{slug}/`
///
/// Actualiza nombre u otros campos. Requiere rol Admin.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    request_body = UpdateWorkspaceRequest,
    responses(
        (status = 200, description = "Updated workspace", body = WorkspaceResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    if let Some(ref name) = body.name {
        if name.is_empty() || name.len() > 80 {
            return Err(AppError::BadRequest(
                "Name must be between 1 and 80 characters".into(),
            ));
        }
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspaces::ActiveModel = ws.into();
    if let Some(name) = body.name {
        active.name = Set(name);
    }
    if let Some(org_size) = body.organization_size {
        active.organization_size = Set(Some(org_size));
    }
    if let Some(tz) = body.timezone {
        active.timezone = Set(tz);
    }
    if let Some(bg) = body.background_color {
        active.background_color = Set(bg);
    }
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(WorkspaceResponse::from_model(
        &updated,
        None,
        Some(member.role),
    )))
}

/// `DELETE /api/workspaces/{slug}/`
///
/// Soft-delete del workspace. Solo el owner puede eliminarlo.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 204, description = "Workspace deleted"),
        (status = 403, description = "Forbidden — only owner can delete"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_workspace(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    // Solo el owner puede eliminar el workspace (equivalente Django)
    if ws.owner_id != user.id {
        let member = require_workspace_member(&state.db, ws.id, user.id).await?;
        require_admin(&member)?;
        // Admin no owner puede eliminar solo si tiene permiso — aquí solo owner
        return Err(AppError::Forbidden);
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspaces::ActiveModel = ws.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Members ─────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/members/`
///
/// Lista miembros activos del workspace.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/members/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Member list", body = Vec<WorkspaceMemberResponse>),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<WorkspaceMemberResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    require_workspace_member(&state.db, ws.id, user.id).await?;

    let members = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .order_by_asc(workspace_members::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(members.iter().map(WorkspaceMemberResponse::from).collect()))
}

/// `PATCH /api/workspaces/{slug}/members/{pk}/`
///
/// Actualiza rol de un miembro. Requiere Admin.
/// No se puede degradar al owner.
#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/members/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Member record UUID"),
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Updated member", body = WorkspaceMemberResponse),
        (status = 400, description = "Cannot downgrade owner"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Member not found"),
    )
)]
pub async fn update_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateMemberRoleRequest>,
) -> Result<Json<WorkspaceMemberResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&caller)?;

    // Validar rol
    if ![ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&body.role) {
        return Err(AppError::BadRequest("Invalid role value".into()));
    }

    let target = workspace_members::Entity::find_by_id(pk)
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // No degradar al owner del workspace
    if target.member_id == ws.owner_id && body.role < ROLE_ADMIN {
        return Err(AppError::BadRequest(
            "Cannot downgrade workspace owner".into(),
        ));
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspace_members::ActiveModel = target.into();
    active.role = Set(body.role);
    if let Some(cr) = body.company_role {
        active.company_role = Set(Some(cr));
    }
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(WorkspaceMemberResponse::from(&updated)))
}

/// `DELETE /api/workspaces/{slug}/members/{pk}/`
///
/// Desactiva un miembro (soft-remove). Requiere Admin.
/// El owner no puede ser removido.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/members/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Member record UUID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 400, description = "Cannot remove owner"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Member not found"),
    )
)]
pub async fn remove_member(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&caller)?;

    let target = workspace_members::Entity::find_by_id(pk)
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if target.member_id == ws.owner_id {
        return Err(AppError::BadRequest(
            "Cannot remove the workspace owner".into(),
        ));
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspace_members::ActiveModel = target.into();
    // Marcar inactivo + soft-delete para consistencia
    active.is_active = Set(false);
    active.deleted_at = Set(Some(now));
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Invitations ─────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/invitations/`
///
/// Lista invitaciones pendientes del workspace. Requiere Admin.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/invitations/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Invitation list", body = Vec<InvitationResponse>),
        (status = 403, description = "Forbidden — requires Admin"),
    )
)]
pub async fn list_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<InvitationResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    let invites = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::Accepted.eq(false))
        .order_by_desc(workspace_member_invites::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(invites.iter().map(InvitationResponse::from).collect()))
}

/// `POST /api/workspaces/{slug}/invitations/`
///
/// Crea una o varias invitaciones al workspace. Requiere Admin.
///
/// Nota: el envío de email se delegará al job `InvitationEmailJob` (Fase 3).
/// Por ahora el registro se persiste y el token queda en la BD.
#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/invitations/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    request_body = CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitations created", body = Vec<InvitationResponse>),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Forbidden — requires Admin"),
    )
)]
pub async fn create_invitations(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
    Json(body): Json<CreateInvitationRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.emails.is_empty() {
        return Err(AppError::BadRequest("emails list is required".into()));
    }

    // ── 1. Validación de roles — upfront, antes de cualquier query de BD ─────
    //
    // Fallar rápido evita que una solicitud parcialmente inválida emita queries
    // o inserciones antes de descubrir el error.
    for invite_req in &body.emails {
        if ![ROLE_GUEST, ROLE_VIEWER, ROLE_MEMBER, ROLE_ADMIN].contains(&invite_req.role) {
            return Err(AppError::BadRequest(format!(
                "Invalid role {} for email {}",
                invite_req.role, invite_req.email
            )));
        }
    }

    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    // ── 2. Deduplicación en memoria del payload ───────────────────────────────
    //
    // Un mismo email listado dos veces en la request no debe producir dos rows;
    // normalizar a lowercase para comparación case-insensitive (igual que Django).
    let mut seen_in_payload: HashSet<String> = HashSet::with_capacity(body.emails.len());
    let unique_invites: Vec<&InviteEmail> = body.emails
        .iter()
        .filter(|i| seen_in_payload.insert(i.email.to_lowercase()))
        .collect();

    // ── 3. Bulk-check — 1 SELECT reemplaza N COUNT queries ───────────────────
    //
    // Antes: por cada email → COUNT(*) WHERE email = ? (N queries)
    // Ahora: 1 query → SELECT email WHERE email IN (e1, e2, …) AND accepted = false
    let candidate_emails: Vec<String> =
        unique_invites.iter().map(|i| i.email.clone()).collect();

    let already_invited: HashSet<String> = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::Email.is_in(candidate_emails))
        .filter(workspace_member_invites::Column::Accepted.eq(false))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|m| m.email.to_lowercase())
        .collect();

    // ── 4. Construir ActiveModels y Models en memoria ─────────────────────────
    //
    // Los UUIDs se generan localmente — no se necesita `exec_with_returning`
    // ni una segunda query para recuperar las filas recién insertadas.
    let now = chrono::Utc::now().fixed_offset();
    let mut active_models: Vec<workspace_member_invites::ActiveModel> =
        Vec::with_capacity(unique_invites.len());
    let mut created_models: Vec<workspace_member_invites::Model> =
        Vec::with_capacity(unique_invites.len());

    for invite_req in &unique_invites {
        if already_invited.contains(&invite_req.email.to_lowercase()) {
            tracing::warn!(
                email = %invite_req.email,
                workspace = %ws.slug,
                "Skipping duplicate pending invitation"
            );
            continue;
        }

        let id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();

        active_models.push(workspace_member_invites::ActiveModel {
            id: Set(id),
            workspace_id: Set(ws.id),
            email: Set(invite_req.email.clone()),
            role: Set(invite_req.role),
            accepted: Set(false),
            token: Set(token.clone()),
            message: Set(None),
            responded_at: Set(None),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        });

        // Model local — refleja exactamente lo que se va a persistir.
        created_models.push(workspace_member_invites::Model {
            id,
            workspace_id: ws.id,
            email: invite_req.email.clone(),
            role: invite_req.role,
            accepted: false,
            token,
            message: None,
            responded_at: None,
            created_by_id: Some(user.id),
            updated_by_id: Some(user.id),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        });
    }

    // ── 5. Bulk insert en transacción — 1 INSERT … VALUES (…), (…) ───────────
    //
    // Si no hay nada nuevo que insertar (todos eran duplicados) se omite la
    // transacción por completo.
    if !active_models.is_empty() {
        let txn = state.db.begin().await.map_err(AppError::Database)?;
        workspace_member_invites::Entity::insert_many(active_models)
            .exec(&txn)
            .await
            .map_err(AppError::Database)?;
        txn.commit().await.map_err(AppError::Database)?;
    }

    let responses: Vec<InvitationResponse> =
        created_models.iter().map(InvitationResponse::from).collect();
    Ok((StatusCode::CREATED, Json(responses)))
}

/// `DELETE /api/workspaces/{slug}/invitations/{pk}/`
///
/// Soft-delete de una invitación. Requiere Admin.
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/invitations/{pk}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk"   = Uuid,   Path, description = "Invitation UUID"),
    ),
    responses(
        (status = 204, description = "Invitation deleted"),
        (status = 403, description = "Forbidden — requires Admin"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_invitation(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let member = require_workspace_member(&state.db, ws.id, user.id).await?;
    require_admin(&member)?;

    let invite = workspace_member_invites::Entity::find_by_id(pk)
        .filter(workspace_member_invites::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_member_invites::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: workspace_member_invites::ActiveModel = invite.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
