// src/utils/django_defaults.rs
//! Defaults JSON canónicos de Django para columnas `jsonb NOT NULL`.
//!
//! Django define estos valores en `models.JSONField(default=...)` a nivel
//! Python — la BD NO tiene cláusulas DEFAULT correspondientes (las migraciones
//! solo aplican el default en el momento del INSERT vía Python). Por eso los
//! handlers Rust deben proveer el mismo JSON explícitamente al insertar.
//!
//! Fuente canónica:
//! - `apps/api/plane/db/models/project.py` → `get_default_props`,
//!   `get_default_preferences`
//! - `apps/api/plane/db/models/issue.py` → `get_default_filters`,
//!   `get_default_display_filters`, `get_default_display_properties`
//!
//! Estos defaults se usan al crear `project_members`,
//! `project_user_properties`, `cycle_user_properties`, `module_user_properties`,
//! `workspace_members` y otras tablas que hereden de `IssueProperty`.

use serde_json::{json, Value};

/// Mirror de `get_default_filters()` (issue.py).
///
/// Estructura usada en `*UserProperty.filters`. Todos los slots arrancan en
/// `null` para que el frontend muestre todos los issues sin filtrar.
pub fn default_filters() -> Value {
    json!({
        "priority": null,
        "state": null,
        "state_group": null,
        "assignees": null,
        "created_by": null,
        "labels": null,
        "start_date": null,
        "target_date": null,
        "subscriber": null,
    })
}

/// Mirror de `get_default_display_filters()` (issue.py).
///
/// Configuración de visualización por defecto: lista, ordenado por fecha de
/// creación descendente, mostrando sub-issues y grupos vacíos.
pub fn default_display_filters() -> Value {
    json!({
        "group_by": null,
        "order_by": "-created_at",
        "type": null,
        "sub_issue": true,
        "show_empty_groups": true,
        "layout": "list",
        "calendar_date_range": "",
    })
}

/// Mirror de `get_default_display_properties()` (issue.py).
///
/// Columnas/badges visibles por defecto en la vista de issues.
pub fn default_display_properties() -> Value {
    json!({
        "assignee": true,
        "attachment_count": true,
        "created_on": true,
        "due_date": true,
        "estimate": true,
        "key": true,
        "labels": true,
        "link": true,
        "priority": true,
        "start_date": true,
        "state": true,
        "sub_issue_count": true,
        "updated_on": true,
    })
}

/// Mirror de `get_default_props()` (project.py).
///
/// Estructura compuesta usada en `project_members.view_props` y
/// `project_members.default_props`: combina filtros base + filtros de
/// visualización en un solo objeto.
pub fn default_props() -> Value {
    json!({
        "filters": default_filters(),
        "display_filters": default_display_filters(),
    })
}

/// Mirror de `get_default_preferences()` (project.py).
///
/// Preferencias de UI a nivel proyecto. `pages.block_display=true` y
/// `navigation.default_tab="work_items"` son los valores que asume el frontend
/// si no recibe nada del backend.
pub fn default_preferences() -> Value {
    json!({
        "pages": { "block_display": true },
        "navigation": {
            "default_tab": "work_items",
            "hide_in_more_menu": [],
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_objects() {
        // Si alguno de estos no es objeto, ningún INSERT con jsonb NOT NULL
        // funcionaría — falla rápido en compile/test en lugar de en runtime.
        assert!(default_filters().is_object());
        assert!(default_display_filters().is_object());
        assert!(default_display_properties().is_object());
        assert!(default_props().is_object());
        assert!(default_preferences().is_object());
    }

    #[test]
    fn default_props_combines_filters_and_display_filters() {
        let v = default_props();
        assert!(v["filters"].is_object());
        assert!(v["display_filters"].is_object());
        // Sanity: order_by debe ser el default de Django.
        assert_eq!(v["display_filters"]["order_by"], "-created_at");
    }
}
