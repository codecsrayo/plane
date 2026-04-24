//! Tests de integración para los endpoints de autenticación mágica
//! (paridad con `MagicGenerateEndpoint`, `MagicSignInEndpoint`,
//! `MagicSignUpEndpoint` de Django).
//!
//! - `POST /auth/magic-generate`   → JSON, 200 + `key` / 400 `AuthError`.
//! - `POST /auth/magic-sign-in`    → Form urlencoded, 303 redirect.
//! - `POST /auth/magic-sign-up`    → Form urlencoded, 303 redirect.

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

fn location_has_error(location: &str, code: &str) -> bool {
    location.contains(&format!("error_code={code}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/magic-generate
// ─────────────────────────────────────────────────────────────────────────────

/// Contrato: email válido genera un código mágico. La respuesta es 200 con
/// una `key` (clave Redis donde vive el código), paridad con Django.
#[tokio::test(flavor = "multi_thread")]
async fn magic_generate_valid_email_returns_key() {
    let app = TestApp::spawn().await;
    // magic-generate depende de `instances.is_setup_done=true` y de la config
    // ENABLE_MAGIC_LINK_LOGIN. Sin instancia configurada el handler corta
    // con 400 INSTANCE_NOT_CONFIGURED antes de cualquier otra validación.
    app.ensure_instance_configured().await;

    let res = app
        .post_json(
            "/auth/magic-generate/",
            &json!({ "email": "magic@plane.local" }),
        )
        .await;

    assert_eq!(
        res.status, 200,
        "magic-generate debe responder 200, body={}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    let key = body
        .get("key")
        .and_then(|v| v.as_str())
        .expect("key presente");
    assert!(
        key.starts_with("magic_"),
        "key debe tener prefijo 'magic_', obtuvo {key:?}"
    );
}

/// Email vacío → 400 (`EMAIL_REQUIRED`).
#[tokio::test(flavor = "multi_thread")]
async fn magic_generate_empty_email_returns_400() {
    let app = TestApp::spawn().await;

    for input in &["", "   "] {
        let res = app
            .post_json("/auth/magic-generate/", &json!({ "email": input }))
            .await;
        assert_eq!(
            res.status, 400,
            "magic-generate con email vacío debe retornar 400: input={input:?}"
        );
    }
}

/// Email malformado → 400 (`INVALID_EMAIL`). Property-based.
#[tokio::test(flavor = "multi_thread")]
async fn magic_generate_rejects_malformed_emails() {
    let app = TestApp::spawn().await;

    let strategy = "[a-zA-Z0-9._+-]{1,24}"; // sin '@' → inválido
    let mut runner = TestRunner::new(PropConfig {
        cases: 12,
        ..PropConfig::default()
    });

    runner
        .run(&strategy.prop_map(|s| s), |email| {
            let app = &app;
            let rt = tokio::runtime::Handle::current();
            let res = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json("/auth/magic-generate/", &json!({ "email": email }))
                        .await
                })
            });
            prop_assert_eq!(
                res.status.as_u16(),
                400,
                "email inválido debe retornar 400: {:?}",
                email
            );
            Ok(())
        })
        .expect("proptest magic-generate email inválido");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/magic-sign-in
// ─────────────────────────────────────────────────────────────────────────────

/// Sign-in sin code → `error_code=5085` (MAGIC_SIGN_IN_EMAIL_CODE_REQUIRED).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_in_without_code_redirects_with_error() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-in/",
            &[("email", "magic@plane.local")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5085"),
        "error_code=5085 esperado, Location={loc:?}"
    );
}

/// Sign-in con usuario inexistente → `error_code=5060` (USER_DOES_NOT_EXIST).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_in_unknown_user_redirects_with_error() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-in/",
            &[
                ("email", "fantasma@plane.local"),
                ("code", "123456"),
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

/// Sign-in para usuario existente sin código previamente generado →
/// `error_code=5095` (EXPIRED_MAGIC_CODE_SIGN_IN).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_in_expired_code_redirects_with_error() {
    let app = TestApp::spawn().await;
    // sign-up previo + magic-sign-in dependen de instance configurada. Sin
    // ella el sign-up falla silenciosamente y magic-sign-in ve USER_DOES_NOT_EXIST
    // (5060) en lugar del EXPIRED_MAGIC_CODE_SIGN_IN (5095) que verifica el test.
    app.ensure_instance_configured().await;

    // Bootstrapea un usuario vía sign-up para que exista en la DB.
    let (s_up, _) = app
        .post_form_location(
            "/auth/sign-up/",
            &[
                ("email", "user@plane.local"),
                ("password", "Correcta1!"),
            ],
        )
        .await;
    assert!(s_up.is_redirection(), "sign-up previo debe redireccionar");

    // Intenta magic-sign-in SIN haber ejecutado magic-generate (no hay
    // entrada en Redis para ese email) → código expirado/inexistente.
    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-in/",
            &[("email", "user@plane.local"), ("code", "000000")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5095"),
        "error_code=5095 esperado, Location={loc:?}"
    );
}

