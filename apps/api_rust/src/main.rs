// src/main.rs
use apalis::prelude::{Monitor, WorkerBuilder, WorkerFactoryFn};
use apalis_sql::postgres::PostgresStorage;
use fred::prelude::{
    Builder as RedisBuilder, ClientLike, Config as RedisConfig, Pool as RedisPool,
};
use migration::{Migrator, MigratorTrait};
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

/// Inicializa los workers de apalis (jobs en segundo plano).
/// Cada worker consume una cola de PostgreSQL de forma independiente.
async fn start_job_workers(state: AppState) -> anyhow::Result<()> {
    use jobs::{
        export::{ExportIssuesJob, handle_export_issues},
        github_sync::{GithubInitialSyncJob, handle_github_initial_sync},
        notifications::{IssueActivityNotificationJob, handle_issue_activity_notification},
        scheduled::{RunIssueAutomationJob, handle_run_issue_automation},
        workspace_seed::{WorkspaceSeedJob, handle_workspace_seed},
    };

    // Reuse shared PgPool from AppState — no new connection needed
    let pg_pool = state.pg_pool.clone();

    // setup() es asociado en PostgresStorage::<()> — se llama una vez para
    // crear/verificar la tabla de jobs en la base de datos.
    PostgresStorage::<()>::setup(&pg_pool).await?;

    // Crear storages tipados (no requieren setup individual)
    let github_storage: PostgresStorage<GithubInitialSyncJob> =
        PostgresStorage::new(pg_pool.clone());
    let notif_storage: PostgresStorage<IssueActivityNotificationJob> =
        PostgresStorage::new(pg_pool.clone());
    let export_storage: PostgresStorage<ExportIssuesJob> =
        PostgresStorage::new(pg_pool.clone());
    let scheduled_storage: PostgresStorage<RunIssueAutomationJob> =
        PostgresStorage::new(pg_pool.clone());
    let seed_storage: PostgresStorage<WorkspaceSeedJob> =
        PostgresStorage::new(pg_pool);

    // Monitor agrupa todos los workers y los ejecuta concurrentemente
    Monitor::new()
        .register(
            WorkerBuilder::new("github-initial-sync")
                .data(state.clone())
                .backend(github_storage)
                .build_fn(handle_github_initial_sync),
        )
        .register(
            WorkerBuilder::new("issue-activity-notification")
                .data(state.clone())
                .backend(notif_storage)
                .build_fn(handle_issue_activity_notification),
        )
        .register(
            WorkerBuilder::new("export-issues")
                .data(state.clone())
                .backend(export_storage)
                .build_fn(handle_export_issues),
        )
        .register(
            WorkerBuilder::new("issue-automation")
                .data(state.clone())
                .backend(scheduled_storage)
                .build_fn(handle_run_issue_automation),
        )
        .register(
            WorkerBuilder::new("workspace-seed")
                .data(state.clone())
                .backend(seed_storage)
                .build_fn(handle_workspace_seed),
        )
        .run()
        .await?;

    Ok(())
}


pub mod auth;
mod config;
pub mod entities;
mod error;
pub mod jobs;
mod routes;
pub mod utils;

use auth::rate_limit::RateLimitState;
use config::Config;

/// Estado global del servidor — Fase 1.
/// Fase 3 agrega: redis (fred::Pool), s3 (aws_sdk_s3::Client).
#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub db: sea_orm::DatabaseConnection,
    pub redis: RedisPool,
    pub config: Arc<Config>,
    pub rate_limit: Arc<RateLimitState>,
    /// Shared sqlx PgPool for apalis job enqueue — avoids creating a new
    /// connection per enqueue (was an anti-pattern in create_workspace).
    pub pg_pool: sqlx::PgPool,
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

    // Ejecutar migraciones SeaORM pendientes — reemplaza el servicio `migrator`
    // (manage.py migrate). Idempotente: no hace nada si el schema ya está al día.
    tracing::info!("Ejecutando migraciones pendientes...");
    Migrator::up(&db, None).await.map_err(|e| {
        tracing::error!("Fallo al ejecutar migraciones: {e}");
        e
    })?;
    tracing::info!("✅ Migraciones aplicadas");

    // 3b. sqlx PgPool — shared pool for apalis job enqueue + workers.
    // Avoids creating a new PgPool per job enqueue (anti-pattern).
    let pg_pool = sqlx::PgPool::connect(&config.database_url).await.map_err(|e| {
        tracing::error!("No se pudo crear sqlx PgPool: {e}");
        e
    })?;
    tracing::info!("✅ sqlx PgPool conectado");

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
        pg_pool: pg_pool.clone(),
        config: Arc::new(config.clone()),
        rate_limit: Arc::new(RateLimitState::default()),
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Error al construir reqwest::Client"),
    };

    // 4a. Startup equivalente a `register_instance` + `configure_instance` Django.
    //     Idempotente: no modifica filas ya existentes.
    tracing::info!("Ejecutando bootstrap de instancia...");
    utils::startup::ensure_instance_registered(&state).await.map_err(|e| {
        tracing::error!("Error en register_instance: {e:?}");
        anyhow::anyhow!("register_instance falló")
    })?;
    utils::startup::ensure_configurations_seeded(&state).await.map_err(|e| {
        tracing::error!("Error en configure_instance: {e:?}");
        anyhow::anyhow!("configure_instance falló")
    })?;
    tracing::info!("✅ Bootstrap de instancia completado");


    // 5a. Apalis job workers — inician en background, no bloquean el servidor
    let state_for_jobs = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_job_workers(state_for_jobs).await {
            tracing::error!(error = %e, "Error al iniciar workers de apalis");
        }
    });

    // 5b. Scheduler de tareas periódicas — reemplaza Celery beat
    let state_for_cron = state.clone();
    tokio::spawn(async move {
        jobs::cron::start_cron(state_for_cron).await;
    });

    // 5. Router
    //
    // NormalizePathLayer DEBE envolver al Router *desde fuera* usando
    // `tower::Layer::layer()`, NO `Router::layer()`.
    //
    // Razón: en axum 0.8 `Router::layer()` aplica middleware DESPUÉS del
    // path-matching (envuelve cada handler individualmente), por lo que un
    // request a `/api/workspace-slug-check/` ya falló el match contra
    // `/workspace-slug-check` antes de que la capa pueda strippear la barra
    // final → 404.
    //
    // Al envolver externamente, NormalizePathLayer intercepta el request
    // ANTES de que el Router haga routing, trimmeando la barra final
    // correctamente.
    use tower::Layer;
    use tower_http::normalize_path::NormalizePathLayer;

    let router = routes::build_router(state);
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    // 6. Servidor TCP
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(
        addr = %addr,
        scalar_ui = format!("http://{}:{}/api/docs",   config.host, config.port),
        health    = format!("http://{}:{}/api/health",  config.host, config.port),
        "🚀 Servidor listo"
    );

    // El tipo resultante de NormalizePathLayer::layer() es `Trim<Router>`,
    // que implementa `tower::Service<Request>` pero NO tiene
    // `into_make_service()` (método exclusivo de `axum::Router`).
    // `tower::make::Shared` convierte cualquier `Service + Clone` en un
    // `MakeService` compatible con `axum::serve`.
    axum::serve(
        listener,
        tower::make::Shared::new(app),
    )
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
