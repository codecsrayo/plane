// src/routes/issues.rs
//! Endpoints de Issues (work items).
//!
//! Endpoints implementados:
//!   GET    /api/workspaces/{slug}/projects/{project_id}/issues/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/issues/
//!   GET    /api/workspaces/{slug}/projects/{project_id}/issues/{pk}/
//!   PATCH  /api/workspaces/{slug}/projects/{project_id}/issues/{pk}/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/issues/{pk}/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, PaginatorTrait,
    QueryFilter, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{issue_assignees, issue_labels, issues, labels},
    error::AppError,
    routes::issue_pagination::{
        apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
        load_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
    },
    routes::issue_filters::{apply_issue_filters, reject_if_rich_filters, FilteredQuery},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Shape de `GET /workspaces/{slug}/projects/{project_id}/issues/{pk}/`.
///
/// Espejo EXACTO de `IssueDetailSerializer` en
/// `apps/api/plane/app/serializers/issue.py:924-935`, que extiende
/// `IssueSerializer` (línea 760) con `description_html`, `is_subscribed`,
/// `is_intake`. El frontend (`packages/types/src/issues/issue.ts:TIssue`)
/// consume exactamente este shape.
///
/// # Diferencias con el DTO previo (`IssueResponse`)
/// - Sin `workspace_id` — Django no lo incluye en el serializer de detail.
/// - Sin `type_id` — tampoco está en `IssueSerializer.Meta.fields`.
/// - Renombres: `created_by_id → created_by`, `updated_by_id → updated_by`,
///   `estimate_point_id → estimate_point` (convención Django cuando el campo
///   se declara como FK en el serializer, no como UUIDField crudo).
/// - Añadidos: `cycle_id`, `module_ids`, `sub_issues_count`, `attachment_count`,
///   `link_count`, `is_subscribed`, `is_intake`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub state_id: Option<Uuid>,
    pub sort_order: f64,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    // Django expone el FK de estimate como `estimate_point` (source de la FK),
    // no como `estimate_point_id` crudo. El frontend lee `estimate_point`.
    #[serde(rename = "estimate_point")]
    pub estimate_point_id: Option<Uuid>,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    // Enriquecido — primer cycle activo asociado al issue.
    pub cycle_id: Option<Uuid>,
    // Enriquecidos — arrays de IDs de relaciones M2M.
    pub module_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    // Enriquecidos — contadores agregados.
    pub sub_issues_count: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    // Django renderiza los FKs de auditoría como `created_by`/`updated_by`
    // (no `_id`) porque el serializer los declara como ForeignKey fields.
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub attachment_count: i64,
    pub link_count: i64,
    pub is_draft: bool,
    pub archived_at: Option<chrono::NaiveDate>,
    // Extras propios de `IssueDetailSerializer` (no están en el shape list).
    pub description_html: String,
    pub is_subscribed: bool,
    pub is_intake: bool,
}

/// Shape de `POST /workspaces/{slug}/projects/{project_id}/issues/`.
///
/// Espejo EXACTO de la proyección `.values(...)` que Django usa en
/// `apps/api/plane/app/views/issue/base.py:427-454` tras crear un issue.
///
/// # Diferencias con `IssueDetailResponse`
/// - Incluye `deleted_at` (siempre `null` inmediatamente tras create, pero
///   Django lo proyecta — el frontend puede leerlo sin romperse).
/// - Omite `description_html`, `is_subscribed`, `is_intake` — la vista de
///   create no los expone.
///
/// Mantener ambos shapes separados evita el antipatrón de "DTO unión con
/// todos los campos opcionales", que pierde información y confunde al
/// consumidor sobre qué endpoint está llamando.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueCreateResponse {
    pub id: Uuid,
    pub name: String,
    pub state_id: Option<Uuid>,
    pub sort_order: f64,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    #[serde(rename = "estimate_point")]
    pub estimate_point_id: Option<Uuid>,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub cycle_id: Option<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    pub sub_issues_count: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub attachment_count: i64,
    pub link_count: i64,
    pub is_draft: bool,
    pub archived_at: Option<chrono::NaiveDate>,
    pub deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIssueRequest {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub estimate_point_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub assignees: Option<Vec<Uuid>>,
    pub labels: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub estimate_point_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub assignees: Option<Vec<Uuid>>,
    pub labels: Option<Vec<Uuid>>,
    pub is_draft: Option<bool>,
}

