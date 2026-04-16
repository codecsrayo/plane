// src/utils/s3.rs
//! Helper para construir un cliente S3/MinIO y generar presigned URLs.
//!
//! El cliente se crea on-demand por request (el SDK hace pooling internamente).
//! Equivalente a `plane/settings/storage.py → S3Storage`.

use std::time::Duration;

use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{BehaviorVersion, Region},
    presigning::PresigningConfig,
    Client,
};

use crate::{config::Config, error::AppError};

/// Construye un cliente S3 configurado con credenciales, región y endpoint explícitos.
///
/// Equivalente a `boto3.client("s3", aws_access_key_id=..., aws_secret_access_key=...,
/// region_name=..., endpoint_url=..., config=Config(signature_version="s3v4"))` en Django.
///
/// - `force_path_style = true` siempre que haya un `aws_endpoint` personalizado — MinIO
///   self-hosted no soporta virtual-hosted style.
/// - La región cae a `us-east-1` si `AWS_REGION` no está configurada (igual que boto3).
pub fn build_s3_client(config: &Config) -> Client {
    let creds = Credentials::new(
        &config.aws_access_key_id,
        &config.aws_secret_access_key,
        None,        // session token
        None,        // expiry
        "plane-env", // provider name para logs
    );

    let region = Region::new(config.aws_region.clone());

    // force_path_style se activa cuando hay un endpoint personalizado (MinIO / compatible)
    // porque la mayoría de instalaciones self-hosted no soportan virtual-hosted style.
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

/// Genera una presigned URL de `PUT` para subir un objeto.
///
/// El cliente sube directamente al bucket; la API solo firma la URL.
/// TTL por defecto: 1 hora (para completar el upload desde el browser).
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

/// Genera una presigned URL de `GET` para acceder a un objeto.
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
