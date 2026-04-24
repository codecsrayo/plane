// src/jobs/webhook_delivery.rs
//! Job: entrega (fan-out) de webhooks salientes.
//!
//! Equivalente a `plane/bgtasks/webhook_task.py` de Django.
//!
//! Flujo:
//!   1. Recibir `DeliverWebhookJob { webhook_id, event, action, data, activity, delivery_id }`.
//!   2. Cargar el webhook (activo y no soft-deleted).
//!   3. Validar URL destino (defensa SSRF: rechaza loopback/privadas/link-local/multicast).
//!   4. Firmar el body con HMAC-SHA256(secret_key).
//!   5. POST con timeout de 10s y headers estandarizados.
//!   6. Persistir request/response en `webhook_logs` (body truncado para no saturar DB).
//!   7. Devolver `Err` en 5xx/408/429/network — permite retry cuando se añada
//!      el middleware de reintentos de apalis (features `retry` ya habilitadas).
//!
//! Seguridad:
//!   - Solo esquemas `http`/`https`.
//!   - Rechazo de IPs RFC1918, loopback, link-local, CGNAT, ULA IPv6, multicast.
//!   - El valor del header `X-Plane-Signature` se redacta en el log — el secreto
//!     no se filtra a operadores que revisen la tabla.
//!   - `reqwest` usa TLS rustls (ver Cargo.toml); no se desactiva la verificación.
//!
//! No cubre aún (siguientes commits):
//!   - Fan-out: este job entrega a UN webhook; el dispatcher que descubre los
//!     webhooks suscritos y encola N jobs vive en `utils::webhook_dispatch`.
//!   - Reintentos automáticos: requiere apilar `.retry(...)` sobre el worker.
//!   - Pinning DNS (anti rebind): `reqwest` resuelve el hostname después de
//!     nuestra validación; una IP pública podría re-resolver a una privada.

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

// ── Constantes ───────────────────────────────────────────────────────────────

/// Límite de tamaño del body guardado en `webhook_logs.request_body`.
const MAX_REQUEST_BODY_LOG: usize = 64 * 1024;
/// Límite de tamaño del body de respuesta guardado en `webhook_logs.response_body`.
const MAX_RESPONSE_BODY_LOG: usize = 8 * 1024;
/// Timeout por intento — menor que el timeout global del `AppState.http` (30s)
/// para que un destino lento no bloquee el worker.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

// ── Job payload ──────────────────────────────────────────────────────────────

/// Un envío individual. El fan-out a N webhooks encola N `DeliverWebhookJob`.
///
/// El envelope emitido al endpoint del cliente replica el formato de Django
/// (`plane/bgtasks/webhook_task.py::webhook_send_task`):
///
/// ```json
/// { "event": ..., "action": ..., "webhook_id": ..., "workspace_id": ...,
///   "data": ..., "activity": ... }
/// ```
///
/// `workspace_id` NO viaja en el job porque siempre se deriva del row del
/// webhook — evita cualquier desalineación con el workspace del que se
/// disparó el evento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverWebhookJob {
    pub webhook_id: Uuid,
    /// `"project"`, `"issue"`, `"cycle"`, `"module"`, `"issue_comment"`, etc.
    pub event: String,
    /// `"created"`, `"updated"`, `"deleted"`.
    pub action: String,
    /// Modelo serializado (o `{"id": ...}` para deletes). Va en el campo `data`
    /// del envelope — nombre alineado con Django.
    pub data: serde_json::Value,
    /// Bloque opcional `activity` del envelope. Para create/delete normalmente
    /// es `None`; para updates Django lo usa para reportar el diff por campo
    /// (`{field, old_value, new_value, actor, ...}`). `None` → `"activity": null`
    /// en el body, mismo contrato que Django, cuyo `webhook_activity` siempre
    /// envía la clave.
    #[serde(default)]
    pub activity: Option<serde_json::Value>,
    /// UUID único por intento — se expone como header `X-Plane-Delivery`.
    /// NO se incluye en el body del POST (Django tampoco lo incluye).
    pub delivery_id: Uuid,
}

// ── Handler ──────────────────────────────────────────────────────────────────

