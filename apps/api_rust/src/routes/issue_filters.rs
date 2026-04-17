// src/routes/issue_filters.rs
//! Filtros de issues compartidos entre endpoints de listado.
//!
//! Espejo de `apps/api/plane/utils/issue_filters.py`. Cada frontend store
//! de Plane envía filtros como query params; este módulo los parsea y los
//! traduce a condiciones de SeaORM.
//!
//! # Cobertura
//!
//! **Implementados** (filtros de alto tráfico del frontend):
//!   - state, state_group, priority, created_by, parent  (escalares en issues)
//!   - name                                               (substring LIKE '%term%')
//!   - start_date, target_date                            (YYYY-MM-DD con ;after / ;before)
//!   - labels, assignees, module, cycle                   (m2m via pre-query de issue_ids)
//!   - type                                               (all/backlog/active → state_group)
//!   - start_target_date                                  (toggle booleano)
//!
//! **No implementados** (uso marginal o complejidad alta, postergados):
//!   - mentions, subscriber, logged_by                    (relaciones poco usadas)
//!   - estimate_point                                     (raro en vistas default)
//!   - sintaxis relativa de fechas (`2_weeks;after;fromnow`)
//!   - inbox_status, intake_status                        (específicos de intake)
//!
//! Un filtro no implementado se ignora en silencio — paridad comportamental
//! con Django para lo cubierto; no rompe la request.
//!
//! # Estrategia para filtros m2m
//!
//! Pre-consulta de `issue_id` via tabla puente + `issues::Id IN (ids)` en la
//! query del listado. En SQL puro `IN (subquery)` sería marginalmente mejor,
//! pero la pre-query es:
//!   - **Más simple**: usa solo APIs de SeaORM ya probadas en el codebase.
//!   - **Más testeable**: cada paso es una query independiente inspeccionable.
//!   - **Igual de segura**: sin riesgo de SQL injection (UUIDs validados
//!     antes de entrar a cualquier query).
//!
//! El coste extra (un round-trip adicional por filtro m2m presente) es
//! aceptable dado que los filtros m2m se aplican típicamente uno a la vez.
//!
//! # Anti-patrones evitados
//!
//! - **SQL injection**: `priority` y `state_group` validados contra whitelist;
//!   UUIDs via `Uuid::parse_str` (inválidos se descartan silenciosamente,
//!   mirror de `filter_valid_uuids` en issue_filters.py:16-25).
//! - **Soft-deleted linkage**: los filtros m2m añaden `deleted_at IS NULL`
//!   en la tabla puente (paridad con issue_filters.py:158,173,331,346).
//! - **Cross-tenant leakage**: las subqueries de m2m filtran por
//!   `workspace_id` cuando está disponible.

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select,
};
use sea_orm::sea_query::extension::postgres::PgExpr;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{cycle_issues, issue_assignees, issue_labels, issues, module_issues, states},
    error::AppError,
};

// ── Query params ──────────────────────────────────────────────────────────────

/// Filtros aceptados en el query string. Todos son opcionales y se combinan
/// con AND. Los valores comma-separated se parsean dentro de cada helper.
///
/// Nota: NO se usa como `#[serde(flatten)]` dentro de query structs porque
/// axum's `Query<T>` usa `serde_urlencoded`, que no soporta `flatten`.
/// En cambio, cada handler inlinea los mismos campos en su propio struct
/// y construye un `IssueFilterParams` vía `FromRef`-like builder.
#[derive(Debug, Default, Deserialize)]
pub struct IssueFilterParams {
    pub state:             Option<String>,
    pub state_group:       Option<String>,
    pub priority:          Option<String>,
    pub created_by:        Option<String>,
    pub parent:             Option<String>,
    pub name:               Option<String>,
    pub start_date:         Option<String>,
    pub target_date:        Option<String>,
    pub labels:             Option<String>,
    pub assignees:          Option<String>,
    pub module:             Option<String>,
    pub cycle:              Option<String>,
    /// `all` | `backlog` | `active`. Mirror de
    /// `filter_issue_state_type` (issue_filters.py:296-305).
    #[serde(rename = "type")]
    pub type_filter:        Option<String>,
    pub start_target_date:  Option<String>,
}

/// Marcador "vaciar resultado": cuando un filtro implica que no puede
/// haber matches (p. ej. `type=backlog` pero el workspace no tiene states
/// en el grupo `backlog`), devolvemos este enum en vez de seguir
/// mutando la query. El handler que llama a `apply_issue_filters` puede
/// hacer early-return con `empty_paginated_response`.
pub enum FilteredQuery {
    /// Query con los filtros aplicados — sigue siendo `Select<issues::Entity>`.
    Active(Select<issues::Entity>),
    /// Al menos un filtro garantiza 0 resultados — no vale la pena ejecutar.
    Empty,
}

