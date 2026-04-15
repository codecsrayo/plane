// src/jobs/email_notification.rs
//! Tarea periódica de envío de notificaciones por email.
//!
//! Equivalente a `plane/bgtasks/email_notification_task.py →
//! stack_email_notification + send_email_notification`.
//!
//! Flujo:
//!   1. Buscar `email_notification_logs` sin `processed_at`
//!   2. Agrupar por `receiver_id` → por `entity_identifier` (issue_id)
//!   3. Para cada (receiver, issue): construir HTML y enviar vía SMTP (lettre)
//!   4. Marcar `processed_at = NOW()` en los logs procesados
//!   5. Marcar `sent_at = NOW()` en los logs enviados exitosamente
//!
//! Redis Lock: se usa para evitar envíos duplicados en caso de concurrencia
//! (misma semántica que el lock de Django con nx=True).
//!
//! base_api: Django lo leía de Redis (set por issue_activities_task).
//! En Rust usamos `config.app_base_url` como fuente canónica.

use std::collections::HashMap;

use fred::prelude::{KeysInterface, Pool as RedisPool};
use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    config::Config,
    entities::{email_notification_logs, issues, projects, users, workspaces},
};

// ── Tipos internos ─────────────────────────────────────────────────────────────

/// Representa un cambio agrupado por actor dentro de un issue.
#[derive(Debug)]
struct ActorChange {
    actor_id: Uuid,
    field: Option<String>,
    old_value: Option<String>,
    new_value: Option<String>,
}

// ── Punto de entrada ──────────────────────────────────────────────────────────

/// Procesa todos los `email_notification_logs` sin `processed_at` y envía
/// los emails correspondientes.
///
/// Llamado cada 5 minutos por el scheduler (`cron.rs`).
pub async fn stack_email_notification(
    db: &DatabaseConnection,
    redis: &RedisPool,
    config: &Config,
) -> anyhow::Result<()> {
    // 1. Obtener notificaciones sin procesar, ordenadas por receiver
    let pending = email_notification_logs::Entity::find()
        .filter(email_notification_logs::Column::ProcessedAt.is_null())
        .filter(email_notification_logs::Column::DeletedAt.is_null())
        .order_by_asc(email_notification_logs::Column::ReceiverId)
        .all(db)
        .await?;

    if pending.is_empty() {
        return Ok(());
    }

    tracing::info!(count = pending.len(), "stack_email_notification: procesando");

    // 2. Agrupar: receiver_id → issue_id → Vec<log_id>
    let mut by_receiver: HashMap<Uuid, HashMap<Uuid, Vec<Uuid>>> = HashMap::new();
    let mut all_ids: Vec<Uuid> = Vec::new();

    for log in &pending {
        let issue_id = match log.entity_identifier {
            Some(id) => id,
            None => continue,
        };
        by_receiver
            .entry(log.receiver_id)
            .or_default()
            .entry(issue_id)
            .or_default()
            .push(log.id);
        all_ids.push(log.id);
    }

    // 3. Enviar emails por (receiver, issue)
    for (receiver_id, issues_map) in &by_receiver {
        for (issue_id, notif_ids) in issues_map {
            if let Err(e) = send_email_for_issue(
                db,
                redis,
                config,
                *receiver_id,
                *issue_id,
                notif_ids,
            )
            .await
            {
                tracing::warn!(
                    receiver_id = %receiver_id,
                    issue_id = %issue_id,
                    error = %e,
                    "stack_email_notification: fallo al enviar email"
                );
            }
        }
    }

    // 4. Marcar todos como processed_at = NOW()
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        &format!(
            "UPDATE email_notification_logs SET processed_at = $1 WHERE id = ANY($2::uuid[])"
        ),
        vec![
            now.into(),
            all_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
                .into(),
        ],
    );
    // Usamos UPDATE individual para compatibilidad con el tipo de parámetro
    for id in &all_ids {
        let s = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "UPDATE email_notification_logs SET processed_at = $1 WHERE id = $2",
            vec![now.into(), (*id).into()],
        );
        let _ = db.execute(s).await;
    }

    tracing::info!(
        processed = all_ids.len(),
        "stack_email_notification: completado"
    );
    Ok(())
}

