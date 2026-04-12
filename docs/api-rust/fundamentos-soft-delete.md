---
titulo: Soft Delete — SoftDeleteExt
aliases:
  - soft-delete
  - SoftDeleteExt
tags:
  - seaorm
  - soft-delete
  - rust
  - fundamentos
relacionado:
  - "[[MOC]]"
  - "[[fundamentos-orm]]"
  - "[[impl-appstate-repository]]"
  - "[[plan-riesgos]]"
estado: activo
---

# Soft Delete — SoftDeleteExt

> [!SUCCESS] Estado: ✅ Implementado
> Archivos presentes en el repo y funcionando.

---

## Decisión técnica

**No se usa `seaorm-soft-delete` (crate externo).** La versión 0.1.0 es incompatible con sea-orm 1.1.x.

Se usa un **trait custom** en `src/utils/soft_delete.rs`.

---

## Archivos involucrados

| Archivo                    | Rol                                                  |
| -------------------------- | ---------------------------------------------------- |
| `src/utils/soft_delete.rs` | Trait `SoftDeleteExt<E>` + macro `impl_soft_delete!` |
| `src/entities/mod.rs`      | Macro aplicada a las 98 entidades con `deleted_at`   |
| `src/utils/mod.rs`         | Módulo registrado                                    |
| `src/main.rs`              | `pub mod utils` agregado                             |

---

## Implementación

### El trait

```rust
// src/utils/soft_delete.rs
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use chrono::Utc;

pub trait SoftDeleteExt<E: EntityTrait> {
    /// Filtra solo registros activos (WHERE deleted_at IS NULL)
    fn active(self) -> Self;
}

impl<E: EntityTrait> SoftDeleteExt<E> for Select<E>
where
    E::Column: ColumnTrait,
{
    fn active(self) -> Self {
        // Requiere que la entidad tenga una columna DeletedAt
        self.filter(/* E::Column::DeletedAt */ .is_null())
    }
}

/// Macro para aplicar el trait a entidades específicas
#[macro_export]
macro_rules! impl_soft_delete {
    ($entity:ty) => {
        impl crate::utils::soft_delete::SoftDeleteExt<$entity> for sea_orm::Select<$entity> {
            fn active(self) -> Self {
                use sea_orm::QueryFilter;
                self.filter(<$entity as sea_orm::EntityTrait>::Column::DeletedAt.is_null())
            }
        }
    };
}
```

### Aplicación en entities

```rust
// src/entities/mod.rs — aplicado a las 98 entidades con deleted_at
use crate::impl_soft_delete;

impl_soft_delete!(issues::Entity);
impl_soft_delete!(projects::Entity);
impl_soft_delete!(workspaces::Entity);
impl_soft_delete!(workspace_members::Entity);
// ... 94 entidades más
```

---

## Uso en rutas (handlers)

```rust
use crate::utils::soft_delete::SoftDeleteExt;

// ✅ Filtrar solo activos
let issues = issues::Entity::find()
    .active()                                      // WHERE deleted_at IS NULL
    .filter(issues::Column::ProjectId.eq(project_id))
    .all(&db)
    .await?;

// ⚠️ Cuando NO usar .active() — soft-delete de integraciones
// Al crear GithubRepositorySync, buscar INCLUDING deleted para evitar violar UNIQUE constraint
// Ver plan-riesgos punto 3
let existing = github_repository_syncs::Entity::find()
    // NO usar .active() aquí — necesitamos ver los soft-deleted
    .filter(github_repository_syncs::Column::GithubRepoId.eq(repo_id))
    .one(&db).await?;
```

---

## Soft delete de un registro

```rust
// Soft delete — actualizar deleted_at en lugar de DELETE
let mut active_model: issues::ActiveModel = issue.into();
active_model.deleted_at = Set(Some(chrono::Utc::now().into()));
active_model.update(&db).await?;
```

---

## Errores del borrador original (corregidos)

| Error original          | Corrección                                          |
| ----------------------- | --------------------------------------------------- |
| `pub use ..utils::...`  | `use crate::utils::...` (ruta relativa inválida)    |
| `pub use SoftDeleteExt` | `use ... as _` (no re-exportar, solo activar impls) |

---

## Entidades sin soft delete

Algunas entidades no tienen `deleted_at` — no usar `.active()` en ellas:

- `sessions` — expiración por `expire_date`
- `api_tokens` — desactivación por `is_active`
- `authtoken_token` — no tiene soft delete
- Tablas de relación M2M simples (`label_issue`, `cycle_issue`, etc.)

---

## Caso especial — integraciones

> [!WARNING] GithubRepository y GithubRepositorySync
> Al **crear** un repo sync, buscar PRIMERO con `all_objects` (incluyendo `deleted_at IS NOT NULL`) para no violar el constraint unique.
> Si existe uno soft-deleted → resucitar con `deleted_at = NULL`.
> Ver [[plan-riesgos]] punto 3 y [[dominio-integraciones]].

---

## 🔗 Navegar

← [[fundamentos-orm]] | [[MOC]] | Uso en repository: [[impl-appstate-repository]] | Riesgos: [[plan-riesgos]]
