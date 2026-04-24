//! Tests de integración: `POST /auth/sign-in`, `/auth/sign-up`, `/auth/sign-out`.
//!
//! Estos endpoints usan `application/x-www-form-urlencoded` (paridad Django)
//! y responden siempre con `303 See Other`:
//!   - éxito  → redirige al destino de la superficie (`app_base` o `space_base`)
//!   - error  → redirige a la misma URL con `?error_code=NNNN&error_message=…`
//!
//! Los tests verifican los **códigos de error** en la URL, no el body.

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};

fn location_has_error(location: &str, code: &str) -> bool {
    location.contains(&format!("error_code={code}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/sign-up
//   Se testea ANTES que sign-in: crea el usuario que sign-in consume.
// ─────────────────────────────────────────────────────────────────────────────

/// Sign-up sin email → redirect con `error_code=5040` (REQUIRED_EMAIL_PASSWORD_SIGN_UP).
#[tokio::test(flavor = "multi_thread")]
async fn sign_up_without_email_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, location) = app
        .post_form_location("/auth/sign-up/", &[("password", "Test12345!")])
        .await;

    assert!(
        status.is_redirection(),
        "sign-up debe redireccionar (3xx), obtuvo {status}"
    );
    let loc = location.expect("sign-up debe emitir header Location");
    assert!(
        location_has_error(&loc, "5040"),
        "error_code=5040 esperado en Location={loc:?}"
    );
}

/// Sign-up sin password → redirect con `error_code=5040`.
#[tokio::test(flavor = "multi_thread")]
async fn sign_up_without_password_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, location) = app
        .post_form_location("/auth/sign-up/", &[("email", "test@plane.local")])
        .await;

    assert!(status.is_redirection());
    let loc = location.expect("header Location requerido");
    assert!(
        location_has_error(&loc, "5040"),
        "error_code=5040 esperado en Location={loc:?}"
    );
}

/// Sign-up con email sintácticamente inválido → `error_code=5045`
/// (INVALID_EMAIL_SIGN_UP). Property-based: cualquier string sin `@` falla.
#[tokio::test(flavor = "multi_thread")]
async fn sign_up_rejects_invalid_emails() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let strategy = "[a-zA-Z0-9]{1,16}"; // garantiza ausencia de '@' → inválido
    let mut runner = TestRunner::new(PropConfig {
        cases: 12,
        ..PropConfig::default()
    });

    runner
        .run(&strategy.prop_map(|s| s), |email| {
            let app = &app;
            let rt = tokio::runtime::Handle::current();
            let (status, loc) = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_form_location(
                        "/auth/sign-up/",
                        &[("email", email.as_str()), ("password", "Test12345!")],
                    )
                    .await
                })
            });
            prop_assert!(
                status.is_redirection(),
                "3xx esperado para email={:?}, obtuvo {}",
                email, status
            );
            let loc = loc.expect("Location requerido");
            prop_assert!(
                location_has_error(&loc, "5045"),
                "error_code=5045 esperado para email={:?}, Location={:?}",
                email, loc
            );
            Ok(())
        })
        .expect("proptest sign-up email inválido");
}

/// Sign-up con credenciales válidas crea el usuario → redirige SIN `error_code`.
#[tokio::test(flavor = "multi_thread")]
async fn sign_up_with_valid_credentials_redirects_to_success() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, location) = app
        .post_form_location(
            "/auth/sign-up/",
            &[
                ("email", "nuevo@plane.local"),
                ("password", "Test12345!"),
            ],
        )
        .await;

    assert!(
        status.is_redirection(),
        "sign-up debe redireccionar, obtuvo {status}"
    );
    let loc = location.expect("Location requerido");
    assert!(
        !loc.contains("error_code="),
        "sign-up exitoso NO debe llevar error_code en Location={loc:?}"
    );
}

