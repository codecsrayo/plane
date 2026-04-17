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

/// Cliente S3 específico para **generar presigned URLs públicas** cuando se
/// usa MinIO self-hosted.
///
/// Paridad Django (apps/api/plane/bgtasks/export_task.py:49-79): con MinIO,
/// Django instancia dos clientes distintos:
///   - uno con `AWS_S3_ENDPOINT_URL` (interno, ej. `http://plane-minio:9000`)
///     para **subir** el archivo.
///   - otro con `f"{AWS_S3_URL_PROTOCOL}//{AWS_S3_CUSTOM_DOMAIN sin /uploads}/"`
///     —derivado de `WEB_URL`— para **firmar** la URL que retorna al browser.
///
/// Sin esto, el presigned URL apunta al hostname Docker interno
/// (`http://plane-minio:9000/...`) que el browser no puede resolver.
///
/// Fallback: si `USE_MINIO=false` o `WEB_URL` no está seteado, usamos el
/// cliente normal — el endpoint público coincide con el de upload (caso S3
/// administrado, o deploys donde el MinIO es accesible directo por el
/// browser).
pub fn build_s3_presign_client(config: &Config) -> Client {
    // Sólo aplica la derivación public-domain cuando MinIO está activo Y
    // tenemos WEB_URL — es la misma guardia que el if de common.py:253.
    let public_endpoint = if config.use_minio {
        config.web_url.as_deref().and_then(parse_web_url_origin)
    } else {
        None
    };

    // Sin endpoint público derivable → mismo cliente que upload.
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
        // Path-style obligatorio: el bucket va como primer segmento del
        // path (`https://<host>/<bucket>/<key>`), nunca como subdominio.
        .force_path_style(true)
        .build();

    Client::from_conf(s3_conf)
}

/// Extrae `<scheme>://<host>[:<port>]/` de una URL como `https://plane.example.com/foo/bar`.
///
/// Retorna `None` si la URL no tiene scheme o host válido. No dependemos del
/// crate `url` porque el resto del codebase no lo usa y no justifica sumarlo
/// sólo para este helper — el formato de `WEB_URL` es suficientemente
/// restringido (`http(s)://host[:port][/path]`).
fn parse_web_url_origin(web_url: &str) -> Option<String> {
    let (scheme, rest) = web_url.split_once("://")?;
    if scheme.is_empty() {
        return None;
    }
    // `rest` = `host[:port][/path...]`. Nos quedamos con el primer segmento.
    let host_and_port = rest.split('/').next().unwrap_or("");
    if host_and_port.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{host_and_port}/"))
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

/// Genera una presigned URL de `PUT` para subir un objeto.
///
/// El cliente sube directamente al bucket; la API solo firma la URL.
/// TTL por defecto: 1 hora (para completar el upload desde el browser).
///
/// NOTA: Actualmente no tiene callers en el código: el frontend
/// (`packages/services/src/file/file-upload.service.ts`) espera el contrato
/// de Django/boto3 que es **POST multipart/form-data** con `policy` firmada,
/// no `PUT` directo. El flujo vigente usa
/// [`crate::utils::s3_presigned_post::generate_presigned_post`].
/// Se conserva este helper por si algún flujo server-to-server futuro (ej.
/// carga server-side de archivos desde un worker) requiere un PUT simple.
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

/// Genera una presigned URL de `GET` para acceder a un objeto.
///
/// Sin callers actuales; se conserva como helper para futuros endpoints que
/// necesiten servir URLs de descarga con expiración.
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
