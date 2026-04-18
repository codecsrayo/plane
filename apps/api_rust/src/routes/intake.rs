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
    entities::{intake_issues, intakes, issues, states},
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

// ── Source constants (matches Django SourceType.TextChoices) ──────────────────
// Ver: apps/api/plane/db/models/intake.py → class SourceType.
// Django sólo define IN_APP por ahora; el handler de create hard-codea este
// valor e ignora el `source` que venga del cliente (base.py:270).
pub const SOURCE_IN_APP: &str = "IN_APP";

// ── Priority constants (matches Django IssueCreateSerializer validation) ──────
pub const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "urgent", "none"];

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
/// Detalle del issue anidado dentro de IntakeIssueResponse.
///
/// Paridad con Django `IssueDetailSerializer` (serializers/issue.py:924-935)
/// usado por `IntakeIssueDetailSerializer` (serializers/intake.py:93-107).
/// El frontend lee `inboxIssue.issue.created_by` (root.tsx:77,80),
/// `issue.name`, `issue.description_html`, `issue.priority`, `issue.sequence_id`,
/// `issue.label_ids`, `issue.assignee_ids` — todos obligatorios.
///
/// TODO(paridad-agregados): los campos de conteo (sub_issues_count,
/// attachment_count, link_count) y los *_ids (label_ids, assignee_ids,
/// module_ids) requieren queries extra (ArrayAgg en Django). Por ahora se
/// devuelven con valores por defecto (0 / vec vacío) para desbloquear el
/// frontend. Refactor pendiente: hacer batch en list_intake_issues.
pub struct IntakeIssueNestedIssue {
    pub id: Uuid,
    pub name: String,
    pub description_html: String,
    pub state_id: Option<Uuid>,
    pub priority: String,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub sort_order: f64,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub archived_at: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    /// Django serializa el FK `created_by` como UUID sin `_id`.
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub is_draft: bool,
    /// Django `is_intake` se calcula en runtime. Siempre `false` aquí porque
    /// el issue aún está en draft/intake — el flag solo se vuelve true tras
    /// aceptarse. TODO: calcular correctamente.
    pub is_intake: bool,
    pub estimate_point: Option<Uuid>,
    pub cycle_id: Option<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub sub_issues_count: i64,
    pub attachment_count: i64,
    pub link_count: i64,
}

