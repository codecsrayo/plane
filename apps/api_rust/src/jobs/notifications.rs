// src/jobs/notifications.rs
//! Job: creación de notificaciones in-app para actividades de issues.
//!
//! Equivalente a `plane/bgtasks/notification_task.py`.
//!
//! Flujo:
//!   1. Recibir IssueActivityNotificationJob con el ID de actividad
//!   2. Determinar destinatarios: assignees + subscribers + creador + mencionados
//!   3. Insertar registros en la tabla `notifications` (sin duplicados)
//!
//! No envía emails en este job — el envío de emails es responsabilidad
//! de un job separado que lee `email_notification_logs`.

use apalis::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{
        issue_activities, issue_assignees, issue_subscribers, issues, notifications,
        project_members,
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Job payload ───────────────────────────────────────────────────────────────

/// Payload del job: ID de la actividad que disparó la notificación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueActivityNotificationJob {
    pub activity_id: Uuid,
}


// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn handle_issue_activity_notification(
    job: IssueActivityNotificationJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state = ctx.0.clone();

    if let Err(e) = run_notification(&state, job.activity_id).await {
        tracing::error!(
            activity_id = %job.activity_id,
            error = %e,
            "issue_activity_notification: job falló"
        );
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

async fn run_notification(state: &AppState, activity_id: Uuid) -> anyhow::Result<()> {
    use anyhow::Context as _;

    // 1. Cargar la actividad
    let activity = issue_activities::Entity::find_by_id(activity_id)
        .one(&state.db)
        .await?
        .context("IssueActivity no encontrada")?;

    let issue_id = match activity.issue_id {
        Some(id) => id,
        None => return Ok(()), // actividad sin issue asociado — ignorar
    };

    let actor_id = activity.actor_id;

    // 2. Cargar el issue para contexto
    let issue = issues::Entity::find_by_id(issue_id)
        .active()
        .one(&state.db)
        .await?
        .context("Issue no encontrado")?;

    // 3. Recolectar destinatarios únicos (excluyendo al actor)
    let mut receiver_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

    // Creador del issue
    if let Some(creator) = issue.created_by_id {
        receiver_ids.insert(creator);
    }

    // Assignees activos
    let assignees = issue_assignees::Entity::find()
        .active()
        .filter(issue_assignees::Column::IssueId.eq(issue_id))
        .all(&state.db)
        .await?;
    for a in assignees {
        receiver_ids.insert(a.assignee_id);
    }

    // Subscribers activos
    let subscribers = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .all(&state.db)
        .await?;
    for s in subscribers {
        receiver_ids.insert(s.subscriber_id);
    }

    // Excluir al actor — no se notifica a uno mismo
    if let Some(actor) = actor_id {
        receiver_ids.remove(&actor);
    }

    if receiver_ids.is_empty() {
        return Ok(());
    }

    // 4. Verificar que los destinatarios son miembros activos del proyecto
    let active_members: std::collections::HashSet<Uuid> = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(issue.project_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(&state.db)
        .await?
        .into_iter()
        .filter_map(|m| m.member_id)
        .collect();

    let valid_receivers: Vec<Uuid> = receiver_ids
        .into_iter()
        .filter(|id| active_members.contains(id))
        .collect();

    // 5. Construir el título de la notificación
    let title = build_notification_title(&activity, &issue.name);
    let message_html = format!(
        "<p>{}</p>",
        ammonia::clean(&title)
    );

    // 6. Insertar notificaciones (ignorar duplicados: misma actividad + receiver)
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let triggered_by = activity.actor_id;

    for receiver_id in valid_receivers {
        // Comprobar si ya existe una notificación para esta actividad + receptor
        // para garantizar idempotencia en caso de re-ejecución del job.
        let already_exists = notifications::Entity::find()
            .filter(notifications::Column::ReceiverId.eq(receiver_id))
            .filter(notifications::Column::EntityIdentifier.eq(activity_id))
            .filter(notifications::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?;

        if already_exists.is_some() {
            continue;
        }

        notifications::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title.clone()),
            message_html: Set(message_html.clone()),
            message_stripped: Set(Some(title.clone())),
            entity_identifier: Set(Some(activity_id)),
            entity_name: Set("issue_activity".to_owned()),
            sender: Set("in_app".to_owned()),
            receiver_id: Set(receiver_id),
            triggered_by_id: Set(triggered_by),
            project_id: Set(Some(issue.project_id)),
            workspace_id: Set(issue.workspace_id),
            created_by_id: Set(triggered_by),
            updated_by_id: Set(triggered_by),
            data: Set(Some(serde_json::json!({
                "issue_id": issue_id,
                "issue_name": issue.name,
                "activity_id": activity_id,
                "field": activity.field,
                "old_value": activity.old_value,
                "new_value": activity.new_value,
            }))),
            message: Set(None),
            read_at: Set(None),
            snoozed_till: Set(None),
            archived_at: Set(None),
            deleted_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&state.db)
        .await?;
    }

    tracing::debug!(
        activity_id = %activity_id,
        issue_id = %issue_id,
        "issue_activity_notification: notificaciones creadas"
    );

    Ok(())
}

/// Construye el título legible de la notificación según el campo modificado.
fn build_notification_title(activity: &issue_activities::Model, issue_name: &str) -> String {
    let field = activity.field.as_deref().unwrap_or("unknown");
    let new_val = activity.new_value.as_deref().unwrap_or("");

    match field {
        "state" => format!("Estado actualizado a «{new_val}» en «{issue_name}»"),
        "assignees" => format!("Asignados cambiados en «{issue_name}»"),
        "priority" => format!("Prioridad cambiada a «{new_val}» en «{issue_name}»"),
        "comment" => format!("Nuevo comentario en «{issue_name}»"),
        "name" => format!("Título actualizado a «{new_val}»"),
        "description" => format!("Descripción actualizada en «{issue_name}»"),
        "target_date" => format!("Fecha límite actualizada en «{issue_name}»"),
        "cycle" => format!("Issue movido a ciclo «{new_val}»"),
        "module" => format!("Issue movido al módulo «{new_val}»"),
        _ => format!("Actualización en «{issue_name}»"),
    }
}
