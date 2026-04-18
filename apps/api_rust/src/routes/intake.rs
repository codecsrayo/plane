// src/routes/intake.rs
//! Endpoints de Intake (buzón de entrada de issues).
//!
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intakes/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/intakes/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intake-issues/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/intake-issues/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{intake_issues, intakes, issues},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Intake status constants (matches Django IntakeIssue.STATUS_CHOICES) ───────
pub const STATUS_PENDING: i32 = -2;
pub const STATUS_REJECTED: i32 = -1;
pub const STATUS_SNOOZED: i32 = 0;
pub const STATUS_ACCEPTED: i32 = 1;
pub const STATUS_DUPLICATE: i32 = 2;

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakeResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl IntakeResponse {
    fn from_model(m: intakes::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            is_default: m.is_default,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IntakeIssueResponse {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub intake_id: Uuid,
    pub status: i32,
    pub source: Option<String>,
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duplicate_to_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl IntakeIssueResponse {
    fn from_model(m: intake_issues::Model) -> Self {
        Self {
            id: m.id,
            issue_id: m.issue_id,
            intake_id: m.intake_id,
            status: m.status,
            source: m.source,
            snoozed_till: m.snoozed_till,
            duplicate_to_id: m.duplicate_to_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIntakeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeIssueRequest {
    pub intake_id: Uuid,
    // Issue fields
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIntakeIssueRequest {
    pub status: Option<i32>,
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub duplicate_to_id: Option<Uuid>,
    pub source: Option<String>,
}

// ── GET /intakes/ ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Lista de intakes")),
    security(("TokenAuth" = []))
)]
pub async fn list_intakes(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<IntakeResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = intakes::Entity::find()
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(intakes::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(IntakeResponse::from_model).collect()))
}

// ── POST /intakes/ ────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Intake creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateIntakeRequest>,
) -> Result<(StatusCode, Json<IntakeResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    let intake = intakes::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description: Set(body.description.unwrap_or_default()),
        is_default: Set(false),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        view_props: Set(serde_json::json!({})),
        logo_props: Set(serde_json::json!({})),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(IntakeResponse::from_model(intake))))
}

// ── GET /intakes/{pk}/ ────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Intake ID"),
    ),
    responses(
        (status = 200, description = "Detalle del intake"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<IntakeResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let intake = intakes::Entity::find_by_id(pk)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(IntakeResponse::from_model(intake)))
}

// ── PATCH /intakes/{pk}/ ──────────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Intake ID"),
    ),
    responses(
        (status = 200, description = "Intake actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIntakeRequest>,
) -> Result<Json<IntakeResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let intake = intakes::Entity::find_by_id(pk)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: intakes::ActiveModel = intake.into();
    if let Some(name) = body.name {
        am.name = Set(name);
    }
    if let Some(desc) = body.description {
        am.description = Set(desc);
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(IntakeResponse::from_model(updated)))
}

// ── DELETE /intakes/{pk}/ ─────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/intakes/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Intake ID"),
    ),
    responses((status = 204, description = "Eliminado")),
    security(("TokenAuth" = []))
)]
pub async fn delete_intake(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let intake = intakes::Entity::find_by_id(pk)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: intakes::ActiveModel = intake.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /intake-issues/ ───────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses((status = 200, description = "Lista de intake issues")),
    security(("TokenAuth" = []))
)]
pub async fn list_intake_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<IntakeIssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .order_by_desc(intake_issues::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(rows.into_iter().map(IntakeIssueResponse::from_model).collect()))
}

