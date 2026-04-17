// src/routes/workspace_view_issues.rs
//! Endpoint de issues a nivel de workspace (vista global / spreadsheet).
//!
//! Equivalente a `WorkspaceViewIssuesViewSet` en
//! `plane/app/views/view/base.py` → `plane/app/urls/views.py:52`.
//!
//! Ruta implementada:
//!   GET /api/workspaces/{slug}/issues/
//!
//! Parámetros de query:
//!   - cursor      : paginación Django (`{page_size}:{page}:{is_prev}`), default `100:0:0`
//!   - per_page    : ignorado si viene en cursor; default 100
//!   - order_by    : campo de ordenamiento, default `-created_at`
//!   - sub_issue   : `false` = excluir sub-issues (parent_id IS NOT NULL), default muestra todo
//!
//! Lógica de permisos (mirror Django `_get_project_permission_filters`):
//!   Para guest (role = 5):
//!     - si project.guest_view_all_features = true  → ve todos los issues del proyecto
//!     - si project.guest_view_all_features = false → ve solo sus propios issues (created_by)
//!   Para member / admin (role > 5):
//!     → ve todos los issues de los proyectos donde es miembro activo

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    auth::extractors::WorkspaceMemberGuard,
    auth::permissions::ROLE_GUEST,
    entities::{issues, project_members, projects},
    error::AppError,
    routes::issue_pagination::{
        apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
        load_workspace_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
    },
    routes::issue_filters::{apply_issue_filters, reject_if_rich_filters, FilteredQuery},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WorkspaceIssuesQuery {
    // ── Paginación / orden ────────────────────────────────────────────────────
    pub cursor:        Option<String>,
    pub per_page:      Option<u64>,
    pub order_by:      Option<String>,

    // ── Toggles simples ───────────────────────────────────────────────────────
    pub sub_issue:     Option<String>,
    /// Filtro incremental — solo issues actualizados después de este timestamp.
    /// Mirror del `updated_at__gt` en base.py:256.
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,

    // ── Filtros delegados al módulo `issue_filters` (inlineados por
    //    limitación de serde_urlencoded con `flatten`). La lógica de
    //    parsing/aplicación vive en un solo lugar.
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
    // Capturado solo para detectar presencia y rechazar con 400.
    // Ver `issue_filters::reject_if_rich_filters`.
    pub filters:           Option<String>,
}

