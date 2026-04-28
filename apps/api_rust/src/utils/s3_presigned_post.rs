//! Generation of S3 POST presigned (browser-based upload) with SigV4.
//!
//! Functional mirror of `boto3.s3_client.generate_presigned_post(...)`, which Django uses
//! in `plane/settings/storage.py::S3Storage.generate_presigned_post`. The Plane
//! frontend (`packages/services/src/file/helper.ts::generateFileUploadPayload`) builds a
//! `FormData` with the returned `fields` + the `file` and performs a `POST multipart/form-data`
//! to the `url`. This module produces exactly that contract.
//!
//! Why it exists: `aws-sdk-s3` 1.x for Rust DOES NOT expose presigned POST
//! (see awslabs/aws-sdk-rust#863, open since 2023). It only supports presigned
//! GET/PUT/DELETE. The contract with the frontend requires POST multipart with a
//! signed policy, so SigV4 must be calculated manually.
//!
//! Canonical references:
//! - Policy + string-to-sign: https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-authentication-HTTPPOST.html
//! - Signing key derivation (same as the rest of SigV4):
//!   https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
//!
//! Critical difference vs. SigV4 for headers/query:
//! - In presigned POST the **string-to-sign IS the base64 policy** (no canonical request).
//! - The signing key is derived the same way: kDate → kRegion → kService → kSigning.

use std::collections::HashMap;

use axum::http::HeaderMap;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

use crate::{config::Config, error::AppError};

/// Result of `generate_presigned_post`: serialized directly to the frontend as
/// `{ url, fields }`, matching `TFileSignedURLResponse.upload_data` in
/// `packages/types/src/file.ts`.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PresignedPost {
    pub url: String,
    pub fields: HashMap<String, String>,
}

/// Resolves the **public** endpoint for signing presigned URLs that will be used
/// directly by the user's browser.
///
/// Exact mirror of Django's `S3Storage.__init__` behavior when
/// `USE_MINIO=1` (`apps/api/plane/settings/storage.py:40-58`):
///
/// - If `!use_minio` → returns `aws_endpoint` as is. In this mode, real AWS S3
///   (or a MinIO exposed with a public domain) is assumed, and the config
///   endpoint ALREADY is the correct public URL.
/// - If `use_minio` → builds `{scheme}://{host}` where:
///     * `scheme` comes from `WEB_URL` (e.g., "https://plane.example.com"
///       → "https"). Fallback: "http".
///     * `host` comes from the `X-Forwarded-Host` header (if the proxy sets it)
///       or the `Host` header. Fallback: complete `aws_endpoint` from config.
///
/// **Why this is necessary**: in self-hosted deployments with MinIO behind
/// a proxy (nginx), `AWS_S3_ENDPOINT_URL` typically points to the internal
/// Docker hostname (`http://plane-minio:9000`), which the browser cannot
/// resolve and would also violate Mixed-Content in HTTPS. The proxy routes
/// `https://<public-domain>/uploads/...` to MinIO internally; the
/// presigned must be signed against the public domain so that the browser
/// can perform the POST without blocking.
pub fn public_s3_endpoint(config: &Config, headers: &HeaderMap) -> String {
    if !config.use_minio {
        return config.aws_endpoint.clone();
    }

    // Scheme: from configured WEB_URL (typically https in prod).
    let scheme = config
        .web_url
        .as_deref()
        .and_then(scheme_from_url)
        .unwrap_or("http")
        .to_string();

    // Host: honor X-Forwarded-Host first (when proxied), then Host.
    // We only accept the first value and discard lists (CVE-2019-16782-like
    // defensive rarity: an attacker could inject comma-separated values).
    let host_header = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| raw.split(',').next())
        .map(|s| s.trim().to_string());

    match host_header {
        Some(h) if !h.is_empty() => format!("{scheme}://{h}"),
        // Conservative fallback: if for some reason there is no Host header,
        // we fall back to the internal endpoint. This doesn't break more than
        // what was already broken, and avoids issuing a presigned with an empty URL.
        _ => config.aws_endpoint.clone(),
    }
}

/// Extracts the scheme (`https`, `http`, …) from a URL without importing the `url` crate.
/// Returns `None` if there is no `://` or if the prefix is empty.
fn scheme_from_url(url: &str) -> Option<&str> {
    let (scheme, _) = url.split_once("://")?;
    let scheme = scheme.trim();
    if scheme.is_empty() {
        None
    } else {
        Some(scheme)
    }
}

