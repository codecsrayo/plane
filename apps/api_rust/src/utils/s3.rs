// src/utils/s3.rs
//! Helper to build an S3/MinIO client and generate presigned URLs.
//!
//! The client is created on-demand per request (the SDK handles pooling internally).
//! Equivalente a `plane/settings/storage.py → S3Storage`.

use std::time::Duration;

use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{BehaviorVersion, Region},
    presigning::PresigningConfig,
    Client,
};

use crate::{config::Config, error::AppError};

/// Builds an S3 client configured with explicit credentials, region, and endpoint.
///
/// Equivalent to `boto3.client("s3", aws_access_key_id=..., aws_secret_access_key=...,
/// region_name=..., endpoint_url=..., config=Config(signature_version="s3v4"))` in Django.
///
/// - `force_path_style = true` whenever there is a custom `aws_endpoint` — self-hosted
///   MinIO does not support virtual-hosted style.
/// - The region defaults to `us-east-1` if `AWS_REGION` is not configured (same as boto3).
pub fn build_s3_client(config: &Config) -> Client {
    let creds = Credentials::new(
        &config.aws_access_key_id,
        &config.aws_secret_access_key,
        None,        // session token
        None,        // expiry
        "plane-env", // provider name for logs
    );

    let region = Region::new(config.aws_region.clone());

    // force_path_style is activated when there is a custom endpoint (MinIO / compatible)
    // because most self-hosted installations do not support virtual-hosted style.
    let force_path = config.use_minio || !config.aws_endpoint.is_empty();

    let s3_conf = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .credentials_provider(creds)
        .region(region)
        .endpoint_url(&config.aws_endpoint)
        .force_path_style(force_path)
        .build();

    Client::from_conf(s3_conf)
}

/// S3 client specifically for **generating public presigned URLs** when
/// using self-hosted MinIO.
///
/// Django parity (apps/api/plane/bgtasks/export_task.py:49-79): with MinIO,
/// Django instantiates two different clients:
///   - one with `AWS_S3_ENDPOINT_URL` (internal, e.g. `http://plane-minio:9000`)
///     to **upload** the file.
///   - another with `f"{AWS_S3_URL_PROTOCOL}//{AWS_S3_CUSTOM_DOMAIN without /uploads}/"`
///     —derived from `WEB_URL`— to **sign** the URL returned to the browser.
///
/// Without this, the presigned URL points to the internal Docker hostname
/// (`http://plane-minio:9000/...`) which the browser cannot resolve.
///
/// Fallback: if `USE_MINIO=false` or `WEB_URL` is not set, we use the
/// normal client — the public endpoint matches the upload one (managed S3
/// case, or deployments where MinIO is directly accessible by the
/// browser).
pub fn build_s3_presign_client(config: &Config) -> Client {
    // Only apply public-domain derivation when MinIO is active AND
    // we have WEB_URL — same guard as the if in common.py:253.
    let public_endpoint = if config.use_minio {
        config.web_url.as_deref().and_then(parse_web_url_origin)
    } else {
        None
    };

    // Without derivable public endpoint → same client as upload.
    let Some(endpoint) = public_endpoint else {
        return build_s3_client(config);
    };

    let creds = Credentials::new(
        &config.aws_access_key_id,
        &config.aws_secret_access_key,
        None,
        None,
        "plane-env-presign",
    );

    let s3_conf = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .credentials_provider(creds)
        .region(Region::new(config.aws_region.clone()))
        .endpoint_url(endpoint)
        // Mandatory path-style: the bucket goes as the first segment of the
        // path (`https://<host>/<bucket>/<key>`), never as a subdomain.
        .force_path_style(true)
        .build();

    Client::from_conf(s3_conf)
}

/// Extracts `<scheme>://<host>[:<port>]/` from a URL like `https://plane.example.com/foo/bar`.
///
/// Returns `None` if the URL has no valid scheme or host. We do not depend on the
/// `url` crate because the rest of the codebase does not use it and it doesn't justify adding it
/// just for this helper — the format of `WEB_URL` is sufficiently
/// restricted (`http(s)://host[:port][/path]`).
fn parse_web_url_origin(web_url: &str) -> Option<String> {
    let (scheme, rest) = web_url.split_once("://")?;
    if scheme.is_empty() {
        return None;
    }
    // `rest` = `host[:port][/path...]`. We keep the first segment.
    let host_and_port = rest.split('/').next().unwrap_or("");
    if host_and_port.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{host_and_port}/"))
}

/// Generates a presigned `PUT` URL for uploading an object.
///
/// The client uploads directly to the bucket; the API only signs the URL.
/// Default TTL: 1 hour (to complete the upload from the browser).
///
/// NOTE: Currently has no callers in the code: the frontend
/// (`packages/services/src/file/file-upload.service.ts`) expects the
/// Django/boto3 contract which is **POST multipart/form-data** with signed `policy`,
/// not direct `PUT`. The current flow uses
/// [`crate::utils::s3_presigned_post::generate_presigned_post`].
/// This helper is preserved in case any future server-to-server flow (e.g.
/// server-side file upload from a worker) requires a simple PUT.
#[allow(dead_code)]
pub async fn presigned_put_url(
    client: &Client,
    bucket: &str,
    key: &str,
    content_type: &str,
    ttl_secs: u64,
) -> Result<String, AppError> {
    let config = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("presigning config error: {e}")))?;

    let req = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .presigned(config)
        .await
        .map_err(|e| {
            tracing::error!("presigned_put_url error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to generate upload URL"))
        })?;

    Ok(req.uri().to_string())
}

/// Copies an object within the same bucket (server-side copy).
///
/// Mirror of `S3Storage.copy_object` in Django (`plane/settings/storage.py`).
/// Used by `DuplicateAssetEndpoint` to duplicate assets without re-upload.
pub async fn copy_object(
    client: &Client,
    bucket: &str,
    source_key: &str,
    dest_key: &str,
) -> Result<(), AppError> {
    let copy_source = format!("{}/{}", bucket, source_key);
    client
        .copy_object()
        .bucket(bucket)
        .copy_source(&copy_source)
        .key(dest_key)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("copy_object error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to copy S3 object"))
        })?;
    Ok(())
}

/// Generates a presigned `GET` URL to access an object.
///
/// Used by download endpoints that generate a redirect with
/// `content-disposition: attachment`.
#[allow(dead_code)]
pub async fn presigned_get_url(
    client: &Client,
    bucket: &str,
    key: &str,
    ttl_secs: u64,
) -> Result<String, AppError> {
    let config = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("presigning config error: {e}")))?;

    let req = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(config)
        .await
        .map_err(|e| {
            tracing::error!("presigned_get_url error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to generate download URL"))
        })?;

    Ok(req.uri().to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_web_url_origin;

    #[test]
    fn origin_strips_path_and_keeps_port() {
        assert_eq!(
            parse_web_url_origin("https://plane.example.com/app").as_deref(),
            Some("https://plane.example.com/")
        );
        assert_eq!(
            parse_web_url_origin("http://localhost:3000/").as_deref(),
            Some("http://localhost:3000/")
        );
        assert_eq!(
            parse_web_url_origin("https://plane.codecsrayo.com").as_deref(),
            Some("https://plane.codecsrayo.com/")
        );
    }

    #[test]
    fn origin_rejects_malformed() {
        assert!(parse_web_url_origin("plane.example.com").is_none());
        assert!(parse_web_url_origin("://no-scheme.com").is_none());
        assert!(parse_web_url_origin("https://").is_none());
    }
}