// ── Envío individual por (receiver, issue) ────────────────────────────────────

async fn send_email_for_issue(
    db: &DatabaseConnection,
    redis: &RedisPool,
    config: &Config,
    receiver_id: Uuid,
    issue_id: Uuid,
    notif_ids: &[Uuid],
) -> anyhow::Result<()> {
    // Redis lock para evitar envíos duplicados
    let lock_key = format!("email_notif_lock:{issue_id}:{receiver_id}");
    let acquired: Option<String> = redis
        .set(
            &lock_key,
            "1",
            Some(fred::types::Expiration::EX(300)),
            Some(fred::types::SetOptions::NX),
            false,
        )
        .await?;

    if acquired.is_none() {
        tracing::debug!(%lock_key, "send_email_for_issue: lock ya tomado, omitiendo duplicado");
        return Ok(());
    }

    let _guard = RedisLockGuard {
        redis: redis.clone(),
        key: lock_key.clone(),
    };

    // Cargar receptor
    let receiver = users::Entity::find_by_id(receiver_id).one(db).await?;
    let receiver = match receiver {
        Some(u) => u,
        None => {
            tracing::warn!(%receiver_id, "send_email_for_issue: receptor no encontrado");
            return Ok(());
        }
    };
    let receiver_email = match &receiver.email {
        Some(e) if !e.is_empty() => e.clone(),
        _ => {
            tracing::debug!(%receiver_id, "send_email_for_issue: receptor sin email");
            return Ok(());
        }
    };

    // Cargar issue + proyecto + workspace
    let issue = issues::Entity::find_by_id(issue_id).one(db).await?;
    let issue = match issue {
        Some(i) => i,
        None => return Ok(()),
    };
    let project = projects::Entity::find_by_id(issue.project_id).one(db).await?;
    let project = match project {
        Some(p) => p,
        None => return Ok(()),
    };
    let workspace = workspaces::Entity::find_by_id(project.workspace_id).one(db).await?;
    let workspace = match workspace {
        Some(w) => w,
        None => return Ok(()),
    };

    // base_api desde config (equivale a leer Redis en Django)
    let base_url = config
        .app_base_url
        .as_deref()
        .unwrap_or("")
        .trim_end_matches('/');

    let issue_url = format!(
        "{base_url}/{slug}/projects/{project_id}/issues/{issue_id}",
        base_url = base_url,
        slug = workspace.slug,
        project_id = project.id,
        issue_id = issue.id,
    );
    let issue_identifier = format!("{}-{}", project.identifier, issue.sequence_id);

    // Obtener notificaciones de este grupo para extraer cambios
    let logs = email_notification_logs::Entity::find()
        .filter(email_notification_logs::Column::Id.is_in(notif_ids.to_vec()))
        .all(db)
        .await?;

    // Construir HTML del email
    let html = build_email_html(
        &issue.name,
        &issue_identifier,
        &issue_url,
        &receiver.first_name,
        &logs,
    );

    let subject = format!("{issue_identifier} {}", issue.name);

    // Enviar via SMTP
    send_smtp_email(config, &receiver_email, &subject, &html).await?;

    // Marcar sent_at en los logs enviados
    let sent_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    for id in notif_ids {
        let s = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "UPDATE email_notification_logs SET sent_at = $1 WHERE id = $2",
            vec![sent_at.into(), (*id).into()],
        );
        let _ = db.execute(s).await;
    }

    tracing::info!(
        %receiver_email,
        %issue_identifier,
        sent = notif_ids.len(),
        "send_email_for_issue: email enviado"
    );
    Ok(())
}

// ── SMTP transport ────────────────────────────────────────────────────────────