impl IntakeIssueNestedIssue {
    /// Construye el detalle del issue desde el Model, con agregados en valores
    /// por defecto. Ver TODO en la doc del struct.
    fn from_model(m: &issues::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            description_html: m.description_html.clone(),
            state_id: m.state_id,
            priority: m.priority.clone(),
            sequence_id: m.sequence_id,
            project_id: m.project_id,
            parent_id: m.parent_id,
            sort_order: m.sort_order,
            start_date: m.start_date,
            target_date: m.target_date,
            completed_at: m.completed_at,
            archived_at: m.archived_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            is_draft: m.is_draft,
            is_intake: false,
            estimate_point: m.estimate_point_id,
            cycle_id: None,
            label_ids: Vec::new(),
            assignee_ids: Vec::new(),
            module_ids: Vec::new(),
            sub_issues_count: 0,
            attachment_count: 0,
            link_count: 0,
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
    /// Detalle del issue asociado — requerido por el frontend. Ver
    /// IntakeIssueNestedIssue.
    pub issue: IntakeIssueNestedIssue,
}

impl IntakeIssueResponse {
    fn from_joined(ii: intake_issues::Model, issue: &issues::Model) -> Self {
        Self {
            id: ii.id,
            issue_id: ii.issue_id,
            intake_id: ii.intake_id,
            status: ii.status,
            source: ii.source,
            snoozed_till: ii.snoozed_till,
            duplicate_to_id: ii.duplicate_to_id,
            project_id: ii.project_id,
            workspace_id: ii.workspace_id,
            created_by_id: ii.created_by_id,
            created_at: ii.created_at,
            issue: IntakeIssueNestedIssue::from_model(issue),
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

/// Body del POST /intake-issues/ — paridad con Django IntakeIssueViewSet.create
/// (`apps/api/plane/app/views/intake/base.py:222-284`).
///
/// El frontend envía el payload anidado `{source, issue: {...}}`. Django **ignora**
/// cualquier `intake_id` que venga en el body y auto-resuelve el intake del
/// proyecto (`base.py:264`). El `source` del cliente también se descarta y se
/// hard-codea a `IN_APP` (`base.py:270`). Mantenemos `source` en el DTO para
/// log/telemetría, pero nunca se usa al persistir.
///
/// Campos adicionales que el frontend puede mandar en `issue` (parent_id,
/// start_date, target_date, estimate_point, type_id, assignee_ids, label_ids)
/// los procesa Django vía `IssueCreateSerializer`. Aquí serde los descarta
/// silenciosamente — ver TODO en el handler.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeIssueRequest {
    /// Ignorado por paridad con Django (hard-code `IN_APP`). Aceptado para no
    /// reventar clientes que lo envíen.
    #[serde(default)]
    pub source: Option<String>,
    pub issue: CreateIntakeIssueBody,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIntakeIssueBody {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
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

    // created_at/updated_at explícitos (NOT NULL sin DEFAULT).
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

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
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
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

    // Batch fetch de los issues asociados. Django usa
    // `.select_related("issue")` en base.py (prefetch inline); nosotros
    // hacemos un segundo query con `IN` para evitar N+1 y evitar depender de
    // `find_also_related` (que tiene ambigüedad con Issues1/Issues2 en la
    // entidad intake_issues — hay dos FKs hacia issues: issue_id y
    // duplicate_to_id).
    //
    // Issues soft-deleted: si un issue_id apunta a un issue borrado, ese
    // intake_issue se omite del response (Django también los excluye por el
    // default manager que filtra deleted_at=null).
    if rows.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let issue_ids: Vec<Uuid> = rows.iter().map(|r| r.issue_id).collect();
    let issues_by_id: std::collections::HashMap<Uuid, issues::Model> = issues::Entity::find()
        .active()
        .filter(issues::Column::Id.is_in(issue_ids))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .map(|i| (i.id, i))
        .collect();

    let responses: Vec<IntakeIssueResponse> = rows
        .into_iter()
        .filter_map(|ii| {
            issues_by_id
                .get(&ii.issue_id)
                .map(|issue| IntakeIssueResponse::from_joined(ii, issue))
        })
        .collect();

    Ok(Json(responses))
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
    // Django usa @allow_permission([ADMIN, MEMBER, GUEST]) — el intake acepta
    // tickets de guests (base.py:221). Bajamos a ROLE_GUEST.
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // ── Validaciones (paridad Django base.py:223-234) ─────────────────────────
    if body.issue.name.trim().is_empty() {
        return Err(AppError::BadRequest("Name is required".into()));
    }

    if let Some(ref p) = body.issue.priority {
        if !VALID_PRIORITIES.contains(&p.as_str()) {
            return Err(AppError::BadRequest("Invalid priority".into()));
        }
    }

    // ── Auto-resolver o crear intake del proyecto ────────────────────────────
    //
    // Django (base.py:264) hace `Intake.objects.filter(...).first()` y asume
    // que existe — si no existe, crashearía con AttributeError 500 en
    // `intake_id.id`. Nosotros somos más defensivos: hacemos get-or-create
    // inline para reparar proyectos donde `projects.intake_view=true` pero la
    // fila del intake nunca se creó (bug histórico de update_project en el API
    // Rust, fix separado — ver projects.rs).
    //
    // Aún así, si el proyecto tiene `intake_view=false`, devolvemos 400 para
    // no crear intakes en proyectos que explícitamente no lo tienen habilitado.
    //
    // NOTA: ignoramos cualquier `intake_id` que venga en el body. El DTO ya no
    // lo acepta (paridad estricta con Django — él también lo descarta).
    if !guard.project.intake_view {
        return Err(AppError::BadRequest(
            "Intake no está habilitado en este proyecto".into(),
        ));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let intake = if let Some(existing) = intakes::Entity::find()
        .active()
        .filter(intakes::Column::ProjectId.eq(guard.project.id))
        .filter(intakes::Column::WorkspaceId.eq(guard.workspace.id))
        .order_by_asc(intakes::Column::CreatedAt)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    {
        existing
    } else {
        // Mismos valores que Django (project/base.py:356-360):
        // name=f"{project.name} Intake", is_default=true.
        // `view_props` y `logo_props` son JSON NOT NULL con default {} en la DB.
        intakes::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(format!("{} Intake", guard.project.name)),
            description: Set(String::new()),
            is_default: Set(true),
            view_props: Set(serde_json::json!({})),
            logo_props: Set(serde_json::json!({})),
            project_id: Set(guard.project.id),
            workspace_id: Set(guard.workspace.id),
            created_by_id: Set(Some(guard.user.id)),
            updated_by_id: Set(Some(guard.user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?
    };

    // ── Get-or-create triage state (paridad Django base.py:239-249) ───────────
    //
    // TODO(refactor): esta lógica es idéntica a routes::states::intake_state.
    // Extraer a `pub(crate) fn ensure_triage_state(db, workspace_id, project_id)`
    // en un helper compartido.

    let triage_state = if let Some(existing) = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(guard.project.id))
        .filter(states::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(states::Column::IsTriage.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    {
        existing
    } else {
        // Mismos valores que Django (base.py:241-249):
        // name="Triage", color="#4E5355", sequence=65000, group="triage",
        // default=false, is_triage=true. El estado triage es de sistema
        // (sin propietario explícito), igual que en routes::states::intake_state.
        states::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Triage".to_string()),
            description: Set(String::new()),
            color: Set("#4E5355".to_string()),
            slug: Set("triage".to_string()),
            group: Set("triage".to_string()),
            sequence: Set(65000.0),
            default: Set(false),
            is_triage: Set(true),
            project_id: Set(guard.project.id),
            workspace_id: Set(guard.workspace.id),
            created_by_id: Set(None),
            updated_by_id: Set(None),
            external_id: Set(None),
            external_source: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?
    };

    // ── Calcular sequence_id ──────────────────────────────────────────────────
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

    // ── Crear el issue (paridad Django IssueCreateSerializer.save) ────────────
    //
    // DIVERGENCIA: Django usa IssueCreateSerializer que acepta parent_id,
    // start_date, target_date, estimate_point, type_id, assignee_ids,
    // label_ids. Aquí esos campos se descartan silenciosamente por serde.
    //
    // TODO(paridad-completa): añadir esos campos a CreateIntakeIssueBody,
    // envolver issue INSERT + sync_assignees + sync_labels en una transacción
    // (patrón ya existe en issues::create_issue), y hacer pub(crate) las
    // funciones sync_assignees/sync_labels de issues.rs.
    //
    // is_draft=false: paridad Django. IssueCreateSerializer no marca is_draft,
    // y el campo es bool NOT NULL con default false en la DB. El tweak
    // anterior (is_draft=true) era una divergencia nuestra.
    let issue = issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.issue.name),
        description_html: Set(body.issue.description_html.unwrap_or_default()),
        description_json: Set(serde_json::json!({})),
        priority: Set(body.issue.priority.unwrap_or_else(|| "none".to_owned())),
        sequence_id: Set(sequence_id),
        sort_order: Set(65535.0),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        state_id: Set(Some(triage_state.id)), // Django base.py:250
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        is_draft: Set(false),
        description_stripped: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // ── Crear el intake_issue (paridad Django base.py:266-271) ────────────────
    //
    // `source` hard-codeado a IN_APP — Django ignora el valor del cliente
    // (base.py:270). El `body.source` solo se acepta para no reventar clientes
    // viejos; se descarta aquí.
    let _ = body.source; // silence unused-field lint; ver doc del DTO
    let intake_issue = intake_issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        issue_id: Set(issue.id),
        intake_id: Set(intake.id),
        status: Set(STATUS_PENDING),
        source: Set(Some(SOURCE_IN_APP.to_string())),
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

    Ok((StatusCode::CREATED, Json(IntakeIssueResponse::from_joined(intake_issue, &issue))))
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

    // Fetch del issue asociado (paridad Django select_related("issue")).
    // Si el issue está soft-deleted, devolvemos 404 — el intake_issue sin
    // issue válido es un estado inconsistente.
    let issue = issues::Entity::find_by_id(ii.issue_id)
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(IntakeIssueResponse::from_joined(ii, &issue)))
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

    // Fetch del issue tras el update. Si status=ACCEPTED se promovió is_draft
    // en el bloque de arriba; necesitamos la versión post-update para reflejar
    // ese cambio en el response.
    let issue = issues::Entity::find_by_id(updated.issue_id)
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(IntakeIssueResponse::from_joined(updated, &issue)))
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