/// Sign-up con email ya registrado → `error_code=5030` (USER_ALREADY_EXIST).
#[tokio::test(flavor = "multi_thread")]
async fn sign_up_duplicate_email_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    // Primer sign-up: éxito
    let (s1, _) = app
        .post_form_location(
            "/auth/sign-up/",
            &[("email", "dup@plane.local"), ("password", "Test12345!")],
        )
        .await;
    assert!(s1.is_redirection(), "primer sign-up debe redireccionar");

    // Segundo sign-up con el mismo email: debe fallar
    let (s2, loc) = app
        .post_form_location(
            "/auth/sign-up/",
            &[("email", "dup@plane.local"), ("password", "Otro123!")],
        )
        .await;
    assert!(s2.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5030"),
        "duplicado debe producir error_code=5030, Location={loc:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/sign-in
// ─────────────────────────────────────────────────────────────────────────────

/// Sign-in sin email / sin password → `error_code=5070`
/// (REQUIRED_EMAIL_PASSWORD_SIGN_IN).
#[tokio::test(flavor = "multi_thread")]
async fn sign_in_missing_credentials_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    for fields in [
        vec![("password", "whatever")],
        vec![("email", "foo@bar.com")],
        vec![],
    ] {
        let (status, loc) = app.post_form_location("/auth/sign-in/", &fields).await;
        assert!(status.is_redirection());
        let loc = loc.expect("Location requerido");
        assert!(
            location_has_error(&loc, "5070"),
            "error_code=5070 esperado para fields={fields:?}, Location={loc:?}"
        );
    }
}

/// Sign-in con email malformado → `error_code=5075` (INVALID_EMAIL_SIGN_IN).
#[tokio::test(flavor = "multi_thread")]
async fn sign_in_invalid_email_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/sign-in/",
            &[("email", "no-es-email"), ("password", "Whatever123!")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5075"),
        "error_code=5075 esperado, Location={loc:?}"
    );
}

/// Sign-in con usuario inexistente → `error_code=5060` (USER_DOES_NOT_EXIST).
#[tokio::test(flavor = "multi_thread")]
async fn sign_in_unknown_user_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/sign-in/",
            &[
                ("email", "fantasma@plane.local"),
                ("password", "Whatever123!"),
            ],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5060"),
        "error_code=5060 esperado, Location={loc:?}"
    );
}

/// Sign-in con password incorrecto tras un sign-up → `error_code=5065`
/// (AUTHENTICATION_FAILED_SIGN_IN).
#[tokio::test(flavor = "multi_thread")]
async fn sign_in_wrong_password_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    // Crea el usuario.
    let (s_up, _) = app
        .post_form_location(
            "/auth/sign-up/",
            &[("email", "brayan@plane.local"), ("password", "Correcta1!")],
        )
        .await;
    assert!(s_up.is_redirection());

    // Intenta iniciar sesión con password diferente.
    let (status, loc) = app
        .post_form_location(
            "/auth/sign-in/",
            &[("email", "brayan@plane.local"), ("password", "INcorrecta1!")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5065"),
        "error_code=5065 esperado, Location={loc:?}"
    );
}

/// Sign-in con password correcto tras un sign-up → redirect SIN `error_code`.
#[tokio::test(flavor = "multi_thread")]
async fn sign_in_valid_credentials_redirects_to_success() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (s_up, _) = app
        .post_form_location(
            "/auth/sign-up/",
            &[("email", "ok@plane.local"), ("password", "Correcta1!")],
        )
        .await;
    assert!(s_up.is_redirection());

    let (status, loc) = app
        .post_form_location(
            "/auth/sign-in/",
            &[("email", "ok@plane.local"), ("password", "Correcta1!")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        !loc.contains("error_code="),
        "sign-in exitoso NO debe llevar error_code, Location={loc:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/sign-out
// ─────────────────────────────────────────────────────────────────────────────

/// Sign-out sin sesión activa → 401 (el extractor `SessionUser` falla).
#[tokio::test(flavor = "multi_thread")]
async fn sign_out_without_session_returns_401() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let (status, _loc) = app
        .post_form_location(
            "/auth/sign-out/",
            &[("csrfmiddlewaretoken", "deadbeef")],
        )
        .await;

    assert_eq!(
        status.as_u16(),
        401,
        "sign-out sin sesión debe devolver 401, obtuvo {status}"
    );
}
