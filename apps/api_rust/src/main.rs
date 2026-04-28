// src/main.rs
use apalis::prelude::{Monitor, WorkerBuilder, WorkerFactoryFn};
use apalis_sql::postgres::PostgresStorage;
use fred::prelude::{Builder as RedisBuilder, ClientLike, Config as RedisConfig};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Waits for SIGINT (Ctrl-C) or SIGTERM before starting graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not install Ctrl-C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Could not install SIGTERM handler")
            .recv()
            .await;
    };

    // On non-Unix platforms (Windows) SIGTERM does not exist; a future that
    // never resolves is used so only Ctrl-C is effective.
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c  => tracing::info!("SIGINT received, starting graceful shutdown..."),
        _ = sigterm => tracing::info!("SIGTERM received, starting graceful shutdown..."),
    }
}

/// Initializes apalis workers (background jobs).
/// Each worker consumes a PostgreSQL queue independently.
async fn start_job_workers(state: AppState) -> anyhow::Result<()> {
    use jobs::{
        export::{ExportIssuesJob, handle_export_issues},
        github_sync::{GithubInitialSyncJob, handle_github_initial_sync},
        notifications::{IssueActivityNotificationJob, handle_issue_activity_notification},
        scheduled::{RunIssueAutomationJob, handle_run_issue_automation},
        webhook_delivery::{DeliverWebhookJob, handle_deliver_webhook},
        workspace_seed::{WorkspaceSeedJob, handle_workspace_seed},
    };

    // Reuse shared PgPool from AppState — no new connection needed
    let pg_pool = state.pg_pool.clone();

    // setup() is associated in PostgresStorage::<()> — called once to
    // create/verify the jobs table in the database.
    PostgresStorage::<()>::setup(&pg_pool).await?;

    // Create typed storages (do not require individual setup)
    let github_storage: PostgresStorage<GithubInitialSyncJob> =
        PostgresStorage::new(pg_pool.clone());
    let notif_storage: PostgresStorage<IssueActivityNotificationJob> =
        PostgresStorage::new(pg_pool.clone());
    let export_storage: PostgresStorage<ExportIssuesJob> =
        PostgresStorage::new(pg_pool.clone());
    let scheduled_storage: PostgresStorage<RunIssueAutomationJob> =
        PostgresStorage::new(pg_pool.clone());
    let webhook_storage: PostgresStorage<DeliverWebhookJob> =
        PostgresStorage::new(pg_pool.clone());
    let seed_storage: PostgresStorage<WorkspaceSeedJob> =
        PostgresStorage::new(pg_pool);

    // Monitor groups all workers and runs them concurrently
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
            WorkerBuilder::new("webhook-delivery")
                .data(state.clone())
                .backend(webhook_storage)
                .build_fn(handle_deliver_webhook),
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


// Modules and AppState now live in `src/lib.rs` so that
// integration tests under `tests/` can reuse them via `use api_rust::…`.
use api_rust::{auth::rate_limit::RateLimitState, config::Config, jobs, routes, utils, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Structured logging
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api_rust=debug,sea_orm=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🦀 Plane API Rust starting up...");

    // 2. Configuration
    let config = Config::from_env().map_err(|e| {
        tracing::error!("Configuration error: {e}");
        e
    })?;

    tracing::info!(port = config.port, host = %config.host, "Configuration loaded");

    // 3. PostgreSQL
    tracing::info!("Connecting to PostgreSQL...");
    let db = Database::connect(&config.database_url).await.map_err(|e| {
        tracing::error!("Could not connect to PostgreSQL: {e}");
        e
    })?;

    tracing::info!("✅ PostgreSQL connected");

    // Run pending SeaORM migrations — replaces the `migrator` service
    // (manage.py migrate). Idempotent: does nothing if the schema is already up to date.
    tracing::info!("Running pending migrations...");
    Migrator::up(&db, None::<u32>).await.map_err(|e| {
        tracing::error!("Failed to run migrations: {e}");
        e
    })?;
    tracing::info!("✅ Migrations applied");

    // 3b. sqlx PgPool — shared pool for apalis job enqueue + workers.
    // Avoids creating a new PgPool per job enqueue (anti-pattern).
    let pg_pool = sqlx::PgPool::connect(&config.database_url).await.map_err(|e| {
        tracing::error!("Could not create sqlx PgPool: {e}");
        e
    })?;
    tracing::info!("✅ sqlx PgPool connected");

    tracing::info!("Connecting to Redis...");
    let redis_config = RedisConfig::from_url(&config.redis_url).map_err(|e| {
        tracing::error!("Could not build Redis configuration: {e}");
        anyhow::anyhow!(e.to_string())
    })?;
    let redis = RedisBuilder::from_config(redis_config)
        .build_pool(4)
        .map_err(|e| {
            tracing::error!("Could not create Redis pool: {e}");
            anyhow::anyhow!(e.to_string())
        })?;

    // JoinHandles are kept to perform an ordered join during shutdown.
    // RedisPool is Clone (internal Arc): cloned before moving to AppState
    // to keep a reference available for graceful shutdown.
    let redis_tasks = redis.connect();
    redis.wait_for_connect().await.map_err(|e| {
        tracing::error!("Could not connect to Redis: {e}");
        anyhow::anyhow!(e.to_string())
    })?;
    tracing::info!("✅ Redis connected");

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
            .expect("Error building reqwest::Client"),
    };

    // 4a. Startup equivalent to Django's `register_instance` + `configure_instance`.
    //     Idempotent: does not modify existing rows.
    tracing::info!("Running instance bootstrap...");
    utils::startup::ensure_instance_registered(&state).await.map_err(|e| {
        tracing::error!("Error in register_instance: {e:?}");
        anyhow::anyhow!("register_instance failed")
    })?;
    utils::startup::ensure_configurations_seeded(&state).await.map_err(|e| {
        tracing::error!("Error in configure_instance: {e:?}");
        anyhow::anyhow!("configure_instance failed")
    })?;
    tracing::info!("✅ Instance bootstrap completed");


    // 5a. Apalis job workers — start in background, do not block the server
    let state_for_jobs = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_job_workers(state_for_jobs).await {
            tracing::error!(error = %e, "Error starting apalis workers");
        }
    });

    // 5b. Periodic task scheduler — replaces Celery beat
    let state_for_cron = state.clone();
    tokio::spawn(async move {
        jobs::cron::start_cron(state_for_cron).await;
    });

    // 5. Router
    //
    // NormalizePathLayer MUST wrap the Router *from outside* using
    // `tower::Layer::layer()`, NOT `Router::layer()`.
    //
    // Reason: in axum 0.8 `Router::layer()` applies middleware AFTER
    // path-matching (wraps each handler individually), so a
    // request to `/api/workspace-slug-check/` already failed to match
    // `/workspace-slug-check` before the layer could strip the trailing
    // slash → 404.
    //
    // By wrapping externally, NormalizePathLayer intercepts the request
    // BEFORE the Router does routing, correctly trimming the trailing slash.
    use tower::Layer;
    use tower_http::normalize_path::NormalizePathLayer;

    let router = routes::build_router(state);
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    // 6. TCP Server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(
        addr = %addr,
        scalar_ui = format!("http://{}:{}/api/docs",   config.host, config.port),
        health    = format!("http://{}:{}/api/health",  config.host, config.port),
        "🚀 Server ready"
    );

    // The resulting type of NormalizePathLayer::layer() is `Trim<Router>`,
    // which implements `tower::Service<Request>` but DOES NOT have
    // `into_make_service()` (method exclusive to `axum::Router`).
    // `tower::make::Shared` converts any `Service + Clone` into a
    // `MakeService` compatible with `axum::serve`.
    axum::serve(
        listener,
        tower::make::Shared::new(app),
    )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // 7. Ordered Redis shutdown
    tracing::info!("Closing Redis connections...");
    if let Err(e) = redis_for_shutdown.quit().await {
        tracing::warn!("Error sending QUIT to Redis: {e}");
    }
    // redis.connect() returns a single JoinHandle, not a Vec.
    if let Err(e) = redis_tasks.await {
        tracing::warn!("Error joining Redis task: {e}");
    }
    tracing::info!("✅ Redis closed successfully");

    Ok(())
}