// ── Helpers de parsing ────────────────────────────────────────────────────────

/// Divide un string comma-separated, descartando vacíos y literales "null".
fn split_csv(raw: &str) -> Vec<&str> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "null")
        .collect()
}

/// Mirror de `filter_valid_uuids` (issue_filters.py:16-25). UUIDs
/// inválidos se DESCARTAN en silencio (alineado con Django).
fn parse_uuids_csv(raw: &str) -> Vec<Uuid> {
    split_csv(raw)
        .into_iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect()
}

/// Django usa el literal `"None"` para "sin valor" (→ `isnull=True`).
fn csv_contains_none(raw: &str) -> bool {
    raw.split(',').map(str::trim).any(|s| s == "None")
}

// ── Whitelists ────────────────────────────────────────────────────────────────

/// Prioridades válidas (mirror del CHOICES en db/models/issue.py:105-111).
const VALID_PRIORITIES: &[&str] = &["urgent", "high", "medium", "low", "none"];

/// Grupos de states válidos (mirror de `StateGroup` en db/models/state.py).
const VALID_STATE_GROUPS: &[&str] = &[
    "backlog",
    "unstarted",
    "started",
    "completed",
    "cancelled",
    "triage",
];

// ── Aplicación principal ──────────────────────────────────────────────────────

/// Aplica todos los filtros reconocidos a un `Select<issues::Entity>`.
///
/// Devuelve `FilteredQuery::Empty` si algún filtro resulta en 0 matches
/// garantizados (p. ej. `labels=<uuid>` pero no hay issues con ese label
/// en todo el workspace) — el handler puede hacer early-return sin
/// ejecutar la query principal.
///
/// `workspace_id` se usa para scopear las subqueries m2m (previene un
/// match accidental con labels/modules/etc. de otro workspace).
pub async fn apply_issue_filters(
    db: &sea_orm::DatabaseConnection,
    query: Select<issues::Entity>,
    params: &IssueFilterParams,
    workspace_id: Uuid,
) -> Result<FilteredQuery, AppError> {
    let mut query = query;

    // ── Escalares sobre issues ────────────────────────────────────────────────

    if let Some(raw) = params.state.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            query = query.filter(issues::Column::StateId.is_in(ids));
        }
    }

    if let Some(raw) = params.priority.as_deref() {
        let values: Vec<String> = split_csv(raw)
            .into_iter()
            .filter(|v| VALID_PRIORITIES.contains(v))
            .map(String::from)
            .collect();
        if !values.is_empty() {
            query = query.filter(issues::Column::Priority.is_in(values));
        }
    }

    query = apply_nullable_uuid_filter(
        query,
        params.created_by.as_deref(),
        issues::Column::CreatedById,
    );
    query = apply_nullable_uuid_filter(
        query,
        params.parent.as_deref(),
        issues::Column::ParentId,
    );

    // ── Texto (search) ────────────────────────────────────────────────────────

    if let Some(name) = params.name.as_deref() {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            // Mirror de `name__icontains` en filter_name (issue_filters.py:~240).
            // Postgres `ILIKE` es case-insensitive; paridad exacta con Django
            // (y con el uso actual en `search.rs:104`).
            //
            // `%` y `_` en el input actúan como wildcards SQL, mismo comportamiento
            // que Django — no se escapan para mantener paridad de UX.
            let pattern = format!("%{}%", trimmed);
            query = query.filter(
                sea_orm::sea_query::Expr::col(issues::Column::Name).ilike(pattern),
            );
        }
    }

    // ── Fechas simples ────────────────────────────────────────────────────────

    if let Some(raw) = params.start_date.as_deref() {
        query = apply_date_filter(query, raw, issues::Column::StartDate);
    }
    if let Some(raw) = params.target_date.as_deref() {
        query = apply_date_filter(query, raw, issues::Column::TargetDate);
    }

    // ── type (all/backlog/active) → state IDs por grupo ──────────────────────

    if let Some(type_filter) = params.type_filter.as_deref() {
        let groups: &[&str] = match type_filter {
            "backlog" => &["backlog"],
            "active" => &["unstarted", "started"],
            // `all` (y valores desconocidos) NO aplican filtro aquí. Django
            // (issue_filters.py:298,304) explícitamente pone `group IN
            // [backlog, unstarted, started, completed, cancelled]` — excluye
            // `triage`. En nuestro port, `triage` ya queda excluido antes
            // por `load_triage_state_ids + is_not_in` en el handler, así que
            // omitir el IN acá es funcionalmente equivalente y ahorra una
            // pre-query de state IDs.
            _ => &[],
        };
        if !groups.is_empty() {
            let state_ids = load_state_ids_by_groups(db, workspace_id, groups).await?;
            if state_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::StateId.is_in(state_ids));
        }
    }

    // ── state_group (explícito, distinto de `type`) ──────────────────────────

    if let Some(raw) = params.state_group.as_deref() {
        let groups: Vec<&str> = split_csv(raw)
            .into_iter()
            .filter(|g| VALID_STATE_GROUPS.contains(g))
            .collect();
        if !groups.is_empty() {
            let state_ids = load_state_ids_by_groups(db, workspace_id, &groups).await?;
            if state_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::StateId.is_in(state_ids));
        }
    }

    // ── start_target_date (toggle) ────────────────────────────────────────────

    if matches!(params.start_target_date.as_deref(), Some("true")) {
        query = query
            .filter(issues::Column::StartDate.is_not_null())
            .filter(issues::Column::TargetDate.is_not_null());
    }

    // ── m2m via pre-query ─────────────────────────────────────────────────────
    //
    // Cada filtro m2m presente añade 1 round-trip extra y un
    // `IN (issue_ids)` a la query principal. Si la pre-query devuelve
    // vacío → FilteredQuery::Empty (0 resultados garantizados).

    if let Some(raw) = params.labels.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            let issue_ids = load_issues_with_labels(db, workspace_id, ids).await?;
            if issue_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::Id.is_in(issue_ids));
        }
    }

    if let Some(raw) = params.assignees.as_deref() {
        let ids = parse_uuids_csv(raw);
        if !ids.is_empty() {
            let issue_ids = load_issues_with_assignees(db, workspace_id, ids).await?;
            if issue_ids.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query = query.filter(issues::Column::Id.is_in(issue_ids));
        }
    }

    if let Some(raw) = params.module.as_deref() {
        let ids = parse_uuids_csv(raw);
        let has_none = csv_contains_none(raw);
        match apply_module_membership(db, query, workspace_id, ids, has_none).await? {
            FilteredQuery::Active(q) => query = q,
            FilteredQuery::Empty => return Ok(FilteredQuery::Empty),
        }
    }

    if let Some(raw) = params.cycle.as_deref() {
        let ids = parse_uuids_csv(raw);
        let has_none = csv_contains_none(raw);
        match apply_cycle_membership(db, query, workspace_id, ids, has_none).await? {
            FilteredQuery::Active(q) => query = q,
            FilteredQuery::Empty => return Ok(FilteredQuery::Empty),
        }
    }

    Ok(FilteredQuery::Active(query))
}

