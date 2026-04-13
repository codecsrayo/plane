// src/main.rs
use fred::prelude::{
    Builder as RedisBuilder, ClientLike, Config as RedisConfig, Pool as RedisPool,
};
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod auth;
mod config;
pub mod entities;
mod error;
mod routes;
pub mod utils;

use auth::rate_limit::RateLimitState;
use config::Config;

/// Estado global del servidor — Fase 1.
/// Fase 3 agrega: redis (fred::Pool), s3 (aws_sdk_s3::Client), pg_pool (sqlx::PgPool).
#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis: RedisPool,
    pub config: Arc<Config>,
    pub rate_limit: Arc<RateLimitState>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Logging estructurado
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api_rust=debug,sea_orm=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🦀 Plane API Rust arrancando...");

    // 2. Configuración
    let config = Config::from_env().map_err(|e| {
        tracing::error!("Error de configuración: {e}");
        e
    })?;

    tracing::info!(port = config.port, host = %config.host, "Configuración cargada");

    // 3. PostgreSQL
    tracing::info!("Conectando a PostgreSQL...");
    let db = Database::connect(&config.database_url).await.map_err(|e| {
        tracing::error!("No se pudo conectar a PostgreSQL: {e}");
        e
    })?;

    tracing::info!("✅ PostgreSQL conectado");

    tracing::info!("Conectando a Redis...");
    let redis_config = RedisConfig::from_url(&config.redis_url).map_err(|e| {
        tracing::error!("No se pudo construir la configuración de Redis: {e}");
        anyhow::anyhow!(e.to_string())
    })?;
    let redis = RedisBuilder::from_config(redis_config)
        .build_pool(1)
        .map_err(|e| {
            tracing::error!("No se pudo crear el pool de Redis: {e}");
            anyhow::anyhow!(e.to_string())
        })?;
    let _redis_task = redis.connect();
    redis.wait_for_connect().await.map_err(|e| {
        tracing::error!("No se pudo conectar a Redis: {e}");
        anyhow::anyhow!(e.to_string())
    })?;
    tracing::info!("✅ Redis conectado");

    // 4. AppState
    let state = AppState {
        db,
        redis,
        config: Arc::new(config.clone()),
        rate_limit: Arc::new(RateLimitState::default()),
    };

    // 5. Router
    let app = routes::build_router(state);

    // 6. Servidor TCP
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(
        addr = %addr,
        scalar_ui = format!("http://{}:{}/api/docs",   config.host, config.port),
        health    = format!("http://{}:{}/api/health",  config.host, config.port),
        "🚀 Servidor listo"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
