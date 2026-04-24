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
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{intake_issues, issue_assignees, issue_labels, issue_subscribers, issues, labels},
    error::AppError,
    routes::issue_pagination::{
        apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
        load_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
    },
    routes::issue_filters::{apply_issue_filters, merge_json_filters, FilteredQuery},
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

/// Shape de `POST /workspaces/{slug}/projects/{project_id}/issues/`.
///
/// Paridad exacta con `IssueCreateSerializer` (apps/api/plane/app/serializers/
/// issue.py:82) y con el payload que construye el frontend desde
/// `DEFAULT_WORK_ITEM_FORM_VALUES` (packages/constants/src/issue/modal.ts).
///
/// # Convenciones críticas
/// - `label_ids` / `assignee_ids` (plural + sufijo) son los nombres que usan
///   DRF y el frontend; renombrarlos rompía la deserialización silenciosamente
///   (los campos quedaban en `None` y la issue se creaba sin assignees/labels).
/// - `estimate_point` sin `_id` — DRF expone la FK con ese nombre cuando se
///   declara como `PrimaryKeyRelatedField(source="estimate_point", ...)`.
/// - Todos los `Option<Uuid>` / `Option<NaiveDate>` usan deserializadores que
///   convierten `""` a `None`, porque el frontend manda `state_id: ""` y
///   fechas vacías por default — serde nativo falla con 422.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateIssueRequest {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub state_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub target_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub estimate_point: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub type_id: Option<Uuid>,
    // `deserialize_uuid_list_filter_nulls` — el frontend (react-hook-form)
    // a veces inicializa estos arrays con `[null]` cuando el select de
    // asignee/label arranca en "unassigned". Serde nativo fallaría con 422
    // al topar `null` dentro de `Vec<Uuid>`. Django lo tolera porque
    // Postgres descarta NULLs del `IN (...)` al final.
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateIssueRequest {
    pub name: Option<String>,
    pub description_html: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub state_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub parent_id: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_date")]
    pub target_date: Option<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub estimate_point: Option<Uuid>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_empty_as_none_uuid")]
    pub type_id: Option<Uuid>,
    // `deserialize_uuid_list_filter_nulls` — mismo motivo que en
    // `CreateIssueRequest`: el frontend envía `[null]` desde react-hook-form
    // al deseleccionar asignees o labels (regresión observada con payload
    // real `PATCH /issues/{id}/` devolviendo 422 con el mensaje
    // `assignee_ids[0]: invalid type: null, expected a formatted UUID string`).
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub assignee_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "crate::utils::serde_empty::deserialize_uuid_list_filter_nulls")]
    pub label_ids: Option<Vec<Uuid>>,
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
    /// CSV de `user_ids` — mirror de `filter_subscribed_issues`
    /// (issue_filters.py:392-403). Shared con el resto de listados de issues.
    pub subscriber:        Option<String>,
    #[serde(rename = "type")]
    pub type_filter:       Option<String>,
    pub start_target_date: Option<String>,

    // ── Rich filters (subconjunto Django-parity) ─────────────────────────────
    //
    // Blob JSON que el frontend envía en spreadsheet layout y vistas guardadas.
    // Se parsea con `issue_filters::merge_json_filters` y se fusiona en los
    // flat params; keys desconocidas → 400.
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
            subscriber:        self.subscriber.clone(),
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