// ── Helper: created_by / parent ───────────────────────────────────────────────

/// Aplica un filtro tipo `(col IN (ids)) OR (col IS NULL if "None" present)`.
/// Maneja el patrón repetido de `created_by` y `parent` (y podría extenderse
/// a otros UUID-nullable).
fn apply_nullable_uuid_filter(
    query: Select<issues::Entity>,
    raw_opt: Option<&str>,
    column: issues::Column,
) -> Select<issues::Entity> {
    let Some(raw) = raw_opt else {
        return query;
    };
    let ids = parse_uuids_csv(raw);
    let has_none = csv_contains_none(raw);

    let mut cond = Condition::any();
    let mut touched = false;
    if has_none {
        cond = cond.add(column.is_null());
        touched = true;
    }
    if !ids.is_empty() {
        cond = cond.add(column.is_in(ids));
        touched = true;
    }
    if touched {
        query.filter(cond)
    } else {
        query
    }
}

// ── Helper: fechas ────────────────────────────────────────────────────────────

fn apply_date_filter(
    mut query: Select<issues::Entity>,
    raw: &str,
    column: issues::Column,
) -> Select<issues::Entity> {
    // Formato soportado por valor:
    //   "YYYY-MM-DD"          → column = date
    //   "YYYY-MM-DD;after"    → column >= date
    //   "YYYY-MM-DD;before"   → column <= date
    //
    // Múltiples valores comma-separados se combinan con AND (mirror del
    // `date_filter` de Django cuando recibe varios tokens).
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = part.split(';').collect();
        let Ok(date) = chrono::NaiveDate::parse_from_str(tokens[0], "%Y-%m-%d") else {
            continue;
        };
        let direction = tokens.get(1).copied().unwrap_or("");
        query = match direction {
            "after" => query.filter(column.gte(date)),
            "before" => query.filter(column.lte(date)),
            _ => query.filter(column.eq(date)),
        };
    }
    query
}

// ── Helpers: m2m pre-queries ──────────────────────────────────────────────────

