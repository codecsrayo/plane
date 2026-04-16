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
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    auth::extractors::WorkspaceMemberGuard,
    auth::permissions::ROLE_GUEST,
    entities::{
        cycle_issues, file_assets, issue_assignees, issue_labels, issue_links,
        issues, module_issues, project_members, projects, states,
    },
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Constantes ────────────────────────────────────────────────────────────────

const PAGINATOR_MAX_LIMIT: u64 = 1000;
const DEFAULT_PER_PAGE: u64 = 100;
const ENTITY_TYPE_ISSUE_ATTACHMENT: &str = "issue_attachment";

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WorkspaceIssuesQuery {
    pub cursor:    Option<String>,
    pub per_page:  Option<u64>,
    pub order_by:  Option<String>,
    pub sub_issue: Option<String>,
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

// ── Cursor helper ─────────────────────────────────────────────────────────────

/// Parsea el cursor de Django: `{page_size}:{current_page}:{is_prev}`.
/// Retorna `(page_size, current_page)`.
fn parse_cursor(cursor: Option<&str>, fallback_per_page: u64) -> (u64, u64) {
    if let Some(c) = cursor {
        let parts: Vec<&str> = c.splitn(3, ':').collect();
        if parts.len() == 3 {
            let page_size = parts[0].parse::<u64>().unwrap_or(fallback_per_page);
            let current_page = parts[1].parse::<u64>().unwrap_or(0);
            return (page_size.clamp(1, PAGINATOR_MAX_LIMIT), current_page);
        }
    }
    (fallback_per_page.clamp(1, PAGINATOR_MAX_LIMIT), 0)
}

// ── Enriquecimiento batch ─────────────────────────────────────────────────────

struct EnrichmentMaps {
    assignees:    HashMap<Uuid, Vec<Uuid>>,
    labels:       HashMap<Uuid, Vec<Uuid>>,
    modules:      HashMap<Uuid, Vec<Uuid>>,
    cycles:       HashMap<Uuid, Uuid>,
    sub_counts:   HashMap<Uuid, i64>,
    attachments:  HashMap<Uuid, i64>,
    links:        HashMap<Uuid, i64>,
    state_groups: HashMap<Uuid, String>,
}

/// Carga todos los datos relacionados en batch para evitar N+1.
async fn load_enrichment(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
    state_ids: &[Uuid],
) -> Result<EnrichmentMaps, AppError> {
    if issue_ids.is_empty() {
        return Ok(EnrichmentMaps {
            assignees:    HashMap::new(),
            labels:       HashMap::new(),
            modules:      HashMap::new(),
            cycles:       HashMap::new(),
            sub_counts:   HashMap::new(),
            attachments:  HashMap::new(),
            links:        HashMap::new(),
            state_groups: HashMap::new(),
        });
    }

    // Assignees
    let raw_assignees = issue_assignees::Entity::find()
        .active()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut assignees: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for a in raw_assignees {
        assignees.entry(a.issue_id).or_default().push(a.assignee_id);
    }

    // Labels
    let raw_labels = issue_labels::Entity::find()
        .active()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut labels: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for l in raw_labels {
        labels.entry(l.issue_id).or_default().push(l.label_id);
    }

    // Modules
    let raw_modules = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut modules: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for m in raw_modules {
        modules.entry(m.issue_id).or_default().push(m.module_id);
    }

    // Cycle IDs — solo el primer ciclo activo por issue (igual que el Subquery de Django)
    let raw_cycles = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut cycles: HashMap<Uuid, Uuid> = HashMap::new();
    for ci in raw_cycles {
        // El entry solo inserta si no existe, preservando el primero (equivalente a [:1])
        cycles.entry(ci.issue_id).or_insert(ci.cycle_id);
    }

    // Sub-issues count por issue padre
    // Usamos una query raw agrupada para evitar N+1
    let raw_sub: Vec<(Uuid, i64)> = issues::Entity::find()
        .select_only()
        .column(issues::Column::ParentId)
        .column_as(
            sea_orm::sea_query::Expr::col(issues::Column::Id).count(),
            "cnt",
        )
        .filter(issues::Column::ParentId.is_in(issue_ids.to_vec()))
        .filter(issues::Column::DeletedAt.is_null())
        .group_by(issues::Column::ParentId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut sub_counts: HashMap<Uuid, i64> = HashMap::new();
    for (parent_id, cnt) in raw_sub {
        sub_counts.insert(parent_id, cnt);
    }

    // Attachment counts agrupados
    // Nota: file_assets.issue_id es Option<Uuid> en el modelo, por eso el tuple es (Option<Uuid>, i64)
    let raw_attachments: Vec<(Option<Uuid>, i64)> = file_assets::Entity::find()
        .select_only()
        .column(file_assets::Column::IssueId)
        .column_as(
            sea_orm::sea_query::Expr::col(file_assets::Column::Id).count(),
            "cnt",
        )
        .filter(file_assets::Column::IssueId.is_in(
            issue_ids.iter().cloned().map(Some).collect::<Vec<_>>(),
        ))
        .filter(
            file_assets::Column::EntityType
                .eq(ENTITY_TYPE_ISSUE_ATTACHMENT),
        )
        .filter(file_assets::Column::DeletedAt.is_null())
        .group_by(file_assets::Column::IssueId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut attachments: HashMap<Uuid, i64> = HashMap::new();
    for (issue_id, cnt) in raw_attachments {
        if let Some(id) = issue_id {
            attachments.insert(id, cnt);
        }
    }

    // Link counts agrupados
    let raw_links: Vec<(Uuid, i64)> = issue_links::Entity::find()
        .select_only()
        .column(issue_links::Column::IssueId)
        .column_as(
            sea_orm::sea_query::Expr::col(issue_links::Column::Id).count(),
            "cnt",
        )
        .filter(issue_links::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_links::Column::DeletedAt.is_null())
        .group_by(issue_links::Column::IssueId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut links: HashMap<Uuid, i64> = HashMap::new();
    for (issue_id, cnt) in raw_links {
        links.insert(issue_id, cnt);
    }

    // State groups (solo para states referenciados por estos issues)
    let state_rows = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .column(states::Column::Group)
        .filter(states::Column::Id.is_in(state_ids.to_vec()))
        .into_tuple::<(Uuid, String)>()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut state_groups: HashMap<Uuid, String> = HashMap::new();
    for (id, group) in state_rows {
        state_groups.insert(id, group);
    }

    Ok(EnrichmentMaps {
        assignees,
        labels,
        modules,
        cycles,
        sub_counts,
        attachments,
        links,
        state_groups,
    })
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
        // El usuario no pertenece a ningún proyecto en el workspace
        return Ok(Json(serde_json::json!({
            "prev_cursor":       format!("{page_size}:-1:0"),
            "cursor":            format!("{page_size}:0:0"),
            "next_cursor":       null,
            "prev_page_results": false,
            "next_page_results": false,
            "page_count":        0,
            "total_results":     0,
            "total_pages":       0,
            "results":           [],
        })));
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

    // Si no hay ninguna condición válida, el usuario no ve nada
    if full_ids.is_empty() && rest_ids.is_empty() {
        return Ok(Json(serde_json::json!({
            "prev_cursor":       format!("{page_size}:-1:0"),
            "cursor":            format!("{page_size}:0:0"),
            "next_cursor":       null,
            "prev_page_results": false,
            "next_page_results": false,
            "page_count":        0,
            "total_results":     0,
            "total_pages":       0,
            "results":           [],
        })));
    }

    // Query base: issues activos (no soft-deleted), no archivados
    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::IsDraft.eq(false))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(permission_condition.clone());

    if exclude_sub_issues {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // ── 5. Total count (para paginación) ──────────────────────────────────────
    let total_results = base_query.clone().count(db).await.map_err(AppError::Database)?;

    let total_pages = if total_results == 0 {
        0u64
    } else {
        total_results.div_ceil(page_size)
    };

    // ── 6. Ordenamiento ───────────────────────────────────────────────────────
    let order_by_param = params
        .order_by
        .as_deref()
        .unwrap_or("-created_at");

    let ordered_query = apply_order(base_query, order_by_param);

    // ── 7. Paginación offset ──────────────────────────────────────────────────
    let start_index = current_page * page_size;
    let end_index = (start_index + page_size).min(total_results);

    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let page_count = issue_models.len() as u64;

    // ── 8. Enriquecimiento batch ──────────────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids: Vec<Uuid> = issue_models
        .iter()
        .filter_map(|i| i.state_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

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

    // ── 10. Respuesta paginada (formato global_paginator.py de Django) ─────────
    let has_next = end_index < total_results;
    let has_prev = current_page > 0;

    let prev_cursor = if current_page == 0 {
        format!("{page_size}:-1:0")
    } else {
        format!("{page_size}:{}:0", current_page - 1)
    };
    let cursor_str  = format!("{page_size}:{current_page}:0");
    let next_cursor = if has_next {
        Some(format!("{page_size}:{}:0", current_page + 1))
    } else {
        None
    };

    Ok(Json(serde_json::json!({
        "prev_cursor":       prev_cursor,
        "cursor":            cursor_str,
        "next_cursor":       next_cursor,
        "prev_page_results": has_prev,
        "next_page_results": has_next,
        "page_count":        page_count,
        "total_results":     total_results,
        "total_pages":       total_pages,
        "results":           results,
    })))
}

// ── Ordenamiento ──────────────────────────────────────────────────────────────

/// Aplica el `order_by` de Django al SelectModel de SeaORM.
/// Prefijo `-` indica descendente. Se soportan los campos más usados
/// en la vista global (spreadsheet / list).
fn apply_order(
    query: sea_orm::Select<issues::Entity>,
    order_by: &str,
) -> sea_orm::Select<issues::Entity> {
    use sea_orm::Order::{Asc, Desc};

    let (col, dir): (issues::Column, _) = match order_by {
        "-created_at"    => (issues::Column::CreatedAt,   Desc),
        "created_at"     => (issues::Column::CreatedAt,   Asc),
        "-updated_at"    => (issues::Column::UpdatedAt,   Desc),
        "updated_at"     => (issues::Column::UpdatedAt,   Asc),
        "-priority"      => (issues::Column::Priority,    Desc),
        "priority"       => (issues::Column::Priority,    Asc),
        "-sort_order"    => (issues::Column::SortOrder,   Desc),
        "sort_order"     => (issues::Column::SortOrder,   Asc),
        "-sequence_id"   => (issues::Column::SequenceId,  Desc),
        "sequence_id"    => (issues::Column::SequenceId,  Asc),
        "-target_date"   => (issues::Column::TargetDate,  Desc),
        "target_date"    => (issues::Column::TargetDate,  Asc),
        "-start_date"    => (issues::Column::StartDate,   Desc),
        "start_date"     => (issues::Column::StartDate,   Asc),
        "-completed_at"  => (issues::Column::CompletedAt, Desc),
        "completed_at"   => (issues::Column::CompletedAt, Asc),
        // Default seguro
        _                => (issues::Column::CreatedAt,   Desc),
    };

    query.order_by(col, dir)
}
