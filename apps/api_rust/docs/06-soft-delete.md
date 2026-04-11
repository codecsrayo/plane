---
titulo: Soft Delete — SoftDeleteExt
tags:
  - seaorm
  - soft-delete
  - rust
relacionado: [[10-patrones]], [[12-cosas-criticas]]
---

## Soft delete — implementado ✅

**Decisión:** trait custom en `src/utils/soft_delete.rs` — NO se usa `seaorm-soft-delete` (crate 0.1.0, riesgo de incompatibilidad con sea-orm 1.1.x).

**Archivos:**

- `src/utils/soft_delete.rs` — trait `SoftDeleteExt<E>` + macro `impl_soft_delete!`
- `src/entities/mod.rs` — macro aplicado a las 98 entidades con `deleted_at`
- `src/utils/mod.rs` — módulo registrado
- `src/main.rs` — `pub mod utils` agregado

**Uso en rutas:**

```rust
use crate::utils::soft_delete::SoftDeleteExt;

let issues = issues::Entity::find()
    .active()                                      // WHERE deleted_at IS NULL
    .filter(issues::Column::ProjectId.eq(project_id))
    .all(&db)
    .await?;
```

**Errores del borrador original corregidos:**

- `pub use ..utils::...` → `use crate::utils::...` (ruta relativa inválida)
- `pub use SoftDeleteExt` → `use ... as _` (no re-exportar, solo activar impls)

---

> [!WARNING] No usar seaorm-soft-delete (crate externo)
> La versión 0.1.0 es incompatible con sea-orm 1.1.x.
> Se usa el trait custom en `src/utils/soft_delete.rs`.

