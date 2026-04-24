//! Tests de integración: Users Extended (settings, email, instance-admin)
//!
//! Cobertura:
//!   - GET  /users/me/settings                    → 200
//!   - GET  /users/me/instance-admin              → 200
//!   - DELETE /users/me/accounts/{pk}             → 4xx (sin cuenta vinculada)
//!   - POST /users/me/email/generate-code         → 200/400
//!   - POST /users/me/email                       → 200/400

mod common;

use common::TestApp;
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/settings
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_settings_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/users/me/settings").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_settings_authenticated_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("settings_user@plane.test").await;
    let res = app.get_authed(&api_key, "/users/me/settings").await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /users/me/settings debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_object(), "settings debe ser un objeto");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/instance-admin
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_instance_admin_status_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/users/me/instance-admin").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_instance_admin_status_regular_user_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("instadmin_check@plane.test").await;
    let res = app.get_authed(&api_key, "/users/me/instance-admin").await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /users/me/instance-admin debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    // Un usuario regular no es instance admin
    assert!(
        body.is_object() || body.is_array(),
        "respuesta debe ser objeto o array, got: {body}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /users/me/accounts/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_account_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed("no-key", &format!("/users/me/accounts/{fake_pk}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_nonexistent_account_returns_4xx() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("del_account@plane.test").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/users/me/accounts/{fake_pk}"))
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "eliminar cuenta inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /users/me/email/generate-code
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn generate_email_code_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .post_json("/users/me/email/generate-code", &json!({}))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn generate_email_code_authenticated_returns_2xx_or_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("gen_email_code@plane.test").await;
    let res = app
        .post_json_authed(
            &api_key,
            "/users/me/email/generate-code",
            &json!({ "email": "new_email@plane.test" }),
        )
        .await;
    let status = res.status.as_u16();
    // Sin EMAIL_HOST configurado puede devolver 400/503
    assert!(
        status == 200 || status == 201 || status == 400 || status == 503,
        "generate-code debe devolver 200/201/400/503, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /users/me/email
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_user_email_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .post_json("/users/me/email", &json!({}))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_user_email_with_invalid_code_returns_4xx() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("upd_email@plane.test").await;
    let res = app
        .post_json_authed(
            &api_key,
            "/users/me/email",
            &json!({
                "email": "changed@plane.test",
                "code": "000000"
            }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "código inválido debe devolver 4xx, obtuvo {status}"
    );
}
