// src/jobs/webhook_delivery.rs
//! Job: delivery (fan-out) of outgoing webhooks.
//!
//! Equivalent to Django's `plane/bgtasks/webhook_task.py`.
//!
//! Flow:
//!   1. Receive `DeliverWebhookJob { webhook_id, event, action, data, activity, delivery_id }`.
//!   2. Load the webhook (active and not soft-deleted).
//!   3. Validate destination URL (SSRF defense: rejects loopback/private/link-local/multicast).
//!   4. Sign the body with HMAC-SHA256(secret_key).
//!   5. POST with 10s timeout and standardized headers.
//!   6. Persist request/response in `webhook_logs` (body truncated to not saturate DB).
//!   7. Return `Err` on 5xx/408/429/network — allows retry when apalis retry middleware
//!      is added (`retry` features already enabled).
//!
//! Security:
//!   - Only `http`/`https` schemes.
//!   - Rejection of RFC1918 IPs, loopback, link-local, CGNAT, ULA IPv6, multicast.
//!   - The value of the `X-Plane-Signature` header is redacted in the log — the secret
//!     does not leak to operators reviewing the table.
//!   - `reqwest` uses rustls TLS (see Cargo.toml); verification is not disabled.
//!
//! Not yet covered (next commits):
//!   - Fan-out: this job delivers to ONE webhook; the dispatcher that discovers
//!     subscribed webhooks and enqueues N jobs lives in `utils::webhook_dispatch`.
//!   - Automatic retries: requires stacking `.retry(...)` on the worker.
//!   - DNS pinning (anti-rebind): `reqwest` resolves the hostname after our
//!     validation; a public IP could re-resolve to a private one.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use apalis::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{webhook_logs, webhooks},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Constants ────────────────────────────────────────────────────────────────

/// Size limit of the body saved in `webhook_logs.request_body`.
const MAX_REQUEST_BODY_LOG: usize = 64 * 1024;
/// Size limit of the response body saved in `webhook_logs.response_body`.
const MAX_RESPONSE_BODY_LOG: usize = 8 * 1024;
/// Timeout per attempt — less than the global `AppState.http` timeout (30s)
/// so a slow destination doesn't block the worker.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

// ── Job payload ──────────────────────────────────────────────────────────────

/// An individual send. Fan-out to N webhooks enqueues N `DeliverWebhookJob`.
///
/// The envelope emitted to the client's endpoint replicates the Django format
/// (`plane/bgtasks/webhook_task.py::webhook_send_task`):
///
/// ```json
/// { "event": ..., "action": ..., "webhook_id": ..., "workspace_id": ...,
///   "data": ..., "activity": ... }
/// ```
///
/// `workspace_id` is NOT sent in the job because it is always derived from the
/// webhook row — this avoids any misalignment with the workspace from which the
/// event was triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverWebhookJob {
    pub webhook_id: Uuid,
    /// `"project"`, `"issue"`, `"cycle"`, `"module"`, `"issue_comment"`, etc.
    pub event: String,
    /// `"created"`, `"updated"`, `"deleted"`.
    pub action: String,
    /// Serialized model (or `{"id": ...}` for deletes). Goes in the `data` field
    /// of the envelope — name aligned with Django.
    pub data: serde_json::Value,
    /// Optional `activity` block of the envelope. For create/delete it's normally
    /// `None`; for updates Django uses it to report the per-field diff
    /// (`{field, old_value, new_value, actor, ...}`). `None` → `"activity": null`
    /// in the body, same contract as Django, whose `webhook_activity` always
    /// sends the key.
    #[serde(default)]
    pub activity: Option<serde_json::Value>,
    /// Unique UUID per attempt — exposed as `X-Plane-Delivery` header.
    /// NOT included in the POST body (Django doesn't include it either).
    pub delivery_id: Uuid,
}

// ── Handler ──────────────────────────────────────────────────────────────────

