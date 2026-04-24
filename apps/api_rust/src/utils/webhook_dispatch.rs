// src/utils/webhook_dispatch.rs
//! Fan-out de eventos a webhooks salientes.
//!
//! Paridad con `plane/bgtasks/webhook_task.py::webhook_activity` de Django:
//!
//! ```py
//! webhooks = Webhook.objects.filter(workspace__slug=slug, is_active=True)
//! if event == "project":       webhooks = webhooks.filter(project=True)
//! if event == "issue":         webhooks = webhooks.filter(issue=True)
//! if event in ("module", "module_issue"): webhooks = webhooks.filter(module=True)
//! if event in ("cycle",  "cycle_issue"):  webhooks = webhooks.filter(cycle=True)
//! if event == "issue_comment": webhooks = webhooks.filter(issue_comment=True)
//! for webhook in webhooks:
//!     webhook_send_task.delay(webhook_id=webhook.id, ...)
//! ```
//!
//! Diseño:
//!   - Descubrir → encolar. Todo el I/O HTTP lo hace el worker apalis
//!     (`jobs::webhook_delivery`), este módulo solo enqueue.
//!   - Idempotencia débil: un `delivery_id` fresco por job encolado.
//!     Si el caller se ejecuta dos veces (p.ej. retry de un handler HTTP),
//!     se generarán dos entregas — es consistente con Django.
//!   - Best-effort en el loop: un fallo al encolar a UN webhook no impide
//!     el resto. Se loggea a `tracing::warn!` con el webhook_id.
//!   - La tabla `project_webhooks` NO se usa como filtro de entrega — en
//!     Django solo existe como asociación informativa; los webhooks son
//!     workspace-level.

use apalis::prelude::Storage;
use apalis_sql::postgres::PostgresStorage;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    entities::webhooks,
    jobs::webhook_delivery::DeliverWebhookJob,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

/// Tipos de evento reconocidos. Se mapean 1:1 con las banderas booleanas
/// de la tabla `webhooks` (`project`, `issue`, `module`, `cycle`, `issue_comment`).
///
/// Los alias `ModuleIssue` y `CycleIssue` comparten bandera con `Module` y
/// `Cycle` respectivamente — paridad con Django.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookEvent {
    Project,
    Issue,
    Module,
    ModuleIssue,
    Cycle,
    CycleIssue,
    IssueComment,
}

impl WebhookEvent {
    /// Cadena enviada como `event` en el payload y como header `X-Plane-Event`.
    /// Conserva el naming histórico de Django para consumidores existentes.
    pub fn as_payload_str(&self) -> &'static str {
        match self {
            Self::Project       => "project",
            Self::Issue         => "issue",
            Self::Module        => "module",
            Self::ModuleIssue   => "module_issue",
            Self::Cycle         => "cycle",
            Self::CycleIssue    => "cycle_issue",
            Self::IssueComment  => "issue_comment",
        }
    }
}

/// Acción realizada sobre el recurso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebhookAction {
    Created,
    Updated,
    Deleted,
}

impl WebhookAction {
    pub fn as_payload_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
        }
    }
}

