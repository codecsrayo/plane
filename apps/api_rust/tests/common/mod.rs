//! Helpers compartidos entre los tests de integración.
//!
//! Reglas de diseño:
//!   1. **Aislamiento**: cada test recibe su propio `AppState` y su propio
//!      par de contenedores Postgres + Redis. No hay estado global.
//!   2. **Paridad con producción**: se usan los mismos builders (`build_router`
//!      + migraciones SeaORM + bootstrap de instancia) que el binario.
//!   3. **Sin red externa**: los contenedores corren vía `testcontainers`
//!      sobre el docker daemon local; nada toca internet.
//!   4. **Determinismo**: para proptest, las estrategias generan inputs
//!      acotados; los contenedores viven solo mientras el test los usa.
//!
//! Uso típico:
//! ```no_run
//! use crate::common::TestApp;
//!
//! #[tokio::test]
//! async fn my_test() {
//!     let app = TestApp::spawn().await;
//!     let response = app.get("/api/auth/get-csrf-token").await;
//!     assert_eq!(response.status, 200);
//! }
//! ```

use std::sync::Arc;

use api_rust::{
    auth::rate_limit::RateLimitState,
    config::Config,
    routes::build_router,
    utils::startup::{ensure_configurations_seeded, ensure_instance_registered},
    AppState,
};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use fred::prelude::{Builder as RedisBuilder, ClientLike, Config as RedisConfig};
use http_body_util::BodyExt;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use serde_json::Value;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use tower::ServiceExt;

/// App de test completa: contenedores + router listo para recibir requests.
///
/// Los campos `_pg` y `_redis` deben mantenerse vivos mientras se usa la app;
/// al dropearse, testcontainers detiene los contenedores.
pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    // Guardas de lifetime — no se leen, solo sostienen el contenedor.
    _pg: ContainerAsync<Postgres>,
    _redis: ContainerAsync<Redis>,
}

/// Respuesta HTTP capturada tras `oneshot`.
pub struct TestResponse {
    pub status: StatusCode,
    pub body: Vec<u8>,
}

impl TestResponse {
    /// Deserializa el cuerpo como JSON. Falla el test si el body no es JSON.
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|e| panic!("respuesta no es JSON válido ({e}): {:?}", String::from_utf8_lossy(&self.body)))
    }
}

impl TestApp {
    /// Crea un entorno aislado: Postgres + Redis + migraciones + bootstrap.
    ///
    /// Se hace vía `testcontainers` — requiere docker daemon en el host.
    pub async fn spawn() -> Self {
        // ── 1. Contenedores ──────────────────────────────────────────────
        let pg = Postgres::default()
            .with_db_name("plane_test")
            .with_user("plane")
            .with_password("plane")
            .start()
            .await
            .expect("postgres container no arrancó — ¿está docker corriendo?");
        let redis_ct = Redis::default()
            .start()
            .await
            .expect("redis container no arrancó — ¿está docker corriendo?");

        let pg_port = pg.get_host_port_ipv4(5432).await.expect("puerto pg");
        let redis_port = redis_ct.get_host_port_ipv4(6379).await.expect("puerto redis");
        let database_url = format!("postgresql://plane:plane@127.0.0.1:{pg_port}/plane_test");
        let redis_url = format!("redis://127.0.0.1:{redis_port}");

        // ── 2. Conexiones ────────────────────────────────────────────────
        let db = Database::connect(&database_url)
            .await
            .expect("sea_orm connect");
        Migrator::up(&db, None::<u32>)
            .await
            .expect("migraciones fallaron");

        let pg_pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("sqlx PgPool connect");

        let redis_config = RedisConfig::from_url(&redis_url).expect("redis config");
        let redis = RedisBuilder::from_config(redis_config)
            .build_pool(2)
            .expect("redis pool");
        let _handle = redis.connect();
        redis.wait_for_connect().await.expect("redis connect");

        // ── 3. Config de test ────────────────────────────────────────────
        let config = test_config(database_url.clone(), redis_url.clone());

        // ── 4. AppState + bootstrap (registra Instance + seed de configs) ─
        let state = AppState {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            db,
            redis,
            pg_pool,
            config: Arc::new(config),
            rate_limit: Arc::new(RateLimitState::default()),
        };

        ensure_instance_registered(&state)
            .await
            .expect("register_instance");
        ensure_configurations_seeded(&state)
            .await
            .expect("configure_instance");

        // ── 5. Router ────────────────────────────────────────────────────
        let router = build_router(state.clone());

        Self {
            router,
            state,
            _pg: pg,
            _redis: redis_ct,
        }
    }

    /// GET sin body, sin cookies.
    pub async fn get(&self, path: &str) -> TestResponse {
        self.request(Method::GET, path, None, &[]).await
    }

    /// POST con body JSON.
    pub async fn post_json(&self, path: &str, payload: &Value) -> TestResponse {
        let body = serde_json::to_vec(payload).expect("serializar payload");
        self.request(
            Method::POST,
            path,
            Some(body),
            &[("content-type", "application/json")],
        )
        .await
    }

    /// Ejecuta una request arbitraria contra el router.
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
        headers: &[(&str, &str)],
    ) -> TestResponse {
        let mut builder = Request::builder().method(method).uri(path);
        for (k, v) in headers {
            builder = builder.header(*k, *v);
        }
        let req = builder
            .body(body.map(Body::from).unwrap_or_else(Body::empty))
            .expect("request build");

        let res = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("router oneshot");

        let status = res.status();
        let bytes = res
            .into_body()
            .collect()
            .await
            .expect("leer body")
            .to_bytes();

        TestResponse {
            status,
            body: bytes.to_vec(),
        }
    }
}

/// Construye un `Config` válido para tests, apuntando a los contenedores.
/// Mantener en paralelo con `Config::from_env` para no desviarse en producción.
fn test_config(database_url: String, redis_url: String) -> Config {
    Config {
        database_url,
        redis_url,
        host: "127.0.0.1".into(),
        port: 0,
        secret_key: "test-secret-key-very-long-and-insecure".into(),
        debug: true,
        web_url: Some("http://localhost:3000".into()),
        app_base_url: Some("http://localhost:3000".into()),
        app_base_path: None,
        space_base_url: Some("http://localhost:3001".into()),
        space_base_path: None,
        admin_base_url: Some("http://localhost:3002".into()),
        admin_base_path: None,
        aws_s3_bucket: "test-bucket".into(),
        aws_endpoint: "http://127.0.0.1:9000".into(),
        aws_access_key_id: "test".into(),
        aws_secret_access_key: "test".into(),
        aws_region: "us-east-1".into(),
        use_minio: true,
        llm_api_key: None,
        llm_provider: "openai".into(),
        llm_model: None,
        unsplash_access_key: None,
        cookie_domain: None,
        is_production: false,
        session_cookie_age: 604_800,
        admin_session_cookie_age: 3600,
        cors_origins: vec![],
        email_host: None,
        email_port: 587,
        email_host_user: None,
        email_host_password: None,
        email_use_tls: false,
        email_use_ssl: false,
        email_from: "test@plane.local".into(),
        hard_delete_after_days: 30,
        unuploaded_asset_delete_days: 7,
    }
}
