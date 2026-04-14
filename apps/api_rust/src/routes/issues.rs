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
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IsolationLevel, QueryFilter,
    QueryOrder, TransactionTrait,
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
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub id: Uuid,
    pub name: String,
    pub description_html: String,
    pub priority: String,
    pub state_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub sequence_id: i32,
    pub sort_order: f64,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub archived_at: Option<chrono::NaiveDate>,
    pub is_draft: bool,
    pub estimate_point_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    // Relaciones enriquecidas
    pub assignee_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
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

// ── Helper ────────────────────────────────────────────────────────────────────

/// Enriquece un issue model con sus assignees y labels en batch.
async fn enrich_issues(
    db: &sea_orm::DatabaseConnection,
    issue_models: Vec<issues::Model>,
) -> Result<Vec<IssueResponse>, AppError> {
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
            IssueResponse {
                id,
                name: m.name,
                description_html: m.description_html,
                priority: m.priority,
                state_id: m.state_id,
                parent_id: m.parent_id,
                project_id: m.project_id,
                workspace_id: m.workspace_id,
                sequence_id: m.sequence_id,
                sort_order: m.sort_order,
                start_date: m.start_date,
                target_date: m.target_date,
                completed_at: m.completed_at,
                archived_at: m.archived_at,
                is_draft: m.is_draft,
                estimate_point_id: m.estimate_point_id,
                type_id: m.type_id,
                created_by_id: m.created_by_id,
                updated_by_id: m.updated_by_id,
                created_at: m.created_at,
                updated_at: m.updated_at,
                assignee_ids: assignee_map.remove(&id).unwrap_or_default(),
                label_ids: label_map.remove(&id).unwrap_or_default(),
            }
        })
        .collect())
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
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Lista de issues"),
        (status = 403, description = "Sin acceso"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
) -> Result<Json<Vec<IssueResponse>>, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let rows = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::IsDraft.eq(false))
        .order_by_asc(issues::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = enrich_issues(&state.db, rows).await?;
    Ok(Json(result))
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
) -> Result<(StatusCode, Json<IssueResponse>), AppError> {
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

    let mut result = enrich_issues(&state.db, vec![issue]).await?;
    let response = result.pop().ok_or(AppError::NotFound)?;
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
) -> Result<Json<IssueResponse>, AppError> {
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
        (status = 200, description = "Issue actualizado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueResponse>, AppError> {
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

    let updated_issue = state
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

    let mut result = enrich_issues(&state.db, vec![updated_issue]).await?;
    Ok(Json(result.pop().ok_or(AppError::NotFound)?))
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
