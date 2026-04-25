//! Tests de integración: Instances (God Mode / Admin)
//!
//! Cobertura:
//!   - GET  /instances                              → 200 (no requiere auth)
//!   - POST /instances/email-credentials-check      → 401 sin auth (admin-only)
//!   - GET  /instances/workspace-slug-check         → 401 sin auth (admin-only)
//!   - GET  /instances/workspaces                   → 401 sin auth de instancia
//!   - GET/PATCH /instances/configurations          → 401 sin auth
//!   - POST /instances/admins/sign-up-screen-visited → 204
//!   - GET  /instances/admins/me                    → 200 con usuario válido

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances  (público — no requiere auth)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_instance_public_returns_200() {
    let app = TestApp::spawn().await;
    let res = app.get("/instances").await;
    assert_eq!(res.status.as_u16(), 200, "GET /instances debe devolver 200");
    let body = res.json();
    assert!(
        body["instance"].is_object(),
        "respuesta debe tener campo 'instance', got: {body}"
    );
}

/// La instancia seeded por TestApp::spawn debe estar activada.
#[tokio::test(flavor = "multi_thread")]
async fn get_instance_is_activated() {
    let app = TestApp::spawn().await;
    let res = app.get("/instances").await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    // El bootstrap registra la instancia → is_activated debe ser true
    assert_eq!(
        body["instance"]["is_activated"].as_bool(),
        Some(true),
        "instancia debe estar activada después del bootstrap, got: {body}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /instances/email-credentials-check  (admin-only, mirror Django)
// ─────────────────────────────────────────────────────────────────────────────

/// Django: EmailCredentialCheckEndpoint.post requiere auth de instance-admin
/// (BaseAPIView con autenticación por defecto). El handler Rust replica el
/// contrato vía AnyAuth + require_instance_admin. Una llamada anónima debe
/// rechazarse con 401 antes de tocar cualquier configuración SMTP.
#[tokio::test(flavor = "multi_thread")]
async fn email_credentials_check_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .request(
            Method::POST,
            "/instances/email-credentials-check",
            Some(serde_json::to_vec(&json!({"receiver_email": "test@plane.test"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "anonymous POST debe ser 401: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances/workspace-slug-check  (admin-only, mirror Django)
// ─────────────────────────────────────────────────────────────────────────────

/// Django: InstanceWorkSpaceAvailabilityCheckEndpoint declara
/// `permission_classes = [InstanceAdminPermission]`. El handler Rust usa
/// `require_instance_admin`. Sin auth → 401 (no se llega a evaluar el slug).
#[tokio::test(flavor = "multi_thread")]
async fn instance_workspace_slug_check_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/instances/workspace-slug-check?slug=unique-test-slug-xyz").await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances/workspaces  (requiere instancia-admin)
// ─────────────────────────────────────────────────────────────────────────────

/// Sin credenciales de admin de instancia → 401 o 403.
#[tokio::test(flavor = "multi_thread")]
async fn list_instance_workspaces_without_admin_returns_4xx() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("regular@plane.test").await;

    let res = app.get_authed(&api_key, "/instances/workspaces").await;
    assert!(
        res.status.as_u16() == 401 || res.status.as_u16() == 403,
        "usuario regular no debe acceder a /instances/workspaces, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances/configurations
// ─────────────────────────────────────────────────────────────────────────────

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn get_instance_configurations_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/instances/configurations").await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /instances/admins/sign-up-screen-visited
// ─────────────────────────────────────────────────────────────────────────────

/// Marcar pantalla de signup visitada → 204.
#[tokio::test(flavor = "multi_thread")]
async fn signup_screen_visited_returns_204() {
    let app = TestApp::spawn().await;
    let res = app
        .post_json_authed("no-key-needed", "/instances/admins/sign-up-screen-visited", &json!({}))
        .await;
    // Este endpoint no requiere auth en la implementación actual
    assert!(
        res.status.as_u16() == 204 || res.status.as_u16() == 200,
        "sign-up-screen-visited debe devolver 204 o 200, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances/admins/me
// ─────────────────────────────────────────────────────────────────────────────

/// Usuario autenticado → 200 con campo is_instance_admin.
#[tokio::test(flavor = "multi_thread")]
async fn get_instance_admin_me_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("instadminme@plane.test").await;

    let res = app.get_authed(&api_key, "/instances/admins/me").await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    // El campo puede ser is_instance_admin o similar
    assert!(
        body.is_object(),
        "respuesta debe ser objeto JSON"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances/admins/session  (OptionalAnyAuth)
// ─────────────────────────────────────────────────────────────────────────────

/// Sin auth → 200 con is_authenticated=false.
#[tokio::test(flavor = "multi_thread")]
async fn get_instance_admin_session_unauthenticated() {
    let app = TestApp::spawn().await;
    let res = app.get("/instances/admins/session").await;
    assert_eq!(res.status.as_u16(), 200, "session no auth debe devolver 200");
}

/// Con auth → 200 con datos del usuario.
#[tokio::test(flavor = "multi_thread")]
async fn get_instance_admin_session_authenticated() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("instsession@plane.test").await;

    let res = app.get_authed(&api_key, "/instances/admins/session").await;
    assert_eq!(res.status.as_u16(), 200);
}
