// src/utils/token_cipher.rs
//
//! Cifrado simétrico de tokens OAuth en reposo.
//!
//! ## Algoritmo
//! AES-256-GCM con nonce aleatorio de 96 bits (12 bytes) generado por el OS.
//! El auth tag de 128 bits (16 bytes) está incluido en el ciphertext devuelto
//! por `aes_gcm::Aead::encrypt`.
//!
//! ## Formato en base de datos
//! ```text
//! v1:<base64-standard(nonce[12] || ciphertext_with_tag)>
//! ```
//!
//! El prefijo `v1:` permite detectar versión del esquema y distinguir filas
//! cifradas de filas plaintext heredadas de la API Django.
//!
//! ## Compatibilidad hacia atrás
//! `decrypt_token` acepta valores sin prefijo `v1:` y los retorna tal cual.
//! Esto permite migración gradual: filas Django siguen siendo legibles hasta
//! que se re-cifren en un job de migración coordinado.
//!
//! ## Configuración
//! Variable de entorno: `TOKEN_ENCRYPTION_KEY`
//! Valor: clave de 32 bytes codificada en base64-standard.
//!
//! Generación de clave de ejemplo:
//! ```bash
//! openssl rand -base64 32
//! ```
//!
//! ## Degradación cuando la clave no está configurada
//! - **Escritura**: se loggea un `WARN` y se almacena en texto plano.
//!   Permite deploys sin romper funcionalidad, pero la advertencia es visible
//!   en logs de producción para exigir configuración.
//! - **Lectura**: token sin prefijo `v1:` → retorno directo (plaintext legacy).
//!   Token con prefijo `v1:` sin clave → `AppError::Internal` (inconsistencia).

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::error::AppError;

/// Prefijo de versión. Permite distinguir tokens cifrados de plaintext legacy.
const TOKEN_PREFIX: &str = "v1:";

/// Variable de entorno con la clave AES-256-GCM (base64, 32 bytes).
const KEY_ENV_VAR: &str = "TOKEN_ENCRYPTION_KEY";

