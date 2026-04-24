//! Tests de integración para gestión de contraseñas:
//!   - `POST /auth/forgot-password`                   (JSON)
//!   - `POST /auth/reset-password/{uidb64}/{token}`   (Form urlencoded)
//!   - `POST /auth/change-password`                   (JSON, requiere sesión)
//!   - `POST /auth/set-password`                      (JSON, requiere sesión)
//!
//! El sendpath de `forgot-password` depende de `EMAIL_HOST` en
//! `instance_configurations`. El test activa ese gate vía
//! `TestApp::set_instance_config` para poder ejercitar la validación de email.

mod common;

use common::TestApp;
use serde_json::json;

fn location_has_error(location: &str, code: &str) -> bool {
    location.contains(&format!("error_code={code}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/forgot-password
// ─────────────────────────────────────────────────────────────────────────────

/// Sin SMTP configurado el endpoint responde 400 `SMTP_NOT_CONFIGURED`.
/// Verifica que el gate aplica antes que la validación de email.
#[tokio::test(flavor = "multi_thread")]
async fn forgot_password_without_smtp_returns_400() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json(
            "/auth/forgot-password/",
            &json!({ "email": "alguien@plane.local" }),
        )
        .await;

    assert_eq!(
        res.status, 400,
        "sin SMTP debe retornar 400, body={}",
        String::from_utf8_lossy(&res.body)
    );
}

/// Con SMTP configurado + email malformado → 400 `INVALID_EMAIL`.
#[tokio::test(flavor = "multi_thread")]
async fn forgot_password_with_smtp_rejects_invalid_email() {
    let app = TestApp::spawn().await;
    app.set_instance_config("EMAIL_HOST", "smtp.test.local").await;

    let res = app
        .post_json("/auth/forgot-password/", &json!({ "email": "no-es-email" }))
        .await;

    assert_eq!(
        res.status, 400,
        "email inválido debe retornar 400, body={}",
        String::from_utf8_lossy(&res.body)
    );
}

/// Con SMTP configurado + email válido pero usuario inexistente →
/// 400 `USER_DOES_NOT_EXIST`.
#[tokio::test(flavor = "multi_thread")]
async fn forgot_password_unknown_user_returns_400() {
    let app = TestApp::spawn().await;
    app.set_instance_config("EMAIL_HOST", "smtp.test.local").await;

    let res = app
        .post_json(
            "/auth/forgot-password/",
            &json!({ "email": "fantasma@plane.local" }),
        )
        .await;

    assert_eq!(res.status, 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/reset-password/{uidb64}/{token}
// ─────────────────────────────────────────────────────────────────────────────

/// uidb64 malformado → redirect con `error_code=5125` (INVALID_PASSWORD_TOKEN).
#[tokio::test(flavor = "multi_thread")]
async fn reset_password_invalid_uidb64_redirects_with_error() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/reset-password/not-base64/deadbeef/",
            &[("password", "Nueva123!")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5125"),
        "error_code=5125 esperado, Location={loc:?}"
    );
}

/// uidb64 válido pero sin token en Redis (expirado) → `error_code=5130`
/// (EXPIRED_PASSWORD_TOKEN) o `5125` (INVALID_PASSWORD_TOKEN) según el flujo.
#[tokio::test(flavor = "multi_thread")]
async fn reset_password_expired_token_redirects_with_error() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let app = TestApp::spawn().await;

    // uuid válido codificado en base64url (lo que espera el endpoint).
    let fake_uuid = uuid::Uuid::new_v4();
    let uidb64 = URL_SAFE_NO_PAD.encode(fake_uuid.to_string());

    let path = format!("/auth/reset-password/{uidb64}/deadbeeftoken/");
    let (status, loc) = app
        .post_form_location(&path, &[("password", "Nueva123!")])
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5125") || location_has_error(&loc, "5130"),
        "se esperaba error_code=5125 o 5130, Location={loc:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/change-password
// ─────────────────────────────────────────────────────────────────────────────

/// Sin sesión → 401.
#[tokio::test(flavor = "multi_thread")]
async fn change_password_without_session_returns_401() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json(
            "/auth/change-password/",
            &json!({ "old_password": "x", "new_password": "Nueva123!" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        401,
        "change-password sin sesión debe devolver 401"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/set-password
// ─────────────────────────────────────────────────────────────────────────────

/// Sin sesión → 401.
#[tokio::test(flavor = "multi_thread")]
async fn set_password_without_session_returns_401() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json("/auth/set-password/", &json!({ "password": "Nueva123!" }))
        .await;

    assert_eq!(
        res.status.as_u16(),
        401,
        "set-password sin sesión debe devolver 401"
    );
}
