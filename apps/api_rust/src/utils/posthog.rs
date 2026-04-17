// src/utils/posthog.rs
//! Cliente mínimo de PostHog para eventos de producto.
//!
//! Espejo funcional de `apps/api/plane/bgtasks/event_tracking_task.py`:
//!
//! - Django resuelve `POSTHOG_API_KEY` / `POSTHOG_HOST` vía
//!   `get_configuration_value` (DB → env). Replicamos con
//!   `utils::instance_config::get_config_value`.
//! - Si falta alguna de las dos claves, Django escribe
//!   `"Event tracking is not configured"` y retorna. Hacemos el mismo warn y
//!   salimos sin error.
//! - Django encola el envío vía Celery (`@shared_task`). En Rust replicamos el
//!   fire-and-forget con `tokio::spawn`: el handler HTTP responde sin esperar
//!   al capture. Errores del POST se loguean con `tracing`, nunca se propagan
//!   al cliente — igual que Django (`log_exception(e); return False`).
//!
//! El handler `/capture/` de PostHog acepta payload JSON:
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
//! El cliente oficial de Python envía los `groups` bajo `properties["$groups"]`;
//! replicamos ese shape aquí para que los eventos se agrupen igual en PostHog.

use std::time::Duration;

use serde_json::{json, Value};
use uuid::Uuid;

use crate::{utils::instance_config::get_config_value, AppState};

// Event names — deben coincidir exactamente con
// `apps/api/plane/utils/analytics_events.py`.
pub const EVENT_WORKSPACE_DELETED: &str = "workspace_deleted";

/// Timeout máximo del POST a PostHog. No bloquea al usuario (fire-and-forget),
/// pero evita que tareas colgadas se acumulen indefinidamente si el host está
/// caído.
const POSTHOG_TIMEOUT: Duration = Duration::from_secs(5);

/// Dispara un evento a PostHog de forma fire-and-forget.
///
/// - `distinct_id`: user id que originó el evento (equivalente al `user_id`
///   posicional de Django).
/// - `event_name`: nombre del evento (ver constantes del módulo).
/// - `workspace_slug`: usado para el grouping `$groups.workspace` de PostHog.
/// - `properties`: payload arbitrario del evento. Se fusionará con `$groups`.
///
/// Si PostHog no está configurado, emite un warn de una línea y retorna.
/// Nunca propaga errores — los errores de red se loguean y se descartan,
/// igual que `event_tracking_task.track_event` en Django.
pub fn track_event(
    state: &AppState,
    distinct_id: Uuid,
    event_name: &'static str,
    workspace_slug: String,
    mut properties: serde_json::Map<String, Value>,
) {
    // Clonamos solo lo necesario para no mover `state` al task.
    let http = state.http.clone();
    let app_state = state.clone();
    let event = event_name.to_owned();

    tokio::spawn(async move {
        // 1. Resolver configuración.
        // Replicamos `posthogConfiguration()`: DB tiene prioridad sobre env.
        // `get_config_value` acepta un env fallback; lo pasamos explícito.
        let api_key = match get_config_value(
            &app_state,
            "POSTHOG_API_KEY",
            std::env::var("POSTHOG_API_KEY").ok().as_deref(),
        )
        .await
        {
            Ok(Some(v)) if !v.is_empty() => v,
            _ => {
                // Paridad exacta con Django:
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

        // 2. Inyectar `$groups.workspace` en properties — el cliente oficial
        //    de Python hace esto cuando se pasa `groups={"workspace": slug}`.
        properties.insert(
            "$groups".to_string(),
            json!({ "workspace": workspace_slug }),
        );

        // 3. POST a {host}/capture/ — sin trailing slash normalization ya que
        //    PostHog es estricto respecto al path.
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
                    // No propagamos; solo loguea. Igual que `log_exception`.
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