// ── POST /intake-issues/ ──────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Issue de intake creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateIntakeIssueRequest>,
) -> Result<(StatusCode, Json<IntakeIssueResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    // Verificar que el intake pertenece al proyecto
    let _ = intakes::Entity::find_by_id(body.intake_id)
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::BadRequest("intake_id no válido para este proyecto".into()))?;

    // Calcular sequence_id.
    //
    // NOTA 1: MAX() sin GROUP BY siempre devuelve una fila (aunque la tabla
    // esté vacía, con valor NULL). Decodificamos a Option<i32> y flatten
    // sobre el Option<Option<i32>> de .one().
    //
    // NOTA 2: el target es i32 (NO i64). En Postgres MAX(INT4) → INT4;
    // no se promueve a BIGINT como en MySQL. Usar i64 produce
    // "mismatched types; Rust type Option<i64> (as SQL type INT8) is not
    // compatible with SQL type INT4".
    use sea_orm::QuerySelect;
    let max_seq: Option<i32> = issues::Entity::find()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .select_only()
        .column_as(
            sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
            "max_seq",
        )
        .into_tuple::<Option<i32>>()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .flatten();
    let sequence_id = max_seq.unwrap_or(0) + 1;

    // created_at / updated_at explícitos: ActiveModelBehavior vacío,
    // columnas NOT NULL. Mismo patrón que labels.rs / issues.rs::create_issue.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    // Crear el issue subyacente con is_draft=true
    let issue = issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description_html: Set(body.description_html.unwrap_or_default()),
        description_json: Set(serde_json::json!({})),
        priority: Set(body.priority.unwrap_or_else(|| "none".to_owned())),
        sequence_id: Set(sequence_id),
        sort_order: Set(65535.0),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        is_draft: Set(true), // intake issues son drafts hasta ser aceptados
        description_stripped: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    let intake_issue = intake_issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        issue_id: Set(issue.id),
        intake_id: Set(body.intake_id),
        status: Set(STATUS_PENDING),
        source: Set(body.source),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        extra: Set(serde_json::json!({})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(IntakeIssueResponse::from_model(intake_issue))))
}

// ── GET /intake-issues/{pk}/ ──────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "IntakeIssue ID"),
    ),
    responses(
        (status = 200, description = "Detalle del intake issue"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<IntakeIssueResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let ii = intake_issues::Entity::find_by_id(pk)
        .active()
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(IntakeIssueResponse::from_model(ii)))
}

// ── PATCH /intake-issues/{pk}/ ────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "IntakeIssue ID"),
    ),
    responses(
        (status = 200, description = "IntakeIssue actualizado"),
        (status = 400, description = "Status inválido"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIntakeIssueRequest>,
) -> Result<Json<IntakeIssueResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let ii = intake_issues::Entity::find_by_id(pk)
        .active()
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Validar status si se envía
    if let Some(status) = body.status {
        let valid = [STATUS_PENDING, STATUS_REJECTED, STATUS_SNOOZED, STATUS_ACCEPTED, STATUS_DUPLICATE];
        if !valid.contains(&status) {
            return Err(AppError::BadRequest(format!(
                "status inválido: {status}. Valores permitidos: -2, -1, 0, 1, 2"
            )));
        }
    }

    let issue_id = ii.issue_id;
    let mut am: intake_issues::ActiveModel = ii.into();

    if let Some(status) = body.status {
        am.status = Set(status);
        // Al aceptar, promover el issue de draft a activo
        if status == STATUS_ACCEPTED {
            if let Some(issue) = issues::Entity::find_by_id(issue_id)
                .one(&state.db)
                .await
                .map_err(AppError::Database)?
            {
                let mut iam: issues::ActiveModel = issue.into();
                iam.is_draft = Set(false);
                iam.update(&state.db).await.map_err(AppError::Database)?;
            }
        }
    }
    if body.snoozed_till.is_some() {
        am.snoozed_till = Set(body.snoozed_till);
    }
    if body.duplicate_to_id.is_some() {
        am.duplicate_to_id = Set(body.duplicate_to_id);
    }
    if let Some(src) = body.source {
        am.source = Set(Some(src));
    }
    am.updated_by_id = Set(Some(guard.user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(IntakeIssueResponse::from_model(updated)))
}

// ── DELETE /intake-issues/{pk}/ ───────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/",
    tag = "Intake",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "IntakeIssue ID"),
    ),
    responses((status = 204, description = "Eliminado")),
    security(("TokenAuth" = []))
)]
pub async fn delete_intake_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let ii = intake_issues::Entity::find_by_id(pk)
        .active()
        .filter(intake_issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: intake_issues::ActiveModel = ii.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
