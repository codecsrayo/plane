// src/routes/states.rs
//! Endpoints de State — Fase 3.
//!
//! Equivalente a `plane/app/views/state/base.py` en Django.
//! Autenticación: sesión cookie **o** API key (via `AnyAuth`).
//! Autorización: membresía activa en el proyecto.
//! - Lectura: cualquier miembro activo (Guest, Viewer, Member, Admin).
//! - Escritura / borrado / mark-default: solo Admin del proyecto o Admin del workspace.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::{project_members, states, workspace_members, workspaces},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ─── Constantes de rol ────────────────────────────────────────────────────────

const ROLE_ADMIN: i16 = 20;

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn workspace_by_slug(
    db: &sea_orm::DatabaseConnection,
    slug: &str,
) -> Result<workspaces::Model, AppError> {
    workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

async fn require_workspace_member(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<workspace_members::Model, AppError> {
    workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
        .filter(workspace_members::Column::MemberId.eq(user_id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Forbidden)
}

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

/// El usuario necesita ser Admin del proyecto **o** Admin del workspace.
fn require_admin(
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

/// El usuario necesita ser miembro activo del proyecto (cualquier rol).
fn require_project_member(pm: &Option<project_members::Model>) -> Result<(), AppError> {
    if pm.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub color: String,
    pub slug: String,
    pub group: String,
    /// Posición normalizada dentro del grupo (calculada en tiempo de respuesta).
    pub sequence: f64,
    pub default: bool,
    pub is_triage: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&states::Model> for StateResponse {
    fn from(s: &states::Model) -> Self {
        Self {
            id: s.id,
            name: s.name.clone(),
            description: s.description.clone(),
            color: s.color.clone(),
            slug: s.slug.clone(),
            group: s.group.clone(),
            sequence: s.sequence,
            default: s.default,
            is_triage: s.is_triage,
            project_id: s.project_id,
            workspace_id: s.workspace_id,
            created_by_id: s.created_by_id,
            updated_by_id: s.updated_by_id,
            external_id: s.external_id.clone(),
            external_source: s.external_source.clone(),
            created_at: s.created_at.with_timezone(&Utc),
            updated_at: s.updated_at.with_timezone(&Utc),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateStateRequest {
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub description: String,
    pub group: String,
    pub sequence: Option<f64>,
    pub default: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStateRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub group: Option<String>,
    pub sequence: Option<f64>,
    pub default: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct GroupedQuery {
    #[serde(default)]
    pub grouped: Option<String>,
}

// ─── Path params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WorkspaceProjectPath {
    pub slug: String,
    pub project_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceProjectStatePath {
    pub slug: String,
    pub project_id: Uuid,
    pub pk: Uuid,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// Lista todos los estados activos (no-triage) de un proyecto.
///
/// Acepta `?grouped=true` para agrupar por grupo de estado.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/states/",
    tag = "States",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project UUID"),
        GroupedQuery,
    ),
    responses(
        (status = 200, description = "Lista de estados"),
        (status = 403, description = "Sin permisos"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_states(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectPath>,
    Query(q): Query<GroupedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_project_member(&pm)?;

    let raw_states = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .order_by_asc(states::Column::Sequence)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Normaliza `sequence` dentro de cada grupo (espeja lógica Django)
    let mut responses: Vec<StateResponse> = raw_states.iter().map(StateResponse::from).collect();
    normalize_sequence_by_group(&mut responses);

    if q.grouped.as_deref() == Some("true") {
        // Devuelve mapa { group: [states] }
        let mut map: std::collections::HashMap<String, Vec<StateResponse>> =
            std::collections::HashMap::new();
        for s in responses {
            map.entry(s.group.clone()).or_default().push(s);
        }
        return Ok((StatusCode::OK, Json(serde_json::to_value(map).unwrap())).into_response());
    }

    Ok((StatusCode::OK, Json(responses)).into_response())
}

/// Obtiene un estado específico por ID.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path, description = "State UUID"),
    ),
    responses(
        (status = 200, description = "Estado encontrado", body = StateResponse),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_state(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_project_member(&pm)?;

    let state = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::OK, Json(StateResponse::from(&state))).into_response())
}

/// Crea un nuevo estado en el proyecto. Solo Admin.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/states/",
    tag = "States",
    request_body = CreateStateRequest,
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
    ),
    responses(
        (status = 200, description = "Estado creado", body = StateResponse),
        (status = 400, description = "Nombre duplicado o datos inválidos"),
        (status = 403, description = "Sin permisos"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_state(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectPath>,
    Json(body): Json<CreateStateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_admin(&pm, &wm)?;

    validate_group(&body.group)?;

    // Nombre único por proyecto (excluyendo soft-deleted)
    let duplicate = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::Name.eq(&body.name))
        .one(db)
        .await
        .map_err(AppError::Database)?;
    if duplicate.is_some() {
        return Err(AppError::BadRequest("The state name is already taken".into()));
    }

    // Calcula sequence máxima
    let max_seq = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .all(db)
        .await
        .map_err(AppError::Database)?
        .iter()
        .map(|s| s.sequence as i64)
        .max()
        .unwrap_or(0);

    let sequence = body.sequence.unwrap_or((max_seq + 15000) as f64);
    let slug = slugify(&body.name);
    let now = chrono::Utc::now().fixed_offset();

    let new_state = states::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: Set(body.name.clone()),
        description: Set(body.description.clone()),
        color: Set(body.color.clone()),
        slug: Set(slug),
        group: Set(body.group.clone()),
        sequence: Set(sequence),
        default: Set(body.default.unwrap_or(false)),
        is_triage: Set(false),
        project_id: Set(p.project_id),
        workspace_id: Set(ws.id),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        external_id: Set(None),
        external_source: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let inserted = new_state
        .insert(db)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") {
                AppError::BadRequest("The state name is already taken".into())
            } else {
                AppError::Database(e)
            }
        })?;

    Ok((StatusCode::OK, Json(StateResponse::from(&inserted))).into_response())
}

/// Actualiza parcialmente un estado. Solo Admin.
#[utoipa::path(
    patch,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/",
    tag = "States",
    request_body = UpdateStateRequest,
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path),
    ),
    responses(
        (status = 200, description = "Estado actualizado", body = StateResponse),
        (status = 400, description = "Nombre duplicado"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_state(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
    Json(body): Json<UpdateStateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_admin(&pm, &wm)?;

    let existing = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Validar nombre único si se está cambiando
    if let Some(ref new_name) = body.name {
        let duplicate = states::Entity::find()
            .active()
            .filter(states::Column::ProjectId.eq(p.project_id))
            .filter(states::Column::Name.eq(new_name))
            .filter(states::Column::Id.ne(p.pk))
            .one(db)
            .await
            .map_err(AppError::Database)?;
        if duplicate.is_some() {
            return Err(AppError::BadRequest("The state name is already taken".into()));
        }
    }

    if let Some(ref g) = body.group {
        validate_group(g)?;
    }

    let now = chrono::Utc::now().fixed_offset();
    let mut active: states::ActiveModel = existing.into();

    if let Some(name) = body.name {
        let slug = slugify(&name);
        active.slug = Set(slug);
        active.name = Set(name);
    }
    if let Some(color) = body.color {
        active.color = Set(color);
    }
    if let Some(desc) = body.description {
        active.description = Set(desc);
    }
    if let Some(group) = body.group {
        active.group = Set(group);
    }
    if let Some(seq) = body.sequence {
        active.sequence = Set(seq);
    }
    if let Some(def) = body.default {
        active.default = Set(def);
    }
    active.updated_by_id = Set(Some(user_id));
    active.updated_at = Set(now);

    let updated = active.update(db).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("already exists") || msg.contains("duplicate") {
            AppError::BadRequest("The state name is already taken".into())
        } else {
            AppError::Database(e)
        }
    })?;

    Ok((StatusCode::OK, Json(StateResponse::from(&updated))).into_response())
}

/// Elimina un estado (solo si está vacío y no es default). Solo Admin.
#[utoipa::path(
    delete,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path),
    ),
    responses(
        (status = 204, description = "Estado eliminado"),
        (status = 400, description = "Estado default o con issues"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_state(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_admin(&pm, &wm)?;

    let state = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(false))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if state.default {
        return Err(AppError::BadRequest(
            "Default state cannot be deleted".into(),
        ));
    }

    // Verificar que no haya issues en este estado
    use crate::entities::issues;
    let issue_exists = issues::Entity::find()
        .active()
        .filter(issues::Column::StateId.eq(p.pk))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if issue_exists {
        return Err(AppError::BadRequest(
            "The state is not empty, only empty states can be deleted".into(),
        ));
    }

    // Soft-delete: setear deleted_at
    let now = chrono::Utc::now().fixed_offset();
    let mut active: states::ActiveModel = state.into();
    active.deleted_at = Set(Some(now));
    active.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Obtiene el estado de triage del proyecto (intake state).
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/intake-state/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
    ),
    responses(
        (status = 200, description = "Estado triage", body = StateResponse),
        (status = 404, description = "Sin estado triage"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn intake_state(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectPath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_project_member(&pm)?;

    let state = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::IsTriage.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::OK, Json(StateResponse::from(&state))).into_response())
}

/// Marca un estado como default del proyecto (desactiva el anterior). Solo Admin.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default/",
    tag = "States",
    params(
        ("slug" = String, Path),
        ("project_id" = Uuid, Path),
        ("pk" = Uuid, Path),
    ),
    responses(
        (status = 204, description = "Marcado como default"),
        (status = 403, description = "Sin permisos"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn mark_default(
    State(app): State<AppState>,
    AnyAuth(user_id): AnyAuth,
    Path(p): Path<WorkspaceProjectStatePath>,
) -> Result<impl IntoResponse, AppError> {
    let db = &app.db;
    let ws = workspace_by_slug(db, &p.slug).await?;
    let wm = require_workspace_member(db, ws.id, user_id).await?;
    let pm = project_member_for_user(db, p.project_id, user_id).await?;
    require_admin(&pm, &wm)?;

    // Verificar que el estado existe
    let _target = states::Entity::find_by_id(p.pk)
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let txn = db.begin().await.map_err(AppError::Database)?;
    let now = chrono::Utc::now().fixed_offset();

    // Quitar default de todos los estados del proyecto
    let all_defaults = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(p.project_id))
        .filter(states::Column::WorkspaceId.eq(ws.id))
        .filter(states::Column::Default.eq(true))
        .all(&txn)
        .await
        .map_err(AppError::Database)?;

    for s in all_defaults {
        let mut a: states::ActiveModel = s.into();
        a.default = Set(false);
        a.updated_at = Set(now);
        a.update(&txn).await.map_err(AppError::Database)?;
    }

    // Marcar el seleccionado
    let target = states::Entity::find_by_id(p.pk)
        .one(&txn)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;
    let mut a: states::ActiveModel = target.into();
    a.default = Set(true);
    a.updated_at = Set(now);
    a.update(&txn).await.map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

// ─── Utilidades internas ──────────────────────────────────────────────────────

/// Normaliza la secuencia dentro de cada grupo a un valor entre 0 y 1
/// (posición relativa), espejando la lógica de Django.
fn normalize_sequence_by_group(states: &mut Vec<StateResponse>) {
    use std::collections::HashMap;
    let mut group_counts: HashMap<String, usize> = HashMap::new();
    let mut group_indices: HashMap<String, usize> = HashMap::new();

    for s in states.iter() {
        *group_counts.entry(s.group.clone()).or_insert(0) += 1;
    }
    for s in states.iter_mut() {
        let count = *group_counts.get(&s.group).unwrap_or(&1);
        let idx = group_indices.entry(s.group.clone()).or_insert(0);
        *idx += 1;
        s.sequence = *idx as f64 / count as f64;
    }
}

/// Valida que el grupo sea uno de los valores aceptados.
fn validate_group(group: &str) -> Result<(), AppError> {
    const VALID: &[&str] = &[
        "backlog", "unstarted", "started", "completed", "cancelled", "triage",
    ];
    if VALID.contains(&group) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Invalid group '{}'. Valid values: backlog, unstarted, started, completed, cancelled, triage",
            group
        )))
    }
}

/// Genera slug simple a partir de un nombre (baja, reemplaza espacios por guiones).
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}
