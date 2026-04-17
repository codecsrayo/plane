// src/routes/workspaces.rs
// NOTE: get_user_profile added below — mirror of WorkspaceUserProfileEndpoint
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
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult,
    PaginatorTrait, QueryFilter, QueryOrder, Set, Statement, TransactionTrait,
};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        permissions::{require_workspace_admin, ROLE_ADMIN, ROLE_GUEST, ROLE_MEMBER, ROLE_VIEWER},
    },
    entities::{
        draft_issues, issue_activities, issues, profiles, project_members, projects, users,
        workspace_member_invites, workspace_members, workspaces,
    },
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::{
        color::get_random_color, instance_config::get_config_value,
        pagination,
        posthog::{track_event, EVENT_WORKSPACE_DELETED},
        soft_delete::SoftDeleteExt, url::contains_url,
    },
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

/// Slugs reservados — espejo exacto de `plane/utils/constants.py::RESTRICTED_WORKSPACE_SLUGS`.
/// Actualizar aquí cuando se modifique el archivo Python.
const RESTRICTED_SLUGS: &[&str] = &[
    "404",
    "accounts",
    "api",
    "create-workspace",
    "god-mode",
    "installations",
    "invitations",
    "onboarding",
    "profile",
    "spaces",
    "workspace-invitations",
    "password",
    "flags",
    "monitor",
    "monitoring",
    "ingest",
    "plane-pro",
    "plane-ultimate",
    "enterprise",
    "plane-enterprise",
    "disco",
    "silo",
    "chat",
    "calendar",
    "drive",
    "channels",
    "upgrade",
    "billing",
    "sign-in",
    "sign-up",
    "signin",
    "signup",
    "config",
    "live",
    "admin",
    "m",
    "import",
    "importers",
    "integrations",
    "integration",
    "configuration",
    "initiatives",
    "initiative",
    "workflow",
    "workflows",
    "epics",
    "epic",
    "story",
    "mobile",
    "dashboard",
    "desktop",
    "onload",
    "real-time",
    "one",
    "pages",
    "business",
    "pro",
    "settings",
    "license",
    "licenses",
    "instances",
    "instance",
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
    /// Computed logo URL: prefers logo_asset, falls back to logo field.
    /// Mirrors Django's `Workspace.logo_url` property.
    pub logo_url: Option<String>,
    pub organization_size: Option<String>,
    pub owner_id: Uuid,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
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
    /// Build response from a workspace model.
    ///
    /// `logo_url` mirrors Django's computed property:
    ///   - If `logo_asset` exists → use its `asset_url` (resolved via join)
    ///   - Else if `logo` is set → use it directly
    ///   - Else → `None`
    ///
    /// When the caller hasn't performed a join on `file_assets`, pass `None`
    /// for `logo_asset_url` and the function falls back to `ws.logo`.
    fn from_model(
        ws: &workspaces::Model,
        total_members: Option<i64>,
        role: Option<i16>,
        logo_asset_url: Option<String>,
    ) -> Self {
        // Compute logo_url mirroring Django: logo_asset > logo > None
        let logo_url = logo_asset_url.or_else(|| ws.logo.clone());

        Self {
            id: ws.id,
            name: ws.name.clone(),
            slug: ws.slug.clone(),
            logo: ws.logo.clone(),
            logo_url,
            organization_size: ws.organization_size.clone(),
            owner_id: ws.owner_id,
            created_by: ws.created_by_id,
            updated_by: ws.updated_by_id,
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
    /// Logo URL — mirrors Django's `logo` TextField.
    pub logo: Option<String>,
    /// Company role of the creating user — stored on WorkspaceMember.
    /// Mirrors Django: `request.data.get("company_role", "")`.
    pub company_role: Option<String>,
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

// ─── DTOs Django-compat para listado de miembros ─────────────────────────────
//
// Django expone los miembros del workspace con el `member` ANIDADO como objeto
// de usuario (ver `WorkSpaceMemberSerializer` en
// `apps/api/plane/app/serializers/workspace.py:85-90` con
// `member = UserLiteSerializer(read_only=True)`). El frontend consume este
// shape directamente en `workspace-member.store.ts:240`:
//
//     set(this.memberRoot?.memberMap, member.member.id, { ...member.member, ... });
//
// El DTO flat de arriba (`WorkspaceMemberResponse`) NO es compatible con ese
// acceso a `member.member.id` y causa
//   TypeError: Cannot read properties of undefined (reading 'id')
// cuando el frontend apunta al Rust. Estos DTOs nuevos reflejan exactamente la
// salida del serializer Django con `fields=("id", "member", "role")`.

/// Mirror de `UserLiteSerializer` + rama admin de `UserAdminLiteSerializer`
/// (`apps/api/plane/app/serializers/user.py:141-170`).
///
/// `email` y `last_login_medium` solo se emiten cuando el caller es no-Guest
/// (Django: `if workspace_member.role > 5` en
/// `apps/api/plane/app/views/workspace/member.py:51`). `skip_serializing_if`
/// mantiene el shape JSON idéntico al de Django cuando el caller es Guest —
/// las claves no aparecen, no se mandan como `null`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserLiteDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub display_name: String,
    /// Solo presente si el caller es no-Guest (paridad con `UserAdminLiteSerializer`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Solo presente si el caller es no-Guest (paridad con `UserAdminLiteSerializer`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_medium: Option<String>,
}

/// Construye un `UserLiteDto` a partir del modelo SeaORM, aplicando la misma
/// lógica de `avatar_url` que `User.avatar_url` en Django
/// (`apps/api/plane/db/models/user.py:142-151`):
///
///   1. Si hay `avatar_asset_id`, devuelve `/api/assets/v2/static/{id}/`.
///      Este path es el branch `USER_AVATAR` de `FileAsset.asset_url`
///      (`apps/api/plane/db/models/asset.py:79-100`), por lo que se compone
///      directamente desde el UUID sin necesidad de un JOIN a `file_assets` —
///      el `entity_type` del asset apuntado por `users.avatar_asset_id` es
///      siempre `USER_AVATAR` por invariante del modelo Django.
///   2. Si no, devuelve el string legacy `avatar` si no está vacío.
///   3. En cualquier otro caso, `None`.
///
/// `is_admin` activa los campos que `UserAdminLiteSerializer` añade sobre
/// `UserLiteSerializer`.
pub(crate) fn user_to_lite(user: &users::Model, is_admin: bool) -> UserLiteDto {
    let avatar_url = if let Some(asset_id) = user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !user.avatar.is_empty() {
        Some(user.avatar.clone())
    } else {
        None
    };

    UserLiteDto {
        id: user.id,
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        avatar: user.avatar.clone(),
        avatar_url,
        is_bot: user.is_bot,
        display_name: user.display_name.clone(),
        email: if is_admin { user.email.clone() } else { None },
        last_login_medium: if is_admin {
            Some(user.last_login_medium.clone())
        } else {
            None
        },
    }
}

/// Mirror de `WorkSpaceMemberSerializer` con `fields=("id","member","role")`
/// (uso explícito en
/// `apps/api/plane/app/views/workspace/member.py:52,54,71,73`). NO incluye
/// `company_role`, `is_active`, `created_at`, etc. — el serializer Django con
/// esa whitelist tampoco los emite, y emitir campos extra rompería consumidores
/// que hacen `{ ...member }` spread (cualquier campo extra pisaría propiedades
/// del store MobX).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMemberNestedResponse {
    pub id: Uuid,
    pub member: UserLiteDto,
    pub role: i16,
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
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
    // No usar .active() aquí: la unique constraint de la BD aplica sobre TODAS
    // las filas (incluyendo soft-deleted).  Si filtramos solo activas, reportamos
    // el slug como disponible pero el INSERT real falla por la constraint.
    let exists = workspaces::Entity::find()
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
            WorkspaceResponse::from_model(ws, None, role, None)
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
        (status = 409, description = "Slug already exists"),
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
    if contains_url(&body.name) {
        return Err(AppError::BadRequest(
            "Name cannot contain a URL".into(),
        ));
    }
    let slug = body.slug.to_lowercase();
    validate_slug(&slug)?;

    // No hacemos pre-check de slug disponible: es un patrón TOCTOU (time-of-check
    // vs time-of-use). Dos requests concurrentes pueden ambas pasar la verificación
    // y una falla en el INSERT.  Dejamos que la unique constraint de la BD sea la
    // fuente de verdad y mapeamos la violación a 409 Conflict (como Django).

    let ws_id = Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    // Crear workspace
    let new_ws = workspaces::ActiveModel {
        id: Set(ws_id),
        name: Set(body.name.clone()),
        slug: Set(slug),
        logo: Set(body.logo.clone()),
        organization_size: Set(body.organization_size.clone()),
        owner_id: Set(user.id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        timezone: Set(body.timezone.unwrap_or_else(|| "UTC".into())),
        background_color: Set(get_random_color()),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        logo_asset_id: Set(None),
    };
    let ws = new_ws.insert(&txn).await.map_err(|e| {
        // Mapear unique constraint violation → 409 Conflict (espejo de Django)
        if let sea_orm::DbErr::Query(ref runtime_err) = e {
            let msg = runtime_err.to_string();
            if msg.contains("unique") || msg.contains("duplicate key") {
                return AppError::Conflict(
                    "The workspace with the slug already exists".into(),
                );
            }
        }
        // Capturar también Exec errors que SeaORM puede emitir en insert
        if let sea_orm::DbErr::Exec(ref runtime_err) = e {
            let msg = runtime_err.to_string();
            if msg.contains("unique") || msg.contains("duplicate key") {
                return AppError::Conflict(
                    "The workspace with the slug already exists".into(),
                );
            }
        }
        tracing::error!(error = %e, "Failed to insert workspace");
        AppError::Database(e)
    })?;

    // Crear membresía Admin — mirrors Django: role=20, company_role from request
    let new_member = workspace_members::ActiveModel {
        id: Set(Uuid::new_v4()),
        workspace_id: Set(ws_id),
        member_id: Set(user.id),
        role: Set(ROLE_ADMIN),
        company_role: Set(body.company_role.clone()),
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

    // Encolar siembra de datos iniciales — best-effort (no bloquea la respuesta).
    // Uses the shared pg_pool from AppState instead of opening a new connection.
    {
        use crate::jobs::workspace_seed::WorkspaceSeedJob;
        use apalis::prelude::Storage;
        use apalis_sql::postgres::PostgresStorage;

        let mut seed_storage: PostgresStorage<WorkspaceSeedJob> =
            PostgresStorage::new(state.pg_pool.clone());
        if let Err(e) = seed_storage
            .push(WorkspaceSeedJob {
                workspace_id: ws_id,
                owner_id: user.id,
                workspace_name: body.name.clone(),
            })
            .await
        {
            tracing::warn!(error = %e, "Failed to enqueue workspace seed job");
        }
    }

    let resp = WorkspaceResponse::from_model(&ws, Some(1), Some(ROLE_ADMIN), None);
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
        None,
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
        if contains_url(name) {
            return Err(AppError::BadRequest(
                "Name cannot contain a URL".into(),
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
        None,
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
    let ws_id = ws.id;
    let ws_name = ws.name.clone();
    let ws_slug = ws.slug.clone();

    // Espejo de `WorkspaceViewSet.destroy`
    // (`apps/api/plane/app/views/workspace/base.py:184-201`):
    //
    //   Profile.objects.filter(last_workspace_id=id).update(last_workspace_id=None)
    //   return super().destroy(...)
    //
    // Sin este paso, los perfiles que apuntaban al workspace eliminado siguen
    // apuntando a él — `GET /users/me/profile/` devuelve el ID muerto vía
    // `profile_to_response` (users.rs) y el frontend redirige al href muerto
    // en lugar de a la pantalla de selección de workspace. Se envuelve en
    // transacción para que la limpieza de perfiles y el soft-delete del
    // workspace sean atómicos frente a lectores concurrentes.
    //
    // Nota de paridad: Django usa `QuerySet.update()`, que explícitamente
    // **no** dispara `auto_now=True` sobre `updated_at`
    // (`apps/api/plane/db/mixins.py:20`). Los perfiles afectados conservan
    // su `updated_at` anterior. Replicamos ese comportamiento: solo se toca
    // `last_workspace_id`, no `updated_at`.
    let txn = state.db.begin().await.map_err(AppError::Database)?;

    profiles::Entity::update_many()
        .col_expr(
            profiles::Column::LastWorkspaceId,
            Expr::value(Option::<Uuid>::None),
        )
        .filter(profiles::Column::LastWorkspaceId.eq(ws_id))
        .exec(&txn)
        .await
        .map_err(AppError::Database)?;

    let mut active: workspaces::ActiveModel = ws.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&txn).await.map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    // Espejo de `track_event.delay(event_name=WORKSPACE_DELETED, ...)` en
    // `base.py:188-200`. Django lo corre via Celery; nosotros vía `tokio::spawn`
    // dentro de `utils::posthog::track_event`. El evento se emite sólo tras
    // commit exitoso — si la transacción hubiera fallado, no queremos reportar
    // un delete que no sucedió.
    //
    // Nota sobre `role`: Django hardcodea `"role": "owner"` en la view y luego
    // `preprocess_data_properties` lo recalcula consultando `Workspace.objects`
    // con el slug. Pero para este punto el workspace ya está soft-deleted, y el
    // manager por defecto filtra por `deleted_at IS NULL` → `DoesNotExist` →
    // `"role": "unknown"`. En Rust somos deterministas: ya validamos arriba
    // que `ws.owner_id == user.id`, así que el role siempre es `owner`.
    let mut props = serde_json::Map::new();
    props.insert("user_id".into(), serde_json::json!(user.id.to_string()));
    props.insert("workspace_id".into(), serde_json::json!(ws_id.to_string()));
    props.insert("workspace_slug".into(), serde_json::json!(ws_slug));
    props.insert("workspace_name".into(), serde_json::json!(ws_name));
    props.insert("role".into(), serde_json::json!("owner"));
    props.insert("deleted_at".into(), serde_json::json!(now.to_rfc3339()));
    track_event(
        &state,
        user.id,
        EVENT_WORKSPACE_DELETED,
        slug,
        props,
    );

    Ok(StatusCode::NO_CONTENT)
}

// ─── Members ─────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/members/`
///
/// Lista miembros activos del workspace con el usuario anidado.
///
/// Mirror exacto de `WorkSpaceMemberViewSet.list`
/// (`apps/api/plane/app/views/workspace/member.py:45-55`). Shape de respuesta
/// idéntico al de Django con `WorkSpaceMemberSerializer(fields=("id","member",
/// "role"))` — necesario porque el frontend (`workspace-member.store.ts:240`)
/// accede a `member.member.id` y falla con TypeError si el shape no es nested.
///
/// El branch admin/no-admin sigue a Django: `if workspace_member.role > 5`
/// (ROLE_GUEST) usa `UserAdminLiteSerializer` (incluye `email`/
/// `last_login_medium`); caso contrario `UserLiteSerializer`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/members/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Member list", body = Vec<WorkspaceMemberNestedResponse>),
        (status = 403, description = "Not a member"),
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<Vec<WorkspaceMemberNestedResponse>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let caller = require_workspace_member(&state.db, ws.id, user.id).await?;

    // Paridad Django: `if workspace_member.role > 5` usa AdminSerializer.
    // ROLE_GUEST = 5 en plane/app/permissions/base.py y en auth/permissions.rs.
    let is_admin = caller.role > ROLE_GUEST;

    // ── 1. Fetch de memberships ───────────────────────────────────────────────
    //
    // Django: `.filter(workspace__slug=self.kwargs.get("slug"))` sobre el
    // queryset base del viewset + list-view adicional filtra por is_active=True.
    // `.active()` aplica el filtro soft-delete (deleted_at IS NULL).
    let members = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .order_by_asc(workspace_members::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    if members.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // ── 2. Batch-fetch de usuarios referenciados ──────────────────────────────
    //
    // Dos queries total (no N+1). Django hace lo equivalente con
    // `select_related("member", "member__avatar_asset")` — nosotros resolvemos
    // avatar_url sin JOIN adicional a `file_assets` porque el path se computa
    // directo desde `avatar_asset_id` (ver `user_to_lite`). Deduplicamos via
    // HashSet por si hubiera duplicados inesperados (no debería, la unique
    // constraint parcial lo impide, pero cuesta poco protegerse).
    let member_ids: Vec<Uuid> = members
        .iter()
        .map(|m| m.member_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let users_vec = users::Entity::find()
        .filter(users::Column::Id.is_in(member_ids))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let users_by_id: std::collections::HashMap<Uuid, users::Model> =
        users_vec.into_iter().map(|u| (u.id, u)).collect();

    // ── 3. Ensamblado del shape Django-compatible ─────────────────────────────
    //
    // Django hace INNER JOIN vía `select_related("member")`: si un user fue
    // hard-deleted pero la membership quedó huérfana, la fila desaparece del
    // resultset (comportamiento implícito de join required). Replicamos con
    // `filter_map`: si no hay entrada en `users_by_id`, se omite la fila. Esto
    // evita devolver `member: null` al frontend (que reventaría igual en
    // `member.member.id`).
    let response: Vec<WorkspaceMemberNestedResponse> = members
        .iter()
        .filter_map(|m| {
            users_by_id.get(&m.member_id).map(|u| WorkspaceMemberNestedResponse {
                id: m.id,
                member: user_to_lite(u, is_admin),
                role: m.role,
            })
        })
        .collect();

    Ok(Json(response))
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

// ─── GET /api/workspaces/{slug}/workspace-members/me ─────────────────────────
//
// Espejo de Django `WorkspaceMemberUserEndpoint.get` en
// `plane/app/views/workspace/member.py:217`. Devuelve la membresía completa
// del usuario autenticado en el workspace, con un campo extra
// `draft_issue_count` que el frontend usa para badges de "Drafts".
//
// Comportamiento Django: si el usuario NO tiene una membresía activa,
// `WorkspaceMemberMeSerializer(None).data` produce un objeto vacío `{}` con
// status 200 — no 404 ni 403. Replicamos exactamente para no romper al
// frontend que asume 200 estable y solo lee campos opcionales.

/// Respuesta de `GET /api/workspaces/{slug}/workspace-members/me`.
///
/// Mirror de `WorkspaceMemberMeSerializer(model=WorkspaceMember, fields="__all__")`
/// más el campo anotado `draft_issue_count`. Todos los campos JSONB
/// (`view_props`, `default_props`, etc.) se exponen tal cual los almacena
/// Postgres, igual que en Django.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMemberMeResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: i16,
    pub company_role: Option<String>,
    pub view_props: serde_json::Value,
    pub default_props: serde_json::Value,
    pub issue_props: serde_json::Value,
    pub is_active: bool,
    pub explored_features: serde_json::Value,
    pub getting_started_checklist: serde_json::Value,
    pub tips: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub member: Uuid,
    pub workspace: Uuid,
    pub draft_issue_count: u64,
}

/// `GET /api/workspaces/{slug}/workspace-members/me`
///
/// Devuelve la membresía del usuario autenticado en el workspace, o un
/// objeto vacío si no es miembro activo (compat Django).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-members/me",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Current user's workspace membership (or empty object if not a member)"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn get_workspace_member_me(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let membership = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(m) = membership else {
        // Compat Django: serializer sobre None produce {} con status 200.
        return Ok(Json(serde_json::json!({})));
    };

    // draft_issue_count: borradores creados por el usuario en este workspace
    // (no soft-deleted). Espejo de la subquery anotada en Django:
    //   DraftIssue.objects.filter(created_by=request.user,
    //                             workspace_id=OuterRef("workspace_id"))
    //     .annotate(count=Count("id"))
    let draft_issue_count = draft_issues::Entity::find()
        .active()
        .filter(draft_issues::Column::CreatedById.eq(user.id))
        .filter(draft_issues::Column::WorkspaceId.eq(ws.id))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp = WorkspaceMemberMeResponse {
        id: m.id,
        created_at: m.created_at.into(),
        updated_at: m.updated_at.into(),
        role: m.role,
        company_role: m.company_role,
        view_props: m.view_props,
        default_props: m.default_props,
        issue_props: m.issue_props,
        is_active: m.is_active,
        explored_features: m.explored_features,
        getting_started_checklist: m.getting_started_checklist,
        tips: m.tips,
        created_by: m.created_by_id,
        updated_by: m.updated_by_id,
        member: m.member_id,
        workspace: m.workspace_id,
        draft_issue_count,
    };

    Ok(Json(serde_json::to_value(resp).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("serialize WorkspaceMemberMeResponse: {e}"))
    })?))
}

// ─── User Profile ─────────────────────────────────────────────────────────────
//
// Mirror de `WorkspaceUserProfileEndpoint.get`
// (`apps/api/plane/app/views/workspace/user.py:280`).
//
// Shape de respuesta:
//   { project_data: [...], user_data: { email, first_name, ... } }
//
// `project_data` solo se incluye cuando `requesting_workspace_member.role >= 15`
// (MEMBER+). Cada entrada contiene contadores de issues del target user en ese
// proyecto.  Los proyectos filtrados son aquellos donde el REQUESTER es miembro
// activo (no el target) — igual que en Django.

/// Fila de resultado del SQL de estadísticas por proyecto.
///
/// `logo_props` se recupera como `String` (JSON serializado) porque SeaORM
/// no implementa `FromQueryResult` para `serde_json::Value` directamente en
/// consultas raw. Se deserializa en el ensamblado del DTO.
#[derive(Debug, FromQueryResult)]
struct ProjectProfileRow {
    id: Uuid,
    logo_props: String,
    created_issues: i64,
    assigned_issues: i64,
    completed_issues: i64,
    pending_issues: i64,
}

/// Entrada de `project_data` en la respuesta final.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProjectProfileData {
    pub id: Uuid,
    pub logo_props: serde_json::Value,
    pub created_issues: i64,
    pub assigned_issues: i64,
    pub completed_issues: i64,
    pub pending_issues: i64,
}

