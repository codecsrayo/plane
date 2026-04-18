//! Generación de S3 POST presigned (browser-based upload) con SigV4.
//!
//! Espejo funcional de `boto3.s3_client.generate_presigned_post(...)`, que Django usa
//! en `plane/settings/storage.py::S3Storage.generate_presigned_post`. El frontend de
//! Plane (`packages/services/src/file/helper.ts::generateFileUploadPayload`) arma un
//! `FormData` con los `fields` devueltos + el `file` y hace `POST multipart/form-data`
//! al `url`. Este módulo produce exactamente ese contrato.
//!
//! Por qué existe: `aws-sdk-s3` 1.x para Rust NO expone presigned POST
//! (ver awslabs/aws-sdk-rust#863, abierto desde 2023). Solo soporta presigned
//! GET/PUT/DELETE. El contrato con el frontend requiere POST multipart con policy
//! firmada, así que hay que calcular el SigV4 a mano.
//!
//! Referencias canónicas:
//! - Policy + string-to-sign: https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-authentication-HTTPPOST.html
//! - Derivación de signing key (igual que en el resto de SigV4):
//!   https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
//!
//! Diferencia crítica vs. SigV4 de headers/query:
//! - En presigned POST el **string-to-sign ES el policy base64** (no hay canonical request).
//! - El signing key se deriva igual: kDate → kRegion → kService → kSigning.

use std::collections::HashMap;

use axum::http::HeaderMap;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

use crate::{config::Config, error::AppError};

/// Resultado del `generate_presigned_post`: se serializa directo al frontend como
/// `{ url, fields }`, coincidiendo con `TFileSignedURLResponse.upload_data` en
/// `packages/types/src/file.ts`.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PresignedPost {
    pub url: String,
    pub fields: HashMap<String, String>,
}

/// Resuelve el endpoint **público** para firmar presigned URLs que serán usados
/// directamente por el navegador del usuario.
///
/// Mirror exacto del comportamiento de Django `S3Storage.__init__` cuando
/// `USE_MINIO=1` (`apps/api/plane/settings/storage.py:40-58`):
///
/// - Si `!use_minio` → devuelve `aws_endpoint` tal cual. En este modo se asume
///   AWS S3 real (o un MinIO expuesto con dominio público), y el endpoint del
///   config YA es la URL pública correcta.
/// - Si `use_minio` → construye `{scheme}://{host}` donde:
///     * `scheme` proviene de `WEB_URL` (ej. "https://plane.example.com"
///       → "https"). Fallback: "http".
///     * `host` proviene del header `X-Forwarded-Host` (si el proxy lo setea)
///       o del header `Host`. Fallback: `aws_endpoint` completo del config.
///
/// **Por qué esto es necesario**: en despliegues self-hosted con MinIO detrás
/// de un proxy (nginx), `AWS_S3_ENDPOINT_URL` típicamente apunta al hostname
/// interno de Docker (`http://plane-minio:9000`), que el navegador no puede
/// resolver y además violaría Mixed-Content en HTTPS. El proxy rutea
/// `https://<dominio-público>/uploads/...` hacia MinIO internamente; el
/// presigned debe estar firmado contra el dominio público para que el browser
/// pueda hacer el POST sin bloqueo.
pub fn public_s3_endpoint(config: &Config, headers: &HeaderMap) -> String {
    if !config.use_minio {
        return config.aws_endpoint.clone();
    }

    // Scheme: del WEB_URL configurado (https en prod típico).
    let scheme = config
        .web_url
        .as_deref()
        .and_then(scheme_from_url)
        .unwrap_or("http")
        .to_string();

    // Host: honrar X-Forwarded-Host primero (cuando hay proxy), luego Host.
    // Sólo aceptamos el primer valor y desechamos listas (CVE-2019-16782-like
    // rarity defensiva: un atacante podría inyectar coma-separados).
    let host_header = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| raw.split(',').next())
        .map(|s| s.trim().to_string());

    match host_header {
        Some(h) if !h.is_empty() => format!("{scheme}://{h}"),
        // Fallback conservador: si por alguna razón no hay Host header,
        // caemos al endpoint interno. Esto no rompe más de lo que ya estaba
        // roto, y evita emitir un presigned con URL vacía.
        _ => config.aws_endpoint.clone(),
    }
}

