// src/routes/helpers.rs
//! Helpers de dominio compartidos entre rutas.
//!
//! Centraliza las consultas de workspace y membresía para evitar duplicación
//! entre `workspaces.rs`, `projects.rs`, `states.rs`, etc.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entities::{workspace_members, workspaces},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
};

/// Recupera el workspace activo que coincide con `slug`.
/// Retorna `AppError::NotFound` si no existe o fue eliminado (soft-delete).
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

/// Verifica que `user_id` sea miembro activo del workspace `workspace_id`.
/// Retorna `AppError::Forbidden` si no es miembro o fue desactivado.
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