// ── Query params para list_issues ────────────────────────────────────────────

/// Query params de `GET /workspaces/{slug}/projects/{project_id}/issues/`.
/// Mirror parcial de `IssueViewSet.list()` (apps/api/plane/app/views/issue/base.py:251).
///
/// Campos postergados a futuras iteraciones (no críticos para desbloquear el
/// render del panel de integrations):
///   - `group_by` / `sub_group_by`    → requieren paginators dedicados.
///   - filtros de `issue_filters(...)` → labels, assignees, priority, etc.
#[derive(Debug, Deserialize)]
pub struct ListIssuesQuery {
    // ── Paginación / orden ────────────────────────────────────────────────────
    pub cursor:   Option<String>,
    pub per_page: Option<u64>,
    pub order_by: Option<String>,

    // ── Toggles simples ───────────────────────────────────────────────────────
    /// `false` excluye sub-issues (mirror de `filter_sub_issue_toggle`
    /// en plane/utils/issue_filters.py:380).
    pub sub_issue: Option<String>,
    /// Filtro incremental — solo issues actualizados después de este timestamp.
    /// Mirror del `updated_at__gt` en base.py:256.
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,

    // ── Filtros delegados al módulo `issue_filters` ──────────────────────────
    //
    // Se inlinean aquí porque axum's `Query<T>` usa `serde_urlencoded`, que
    // no soporta `#[serde(flatten)]`. La duplicación entre este struct y
    // `WorkspaceIssuesQuery` es el precio — la lógica de parsing y
    // aplicación vive en un solo lugar (`issue_filters::apply_issue_filters`).
    pub state:             Option<String>,
    pub state_group:       Option<String>,
    pub priority:          Option<String>,
    pub created_by:        Option<String>,
    pub parent:            Option<String>,
    pub name:              Option<String>,
    pub start_date:        Option<String>,
    pub target_date:       Option<String>,
    pub labels:            Option<String>,
    pub assignees:         Option<String>,
    pub module:            Option<String>,
    pub cycle:             Option<String>,
    #[serde(rename = "type")]
    pub type_filter:       Option<String>,
    pub start_target_date: Option<String>,

    // ── Rich filters (known gap vs Django) ───────────────────────────────────
    //
    // Capturado solo para detectar su presencia y rechazar con 400 antes de
    // ejecutar la query. NO se parsea — ver
    // `issue_filters::reject_if_rich_filters` para el contexto completo.
    pub filters:           Option<String>,
}

impl ListIssuesQuery {
    /// Construye los parámetros de filtro para pasar a
    /// `issue_filters::apply_issue_filters`. Los campos son `Option<String>`,
    /// así que el move es barato (no allocations extra).
    fn to_filter_params(&self) -> crate::routes::issue_filters::IssueFilterParams {
        crate::routes::issue_filters::IssueFilterParams {
            state:             self.state.clone(),
            state_group:       self.state_group.clone(),
            priority:          self.priority.clone(),
            created_by:        self.created_by.clone(),
            parent:            self.parent.clone(),
            name:              self.name.clone(),
            start_date:        self.start_date.clone(),
            target_date:       self.target_date.clone(),
            labels:            self.labels.clone(),
            assignees:         self.assignees.clone(),
            module:            self.module.clone(),
            cycle:             self.cycle.clone(),
            type_filter:       self.type_filter.clone(),
            start_target_date: self.start_target_date.clone(),
        }
    }
}

// ── DTO serializado en la respuesta paginada ─────────────────────────────────

