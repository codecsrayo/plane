use std::collections::HashMap;

use axum::{
    extract::{FromRef, FromRequestParts, Path},
    http::request::Parts,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    auth::api_key::ApiKeyUser,
    entities::{project_members, projects, workspace_members, workspaces, users},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

pub struct WorkspaceMemberGuard {
    pub user: users::Model,
    pub workspace: workspaces::Model,
    pub member: workspace_members::Model,
}

impl<S> FromRequestParts<S> for WorkspaceMemberGuard
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = AppState::from_ref(state).db;
        let ApiKeyUser(ctx) = ApiKeyUser::from_request_parts(parts, state).await?;
        let user = ctx.user;
        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound)?;
        let slug = params.get("slug").ok_or(AppError::NotFound)?;

        let workspace = workspaces::Entity::find()
            .active()
            .filter(workspaces::Column::Slug.eq(slug.as_str()))
            .one(&db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

        let member = workspace_members::Entity::find()
            .active()
            .filter(workspace_members::Column::WorkspaceId.eq(workspace.id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .filter(workspace_members::Column::IsActive.eq(true))
            .one(&db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Forbidden)?;

        Ok(Self {
            user,
            workspace,
            member,
        })
    }
}

pub struct ProjectMemberGuard {
    pub user: users::Model,
    pub workspace: workspaces::Model,
    pub project: projects::Model,
    pub workspace_member: workspace_members::Model,
    pub project_member: project_members::Model,
}

impl<S> FromRequestParts<S> for ProjectMemberGuard
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = AppState::from_ref(state).db;
        let WorkspaceMemberGuard {
            user,
            workspace,
            member: workspace_member,
        } = WorkspaceMemberGuard::from_request_parts(parts, state).await?;

        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound)?;
        let project_id: Uuid = params
            .get("project_id")
            .or_else(|| params.get("id"))
            .and_then(|s| s.parse().ok())
            .ok_or(AppError::NotFound)?;

        let project = projects::Entity::find_by_id(project_id)
            .filter(projects::Column::WorkspaceId.eq(workspace.id))
            .filter(projects::Column::DeletedAt.is_null())
            .one(&db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

        let project_member = project_members::Entity::find()
            .active()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::WorkspaceId.eq(workspace.id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .one(&db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Forbidden)?;

        Ok(Self {
            user,
            workspace,
            project,
            workspace_member,
            project_member,
        })
    }
}
