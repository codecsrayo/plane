//! Tests de integración para endpoints de autenticación de la superficie
//! **Space** (`/auth/spaces/*`). Son alias de sus contrapartes app-surface
//! con identical lógica de validación — los error_codes son idénticos; la
//! diferencia está en el redirect target (space_base vs app_base).
//!
//! Estos tests priorizan **paridad de contrato**: verifican que la ruta
//! existe, valida lo mismo, y que el Location apunta al space_base_url
//! configurado. Las reglas detalladas ya están cubiertas por los tests
//! app-surface (`auth_sign_in_up_out.rs`, `auth_magic.rs`, etc.).

mod common;

use common::TestApp;
use serde_json::json;

fn location_has_error(location: &str, code: &str) -> bool {
    location.contains(&format!("error_code={code}"))
}

/// `space_base_url` configurado en el harness es `http://localhost:3001`.
const SPACE_BASE: &str = "http://localhost:3001";

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/email-check
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn space_email_check_accepts_valid_email() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let res = app
        .post_json(
            "/auth/spaces/email-check",
            &json!({ "email": "space-user@plane.local" }),
        )
        .await;

    assert_eq!(res.status, 200, "email-check space debe responder 200");
}

#[tokio::test(flavor = "multi_thread")]
async fn space_email_check_rejects_empty_email() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json("/auth/spaces/email-check", &json!({ "email": "" }))
        .await;

    assert_eq!(res.status, 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/sign-in
// ─────────────────────────────────────────────────────────────────────────────

/// Sin credenciales → redirige al space_base con `error_code=5070`.
#[tokio::test(flavor = "multi_thread")]
async fn space_sign_in_missing_credentials_redirects_to_space_base() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, loc) = app.post_form_location("/auth/spaces/sign-in", &[]).await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        loc.starts_with(SPACE_BASE),
        "redirect debe ir a space_base ({SPACE_BASE}), obtuvo {loc:?}"
    );
    assert!(
        location_has_error(&loc, "5070"),
        "error_code=5070 esperado, Location={loc:?}"
    );
}

/// Usuario desconocido → `error_code=5060`.
#[tokio::test(flavor = "multi_thread")]
async fn space_sign_in_unknown_user_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/spaces/sign-in",
            &[
                ("email", "fantasma@plane.local"),
                ("password", "Whatever1!"),
            ],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(loc.starts_with(SPACE_BASE));
    assert!(location_has_error(&loc, "5060"));
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/magic-generate
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn space_magic_generate_valid_email_returns_key() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;
    // Gates de magic-generate (magic_auth.rs:559-591): EMAIL_HOST no vacío
    // (5025 SMTP_NOT_CONFIGURED) + ENABLE_MAGIC_LINK_LOGIN="1" (5016
    // MAGIC_LINK_LOGIN_DISABLED, default "0" en startup.rs:102).
    app.set_instance_config("EMAIL_HOST", "localhost").await;
    app.set_instance_config("ENABLE_MAGIC_LINK_LOGIN", "1").await;

    let res = app
        .post_json(
            "/auth/spaces/magic-generate",
            &json!({ "email": "space-magic@plane.local" }),
        )
        .await;

    assert_eq!(res.status, 200);
    let body = res.json();
    assert!(
        body.get("key")
            .and_then(|v| v.as_str())
            .is_some_and(|k| k.starts_with("magic_"))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn space_magic_generate_invalid_email_returns_400() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json(
            "/auth/spaces/magic-generate",
            &json!({ "email": "no-es-email" }),
        )
        .await;

    assert_eq!(res.status, 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/magic-sign-in
// ─────────────────────────────────────────────────────────────────────────────

/// Sin code → `error_code=5085`, redirect al space_base.
#[tokio::test(flavor = "multi_thread")]
async fn space_magic_sign_in_without_code_redirects_to_space_base() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/spaces/magic-sign-in",
            &[("email", "space@plane.local")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        loc.starts_with(SPACE_BASE),
        "debe redirigir a space_base, Location={loc:?}"
    );
    assert!(location_has_error(&loc, "5085"));
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/magic-sign-up
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn space_magic_sign_up_without_code_redirects_to_space_base() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/spaces/magic-sign-up",
            &[("email", "space-new@plane.local")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(loc.starts_with(SPACE_BASE));
    assert!(location_has_error(&loc, "5055"));
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/sign-out
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn space_sign_out_without_session_returns_401() {
    let app = TestApp::spawn().await;

    let (status, _loc) = app
        .post_form_location(
            "/auth/spaces/sign-out",
            &[("csrfmiddlewaretoken", "deadbeef")],
        )
        .await;

    assert_eq!(status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/forgot-password
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn space_forgot_password_without_smtp_returns_400() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json(
            "/auth/spaces/forgot-password",
            &json!({ "email": "alguien@plane.local" }),
        )
        .await;

    assert_eq!(res.status, 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// /auth/spaces/reset-password/{uidb64}/{token}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn space_reset_password_invalid_uidb64_redirects_to_space_base() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/spaces/reset-password/not-base64/tok",
            &[("password", "Nueva123!")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        loc.starts_with(SPACE_BASE),
        "debe redirigir a space_base, Location={loc:?}"
    );
    assert!(location_has_error(&loc, "5125"));
}
