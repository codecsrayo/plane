// src/utils/fernet.rs
//! Compatibility with Django's Fernet encryption.
//!
//! Uses `openssl` (already in Cargo.toml) to avoid version conflicts between
//! `digest 0.10` (hmac 0.12) and `digest 0.11` (sha2 0.11 of the project).
//!
//! ## Binary Fernet Format
//! ```text
//! base64url( VERSION[1] || TIME[8] || IV[16] || CIPHERTEXT[N] || HMAC[32] )
//! ```
//! - VERSION = 0x80, HMAC = HMAC-SHA256(version||time||iv||ciphertext)
//! - CIPHERTEXT = AES-128-CBC(key=enc_key, iv=IV, PKCS7(plaintext))
//! - derived = PBKDF2-HMAC-SHA256(SECRET_KEY, salt=b"salt", 100_000 iter, 32 bytes)
//! - signing_key = derived[0..16], enc_key = derived[16..32]

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use openssl::{
    hash::MessageDigest,
    pkcs5::pbkdf2_hmac,
    pkey::PKey,
    sign::Signer,
    symm::{Cipher, Crypter, Mode},
};

use crate::{error::AppError, utils::token_cipher::decrypt_token};

// ─── Public API ──────────────────────────────────────────────────────────────

/// Detects and decrypts values from `instance_configurations`.
///
/// | Value                          | Action                              |
/// |--------------------------------|-------------------------------------|
/// | `v1:<b64>`                     | AES-256-GCM (token_cipher)          |
/// | base64url whose byte[0] = 0x80  | Django Fernet                       |
/// | other                           | plain text — return as is           |
/// | None / ""                      | `""`                                |
pub fn decrypt_config_value(stored: Option<&str>) -> Result<String, AppError> {
    let value = match stored {
        None | Some("") => return Ok(String::new()),
        Some(v) => v,
    };
    if value.starts_with("v1:") {
        return decrypt_token(value);
    }
    if let Ok(raw) = URL_SAFE.decode(value) {
        if raw.first() == Some(&0x80) && raw.len() > 57 {
            return fernet_decrypt(&raw);
        }
    }
    Ok(value.to_owned())
}

/// Encrypts with AES-256-GCM (`v1:` prefix). Always Rust-native format.
pub fn encrypt_config_value(plaintext: &str) -> String {
    crate::utils::token_cipher::encrypt_token(plaintext)
}

// ─── Internal Fernet ───────────────────────────────────────────────────────────

fn derive_fernet_key(secret_key: &str) -> Result<[u8; 32], AppError> {
    let mut dk = [0u8; 32];
    pbkdf2_hmac(
        secret_key.as_bytes(),
        b"salt",
        100_000,
        MessageDigest::sha256(),
        &mut dk,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet derive_key: {e}")))?;
    Ok(dk)
}

/// Decrypts a Fernet token already decoded from base64url.
fn fernet_decrypt(raw: &[u8]) -> Result<String, AppError> {
    let secret_key = std::env::var("SECRET_KEY")
        .map_err(|_| AppError::Internal(anyhow::anyhow!("fernet: SECRET_KEY not configured")))?;

    if raw.len() < 57 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet: token too short ({} bytes)", raw.len()
        )));
    }
    if raw[0] != 0x80 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet: unknown version 0x{:02X}", raw[0]
        )));
    }

    let hmac_start  = raw.len() - 32;
    let signed_data = &raw[..hmac_start];
    let iv          = &raw[9..25];
    let ciphertext  = &raw[25..hmac_start];
    let stored_hmac = &raw[hmac_start..];

    let derived     = derive_fernet_key(&secret_key)?;
    let signing_key = &derived[..16];
    let enc_key     = &derived[16..];

    // ── HMAC-SHA256 verify ────────────────────────────────────────────────
    let pkey = PKey::hmac(signing_key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: PKey::hmac: {e}")))?;
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: Signer::new: {e}")))?;
    signer.update(signed_data)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: signer.update: {e}")))?;
    let computed = signer.sign_to_vec()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: sign_to_vec: {e}")))?;

    if computed.len() != stored_hmac.len() || !openssl::memcmp::eq(&computed, stored_hmac) {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet: invalid HMAC — corrupt data or incorrect SECRET_KEY"
        )));
    }

    // ── AES-128-CBC decrypt ───────────────────────────────────────────────
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(16) {
        return Err(AppError::Internal(anyhow::anyhow!(
            "fernet: invalid ciphertext length ({})", ciphertext.len()
        )));
    }
    let cipher = Cipher::aes_128_cbc();
    let mut dec = Crypter::new(cipher, Mode::Decrypt, enc_key, Some(iv))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: Crypter::new: {e}")))?;
    dec.pad(true);

    let mut buf = vec![0u8; ciphertext.len() + cipher.block_size()];
    let mut n = dec.update(ciphertext, &mut buf)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: update: {e}")))?;
    n += dec.finalize(&mut buf[n..])
        .map_err(|_| AppError::Internal(anyhow::anyhow!(
            "fernet: finalize — corrupt data or incorrect key"
        )))?;
    buf.truncate(n);

    String::from_utf8(buf)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("fernet: UTF-8: {e}")))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert_eq!(decrypt_config_value(None).unwrap(), "");
        assert_eq!(decrypt_config_value(Some("")).unwrap(), "");
    }

    #[test]
    fn plaintext_passthrough() {
        assert_eq!(decrypt_config_value(Some("1")).unwrap(), "1");
        assert_eq!(decrypt_config_value(Some("https://gitlab.com")).unwrap(), "https://gitlab.com");
    }
}
