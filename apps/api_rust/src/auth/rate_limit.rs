// src/auth/rate_limit.rs
//
// Fase 1: rate limiter en memoria con std::sync::Mutex<HashMap>.
// Fase 3: migrar a Redis (fred) para soporte multi-réplica.
//
// ✅ std::sync::Mutex — NO tokio::sync::Mutex.
// La sección crítica es trivial (lookup + increment en HashMap) y no contiene
// ningún .await, por lo que bloquear el hilo del OS es correcto y eficiente.
use std::collections::HashMap;

/// sha256(token) → (request_count, window_start_unix_secs)
#[derive(Debug, Default)]
pub struct RateLimitState {
    pub buckets: std::sync::Mutex<HashMap<String, (u32, u64)>>,
}