/// Datos del usuario target en la respuesta.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProfileData {
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub date_joined: DateTime<Utc>,
    pub user_timezone: String,
    pub display_name: String,
}

/// Respuesta de `GET /api/workspaces/{slug}/user-profile/{user_id}/`.
///
/// Mirror exacto de `WorkspaceUserProfileEndpoint.get` en Django.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProfileResponse {
    /// Estadísticas por proyecto — vacío si el requester es Guest/Viewer.
    pub project_data: Vec<ProjectProfileData>,
    pub user_data: UserProfileData,
}

/// `GET /api/workspaces/{slug}/user-profile/{user_id}/`
///
/// Devuelve el perfil de un usuario en el contexto del workspace:
/// datos personales + estadísticas de issues por proyecto.
///
/// `project_data` solo se popula cuando el requester tiene rol >= Member (15).
/// Los proyectos devueltos son aquellos donde el **requester** es miembro activo,
/// y los contadores de issues refieren al usuario **target** (`user_id`).
///
/// Mirror de `WorkspaceUserProfileEndpoint`
/// (`apps/api/plane/app/views/workspace/user.py:280`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-profile/{user_id}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"    = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid,   Path, description = "Target user UUID"),
    ),
    responses(
        (status = 200, description = "User profile", body = UserProfileResponse),
        (status = 403, description = "Not a workspace member"),
        (status = 404, description = "User or workspace not found"),
    )
)]
pub async fn get_user_profile(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;
    let requester = require_workspace_member(&state.db, ws.id, user.id).await?;

    // ── 1. Fetch del usuario target ───────────────────────────────────────────
    //
    // Django: `User.objects.get(pk=user_id)` — lanza 404 si no existe.
    let target_user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // ── 2. Compute avatar_url — mirror de User.avatar_url (Django) ───────────
    //
    // Django: `avatar_asset.asset_url` > `avatar` > None
    // Path asset: `/api/assets/v2/static/{id}/` (entity_type = USER_AVATAR)
    let avatar_url = if let Some(asset_id) = target_user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !target_user.avatar.is_empty() {
        Some(target_user.avatar.clone())
    } else {
        None
    };

    // ── 3. Compute cover_image_url — mirror de User.cover_image_url (Django) ─
    let cover_image_url = if let Some(asset_id) = target_user.cover_image_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else {
        target_user.cover_image.clone()
    };

    let user_data = UserProfileData {
        email: target_user.email.clone(),
        first_name: target_user.first_name.clone(),
        last_name: target_user.last_name.clone(),
        avatar_url,
        cover_image_url,
        date_joined: target_user.date_joined.into(),
        user_timezone: target_user.user_timezone.clone(),
        display_name: target_user.display_name.clone(),
    };

    // ── 4. Project stats — solo si requester.role >= MEMBER (15) ────────────
    //
    // Django: `if requesting_workspace_member.role >= 15`
    // Roles: Guest=5, Viewer=10, Member=15, Admin=20
    let project_data = if requester.role >= ROLE_MEMBER {
        let ws_id = ws.id;

        // Una sola query agrega todos los contadores por proyecto, evitando
        // N+1 queries. Subqueries correlacionadas se reemplazan por LEFT JOINs
        // sobre subqueries agrupadas — mismo plan que si Django hiciera
        // `annotate(Count(...))` en bulk.
        //
        // Filtro de proyectos: archivados = false, el REQUESTER es miembro activo.
        // Contadores: issues del TARGET user (created / assigned / completed / pending).
        //
        // Placeholders: $1 = requester_id, $2 = user_id, $3 = ws_id.
        // Se usan parámetros bindados en vez de interpolar vía `format!` para
        // blindar contra SQL injection (defensa en profundidad). `group` se
        // escapa como `"group"` por ser palabra reservada en PostgreSQL.
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT
                p.id                                  AS id,
                p.logo_props::text                    AS logo_props,
                COALESCE(ci.cnt,    0)                AS created_issues,
                COALESCE(ai.cnt,    0)                AS assigned_issues,
                COALESCE(compi.cnt, 0)                AS completed_issues,
                COALESCE(pi2.cnt,   0)                AS pending_issues
            FROM projects p
            JOIN project_members pm
                ON  pm.project_id  = p.id
                AND pm.member_id   = $1
                AND pm.is_active   = true
                AND pm.deleted_at  IS NULL

            -- created_issues: issues creadas por el target user en el proyecto
            LEFT JOIN (
                SELECT project_id, COUNT(*) AS cnt
                FROM   issues
                WHERE  created_by_id = $2
                  AND  archived_at   IS NULL
                  AND  is_draft      = false
                  AND  deleted_at    IS NULL
                GROUP BY project_id
            ) ci ON ci.project_id = p.id

            -- assigned_issues: issues asignadas al target user
            LEFT JOIN (
                SELECT i.project_id, COUNT(DISTINCT ia.id) AS cnt
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                GROUP BY i.project_id
            ) ai ON ai.project_id = p.id

            -- completed_issues: asignadas + completadas
            LEFT JOIN (
                SELECT i.project_id, COUNT(DISTINCT ia.id) AS cnt
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                WHERE  ia.assignee_id    = $2
                  AND  ia.deleted_at     IS NULL
                  AND  i.completed_at    IS NOT NULL
                  AND  i.archived_at     IS NULL
                  AND  i.is_draft        = false
                  AND  i.deleted_at      IS NULL
                GROUP BY i.project_id
            ) compi ON compi.project_id = p.id

            -- pending_issues: asignadas en estados backlog/unstarted/started
            LEFT JOIN (
                SELECT i.project_id, COUNT(DISTINCT ia.id) AS cnt
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   states s ON s.id = i.state_id
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  s."group"      IN ('backlog', 'unstarted', 'started')
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                GROUP BY i.project_id
            ) pi2 ON pi2.project_id = p.id

            WHERE p.workspace_id = $3
              AND p.archived_at  IS NULL
              AND p.deleted_at   IS NULL
            "#,
            vec![requester.member_id.into(), user_id.into(), ws_id.into()],
        );

        let rows = ProjectProfileRow::find_by_statement(stmt)
            .all(&state.db)
            .await
            .map_err(AppError::Database)?;

        rows.into_iter()
            .map(|row| {
                // logo_props viene como String JSON; deserializar con fallback seguro.
                let logo_props = serde_json::from_str(&row.logo_props)
                    .unwrap_or(serde_json::json!({}));
                ProjectProfileData {
                    id: row.id,
                    logo_props,
                    created_issues: row.created_issues,
                    assigned_issues: row.assigned_issues,
                    completed_issues: row.completed_issues,
                    pending_issues: row.pending_issues,
                }
            })
            .collect()
    } else {
        // Guest / Viewer — sin acceso a estadísticas de proyectos (paridad Django)
        Vec::new()
    };

    Ok(Json(UserProfileResponse {
        project_data,
        user_data,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE USER STATS  (WorkspaceUserProfileStatsEndpoint)
// ═══════════════════════════════════════════════════════════════════════════
//
// GET /workspaces/{slug}/user-stats/{user_id}/
//
// Mirror de Django `WorkspaceUserProfileStatsEndpoint.get`
// (plane/app/views/workspace/user.py).
//
// Devuelve:
//   - state_distribution     : [{state_group, state_count}]
//   - priority_distribution  : [{priority, priority_count}]
//   - created_issues         : i64
//   - assigned_issues        : i64
//   - completed_issues       : i64
//   - pending_issues         : i64
//   - subscribed_issues      : i64
//   - present_cycles         : [{cycle__name, cycle__id, cycle__project_id}]
//   - upcoming_cycles        : [{cycle__name, cycle__id, cycle__project_id}]
//
// Permiso: miembro activo del workspace (cualquier rol).

#[derive(Debug, Serialize, FromQueryResult)]
pub struct StateDistributionRow {
    pub state_group: String,
    pub state_count: i64,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct PriorityDistributionRow {
    pub priority: String,
    pub priority_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct CycleInfoRow {
    pub cycle_name: String,
    pub cycle_id: Uuid,
    pub cycle_project_id: Uuid,
}

// Serialización que reproduce la forma Django: keys con doble underscore
// como `cycle__name`, `cycle__id`, `cycle__project_id`.
impl serde::Serialize for CycleInfoRow {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(3))?;
        map.serialize_entry("cycle__name", &self.cycle_name)?;
        map.serialize_entry("cycle__id", &self.cycle_id)?;
        map.serialize_entry("cycle__project_id", &self.cycle_project_id)?;
        map.end()
    }
}

#[derive(Debug, Serialize)]
pub struct UserStatsResponse {
    pub state_distribution: Vec<StateDistributionRow>,
    pub priority_distribution: Vec<PriorityDistributionRow>,
    pub created_issues: i64,
    pub assigned_issues: i64,
    pub completed_issues: i64,
    pub pending_issues: i64,
    pub subscribed_issues: i64,
    pub present_cycles: Vec<CycleInfoRow>,
    pub upcoming_cycles: Vec<CycleInfoRow>,
}

/// GET /workspaces/{slug}/user-stats/{user_id}/
///
/// Mirror Django `WorkspaceUserProfileStatsEndpoint.get`.
/// Requiere membresía activa del requester en el workspace; estadísticas
/// calculadas para el `user_id` indicado en la URL.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-stats/{user_id}/",
    tag = "Workspaces",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid, Path, description = "Target user ID"),
    ),
    responses(
        (status = 200, description = "Estadísticas del usuario en el workspace"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No es miembro activo del workspace"),
        (status = 404, description = "Workspace no encontrado"),
    )
)]
pub async fn get_user_stats(
    State(state): State<AppState>,
    AnyAuth(requester): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
) -> Result<Json<UserStatsResponse>, AppError> {
    let db = &state.db;
    let ws = workspace_by_slug(db, &slug).await?;
    // Autorización: el requester debe ser miembro activo (cualquier rol)
    let requester_id = requester.id;
    let _member = require_workspace_member(db, ws.id, requester_id).await?;
    let ws_id = ws.id;

    // ── state_distribution ───────────────────────────────────────────────────
    // Mirror: issues asignadas al user_id, agrupadas por state.group, excluye
    // issue_assignees con deleted_at != NULL (paridad con la condición
    // `Q(issue_assignee__deleted_at__isnull=True)` en Django).
    //
    // NOTA: `group` es palabra reservada en PostgreSQL — debe escaparse con
    // comillas dobles (`s."group"`) para evitar errores de parsing en
    // `GROUP BY` / `ORDER BY`. Parámetros bindados ($1..$N) en vez de
    // interpolación `format!` para prevenir SQL injection en profundidad.
    let state_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT s."group" AS state_group, COUNT(DISTINCT ia.id) AS state_count
        FROM issue_assignees ia
        JOIN issues      i  ON i.id  = ia.issue_id
        JOIN states      s  ON s.id  = i.state_id
        JOIN projects    p  ON p.id  = i.project_id
        JOIN project_members pm
             ON  pm.project_id = p.id
             AND pm.member_id  = $1
             AND pm.is_active  = true
             AND pm.deleted_at IS NULL
        WHERE ia.assignee_id = $2
          AND ia.deleted_at  IS NULL
          AND i.workspace_id = $3
          AND i.archived_at  IS NULL
          AND i.is_draft     = false
          AND i.deleted_at   IS NULL
          AND p.deleted_at   IS NULL
        GROUP BY s."group"
        ORDER BY s."group"
        "#,
        vec![requester_id.into(), user_id.into(), ws_id.into()],
    );

    let state_distribution = StateDistributionRow::find_by_statement(state_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── priority_distribution ────────────────────────────────────────────────
    // Mirror: mismas issues asignadas, agrupadas por priority.
    // Orden Django: urgent=0, high=1, medium=2, low=3, none=4.
    let priority_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT i.priority,
               COUNT(DISTINCT ia.id) AS priority_count,
               CASE i.priority
                   WHEN 'urgent' THEN 0
                   WHEN 'high'   THEN 1
                   WHEN 'medium' THEN 2
                   WHEN 'low'    THEN 3
                   ELSE 4
               END AS priority_order
        FROM issue_assignees ia
        JOIN issues   i  ON i.id  = ia.issue_id
        JOIN projects p  ON p.id  = i.project_id
        JOIN project_members pm
             ON  pm.project_id = p.id
             AND pm.member_id  = $1
             AND pm.is_active  = true
             AND pm.deleted_at IS NULL
        WHERE ia.assignee_id = $2
          AND ia.deleted_at  IS NULL
          AND i.workspace_id = $3
          AND i.archived_at  IS NULL
          AND i.is_draft     = false
          AND i.deleted_at   IS NULL
          AND p.deleted_at   IS NULL
        GROUP BY i.priority
        HAVING COUNT(DISTINCT ia.id) >= 1
        ORDER BY priority_order
        "#,
        vec![requester_id.into(), user_id.into(), ws_id.into()],
    );

    let priority_distribution = PriorityDistributionRow::find_by_statement(priority_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── contadores escalares ─────────────────────────────────────────────────
    // Una sola query multi-columna para reducir round-trips a la DB.
    // Placeholders: $1 = requester_id, $2 = user_id, $3 = ws_id.
    // Postgres permite reutilizar el mismo `$n` varias veces en la misma query.
    let counters_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT
            -- created_issues
            (
                SELECT COUNT(*)
                FROM   issues i
                JOIN   projects p  ON p.id  = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  i.workspace_id  = $3
                  AND  i.created_by_id = $2
                  AND  i.archived_at   IS NULL
                  AND  i.is_draft      = false
                  AND  i.deleted_at    IS NULL
                  AND  p.deleted_at    IS NULL
            ) AS created_issues,

            -- assigned_issues
            (
                SELECT COUNT(DISTINCT ia.id)
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   projects p ON p.id = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  i.workspace_id = $3
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                  AND  p.deleted_at   IS NULL
            ) AS assigned_issues,

            -- completed_issues
            (
                SELECT COUNT(DISTINCT ia.id)
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   states s ON s.id = i.state_id
                JOIN   projects p ON p.id = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  s."group"      = 'completed'
                  AND  i.workspace_id = $3
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                  AND  p.deleted_at   IS NULL
            ) AS completed_issues,

            -- pending_issues: no completadas ni canceladas
            (
                SELECT COUNT(DISTINCT ia.id)
                FROM   issue_assignees ia
                JOIN   issues i ON i.id = ia.issue_id
                JOIN   states s ON s.id = i.state_id
                JOIN   projects p ON p.id = i.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  ia.assignee_id = $2
                  AND  ia.deleted_at  IS NULL
                  AND  s."group"      NOT IN ('completed', 'cancelled')
                  AND  i.workspace_id = $3
                  AND  i.archived_at  IS NULL
                  AND  i.is_draft     = false
                  AND  i.deleted_at   IS NULL
                  AND  p.deleted_at   IS NULL
            ) AS pending_issues,

            -- subscribed_issues
            (
                SELECT COUNT(DISTINCT isub.id)
                FROM   issue_subscribers isub
                JOIN   projects p ON p.id = isub.project_id
                JOIN   project_members pm
                       ON  pm.project_id = p.id
                       AND pm.member_id  = $1
                       AND pm.is_active  = true
                       AND pm.deleted_at IS NULL
                WHERE  isub.subscriber_id = $2
                  AND  isub.workspace_id  = $3
                  AND  p.archived_at      IS NULL
                  AND  p.deleted_at       IS NULL
            ) AS subscribed_issues
        "#,
        vec![requester_id.into(), user_id.into(), ws_id.into()],
    );

    #[derive(Debug, FromQueryResult)]
    struct CountersRow {
        created_issues: i64,
        assigned_issues: i64,
        completed_issues: i64,
        pending_issues: i64,
        subscribed_issues: i64,
    }

    let counters = CountersRow::find_by_statement(counters_stmt)
        .one(db)
        .await
        .map_err(AppError::Database)?
        .unwrap_or(CountersRow {
            created_issues: 0,
            assigned_issues: 0,
            completed_issues: 0,
            pending_issues: 0,
            subscribed_issues: 0,
        });

    // ── upcoming_cycles ──────────────────────────────────────────────────────
    // Mirror: CycleIssue donde cycle.start_date > now() e issue tiene al user asignado.
    let upcoming_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT DISTINCT
               c.name       AS cycle_name,
               c.id         AS cycle_id,
               c.project_id AS cycle_project_id
        FROM   cycle_issues ci
        JOIN   cycles c ON c.id = ci.cycle_id
        JOIN   issue_assignees ia ON ia.issue_id = ci.issue_id AND ia.deleted_at IS NULL
        WHERE  c.workspace_id = $1
          AND  c.start_date   > NOW()
          AND  ia.assignee_id = $2
          AND  ci.deleted_at  IS NULL
          AND  c.deleted_at   IS NULL
        "#,
        vec![ws_id.into(), user_id.into()],
    );

    let upcoming_cycles = CycleInfoRow::find_by_statement(upcoming_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── present_cycles ───────────────────────────────────────────────────────
    // Mirror: start_date < now() AND end_date > now().
    let present_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        SELECT DISTINCT
               c.name       AS cycle_name,
               c.id         AS cycle_id,
               c.project_id AS cycle_project_id
        FROM   cycle_issues ci
        JOIN   cycles c ON c.id = ci.cycle_id
        JOIN   issue_assignees ia ON ia.issue_id = ci.issue_id AND ia.deleted_at IS NULL
        WHERE  c.workspace_id = $1
          AND  c.start_date   < NOW()
          AND  c.end_date     > NOW()
          AND  ia.assignee_id = $2
          AND  ci.deleted_at  IS NULL
          AND  c.deleted_at   IS NULL
        "#,
        vec![ws_id.into(), user_id.into()],
    );

    let present_cycles = CycleInfoRow::find_by_statement(present_stmt)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(UserStatsResponse {
        state_distribution,
        priority_distribution,
        created_issues: counters.created_issues,
        assigned_issues: counters.assigned_issues,
        completed_issues: counters.completed_issues,
        pending_issues: counters.pending_issues,
        subscribed_issues: counters.subscribed_issues,
        present_cycles,
        upcoming_cycles,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// WORKSPACE USER ACTIVITY  (WorkspaceUserActivityEndpoint)
// ═══════════════════════════════════════════════════════════════════════════
//
// GET /workspaces/{slug}/user-activity/{user_id}/
//
// Mirror de Django `WorkspaceUserActivityEndpoint.get`
// (plane/app/views/workspace/user.py:370).
//
// Devuelve actividades de `IssueActivity` del actor `user_id` en el workspace,
// excluyendo los campos virtuales (comment, vote, reaction, draft), y filtrando
// solo proyectos donde el **requester** es miembro activo no archivado.
//
// Query params opcionales:
//   - `project` (repetible): UUIDs de proyecto para filtrar
//   - `per_page`: registros por página (default 10)
//   - `cursor`: cursor de paginación estilo Django
//   - `order_by`: columna de ordenamiento (default `-created_at`)
//
// Serialización: mirror de `IssueActivitySerializer(fields="__all__")` con
// nested `actor_detail`, `issue_detail`, `project_detail`, `workspace_detail`.

/// Mirror de `IssueActivitySerializer.actor_detail` → `UserLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ActivityActorDetail {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub display_name: String,
}

/// Mirror de `IssueFlatSerializer` — campos mínimos que el frontend consume en
/// el panel de actividad del perfil de usuario.
#[derive(Debug, Serialize)]
pub struct ActivityIssueDetail {
    pub id: Uuid,
    pub name: String,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
}

/// Mirror de `ProjectLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ActivityProjectDetail {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub logo_props: serde_json::Value,
}

