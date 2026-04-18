// src/routes/user_profile_issues.rs
//! Endpoint `GET /api/workspaces/{slug}/user-issues/{user_id}/`.
//!
//! Mirror de `WorkspaceUserProfileIssuesEndpoint` en
//! `plane/app/views/workspace/user.py:98-249`.
//!
//! Lista issues asociadas al `user_id` (target) dentro del workspace,
//! filtradas por proyectos a los que el **requester** tiene acceso.
//!
//! # Scope del target user (mirror Django)
//!
//! ```python
//! id__in=Issue.issue_objects.filter(
//!     Q(assignees__in=[user_id])
//!     | Q(created_by_id=user_id)
//!     | Q(issue_subscribers__subscriber_id=user_id),
//!     workspace__slug=slug,
//! ).values_list("id", flat=True)
//! ```
//!
//! Ese OR se aplica SIEMPRE, independientemente de que el frontend además
//! envíe `?assignees=user_id`, `?created_by=user_id` o `?subscriber=user_id`
//! para las pestañas Assigned / Created / Subscribed. La ausencia del OR
//! permitiría leer todos los issues del workspace cuando el frontend
//! omitiera el filtro — un privilege escalation silencioso respecto a Django.
//!
//! # Permisos
//!
//! Mirror de `WorkspaceViewerPermission`: el requester debe ser miembro
//! activo del workspace (cualquier rol). Issues se scope-an a proyectos
//! donde el requester sea miembro activo + lógica de guest:
//! `guest_view_all_features = false` restringe a issues creadas por el
//! propio requester. Idéntico a `list_workspace_view_issues`.
//!
//! # Antipatrones evitados
//!
//! - **N+1**: pre-query única del `id__in` del target; enrichment batch
//!   (8 relaciones) a través de `load_enrichment`.
//! - **Cross-tenant leakage**: todas las subqueries filtran por
//!   `workspace_id`, incluidas las del scope del target user.
//! - **Privilege escalation**: el OR-scope del target user no es opcional
//!   — se aplica aunque el frontend mande filtros; además se aplica la
//!   `permission_condition` estándar (full_access vs guest restringido).
//! - **SQL injection**: UUIDs se parsean/validan antes de entrar a la query;
//!   SeaORM bindea los parámetros. Sin `format!`.
//! - **DoS por paginación**: `parse_cursor` clampa `page_size` a
//!   `PAGINATOR_MAX_LIMIT`.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    auth::{any_auth::AnyAuth, permissions::ROLE_GUEST},
    entities::{issue_assignees, issue_subscribers, issues, project_members, projects},
    error::AppError,
    routes::{
        helpers::{require_workspace_member, workspace_by_slug},
        issue_filters::{apply_issue_filters, merge_json_filters, FilteredQuery, IssueFilterParams},
        issue_pagination::{
            apply_issue_order, collect_state_ids, empty_paginated_response, load_enrichment,
            load_workspace_triage_state_ids, paginated_response, parse_cursor, DEFAULT_PER_PAGE,
        },
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Query params ──────────────────────────────────────────────────────────────
//
// `serde_urlencoded` no soporta `#[serde(flatten)]`, por eso los campos de
// `IssueFilterParams` se inlinean en el struct del query. Siguiendo el mismo
// patrón de `WorkspaceIssuesQuery` en `workspace_view_issues.rs`.

#[derive(Debug, Deserialize)]
pub struct UserProfileIssuesQuery {
    // Paginación / orden
    pub cursor:            Option<String>,
    pub per_page:          Option<u64>,
    pub order_by:          Option<String>,

    // Toggles
    pub sub_issue:         Option<String>,
    #[serde(rename = "updated_at__gt")]
    pub updated_at_gt:     Option<chrono::DateTime<chrono::FixedOffset>>,

    // Filtros (delegados a issue_filters)
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
    pub subscriber:        Option<String>,
    #[serde(rename = "type")]
    pub type_filter:       Option<String>,
    pub start_target_date: Option<String>,

    // Rich filters (JSON blob para views guardadas / spreadsheet layout)
    pub filters:           Option<String>,
}

impl UserProfileIssuesQuery {
    fn to_filter_params(&self) -> IssueFilterParams {
        IssueFilterParams {
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

// ── DTO ───────────────────────────────────────────────────────────────────────
//
// Shape idéntico a `WorkspaceIssueItem` de `workspace_view_issues.rs`. El
// frontend consume ambos con `TIssuesResponse` (packages/types), por lo que
// los campos deben permanecer en paridad.

#[derive(Debug, Serialize)]
pub struct UserProfileIssueItem {
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

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/workspaces/{slug}/user-issues/{user_id}/`
///
/// Lista paginada de issues asociadas al `user_id` (target) dentro de los
/// proyectos a los que el **requester** tiene acceso en el workspace.
///
/// Mirror de `WorkspaceUserProfileIssuesEndpoint`
/// (`plane/app/views/workspace/user.py:98-249`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/user-issues/{user_id}/",
    tag = "Workspaces",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug"     = String, Path,  description = "Workspace slug"),
        ("user_id"  = Uuid,   Path,  description = "Target user UUID"),
        ("cursor"   = Option<String>, Query, description = "Cursor Django: {per_page}:{page}:{is_prev}"),
        ("per_page" = Option<u64>,    Query, description = "Tamaño de página (ignorado si está en cursor)"),
        ("order_by" = Option<String>, Query, description = "Campo de ordenamiento, default `-created_at`"),
    ),
    responses(
        (status = 200, description = "Lista paginada de issues del perfil del usuario"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "No es miembro activo del workspace"),
        (status = 404, description = "Workspace no encontrado"),
    )
)]
pub async fn list_user_profile_issues(
    State(state): State<AppState>,
    AnyAuth(requester): AnyAuth,
    Path((slug, target_user_id)): Path<(String, Uuid)>,
    Query(params): Query<UserProfileIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;

    // ── 1. Auth + permisos de workspace ──────────────────────────────────────
    let ws = workspace_by_slug(db, &slug).await?;
    let workspace_id = ws.id;
    let requester_id = requester.id;
    let workspace_member = require_workspace_member(db, workspace_id, requester_id).await?;
    let workspace_member_role = workspace_member.role;

    // ── 2. Paginación ────────────────────────────────────────────────────────
    let fallback_per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE);
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), fallback_per_page);

    // ── 3. Proyectos accesibles por el requester (mirror view_issues) ────────
    //
    // Misma lógica de `_get_project_permission_filters` de Django:
    //   - Guest (role = 5) con `guest_view_all_features = false` → sólo ve
    //     sus propios issues (created_by_id = requester).
    //   - Guest con `guest_view_all_features = true` o roles > 5 → ve todos
    //     los issues del proyecto.
    let memberships = project_members::Entity::find()
        .active()
        .filter(project_members::Column::WorkspaceId.eq(workspace_id))
        .filter(project_members::Column::MemberId.eq(requester_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if memberships.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let project_ids: Vec<Uuid> = memberships.iter().map(|m| m.project_id).collect();

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

    let project_meta: HashMap<Uuid, (bool, bool)> = project_rows
        .into_iter()
        .map(|(id, gvaf, archived)| (id, (gvaf, archived.is_some())))
        .collect();

    let mut full_access_ids: HashSet<Uuid> = HashSet::new();
    let mut restricted_ids: HashSet<Uuid> = HashSet::new();

    for membership in &memberships {
        let pid = membership.project_id;
        let role = membership.role;

        let (guest_view_all_features, is_archived) = match project_meta.get(&pid) {
            Some(meta) => *meta,
            None => continue,
        };
        if is_archived {
            continue;
        }

        if workspace_member_role >= 20 || role > ROLE_GUEST {
            full_access_ids.insert(pid);
        } else if guest_view_all_features {
            full_access_ids.insert(pid);
        } else {
            restricted_ids.insert(pid);
        }
    }

    let full_ids: Vec<Uuid> = full_access_ids.into_iter().collect();
    let rest_ids: Vec<Uuid> = restricted_ids.into_iter().collect();

    if full_ids.is_empty() && rest_ids.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    // Condición de permisos: full_access OR (restricted AND created_by = requester)
    let permission_condition = {
        let mut cond = Condition::any();
        if !full_ids.is_empty() {
            cond = cond.add(issues::Column::ProjectId.is_in(full_ids.clone()));
        }
        if !rest_ids.is_empty() {
            cond = cond.add(
                Condition::all()
                    .add(issues::Column::ProjectId.is_in(rest_ids.clone()))
                    .add(issues::Column::CreatedById.eq(requester_id)),
            );
        }
        cond
    };

    // ── 4. Scope OR del target user (assignee ∪ created_by ∪ subscriber) ─────
    //
    // Mirror EXACTO del `id__in=...` de Django. Se ejecuta SIEMPRE — es el
    // invariante de este endpoint: nunca devuelve issues no ligadas al target.
    // Los filtros del query string del frontend narrowean sobre este OR.
    //
    // Implementación: tres pre-queries batch, luego union en memoria.
    // Alternativa SQL `UNION` sería marginalmente más rápida, pero este
    // patrón es consistente con `issue_filters::load_issues_with_*` y evita
    // un statement custom adicional.
    let target_issue_ids = load_target_user_issue_ids(db, workspace_id, target_user_id).await?;
    if target_issue_ids.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    // ── 5. Query base ────────────────────────────────────────────────────────
    let exclude_sub_issues = params
        .sub_issue
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    let mut base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::IsDraft.eq(false))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::Id.is_in(target_issue_ids))
        .filter(permission_condition);

    if exclude_sub_issues {
        base_query = base_query.filter(issues::Column::ParentId.is_null());
    }

    // Exclusión de triage (mirror `IssueManager.get_queryset` en
    // db/models/issue.py:97).
    let triage_state_ids = load_workspace_triage_state_ids(db, workspace_id).await?;
    if !triage_state_ids.is_empty() {
        base_query = base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids));
    }

    if let Some(updated_at_gt) = params.updated_at_gt {
        base_query = base_query.filter(issues::Column::UpdatedAt.gt(updated_at_gt));
    }

    // ── 6. Filtros de query string + blob JSON ───────────────────────────────
    let mut filter_params = params.to_filter_params();
    merge_json_filters(params.filters.as_deref(), &mut filter_params)?;
    let filtered = apply_issue_filters(db, base_query, &filter_params, workspace_id).await?;
    let base_query = match filtered {
        FilteredQuery::Active(q) => q,
        FilteredQuery::Empty => return Ok(Json(empty_paginated_response(page_size))),
    };

    // ── 7. Total count + orden + paginación offset ───────────────────────────
    let total_results = base_query
        .clone()
        .count(db)
        .await
        .map_err(AppError::Database)?;

    let order_by_param = params.order_by.as_deref().unwrap_or("-created_at");
    let ordered_query = apply_issue_order(base_query, order_by_param);

    let start_index = current_page * page_size;

    let issue_models = ordered_query
        .offset(start_index)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // ── 8. Enrichment batch (sin N+1) ────────────────────────────────────────
    let issue_ids: Vec<Uuid> = issue_models.iter().map(|i| i.id).collect();
    let state_ids = collect_state_ids(&issue_models);
    let mut enrich = load_enrichment(db, &issue_ids, &state_ids).await?;

    // ── 9. Serializar ────────────────────────────────────────────────────────
    let results: Vec<UserProfileIssueItem> = issue_models
        .into_iter()
        .map(|m| {
            let id = m.id;
            let state_group = m
                .state_id
                .and_then(|sid| enrich.state_groups.get(&sid).cloned());

            UserProfileIssueItem {
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

    Ok(Json(paginated_response(
        results,
        page_size,
        current_page,
        total_results,
    )))
}

// ── Helpers internos ──────────────────────────────────────────────────────────

/// Pre-query: IDs de issues donde `target_user_id` es assignee, created_by
/// o subscriber dentro del workspace. Mirror del subquery Django:
///
/// ```python
/// Issue.issue_objects.filter(
///     Q(assignees__in=[user_id])
///     | Q(created_by_id=user_id)
///     | Q(issue_subscribers__subscriber_id=user_id),
///     workspace__slug=slug,
/// ).values_list("id", flat=True)
/// ```
///
/// Se hacen 3 queries batch y se unen en memoria (via `HashSet`). Más simple
/// y testeable que un `UNION` SQL custom, con overhead despreciable para los
/// volúmenes esperados (issues por usuario típicamente < 10k).
async fn load_target_user_issue_ids(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    target_user_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let mut ids: HashSet<Uuid> = HashSet::new();

    // Assignee
    let assigned: Vec<Uuid> = issue_assignees::Entity::find()
        .select_only()
        .column(issue_assignees::Column::IssueId)
        .filter(issue_assignees::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_assignees::Column::AssigneeId.eq(target_user_id))
        .filter(issue_assignees::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    ids.extend(assigned);

    // Created by
    let created: Vec<Uuid> = issues::Entity::find()
        .select_only()
        .column(issues::Column::Id)
        .filter(issues::Column::WorkspaceId.eq(workspace_id))
        .filter(issues::Column::CreatedById.eq(target_user_id))
        .filter(issues::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    ids.extend(created);

    // Subscriber
    let subscribed: Vec<Uuid> = issue_subscribers::Entity::find()
        .select_only()
        .column(issue_subscribers::Column::IssueId)
        .filter(issue_subscribers::Column::WorkspaceId.eq(workspace_id))
        .filter(issue_subscribers::Column::SubscriberId.eq(target_user_id))
        .filter(issue_subscribers::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    ids.extend(subscribed);

    Ok(ids.into_iter().collect())
}
