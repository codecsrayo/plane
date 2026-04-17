// src/routes/issue_pagination.rs
//! Helpers compartidos para paginación de issues estilo Django.
//!
//! Extraído de `workspace_view_issues.rs` para reutilizarlo en
//! `issues.rs::list_issues` (y futuros endpoints que repliquen el shape
//! de `OffsetPaginator.paginate()` en `plane/utils/paginator.py:715-730`).
//!
//! Incluye:
//!   - `parse_cursor`               → parsea `{page_size}:{page}:{is_prev}`.
//!   - `EnrichmentMaps` +
//!     `load_enrichment`            → carga batch de relaciones (N+1 evitado)
//!                                    para los 8 campos enriquecidos que
//!                                    replica el `issue_on_results` de Django
//!                                    (grouper.py:93-141).
//!   - `apply_issue_order`          → mapea `order_by` de Django a SeaORM.
//!   - `empty_paginated_response`   → shape exacto del paginator Django para
//!                                    respuestas vacías (evita construirlo a
//!                                    mano en cada early-return).
//!
//! # Antipatrones evitados
//! - **N+1**: todas las relaciones se cargan con `is_in()` en una sola query.
//! - **Duplicación**: este módulo sustituye ~250 líneas duplicadas entre
//!   `workspace_view_issues.rs` y `issues.rs`.
//! - **SQL inyection**: `order_by` se mapea vía `match` contra una whitelist;
//!   valores no reconocidos caen al default seguro (`-created_at`).

use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    entities::{
        cycle_issues, file_assets, issue_assignees, issue_labels, issue_links, issues,
        module_issues, states,
    },
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
};

// ── Constantes ────────────────────────────────────────────────────────────────

/// Cota superior de `page_size` (paralelo al `max_limit` del paginator Django).
pub const PAGINATOR_MAX_LIMIT: u64 = 1000;

/// Tamaño de página por defecto cuando no se envía `cursor` ni `per_page`.
pub const DEFAULT_PER_PAGE: u64 = 100;

/// Identificador del entity_type en `file_assets` que representa attachments
/// de issues (mirror de `FileAsset.EntityTypeContext.ISSUE_ATTACHMENT`).
pub const ENTITY_TYPE_ISSUE_ATTACHMENT: &str = "issue_attachment";

/// Nombre del `state.group` que Django excluye en `IssueManager.get_queryset`
/// (db/models/issue.py:97). Los issues con state en triage NO deben
/// aparecer en listados generales — se exponen vía intake.
pub const STATE_GROUP_TRIAGE: &str = "triage";

// ── Cursor ────────────────────────────────────────────────────────────────────