/// Extrae el scheme (`https`, `http`, …) de una URL sin traer el crate `url`.
/// Devuelve `None` si no hay `://` o si el prefijo está vacío.
fn scheme_from_url(url: &str) -> Option<&str> {
    let (scheme, _) = url.split_once("://")?;
    let scheme = scheme.trim();
    if scheme.is_empty() {
        None
    } else {
        Some(scheme)
    }
}

/// Genera un presigned POST para subir un objeto a S3/MinIO por multipart/form-data.
///
/// - `endpoint_url`: vacío ⇒ AWS S3 real, virtual-hosted style (`https://{bucket}.s3.{region}.amazonaws.com/`);
///   con valor ⇒ MinIO/compatible, path-style (`{endpoint}/{bucket}`).
/// - `file_size`: límite superior inclusivo para `content-length-range`. El mínimo se
///   deja en 1 byte para rechazar uploads vacíos (igual que Django).
/// - `ttl_secs`: ventana de validez del policy (`expiration`).
///
/// Conditions emitidas (igual que Django `S3Storage.generate_presigned_post`):
///   * `{"bucket": <bucket>}`
///   * `["content-length-range", 1, file_size]`
///   * `{"Content-Type": content_type}`
///   * `{"key": object_key}`
///
/// Se añaden las condiciones requeridas por SigV4 para que el servidor re-valide la firma:
///   * `{"x-amz-algorithm": "AWS4-HMAC-SHA256"}`
///   * `{"x-amz-credential": "<access_key>/<yyyymmdd>/<region>/s3/aws4_request"}`
///   * `{"x-amz-date": "<yyyymmddThhmmssZ>"}`
pub fn generate_presigned_post(
    bucket: &str,
    endpoint_url: &str,
    region: &str,
    access_key_id: &str,
    secret_access_key: &str,
    object_key: &str,
    content_type: &str,
    file_size: i64,
    ttl_secs: i64,
) -> Result<PresignedPost, AppError> {
    let now: DateTime<Utc> = Utc::now();
    let date_stamp = now.format("%Y%m%d").to_string();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let expiration = (now + Duration::seconds(ttl_secs))
        .format("%Y-%m-%dT%H:%M:%S.000Z")
        .to_string();

    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, region);
    let x_amz_credential = format!("{}/{}", access_key_id, credential_scope);

    // Ajustar file_size a un mínimo razonable. Django pasa `size_limit = min(FILE_SIZE_LIMIT, size)`;
    // si cae a 0 o negativo por un payload malformado, corregimos para no emitir un policy inválido.
    let max_bytes = file_size.max(1);

    // ── Policy document ───────────────────────────────────────────────────────
    let policy_doc = json!({
        "expiration": expiration,
        "conditions": [
            { "bucket": bucket },
            ["content-length-range", 1, max_bytes],
            { "Content-Type": content_type },
            { "key": object_key },
            { "x-amz-algorithm": "AWS4-HMAC-SHA256" },
            { "x-amz-credential": x_amz_credential },
            { "x-amz-date": amz_date },
        ],
    });

    let policy_json = serde_json::to_string(&policy_doc)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize policy: {e}")))?;
    // STANDARD (no URL-safe) coincide con boto3; S3 rechaza URL-safe aquí.
    let policy_b64 = STANDARD.encode(policy_json.as_bytes());

    // ── Signing key — HMAC-SHA256 en cadena: AWS4+secret → date → region → service → aws4_request
    let k_date = hmac_sha256::HMAC::mac(
        date_stamp.as_bytes(),
        format!("AWS4{}", secret_access_key).as_bytes(),
    );
    let k_region = hmac_sha256::HMAC::mac(region.as_bytes(), &k_date);
    let k_service = hmac_sha256::HMAC::mac(b"s3", &k_region);
    let k_signing = hmac_sha256::HMAC::mac(b"aws4_request", &k_service);

    // En presigned POST el string-to-sign es EL policy base64 directamente
    // (no el "AWS4-HMAC-SHA256\n<date>\n<scope>\n<hash>" típico de los otros flujos SigV4).
    let signature_bytes = hmac_sha256::HMAC::mac(policy_b64.as_bytes(), &k_signing);
    let signature_hex = hex_encode_lower(&signature_bytes);

    // ── URL al bucket ────────────────────────────────────────────────────────
    let url = if endpoint_url.trim().is_empty() {
        // AWS S3 real → virtual-hosted style. Incluye la region para evitar el redirect 307
        // que rompe CORS (ver boto3 issue #1982).
        format!("https://{bucket}.s3.{region}.amazonaws.com/")
    } else {
        // MinIO / compatible → path-style. El endpoint ya viene con scheme.
        let trimmed = endpoint_url.trim_end_matches('/');
        format!("{trimmed}/{bucket}")
    };

    // ── Fields que el cliente debe enviar en el multipart ────────────────────
    let mut fields: HashMap<String, String> = HashMap::new();
    fields.insert("key".to_string(), object_key.to_string());
    fields.insert("Content-Type".to_string(), content_type.to_string());
    fields.insert("x-amz-algorithm".to_string(), "AWS4-HMAC-SHA256".to_string());
    fields.insert("x-amz-credential".to_string(), x_amz_credential);
    fields.insert("x-amz-date".to_string(), amz_date);
    fields.insert("policy".to_string(), policy_b64);
    fields.insert("x-amz-signature".to_string(), signature_hex);

    Ok(PresignedPost { url, fields })
}

