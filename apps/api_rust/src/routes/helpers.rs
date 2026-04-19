// src/routes/helpers.rs
//! Helpers de dominio compartidos entre rutas.
//!
//! Centraliza las consultas de workspace y membresÃ­a para evitar duplicaciÃ³n
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

/// Recupera el ProjectMember activo para (project_id, user_id).
/// Retorna Ok(None) si el usuario no es miembro del proyecto.
/// Usado por workspace_extras.rs para verificar acceso antes de
/// convertir un draft en issue — refleja la misma logica de
/// Django ProjectViewSet.get_queryset() que filtra por ProjectMember.
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
