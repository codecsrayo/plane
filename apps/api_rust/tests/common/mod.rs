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

// Cada binario de tests/*.rs compila este módulo de forma independiente y
// usa solo un subconjunto de los helpers; el resto aparece como `dead_code`
// aunque sí esté en uso desde otros binarios. Es el patrón estándar para
// `tests/common/mod.rs` — silenciar a nivel de módulo evita decorar cada
// helper individualmente.
#![allow(dead_code)]

use std::sync::Arc;

use api_rust::{
    auth::rate_limit::RateLimitState,
    config::Config,
    entities::{api_tokens, instances, project_identifiers, project_members, projects, users, workspace_members, workspaces},
    routes::build_router,
    utils::startup::{ensure_configurations_seeded, ensure_instance_registered},
    AppState,
};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use chrono::Utc;
use fred::prelude::{Builder as RedisBuilder, ClientLike, Config as RedisConfig};
use http_body_util::BodyExt;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database};
use serde_json::Value;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use tower::ServiceExt;
use uuid::Uuid;

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
        // Imagen alineada con docker-compose.yml de producción.
        // Postgres < 12 no soporta el GUC `default_table_access_method`
        // que aparece en migration/src/sql/baseline.sql (dump de PG 15.7).
        // `.with_tag()` (ImageExt) consume `Postgres` y devuelve
        // `ContainerRequest<Postgres>`, así que va AL FINAL — después de
        // los métodos propios de la imagen (with_db_name/user/password).
        let pg = Postgres::default()
            .with_db_name("plane_test")
            .with_user("plane")
            .with_password("plane")
            .with_tag("15.7-alpine")
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

    /// Actualiza (o inserta si no existe) un valor de `instance_configurations`.
    /// Asegura que exista una `instances` activa con `is_setup_done = true`
    /// en la DB del test. Es requisito para cualquier endpoint de auth que
    /// llame a `instances::Entity::find().active()` (p.ej. `email-check`,
    /// `sign-in`, `sign-up`, magic link) — sin esta fila el handler corta
    /// temprano con `INSTANCE_NOT_CONFIGURED` (400).
    ///
    /// Idempotente: si ya hay una instance, fuerza `is_setup_done = true`.
    /// Valores por defecto replican los del onboarding de Plane.
    pub async fn ensure_instance_configured(&self) {
        use sea_orm::{ActiveModelTrait, EntityTrait};

        if let Some(existing) = instances::Entity::find()
            .one(&self.state.db)
            .await
            .expect("query instances")
        {
            if existing.is_setup_done {
                return;
            }
            let mut am: instances::ActiveModel = existing.into();
            am.is_setup_done = Set(true);
            am.update(&self.state.db).await.expect("mark instance setup_done");
            return;
        }

        let now = Utc::now();
        let am = instances::ActiveModel {
            id: Set(Uuid::new_v4()),
            instance_name: Set("Plane Test".to_owned()),
            instance_id: Set(format!("test-{}", Uuid::new_v4())),
            current_version: Set("0.0.0-test".to_owned()),
            last_checked_at: Set(now.into()),
            namespace: Set(None),
            is_telemetry_enabled: Set(false),
            is_support_required: Set(false),
            is_setup_done: Set(true),
            is_signup_screen_visited: Set(true),
            is_verified: Set(true),
            created_by_id: Set(None),
            updated_by_id: Set(None),
            domain: Set("localhost".to_owned()),
            latest_version: Set(None),
            edition: Set("plane-ce".to_owned()),
            is_test: Set(true),
            is_current_version_deprecated: Set(false),
            whitelist_emails: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
        };
        am.insert(&self.state.db).await.expect("insert test instance");
    }

    /// Útil para activar gates en tests (p.ej. `EMAIL_HOST` para desbloquear
    /// `forgot-password`, `ENABLE_SIGNUP=0` para probar signup deshabilitado).
    ///
    /// NOTA: no cifra — solo válido para configs con `is_encrypted = false`.
    pub async fn set_instance_config(&self, key: &str, value: &str) {
        use sea_orm::{ConnectionTrait, Statement};

        // UPSERT vía SQL raw para eliminar cualquier ambigüedad del
        // ActiveModel update (que solo actualiza campos marcados como Set
        // y puede caerse al llegar from `Model::into()` con PKs Unchanged).
        // `key` tiene constraint UNIQUE, así que ON CONFLICT (key) es seguro.
        let sql = r#"
            INSERT INTO instance_configurations
                (id, key, value, category, is_encrypted, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, false, NOW(), NOW(), NULL)
            ON CONFLICT (key) DO UPDATE
              SET value = EXCLUDED.value,
                  updated_at = NOW(),
                  deleted_at = NULL
        "#;
        self.state
            .db
            .execute(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                sql,
                [
                    Uuid::new_v4().into(),
                    key.to_owned().into(),
                    value.to_owned().into(),
                    "TEST".to_owned().into(),
                ],
            ))
            .await
            .expect("upsert instance_configurations");
    }

    /// Crea un usuario de test directamente en la base de datos y devuelve
    /// `(user_id, api_key_string)`. El api_key se puede usar en el header
    /// `x-api-key` para autenticar requests.
    ///
    /// No toca la lógica de sign-up — inserta directamente en `users` y
    /// `api_tokens`, lo que hace los tests independientes de los endpoints
    /// de auth y más rápidos.
    pub async fn create_test_user(&self, email: &str) -> (Uuid, String) {
        let now = Utc::now();
        let user_id = Uuid::new_v4();
        let api_key = format!("test-key-{}", Uuid::new_v4().as_simple());

        // ── Usuario ──────────────────────────────────────────────────────
        let user_am = users::ActiveModel {
            id: Set(user_id),
            username: Set(format!("test_{}", user_id.as_simple())),
            email: Set(Some(email.to_owned())),
            first_name: Set(String::new()),
            last_name: Set(String::new()),
            display_name: Set(email.split('@').next().unwrap_or("test").to_owned()),
            password: Set("!unusable".into()),
            is_active: Set(true),
            is_staff: Set(false),
            is_superuser: Set(false),
            is_managed: Set(false),
            is_password_expired: Set(false),
            is_email_verified: Set(true),
            is_email_valid: Set(true),
            is_password_autoset: Set(false),
            is_bot: Set(false),
            is_password_reset_required: Set(false),
            avatar: Set(String::new()),
            user_timezone: Set("UTC".into()),
            last_login_ip: Set(String::new()),
            last_logout_ip: Set(String::new()),
            last_login_medium: Set("email".into()),
            last_login_uagent: Set(String::new()),
            last_location: Set(String::new()),
            created_location: Set(String::new()),
            token: Set(Uuid::new_v4().to_string()),
            date_joined: Set(now.into()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };
        user_am.insert(&self.state.db).await.expect("insert test user");

        // ── API Token ─────────────────────────────────────────────────────
        let token_am = api_tokens::ActiveModel {
            id: Set(Uuid::new_v4()),
            token: Set(api_key.clone()),
            label: Set("test-token".into()),
            user_type: Set(0),
            user_id: Set(user_id),
            description: Set(String::new()),
            is_active: Set(true),
            is_service: Set(false),
            allowed_rate_limit: Set("default".into()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };
        token_am.insert(&self.state.db).await.expect("insert api token");

        (user_id, api_key)
    }

    /// Crea un workspace de test directamente en la DB y registra al usuario
    /// como miembro con rol **Admin** (role = 20, igual que Django).
    ///
    /// Devuelve el `slug` del workspace creado.
    pub async fn create_test_workspace(&self, owner_id: Uuid, slug: &str) -> String {
        let now = Utc::now();
        let ws_id = Uuid::new_v4();

        // ── Workspace ────────────────────────────────────────────────────
        let ws_am = workspaces::ActiveModel {
            id: Set(ws_id),
            name: Set(format!("Test WS {slug}")),
            slug: Set(slug.to_owned()),
            owner_id: Set(owner_id),
            organization_size: Set(None),
            logo: Set(None),
            logo_asset_id: Set(None),
            timezone: Set("UTC".into()),
            background_color: Set("#000000".into()),
            created_by_id: Set(Some(owner_id)),
            updated_by_id: Set(Some(owner_id)),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
        };
        ws_am.insert(&self.state.db).await.expect("insert test workspace");

        // ── WorkspaceMember (Admin = 20) ─────────────────────────────────
        let member_am = workspace_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            workspace_id: Set(ws_id),
            member_id: Set(owner_id),
            role: Set(20),
            is_active: Set(true),
            created_by_id: Set(Some(owner_id)),
            updated_by_id: Set(Some(owner_id)),
            // 5 columnas JSON NOT NULL sin DEFAULT en la baseline; Django
            // las llena vía `default=dict` en el modelo, acá hay que ser
            // explícito.
            view_props: Set(serde_json::json!({})),
            default_props: Set(serde_json::json!({})),
            issue_props: Set(serde_json::json!({})),
            explored_features: Set(serde_json::json!({})),
            getting_started_checklist: Set(serde_json::json!({})),
            tips: Set(serde_json::json!({})),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
            ..Default::default()
        };
        member_am
            .insert(&self.state.db)
            .await
            .expect("insert workspace member");

        slug.to_owned()
    }

    /// Crea un proyecto de test directamente en la DB con el usuario como
    /// miembro **Admin** (role = 20) y registra el identificador en
    /// `project_identifiers`.
    ///
    /// Devuelve el `project_id` (UUID).
    pub async fn create_test_project(
        &self,
        owner_id: Uuid,
        workspace_id: Uuid,
        name: &str,
        identifier: &str,
    ) -> Uuid {
        let now = Utc::now();
        let project_id = Uuid::new_v4();

        // ── Project ──────────────────────────────────────────────────────
        let proj_am = projects::ActiveModel {
            id: Set(project_id),
            name: Set(name.to_owned()),
            identifier: Set(identifier.to_uppercase()),
            description: Set(String::new()),
            network: Set(0),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(owner_id)),
            updated_by_id: Set(Some(owner_id)),
            cycle_view: Set(true),
            module_view: Set(true),
            issue_views_view: Set(true),
            // Columnas NOT NULL sin DEFAULT en la baseline SQL. Django las llena
            // vía `default=` en el modelo (`apps/api/plane/db/models/project.py`);
            // acá hay que ser explícito o el INSERT falla con 23502.
            page_view: Set(true),
            // Paridad con el route real `routes::projects::create_project` —
            // el endpoint productivo setea `intake_view: true` al crear un
            // proyecto (projects.rs:919). El helper antes seteaba `false`,
            // lo que impedía a los tests de intake/inbox funcionar (POST
            // /intake-issues exige `project.intake_view=true`). Mantener
            // esto alineado con el route evita scaffolding por test y
            // refleja el estado real de un proyecto recién creado.
            intake_view: Set(true),
            archive_in: Set(0),
            close_in: Set(0),
            is_time_tracking_enabled: Set(false),
            is_issue_type_enabled: Set(false),
            guest_view_all_features: Set(false),
            timezone: Set("UTC".to_owned()),
            logo_props: Set(serde_json::json!({})),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
            ..Default::default()
        };
        proj_am.insert(&self.state.db).await.expect("insert test project");

        // ── ProjectIdentifier ────────────────────────────────────────────
        let ident_am = project_identifiers::ActiveModel {
            name: Set(identifier.to_uppercase()),
            project_id: Set(project_id),
            workspace_id: Set(Some(workspace_id)),
            created_by_id: Set(Some(owner_id)),
            updated_by_id: Set(Some(owner_id)),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
            ..Default::default()
        };
        ident_am
            .insert(&self.state.db)
            .await
            .expect("insert project identifier");

        // ── ProjectMember (Admin = 20) ────────────────────────────────────
        let pm_am = project_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            member_id: Set(Some(owner_id)),
            role: Set(20),
            is_active: Set(true),
            created_by_id: Set(Some(owner_id)),
            updated_by_id: Set(Some(owner_id)),
            // 3 columnas JSON NOT NULL en project_members.
            view_props: Set(serde_json::json!({})),
            default_props: Set(serde_json::json!({})),
            preferences: Set(serde_json::json!({})),
            // NOT NULL sin DEFAULT en baseline; Django usa default=65535
            // (apps/api/plane/db/models/project.py::ProjectMember.sort_order).
            sort_order: Set(65535.0),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
            ..Default::default()
        };
        pm_am
            .insert(&self.state.db)
            .await
            .expect("insert project member");

        project_id
    }

    /// Agrega un usuario como miembro de un workspace con el rol dado.
    /// Roles: 20 = Admin, 15 = Member, 10 = Viewer, 5 = Guest.
    pub async fn add_workspace_member(&self, user_id: Uuid, workspace_id: Uuid, role: i16) {
        let now = Utc::now();
        let member_am = workspace_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            workspace_id: Set(workspace_id),
            member_id: Set(user_id),
            role: Set(role),
            is_active: Set(true),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            // Mismas 5 columnas JSON NOT NULL que en create_test_workspace.
            view_props: Set(serde_json::json!({})),
            default_props: Set(serde_json::json!({})),
            issue_props: Set(serde_json::json!({})),
            explored_features: Set(serde_json::json!({})),
            getting_started_checklist: Set(serde_json::json!({})),
            tips: Set(serde_json::json!({})),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            deleted_at: Set(None),
            ..Default::default()
        };
        member_am
            .insert(&self.state.db)
            .await
            .expect("insert workspace member");
    }

    /// Devuelve el `id` del workspace con el slug dado.
    /// Falla el test si no existe.
    pub async fn workspace_id_by_slug(&self, slug: &str) -> Uuid {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        workspaces::Entity::find()
            .filter(workspaces::Column::Slug.eq(slug))
            .one(&self.state.db)
            .await
            .expect("query workspace by slug")
            .unwrap_or_else(|| panic!("workspace slug={slug} no existe"))
            .id
    }

    /// GET autenticado vía API key (`x-api-key` header).
    pub async fn get_authed(&self, api_key: &str, path: &str) -> TestResponse {
        self.request(Method::GET, path, None, &[("x-api-key", api_key)])
            .await
    }

    /// DELETE autenticado vía API key, sin body.
    pub async fn delete_authed(&self, api_key: &str, path: &str) -> TestResponse {
        self.request(Method::DELETE, path, None, &[("x-api-key", api_key)])
            .await
    }

    /// PATCH con body JSON, autenticado vía API key.
    pub async fn patch_json_authed(
        &self,
        api_key: &str,
        path: &str,
        payload: &Value,
    ) -> TestResponse {
        let body = serde_json::to_vec(payload).expect("serializar payload");
        self.request(
            Method::PATCH,
            path,
            Some(body),
            &[
                ("content-type", "application/json"),
                ("x-api-key", api_key),
            ],
        )
        .await
    }

    /// POST con body JSON, autenticado vía API key.
    pub async fn post_json_authed(
        &self,
        api_key: &str,
        path: &str,
        payload: &Value,
    ) -> TestResponse {
        let body = serde_json::to_vec(payload).expect("serializar payload");
        self.request(
            Method::POST,
            path,
            Some(body),
            &[
                ("content-type", "application/json"),
                ("x-api-key", api_key),
            ],
        )
        .await
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

    /// POST con body `application/x-www-form-urlencoded` — usado por los
    /// endpoints heredados de Django (`sign-in`, `sign-up`, `sign-out`,
    /// `magic-*`, `forgot-password`, `reset-password`).
    pub async fn post_form(&self, path: &str, fields: &[(&str, &str)]) -> TestResponse {
        let body = serde_urlencoded::to_string(fields).expect("urlencode");
        self.request(
            Method::POST,
            path,
            Some(body.into_bytes()),
            &[("content-type", "application/x-www-form-urlencoded")],
        )
        .await
    }

    /// Extrae la ubicación (header `Location`) de una respuesta de redirect.
    /// Devuelve `None` si la respuesta no es redirect.
    pub async fn post_form_location(&self, path: &str, fields: &[(&str, &str)]) -> (StatusCode, Option<String>) {
        let body = serde_urlencoded::to_string(fields).expect("urlencode");
        let req = Request::builder()
            .method(Method::POST)
            .uri(normalize_path(path))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .expect("request build");
        let res = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("router oneshot");
        let status = res.status();
        let location = res
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok().map(String::from));
        (status, location)
    }

    /// Ejecuta una request arbitraria contra el router.
    ///
    /// Trimea la barra final del path para imitar el comportamiento del
    /// `NormalizePathLayer::trim_trailing_slash()` que envuelve al router
    /// en producción (ver `main.rs`). Así los tests pueden usar tanto
    /// `"/auth/sign-in"` como `"/auth/sign-in/"` sin diferencia, igual que
    /// hacen el frontend y Caddy.
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
        headers: &[(&str, &str)],
    ) -> TestResponse {
        let normalized = normalize_path(path);
        let mut builder = Request::builder().method(method).uri(normalized);
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

/// Imita el `NormalizePathLayer::trim_trailing_slash()` de producción.
/// Trimea SOLO la barra final del path, conservando query string si existe.
///
/// Además **prefija `/api`** cuando el path no empieza por un mount conocido
/// (`/api`, `/auth`) ni es la raíz. El router real anida TODAS las rutas
/// de negocio bajo `.nest("/api", api_router)` (ver `src/routes/mod.rs`),
/// mientras que los tests de integración se escribieron contra los paths
/// "lógicos" (ej. `/workspaces/{slug}/analytics`). Sin este prefijo
/// automático, el router devuelve 404 en lugar del 401/200 esperado.
/// Los tests que quieran hitear `/api/...` o `/auth/...` explícitamente
/// (ej. OAuth callbacks, auth flows) siguen funcionando porque ya traen
/// el prefijo.
fn normalize_path(path: &str) -> String {
    let (path_only, query) = match path.find('?') {
        Some(i) => (&path[..i], Some(&path[i..])),
        None => (path, None),
    };
    let trimmed = if path_only.len() > 1 && path_only.ends_with('/') {
        path_only.trim_end_matches('/')
    } else {
        path_only
    };
    // Prefija `/api` si el path no viene ya con un mount conocido.
    let prefixed: std::borrow::Cow<'_, str> = if trimmed == "/"
        || trimmed.starts_with("/api/")
        || trimmed == "/api"
        || trimmed.starts_with("/auth/")
        || trimmed == "/auth"
    {
        std::borrow::Cow::Borrowed(trimmed)
    } else {
        std::borrow::Cow::Owned(format!("/api{trimmed}"))
    };
    match query {
        Some(q) => format!("{prefixed}{q}"),
        None => prefixed.into_owned(),
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
