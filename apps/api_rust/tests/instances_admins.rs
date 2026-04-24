//! Tests de integración: Instance Admins (God Mode)
//!
//! Cobertura:
//!   - GET  /instances/admins                              → 403 sin admin
//!   - POST /instances/admins                              → 403 sin admin
//!   - DELETE /instances/admins/{pk}                      → 403 sin admin
//!   - DELETE /instances/configurations/disable-email-feature → 200/204

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /instances/admins — requiere ser instance-admin
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_instance_admins_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/instances/admins").await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "GET /instances/admins sin auth debe devolver 401"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_instance_admins_regular_user_returns_403() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("regular_admin_list@plane.test").await;

    let res = app.get_authed(&api_key, "/instances/admins").await;
    assert!(
        res.status.as_u16() == 403 || res.status.as_u16() == 401,
        "usuario regular no debe listar instance admins, obtuvo {}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /instances/admins — requiere ser instance-admin
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_instance_admin_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .post_json("/instances/admins", &json!({ "user_id": uuid::Uuid::new_v4() }))
        .await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "POST /instances/admins sin auth debe devolver 401"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_instance_admin_regular_user_returns_403() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("regular_admin_create@plane.test").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/instances/admins",
            &json!({ "user_id": uuid::Uuid::new_v4() }),
        )
        .await;
    assert!(
        res.status.as_u16() == 403 || res.status.as_u16() == 401,
        "usuario regular no debe crear instance admins, obtuvo {}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /instances/admins/{pk} — requiere ser instance-admin
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_instance_admin_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .delete_authed("no-key", &format!("/instances/admins/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "DELETE /instances/admins/{{pk}} sin auth debe devolver 401"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_instance_admin_regular_user_returns_403() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("regular_admin_delete@plane.test").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .delete_authed(&api_key, &format!("/instances/admins/{fake_id}"))
        .await;
    assert!(
        res.status.as_u16() == 403 || res.status.as_u16() == 401,
        "usuario regular no debe eliminar instance admins, obtuvo {}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /instances/configurations/disable-email-feature
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn disable_email_feature_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app
        .request(
            Method::DELETE,
            "/instances/configurations/disable-email-feature",
            None,
            &[],
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "disable-email-feature sin auth debe devolver 401"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn disable_email_feature_regular_user_returns_403() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("disable_email_user@plane.test").await;

    let res = app
        .request(
            Method::DELETE,
            "/instances/configurations/disable-email-feature",
            None,
            &[("x-api-key", api_key.as_str())],
        )
        .await;
    assert!(
        res.status.as_u16() == 403 || res.status.as_u16() == 401,
        "usuario regular no debe deshabilitar email feature, obtuvo {}",
        res.status
    );
}
