// src/config.rs
use dotenvy::dotenv;
use std::env;

/// Equivalente a `plane/settings/common.py` en Django.
#[derive(Debug, Clone)]
pub struct Config {
    // Base de datos — construida en from_env() desde POSTGRES_*
    pub database_url: String,

    // Redis — construida en from_env() desde REDIS_HOST / REDIS_PORT
    pub redis_url: String,

    // Servidor
    pub host: String, // API_HOST — default "0.0.0.0"
    pub port: u16,    // API_PORT — default 8000

    // App
    pub secret_key: String,             // SECRET_KEY
    pub debug: bool,                    // DEBUG
    pub web_url: Option<String>,        // WEB_URL
    pub app_base_url: Option<String>,   // APP_BASE_URL
    pub app_base_path: Option<String>,  // APP_BASE_PATH
    pub space_base_url: Option<String>, // SPACE_BASE_URL
    pub space_base_path: Option<String>, // SPACE_BASE_PATH
    pub admin_base_url: Option<String>, // ADMIN_BASE_URL
    pub admin_base_path: Option<String>, // ADMIN_BASE_PATH

    // S3/MinIO
    pub aws_s3_bucket: String,       // AWS_S3_BUCKET_NAME
    pub aws_endpoint: String,        // AWS_S3_ENDPOINT_URL
    pub aws_access_key_id: String,   // AWS_ACCESS_KEY_ID
    pub aws_secret_access_key: String, // AWS_SECRET_ACCESS_KEY
    pub aws_region: String,          // AWS_REGION (default: us-east-1)
    pub use_minio: bool,             // USE_MINIO == "1"

    // LLM AI assistant
    pub llm_api_key: Option<String>,   // LLM_API_KEY
    pub llm_provider: String,          // LLM_PROVIDER (openai|anthropic|gemini)
    pub llm_model: Option<String>,     // LLM_MODEL

    // Unsplash
    pub unsplash_access_key: Option<String>, // UNSPLASH_ACCESS_KEY

    // Cookies
    pub cookie_domain: Option<String>, // COOKIE_DOMAIN
    pub is_production: bool,           // derivado de DEBUG=0
    pub session_cookie_age: i64,       // SESSION_COOKIE_AGE
    pub admin_session_cookie_age: i64, // ADMIN_SESSION_COOKIE_AGE

    // CORS
    pub cors_origins: Vec<String>, // CORS_ORIGINS — comma-separated

    // Email — equivalente a plane/settings/common.py EMAIL_*
    pub email_host: Option<String>,          // EMAIL_HOST
    pub email_port: u16,                     // EMAIL_PORT (default 587)
    pub email_host_user: Option<String>,     // EMAIL_HOST_USER
    pub email_host_password: Option<String>, // EMAIL_HOST_PASSWORD
    pub email_use_tls: bool,                 // EMAIL_USE_TLS == "1"
    pub email_use_ssl: bool,                 // EMAIL_USE_SSL == "1"
    pub email_from: String,                  // EMAIL_FROM

    // Limpieza periódica
    pub hard_delete_after_days: i64,        // HARD_DELETE_AFTER_DAYS (default 30)
    pub unuploaded_asset_delete_days: i64,  // UNUPLOADED_ASSET_DELETE_DAYS (default 7)
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // dotenv es best-effort: en producción no hay .env file y eso es normal.
        match dotenv() {
            Ok(path) => tracing::debug!(".env loaded from {}", path.display()),
            Err(dotenvy::Error::Io(_)) => {
                tracing::debug!("No .env file found, using environment variables directly")
            }
            Err(e) => tracing::warn!("dotenv error (non-fatal): {e}"),
        }

        let debug = env::var("DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            database_url: build_database_url()?,
            redis_url: build_redis_url(),
            host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("API_PORT")
                .unwrap_or_else(|_| "8000".into())
                .parse()?,
            secret_key: required("SECRET_KEY")?,
            debug,
            is_production: !debug,
            web_url: env::var("WEB_URL").ok(),
            app_base_url: env::var("APP_BASE_URL").ok(),
            app_base_path: env::var("APP_BASE_PATH").ok(),
            space_base_url: env::var("SPACE_BASE_URL").ok(),
            space_base_path: env::var("SPACE_BASE_PATH").ok(),
            admin_base_url: env::var("ADMIN_BASE_URL").ok(),
            admin_base_path: env::var("ADMIN_BASE_PATH").ok(),
            aws_s3_bucket: env::var("AWS_S3_BUCKET_NAME").unwrap_or_default(),
            aws_endpoint: env::var("AWS_S3_ENDPOINT_URL").unwrap_or_default(),
            aws_access_key_id: env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "access-key".into()),
            aws_secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_else(|_| "secret-key".into()),
            aws_region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into()),
            use_minio: env::var("USE_MINIO").map(|v| v == "1").unwrap_or(false),
            llm_api_key: env::var("LLM_API_KEY").ok(),
            llm_provider: env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".into()),
            llm_model: env::var("LLM_MODEL").ok(),
            unsplash_access_key: env::var("UNSPLASH_ACCESS_KEY").ok(),
            cookie_domain: env::var("COOKIE_DOMAIN").ok(),
            session_cookie_age: env::var("SESSION_COOKIE_AGE")
                .unwrap_or_else(|_| "604800".into())
                .parse()?,
            admin_session_cookie_age: env::var("ADMIN_SESSION_COOKIE_AGE")
                .unwrap_or_else(|_| "3600".into())
                .parse()?,
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            email_host: env::var("EMAIL_HOST").ok(),
            email_port: env::var("EMAIL_PORT")
                .unwrap_or_else(|_| "587".into())
                .parse()
                .unwrap_or(587),
            email_host_user: env::var("EMAIL_HOST_USER").ok(),
            email_host_password: env::var("EMAIL_HOST_PASSWORD").ok(),
            email_use_tls: env::var("EMAIL_USE_TLS").map(|v| v == "1").unwrap_or(false),
            email_use_ssl: env::var("EMAIL_USE_SSL").map(|v| v == "1").unwrap_or(false),
            email_from: env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "noreply@plane.so".into()),
            hard_delete_after_days: env::var("HARD_DELETE_AFTER_DAYS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            unuploaded_asset_delete_days: env::var("UNUPLOADED_ASSET_DELETE_DAYS")
                .unwrap_or_else(|_| "7".into())
                .parse()
                .unwrap_or(7),
        })
    }
}

/// Construye `postgresql://user:pass@host:port/db` desde variables crudas.
fn build_database_url() -> anyhow::Result<String> {
    let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "plane".into());
    let pass = env::var("POSTGRES_PASSWORD").unwrap_or_default();
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".into());
    let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "plane".into());

    port.parse::<u16>().map_err(|_| {
        anyhow::anyhow!("POSTGRES_PORT inválido: '{port}' — debe ser un número entre 1 y 65535")
    })?;

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