/// Hex-encoding de 32 bytes (salida HMAC-SHA256) en minúsculas. Evitamos añadir
/// un crate solo para esto; la función es trivial y libre de alocaciones intermedias.
fn hex_encode_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0F) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector oficial de AWS: deriva la signing key para los parámetros
    /// canónicos de la documentación SigV4 y verifica el hex.
    /// https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html
    #[test]
    fn signing_key_matches_aws_reference() {
        let secret = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
        let k_date = hmac_sha256::HMAC::mac(b"20150830", format!("AWS4{}", secret).as_bytes());
        let k_region = hmac_sha256::HMAC::mac(b"us-east-1", &k_date);
        let k_service = hmac_sha256::HMAC::mac(b"iam", &k_region);
        let k_signing = hmac_sha256::HMAC::mac(b"aws4_request", &k_service);
        // Valor documentado por AWS para estos inputs (sección "Derive signing key").
        assert_eq!(
            hex_encode_lower(&k_signing),
            "c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9"
        );
    }

    #[test]
    fn presigned_post_emits_expected_fields() {
        let p = generate_presigned_post(
            "my-bucket",
            "",
            "us-east-1",
            "AKIAEXAMPLE",
            "secretEXAMPLE",
            "ws-id/abc-cover.jpg",
            "image/jpeg",
            5 * 1024 * 1024,
            3600,
        )
        .expect("generates");

        assert_eq!(p.url, "https://my-bucket.s3.us-east-1.amazonaws.com/");
        for key in [
            "key",
            "Content-Type",
            "x-amz-algorithm",
            "x-amz-credential",
            "x-amz-date",
            "policy",
            "x-amz-signature",
        ] {
            assert!(p.fields.contains_key(key), "missing field {key}");
        }
        assert_eq!(p.fields["key"], "ws-id/abc-cover.jpg");
        assert_eq!(p.fields["Content-Type"], "image/jpeg");
        assert_eq!(p.fields["x-amz-algorithm"], "AWS4-HMAC-SHA256");
        assert!(p.fields["x-amz-credential"].contains("/us-east-1/s3/aws4_request"));
        // signature es hex de 64 chars
        assert_eq!(p.fields["x-amz-signature"].len(), 64);
        assert!(p.fields["x-amz-signature"]
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn presigned_post_path_style_for_minio() {
        let p = generate_presigned_post(
            "uploads",
            "http://minio:9000",
            "us-east-1",
            "AKIAEXAMPLE",
            "secretEXAMPLE",
            "key",
            "image/png",
            1024,
            600,
        )
        .expect("generates");
        assert_eq!(p.url, "http://minio:9000/uploads");
    }

    #[test]
    fn policy_embeds_conditions() {
        let p = generate_presigned_post(
            "b",
            "",
            "us-east-1",
            "ak",
            "sk",
            "k",
            "image/webp",
            2048,
            3600,
        )
        .expect("generates");
        let decoded = STANDARD.decode(&p.fields["policy"]).expect("base64");
        let parsed: serde_json::Value = serde_json::from_slice(&decoded).expect("json");
        let conds = parsed["conditions"].as_array().expect("array");

        // content-length-range
        assert!(conds.iter().any(|c| c.as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            == Some("content-length-range")));
        // bucket condition
        assert!(conds.iter().any(|c| c.get("bucket").is_some()));
        // key condition
        assert!(conds.iter().any(|c| c.get("key").is_some()));
    }
}
