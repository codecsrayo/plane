// src/utils/s3.rs
//! Helper para construir un cliente S3/MinIO y generar presigned URLs.
//!
//! El cliente se crea on-demand por request (el SDK hace pooling internamente).
//! Equivalente a `plane/settings/storage.py → S3Storage`.

use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;

use crate::{config::Config, error::AppError};

/// Construye un cliente S3 configurado con el endpoint y región de la instancia.
pub async fn build_s3_client(config: &Config) -> Client {
    let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(&config.aws_endpoint)
        .load()
        .await;
    Client::new(&s3_config)
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