async fn send_smtp_email(
    config: &Config,
    to: &str,
    subject: &str,
    html: &str,
) -> anyhow::Result<()> {
    let host = match &config.email_host {
        Some(h) => h.clone(),
        None => {
            tracing::debug!("send_smtp_email: EMAIL_HOST no configurado, omitiendo envío");
            return Ok(());
        }
    };

    let from_addr = config.email_from.parse::<lettre::Address>()?;
    let to_addr = to.parse::<lettre::Address>()?;

    let email = Message::builder()
        .from(lettre::message::Mailbox::new(
            Some("Plane".to_string()),
            from_addr,
        ))
        .to(lettre::message::Mailbox::new(None, to_addr))
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(html_to_text(html)),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html.to_string()),
                ),
        )?;

    let mailer: AsyncSmtpTransport<Tokio1Executor> = if config.email_use_ssl {
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
            .port(config.email_port);
        if let (Some(user), Some(pass)) =
            (&config.email_host_user, &config.email_host_password)
        {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }
        builder.build()
    } else if config.email_use_tls {
        let mut builder =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)?.port(config.email_port);
        if let (Some(user), Some(pass)) =
            (&config.email_host_user, &config.email_host_password)
        {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }
        builder.build()
    } else {
        // Sin TLS — sólo para dev/testing
        let mut builder =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host).port(config.email_port);
        if let (Some(user), Some(pass)) =
            (&config.email_host_user, &config.email_host_password)
        {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }
        builder.build()
    };

    mailer.send(email).await?;
    Ok(())
}

// ── HTML builder ──────────────────────────────────────────────────────────────

fn build_email_html(
    issue_name: &str,
    issue_identifier: &str,
    issue_url: &str,
    receiver_name: &str,
    logs: &[email_notification_logs::Model],
) -> String {
    let changes: String = logs
        .iter()
        .filter_map(|log| {
            let field = log.entity_name.as_str();
            let old = log.old_value.as_deref().unwrap_or("");
            let new = log.new_value.as_deref().unwrap_or("");
            if old.is_empty() && new.is_empty() {
                return None;
            }
            Some(format!(
                "<li><strong>{field}</strong>: {old} → {new}</li>",
                old = html_escape(old),
                new = html_escape(new),
            ))
        })
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8" /></head>
<body style="font-family: sans-serif; color: #333; max-width: 600px; margin: auto;">
  <p>Hola {receiver_name},</p>
  <p>Se realizaron actualizaciones en el issue
     <a href="{issue_url}"><strong>{issue_identifier} — {issue_name}</strong></a>:
  </p>
  <ul>{changes}</ul>
  <p>
    <a href="{issue_url}" style="
      background:#5b55f6;color:#fff;padding:8px 16px;
      border-radius:4px;text-decoration:none;display:inline-block">
      Ver issue
    </a>
  </p>
  <hr />
  <p style="font-size:12px;color:#888;">
    Recibes este email porque estás suscrito a este issue en Plane.
  </p>
</body>
</html>"#,
        receiver_name = html_escape(receiver_name),
        issue_identifier = html_escape(issue_identifier),
        issue_name = html_escape(issue_name),
        issue_url = issue_url,
        changes = changes,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_to_text(html: &str) -> String {
    // Eliminación simple de tags HTML para el part text/plain
    let re = regex::Regex::new(r"<[^>]+>").expect("regex válido");
    let text = re.replace_all(html, " ");
    // Colapsar espacios múltiples
    let re2 = regex::Regex::new(r"\s{2,}").expect("regex válido");
    re2.replace_all(&text, "\n").trim().to_string()
}

// ── Redis lock guard (RAII) ───────────────────────────────────────────────────

/// Libera el lock de Redis al salir del scope.
struct RedisLockGuard {
    redis: RedisPool,
    key: String,
}

impl Drop for RedisLockGuard {
    fn drop(&mut self) {
        let redis = self.redis.clone();
        let key = self.key.clone();
        tokio::spawn(async move {
            let _: Result<(), _> = redis.del::<(), _>(&key).await;
        });
    }
}