/// Construye el shape `IssueDetailResponse` para un issue individual,
/// reutilizando el helper compartido `load_enrichment` + dos queries
/// específicas de detalle.
///
/// Espejo de las anotaciones del `retrieve()` en
/// `apps/api/plane/app/views/issue/base.py:480-575`.
///
/// # Campos poblados (vs commit 1, que dejaba stubs)
/// - `cycle_id`, `module_ids`, `sub_issues_count`, `attachment_count`,
///   `link_count` — vía `load_enrichment`.
/// - `is_subscribed` — query puntual a `issue_subscribers`.
/// - `is_intake`    — query puntual a `intake_issues` con status ∈ (-2, 0).
///
/// # Estrategia
/// `load_enrichment` está diseñado para batching N issues; aquí lo usamos
/// con N=1. El overhead es una query por relación en lugar de una query
/// por issue × relación — sigue siendo O(1) roundtrips. Mantener un solo
/// helper de enrichment (en vez de una versión "single" y otra "batch")
/// evita drift entre ambos código paths.
///
/// # `state_ids = &[]`
/// `IssueDetailResponse` no expone `state__group` (solo el listing lo usa
/// vía `ProjectIssueItem`). Pasamos slice vacío → `load_enrichment` hace
/// early-return de la query de states.
async fn build_detail_response(
    db: &sea_orm::DatabaseConnection,
    issue_model: issues::Model,
    user_id: Uuid,
) -> Result<IssueDetailResponse, AppError> {
    let id = issue_model.id;
    let issue_ids = [id];
    let mut maps = load_enrichment(db, &issue_ids, &[]).await?;

    // is_subscribed — mirror del `Exists(IssueSubscriber.objects.filter(...))`
    // en base.py:566-574. Usamos `count > 0` como equivalente a EXISTS; el
    // índice compuesto (issue_id, subscriber_id) hace que la query sea O(log n).
    let is_subscribed = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(id))
        .filter(issue_subscribers::Column::SubscriberId.eq(user_id))
        .count(db)
        .await
        .map_err(AppError::Database)?
        > 0;

    // is_intake — mirror del pattern en base.py:1305-1313.
    //
    // # Sobre la divergencia con el `retrieve()` principal
    // El `retrieve()` de Django (base.py:480-575) NO anota este campo —
    // parece un oversight, porque `IssueDetailSerializer` lo declara como
    // `BooleanField(read_only=True)`. El endpoint paralelo que consulta
    // issues por `sequence_id` (`base.py:1225-1314`) sí lo anota.
    //
    // El Rust port lo pobla correctamente en ambos casos. Los statuses
    // `-2` (PENDING) y `0` (SNOOZED) son los que cuentan como "en intake"
    // según la lógica del inbox.
    let is_intake = intake_issues::Entity::find()
        .active()
        .filter(intake_issues::Column::IssueId.eq(id))
        .filter(intake_issues::Column::Status.is_in(vec![-2_i32, 0]))
        .count(db)
        .await
        .map_err(AppError::Database)?
        > 0;

    Ok(IssueDetailResponse {
        id,
        name: issue_model.name,
        state_id: issue_model.state_id,
        sort_order: issue_model.sort_order,
        completed_at: issue_model.completed_at,
        estimate_point_id: issue_model.estimate_point_id,
        priority: issue_model.priority,
        start_date: issue_model.start_date,
        target_date: issue_model.target_date,
        sequence_id: issue_model.sequence_id,
        project_id: issue_model.project_id,
        parent_id: issue_model.parent_id,
        cycle_id: maps.cycles.remove(&id),
        module_ids: maps.modules.remove(&id).unwrap_or_default(),
        label_ids: maps.labels.remove(&id).unwrap_or_default(),
        assignee_ids: maps.assignees.remove(&id).unwrap_or_default(),
        sub_issues_count: maps.sub_counts.get(&id).copied().unwrap_or(0),
        attachment_count: maps.attachments.get(&id).copied().unwrap_or(0),
        link_count: maps.links.get(&id).copied().unwrap_or(0),
        created_at: issue_model.created_at,
        updated_at: issue_model.updated_at,
        created_by_id: issue_model.created_by_id,
        updated_by_id: issue_model.updated_by_id,
        is_draft: issue_model.is_draft,
        archived_at: issue_model.archived_at,
        description_html: issue_model.description_html,
        is_subscribed,
        is_intake,
    })
}