pub async fn handle_deliver_webhook(
    job: DeliverWebhookJob,
    ctx: Data<AppState>,
) -> Result<(), Error> {
    let state: AppState = (*ctx).clone();

    if let Err(e) = run_delivery(&state, &job).await {
        // `%e` oculta la causa raíz de `anyhow::Context` — aquí preferimos `?`
        // para ver el detalle completo en incidentes.
        tracing::warn!(
            webhook_id = %job.webhook_id,
            delivery_id = %job.delivery_id,
            event = %job.event,
            action = %job.action,
            error = ?e,
            "webhook_delivery: intento fallido",
        );
        return Err(apalis::prelude::Error::Failed(std::sync::Arc::new(e.into())));
    }

    Ok(())
}

// ── Flujo principal ──────────────────────────────────────────────────────────

async fn run_delivery(state: &AppState, job: &DeliverWebhookJob) -> anyhow::Result<()> {
    use anyhow::Context as _;

    // 1. Cargar webhook (activo + no soft-deleted)
    let webhook = webhooks::Entity::find_by_id(job.webhook_id)
        .active()
        .one(&state.db)
        .await?
        .context("webhook no encontrado o soft-deleted")?;

    if !webhook.is_active {
        tracing::debug!(
            webhook_id = %job.webhook_id,
            "webhook inactivo — se omite la entrega",
        );
        return Ok(());
    }

    // 2. Validar URL destino (defensa SSRF)
    validate_outbound_url(&webhook.url)?;

    // 3. Construir el sobre JSON — mismo orden y shape que Django
    //    (`plane/bgtasks/webhook_task.py::webhook_send_task`):
    //    { event, action, webhook_id, workspace_id, data, activity }
    //
    //    `workspace_id` se toma del row del webhook — NUNCA del job. Garantiza
    //    que el receptor ve el workspace real al que pertenece el webhook,
    //    incluso si un caller pasara un id incorrecto.
    //
    //    `activity` se incluye siempre (como `null` cuando no se proporciona),
    //    porque los consumidores Django existentes esperan la clave presente.
    //    `delivery_id` NO va en el body — solo en el header `X-Plane-Delivery`,
    //    igual que Django.
    let envelope = build_envelope(
        &job.event,
        &job.action,
        webhook.id,
        webhook.workspace_id,
        &job.data,
        job.activity.as_ref(),
    );
    let body_bytes = serde_json::to_vec(&envelope)
        .context("no se pudo serializar el envelope del webhook")?;

    // 4. Firma HMAC-SHA256 del body crudo con `secret_key`
    let signature_bytes = hmac_sha256::HMAC::mac(&body_bytes, webhook.secret_key.as_bytes());
    let signature_hex = hex_encode_lower(&signature_bytes);

    // 5. Log de request — signature redactada para no dejar rastro del HMAC
    let request_headers_logged = serde_json::json!({
        "Content-Type": "application/json",
        "User-Agent": "Plane-Webhook/1.0",
        "X-Plane-Event": job.event,
        "X-Plane-Delivery": job.delivery_id.to_string(),
        "X-Plane-Signature": "[redacted]",
    })
    .to_string();

    let request_body_logged = truncate_utf8(&String::from_utf8_lossy(&body_bytes), MAX_REQUEST_BODY_LOG);

    // 6. Envío HTTP (timeout por request — independiente del global)
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

    // 7. Clasificar respuesta y armar filas de log
    let (status_str, response_headers, response_body, transient_failure) = match send_result {
        Ok(resp) => {
            let status = resp.status();
            let status_num = status.as_u16();

            // Headers → JSON objeto aplanado
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

            // 5xx / 408 / 429 → transitorios (candidatos a reintento)
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
            // network error, timeout, TLS, DNS — siempre transitorio
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

    // 8. Persistir log — best-effort: si falla, se ha enviado igual
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
        // No abortamos: prioridad es la entrega, el log es contabilidad.
        tracing::warn!(
            webhook_id = %webhook.id,
            delivery_id = %job.delivery_id,
            error = %e,
            "no se pudo persistir webhook_logs",
        );
    }

    // 9. Señalar fallo transitorio al scheduler para reintento futuro
    if transient_failure {
        anyhow::bail!(
            "fallo transitorio al entregar webhook (status={status_str})"
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

// ── Validación de URL saliente ───────────────────────────────────────────────

/// Rechaza URLs que apunten a redes internas o loopback.
///
/// Limitaciones conocidas:
///   - Este check mira el string original. Si el host es un DNS que resuelve
///     a una IP privada (DNS rebinding), `reqwest` lo resolverá en vuelo.
///     Un pin de resolver sería el siguiente paso — requiere un custom
///     `reqwest::dns::Resolve`.
fn validate_outbound_url(url: &str) -> anyhow::Result<()> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        anyhow::bail!("solo se aceptan esquemas http(s)");
    }

    // Extraer el componente host sin depender del crate `url`
    let without_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let authority = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("");
    // descartar userinfo
    let host_port = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
    // descartar puerto — ojo con IPv6 entre corchetes
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        // [::1]:8080 — tomar hasta el `]`
        stripped.split_once(']').map(|(h, _)| h).unwrap_or(stripped)
    } else {
        host_port.rsplit_once(':').map(|(h, _)| h).unwrap_or(host_port)
    };

    if host.is_empty() {
        anyhow::bail!("URL sin host");
    }

    let lower = host.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "localhost" | "localhost.localdomain" | "broadcasthost" | "ip6-localhost" | "ip6-loopback"
    ) {
        anyhow::bail!("host loopback no permitido");
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(&ip) {
            anyhow::bail!("IP privada/loopback/link-local no permitida: {ip}");
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

/// Trunca una cadena a un máximo de bytes respetando char-boundaries UTF-8.
/// Evita el panic de `&s[..n]` cuando `n` cae en medio de un codepoint.
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

/// Arma el envelope JSON que viaja en el body del POST.
///
/// Orden y nombres de claves exactos que emite Django
/// (`webhook_send_task`). Se extrae como función pura para poder
/// freezar el contrato en tests — cualquier drift frente a Django
/// (eg. renombrar `data` o mover `workspace_id`) rompería a los
/// consumidores externos (Zapier, n8n, integraciones custom).
fn build_envelope(
    event: &str,
    action: &str,
    webhook_id: Uuid,
    workspace_id: Uuid,
    data: &serde_json::Value,
    activity: Option<&serde_json::Value>,
) -> serde_json::Value {
    // `Option::None` se serializa como `null` — necesario para que la clave
    // `activity` esté siempre presente en el body, como Django.
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

// ── Tests unitarios ──────────────────────────────────────────────────────────

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
        // 32 bytes de ceros → 64 '0'
        let zeros = [0u8; 32];
        assert_eq!(hex_encode_lower(&zeros), "0".repeat(64));
        // Caso conocido: HMAC-SHA256("", "") = "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad"
        let mac = hmac_sha256::HMAC::mac(b"", b"");
        assert_eq!(
            hex_encode_lower(&mac),
            "b613679a0814d9ec772f95d778c35fc5ff1697c493715653c6c712144292c5ad"
        );
    }

    #[test]
    fn truncate_respects_utf8_boundaries() {
        // 'é' = 2 bytes en UTF-8 — truncar en medio debería retroceder
        let input = "a".to_owned() + &"é".repeat(100);
        let out = truncate_utf8(&input, 5);
        // no debe panicar y debe preservar char boundary
        assert!(out.is_char_boundary(out.find('…').unwrap_or(out.len())));
    }

    #[test]
    fn truncate_passes_short_strings_through() {
        assert_eq!(truncate_utf8("hola", 100), "hola");
    }

    // ── Contrato de envelope (paridad Django) ────────────────────────────
    //
    // Estos tests congelan el shape exacto del body que viaja al endpoint
    // del cliente. Cualquier cambio — renombrar una clave, cambiar el orden,
    // omitir `activity` cuando es None — rompería a los consumidores externos
    // que ya procesan este formato.

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

        // Todas las claves presentes
        let obj = envelope.as_object().expect("envelope debe ser un objeto JSON");
        assert_eq!(obj.len(), 6, "envelope debe tener 6 claves (event, action, webhook_id, workspace_id, data, activity)");
        assert_eq!(obj["event"], "project");
        assert_eq!(obj["action"], "created");
        assert_eq!(obj["webhook_id"], webhook_id.to_string());
        assert_eq!(obj["workspace_id"], workspace_id.to_string());
        assert_eq!(obj["data"], data);
        // `activity: null` cuando no se proporciona — Django envía la clave siempre
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
        // Django NO incluye delivery_id en el body — solo en el header
        // `X-Plane-Delivery`. El envelope tampoco debe incluirlo.
        let envelope = build_envelope(
            "issue",
            "deleted",
            Uuid::nil(),
            Uuid::nil(),
            &serde_json::json!({"id": "x"}),
            None,
        );
        assert!(envelope.get("delivery_id").is_none(), "delivery_id no debe estar en el body");
    }
}