/// Mirror de `WorkspaceLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ActivityWorkspaceDetail {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
}

/// Respuesta de una actividad — mirror de `IssueActivitySerializer(fields="__all__")`.
#[derive(Debug, Serialize)]
pub struct UserActivityItem {
    // Campos del modelo IssueActivity
    pub id: Uuid,
    pub verb: String,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub comment: String,
    pub actor_id: Option<Uuid>,
    pub issue_id: Option<Uuid>,
    pub issue_comment_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub old_identifier: Option<Uuid>,
    pub new_identifier: Option<Uuid>,
    pub epoch: Option<f64>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    // Campos anidados (None cuando el objeto referenciado fue eliminado)
    pub actor_detail: Option<ActivityActorDetail>,
    pub issue_detail: Option<ActivityIssueDetail>,
    pub project_detail: Option<ActivityProjectDetail>,
    pub workspace_detail: Option<ActivityWorkspaceDetail>,
}

/// Query params de `GET /workspaces/{slug}/user-activity/{user_id}/`.
#[derive(Debug, Deserialize)]
pub struct UserActivityQuery {
    /// Filtro por proyecto (multi-valor: ?project=A&project=B).
    #[serde(default)]
    pub project: Vec<Uuid>,
    pub per_page: Option<u64>,
    pub cursor: Option<String>,
    pub order_by: Option<String>,
}

