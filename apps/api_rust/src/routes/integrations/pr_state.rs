// src/routes/integrations/pr_state.rs
//! Endpoints de mapeo de estados de PR (GitHub PR State → Plane State).
//!
//! Endpoints implementados:
//!   GET    /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
//!   POST   /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
//!   DELETE /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    auth::{extractors::WorkspaceMemberGuard, permissions::{require_workspace_admin, require_workspace_member}},
    entities::{db_githubprstatemapping, workspace_integrations},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

use super::dtos::{PrStateMappingCreateRequest, PrStateMappingResponse, VALID_PR_STATES};

// ── GET /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/ ─

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 200, description = "Lista de mappings"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_pr_state_mappings(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PrStateMappingResponse>>, AppError> {
    // Django: @allow_permission([ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    // Members también pueden listar los mapeos de estado de PR.
    require_workspace_member(&guard.member)?;

    let mappings = db_githubprstatemapping::Entity::find()
        .active()
        .filter(db_githubprstatemapping::Column::WorkspaceIntegrationId.eq(wi_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        mappings
            .into_iter()
            .map(PrStateMappingResponse::from_model)
            .collect(),
    ))
}

// ── POST /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/ ─

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 201, description = "Mapping creado"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_pr_state_mapping(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id)): Path<(String, Uuid)>,
    Json(body): Json<PrStateMappingCreateRequest>,
) -> Result<(StatusCode, Json<PrStateMappingResponse>), AppError> {
    require_workspace_admin(&guard.member)?;

    if !VALID_PR_STATES.contains(&body.github_pr_state.as_str()) {
        return Err(AppError::BadRequest(format!(
            "github_pr_state inválido: '{}'. Valores permitidos: {}",
            body.github_pr_state,
            VALID_PR_STATES.join(", ")
        )));
    }

    // Verificar que la workspace_integration existe y pertenece al workspace.
    let _ = workspace_integrations::Entity::find_by_id(wi_id)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // created_at/updated_at explícitos (NOT NULL sin DEFAULT).
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let mapping = db_githubprstatemapping::ActiveModel {
        id: Set(Uuid::new_v4()),
        github_pr_state: Set(body.github_pr_state),
        project_id: Set(body.project_id),
        state_id: Set(body.state_id),
        workspace_integration_id: Set(wi_id),
        prevent_regression: Set(body.prevent_regression.unwrap_or(false)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(PrStateMappingResponse::from_model(mapping))))
}

// ── DELETE /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/ ─

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
        ("pk" = Uuid, Path, description = "Mapping ID"),
    ),
    responses(
        (status = 204, description = "Eliminado"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_pr_state_mapping(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, wi_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.member)?;

    let mapping = db_githubprstatemapping::Entity::find_by_id(pk)
        .active()
        .filter(db_githubprstatemapping::Column::WorkspaceIntegrationId.eq(wi_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: db_githubprstatemapping::ActiveModel = mapping.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
