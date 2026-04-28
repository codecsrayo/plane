// src/utils/django_defaults.rs
//! Canonical Django JSON defaults for `jsonb NOT NULL` columns.
//!
//! Django defines these values in `models.JSONField(default=...)` at the Python
//! level — the DB does NOT have corresponding DEFAULT clauses (migrations only
//! apply the default at INSERT time via Python). That's why Rust handlers
//! must provide the same JSON explicitly when inserting.
//!
//! Canonical source:
//! - `apps/api/plane/db/models/project.py` → `get_default_props`,
//!   `get_default_preferences`
//! - `apps/api/plane/db/models/issue.py` → `get_default_filters`,
//!   `get_default_display_filters`, `get_default_display_properties`
//!
//! These defaults are used when creating `project_members`,
//! `project_user_properties`, `cycle_user_properties`, `module_user_properties`,
//! `workspace_members` and other tables that inherit from `IssueProperty`.

use serde_json::{json, Value};

/// Mirror of `get_default_filters()` (issue.py).
///
/// Structure used in `*UserProperty.filters`. All slots start at
/// `null` so the frontend shows all issues without filtering.
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

/// Mirror of `get_default_display_filters()` (issue.py).
///
/// Default display configuration: list, ordered by creation date
/// descending, showing sub-issues and empty groups.
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

/// Mirror of `get_default_display_properties()` (issue.py).
///
/// Columns/badges visible by default in the issues view.
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

/// Mirror of `get_default_props()` (project.py).
///
/// Composite structure used in `project_members.view_props` and
/// `project_members.default_props`: combines base filters + display
/// filters into a single object.
pub fn default_props() -> Value {
    json!({
        "filters": default_filters(),
        "display_filters": default_display_filters(),
    })
}

/// Mirror of `get_default_preferences()` (project.py).
///
/// Project-level UI preferences. `pages.block_display=true` and
/// `navigation.default_tab="work_items"` are the values assumed by the frontend
/// if it receives nothing from the backend.
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
        // If any of these is not an object, no INSERT with jsonb NOT NULL
        // would work — fails fast in compile/test instead of runtime.
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
        // Sanity: order_by must be the Django default.
        assert_eq!(v["display_filters"]["order_by"], "-created_at");
    }
}
