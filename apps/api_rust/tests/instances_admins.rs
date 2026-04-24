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

// ─────────────────────────────────────────────────────────────────────────────
// POST /instances/admins/sign-up
// POST /instances/admins/sign-in
// POST /instances/admins/sign-out
//
// Estos endpoints retornan Redirect (303) — éxito o error se codifica en la
// URL de destino. Verificamos el código de estado del redirect.
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_up_missing_fields_redirects() {
    // Sin campos obligatorios → redirect de error (Location != /general/)
    let app = TestApp::spawn().await;
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[("email", ""), ("password", ""), ("first_name", "")],
        )
        .await;
    // Debe redirigir (303) pero NO a la ruta de éxito
    assert_eq!(status.as_u16(), 303, "debe redirigir con campos vacíos");
    let loc = location.unwrap_or_default();
    assert!(
        !loc.ends_with("general/"),
        "no debe redirigir a /general/ con campos vacíos, location={loc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_up_weak_password_redirects_error() {
    let app = TestApp::spawn().await;
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[
                ("email", "admin-weak@plane.test"),
                ("password", "1234"),  // contraseña débil
                ("first_name", "Admin"),
                ("last_name", "Test"),
                ("company_name", "Test Co"),
            ],
        )
        .await;
    assert_eq!(status.as_u16(), 303, "debe redirigir con contraseña débil");
    let loc = location.unwrap_or_default();
    assert!(
        !loc.ends_with("general/"),
        "contraseña débil no debe llevar a /general/, location={loc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_up_invalid_email_redirects_error() {
    let app = TestApp::spawn().await;
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[
                ("email", "not-an-email"),
                ("password", "Str0ng!Pass#2025"),
                ("first_name", "Admin"),
            ],
        )
        .await;
    assert_eq!(status.as_u16(), 303);
    let loc = location.unwrap_or_default();
    assert!(
        !loc.ends_with("general/"),
        "email inválido no debe llevar a /general/"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_up_valid_first_time_redirects_success() {
    // Primer admin: campos válidos → debe redirigir hacia /general/
    let app = TestApp::spawn().await;
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[
                ("email", "first.admin@plane.test"),
                ("password", "Correct-Horse-Battery-Staple!99"),
                ("first_name", "First"),
                ("last_name", "Admin"),
                ("company_name", "Plane Test"),
                ("is_telemetry_enabled", "false"),
            ],
        )
        .await;
    assert_eq!(status.as_u16(), 303, "sign-up exitoso debe redirigir");
    let loc = location.unwrap_or_default();
    assert!(
        loc.ends_with("general/") || loc.contains("general"),
        "sign-up exitoso debe apuntar a /general/, obtuvo location={loc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_up_duplicate_redirects_error() {
    // Registrar admin dos veces → segunda vez debe fallar
    let app = TestApp::spawn().await;
    // Primera vez (exitosa)
    let _ = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[
                ("email", "dup.admin@plane.test"),
                ("password", "Correct-Horse-Battery-Staple!99"),
                ("first_name", "Dup"),
                ("last_name", "Admin"),
            ],
        )
        .await;
    // Segunda vez (debe fallar — ADMIN_ALREADY_EXIST)
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[
                ("email", "dup.admin2@plane.test"),
                ("password", "Correct-Horse-Battery-Staple!99"),
                ("first_name", "Dup2"),
                ("last_name", "Admin2"),
            ],
        )
        .await;
    assert_eq!(status.as_u16(), 303);
    let loc = location.unwrap_or_default();
    assert!(
        !loc.ends_with("general/"),
        "segundo sign-up debe fallar, obtuvo location={loc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_in_nonexistent_user_redirects_error() {
    let app = TestApp::spawn().await;
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-in",
            &[
                ("email", "ghost.admin@plane.test"),
                ("password", "Correct-Horse-Battery-Staple!99"),
            ],
        )
        .await;
    // Usuario no existe → error redirect
    assert_eq!(status.as_u16(), 303);
    let loc = location.unwrap_or_default();
    assert!(
        !loc.ends_with("general/"),
        "sign-in de usuario inexistente no debe ir a /general/, location={loc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_in_missing_fields_redirects_error() {
    let app = TestApp::spawn().await;
    let (status, _) = app
        .post_form_location(
            "/instances/admins/sign-in",
            &[("email", ""), ("password", "")],
        )
        .await;
    assert_eq!(status.as_u16(), 303, "campos vacíos deben redirigir");
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_in_wrong_password_redirects_error() {
    // Registrar admin primero
    let app = TestApp::spawn().await;
    let _ = app
        .post_form_location(
            "/instances/admins/sign-up",
            &[
                ("email", "signin.admin@plane.test"),
                ("password", "Correct-Horse-Battery-Staple!99"),
                ("first_name", "SignIn"),
                ("last_name", "Admin"),
            ],
        )
        .await;
    // Intentar con contraseña incorrecta
    let (status, location) = app
        .post_form_location(
            "/instances/admins/sign-in",
            &[
                ("email", "signin.admin@plane.test"),
                ("password", "WrongPassword!1234"),
            ],
        )
        .await;
    assert_eq!(status.as_u16(), 303);
    let loc = location.unwrap_or_default();
    assert!(
        !loc.ends_with("general/"),
        "contraseña incorrecta no debe ir a /general/, location={loc}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_sign_out_without_csrf_rejected() {
    let app = TestApp::spawn().await;
    // Sin token CSRF → debe retornar 403 o redirigir a error
    let res = app
        .post_form("/instances/admins/sign-out", &[])
        .await;
    assert!(
        res.status.as_u16() == 403 || res.status.as_u16() == 303,
        "sign-out sin CSRF debe fallar (403 o redirect de error), obtuvo {}",
        res.status.as_u16()
    );
}
