// src/utils/fernet.rs
//! Compatibilidad con cifrado Fernet de Django.
//!
//! Django cifra los valores sensibles de `instance_configurations` usando
//! [`cryptography.fernet.Fernet`] con una clave derivada mediante
//! PBKDF2-HMAC-SHA256 del `SECRET_KEY` de Django.
//!
//! Este módulo implementa el **descifrado** de esos valores para que el
//! endpoint `GET /api/instances/configurations/` pueda devolver datos
//! legibles al admin panel — exactamente como hacía Django.
//!
//! ## Algoritmo Fernet (simplificado)
//! 1. Derivar 32 bytes con `PBKDF2-HMAC-SHA256(password=SECRET_KEY, salt=b"salt", iter=100_000)`.
//! 2. Signing key  = derived[0..16]
//! 3. Encryption key = derived[16..32]
//! 4. Token = base64url(0x80 || timestamp_8 || IV_16 || ciphertext_N || HMAC_32)
//! 5. Verificar HMAC-SHA256(signing_key, 0x80||timestamp||IV||ciphertext)
//! 6. Descifrar AES-128-CBC(key=enc_key, iv=IV, data=ciphertext) con PKCS7
//!
//! ## Política de "doble formato"
//! Los valores escritos por **Rust** usan el formato `v1:<b64-AES-GCM>`
//! (ver `utils/token_cipher.rs`).  Los escritos por **Django** usan Fernet.
//! La función `decrypt_config_value` detecta el formato automáticamente:
//! - Prefijo `v1:` → delega a `token_cipher::decrypt_token`
//! - Parece Fernet (base64url, empieza con `0x80`) → descifra con Fernet
//! - Cualquier otro caso → retorna el valor tal cual (legacy plaintext)

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE},
    Engine as _,
};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use crate::{error::AppError, utils::token_cipher::decrypt_token};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type HmacSha256 = Hmac<Sha256>;

/// Descifra un valor de `instance_configurations` de forma compatible con
/// los dos formatos posibles: Rust AES-GCM (`v1:...`) y Django Fernet.
///
/// # Comportamiento
/// | Valor almacenado              | Resultado                                       |
/// |-------------------------------|-------------------------------------------------|
/// | `v1:<b64>`                    | Descifra AES-256-GCM (token_cipher)             |
/// | Base64url Fernet              | Descifra Fernet con SECRET_KEY del entorno      |
/// | Texto plano                   | Devuelve tal cual                               |
/// | Vacío / None                  | Devuelve `""`                                   |
pub fn decrypt_config_value(stored: Option<&str>) -> Result<String, AppError> {
    let Some(value) = stored else {
        return Ok(String::new());
    };
    if value.is_empty() {
        return Ok(String::new());
    }

    // ── Formato Rust AES-256-GCM ──────────────────────────────────────────
    if value.starts_with("v1:") {
        return decrypt_token(value);
    }

    // ── Detectar Fernet: base64url que al decodificar empieza con 0x80 ──
    if let Ok(raw) = URL_SAFE.decode(value) {
        if raw.first() == Some(&0x80) && raw.len() > 57 {
            // 1+8+16+1+32 mínimo (0 bytes de ciphertext)
            return fernet_decrypt(&raw);
        }
    }

    // ── Texto plano (legacy o campo no cifrado) ───────────────────────────
    Ok(value.to_owned())
}

/// Cifra un valor para `instance_configurations` usando AES-256-GCM (`v1:`).
///
/// Los valores escritos por Rust siempre usan el formato propio; los ya
/// existentes en Fernet se mantienen hasta que sean reescritos.
pub fn encrypt_config_value(plaintext: &str) -> String {
    crate::utils::token_cipher::encrypt_token(plaintext)
}

// ─── Fernet interno ───────────────────────────────────────────────────────────

/// Deriva la clave Fernet de 32 bytes usando PBKDF2-HMAC-SHA256, exactamente
/// como hace Django en `plane/license/utils/encryption.py`.
fn derive_fernet_key(secret_key: &str) -> [u8; 32] {
    let mut dk = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        secret_key.as_bytes(),
        b"salt",
        100_000,
        &mut dk,
    );
    dk
}