/// Espejo exacto de `issue_on_results` en
/// `apps/api/plane/utils/grouper.py:93-141`.
///
/// Son los 23 campos que Django expone vía `.values(*required_fields)` en el
/// listado paginado, más los tres arrays enriquecidos (`assignee_ids`,
/// `label_ids`, `module_ids`).
///
/// Los nombres se serializan exactamente como en Django para que el frontend
/// (`base-issues.store.ts` + `packages/types/src/issues/issue.ts`) no requiera
/// cambios. `state__group` usa rename explícito para respetar el doble-guion
/// bajo de Django.
#[derive(Debug, Serialize)]
pub struct ProjectIssueItem {
    pub id:               Uuid,
    pub name:             String,
    pub state_id:         Option<Uuid>,
    pub sort_order:       f64,
    pub completed_at:     Option<chrono::DateTime<chrono::FixedOffset>>,
    pub estimate_point:   Option<Uuid>,
    pub priority:         String,
    pub start_date:       Option<chrono::NaiveDate>,
    pub target_date:      Option<chrono::NaiveDate>,
    pub sequence_id:      i32,
    pub project_id:       Uuid,
    pub parent_id:        Option<Uuid>,
    pub cycle_id:         Option<Uuid>,
    pub sub_issues_count: i64,
    pub created_at:       chrono::DateTime<chrono::FixedOffset>,
    pub updated_at:       chrono::DateTime<chrono::FixedOffset>,
    pub created_by:       Option<Uuid>,
    pub updated_by:       Option<Uuid>,
    pub attachment_count: i64,
    pub link_count:       i64,
    pub is_draft:         bool,
    pub archived_at:      Option<chrono::NaiveDate>,
    #[serde(rename = "state__group")]
    pub state_group:      Option<String>,
    pub assignee_ids:     Vec<Uuid>,
    pub label_ids:        Vec<Uuid>,
    pub module_ids:       Vec<Uuid>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Enriquece un lote de issues y los convierte al shape de `IssueDetailSerializer`
/// de Django (`GET /issues/{pk}/`).
///
/// # Estado actual (commit 1 del refactor)
/// Carga batch de **assignees** y **labels** — los dos M2M más simples.
/// Los siguientes campos se devuelven con valor por defecto (stub):
///   - `cycle_id`, `module_ids`, `sub_issues_count`, `attachment_count`,
///     `link_count`, `is_subscribed`, `is_intake`.
///
/// Commit 2 del refactor reemplaza este helper por llamadas directas a
/// `load_enrichment` (que ya carga cycle/modules/counts) más queries
/// específicas para `is_subscribed` / `is_intake`. Se mantiene aquí como
/// puente temporal para que el shape sea correcto sin acoplar el cableo.
///
/// # Antipatrón evitado
/// N+1: ambos `is_in()` hacen 1 query por relación, independiente del
/// tamaño del lote. Para un solo issue (caso GET detail) la diferencia
/// no importa, pero mantiene el contrato uniforme.
async fn enrich_issues(
    db: &sea_orm::DatabaseConnection,
    issue_models: Vec<issues::Model>,
) -> Result<Vec<IssueDetailResponse>, AppError> {
    if issue_models.is_empty() {
        return Ok(vec![]);
    }

    let ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();

    // Batch-fetch assignees — evita N+1
    let assignees = issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.is_in(ids.clone()))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut assignee_map: std::collections::HashMap<Uuid, Vec<Uuid>> =
        std::collections::HashMap::new();
    for a in assignees {
        assignee_map.entry(a.issue_id).or_default().push(a.assignee_id);
    }

    // Batch-fetch labels — evita N+1
    let label_rows = issue_labels::Entity::find()
        .filter(issue_labels::Column::IssueId.is_in(ids))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut label_map: std::collections::HashMap<Uuid, Vec<Uuid>> =
        std::collections::HashMap::new();
    for l in label_rows {
        label_map.entry(l.issue_id).or_default().push(l.label_id);
    }

    Ok(issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            IssueDetailResponse {
                id,
                name: m.name,
                state_id: m.state_id,
                sort_order: m.sort_order,
                completed_at: m.completed_at,
                estimate_point_id: m.estimate_point_id,
                priority: m.priority,
                start_date: m.start_date,
                target_date: m.target_date,
                sequence_id: m.sequence_id,
                project_id: m.project_id,
                parent_id: m.parent_id,
                // Stubs — commit 2 los reemplaza por `load_enrichment`.
                cycle_id: None,
                module_ids: Vec::new(),
                sub_issues_count: 0,
                attachment_count: 0,
                link_count: 0,
                is_subscribed: false,
                is_intake: false,
                // Enriquecidos reales:
                label_ids: label_map.remove(&id).unwrap_or_default(),
                assignee_ids: assignee_map.remove(&id).unwrap_or_default(),
                // Flat fields del modelo:
                description_html: m.description_html,
                created_at: m.created_at,
                updated_at: m.updated_at,
                created_by_id: m.created_by_id,
                updated_by_id: m.updated_by_id,
                is_draft: m.is_draft,
                archived_at: m.archived_at,
            }
        })
        .collect())
}