/// `GET /workspaces/{slug}/user-activity/{user_id}/`
///
/// Actividades del actor `user_id` visibles para el requester (solo proyectos
/// donde el requester es miembro activo).
///
/// Mirror de `WorkspaceUserActivityEndpoint` en Django con paginación cursor.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-activity/{user_id}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"    = String, Path, description = "Workspace slug"),
        ("user_id" = Uuid,   Path, description = "Actor user UUID"),
    ),
    responses(
        (status = 200, description = "Paginated user activity list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a workspace member"),
        (status = 404, description = "Workspace not found"),
    )
)]
pub async fn get_workspace_user_activity(
    State(state): State<AppState>,
    AnyAuth(auth_user): AnyAuth,
    Path((slug, user_id)): Path<(String, Uuid)>,
    Query(q): Query<UserActivityQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let db = &state.db;
    let ws = workspace_by_slug(db, &slug).await?;
    // Autorización: el requester debe ser miembro activo del workspace
    let _member = require_workspace_member(db, ws.id, auth_user.id).await?;

    // ── Paginación ────────────────────────────────────────────────────────────
    const DEFAULT_PER_PAGE: u64 = 10;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // ── Proyectos visibles para el requester ──────────────────────────────────
    // Django: `project__project_projectmember__member=request.user`
    //         `project__project_projectmember__is_active=True`
    //         `project__archived_at__isnull=True`
    //
    // Resolvemos en Rust con una query separada para obtener los IDs de proyecto
    // accesibles, luego los usamos como filtro IN sobre IssueActivity.
    // Esto evita un JOIN complejo en SeaORM y mantiene el código legible.
    let accessible_project_ids: Vec<Uuid> = {
        let memberships = project_members::Entity::find()
            .filter(project_members::Column::MemberId.eq(auth_user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .filter(project_members::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        if memberships.is_empty() {
            // Sin proyectos accesibles → respuesta vacía paginada
            let body = pagination::build_response(
                Vec::<UserActivityItem>::new(),
                0,
                limit,
                cursor.offset,
            );
            return Ok((axum::http::StatusCode::OK, axum::Json(body)));
        }

        // Filtrar solo proyectos del workspace actual y no archivados
        let project_ids_from_memberships: Vec<Uuid> =
            memberships.iter().map(|pm| pm.project_id).collect();

        let visible_projects = projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids_from_memberships))
            .filter(projects::Column::WorkspaceId.eq(ws.id))
            .filter(projects::Column::ArchivedAt.is_null())
            .filter(projects::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?;

        visible_projects.into_iter().map(|p| p.id).collect()
    };

    if accessible_project_ids.is_empty() {
        let body = pagination::build_response(
            Vec::<UserActivityItem>::new(),
            0,
            limit,
            cursor.offset,
        );
        return Ok((axum::http::StatusCode::OK, axum::Json(body)));
    }

    // ── Campos excluidos — mirror Django: ~Q(field__in=["comment","vote","reaction","draft"]) ──
    const EXCLUDED_FIELDS: &[&str] = &["comment", "vote", "reaction", "draft"];

    // ── Construir query base ──────────────────────────────────────────────────
    let mut base = issue_activities::Entity::find()
        .filter(issue_activities::Column::WorkspaceId.eq(ws.id))
        .filter(issue_activities::Column::ActorId.eq(user_id))
        .filter(issue_activities::Column::ProjectId.is_in(accessible_project_ids))
        .filter(issue_activities::Column::DeletedAt.is_null())
        // Excluir fields virtuales (NOT IN)
        .filter(
            sea_orm::Condition::any()
                .add(issue_activities::Column::Field.is_null())
                .add(
                    issue_activities::Column::Field
                        .is_not_in(EXCLUDED_FIELDS.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
                ),
        );

    // Filtro opcional por proyecto (?project=UUID)
    if !q.project.is_empty() {
        base = base.filter(issue_activities::Column::ProjectId.is_in(q.project.clone()));
    }

    // ── Count (mismos filtros, sin orden ni offset) ──────────────────────────
    let total_count = base.clone().count(db).await.map_err(AppError::Database)?;

    // ── Ordenamiento — mirror Django: default -created_at ────────────────────
    let order_col = q.order_by.as_deref().unwrap_or("-created_at");
    let (col, asc) = if let Some(stripped) = order_col.strip_prefix('-') {
        (stripped, false)
    } else {
        (order_col, true)
    };

    let ordered = match col {
        "created_at" => {
            if asc {
                base.order_by_asc(issue_activities::Column::CreatedAt)
            } else {
                base.order_by_desc(issue_activities::Column::CreatedAt)
            }
        }
        "updated_at" => {
            if asc {
                base.order_by_asc(issue_activities::Column::UpdatedAt)
            } else {
                base.order_by_desc(issue_activities::Column::UpdatedAt)
            }
        }
        _ => base.order_by_desc(issue_activities::Column::CreatedAt),
    };

    // ── Fetch de la página ───────────────────────────────────────────────────
    let activities = ordered
        .paginate(db, limit)
        .fetch_page(cursor.offset)
        .await
        .map_err(AppError::Database)?;

    if activities.is_empty() {
        let body = pagination::build_response(
            Vec::<UserActivityItem>::new(),
            total_count,
            limit,
            cursor.offset,
        );
        return Ok((axum::http::StatusCode::OK, axum::Json(body)));
    }

    // ── Batch-fetch de objetos relacionados (evitar N+1) ─────────────────────

    // Actor IDs (siempre el mismo user_id, pero mantenemos el patrón genérico)
    let actor_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.actor_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let actors_map: std::collections::HashMap<Uuid, users::Model> = users::Entity::find()
        .filter(users::Column::Id.is_in(actor_ids))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|u| (u.id, u))
        .collect();

    // Issue IDs
    let issue_ids: Vec<Uuid> = activities
        .iter()
        .filter_map(|a| a.issue_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let issues_map: std::collections::HashMap<Uuid, issues::Model> = if !issue_ids.is_empty() {
        issues::Entity::find()
            .filter(issues::Column::Id.is_in(issue_ids))
            .filter(issues::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|i| (i.id, i))
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    // Project IDs de las actividades de esta página
    let page_project_ids: Vec<Uuid> = activities
        .iter()
        .map(|a| a.project_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let projects_map: std::collections::HashMap<Uuid, projects::Model> =
        projects::Entity::find()
            .filter(projects::Column::Id.is_in(page_project_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| (p.id, p))
            .collect();

    // Workspace detail (único para todas las actividades de la página)
    let workspace_detail = ActivityWorkspaceDetail {
        id: ws.id,
        name: ws.name.clone(),
        slug: ws.slug.clone(),
        logo: ws.logo.clone(),
    };

    // ── Ensamblar respuesta ──────────────────────────────────────────────────
    let results: Vec<UserActivityItem> = activities
        .into_iter()
        .map(|a| {
            // actor_detail
            let actor_detail = a.actor_id.and_then(|aid| {
                actors_map.get(&aid).map(|u| {
                    let avatar_url = if let Some(asset_id) = u.avatar_asset_id {
                        Some(format!("/api/assets/v2/static/{}/", asset_id))
                    } else if !u.avatar.is_empty() {
                        Some(u.avatar.clone())
                    } else {
                        None
                    };
                    ActivityActorDetail {
                        id: u.id,
                        first_name: u.first_name.clone(),
                        last_name: u.last_name.clone(),
                        avatar: u.avatar.clone(),
                        avatar_url,
                        is_bot: u.is_bot,
                        display_name: u.display_name.clone(),
                    }
                })
            });

            // issue_detail
            let issue_detail = a.issue_id.and_then(|iid| {
                issues_map.get(&iid).map(|i| ActivityIssueDetail {
                    id: i.id,
                    name: i.name.clone(),
                    sequence_id: i.sequence_id,
                    project_id: i.project_id,
                    workspace_id: i.workspace_id,
                })
            });

            // project_detail
            let project_detail = projects_map.get(&a.project_id).map(|p| {
                ActivityProjectDetail {
                    id: p.id,
                    identifier: p.identifier.clone(),
                    name: p.name.clone(),
                    logo_props: p.logo_props.clone(),
                }
            });

            UserActivityItem {
                id: a.id,
                verb: a.verb,
                field: a.field,
                old_value: a.old_value,
                new_value: a.new_value,
                comment: a.comment,
                actor_id: a.actor_id,
                issue_id: a.issue_id,
                issue_comment_id: a.issue_comment_id,
                project_id: a.project_id,
                workspace_id: a.workspace_id,
                old_identifier: a.old_identifier,
                new_identifier: a.new_identifier,
                epoch: a.epoch,
                created_at: a.created_at.into(),
                updated_at: a.updated_at.into(),
                actor_detail,
                issue_detail,
                project_detail,
                workspace_detail: Some(ActivityWorkspaceDetail {
                    id: workspace_detail.id,
                    name: workspace_detail.name.clone(),
                    slug: workspace_detail.slug.clone(),
                    logo: workspace_detail.logo.clone(),
                }),
            }
        })
        .collect();

    let body = pagination::build_response(results, total_count, limit, cursor.offset);
    Ok((axum::http::StatusCode::OK, axum::Json(body)))
}
