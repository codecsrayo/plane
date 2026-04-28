// src/utils/posthog.rs
//! Minimal PostHog client for product events.
//!
//! Functional mirror of `apps/api/plane/bgtasks/event_tracking_task.py`:
//!
//! - Django resolves `POSTHOG_API_KEY` / `POSTHOG_HOST` via
//!   `get_configuration_value` (DB → env). We replicate with
//!   `utils::instance_config::get_config_value`.
//! - If any of the two keys is missing, Django writes
//!   `"Event tracking is not configured"` and returns. We do the same warning and
//!   exit without error.
//! - Django enqueues the sending via Celery (`@shared_task`). In Rust we replicate the
//!   fire-and-forget with `tokio::spawn`: the HTTP handler responds without waiting
//!   for the capture. POST errors are logged with `tracing`, never propagated
//!   to the client — same as Django (`log_exception(e); return False`).
//!
//! The PostHog `/capture/` handler accepts JSON payload:
//!
//! ```json
//! {
//!   "api_key": "<key>",
//!   "event":   "workspace_deleted",
//!   "distinct_id": "<user_id>",
//!   "properties": {
//!     "$groups": {"workspace": "<slug>"},
//!     "user_id": "...",
//!     "workspace_id": "...",
//!     ...
//!   }
//! }
//! ```
//!
//! The official Python client sends `groups` under `properties["$groups"]`;
//! we replicate that shape here so that events are grouped the same in PostHog.

use std::time::Duration;

use serde_json::{json, Value};
use uuid::Uuid;

use crate::{utils::instance_config::get_config_value, AppState};

// Event names — deben coincidir exactamente con
// `apps/api/plane/utils/analytics_events.py`.
pub const EVENT_WORKSPACE_DELETED: &str = "workspace_deleted";

/// Maximum timeout of the POST to PostHog. Does not block the user (fire-and-forget),
/// but prevents hung tasks from accumulating indefinitely if the host is
/// down.
const POSTHOG_TIMEOUT: Duration = Duration::from_secs(5);

/// Triggers a PostHog event in a fire-and-forget manner.
///
/// - `distinct_id`: user id that originated the event (equivalent to Django's
///   positional `user_id`).
/// - `event_name`: event name (see module constants).
/// - `workspace_slug`: used for PostHog's `$groups.workspace` grouping.
/// - `properties`: arbitrary event payload. Will be merged with `$groups`.
///
/// If PostHog is not configured, it emits a one-line warning and returns.
/// Never propagates errors — network errors are logged and discarded,
/// same as Django's `event_tracking_task.track_event`.
pub fn track_event(
    state: &AppState,
    distinct_id: Uuid,
    event_name: &'static str,
    workspace_slug: String,
    mut properties: serde_json::Map<String, Value>,
) {
    // We clone only what is necessary not to move `state` to the task.
    let http = state.http.clone();
    let app_state = state.clone();
    let event = event_name.to_owned();

    tokio::spawn(async move {
        // 1. Resolve configuration.
        // We replicate `posthogConfiguration()`: DB has priority over env.
        // `get_config_value` accepts an env fallback; we pass it explicitly.
        let api_key = match get_config_value(
            &app_state,
            "POSTHOG_API_KEY",
            std::env::var("POSTHOG_API_KEY").ok().as_deref(),
        )
        .await
        {
            Ok(Some(v)) if !v.is_empty() => v,
            _ => {
                // Exact parity with Django:
                //   logger.warning("Event tracking is not configured")
                //   return
                tracing::warn!(
                    event = %event,
                    "posthog: event tracking is not configured (missing POSTHOG_API_KEY)"
                );
                return;
            }
        };

        let host = match get_config_value(
            &app_state,
            "POSTHOG_HOST",
            std::env::var("POSTHOG_HOST").ok().as_deref(),
        )
        .await
        {
            Ok(Some(v)) if !v.is_empty() => v,
            _ => {
                tracing::warn!(
                    event = %event,
                    "posthog: event tracking is not configured (missing POSTHOG_HOST)"
                );
                return;
            }
        };

        // 2. Inject `$groups.workspace` into properties — the official Python
        //    client does this when `groups={"workspace": slug}` is passed.
        properties.insert(
            "$groups".to_string(),
            json!({ "workspace": workspace_slug }),
        );

        // 3. POST to {host}/capture/ — without trailing slash normalization since
        //    PostHog is strict regarding the path.
        let url = format!("{}/capture/", host.trim_end_matches('/'));
        let payload = json!({
            "api_key":     api_key,
            "event":       event,
            "distinct_id": distinct_id.to_string(),
            "properties":  properties,
        });

        let res = http
            .post(&url)
            .timeout(POSTHOG_TIMEOUT)
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    // We do not propagate; only log. Same as `log_exception`.
                    let body = resp.text().await.unwrap_or_default();
                    tracing::warn!(
                        event = %event,
                        status = %status,
                        body = %body,
                        "posthog: capture returned non-2xx"
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    event = %event,
                    error = %err,
                    "posthog: capture request failed"
                );
            }
        }
    });
}
