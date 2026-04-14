// src/auth/rate_limit.rs
//
// Fase 1: rate limiter en memoria con std::sync::Mutex<HashMap>.
// Fase 3: migrar a Redis (fred) para soporte multi-réplica.
//
// ✅ std::sync::Mutex — NO tokio::sync::Mutex.
// La sección crítica es trivial (lookup + increment en HashMap) y no contiene
// ningún .await, por lo que bloquear el hilo del OS es correcto y eficiente.
use axum::{
    body::Body,
    http::{HeaderValue, Request, Response},
    middleware::Next,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::error::AppError;

/// sha256(token) → (request_count, window_start_unix_secs)
#[derive(Debug, Default)]
pub struct RateLimitState {
    pub buckets: std::sync::Mutex<HashMap<String, (u32, u64)>>,
    /// Contador de inserciones nuevas desde el último ciclo de poda.
    /// Se usa para amortizar el costo de `retain` cada PRUNE_INTERVAL entradas.
    insert_count: std::sync::atomic::AtomicU64,
}

/// Cada cuántas inserciones nuevas se ejecuta la poda de entradas vencidas.
const PRUNE_INTERVAL: u64 = 1_000;

/// Genera un bucket key opaco a partir del raw token.
pub fn bucket_key(raw_token: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw_token.as_bytes());
    h.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Aplica el rate limit en memoria al bucket del token dado.
pub fn apply_rate_limit(state: &RateLimitState, raw_key: &str, limit: u32) -> Result<(), AppError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let window = 60u64;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let window_start = (now / window) * window;

    let bucket = bucket_key(raw_key);
    let mut buckets = state.buckets.lock().unwrap_or_else(|p| p.into_inner());

    let is_new_entry = !buckets.contains_key(&bucket);
    let entry = buckets.entry(bucket).or_insert((0, window_start));

    if entry.1 < window_start {
        *entry = (0, window_start);
    }

    entry.0 += 1;
    let over_limit = entry.0 > limit;

    // Poda amortizada: eliminar buckets de ventanas vencidas cada PRUNE_INTERVAL
    // inserciones nuevas para evitar crecimiento ilimitado del HashMap (memory leak).
    if is_new_entry {
        let prev = state
            .insert_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if prev % PRUNE_INTERVAL == PRUNE_INTERVAL - 1 {
            buckets.retain(|_, v| v.1 >= window_start);
        }
    }

    if over_limit {
        Err(AppError::RateLimited)
    } else {
        Ok(())
    }
}

/// Middleware Tower — inyecta headers X-RateLimit-* en la respuesta.
pub async fn rate_limit_headers_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    let has_api_key = req.headers().contains_key("x-api-key");
    let window = 60u64;
    let now = chrono::Utc::now().timestamp() as u64;
    let reset_at = ((now / window) + 1) * window;
    let reset_hv = HeaderValue::from_str(&reset_at.to_string())
        .unwrap_or_else(|_| HeaderValue::from_static("0"));

    let mut resp = next.run(req).await;
    if has_api_key {
        resp.headers_mut().insert("X-RateLimit-Reset", reset_hv);
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::{apply_rate_limit, bucket_key, RateLimitState};

    #[test]
    fn bucket_key_is_stable_and_hides_raw_token() {
        let first = bucket_key("secret-token");
        let second = bucket_key("secret-token");

        assert_eq!(first, second);
        assert_ne!(first, "secret-token");
    }

    #[test]
    fn apply_rate_limit_rejects_after_limit() {
        let state = RateLimitState::default();

        assert!(apply_rate_limit(&state, "token-a", 2).is_ok());
        assert!(apply_rate_limit(&state, "token-a", 2).is_ok());
        assert!(apply_rate_limit(&state, "token-a", 2).is_err());
    }
}
