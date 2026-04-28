// src/jobs/email_notification.rs
//! Periodic task for sending email notifications.
//!
//! Equivalent to `plane/bgtasks/email_notification_task.py →
//! stack_email_notification + send_email_notification`.
//!
//! Flow:
//!   1. Find `email_notification_logs` without `processed_at`
//!   2. Group by `receiver_id` → by `entity_identifier` (issue_id)
//!   3. For each (receiver, issue): build HTML and send via SMTP (lettre)
//!   4. Mark `processed_at = NOW()` on processed logs
//!   5. Mark `sent_at = NOW()` on successfully sent logs
//!
//! Redis Lock: used to avoid duplicate sends in case of concurrency
//! (same semantics as Django lock with nx=True).
//!
//! base_api: Django read it from Redis (set by issue_activities_task).
//! In Rust we use `config.app_base_url` as canonical source.

use std::collections::HashMap;

use fred::prelude::{KeysInterface, Pool as RedisPool};
use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    config::Config,
    entities::{email_notification_logs, issues, projects, users, workspaces},
};

// ── Entry point ──────────────────────────────────────────────────────────

/// Processes all `email_notification_logs` without `processed_at` and sends
/// corresponding emails.
///
/// Called every 5 minutes by the scheduler (`cron.rs`).
pub async fn stack_email_notification(
    db: &DatabaseConnection,
    redis: &RedisPool,
    config: &Config,
) -> anyhow::Result<()> {
    // 1. Get unprocessed notifications, ordered by receiver
    let pending = email_notification_logs::Entity::find()
        .filter(email_notification_logs::Column::ProcessedAt.is_null())
        .filter(email_notification_logs::Column::DeletedAt.is_null())
        .order_by_asc(email_notification_logs::Column::ReceiverId)
        .all(db)
        .await?;

    if pending.is_empty() {
        return Ok(());
    }

    tracing::info!(count = pending.len(), "stack_email_notification: processing");

    // 2. Group: receiver_id → issue_id → Vec<log_id>
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

    // 3. Send emails by (receiver, issue)
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
                    "stack_email_notification: failed to send email"
                );
            }
        }
    }

    // 4. Mark all as processed_at = NOW() (individual UPDATE for type compatibility)
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
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
        "stack_email_notification: completed"
    );
    Ok(())
}

// ── Individual send by (receiver, issue) ────────────────────────────────────

async fn send_email_for_issue(
    db: &DatabaseConnection,
    redis: &RedisPool,
    config: &Config,
    receiver_id: Uuid,
    issue_id: Uuid,
    notif_ids: &[Uuid],
) -> anyhow::Result<()> {
    // Redis lock to avoid duplicate sends
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
        tracing::debug!(%lock_key, "send_email_for_issue: lock already taken, skipping duplicate");
        return Ok(());
    }

    let _guard = RedisLockGuard {
        redis: redis.clone(),
        key: lock_key.clone(),
    };

    // Load receiver
    let receiver = users::Entity::find_by_id(receiver_id).one(db).await?;
    let receiver = match receiver {
        Some(u) => u,
        None => {
            tracing::warn!(%receiver_id, "send_email_for_issue: receiver not found");
            return Ok(());
        }
    };
    let receiver_email = match &receiver.email {
        Some(e) if !e.is_empty() => e.clone(),
        _ => {
            tracing::debug!(%receiver_id, "send_email_for_issue: receiver has no email");
            return Ok(());
        }
    };

    // Load issue + project + workspace
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

    // base_api from config (equivalent to reading Redis in Django)
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

    // Get notifications of this group to extract changes
    let logs = email_notification_logs::Entity::find()
        .filter(email_notification_logs::Column::Id.is_in(notif_ids.to_vec()))
        .all(db)
        .await?;

    // Build email HTML
    let html = build_email_html(
        &issue.name,
        &issue_identifier,
        &issue_url,
        &receiver.first_name,
        &logs,
    );

    let subject = format!("{issue_identifier} {}", issue.name);

    // Send via SMTP
    send_smtp_email(config, &receiver_email, &subject, &html).await?;

    // Mark sent_at on sent logs
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
        "send_email_for_issue: email sent"
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
            tracing::debug!("send_smtp_email: EMAIL_HOST not configured, skipping send");
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
        // No TLS — only for dev/testing
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
  <p>Hi {receiver_name},</p>
  <p>Updates were made to issue
     <a href="{issue_url}"><strong>{issue_identifier} — {issue_name}</strong></a>:
  </p>
  <ul>{changes}</ul>
  <p>
    <a href="{issue_url}" style="
      background:#5b55f6;color:#fff;padding:8px 16px;
      border-radius:4px;text-decoration:none;display:inline-block">
      View issue
    </a>
  </p>
  <hr />
  <p style="font-size:12px;color:#888;">
    You are receiving this email because you are subscribed to this issue in Plane.
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
    // Simple removal of HTML tags for the text/plain part
    let re = regex::Regex::new(r"<[^>]+>").expect("valid regex");
    let text = re.replace_all(html, " ");
    // Collapse multiple spaces
    let re2 = regex::Regex::new(r"\s{2,}").expect("valid regex");
    re2.replace_all(&text, "\n").trim().to_string()
}

// ── Redis lock guard (RAII) ───────────────────────────────────────────────────

/// Releases Redis lock on scope exit.
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