/// Construye el shape de respuesta de `POST /issues/` — espejo de la
/// proyección `.values(...)` en `base.py:427-454`.
///
/// # Estado actual (commit 1)
/// Reutiliza `enrich_issues` para obtener assignees/labels (los únicos
/// enriquecidos reales del commit 1) y luego copia los campos flat al DTO
/// de create. Los stubs (`cycle_id`, `module_ids`, counts) permanecen en
/// defaults hasta commit 2.
///
/// # Nota sobre `deleted_at`
/// Inmediatamente tras un create siempre es `None`, pero Django lo incluye
/// en la proyección. Lo respetamos para paridad estricta de shape.
async fn build_create_response(
    db: &sea_orm::DatabaseConnection,
    issue_model: issues::Model,
) -> Result<IssueCreateResponse, AppError> {
    let deleted_at = issue_model.deleted_at;
    let mut detail = enrich_issues(db, vec![issue_model]).await?;
    let d = detail.pop().ok_or(AppError::NotFound)?;

    Ok(IssueCreateResponse {
        id: d.id,
        name: d.name,
        state_id: d.state_id,
        sort_order: d.sort_order,
        completed_at: d.completed_at,
        estimate_point_id: d.estimate_point_id,
        priority: d.priority,
        start_date: d.start_date,
        target_date: d.target_date,
        sequence_id: d.sequence_id,
        project_id: d.project_id,
        parent_id: d.parent_id,
        cycle_id: d.cycle_id,
        module_ids: d.module_ids,
        label_ids: d.label_ids,
        assignee_ids: d.assignee_ids,
        sub_issues_count: d.sub_issues_count,
        created_at: d.created_at,
        updated_at: d.updated_at,
        created_by_id: d.created_by_id,
        updated_by_id: d.updated_by_id,
        attachment_count: d.attachment_count,
        link_count: d.link_count,
        is_draft: d.is_draft,
        archived_at: d.archived_at,
        deleted_at,
    })
}

