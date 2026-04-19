// src/utils/soft_delete.rs
/// Soft-delete support para entidades SeaORM.
///
/// Equivalente al `SoftDeletionManager` de Django — filtra `deleted_at IS NULL`
/// automáticamente en cualquier query.
///
/// # Uso
/// ```rust
/// use crate::utils::soft_delete::SoftDeleteExt;
///
/// let issues = Issue::find()
///     .active()                                        // ← deleted_at IS NULL
///     .filter(issues::Column::ProjectId.eq(project_id))
///     .all(&db)
///     .await?;
/// ```
use sea_orm::EntityTrait;

pub trait SoftDeleteExt<E: EntityTrait>: Sized {
    /// Filtra registros activos: `WHERE deleted_at IS NULL`
    fn active(self) -> Self;
}

/// Implementa `SoftDeleteExt` para una entidad con columna `deleted_at`.
/// `QueryFilter` y `Select` se importan dentro del macro — no en el módulo.
///
/// # Ejemplo
/// ```rust
/// impl_soft_delete!(issues::Entity, issues::Column::DeletedAt);
/// ```
#[macro_export]
macro_rules! impl_soft_delete {
    ($entity:path, $column:path) => {
        impl $crate::utils::soft_delete::SoftDeleteExt<$entity> for sea_orm::Select<$entity> {
            fn active(self) -> Self {
                use sea_orm::{ColumnTrait, QueryFilter};
                self.filter($column.is_null())
            }
        }
    };
}
