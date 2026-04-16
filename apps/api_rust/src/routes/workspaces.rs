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
    entities::{draft_issues, users, workspace_member_invites, workspace_members, workspaces},
    error::AppError,
    routes::helpers::{require_workspace_member, workspace_by_slug},
    utils::{instance_config::get_config_value, soft_delete::SoftDeleteExt, url::contains_url, color::get_random_color},
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
    let mut active: workspaces::ActiveModel = ws.into();
    active.deleted_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(&state.db).await.map_err(AppError::Database)?;

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
