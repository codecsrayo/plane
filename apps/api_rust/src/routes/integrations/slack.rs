// src/routes/integrations/slack.rs
//! Slack project sync endpoints.
//!
//! Implemented endpoints:
//!   GET    /api/workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/
//!   POST   /api/workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/
//!   DELETE /api/workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/{sid}/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, require_workspace_admin, ROLE_MEMBER},
    },
    entities::{slack_project_syncs, workspace_integrations},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SlackProjectSyncResponse {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub access_token: String,
    pub scopes: String,
    pub bot_user_id: String,
    pub webhook_url: String,
    pub data: serde_json::Value,
    pub team_id: String,
    pub team_name: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub project: Uuid,
    pub workspace: Uuid,
    pub workspace_integration: Uuid,
}

impl SlackProjectSyncResponse {
    fn from_model(m: slack_project_syncs::Model) -> Self {
        Self {
            id: m.id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            access_token: m.access_token,
            scopes: m.scopes,
            bot_user_id: m.bot_user_id,
            webhook_url: m.webhook_url,
            data: m.data,
            team_id: m.team_id,
            team_name: m.team_name,
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            project: m.project_id,
            workspace: m.workspace_id,
            workspace_integration: m.workspace_integration_id,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SlackProjectSyncCreateRequest {
    pub access_token: String,
    pub scopes: String,
    pub bot_user_id: String,
    pub webhook_url: String,
    pub data: serde_json::Value,
    pub team_id: String,
    pub team_name: String,
}

// ── GET /workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/ ─

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 200, description = "Slack project syncs"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_project_slack_syncs(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, wi_id)): Path<(String, Uuid, Uuid)>,
) -> Result<Json<Vec<SlackProjectSyncResponse>>, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let syncs = slack_project_syncs::Entity::find()
        .active()
        .filter(slack_project_syncs::Column::WorkspaceIntegrationId.eq(wi_id))
        .filter(slack_project_syncs::Column::ProjectId.eq(guard.project.id))
        .filter(slack_project_syncs::Column::WorkspaceId.eq(guard.workspace.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        syncs
            .into_iter()
            .map(SlackProjectSyncResponse::from_model)
            .collect(),
    ))
}

// ── POST /workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/ ─

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
    ),
    responses(
        (status = 201, description = "Slack project sync created"),
        (status = 400, description = "Duplicate or validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn create_project_slack_sync(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, wi_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<SlackProjectSyncCreateRequest>,
) -> Result<(StatusCode, Json<SlackProjectSyncResponse>), AppError> {
    require_workspace_admin(&guard.workspace_member)?;

    // Verify workspace_integration exists and belongs to workspace.
    let _ = workspace_integrations::Entity::find_by_id(wi_id)
        .active()
        .filter(workspace_integrations::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Enforce unique_together = ["team_id", "project"] (Django constraint).
    let existing = slack_project_syncs::Entity::find()
        .active()
        .filter(slack_project_syncs::Column::TeamId.eq(&body.team_id))
        .filter(slack_project_syncs::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Slack sync for this team and project already exists".into(),
        ));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let sync = slack_project_syncs::ActiveModel {
        id: Set(Uuid::new_v4()),
        access_token: Set(body.access_token),
        scopes: Set(body.scopes),
        bot_user_id: Set(body.bot_user_id),
        webhook_url: Set(body.webhook_url),
        data: Set(body.data),
        team_id: Set(body.team_id),
        team_name: Set(body.team_name),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        workspace_integration_id: Set(wi_id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(SlackProjectSyncResponse::from_model(sync)),
    ))
}

// ── DELETE /workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/{sid}/ ─

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/workspace-integrations/{wi_id}/project-slack-sync/{sid}/",
    tag = "Integrations",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("wi_id" = Uuid, Path, description = "WorkspaceIntegration ID"),
        ("sid" = Uuid, Path, description = "SlackProjectSync ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn delete_project_slack_sync(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, wi_id, sid)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_workspace_admin(&guard.workspace_member)?;

    let sync = slack_project_syncs::Entity::find_by_id(sid)
        .active()
        .filter(slack_project_syncs::Column::WorkspaceIntegrationId.eq(wi_id))
        .filter(slack_project_syncs::Column::ProjectId.eq(guard.project.id))
        .filter(slack_project_syncs::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: slack_project_syncs::ActiveModel = sync.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}
