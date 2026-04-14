// src/main.rs
use fred::prelude::{
    Builder as RedisBuilder, ClientLike, Config as RedisConfig, Pool as RedisPool,
};
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Espera SIGINT (Ctrl-C) o SIGTERM antes de iniciar el shutdown graceful.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("No se pudo instalar el handler de Ctrl-C");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("No se pudo instalar el handler de SIGTERM")
            .recv()
            .await;
    };

    // En plataformas no-Unix (Windows) SIGTERM no existe; se usa un future que
    // nunca resuelve para que sólo Ctrl-C sea efectivo.
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c  => tracing::info!("SIGINT recibido, iniciando shutdown graceful..."),
        _ = sigterm => tracing::info!("SIGTERM recibido, iniciando shutdown graceful..."),
    }
}

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
    pub http: reqwest::Client,
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
        .build_pool(4)
        .map_err(|e| {
            tracing::error!("No se pudo crear el pool de Redis: {e}");
            anyhow::anyhow!(e.to_string())
        })?;

    // Se guardan los JoinHandles para hacer join ordenado en shutdown.
    // RedisPool es Clone (Arc interno): se clona antes de moverlo al AppState
    // para conservar una referencia disponible en el cierre graceful.
    let redis_tasks = redis.connect();
    redis.wait_for_connect().await.map_err(|e| {
        tracing::error!("No se pudo conectar a Redis: {e}");
        anyhow::anyhow!(e.to_string())
    })?;
    tracing::info!("✅ Redis conectado");

    let redis_for_shutdown = redis.clone();

    // 4. AppState
    let state = AppState {
        db,
        redis,
        config: Arc::new(config.clone()),
        rate_limit: Arc::new(RateLimitState::default()),
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Error al construir reqwest::Client"),
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

    // El servidor drena conexiones activas antes de retornar.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // 7. Shutdown ordenado de Redis
    tracing::info!("Cerrando conexiones Redis...");
    if let Err(e) = redis_for_shutdown.quit().await {
        tracing::warn!("Error al enviar QUIT a Redis: {e}");
    }
    // redis.connect() retorna un único JoinHandle, no un Vec.
    if let Err(e) = redis_tasks.await {
        tracing::warn!("Error al unir tarea de Redis: {e}");
    }
    tracing::info!("✅ Redis cerrado correctamente");

    Ok(())
}