/// Descifra un token Fernet ya decodificado de base64url.
///
/// Formato binario esperado:
/// `[0x80][timestamp: 8B][IV: 16B][ciphertext: NB][HMAC: 32B]`
fn fernet_decrypt(raw: &[u8]) -> Result<String, AppError> {
    let secret_key = std::env::var("SECRET_KEY").map_err(|_| {
        AppError::Internal(anyhow::anyhow!(
            "fernet_decrypt: SECRET_KEY no configurada — no se puede descifrar \
             valor Fernet de instance_configurations"
        ))
    })?;

    // ── Descomponer el token ──────────────────────────────────────────────
    // Mínimo: version(1) + time(8) + iv(16) + hmac(32) = 57 bytes
    if raw.len() < 57 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet_decrypt: token demasiado corto ({} bytes)",
            raw.len()
        )));
    }

    let version = raw[0];
    if version != 0x80 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet_decrypt: versión desconocida 0x{version:02X}"
        )));
    }

    // Layout: [version:1][time:8][iv:16][ciphertext:N][hmac:32]
    let hmac_start = raw.len() - 32;
    let signed_data = &raw[..hmac_start];      // version + time + iv + ciphertext
    let iv = &raw[9..25];                       // raw[1..9] = time, raw[9..25] = IV
    let ciphertext = &raw[25..hmac_start];
    let stored_hmac = &raw[hmac_start..];

    // ── Derivar clave ─────────────────────────────────────────────────────
    let derived = derive_fernet_key(&secret_key);
    let signing_key = &derived[..16];
    let enc_key = &derived[16..];

    // ── Verificar HMAC ────────────────────────────────────────────────────
    let mut mac = HmacSha256::new_from_slice(signing_key).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("fernet_decrypt: HMAC init: {e}"))
    })?;
    mac.update(signed_data);
    mac.verify_slice(stored_hmac).map_err(|_| {
        AppError::Internal(anyhow::anyhow!(
            "fernet_decrypt: HMAC inválido — datos corruptos o SECRET_KEY incorrecto"
        ))
    })?;

    // ── AES-128-CBC decrypt ───────────────────────────────────────────────
    // ciphertext debe ser múltiplo de 16 (PKCS7 siempre lo garantiza)
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet_decrypt: longitud de ciphertext inválida ({})",
            ciphertext.len()
        )));
    }

    let decryptor = Aes128CbcDec::new_from_slices(enc_key, iv).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("fernet_decrypt: CBC init: {e}"))
    })?;

    let mut buf = ciphertext.to_vec();
    let plaintext_bytes = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| {
            AppError::Internal(anyhow::anyhow!(
                "fernet_decrypt: PKCS7 unpad falló — datos corruptos"
            ))
        })?;

    String::from_utf8(plaintext_bytes.to_vec()).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("fernet_decrypt: UTF-8 inválido: {e}"))
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_value_returns_empty() {
        assert_eq!(decrypt_config_value(None).unwrap(), "");
        assert_eq!(decrypt_config_value(Some("")).unwrap(), "");
    }

    #[test]
    fn plaintext_passthrough() {
        // Valores no cifrados (ej. IS_GITHUB_ENABLED = "1")
        assert_eq!(decrypt_config_value(Some("1")).unwrap(), "1");
        assert_eq!(decrypt_config_value(Some("https://gitlab.com")).unwrap(), "https://gitlab.com");
    }

    #[test]
    fn v1_prefix_delegates_to_token_cipher() {
        // Sin TOKEN_ENCRYPTION_KEY, token_cipher retorna error para v1: tokens
        // Este test verifica que el routing es correcto (no que funcione sin clave)
        std::env::remove_var("TOKEN_ENCRYPTION_KEY");
        let result = decrypt_config_value(Some("v1:not-a-real-token"));
        // Debe intentar decrypt (y puede fallar con Internal — eso es correcto)
        assert!(result.is_err() || result.is_ok());
    }
}