/// Generates a presigned POST to upload an object to S3/MinIO via multipart/form-data.
///
/// - `endpoint_url`: empty ⇒ real AWS S3, virtual-hosted style (`https://{bucket}.s3.{region}.amazonaws.com/`);
///   with value ⇒ MinIO/compatible, path-style (`{endpoint}/{bucket}`).
/// - `file_size`: inclusive upper limit for `content-length-range`. The minimum is
///   left at 1 byte to reject empty uploads (same as Django).
/// - `ttl_secs`: validity window of the policy (`expiration`).
///
/// Emitted conditions (same as Django's `S3Storage.generate_presigned_post`):
///   * `{"bucket": <bucket>}`
///   * `["content-length-range", 1, file_size]`
///   * `{"Content-Type": content_type}`
///   * `{"key": object_key}`
///
/// Conditions required by SigV4 for the server to re-validate the signature are added:
///   * `{"x-amz-algorithm": "AWS4-HMAC-SHA256"}`
///   * `{"x-amz-credential": "<access_key>/<yyyymmdd>/<region>/s3/aws4_request"}`
///   * `{"x-amz-date": "<yyyymmddThhmmssZ>"}`
// SigV4 requires all these inputs separately: grouping them in a struct
// would be noise given that this function is a one-off implementation of the spec.
#[allow(clippy::too_many_arguments)]
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

    // Adjust file_size to a reasonable minimum. Django passes `size_limit = min(FILE_SIZE_LIMIT, size)`;
    // if it falls to 0 or negative due to a malformed payload, we correct it to avoid emitting an invalid policy.
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
    // STANDARD (not URL-safe) matches boto3; S3 rejects URL-safe here.
    let policy_b64 = STANDARD.encode(policy_json.as_bytes());

    // ── Signing key — Chained HMAC-SHA256: AWS4+secret → date → region → service → aws4_request
    let k_date = hmac_sha256::HMAC::mac(
        date_stamp.as_bytes(),
        format!("AWS4{}", secret_access_key).as_bytes(),
    );
    let k_region = hmac_sha256::HMAC::mac(region.as_bytes(), k_date);
    let k_service = hmac_sha256::HMAC::mac(b"s3", k_region);
    let k_signing = hmac_sha256::HMAC::mac(b"aws4_request", k_service);

    // In presigned POST the string-to-sign is the base64 policy directly
    // (not the typical "AWS4-HMAC-SHA256\n<date>\n<scope>\n<hash>" of other SigV4 flows).
    let signature_bytes = hmac_sha256::HMAC::mac(policy_b64.as_bytes(), k_signing);
    let signature_hex = hex_encode_lower(&signature_bytes);

    // ── URL to the bucket ────────────────────────────────────────────────────────
    let url = if endpoint_url.trim().is_empty() {
        // Real AWS S3 → virtual-hosted style. Includes region to avoid 307 redirect
        // that breaks CORS (see boto3 issue #1982).
        format!("https://{bucket}.s3.{region}.amazonaws.com/")
    } else {
        // MinIO / compatible → path-style. Endpoint already comes with scheme.
        let trimmed = endpoint_url.trim_end_matches('/');
        format!("{trimmed}/{bucket}")
    };

    // ── Fields the client must send in the multipart ────────────────────
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

/// Hex-encoding of 32 bytes (HMAC-SHA256 output) in lowercase. We avoid adding
/// a crate just for this; the function is trivial and free of intermediate allocations.
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

    /// AWS official test vector: derives the signing key for the canonical
    /// parameters of the SigV4 documentation and verifies the hex.
    /// https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html
    #[test]
    fn signing_key_matches_aws_reference() {
        let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
        let k_date = hmac_sha256::HMAC::mac(b"20150830", format!("AWS4{}", secret).as_bytes());
        let k_region = hmac_sha256::HMAC::mac(b"us-east-1", k_date);
        let k_service = hmac_sha256::HMAC::mac(b"iam", k_region);
        let k_signing = hmac_sha256::HMAC::mac(b"aws4_request", k_service);
        // Value documented by AWS for these inputs (section "Derive signing key").
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
        // signature is hex of 64 chars
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