/// Sincroniza los assignees de un issue dentro de una transacción.
/// Soft-delete los existentes que no estén en `new_ids`, inserta los nuevos.
async fn sync_assignees(
    txn: &sea_orm::DatabaseTransaction,
    issue_id: Uuid,
    project_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    // Soft-delete todos los existentes
    let existing = issue_assignees::Entity::find()
        .filter(issue_assignees::Column::IssueId.eq(issue_id))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .all(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for row in existing {
        let mut am: issue_assignees::ActiveModel = row.into();
        am.deleted_at = Set(Some(now));
        am.update(txn).await.map_err(AppError::Database)?;
    }

    // Insertar nuevos
    for assignee_id in new_ids {
        issue_assignees::ActiveModel {
            id: Set(Uuid::new_v4()),
            issue_id: Set(issue_id),
            assignee_id: Set(*assignee_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            ..Default::default()
        }
        .insert(txn)
        .await
        .map_err(AppError::Database)?;
    }
    Ok(())
}

/// Sincroniza los labels de un issue dentro de una transacción.
async fn sync_labels(
    txn: &sea_orm::DatabaseTransaction,
    issue_id: Uuid,
    project_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
    new_ids: &[Uuid],
) -> Result<(), AppError> {
    let existing = issue_labels::Entity::find()
        .filter(issue_labels::Column::IssueId.eq(issue_id))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .all(txn)
        .await
        .map_err(AppError::Database)?;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for row in existing {
        let mut am: issue_labels::ActiveModel = row.into();
        am.deleted_at = Set(Some(now));
        am.update(txn).await.map_err(AppError::Database)?;
    }

    for label_id in new_ids {
        // Verificar que el label pertenece al proyecto antes de insertar
        let exists = labels::Entity::find_by_id(*label_id)
            .filter(labels::Column::ProjectId.eq(project_id))
            .filter(labels::Column::DeletedAt.is_null())
            .one(txn)
            .await
            .map_err(AppError::Database)?;

        if exists.is_some() {
            issue_labels::ActiveModel {
                id: Set(Uuid::new_v4()),
                issue_id: Set(issue_id),
                label_id: Set(*label_id),
                project_id: Set(project_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(actor_id)),
                updated_by_id: Set(Some(actor_id)),
                ..Default::default()
            }
            .insert(txn)
            .await
            .map_err(AppError::Database)?;
        }
    }
    Ok(())
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues/ ──────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/",
    tag = "Issues",
    params(
        ("slug"       = String, Path,  description = "Workspace slug"),
        ("project_id" = Uuid,   Path,  description = "Project ID"),
        ("cursor"     = Option<String>, Query, description = "Cursor Django: {per_page}:{page}:{is_prev}"),
        ("per_page"   = Option<u64>,    Query, description = "Tamaño de página (ignorado si viene en cursor)"),
        ("order_by"   = Option<String>, Query, description = "Campo de ordenamiento, ej: -created_at"),
    ),
    responses(
        (status = 200, description = "Lista paginada de issues del proyecto"),
        (status = 403, description = "Sin acceso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<ListIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    // Rechaza `?filters=<JSON>` antes de ejecutar nada — el frontend lo envía
    // cuando hay un rich-filter tree activo (views guardados, etc.). El port
    // Rust aún no implementa `ComplexFilterBackend`; aceptar la request sin
    // aplicar el filtro devolvería resultados incorrectos silenciosamente.
    reject_if_rich_filters(params.filters.as_deref())?;

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    // ── 1. Paginación ─────────────────────────────────────────────────────────
    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // ── 2. Mirror de `IssueManager.get_queryset` (db/models/issue.py:92-101) ──
    //
    // El manager Django aplica 4 exclusiones implícitas a TODOS los listados
    // que parten de `Issue.issue_objects`. La lista hereda además el
    // `SoftDeletionManager.active()` → `deleted_at IS NULL`.
    //
    //   1. deleted_at IS NULL                    ← `.active()`
    //   2. archived_at IS NULL                   ← filtro explícito
    //   3. project.archived_at IS NULL           ← early-return si el project
    //                                               cargado por el guard ya
    //                                               está archivado. No puede
    //                                               cambiar dentro de esta
    //                                               request.
    //   4. is_draft = false                      ← filtro explícito
    //   5. state.group != 'triage'               ← pre-query de state_ids en
    //                                               triage + NOT IN.
    //
    // Antipatrón evitado: NO hacemos JOIN con `states` en cada query; los
    // state IDs en triage se resuelven en una única query adicional.

    if guard.project.archived_at.is_some() {
        // Proyecto archivado → no hay issues visibles (mirror de la exclusión
        // del manager). Respondemos con el shape paginado vacío.
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let triage_state_ids = load_triage_state_ids(db, project_id).await?;

    // ── 3. Query base de issues del proyecto ──────────────────────────────────
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::IsDraft.eq(false));

    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    // ── 4. Restricción guest ──────────────────────────────────────────────────
    //
    // Mirror de base.py:297-308: si el user es role=5 en este proyecto Y el
    // proyecto tiene `guest_view_all_features=false`, solo ve sus propios
    // issues (`created_by = user`).
    //
    // El guard ya validó la membresía; `project_member.role` es el rol en
    // este proyecto específico.
    let is_restricted_guest = guard.project_member.role == 5 && !guard.project.guest_view_all_features;
    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    // Toggle de sub-issues (mirror de `filter_sub_issue_toggle` en
    // plane/utils/issue_filters.py:380). `sub_issue=false` oculta issues que
    // tengan parent; cualquier otro valor (o ausencia) muestra todo.
    if matches!(params.sub_issue.as_deref(), Some("false")) {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // Sync delta (`updated_at__gt` en base.py:256). El frontend usa este
    // filtro para refrescar solo lo que cambió desde el último poll.
    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // ── 4.5. Filtros del módulo compartido ────────────────────────────────────
    //
    // `state`, `state_group`, `priority`, `created_by`, `parent`, `name`,
    // `start_date`, `target_date`, `labels`, `assignees`, `module`, `cycle`,
    // `type`, `start_target_date`. Ver `routes::issue_filters` para la
    // lista completa y el mapeo detallado.
    //
    // `FilteredQuery::Empty` → algún filtro implica 0 matches garantizados
    // (p. ej. `labels=<uuid>` sin ningún issue con ese label). Hacemos
    // early-return con el shape paginado vacío.
    let workspace_id = guard.workspace.id;
    let filter_params = params.to_filter_params();
    let filtered = apply_issue_filters(db, base_query, &filter_params, workspace_id).await?;
    let base_query = match filtered {
        FilteredQuery::Active(q) => q,
        FilteredQuery::Empty => return Ok(Json(empty_paginated_response(page_size))),
    };

    // ── 5. Total count ────────────────────────────────────────────────────────
    let total_results = base_query.clone().count(db).await.map_err(AppError::Database)?;

    // ── 6. Ordenamiento ───────────────────────────────────────────────────────
    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    // ── 7. Paginación offset ──────────────────────────────────────────────────
    let start_index = current_page * page_size;
    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── 8. Enriquecimiento batch (sin N+1) ────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);

    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    // ── 9. Serializar a ProjectIssueItem (mirror `issue_on_results`) ──────────
    let results: Vec<ProjectIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());

            ProjectIssueItem {
                id,
                name: m.name,
                state_id: m.state_id,
                sort_order: m.sort_order,
                completed_at: m.completed_at,
                estimate_point: m.estimate_point_id,
                priority: m.priority,
                start_date: m.start_date,
                target_date: m.target_date,
                sequence_id: m.sequence_id,
                project_id: m.project_id,
                parent_id: m.parent_id,
                cycle_id: enrich.cycles.remove(&id),
                sub_issues_count: enrich.sub_counts.get(&id).copied().unwrap_or(0),
                created_at: m.created_at,
                updated_at: m.updated_at,
                created_by: m.created_by_id,
                updated_by: m.updated_by_id,
                attachment_count: enrich.attachments.get(&id).copied().unwrap_or(0),
                link_count: enrich.links.get(&id).copied().unwrap_or(0),
                is_draft: m.is_draft,
                archived_at: m.archived_at,
                state_group,
                assignee_ids: enrich.assignees.remove(&id).unwrap_or_default(),
                label_ids: enrich.labels.remove(&id).unwrap_or_default(),
                module_ids: enrich.modules.remove(&id).unwrap_or_default(),
            }
        })
        .collect();

    // ── 10. Respuesta paginada ────────────────────────────────────────────────
    //
    // Shape mirror de `OffsetPaginator.paginate()` (plane/utils/paginator.py:715-730).
    // El frontend lee `total_count` y `results` en base-issues.store.ts:1272-1291.
    Ok(Json(paginated_response(
        results,
        page_size,
        current_page,
        total_results,
    )))
}

// ── POST /workspaces/{slug}/projects/{project_id}/issues/ ─────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 201, description = "Issue creado"),
        (status = 400, description = "Error de validación"),
        (status = 403, description = "Sin permiso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<CreateIssueRequest>,
) -> Result<(StatusCode, Json<IssueCreateResponse>), AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name es requerido".into()));
    }

    // Obtener el siguiente sequence_id de forma atómica dentro de SERIALIZABLE.
    // Antipatrón corregido: MAX() fuera de transacción es TOCTOU bajo concurrencia.
    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let assignees = body.assignees.clone().unwrap_or_default();
    let label_ids = body.labels.clone().unwrap_or_default();

    let issue = state
        .db
        .transaction_with_config::<_, issues::Model, AppError>(
            |txn| {
                let name = body.name.clone();
                let description_html = body.description_html.clone().unwrap_or_default();
                let priority = body.priority.clone().unwrap_or_else(|| "none".to_owned());
                let state_id = body.state_id;
                let parent_id = body.parent_id;
                let start_date = body.start_date;
                let target_date = body.target_date;
                let estimate_point_id = body.estimate_point_id;
                let type_id = body.type_id;
                let assignees = assignees.clone();
                let label_ids = label_ids.clone();
                Box::pin(async move {
                    // sequence_id = MAX(sequence_id) + 1 dentro del proyecto
                    use sea_orm::QuerySelect;
                    let max_seq: Option<i64> = issues::Entity::find()
                        .filter(issues::Column::ProjectId.eq(project_id))
                        .select_only()
                        .column_as(
                            sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
                            "max_seq",
                        )
                        .into_tuple()
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?;

                    let sequence_id = (max_seq.unwrap_or(0) + 1) as i32;

                    let new_issue = issues::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        name: Set(name),
                        description_html: Set(description_html),
                        description_json: Set(serde_json::json!({})),
                        priority: Set(priority),
                        state_id: Set(state_id),
                        parent_id: Set(parent_id),
                        start_date: Set(start_date),
                        target_date: Set(target_date),
                        sequence_id: Set(sequence_id),
                        sort_order: Set(65535.0),
                        project_id: Set(project_id),
                        workspace_id: Set(workspace_id),
                        created_by_id: Set(Some(user_id)),
                        updated_by_id: Set(Some(user_id)),
                        estimate_point_id: Set(estimate_point_id),
                        type_id: Set(type_id),
                        is_draft: Set(false),
                        description_stripped: Set(None),
                        ..Default::default()
                    };

                    let issue = new_issue.insert(txn).await.map_err(AppError::Database)?;

                    // Sync assignees y labels dentro de la misma transacción
                    if !assignees.is_empty() {
                        sync_assignees(txn, issue.id, project_id, workspace_id, user_id, &assignees).await?;
                    }
                    if !label_ids.is_empty() {
                        sync_labels(txn, issue.id, project_id, workspace_id, user_id, &label_ids).await?;
                    }

                    Ok(issue)
                })
            },
            Some(IsolationLevel::Serializable),
            None,
        )
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    let response = build_create_response(&state.db, issue).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues/{pk}/ ─────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 200, description = "Detalle del issue"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<IssueDetailResponse>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let issue = issues::Entity::find_by_id(pk)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut result = enrich_issues(&state.db, vec![issue]).await?;
    Ok(Json(result.pop().ok_or(AppError::NotFound)?))
}