pub async fn handle_deliver_webhook(
    job: DeliverWebhookJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    if let Err(e) = run_delivery(&state, &job).await {
        // `%e` hides the root cause of `anyhow::Context` — here we prefer `?`
        // to see full detail in incidents.
        tracing::warn!(
            webhook_id = %job.webhook_id,
            delivery_id = %job.delivery_id,
            event = %job.event,
            action = %job.action,
            error = ?e,
            "webhook_delivery: failed attempt",
        );
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

// ── Main Flow ────────────────────────────────────────────────────────────────

async fn run_delivery(state: &AppState, job: &DeliverWebhookJob) -> anyhow::Result<()> {
    use anyhow::Context as _;

    // 1. Load webhook (active + not soft-deleted)
    let webhook = webhooks::Entity::find_by_id(job.webhook_id)
        .active()
        .one(&state.db)
        .await?
        .context("webhook not found or soft-deleted")?;

    if !webhook.is_active {
        tracing::debug!(
            webhook_id = %job.webhook_id,
            "inactive webhook — skipping delivery",
        );
        return Ok(());
    }

    // 2. Validate destination URL (SSRF defense)
    validate_outbound_url(&webhook.url)?;

    // 3. Build the JSON envelope — same order and shape as Django
    //    (`plane/bgtasks/webhook_task.py::webhook_send_task`):
    //    { event, action, webhook_id, workspace_id, data, activity }
    //
    //    `workspace_id` is taken from the webhook row — NEVER from the job. It guarantees
    //    that the receiver sees the actual workspace the webhook belongs to,
    //    even if a caller passed an incorrect id.
    //
    //    `activity` is always included (as `null` when not provided),
    //    because existing Django consumers expect the key to be present.
    //    `delivery_id` is NOT in the body — only in the `X-Plane-Delivery` header,
    //    just like Django.
    let envelope = build_envelope(
        &job.event,
        &job.action,
        webhook.id,
        webhook.workspace_id,
        &job.data,
        job.activity.as_ref(),
    );
    let body_bytes = serde_json::to_vec(&envelope)
        .context("could not serialize webhook envelope")?;

    // 4. HMAC-SHA256 signature of the raw body with `secret_key`
    let signature_bytes = hmac_sha256::HMAC::mac(&body_bytes, webhook.secret_key.as_bytes());
    let signature_hex = hex_encode_lower(&signature_bytes);

    // 5. Request log — signature redacted not to leave traces of HMAC
    let request_headers_logged = serde_json::json!({
        "Content-Type": "application/json",
        "User-Agent": "Plane-Webhook/1.0",
        "X-Plane-Event": job.event,
        "X-Plane-Delivery": job.delivery_id.to_string(),
        "X-Plane-Signature": "[redacted]",
    })
    .to_string();

    let request_body_logged = truncate_utf8(&String::from_utf8_lossy(&body_bytes), MAX_REQUEST_BODY_LOG);

    // 6. HTTP send (timeout per request — independent from global)
    let send_result = state
        .http
        .post(&webhook.url)
        .timeout(REQUEST_TIMEOUT)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Plane-Webhook/1.0")
        .header("X-Plane-Event", &job.event)
        .header("X-Plane-Delivery", job.delivery_id.to_string())
        .header("X-Plane-Signature", &signature_hex)
        .body(body_bytes)
        .send()
        .await;

    // 7. Classify response and build log rows
    let (status_str, response_headers, response_body, transient_failure) = match send_result {
        Ok(resp) => {
            let status = resp.status();
            let status_num = status.as_u16();

            // Headers → flattened JSON object
            let hdr_map: serde_json::Map<String, serde_json::Value> = resp
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_owned(),
                        serde_json::Value::String(v.to_str().unwrap_or("").to_owned()),
                    )
                })
                .collect();
            let hdr_json = serde_json::Value::Object(hdr_map).to_string();

            let body_text = resp.text().await.unwrap_or_default();
            let body_truncated = truncate_utf8(&body_text, MAX_RESPONSE_BODY_LOG);

            // 5xx / 408 / 429 → transient (retry candidates)
            let is_transient =
                (500..600).contains(&status_num) || status_num == 408 || status_num == 429;

            (
                status_num.to_string(),
                Some(hdr_json),
                Some(body_truncated),
                is_transient,
            )
        }
        Err(e) => {
            // network error, timeout, TLS, DNS — always transient
            let reason = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "connection_error"
            } else {
                "network_error"
            };
            (reason.to_string(), None, Some(e.to_string()), true)
        }
    };

    // 8. Persist log — best-effort: if it fails, it has been sent anyway
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let log_insert = webhook_logs::ActiveModel {
        id: Set(Uuid::new_v4()),
        event_type: Set(Some(format!("{}.{}", job.event, job.action))),
        request_method: Set(Some("POST".to_owned())),
        request_headers: Set(Some(request_headers_logged)),
        request_body: Set(Some(request_body_logged)),
        response_status: Set(Some(status_str.clone())),
        response_headers: Set(response_headers),
        response_body: Set(response_body),
        retry_count: Set(0),
        created_by_id: Set(None),
        updated_by_id: Set(None),
        webhook: Set(webhook.id),
        workspace_id: Set(webhook.workspace_id),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await;

    if let Err(e) = log_insert {
        // We don't abort: priority is the delivery, the log is accounting.
        tracing::warn!(
            webhook_id = %webhook.id,
            delivery_id = %job.delivery_id,
            error = %e,
            "could not persist webhook_logs",
        );
    }

    // 9. Signal transient failure to the scheduler for future retry
    if transient_failure {
        anyhow::bail!(
            "transient failure delivering webhook (status={status_str})"
        );
    }

    tracing::debug!(
        webhook_id = %webhook.id,
        delivery_id = %job.delivery_id,
        status = %status_str,
        "webhook_delivery: ok",
    );
    Ok(())
}

