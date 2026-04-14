use std::borrow::Cow;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use openssl::{hash::MessageDigest, memcmp, pkey::PKey, sign::Signer};
use serde_json::{Map, Value};

pub const AUTH_BACKEND: &str = "django.contrib.auth.backends.ModelBackend";
const SESSION_SALT: &str = "django.contrib.sessions.SessionStore";
const AUTH_HASH_SALT: &str = "django.contrib.auth.models.AbstractBaseUser.get_session_auth_hash";
const SIGNER_SUFFIX: &str = "signer";

pub fn session_auth_hash(password_hash: &str, secret_key: &str) -> anyhow::Result<String> {
    hex_hmac_sha256(
        &format!("{AUTH_HASH_SALT}{SIGNER_SUFFIX}"),
        password_hash,
        secret_key,
    )
}

pub fn encode_session(data: &Map<String, Value>, secret_key: &str) -> anyhow::Result<String> {
    let serialized = serde_json::to_vec(data)?;
    let payload = maybe_compress(&serialized)?;
    let timestamped = format!(
        "{}:{}",
        payload,
        base62_encode(chrono::Utc::now().timestamp() as u64)
    );
    let signature = base64_hmac_sha256(
        &format!("{SESSION_SALT}{SIGNER_SUFFIX}"),
        &timestamped,
        secret_key,
    )?;

    Ok(format!("{timestamped}:{signature}"))
}

pub fn decode_session(session_data: &str, secret_key: &str) -> anyhow::Result<Map<String, Value>> {
    let mut parts = session_data.rsplitn(3, ':');
    let signature = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing session signature"))?;
    let timestamp = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing session timestamp"))?;
    let payload = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing session payload"))?;
    let signed = format!("{payload}:{timestamp}");

    let expected_signature = base64_hmac_sha256(
        &format!("{SESSION_SALT}{SIGNER_SUFFIX}"),
        &signed,
        secret_key,
    )?;
    if !memcmp::eq(signature.as_bytes(), expected_signature.as_bytes()) {
        anyhow::bail!("invalid session signature");
    }

    let decoded = if let Some(compressed) = payload.strip_prefix('.') {
        let raw = URL_SAFE_NO_PAD.decode(compressed)?;
        let mut decoder = ZlibDecoder::new(raw.as_slice());
        let mut bytes = Vec::new();
        std::io::copy(&mut decoder, &mut bytes)?;
        bytes
    } else {
        URL_SAFE_NO_PAD.decode(payload)?
    };

    Ok(serde_json::from_slice(&decoded)?)
}

fn maybe_compress(serialized: &[u8]) -> anyhow::Result<String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    std::io::Write::write_all(&mut encoder, serialized)?;
    let compressed = encoder.finish()?;

    let encoded = if compressed.len() < serialized.len().saturating_sub(1) {
        let mut value = String::from(".");
        value.push_str(&URL_SAFE_NO_PAD.encode(compressed));
        value
    } else {
        URL_SAFE_NO_PAD.encode(serialized)
    };

    Ok(encoded)
}

fn base64_hmac_sha256(salt: &str, value: &str, secret_key: &str) -> anyhow::Result<String> {
    let key = PKey::hmac(secret_key.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
    signer.update(salt.as_bytes())?;
    signer.update(value.as_bytes())?;
    Ok(URL_SAFE_NO_PAD.encode(signer.sign_to_vec()?))
}

fn hex_hmac_sha256(salt: &str, value: &str, secret_key: &str) -> anyhow::Result<String> {
    let key = PKey::hmac(secret_key.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
    signer.update(salt.as_bytes())?;
    signer.update(value.as_bytes())?;
    Ok(hex_encode(&signer.sign_to_vec()?))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn base62_encode(mut value: u64) -> String {
    const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    if value == 0 {
        return "0".to_owned();
    }

    let mut chars = Vec::new();
    while value > 0 {
        chars.push(ALPHABET[(value % 62) as usize] as char);
        value /= 62;
    }
    chars.iter().rev().collect()
}

pub fn email_display_name(email: &str) -> Cow<'_, str> {
    match email.split_once('@') {
        Some((local, _)) if !local.is_empty() => Cow::Borrowed(local),
        _ => Cow::Borrowed("user"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{decode_session, email_display_name, encode_session, session_auth_hash};

    #[test]
    fn session_roundtrip_preserves_payload() {
        let mut payload = serde_json::Map::new();
        payload.insert("_auth_user_id".to_owned(), json!("user-id"));
        payload.insert(
            "_auth_user_backend".to_owned(),
            json!("django.contrib.auth.backends.ModelBackend"),
        );
        payload.insert(
            "device_info".to_owned(),
            json!({"domain":"https://app.example.com"}),
        );

        let encoded = encode_session(&payload, "secret").expect("encode");
        let decoded = decode_session(&encoded, "secret").expect("decode");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn auth_hash_is_stable() {
        let hash = session_auth_hash("pbkdf2_sha256$123$salt$hash", "secret").expect("hash");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn display_name_uses_email_local_part() {
        assert_eq!(email_display_name("user@plane.so"), "user");
        assert_eq!(email_display_name("invalid"), "user");
    }
}