/// Carga la clave de 32 bytes desde `TOKEN_ENCRYPTION_KEY`.
///
/// Retorna `None` si la variable no existe, no es base64 válido o no tiene 32 bytes.
fn load_key() -> Option<[u8; 32]> {
    let b64 = std::env::var(KEY_ENV_VAR).ok()?;
    let bytes = STANDARD.decode(b64.trim()).ok()?;
    if bytes.len() != 32 {
        tracing::error!(
            var = KEY_ENV_VAR,
            actual_bytes = bytes.len(),
            expected_bytes = 32,
            "TOKEN_ENCRYPTION_KEY tiene longitud incorrecta — se ignorará"
        );
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}

/// Cifra un token OAuth con AES-256-GCM y retorna `v1:<b64(nonce||ciphertext)>`.
///
/// Si `TOKEN_ENCRYPTION_KEY` no está configurada, retorna el valor original
/// con un `WARN` en logs — degradación suave que no rompe despliegues existentes.
///
/// # Panics
/// No produce panics: `Aes256Gcm::new_from_slice` solo falla si la clave no
/// tiene 32 bytes, condición que se verifica en `load_key`.
pub fn encrypt_token(plaintext: &str) -> String {
    let Some(key_bytes) = load_key() else {
        tracing::warn!(
            var = KEY_ENV_VAR,
            "TOKEN_ENCRYPTION_KEY no configurada — GitHub access_token almacenado en \
             texto plano. Configure la variable para habilitar AES-256-GCM en reposo."
        );
        return plaintext.to_owned();
    };

    // new_from_slice no falla: load_key garantiza exactamente 32 bytes.
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .expect("load_key garantiza 32 bytes — new_from_slice no puede fallar");

    // Nonce aleatorio de 96 bits por el OS; único por operación de cifrado.
    // Reutilizar nonces con la misma clave rompería la confidencialidad de GCM.
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // encrypt() devuelve ciphertext || tag (los últimos 16 bytes son el auth tag).
    // No puede fallar con nonce y clave válidos.
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("AES-256-GCM encrypt: parámetros válidos, no debe fallar");

    // Layout: nonce[12] || ciphertext_with_tag
    let mut blob = Vec::with_capacity(12 + ciphertext.len());
    blob.extend_from_slice(nonce.as_slice());
    blob.extend_from_slice(&ciphertext);

    format!("{TOKEN_PREFIX}{}", STANDARD.encode(&blob))
}

/// Descifra un token almacenado en base de datos.
///
/// | Valor almacenado         | Comportamiento                                    |
/// |--------------------------|---------------------------------------------------|
/// | `v1:<b64>` + clave OK    | Descifra AES-256-GCM, retorna plaintext            |
/// | `v1:<b64>` + sin clave   | `AppError::Internal` — inconsistencia de config   |
/// | Sin prefijo `v1:`        | Retorno directo (plaintext legacy — Django compat) |
///
/// El caso "sin prefijo" permite leer filas existentes de Django sin migración
/// previa, lo que posibilita el despliegue incremental del cifrado.
pub fn decrypt_token(stored: &str) -> Result<String, AppError> {
    // ── Compatibilidad hacia atrás: token sin prefijo = plaintext legacy ──
    let Some(b64_part) = stored.strip_prefix(TOKEN_PREFIX) else {
        return Ok(stored.to_owned());
    };

    // Token con prefijo v1: — requiere clave para descifrar
    let key_bytes = load_key().ok_or_else(|| {
        AppError::Internal(anyhow::anyhow!(
            "Token cifrado (`v1:`) encontrado en DB pero TOKEN_ENCRYPTION_KEY no está \
             configurada. Configure la variable para poder descifrar el token."
        ))
    })?;

    let blob = STANDARD.decode(b64_part).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("token_cipher: base64 inválido: {e}"))
    })?;

    // Mínimo: nonce(12) + tag(16) = 28 bytes; ciphertext puede ser 0 bytes
    if blob.len() < 28 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "token_cipher: blob demasiado corto ({} bytes, mínimo 28)",
            blob.len()
        )));
    }

    let (nonce_bytes, ciphertext_with_tag) = blob.split_at(12);

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("token_cipher: init cipher: {e}")))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    // Verifica auth tag automáticamente; falla si clave o datos son incorrectos.
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext_with_tag)
        .map_err(|_| {
            AppError::Internal(anyhow::anyhow!(
                "token_cipher: decryption failed — clave incorrecta o datos corruptos"
            ))
        })?;

    String::from_utf8(plaintext_bytes).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("token_cipher: UTF-8 inválido: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::{decrypt_token, encrypt_token, TOKEN_PREFIX};
    use base64::Engine as _;

    // ── Tests que no requieren clave configurada ─────────────────────────────

    #[test]
    fn legacy_plaintext_passthrough() {
        // Token sin prefijo v1: → retorno directo (filas Django existentes)
        let legacy = "ghs_plaintexttoken1234";
        assert_eq!(decrypt_token(legacy).unwrap(), legacy);
    }

    #[test]
    fn encrypt_without_key_returns_plaintext_with_warning() {
        // Sin TOKEN_ENCRYPTION_KEY el valor se almacena sin cifrar
        std::env::remove_var("TOKEN_ENCRYPTION_KEY");
        let token = "raw_token_value";
        let stored = encrypt_token(token);
        assert_eq!(stored, token);
        assert!(!stored.starts_with(TOKEN_PREFIX));
    }

    // ── Tests que requieren clave ─────────────────────────────────────────────

    fn set_test_key() {
        // 32 bytes de ceros codificados en base64 — SOLO para tests
        let key = base64::engine::general_purpose::STANDARD.encode([0u8; 32]);
        std::env::set_var("TOKEN_ENCRYPTION_KEY", key);
    }

    #[test]
    fn roundtrip_encrypt_decrypt() {
        set_test_key();
        let original = "ghs_realGitHubToken_abc123";
        let stored = encrypt_token(original);

        // El valor almacenado es opaco y tiene el prefijo correcto
        assert!(stored.starts_with(TOKEN_PREFIX));
        assert_ne!(stored, original);

        // La decryption recupera el original exacto
        let recovered = decrypt_token(&stored).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn nonces_are_unique_per_call() {
        set_test_key();
        let token = "same_token";
        let a = encrypt_token(token);
        let b = encrypt_token(token);
        // Dos cifrados del mismo plaintext deben ser distintos (nonce aleatorio)
        assert_ne!(a, b, "nonces deben ser únicos por llamada");
    }

    #[test]
    fn tampered_ciphertext_fails_auth() {
        set_test_key();
        let stored = encrypt_token("legit_token");
        // Corromper el último byte del ciphertext
        let mut bytes = stored.as_bytes().to_vec();
        let last = bytes.last_mut().unwrap();
        *last ^= 0xFF;
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(
            decrypt_token(&tampered).is_err(),
            "GCM auth tag debe detectar tampering"
        );
    }
}