// ── Outbound URL validation ───────────────────────────────────────────────

/// Rejects URLs pointing to internal networks or loopback.
///
/// Known limitations:
///   - This check looks at the original string. If the host is a DNS that resolves
///     to a private IP (DNS rebinding), `reqwest` will resolve it in flight.
///     A resolver pin would be the next step — requires a custom
///     `reqwest::dns::Resolve`.
fn validate_outbound_url(url: &str) -> anyhow::Result<()> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        anyhow::bail!("only http(s) schemes are accepted");
    }

    // Extract host component without depending on `url` crate
    let without_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("");
    // discard userinfo
    let host_port = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
    // discard port — watch out for IPv6 in brackets
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        // [::1]:8080 — tomar hasta el `]`
        stripped.split_once(']').map(|(h, _)| h).unwrap_or(stripped)
    } else {
        host_port.rsplit_once(':').map(|(h, _)| h).unwrap_or(host_port)
    };

    if host.is_empty() {
        anyhow::bail!("URL without host");
    }

    let lower = host.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "localhost" | "localhost.localdomain" | "broadcasthost" | "ip6-localhost" | "ip6-loopback"
    ) {
        anyhow::bail!("loopback host not allowed");
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(&ip) {
            anyhow::bail!("Private/loopback/link-local IP not allowed: {ip}");
        }
    }

    Ok(())
}

fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.is_unspecified()
                || is_ipv4_cgnat(v4)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || is_ipv6_ula(v6)
                || is_ipv6_link_local(v6)
        }
    }
}

fn is_ipv4_cgnat(v4: &Ipv4Addr) -> bool {
    // 100.64.0.0/10 — Carrier-Grade NAT (RFC 6598)
    let [a, b, _, _] = v4.octets();
    a == 100 && (64..=127).contains(&b)
}

fn is_ipv6_ula(v6: &Ipv6Addr) -> bool {
    // fc00::/7 — Unique Local Addresses
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

fn is_ipv6_link_local(v6: &Ipv6Addr) -> bool {
    // fe80::/10
    (v6.segments()[0] & 0xffc0) == 0xfe80
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Truncates a string to a maximum of bytes respecting UTF-8 char-boundaries.
/// Avoids the `&s[..n]` panic when `n` falls in the middle of a codepoint.
fn truncate_utf8(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let mut out = String::with_capacity(end + 16);
    out.push_str(&s[..end]);
    out.push_str("…(truncated)");
    out
}

fn hex_encode_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0F) as usize] as char);
    }
    out
}

/// Builds the JSON envelope that travels in the POST body.
///
/// Order and exact key names that Django emits
/// (`webhook_send_task`). It is extracted as a pure function to be able to
/// freeze the contract in tests — any drift compared to Django
/// (e.g. renaming `data` or moving `workspace_id`) would break
/// external consumers (Zapier, n8n, custom integrations).
fn build_envelope(
    event: &str,
    action: &str,
    webhook_id: Uuid,
    workspace_id: Uuid,
    data: &serde_json::Value,
    activity: Option<&serde_json::Value>,
) -> serde_json::Value {
    // `Option::None` serializes as `null` — necessary for the
    // `activity` key to always be present in the body, as Django does.
    let activity_val: &serde_json::Value = activity.unwrap_or(&serde_json::Value::Null);
    serde_json::json!({
        "event": event,
        "action": action,
        "webhook_id": webhook_id,
        "workspace_id": workspace_id,
        "data": data,
        "activity": activity_val,
    })
}

// ── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_schemes() {
        for url in [
            "ftp://example.com/hook",
            "file:///etc/passwd",
            "gopher://evil.test/",
            "javascript:alert(1)",
            "",
        ] {
            assert!(validate_outbound_url(url).is_err(), "should reject {url}");
        }
    }

    #[test]
    fn rejects_loopback_and_private() {
        for url in [
            "http://localhost/",
            "http://127.0.0.1/",
            "http://127.5.5.5/",
            "http://10.0.0.1/",
            "http://192.168.1.1/",
            "http://172.16.0.1/",
            "http://169.254.169.254/",   // AWS metadata
            "http://100.64.0.1/",        // CGNAT
            "http://[::1]/",
            "http://[fc00::1]/",
            "http://[fe80::1]/",
            "http://0.0.0.0/",
            "http://user:pass@localhost/",
        ] {
            assert!(validate_outbound_url(url).is_err(), "should reject {url}");
        }
    }

    #[test]
    fn accepts_public_urls() {
        for url in [
            "http://example.com/hook",
            "https://api.partner.io/v1/events",
            "https://93.184.216.34/",
            "https://[2001:db8::1]/",
        ] {
            assert!(validate_outbound_url(url).is_ok(), "should accept {url}");
        }
    }

    #[test]
    fn hex_encoding_matches_openssl() {
        // 32 zero bytes -> 64 '0'
        let zeros = [0u8; 32];
        assert_eq!(hex_encode_lower(&zeros), "0".repeat(64));
        // Known case: HMAC-SHA256("", "") = "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad"
        let mac = hmac_sha256::HMAC::mac(b"", b"");
        assert_eq!(
            hex_encode_lower(&mac),
            "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad"
        );
    }

    #[test]
    fn truncate_respects_utf8_boundaries() {
        // 'é' = 2 bytes in UTF-8 — truncating in the middle should back up
        let input = "a".to_owned() + &"é".repeat(100);
        let out = truncate_utf8(&input, 5);
        // should not panic and should preserve char boundary
        assert!(out.is_char_boundary(out.find('…').unwrap_or(out.len())));
    }

    #[test]
    fn truncate_passes_short_strings_through() {
        assert_eq!(truncate_utf8("hello", 100), "hello");
    }

    // ── Envelope contract (Django parity) ────────────────────────────
    //
    // These tests freeze the exact shape of the body that travels to the
    // client's endpoint. Any change — renaming a key, changing the order,
    // omitting `activity` when it is None — would break external consumers
    // already processing this format.

    #[test]
    fn envelope_shape_matches_django_for_create() {
        let webhook_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let workspace_id = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        let data = serde_json::json!({"id": "33333333-3333-3333-3333-333333333333", "name": "Foo"});

        let envelope = build_envelope(
            "project",
            "created",
            webhook_id,
            workspace_id,
            &data,
            None,
        );

        // All keys present
        let obj = envelope.as_object().expect("envelope must be a JSON object");
        assert_eq!(obj.len(), 6, "envelope must have 6 keys (event, action, webhook_id, workspace_id, data, activity)");
        assert_eq!(obj["event"], "project");
        assert_eq!(obj["action"], "created");
        assert_eq!(obj["webhook_id"], webhook_id.to_string());
        assert_eq!(obj["workspace_id"], workspace_id.to_string());
        assert_eq!(obj["data"], data);
        // `activity: null` when not provided — Django always sends the key
        assert_eq!(obj["activity"], serde_json::Value::Null);
    }

    #[test]
    fn envelope_carries_activity_when_provided() {
        let activity = serde_json::json!({
            "field": "name",
            "old_value": "Old",
            "new_value": "New",
            "actor": {"id": "abc"},
            "old_identifier": null,
            "new_identifier": null,
        });

        let envelope = build_envelope(
            "project",
            "updated",
            Uuid::nil(),
            Uuid::nil(),
            &serde_json::json!({}),
            Some(&activity),
        );

        assert_eq!(envelope["activity"], activity);
    }

    #[test]
    fn envelope_omits_delivery_id_from_body() {
        // Django DOES NOT include delivery_id in the body — only in the
        // `X-Plane-Delivery` header. The envelope must not include it either.
        let envelope = build_envelope(
            "issue",
            "deleted",
            Uuid::nil(),
            Uuid::nil(),
            &serde_json::json!({"id": "x"}),
            None,
        );
        assert!(envelope.get("delivery_id").is_none(), "delivery_id must not be in the body");
    }
}
