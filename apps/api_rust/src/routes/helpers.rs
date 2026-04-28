// src/routes/helpers.rs
//! Domain helpers shared between routes.
//!
//! Centralizes workspace and membership queries to avoid duplication
//! between `workspaces.rs`, `projects.rs`, `states.rs`, etc.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entities::{workspace_members, workspaces},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
};

/// Retrieves the active workspace matching `slug`.
/// Returns `AppError::NotFound` if it does not exist or was deleted (soft-delete).
pub async fn workspace_by_slug(
    db: &sea_orm::DatabaseConnection,
    slug: &str,
) -> Result<workspaces::Model, AppError> {
    workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

/// Verifies that `user_id` is an active member of workspace `workspace_id`.
/// Returns `AppError::Forbidden` if not a member or deactivated.
pub async fn require_workspace_member(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<workspace_members::Model, AppError> {
    workspace_members::Entity::find()
        .active()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
        .filter(workspace_members::Column::MemberId.eq(user_id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Forbidden)
}

/// Retrieves the active ProjectMember for (project_id, user_id).
/// Returns Ok(None) if the user is not a project member.
/// Used by workspace_extras.rs to verify access before
/// converting a draft to an issue — reflects the same logic as
/// Django ProjectViewSet.get_queryset() which filters by ProjectMember.
pub async fn project_member_for_user(
    db: &sea_orm::DatabaseConnection,
    project_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<Option<crate::entities::project_members::Model>, crate::error::AppError> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::utils::soft_delete::SoftDeleteExt;
    crate::entities::project_members::Entity::find()
        .active()
        .filter(crate::entities::project_members::Column::ProjectId.eq(project_id))
        .filter(crate::entities::project_members::Column::MemberId.eq(user_id))
        .filter(crate::entities::project_members::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(crate::error::AppError::Database)
}