/// Construye el shape `IssueCreateResponse` — espejo de la proyección
/// `.values(...)` en `base.py:427-454`.
///
/// Reutiliza `load_enrichment` para cycle/modules/labels/assignees/counts.
/// No consulta `is_subscribed` ni `is_intake` (Django no los proyecta en
/// create). Incluye `deleted_at` directamente del modelo (siempre `None`
/// inmediatamente tras create, pero lo emitimos para paridad estricta).
async fn build_create_response(
    db: &sea_orm::DatabaseConnection,
    issue_model: issues::Model,
) -> Result<IssueCreateResponse, AppError> {
    let id = issue_model.id;
    let issue_ids = [id];
    let mut maps = load_enrichment(db, &issue_ids, &[]).await?;

    Ok(IssueCreateResponse {
        id,
        name: issue_model.name,
        state_id: issue_model.state_id,
        sort_order: issue_model.sort_order,
        completed_at: issue_model.completed_at,
        estimate_point_id: issue_model.estimate_point_id,
        priority: issue_model.priority,
        start_date: issue_model.start_date,
        target_date: issue_model.target_date,
        sequence_id: issue_model.sequence_id,
        project_id: issue_model.project_id,
        parent_id: issue_model.parent_id,
        cycle_id: maps.cycles.remove(&id),
        module_ids: maps.modules.remove(&id).unwrap_or_default(),
        label_ids: maps.labels.remove(&id).unwrap_or_default(),
        assignee_ids: maps.assignees.remove(&id).unwrap_or_default(),
        sub_issues_count: maps.sub_counts.get(&id).copied().unwrap_or(0),
        attachment_count: maps.attachments.get(&id).copied().unwrap_or(0),
        link_count: maps.links.get(&id).copied().unwrap_or(0),
        created_at: issue_model.created_at,
        updated_at: issue_model.updated_at,
        created_by_id: issue_model.created_by_id,
        updated_by_id: issue_model.updated_by_id,
        is_draft: issue_model.is_draft,
        archived_at: issue_model.archived_at,
        deleted_at: issue_model.deleted_at,
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
        // Django usa auto_now=True en updated_at (TimeAuditModel).
        am.updated_at = Set(now);
        am.update(txn).await.map_err(AppError::Database)?;
    }

    // Insertar nuevos. created_at / updated_at explícitos porque
    // issue_assignees::ActiveModelBehavior está vacío y las columnas son
    // NOT NULL (mismo patrón que labels.rs / issues).
    for assignee_id in new_ids {
        issue_assignees::ActiveModel {
            id: Set(Uuid::new_v4()),
            issue_id: Set(issue_id),
            assignee_id: Set(*assignee_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(actor_id)),
            updated_by_id: Set(Some(actor_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
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
        // Django usa auto_now=True en updated_at (TimeAuditModel).
        am.updated_at = Set(now);
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
            // created_at / updated_at explícitos (misma razón que
            // sync_assignees / labels.rs: ActiveModelBehavior vacío,
            // columnas NOT NULL).
            issue_labels::ActiveModel {
                id: Set(Uuid::new_v4()),
                issue_id: Set(issue_id),
                label_id: Set(*label_id),
                project_id: Set(project_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(actor_id)),
                updated_by_id: Set(Some(actor_id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
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

    // Parsea `?filters=<JSON>` y fusiona los campos reconocidos en los
    // filter_params (state_group__in, priority__in, label_id__in, etc.).
    // Keys desconocidas retornan 400 para prevenir data leaks silenciosos.
    // Ver `issue_filters::merge_json_filters` para el mapeo completo.
    let mut filter_params = params.to_filter_params();
    merge_json_filters(params.filters.as_deref(), &mut filter_params)?;

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
    // `filter_params` ya incluye la fusión del blob `?filters=<JSON>` hecha
    // al inicio del handler — usamos esa versión (NO llamar `to_filter_params`
    // de nuevo, sombrearía la fusión y descartaría el JSON).
    //
    // `FilteredQuery::Empty` → algún filtro implica 0 matches garantizados
    // (p. ej. `labels=<uuid>` sin ningún issue con ese label). Hacemos
    // early-return con el shape paginado vacío.
    let workspace_id = guard.workspace.id;
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
    let assignees = body.assignee_ids.clone().unwrap_or_default();
    let label_ids = body.label_ids.clone().unwrap_or_default();

    let raw_html = body.description_html.clone().unwrap_or_default();
    let description_html = crate::utils::content_validator::sanitize_description(&raw_html)
        .map_err(AppError::BadRequest)?;

    let issue = state
        .db
        .transaction_with_config::<_, issues::Model, AppError>(
            |txn| {
                let name = body.name.clone();
                let description_html = description_html.clone();
                let priority = body.priority.clone().unwrap_or_else(|| "none".to_owned());
                let state_id = body.state_id;
                let parent_id = body.parent_id;
                let start_date = body.start_date;
                let target_date = body.target_date;
                // Django expone la FK como `estimate_point` (sin `_id`) en el wire,
                // pero el modelo SeaORM conserva `estimate_point_id` como nombre de
                // columna. Hacemos el bridge aquí.
                let estimate_point_id = body.estimate_point;
                let type_id = body.type_id;
                let assignees = assignees.clone();
                let label_ids = label_ids.clone();
                Box::pin(async move {
                    // sequence_id = MAX(sequence_id) + 1 dentro del proyecto.
                    //
                    // NOTA 1: MAX() sin GROUP BY siempre devuelve una fila
                    // (aunque la tabla esté vacía, con valor NULL). Por eso
                    // decodificamos a Option<i32> y flatten sobre el
                    // Option<Option<i32>> que devuelve .one(); sin el
                    // Option más externo, SeaORM trataría NULL como error de
                    // decode ("A null value was encountered while decoding 0").
                    //
                    // NOTA 2: el target es i32 (NO i64). En Postgres
                    // MAX(INT4) → INT4; no se promueve a BIGINT como en MySQL.
                    // Usar i64 produce
                    // "mismatched types; Rust type Option<i64> (as SQL type
                    // INT8) is not compatible with SQL type INT4".
                    use sea_orm::QuerySelect;
                    let max_seq: Option<i32> = issues::Entity::find()
                        .filter(issues::Column::ProjectId.eq(project_id))
                        .select_only()
                        .column_as(
                            sea_orm::sea_query::Expr::col(issues::Column::SequenceId).max(),
                            "max_seq",
                        )
                        .into_tuple::<Option<i32>>()
                        .one(txn)
                        .await
                        .map_err(AppError::Database)?
                        .flatten();

                    let sequence_id = max_seq.unwrap_or(0) + 1;

                    // created_at / updated_at se setean explícitamente porque
                    // issues::ActiveModelBehavior está vacío (sin hook
                    // before_save) y las columnas son NOT NULL. Dejarlas con
                    // Default::default() hacía que SeaORM enviara NULL y la BD
                    // rechazara con 23502 ("violates not-null constraint").
                    // Mismo patrón que labels.rs / issue_extras.rs / pages.rs.
                    let now: chrono::DateTime<chrono::FixedOffset> =
                        chrono::Utc::now().into();

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
                        created_at: Set(now),
                        updated_at: Set(now),
                        deleted_at: Set(None),
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

    let response = build_detail_response(&state.db, issue, guard.user.id).await?;
    Ok(Json(response))
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
    let assignees = body.assignee_ids.clone();
    let label_ids = body.label_ids.clone();

    let clean_html = if let Some(ref html) = body.description_html {
        Some(crate::utils::content_validator::sanitize_description(html)
            .map_err(AppError::BadRequest)?)
    } else {
        None
    };

    // El valor se descarta — PATCH devuelve 204 sin body (Django parity).
    // Mantenemos el binding para propagar errores de tx; el `_` evita warning.
    let _ = state
        .db
        .transaction::<_, issues::Model, AppError>(|txn| {
            let mut am: issues::ActiveModel = issue.into();
            if let Some(name) = body.name.clone() {
                am.name = Set(name);
            }
            if let Some(html) = clean_html.clone() {
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
            // Wire: `estimate_point` (paridad DRF) → columna: `estimate_point_id`.
            if body.estimate_point.is_some() {
                am.estimate_point_id = Set(body.estimate_point);
            }
            if body.type_id.is_some() {
                am.type_id = Set(body.type_id);
            }
            am.updated_by_id = Set(Some(user_id));
            // Django usa auto_now=True en updated_at (TimeAuditModel). SeaORM lo
            // dejaría Unchanged y el UPDATE no lo tocaría; hay que setearlo.
            am.updated_at = Set(chrono::Utc::now().into());

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

// ── GET /workspaces/{slug}/projects/{project_id}/issues/list/ ─────────────────
//
// Mirror de `IssueListEndpoint.get()` en `apps/api/plane/app/views/issue/base.py:80-133`.
// Recibe `?issues=uuid1,uuid2,...` y retorna la lista de issues con ese ID
// (sin paginación — el cliente ya conoce los IDs que quiere).
//
// Shape de respuesta: Vec<ProjectIssueItem> (igual al shape de list_issues).

#[derive(Debug, Deserialize)]
pub struct IssueListByIdsQuery {
    /// Comma-separated issue UUIDs.
    pub issues: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/list/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("issues" = String, Query, description = "Comma-separated issue UUIDs"),
    ),
    responses(
        (status = 200, description = "Lista de issues por IDs"),
        (status = 400, description = "Parámetro issues requerido"),
        (status = 403, description = "Sin permiso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues_by_ids(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<IssueListByIdsQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let raw_ids = params.issues.as_deref().unwrap_or("").trim().to_string();
    if raw_ids.is_empty() {
        return Err(AppError::BadRequest(
            "issues query parameter is required".to_string(),
        ));
    }

    // Parse comma-separated UUIDs; ignorar strings vacíos/malformados
    // (mirror del list-comprehension de Django: `[id for id in ids.split(",") if id != ""]`).
    let issue_ids: Vec<Uuid> = raw_ids
        .split(',')
        .filter_map(|s| s.trim().parse::<Uuid>().ok())
        .collect();

    if issue_ids.is_empty() {
        return Ok(Json(serde_json::Value::Array(vec![])).into_response());
    }

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    // Guest restriction: si role=5 y guest_view_all_features=false, solo sus issues.
    let is_restricted_guest =
        guard.project_member.role == 5 && !guard.project.guest_view_all_features;

    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::Id.is_in(issue_ids.clone()));

    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    let issue_models = base_query.all(db).await.map_err(AppError::Database)?;

    if issue_models.is_empty() {
        return Ok(Json(serde_json::Value::Array(vec![])).into_response());
    }

    let fetched_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &fetched_ids, &state_ids).await?;

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

    Ok(Json(results).into_response())
}

// ── GET /workspaces/{slug}/projects/{project_id}/issues-detail/ ───────────────
//
// Mirror de `IssueDetailEndpoint.get()` en `apps/api/plane/app/views/issue/base.py:960-1090`.
// Retorna una página paginada (shape Django OffsetPaginator) con el shape
// completo de IssueListDetailSerializer (igual a ProjectIssueItem).
//
// Diferencias clave vs list_issues:
// - Incluye issues archivados y drafts (no aplica los filtros del IssueManager).
// - Filtra por permisos de guest (owner || guest_view_all_features).

#[derive(Debug, Deserialize)]
pub struct IssueDetailQuery {
    pub cursor: Option<String>,
    pub per_page: Option<u64>,
    pub order_by: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues-detail/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor"),
        ("order_by" = Option<String>, Query, description = "Sort field"),
    ),
    responses(
        (status = 200, description = "Lista paginada de issues con detalle"),
        (status = 403, description = "Sin permiso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues_detail(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<IssueDetailQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // Guest restriction
    let is_restricted_guest =
        guard.project_member.role == 5 && !guard.project.guest_view_all_features;

    // A diferencia de list_issues, IssueDetailEndpoint NO aplica los filtros
    // del IssueManager (archived_at, is_draft, triage) — expone todos los issues.
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id));

    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    let total_results = base_query.clone().count(db).await.map_err(AppError::Database)?;

    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    let start_index = current_page * page_size;
    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if issue_models.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

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

    Ok(Json(paginated_response(results, page_size, current_page, total_results)))
}

// ── GET /workspaces/{slug}/projects/{project_id}/v2/issues/ ──────────────────
//
// Mirror de `IssuePaginatedViewSet.list()` en
// `apps/api/plane/app/views/issue/base.py:803-958`.
//
// Diferencias vs list_issues (v1):
// - Orden por `updated_at` ASC (sincronización delta para el cliente).
// - Soporte de `?updated_at__gt=<datetime>` para sincronización incremental.
// - Campo opcional `description_html` cuando `?description=true`.
// - Aplica las mismas exclusiones del IssueManager (archived, draft, triage).
// - La guest restriction también aplica.

#[derive(Debug, Serialize)]
pub struct V2IssueItem {
    pub id:               Uuid,
    pub name:             String,
    pub state_id:         Option<Uuid>,
    #[serde(rename = "state__group")]
    pub state_group:      Option<String>,
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
    pub created_at:       chrono::DateTime<chrono::FixedOffset>,
    pub updated_at:       chrono::DateTime<chrono::FixedOffset>,
    pub created_by:       Option<Uuid>,
    pub updated_by:       Option<Uuid>,
    pub is_draft:         bool,
    pub archived_at:      Option<chrono::NaiveDate>,
    pub module_ids:       Vec<Uuid>,
    pub label_ids:        Vec<Uuid>,
    pub assignee_ids:     Vec<Uuid>,
    pub link_count:       i64,
    pub attachment_count: i64,
    pub sub_issues_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_html: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct V2IssuesQuery {
    pub cursor:        Option<String>,
    pub per_page:      Option<u64>,
    /// Fecha ISO-8601; solo retorna issues actualizados después de esta fecha.
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,
    /// Si `"true"`, incluye `description_html` en la respuesta.
    pub description:   Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/v2/issues/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor"),
        ("updated_at__gt" = Option<String>, Query, description = "Sync delta filter"),
        ("description" = Option<String>, Query, description = "Include description_html"),
    ),
    responses(
        (status = 200, description = "Lista paginada ligera de issues (v2)"),
        (status = 403, description = "Sin permiso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues_v2(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Query(params): Query<V2IssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let db = &state.db;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // Proyecto archivado → vacío (mirror IssueManager)
    if guard.project.archived_at.is_some() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let triage_state_ids = load_triage_state_ids(db, project_id).await?;

    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::IsDraft.eq(false));

    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    // Guest restriction
    let is_restricted_guest =
        guard.project_member.role == 5 && !guard.project.guest_view_all_features;
    if is_restricted_guest {
        base_query = base_query.filter(issues::Column::CreatedById.eq(user_id));
    }

    // Sync delta filter
    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // v2 siempre ordena por updated_at ASC (para sincronización delta secuencial)
    let ordered_query = base_query.clone().order_by(issues::Column::UpdatedAt, Order::Asc);

    let total_results = base_query.count(db).await.map_err(AppError::Database)?;

    let start_index = current_page * page_size;
    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if issue_models.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let include_description = params.description.as_deref() == Some("true");

    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    let results: Vec<V2IssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());
            let description_html = if include_description {
                Some(m.description_html.clone())
            } else {
                None
            };
            V2IssueItem {
                id,
                name: m.name,
                state_id: m.state_id,
                state_group,
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
                created_at: m.created_at,
                updated_at: m.updated_at,
                created_by: m.created_by_id,
                updated_by: m.updated_by_id,
                is_draft: m.is_draft,
                archived_at: m.archived_at,
                module_ids: enrich.modules.remove(&id).unwrap_or_default(),
                label_ids: enrich.labels.remove(&id).unwrap_or_default(),
                assignee_ids: enrich.assignees.remove(&id).unwrap_or_default(),
                link_count: enrich.links.get(&id).copied().unwrap_or(0),
                attachment_count: enrich.attachments.get(&id).copied().unwrap_or(0),
                sub_issues_count: enrich.sub_counts.get(&id).copied().unwrap_or(0),
                description_html,
            }
        })
        .collect();

    Ok(Json(paginated_response(results, page_size, current_page, total_results)))
}