// ── PATCH /workspaces/{slug}/projects/{project_id}/issues/{pk}/ ───────────────

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 204, description = "Issue actualizado (sin body, paridad Django)"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
/// Actualiza un issue parcialmente.
///
/// # Paridad con Django
/// Django responde **204 No Content** (ver `base.py:700`), no el issue
/// actualizado. El frontend resuelve el nuevo estado por optimistic update
/// a partir del body de la request (`base-issues.store.ts` → `updateIssue`).
/// Devolver un body aquí sería divergencia de contrato.
///
/// # Side-effect del `let _ = ...`
/// La transacción se ejecuta y persiste igual; el valor devuelto se
/// descarta porque ya no se serializa.
pub async fn update_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let issue = issues::Entity::find_by_id(pk)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let project_id = guard.project.id;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let assignees = body.assignees.clone();
    let label_ids = body.labels.clone();

    // El valor se descarta — PATCH devuelve 204 sin body (Django parity).
    // Mantenemos el binding para propagar errores de tx; el `_` evita warning.
    let _ = state
        .db
        .transaction::<_, issues::Model, AppError>(|txn| {
            let mut am: issues::ActiveModel = issue.into();
            if let Some(name) = body.name.clone() {
                am.name = Set(name);
            }
            if let Some(html) = body.description_html.clone() {
                am.description_html = Set(html);
            }
            if let Some(priority) = body.priority.clone() {
                am.priority = Set(priority);
            }
            // Usar Option<Option<>> para diferenciar "no enviado" de "null explícito"
            // En PATCH, si el campo viene en el body se aplica; si no, se preserva.
            if body.state_id.is_some() {
                am.state_id = Set(body.state_id);
            }
            if body.parent_id.is_some() {
                am.parent_id = Set(body.parent_id);
            }
            if body.start_date.is_some() {
                am.start_date = Set(body.start_date);
            }
            if body.target_date.is_some() {
                am.target_date = Set(body.target_date);
            }
            if let Some(draft) = body.is_draft {
                am.is_draft = Set(draft);
            }
            if body.estimate_point_id.is_some() {
                am.estimate_point_id = Set(body.estimate_point_id);
            }
            if body.type_id.is_some() {
                am.type_id = Set(body.type_id);
            }
            am.updated_by_id = Set(Some(user_id));

            let assignees = assignees.clone();
            let label_ids = label_ids.clone();
            Box::pin(async move {
                let updated = am.update(txn).await.map_err(AppError::Database)?;

                if let Some(ref ids) = assignees {
                    sync_assignees(txn, updated.id, project_id, workspace_id, user_id, ids).await?;
                }
                if let Some(ref ids) = label_ids {
                    sync_labels(txn, updated.id, project_id, workspace_id, user_id, ids).await?;
                }
                Ok(updated)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Transaction(app_err) => app_err,
            sea_orm::TransactionError::Connection(db_err) => AppError::Database(db_err),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ── DELETE /workspaces/{slug}/projects/{project_id}/issues/{pk}/ ──────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{pk}/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pk" = Uuid, Path, description = "Issue ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let issue = issues::Entity::find_by_id(pk)
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issues::ActiveModel = issue.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