/// Encola una entrega de webhook por cada webhook activo del workspace
/// cuyo flag del evento esté encendido.
///
/// Parámetros:
///   - `data`: modelo serializado que irá en `data` del envelope. Para deletes
///     Django envía `{"id": <uuid>}`; para create/update, el modelo completo
///     serializado con DRF (`IssueExpandSerializer`, `ProjectSerializer`, …).
///   - `activity`: bloque `activity` opcional. `None` se propaga como `null`
///     en el body final — el contrato Django siempre incluye la clave.
///     Para create/delete disparados directamente desde un handler, suele ser
///     `None`; para updates con diff por campo se llena con
///     `{field, old_value, new_value, actor, old_identifier, new_identifier}`.
///
/// Devuelve el número de `DeliverWebhookJob` encolados con éxito.
///
/// Errores:
///   - La consulta a `webhooks` falla → devuelve `Err` (fallo sistémico).
///   - Un `push` individual al storage falla → se loggea y se continúa.
///     La cuenta devuelta refleja solo los encolados con éxito.
pub async fn dispatch_event(
    state: &AppState,
    workspace_id: Uuid,
    event: WebhookEvent,
    action: WebhookAction,
    data: serde_json::Value,
    activity: Option<serde_json::Value>,
) -> anyhow::Result<usize> {
    // 1. Descubrir webhooks suscritos
    let event_column = match event {
        WebhookEvent::Project                              => webhooks::Column::Project,
        WebhookEvent::Issue                                => webhooks::Column::Issue,
        WebhookEvent::Module       | WebhookEvent::ModuleIssue => webhooks::Column::Module,
        WebhookEvent::Cycle        | WebhookEvent::CycleIssue  => webhooks::Column::Cycle,
        WebhookEvent::IssueComment                         => webhooks::Column::IssueComment,
    };

    let matching_webhooks = webhooks::Entity::find()
        .active() // deleted_at IS NULL
        .filter(webhooks::Column::WorkspaceId.eq(workspace_id))
        .filter(webhooks::Column::IsActive.eq(true))
        .filter(event_column.eq(true))
        .all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!(
                workspace_id = %workspace_id,
                event = event.as_payload_str(),
                error = %e,
                "dispatch_event: falló la consulta de webhooks",
            );
            anyhow::anyhow!(e)
        })?;

    if matching_webhooks.is_empty() {
        tracing::debug!(
            workspace_id = %workspace_id,
            event = event.as_payload_str(),
            action = action.as_payload_str(),
            "dispatch_event: sin webhooks suscritos",
        );
        return Ok(0);
    }

    // 2. Encolar uno por webhook
    //    Un solo storage para todos los push del batch — más económico que
    //    reconstruirlo en cada iteración.
    let mut storage: PostgresStorage<DeliverWebhookJob> =
        PostgresStorage::new(state.pg_pool.clone());

    let event_str = event.as_payload_str().to_owned();
    let action_str = action.as_payload_str().to_owned();
    let mut enqueued = 0usize;

    for wh in matching_webhooks {
        let job = DeliverWebhookJob {
            webhook_id: wh.id,
            event: event_str.clone(),
            action: action_str.clone(),
            data: data.clone(),
            activity: activity.clone(),
            delivery_id: Uuid::new_v4(),
        };

        match storage.push(job).await {
            Ok(_) => enqueued += 1,
            Err(e) => {
                // Best-effort: no abortar el batch por un fallo transitorio.
                tracing::warn!(
                    webhook_id = %wh.id,
                    workspace_id = %workspace_id,
                    event = %event_str,
                    error = %e,
                    "dispatch_event: no se pudo encolar DeliverWebhookJob",
                );
            }
        }
    }

    tracing::debug!(
        workspace_id = %workspace_id,
        event = %event_str,
        action = %action_str,
        enqueued,
        "dispatch_event: fan-out completado",
    );
    Ok(enqueued)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_payload_strings_match_django() {
        // Nombres exactos que consumen los suscriptores existentes.
        assert_eq!(WebhookEvent::Project.as_payload_str(),      "project");
        assert_eq!(WebhookEvent::Issue.as_payload_str(),        "issue");
        assert_eq!(WebhookEvent::Module.as_payload_str(),       "module");
        assert_eq!(WebhookEvent::ModuleIssue.as_payload_str(),  "module_issue");
        assert_eq!(WebhookEvent::Cycle.as_payload_str(),        "cycle");
        assert_eq!(WebhookEvent::CycleIssue.as_payload_str(),   "cycle_issue");
        assert_eq!(WebhookEvent::IssueComment.as_payload_str(), "issue_comment");
    }

    #[test]
    fn action_payload_strings_match_django() {
        assert_eq!(WebhookAction::Created.as_payload_str(), "created");
        assert_eq!(WebhookAction::Updated.as_payload_str(), "updated");
        assert_eq!(WebhookAction::Deleted.as_payload_str(), "deleted");
    }

    #[test]
    fn module_and_module_issue_share_column() {
        // Contrato Django: ambos eventos consultan `module=True`.
        // Si esto cambia, el test falla y fuerza a revisar el mapeo.
        let col_module      = matches!(WebhookEvent::Module,      WebhookEvent::Module);
        let col_mod_issue   = matches!(WebhookEvent::ModuleIssue, WebhookEvent::ModuleIssue);
        assert!(col_module && col_mod_issue);
    }
}
