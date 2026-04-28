// src/utils/soft_delete.rs
/// Soft-delete support for SeaORM entities.
///
/// Equivalent to Django's `SoftDeletionManager` — filters `deleted_at IS NULL`
/// automatically in any query.
///
/// # Uso
/// ```ignore
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
    /// Filters active records: `WHERE deleted_at IS NULL`
    fn active(self) -> Self;
}

/// Implements `SoftDeleteExt` for an entity with a `deleted_at` column.
/// `QueryFilter` and `Select` are imported inside the macro — not in the module.
///
/// # Ejemplo
/// ```ignore
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
