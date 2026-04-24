// src/routes/v1_router.rs
//! Router público `api/v1/` — autenticación por API key (`x-api-key`).
//!
//! Espejo de `apps/api/plane/api/urls/` (Django `plane.api` app).
//! Montado bajo `/api/v1/` en `mod.rs`.
//!
//! ## Handlers reutilizados
//! La mayoría de los handlers del router `api/` (session) usan `ProjectMemberGuard`
//! que internamente usa `AnyAuth` (acepta session cookie O API key).
//! Por tanto, apuntamos a los mismos handlers sin duplicar lógica.
//!
//! ## Handlers nuevos (solo en api/v1)
//! - `get_issue_activity`   — detalle de una actividad individual
//! - `get_project_summary`  — counts de members/states/labels/cycles/modules/issues/intakes/pages
//!
//! ## Rutas cubiertas
//! - Work items (legacy `/issues/` + nuevo `/work-items/`)
//! - Cycles, Modules, Labels, Members, States, Estimates
//! - Projects (CRUD + archive + summary)
//! - Intake issues
//! - Assets (user-assets + workspace assets v1)
//! - Users me
//!
//! ## Antipatrones evitados
//! - Sin duplicación de handlers: reutiliza los existentes vía AnyAuth.
//! - Sin SQL injection: filtros via SeaORM tipado.
//! - Sin N+1: queries batch donde aplica.

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        any_auth::AnyAuth,
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST},
    },
    entities::{
        cycles, intake_issues, issue_activities, labels, modules, project_members,
        project_pages, states,
    },
    error::AppError,
    routes::{
        assets,
        cycles as cycles_routes,
        estimates,
        intake,
        issue_extras,
        issue_extras2,
        issues,
        labels as labels_routes,
        modules as modules_routes,
        projects,
        states as states_routes,
        users,
        workspaces,
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Nuevos handlers exclusivos de api/v1 ─────────────────────────────────────

/// GET /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/{pk}/
/// GET /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}/
///
/// Detalle de una actividad específica de un issue.
/// Espejo de `IssueActivityDetailAPIEndpoint.get`
/// (`apps/api/plane/api/views/issue.py`).
/// Excluye activities de tipo comment/vote/reaction/draft.
pub async fn get_issue_activity(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let excluded = ["comment", "vote", "reaction", "draft"];

    let activity = issue_activities::Entity::find_by_id(pk)
        .active()
        .filter(issue_activities::Column::IssueId.eq(issue_id))
        .filter(issue_activities::Column::ProjectId.eq(guard.project.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .find(|a| {
            !a.field
                .as_deref()
                .map(|f| excluded.contains(&f))
                .unwrap_or(false)
        })
        .ok_or(AppError::NotFound)?;

    #[derive(Serialize)]
    struct ActivityDetail {
        id: Uuid,
        verb: String,
        field: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        comment: String,
        actor_id: Option<Uuid>,
        issue_id: Option<Uuid>,
        project_id: Uuid,
        workspace_id: Uuid,
        created_at: DateTime<FixedOffset>,
    }

    Ok(Json(ActivityDetail {
        id: activity.id,
        verb: activity.verb,
        field: activity.field,
        old_value: activity.old_value,
        new_value: activity.new_value,
        comment: activity.comment,
        actor_id: activity.actor_id,
        issue_id: activity.issue_id,
        project_id: activity.project_id,
        workspace_id: activity.workspace_id,
        created_at: activity.created_at,
    }))
}

/// GET /api/v1/workspaces/{slug}/projects/{project_id}/summary/
///
/// Counts de: members, states, labels, cycles, modules, issues, intakes, pages.
/// Espejo de `ProjectSummaryAPIEndpoint.get`
/// (`apps/api/plane/api/views/project.py`).
/// Requiere workspace admin.
#[derive(Debug, Deserialize)]
pub struct SummaryQuery {
    fields: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectSummaryResponse {
    id: Uuid,
    name: String,
    identifier: String,
    counts: serde_json::Value,
}

const ALLOWED_SUMMARY_FIELDS: &[&str] = &[
    "members", "states", "labels", "cycles", "modules", "issues", "intakes", "pages",
];

pub async fn get_project_summary(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((slug, project_id)): Path<(String, Uuid)>,
    Query(query): Query<SummaryQuery>,
) -> Result<impl IntoResponse, AppError> {
    use crate::entities::{projects, workspace_members, workspaces};

    // Verificar workspace membership y rol admin
    let ws = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let wm = workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(ws.id))
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Forbidden)?;

    // WorkSpaceAdminPermission: role >= ADMIN (20)
    const ROLE_ADMIN: i32 = 20;
    if i32::from(wm.role) < ROLE_ADMIN {
        return Err(AppError::Forbidden);
    }

    let project = projects::Entity::find_by_id(project_id)
        .active()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Determinar campos solicitados
    let requested: Vec<&str> = if let Some(ref f) = query.fields {
        f.split(',')
            .map(str::trim)
            .filter(|s| ALLOWED_SUMMARY_FIELDS.contains(s))
            .collect()
    } else {
        ALLOWED_SUMMARY_FIELDS.to_vec()
    };

    let requested: Vec<&str> = if requested.is_empty() {
        ALLOWED_SUMMARY_FIELDS.to_vec()
    } else {
        requested
    };

    // Contar cada campo solicitado con queries individuales.
    // Antipatrón evitado: NO se hacen N+1 innecesarios; cada count es una
    // sola query COUNT(*) con filtro por project_id.
    let db = &state.db;
    let mut counts = serde_json::Map::new();

    for field in &requested {
        let count: u64 = match *field {
            "members" => project_members::Entity::find()
                .filter(project_members::Column::ProjectId.eq(project_id))
                .filter(project_members::Column::IsActive.eq(true))
                .filter(project_members::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::Database)?,
            "states" => states::Entity::find()
                .filter(states::Column::ProjectId.eq(project_id))
                .filter(states::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::Database)?,
            "labels" => labels::Entity::find()
                .filter(labels::Column::ProjectId.eq(project_id))
                .filter(labels::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::Database)?,
            "cycles" => cycles::Entity::find()
                .filter(cycles::Column::ProjectId.eq(project_id))
                .filter(cycles::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::Database)?,
            "modules" => modules::Entity::find()
                .filter(modules::Column::ProjectId.eq(project_id))
                .filter(modules::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::Database)?,
            "issues" => {
                use crate::entities::issues;
                // Excluye issues en estado triage (group = 'triage')
                let triage_state_ids: Vec<Uuid> = states::Entity::find()
                    .filter(states::Column::ProjectId.eq(project_id))
                    .filter(states::Column::Group.eq("triage"))
                    .filter(states::Column::DeletedAt.is_null())
                    .all(db)
                    .await
                    .map_err(AppError::Database)?
                    .into_iter()
                    .map(|s| s.id)
                    .collect();

                let mut q = issues::Entity::find()
                    .filter(issues::Column::ProjectId.eq(project_id))
                    .filter(issues::Column::DeletedAt.is_null());

                if !triage_state_ids.is_empty() {
                    q = q.filter(
                        issues::Column::StateId
                            .is_not_in(triage_state_ids),
                    );
                }
                q.count(db).await.map_err(AppError::Database)?
            }
            "intakes" => intake_issues::Entity::find()
                .filter(intake_issues::Column::ProjectId.eq(project_id))
                .filter(intake_issues::Column::DeletedAt.is_null())
                .count(db)
                .await
                .map_err(AppError::Database)?,
            "pages" => project_pages::Entity::find()
                .filter(project_pages::Column::ProjectId.eq(project_id))
                .count(db)
                .await
                .map_err(AppError::Database)?,
            _ => 0,
        };
        counts.insert(field.to_string(), serde_json::json!(count));
    }

    Ok(Json(ProjectSummaryResponse {
        id: project.id,
        name: project.name,
        identifier: project.identifier,
        counts: serde_json::Value::Object(counts),
    }))
}

// ── Router builder ────────────────────────────────────────────────────────────

/// Construye el router `/api/v1/` con autenticación por API key.
///
/// Todos los handlers usan `AnyAuth` o `ProjectMemberGuard` (que usa `AnyAuth`)
/// por lo que aceptan tanto session cookie como header `x-api-key`.
pub fn v1_router(state: AppState) -> Router<AppState> {
    Router::new()
        // ── Users ──────────────────────────────────────────────────────────
        // GET/PATCH /api/v1/users/me/
        // Espejo de `UserAPIEndpoint` (plane/api/views/user.py)
        .route(
            "/users/me",
            get(users::get_me).patch(users::update_me),
        )
        // ── Workspace members ──────────────────────────────────────────────
        // GET /api/v1/workspaces/{slug}/members/
        .route(
            "/workspaces/{slug}/members",
            get(workspaces::list_members),
        )
        // ── Projects ───────────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/
        .route(
            "/workspaces/{slug}/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{pk}/
        .route(
            "/workspaces/{slug}/projects/{pk}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        // POST /api/v1/workspaces/{slug}/projects/{project_id}/archive/
        .route(
            "/workspaces/{slug}/projects/{project_id}/archive",
            post(projects::archive_project).delete(projects::unarchive_project),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/summary/
        .route(
            "/workspaces/{slug}/projects/{project_id}/summary",
            get(get_project_summary),
        )
        // ── Project Members ────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/members/
        .route(
            "/workspaces/{slug}/projects/{project_id}/members",
            get(projects::list_project_members).post(projects::create_project_members),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/members/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/members/{pk}",
            get(projects::get_project_member)
                .patch(projects::update_project_member)
                .delete(projects::remove_project_member),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/project-members/
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-members",
            get(projects::list_project_members),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/project-members/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/project-members/{pk}",
            get(projects::get_project_member),
        )
        // ── States ─────────────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/states/
        .route(
            "/workspaces/{slug}/projects/{project_id}/states",
            get(states_routes::list_states).post(states_routes::create_state),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/states/{state_id}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/states/{state_id}",
            get(states_routes::get_state)
                .patch(states_routes::update_state)
                .delete(states_routes::delete_state),
        )
        // ── Labels ─────────────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/labels/
        .route(
            "/workspaces/{slug}/projects/{project_id}/labels",
            get(labels_routes::list_labels).post(labels_routes::create_label),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/labels/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/labels/{pk}",
            get(labels_routes::get_label)
                .patch(labels_routes::update_label)
                .delete(labels_routes::delete_label),
        )
        // ── Estimates ──────────────────────────────────────────────────────
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/estimates/
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates",
            get(estimates::list_estimates),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points",
            post(estimates::create_estimate_point),
        )
        // PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{estimate_point_id}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}",
            patch(estimates::update_estimate_point).delete(estimates::delete_estimate_point),
        )
        // ── Cycles ─────────────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/cycles/
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles",
            get(cycles_routes::list_cycles).post(cycles_routes::create_cycle),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/cycles/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{pk}",
            get(cycles_routes::get_cycle)
                .patch(cycles_routes::update_cycle)
                .delete(cycles_routes::delete_cycle),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues",
            get(cycles_routes::list_cycle_issues).post(cycles_routes::add_issues_to_cycle),
        )
        // DELETE /api/v1/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}",
            delete(cycles_routes::remove_issue_from_cycle),
        )
        // POST /api/v1/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues/
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues",
            post(cycles_routes::transfer_cycle_issues),
        )
        // POST /api/v1/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive/
        .route(
            "/workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive",
            post(cycles_routes::archive_cycle),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/archived-cycles/
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-cycles",
            get(cycles_routes::list_archived_cycles),
        )
        // GET/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}/unarchive/
        // Nota: Django api/v1 usa DELETE para desarchivar (distinto de app/ que usa DELETE en mismo path)
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}",
            get(cycles_routes::get_archived_cycle),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}/unarchive",
            delete(cycles_routes::unarchive_cycle),
        )
        // ── Modules ────────────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/modules/
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules",
            get(modules_routes::list_modules).post(modules_routes::create_module),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/modules/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{pk}",
            get(modules_routes::get_module)
                .patch(modules_routes::update_module)
                .delete(modules_routes::delete_module),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues/
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues",
            get(modules_routes::list_module_issues)
                .post(modules_routes::add_issues_to_module),
        )
        // DELETE /api/v1/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues/{issue_id}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues/{issue_id}",
            delete(modules_routes::remove_issue_from_module),
        )
        // POST /api/v1/workspaces/{slug}/projects/{project_id}/modules/{pk}/archive/
        .route(
            "/workspaces/{slug}/projects/{project_id}/modules/{pk}/archive",
            post(modules_routes::archive_module),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/archived-modules/
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-modules",
            get(modules_routes::list_archived_modules),
        )
        // GET/DELETE-unarchive /api/v1/workspaces/{slug}/projects/{project_id}/archived-modules/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-modules/{pk}",
            get(modules_routes::get_archived_module),
        )
        .route(
            "/workspaces/{slug}/projects/{project_id}/archived-modules/{pk}/unarchive",
            delete(modules_routes::unarchive_module),
        )
        // ── Work Items (nuevo prefijo) ─────────────────────────────────────
        // GET /api/v1/workspaces/{slug}/work-items/search/
        .route(
            "/workspaces/{slug}/work-items/search",
            get(issues::list_issues),  // reutiliza list con ?search= param
        )
        // GET /api/v1/workspaces/{slug}/work-items/{combined}
        // (project_identifier-issue_identifier)
        .route(
            "/workspaces/{slug}/work-items/{combined}",
            get(issue_extras2::get_issue_by_identifier),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/work-items/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items",
            get(issues::list_issues).post(issues::create_issue),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{pk}",
            get(issues::get_issue)
                .patch(issues::update_issue)
                .delete(issues::delete_issue),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links",
            get(issue_extras::list_issue_links).post(issue_extras::create_issue_link),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links/{pk}",
            patch(issue_extras::update_issue_link).delete(issue_extras::delete_issue_link),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments",
            get(issue_extras::list_comments).post(issue_extras::create_comment),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments/{pk}",
            get(issue_extras::get_comment)
                .patch(issue_extras::update_comment)
                .delete(issue_extras::delete_comment),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities",
            get(issue_extras::list_issue_activities),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}",
            get(get_issue_activity),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments",
            get(issue_extras2::list_issue_attachments)
                .post(issue_extras2::initiate_issue_attachment_upload),
        )
        // PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments/{pk}",
            patch(issue_extras2::complete_issue_attachment_upload)
                .delete(issue_extras2::delete_issue_attachment),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/relations/
        .route(
            "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/relations",
            get(issue_extras::list_issue_relations).post(issue_extras::create_issue_relation),
        )
        // ── Issues (legacy prefijo, mismo path que work-items) ────────────
        // GET /api/v1/workspaces/{slug}/issues/search/
        .route(
            "/workspaces/{slug}/issues/search",
            get(issues::list_issues),
        )
        // GET /api/v1/workspaces/{slug}/issues/{combined}
        .route(
            "/workspaces/{slug}/issues/{combined}",
            get(issue_extras2::get_issue_by_identifier),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/issues/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues",
            get(issues::list_issues).post(issues::create_issue),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/issues/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{pk}",
            get(issues::get_issue)
                .patch(issues::update_issue)
                .delete(issues::delete_issue),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links",
            get(issue_extras::list_issue_links).post(issue_extras::create_issue_link),
        )
        // PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links/{pk}",
            patch(issue_extras::update_issue_link).delete(issue_extras::delete_issue_link),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments",
            get(issue_extras::list_comments).post(issue_extras::create_comment),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}",
            get(issue_extras::get_comment)
                .patch(issue_extras::update_comment)
                .delete(issue_extras::delete_comment),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities",
            get(issue_extras::list_issue_activities),
        )
        // GET /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/{pk}",
            get(get_issue_activity),
        )
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments",
            get(issue_extras2::list_issue_attachments)
                .post(issue_extras2::initiate_issue_attachment_upload),
        )
        // PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}",
            patch(issue_extras2::complete_issue_attachment_upload)
                .delete(issue_extras2::delete_issue_attachment),
        )
        // ── Intake Issues ──────────────────────────────────────────────────
        // GET/POST /api/v1/workspaces/{slug}/projects/{project_id}/intake-issues/
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-issues",
            get(intake::list_intake_issues).post(intake::create_intake_issue),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}/
        .route(
            "/workspaces/{slug}/projects/{project_id}/intake-issues/{pk}",
            get(intake::get_intake_issue)
                .patch(intake::update_intake_issue)
                .delete(intake::delete_intake_issue),
        )
        // ── Assets (api/v1 — paths sin prefijo v2/) ────────────────────────
        // POST /api/v1/assets/user-assets/
        .route(
            "/assets/user-assets",
            post(assets::initiate_user_asset_upload),
        )
        // PATCH /api/v1/assets/user-assets/{asset_id}/
        .route(
            "/assets/user-assets/{asset_id}",
            patch(assets::complete_user_asset_upload).delete(assets::delete_user_asset),
        )
        // POST /api/v1/assets/user-assets/server/ — server-side upload
        .route(
            "/assets/user-assets/server",
            post(assets::initiate_user_asset_upload),
        )
        // POST /api/v1/assets/user-assets/{asset_id}/server/
        .route(
            "/assets/user-assets/{asset_id}/server",
            post(assets::complete_user_asset_upload),
        )
        // POST /api/v1/workspaces/{slug}/assets/
        .route(
            "/workspaces/{slug}/assets",
            post(assets::initiate_workspace_asset_upload),
        )
        // GET/PATCH/DELETE /api/v1/workspaces/{slug}/assets/{asset_id}/
        .route(
            "/workspaces/{slug}/assets/{asset_id}",
            get(assets::get_workspace_asset)
                .patch(assets::complete_workspace_asset_upload)
                .delete(assets::delete_workspace_asset),
        )
        .with_state(state)
}
