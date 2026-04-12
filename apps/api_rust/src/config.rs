// src/config.rs
use dotenvy::dotenv;
use std::env;

/// Equivalente a `plane/settings/common.py` en Django.
#[derive(Debug, Clone)]
pub struct Config {
    // Base de datos — construida en from_env() desde POSTGRES_*
    pub database_url:  String,

    // Redis — construida en from_env() desde REDIS_HOST / REDIS_PORT
    pub redis_url:     String,

    // Servidor
    pub host:          String,          // API_HOST — default "0.0.0.0"
    pub port:          u16,             // API_PORT — default 8000

    // App
    pub secret_key:    String,          // SECRET_KEY
    pub debug:         bool,            // DEBUG

    // S3/MinIO
    pub aws_s3_bucket: String,          // AWS_S3_BUCKET_NAME
    pub aws_endpoint:  String,          // AWS_S3_ENDPOINT_URL

    // Cookies
    pub cookie_domain: Option<String>,  // COOKIE_DOMAIN
    pub is_production: bool,            // derivado de DEBUG=0

    // CORS
    pub cors_origins:  Vec<String>,     // CORS_ORIGINS — comma-separated
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // dotenv es best-effort: en producción no hay .env file y eso es normal.
        match dotenv() {
            Ok(path) => tracing::debug!(".env loaded from {}", path.display()),
            Err(dotenvy::Error::Io(_)) => tracing::debug!("No .env file found, using environment variables directly"),
            Err(e) => tracing::warn!("dotenv error (non-fatal): {e}"),
        }

        let debug = env::var("DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            database_url:  build_database_url()?,
            redis_url:     build_redis_url(),
            host:          env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port:          env::var("API_PORT")
                               .unwrap_or_else(|_| "8000".into())
                               .parse()?,
            secret_key:    required("SECRET_KEY")?,
            debug,
            is_production: !debug,
            aws_s3_bucket: env::var("AWS_S3_BUCKET_NAME").unwrap_or_default(),
            aws_endpoint:  env::var("AWS_S3_ENDPOINT_URL").unwrap_or_default(),
            cookie_domain: env::var("COOKIE_DOMAIN").ok(),
            cors_origins:  env::var("CORS_ORIGINS")
                               .unwrap_or_default()
                               .split(',')
                               .map(|s| s.trim().to_string())
                               .filter(|s| !s.is_empty())
                               .collect(),
        })
    }
}

/// Construye `postgresql://user:pass@host:port/db` desde variables crudas.
fn build_database_url() -> anyhow::Result<String> {
    let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "plane".into());
    let pass = env::var("POSTGRES_PASSWORD").unwrap_or_default();
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".into());
    let db   = env::var("POSTGRES_DB").unwrap_or_else(|_| "plane".into());

    port.parse::<u16>()
        .map_err(|_| anyhow::anyhow!("POSTGRES_PORT inválido: '{port}' — debe ser un número entre 1 y 65535"))?;

    Ok(format!("postgresql://{user}:{pass}@{host}:{port}/{db}"))
}

/// Construye `redis://host:port` desde variables crudas.
fn build_redis_url() -> String {
    let host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".into());
    format!("redis://{host}:{port}")
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("Variable de entorno requerida: {key}"))
}