impl WorkspaceIssuesQuery {
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

// ── DTOs de respuesta ─────────────────────────────────────────────────────────

/// Refleja los campos de `ViewIssueListSerializer` en Django.
/// Los nombres de campo son snake_case → serde los serializa tal cual.
#[derive(Debug, Serialize)]
pub struct WorkspaceIssueItem {
    pub id:              Uuid,
    pub name:            String,
    pub state_id:        Option<Uuid>,
    pub sort_order:      f64,
    pub completed_at:    Option<chrono::DateTime<chrono::FixedOffset>>,
    pub estimate_point:  Option<Uuid>,
    pub priority:        String,
    pub start_date:      Option<chrono::NaiveDate>,
    pub target_date:     Option<chrono::NaiveDate>,
    pub sequence_id:     i32,
    pub project_id:      Uuid,
    pub parent_id:       Option<Uuid>,
    pub cycle_id:        Option<Uuid>,
    pub sub_issues_count: i64,
    pub created_at:      chrono::DateTime<chrono::FixedOffset>,
    pub updated_at:      chrono::DateTime<chrono::FixedOffset>,
    pub created_by:      Option<Uuid>,
    pub updated_by:      Option<Uuid>,
    pub attachment_count: i64,
    pub link_count:      i64,
    pub is_draft:        bool,
    pub archived_at:     Option<chrono::NaiveDate>,
    #[serde(rename = "state__group")]
    pub state_group:     Option<String>,
    pub assignee_ids:    Vec<Uuid>,
    pub label_ids:       Vec<Uuid>,
    pub module_ids:      Vec<Uuid>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// GET /api/workspaces/{slug}/issues/
///
/// Lista issues de todos los proyectos del workspace a los que el usuario
/// tiene acceso. Equivale a `WorkspaceViewIssuesViewSet.list()` en Django.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/issues/",
    tag = "Issues",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("cursor" = Option<String>, Query, description = "Cursor de paginación Django: {per_page}:{page}:{is_prev}"),
        ("per_page" = Option<u64>, Query, description = "Resultados por página (ignorado si está en cursor)"),
        ("order_by" = Option<String>, Query, description = "Campo de ordenamiento, ej: -created_at"),
        ("sub_issue" = Option<String>, Query, description = "false = excluir sub-issues"),
    ),
    responses(
        (status = 200, description = "Lista paginada de issues del workspace"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin acceso al workspace"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_workspace_view_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(params): Query<WorkspaceIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Rechaza `?filters=<JSON>` antes de ejecutar nada — ver
    // `issue_filters::reject_if_rich_filters` para el contexto completo del
    // gap vs `ComplexFilterBackend` de Django.
    reject_if_rich_filters(params.filters.as_deref())?;

    let db = &state.db;
    let workspace_id = guard.workspace.id;
    let user_id = guard.user.id;
    let workspace_member_role = guard.member.role;

    // ── 1. Paginación ─────────────────────────────────────────────────────────
    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) =
        parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // ── 2. Filtros base de la query ───────────────────────────────────────────
    let exclude_sub_issues = params
        .sub_issue
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    // ── 3. Calcular proyectos a los que el usuario tiene acceso ───────────────
    //
    // Mirror de `_get_project_permission_filters` en Django:
    //   - Traemos los project_members activos del user en este workspace
    //   - Para guests (role = 5): si guest_view_all_features = false,
    //     el proyecto va a "restricted" (solo ven sus propios issues).
    //   - Para roles > 5: acceso completo al proyecto.
    //
    // Optimización: si el usuario es admin del workspace (role >= 20),
    // asumimos acceso completo a todos los proyectos activos del workspace.

    // Obtener membresías de proyecto del usuario en este workspace
    let memberships = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(workspace_id))
        .filter(project_members::Column::MemberId.eq(user_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if memberships.is_empty() {
        // El usuario no pertenece a ningún proyecto en el workspace.
        // Shape mirror de Django `OffsetPaginator.paginate()`
        // (plane/utils/paginator.py:715-730).
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let project_ids: Vec<Uuid> = memberships.iter().map(|m| m.project_id).collect();

    // Cargar datos de proyectos para verificar guest_view_all_features y archived_at
    let project_rows = projects::Entity::find()
        .select_only()
        .column(projects::Column::Id)
        .column(projects::Column::GuestViewAllFeatures)
        .column(projects::Column::ArchivedAt)
        .filter(projects::Column::Id.is_in(project_ids.clone()))
        .filter(projects::Column::DeletedAt.is_null())
        .into_tuple::<(Uuid, bool, Option<chrono::DateTime<chrono::FixedOffset>>)>()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Índice rápido project_id → (guest_view_all_features, is_archived)
    let project_meta: HashMap<Uuid, (bool, bool)> = project_rows
        .into_iter()
        .map(|(id, gvaf, archived)| (id, (gvaf, archived.is_some())))
        .collect();

    // Clasificar proyectos en: full_access vs own_issues_only
    // (excluir proyectos archivados)
    let mut full_access_ids: HashSet<Uuid> = HashSet::new();
    let mut restricted_ids: HashSet<Uuid> = HashSet::new();

    for membership in &memberships {
        let pid = membership.project_id;
        let role = membership.role;

        let (guest_view_all_features, is_archived) = match project_meta.get(&pid) {
            Some(meta) => *meta,
            None => continue, // proyecto no encontrado o eliminado
        };

        if is_archived {
            continue; // excluir proyectos archivados
        }

        // Workspace admin puede ver todo independientemente del rol de proyecto
        if workspace_member_role >= 20 || role > ROLE_GUEST {
            full_access_ids.insert(pid);
        } else {
            // Guest (role = 5)
            if guest_view_all_features {
                full_access_ids.insert(pid);
            } else {
                restricted_ids.insert(pid);
            }
        }
    }

    // ── 4. Construir query base ───────────────────────────────────────────────
    //
    // Antipatrón evitado: no usamos OR sin índice — separamos en dos condiciones
    // distintas y las unimos con `sea_orm::Condition::any()`.

    let full_ids: Vec<Uuid> = full_access_ids.into_iter().collect();
    let rest_ids: Vec<Uuid> = restricted_ids.into_iter().collect();

    // Condición de permisos: full_access OR (restricted AND created_by = user)
    let permission_condition = {
        use sea_orm::Condition;
        let mut cond = Condition::any();

        if !full_ids.is_empty() {
            cond = cond.add(issues::Column::ProjectId.is_in(full_ids.clone()));
        }

        if !rest_ids.is_empty() {
            cond = cond.add(
                Condition::all()
                    .add(issues::Column::ProjectId.is_in(rest_ids.clone()))
                    .add(issues::Column::CreatedById.eq(user_id)),
            );
        }

        cond
    };

    // Si no hay ninguna condición válida, el usuario no ve nada.
    if full_ids.is_empty() && rest_ids.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    // Query base: issues activos (no soft-deleted), no archivados.
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::IsDraft.eq(false))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(permission_condition.clone());

    if exclude_sub_issues {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // Mirror del `IssueManager.exclude(state__group='triage')` en
    // db/models/issue.py:97. Este endpoint antes no aplicaba esta exclusión
    // — ahora queda en paridad con Django. Usamos pre-query de state IDs
    // en lugar de JOIN para mantener el query builder simple.
    let triage_state_ids = load_workspace_triage_state_ids(db, workspace_id).await?;
    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    // Filtro incremental `updated_at__gt` (base.py:256). Útil para sincronización
    // delta del frontend sin re-descargar toda la lista.
    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // ── 4.5. Filtros del módulo compartido ────────────────────────────────────
    //
    // state, state_group, priority, created_by, parent, name, start_date,
    // target_date, labels, assignees, module, cycle, type, start_target_date.
    // Ver `routes::issue_filters` para el mapeo detallado.
    let filter_params = params.to_filter_params();
    let filtered = apply_issue_filters(db, base_query, &filter_params, workspace_id).await?;
    let base_query = match filtered {
        FilteredQuery::Active(q) => q,
        FilteredQuery::Empty => return Ok(Json(empty_paginated_response(page_size))),
    };

    // ── 5. Total count (para paginación) ──────────────────────────────────────
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

    // ── 8. Enriquecimiento batch ──────────────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);

    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    // ── 9. Serializar resultados ──────────────────────────────────────────────
    let results: Vec<WorkspaceIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());

            WorkspaceIssueItem {
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
    // Shape mirror exacto de `OffsetPaginator.paginate()` en
    // `plane/utils/paginator.py:715-730`. El frontend lee `total_count`
    // en `base-issues.store.ts:1290`, y `TIssuesResponse`
    // (packages/types/src/issues/issue.ts:126) declara
    // `grouped_by`, `count`, `extra_stats` como requeridos.
    Ok(Json(paginated_response(
        results,
        page_size,
        current_page,
        total_results,
    )))
}