/// Sign-in con código incorrecto → `error_code=5090` (INVALID_MAGIC_CODE_SIGN_IN).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_in_wrong_code_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    // Crea usuario
    let (s_up, _) = app
        .post_form_location(
            "/auth/sign-up/",
            &[
                ("email", "codetest@plane.local"),
                ("password", "Correcta1!"),
            ],
        )
        .await;
    assert!(s_up.is_redirection());

    // Genera un código válido (queda en Redis)
    let gen_res = app
        .post_json(
            "/auth/magic-generate/",
            &json!({ "email": "codetest@plane.local" }),
        )
        .await;
    assert_eq!(gen_res.status, 200);

    // Intenta sign-in con un código obviamente incorrecto
    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-in/",
            &[
                ("email", "codetest@plane.local"),
                ("code", "999999"),
            ],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5090"),
        "error_code=5090 esperado, Location={loc:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/magic-sign-up
// ─────────────────────────────────────────────────────────────────────────────

/// Sign-up sin code → `error_code=5055` (MAGIC_SIGN_UP_EMAIL_CODE_REQUIRED).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_up_without_code_redirects_with_error() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-up/",
            &[("email", "magic-new@plane.local")],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5055"),
        "error_code=5055 esperado, Location={loc:?}"
    );
}

/// Sign-up con usuario ya existente → `error_code=5030` (USER_ALREADY_EXIST).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_up_existing_user_redirects_with_error() {
    let app = TestApp::spawn().await;
    // El test depende de que el sign-up previo cree el usuario y que
    // magic-sign-up llegue a validar duplicidad. Sin instance configurada
    // el flujo corta antes y llega EXPIRED_MAGIC_CODE_SIGN_UP (5097) en
    // lugar del USER_ALREADY_EXIST (5030) esperado.
    app.ensure_instance_configured().await;

    // Crea el usuario
    let (s_up, _) = app
        .post_form_location(
            "/auth/sign-up/",
            &[
                ("email", "already@plane.local"),
                ("password", "Correcta1!"),
            ],
        )
        .await;
    assert!(s_up.is_redirection());

    // Intenta magic-sign-up para el mismo email
    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-up/",
            &[
                ("email", "already@plane.local"),
                ("code", "123456"),
            ],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5030"),
        "error_code=5030 esperado, Location={loc:?}"
    );
}

/// Sign-up sin magic-generate previo → `error_code=5097` (EXPIRED_MAGIC_CODE_SIGN_UP).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_up_expired_code_redirects_with_error() {
    let app = TestApp::spawn().await;

    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-up/",
            &[
                ("email", "fresh@plane.local"),
                ("code", "000000"),
            ],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5097"),
        "error_code=5097 esperado, Location={loc:?}"
    );
}

/// Sign-up con código incorrecto (tras magic-generate) → `error_code=5092`
/// (INVALID_MAGIC_CODE_SIGN_UP).
#[tokio::test(flavor = "multi_thread")]
async fn magic_sign_up_wrong_code_redirects_with_error() {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;

    let gen_res = app
        .post_json(
            "/auth/magic-generate/",
            &json!({ "email": "signup@plane.local" }),
        )
        .await;
    assert_eq!(gen_res.status, 200);

    let (status, loc) = app
        .post_form_location(
            "/auth/magic-sign-up/",
            &[
                ("email", "signup@plane.local"),
                ("code", "999999"),
            ],
        )
        .await;

    assert!(status.is_redirection());
    let loc = loc.expect("Location requerido");
    assert!(
        location_has_error(&loc, "5092"),
        "error_code=5092 esperado, Location={loc:?}"
    );
}