/// Parsea el cursor de Django: `{page_size}:{current_page}:{is_prev}`.
///
/// Si el cursor no viene o está malformado, cae al `fallback_per_page` y
/// page 0. El `page_size` se clampa a `[1, PAGINATOR_MAX_LIMIT]` — evita
/// DoS por páginas gigantes.
pub fn parse_cursor(cursor: Option<&str>, fallback_per_page: u64) -> (u64, u64) {
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

// ── Enrichment batch ──────────────────────────────────────────────────────────

/// Relaciones cargadas en batch para los issues de una página.
///
/// Cada mapa es `issue_id → valor(es)`. Claves ausentes = valor default
/// (`vec![]` para listas, `0` para contadores, `None` para opcionales).
pub struct EnrichmentMaps {
    pub assignees:    HashMap<Uuid, Vec<Uuid>>,
    pub labels:       HashMap<Uuid, Vec<Uuid>>,
    pub modules:      HashMap<Uuid, Vec<Uuid>>,
    pub cycles:       HashMap<Uuid, Uuid>,
    pub sub_counts:   HashMap<Uuid, i64>,
    pub attachments:  HashMap<Uuid, i64>,
    pub links:        HashMap<Uuid, i64>,
    pub state_groups: HashMap<Uuid, String>,
}

impl EnrichmentMaps {
    /// Mapa vacío — usado cuando no hay issues en la página.
    fn empty() -> Self {
        Self {
            assignees:    HashMap::new(),
            labels:       HashMap::new(),
            modules:      HashMap::new(),
            cycles:       HashMap::new(),
            sub_counts:   HashMap::new(),
            attachments:  HashMap::new(),
            links:        HashMap::new(),
            state_groups: HashMap::new(),
        }
    }
}

/// Carga todas las relaciones necesarias para serializar una página de issues.
///
/// Espejo de las anotaciones de Django en
/// `apps/api/plane/app/views/issue/base.py:213-247` +
/// `apps/api/plane/utils/grouper.py:70-81`.
///
/// # Batching
/// Una sola query por tipo de relación → coste O(1) en roundtrips,
/// independiente del número de issues. Las queries son `SELECT ... WHERE
/// issue_id IN (...)`.
pub async fn load_enrichment(
    db: &DatabaseConnection,
    issue_ids: &[Uuid],
    state_ids: &[Uuid],
) -> Result<EnrichmentMaps, AppError> {
    if issue_ids.is_empty() {
        return Ok(EnrichmentMaps::empty());
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

    // Cycle IDs — solo el primer ciclo activo por issue.
    // Mirror del Subquery de Django: `CycleIssue.objects.filter(...).values("cycle_id")[:1]`
    // El `entry(...).or_insert(...)` preserva el primer valor visto.
    let raw_cycles = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut cycles: HashMap<Uuid, Uuid> = HashMap::new();
    for ci in raw_cycles {
        cycles.entry(ci.issue_id).or_insert(ci.cycle_id);
    }

    // Sub-issues count — agregación agrupada para evitar N+1.
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

    // Attachment counts — agrupado por issue_id (Option<Uuid> en el modelo,
    // porque `file_assets` también almacena attachments de otras entities
    // como páginas, comments, etc.). El `map(Some).collect()` envuelve los
    // UUIDs en `Option<Uuid>` para que matchee el tipo de la columna.
    let raw_attachments: Vec<(Option<Uuid>, i64)> = file_assets::Entity::find()
        .select_only()
        .column(file_assets::Column::IssueId)
        .column_as(
            sea_orm::sea_query::Expr::col(file_assets::Column::Id).count(),
            "cnt",
        )
        .filter(
            file_assets::Column::IssueId
                .is_in(issue_ids.iter().cloned().map(Some).collect::<Vec<_>>()),
        )
        .filter(file_assets::Column::EntityType.eq(ENTITY_TYPE_ISSUE_ATTACHMENT))
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

    // Link counts — agrupado.
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

    // State groups — solo para los states referenciados por los issues.
    let state_groups: HashMap<Uuid, String> = if state_ids.is_empty() {
        HashMap::new()
    } else {
        let rows = states::Entity::find()
            .select_only()
            .column(states::Column::Id)
            .column(states::Column::Group)
            .filter(states::Column::Id.is_in(state_ids.to_vec()))
            .into_tuple::<(Uuid, String)>()
            .all(db)
            .await
            .map_err(AppError::Database)?;
        rows.into_iter().collect()
    };

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

// ── Triage exclusion ──────────────────────────────────────────────────────────

/// Devuelve los IDs de `states` cuyo `group = 'triage'` en un workspace o
/// proyecto dado. Mirror de `.exclude(state__group=StateGroup.TRIAGE.value)`
/// en `IssueManager.get_queryset` (db/models/issue.py:97).
///
/// Preferimos pre-consultar estos IDs y filtrar `issue.state_id NOT IN (...)`
/// en lugar de hacer JOIN con `states` en cada query → mantiene los query
/// builders de SeaORM simples y tipados.
pub async fn load_triage_state_ids(
    db: &DatabaseConnection,
    project_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::Group.eq(STATE_GROUP_TRIAGE))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}

/// Versión workspace-scoped de `load_triage_state_ids` — carga los IDs de
/// states en triage de TODOS los proyectos del workspace.
///
/// Se usa en `list_workspace_view_issues` para aplicar la misma exclusión
/// que Django hace a nivel de manager, sin requerir JOIN con `states` en
/// cada query del listado.
pub async fn load_workspace_triage_state_ids(
    db: &DatabaseConnection,
    workspace_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::WorkspaceId.eq(workspace_id))
        .filter(states::Column::Group.eq(STATE_GROUP_TRIAGE))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}

// ── Ordenamiento ──────────────────────────────────────────────────────────────

/// Aplica el `order_by` de Django al SelectModel de SeaORM.
/// Prefijo `-` = descendente. Whitelist estricta — cualquier valor no
/// reconocido cae al default `-created_at` (no hay riesgo de SQL injection).
pub fn apply_issue_order(
    query: sea_orm::Select<issues::Entity>,
    order_by: &str,
) -> sea_orm::Select<issues::Entity> {
    use sea_orm::Order::{Asc, Desc};

    let (col, dir): (issues::Column, _) = match order_by {
        "-created_at"   => (issues::Column::CreatedAt,   Desc),
        "created_at"    => (issues::Column::CreatedAt,   Asc),
        "-updated_at"   => (issues::Column::UpdatedAt,   Desc),
        "updated_at"    => (issues::Column::UpdatedAt,   Asc),
        "-priority"     => (issues::Column::Priority,    Desc),
        "priority"      => (issues::Column::Priority,    Asc),
        "-sort_order"   => (issues::Column::SortOrder,   Desc),
        "sort_order"    => (issues::Column::SortOrder,   Asc),
        "-sequence_id"  => (issues::Column::SequenceId,  Desc),
        "sequence_id"   => (issues::Column::SequenceId,  Asc),
        "-target_date"  => (issues::Column::TargetDate,  Desc),
        "target_date"   => (issues::Column::TargetDate,  Asc),
        "-start_date"   => (issues::Column::StartDate,   Desc),
        "start_date"    => (issues::Column::StartDate,   Asc),
        "-completed_at" => (issues::Column::CompletedAt, Desc),
        "completed_at"  => (issues::Column::CompletedAt, Asc),
        // Default seguro
        _ => (issues::Column::CreatedAt, Desc),
    };

    query.order_by(col, dir)
}

// ── Respuesta paginada ────────────────────────────────────────────────────────

/// Construye el shape exacto de `OffsetPaginator.paginate()` vacío.
/// (`plane/utils/paginator.py:715-730`). Se usa en los early-returns cuando
/// no hay resultados válidos para el usuario.
pub fn empty_paginated_response(page_size: u64) -> serde_json::Value {
    json!({
        "grouped_by":        null,
        "sub_grouped_by":    null,
        "total_count":       0,
        "next_cursor":       format!("{page_size}:1:0"),
        "prev_cursor":       format!("{page_size}:-1:1"),
        "next_page_results": false,
        "prev_page_results": false,
        "count":             0,
        "total_pages":       0,
        "total_results":     0,
        "extra_stats":       null,
        "results":           [],
    })
}

/// Construye el shape paginado con resultados. `results` debe ser serializable.
///
/// Calcula `next_cursor` / `prev_cursor` y flags `*_page_results` a partir
/// de `current_page` y `total_results` — lógica idéntica a Django.
pub fn paginated_response<T: serde::Serialize>(
    results: Vec<T>,
    page_size: u64,
    current_page: u64,
    total_results: u64,
) -> serde_json::Value {
    let total_pages = if total_results == 0 {
        0
    } else {
        total_results.div_ceil(page_size)
    };
    let start_index = current_page * page_size;
    let end_index = (start_index + page_size).min(total_results);
    let has_next = end_index < total_results;
    let has_prev = current_page > 0;

    // En Django, next/prev cursor son SIEMPRE strings — la disponibilidad
    // de página se comunica vía `*_page_results`.
    let prev_cursor = if current_page == 0 {
        format!("{page_size}:-1:1")
    } else {
        format!("{page_size}:{}:1", current_page - 1)
    };
    let next_cursor = format!("{page_size}:{}:0", current_page + 1);
    let page_count = results.len() as u64;

    json!({
        "grouped_by":        null,
        "sub_grouped_by":    null,
        "total_count":       total_results,
        "next_cursor":       next_cursor,
        "prev_cursor":       prev_cursor,
        "next_page_results": has_next,
        "prev_page_results": has_prev,
        "count":             page_count,
        "total_pages":       total_pages,
        "total_results":     total_results,
        "extra_stats":       null,
        "results":           results,
    })
}

/// Extrae los `state_ids` únicos (descartando `None`) de una lista de issues.
/// Helper pequeño — evita repetir el pattern `filter_map + HashSet` en cada handler.
pub fn collect_state_ids(models: &[issues::Model]) -> Vec<Uuid> {
    models
        .iter()
        .filter_map(|i| i.state_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}