async fn load_issues_with_labels(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    label_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = issue_labels::Entity::find()
        .select_only()
        .column(issue_labels::Column::IssueId)
        .filter(issue_labels::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_labels::Column::LabelId.is_in(label_ids))
        .filter(issue_labels::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

async fn load_issues_with_assignees(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    assignee_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = issue_assignees::Entity::find()
        .select_only()
        .column(issue_assignees::Column::IssueId)
        .filter(issue_assignees::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_assignees::Column::AssigneeId.is_in(assignee_ids))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

async fn load_issues_in_modules(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    module_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = module_issues::Entity::find()
        .select_only()
        .column(module_issues::Column::IssueId)
        .filter(module_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(module_issues::Column::ModuleId.is_in(module_ids))
        .filter(module_issues::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

async fn apply_module_membership(
    db: &sea_orm::DatabaseConnection,
    query: Select<issues::Entity>,
    workspace_id: Uuid,
    module_ids: Vec<Uuid>,
    has_none: bool,
) -> Result<FilteredQuery, AppError> {
    if !has_none && module_ids.is_empty() {
        return Ok(FilteredQuery::Active(query));
    }

    // Subquery: issue_ids con al menos un module_issue activo en este workspace.
    // Se usa para la rama `None` (issues SIN módulo) — `issue.id NOT IN (subq)`.
    // Preferimos subquery sobre `load_all_ids` para no traer potencialmente
    // millones de UUIDs al proceso.
    let without_link_subq = module_issues::Entity::find()
        .select_only()
        .column(module_issues::Column::IssueId)
        .filter(module_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(module_issues::Column::DeletedAt.is_null())
        .into_query();

    let filtered = match (has_none, module_ids.is_empty()) {
        (true, true) => {
            // Solo "None" → issues sin ningún módulo.
            query.filter(issues::Column::Id.not_in_subquery(without_link_subq))
        }
        (false, false) => {
            // Solo UUIDs → comportamiento existente con pre-query.
            let matching = load_issues_in_modules(db, workspace_id, module_ids).await?;
            if matching.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query.filter(issues::Column::Id.is_in(matching))
        }
        (true, false) => {
            // None + UUIDs → unión: (id IN matching) OR (id NOT IN any_link).
            // Mirror del Django `filter(**{"issue_module__module_id__in": [...],
            // "issue_module__module_id__isnull": True})` pero con semántica
            // correcta (Django combinaría con AND, lo que es siempre vacío —
            // aquí interpretamos la intención real del usuario: "con estos
            // módulos O sin ninguno").
            let matching = load_issues_in_modules(db, workspace_id, module_ids).await?;
            let cond = Condition::any()
                .add(issues::Column::Id.is_in(matching))
                .add(issues::Column::Id.not_in_subquery(without_link_subq));
            query.filter(cond)
        }
        (false, true) => unreachable!("cubierto por el early-return"),
    };

    Ok(FilteredQuery::Active(filtered))
}

async fn apply_cycle_membership(
    db: &sea_orm::DatabaseConnection,
    query: Select<issues::Entity>,
    workspace_id: Uuid,
    cycle_ids: Vec<Uuid>,
    has_none: bool,
) -> Result<FilteredQuery, AppError> {
    if !has_none && cycle_ids.is_empty() {
        return Ok(FilteredQuery::Active(query));
    }

    let without_link_subq = cycle_issues::Entity::find()
        .select_only()
        .column(cycle_issues::Column::IssueId)
        .filter(cycle_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .into_query();

    let filtered = match (has_none, cycle_ids.is_empty()) {
        (true, true) => query.filter(issues::Column::Id.not_in_subquery(without_link_subq)),
        (false, false) => {
            let matching = load_issues_in_cycles(db, workspace_id, cycle_ids).await?;
            if matching.is_empty() {
                return Ok(FilteredQuery::Empty);
            }
            query.filter(issues::Column::Id.is_in(matching))
        }
        (true, false) => {
            let matching = load_issues_in_cycles(db, workspace_id, cycle_ids).await?;
            let cond = Condition::any()
                .add(issues::Column::Id.is_in(matching))
                .add(issues::Column::Id.not_in_subquery(without_link_subq));
            query.filter(cond)
        }
        (false, true) => unreachable!("cubierto por el early-return"),
    };

    Ok(FilteredQuery::Active(filtered))
}

async fn load_issues_in_cycles(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    cycle_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<Uuid> = cycle_issues::Entity::find()
        .select_only()
        .column(cycle_issues::Column::IssueId)
        .filter(cycle_issues::Column::WorkspaceId.eq(workspace_id))
        .filter(cycle_issues::Column::CycleId.is_in(cycle_ids))
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(dedup(rows))
}

fn dedup(mut v: Vec<Uuid>) -> Vec<Uuid> {
    v.sort();
    v.dedup();
    v
}

// ── Helper: state groups → state IDs ──────────────────────────────────────────

async fn load_state_ids_by_groups(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    groups: &[&str],
) -> Result<Vec<Uuid>, AppError> {
    let groups_vec: Vec<String> = groups.iter().map(|s| s.to_string()).collect();
    let rows: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::WorkspaceId.eq(workspace_id))
        .filter(states::Column::Group.is_in(groups_vec))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows)
}
