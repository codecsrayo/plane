//! Tests de integración: endpoints `/users/me`, `/users/session`,
//! `/users/me/settings`.
//!
//! Todos los endpoints requieren autenticación vía `x-api-key` (AnyAuth).
//! El helper `TestApp::create_test_user` inserta el par (user, api_token)
//! directamente en la DB para mantener los tests independientes del flujo
//! de sign-up.
//!
//! Cobertura:
//!   - GET  /users/me              → 401 sin auth, 200 con auth
//!   - PATCH /users/me             → 200 actualiza, 400 en display_name inválido
//!   - GET  /users/session         → 200 is_authenticated=false / true
//!   - GET  /users/me/settings     → 401 sin auth, 200 con auth
//!   - Proptest: PATCH display_name con strings válidos → siempre 200

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me
// ─────────────────────────────────────────────────────────────────────────────

/// Sin `x-api-key` → 401.
#[tokio::test(flavor = "multi_thread")]
async fn get_me_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/users/me").await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "GET /users/me sin auth debe devolver 401, obtuvo {}",
        res.status
    );
}

/// Con API key válida → 200 y el campo `email` coincide.
#[tokio::test(flavor = "multi_thread")]
async fn get_me_authenticated_returns_user() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("alice@plane.test").await;

    let res = app.get_authed(&api_key, "/users/me").await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /users/me con auth debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );

    let body = res.json();
    assert_eq!(
        body["email"].as_str().unwrap_or(""),
        "alice@plane.test",
        "El campo email no coincide"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /users/me
// ─────────────────────────────────────────────────────────────────────────────

/// PATCH con `display_name` válido → 200 y campo reflejado en respuesta.
#[tokio::test(flavor = "multi_thread")]
async fn patch_me_updates_display_name() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("bob@plane.test").await;

    let res = app
        .patch_json_authed(
            &api_key,
            "/users/me",
            &json!({ "display_name": "Bob Updated" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "PATCH /users/me debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );

    let body = res.json();
    assert_eq!(
        body["display_name"].as_str().unwrap_or(""),
        "Bob Updated",
        "display_name no se actualizó en la respuesta"
    );
}

/// PATCH con `display_name` vacío (solo espacios) → 400.
///
/// El handler valida que `trimmed` no sea vacío (≥1 char).
#[tokio::test(flavor = "multi_thread")]
async fn patch_me_empty_display_name_returns_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("carol@plane.test").await;

    let res = app
        .patch_json_authed(
            &api_key,
            "/users/me",
            &json!({ "display_name": "   " }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "PATCH con display_name vacío debe devolver 400, obtuvo {}",
        res.status
    );
}

/// PATCH con `display_name` de 256 caracteres → 400 (máximo es 255).
#[tokio::test(flavor = "multi_thread")]
async fn patch_me_display_name_too_long_returns_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("dave@plane.test").await;

    let long_name = "x".repeat(256);
    let res = app
        .patch_json_authed(
            &api_key,
            "/users/me",
            &json!({ "display_name": long_name }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "PATCH con display_name de 256 chars debe devolver 400, obtuvo {}",
        res.status
    );
}

/// Proptest: PATCH con strings de entre 1 y 200 chars alfanuméricos siempre → 200.
///
/// Verifica que el handler no rompe con inputs arbitrarios dentro del límite.
#[tokio::test(flavor = "multi_thread")]
async fn patch_me_valid_display_name_always_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("proptest_user@plane.test").await;

    let strategy = "[a-zA-Z0-9 ]{1,200}";
    let mut runner = TestRunner::new(PropConfig {
        cases: 15,
        ..PropConfig::default()
    });

    runner
        .run(&strategy.prop_filter("no vacío después de trim", |s| !s.trim().is_empty()), |name| {
            let app = &app;
            let api_key = &api_key;
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.patch_json_authed(api_key, "/users/me", &json!({ "display_name": name }))
                        .await
                        .status
                        .as_u16()
                })
            });
            prop_assert_eq!(
                status,
                200,
                "display_name válido debe devolver 200, obtuvo {status} para input={name:?}"
            );
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/session
// ─────────────────────────────────────────────────────────────────────────────

/// Sin credenciales → 200 con `is_authenticated: false`.
///
/// Este endpoint usa `OptionalAnyAuth` — no rechaza con 401.
#[tokio::test(flavor = "multi_thread")]
async fn get_session_unauthenticated_returns_not_authenticated() {
    let app = TestApp::spawn().await;
    let res = app.get("/users/session").await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /users/session sin auth debe devolver 200, obtuvo {}",
        res.status
    );

    let body = res.json();
    assert_eq!(
        body["is_authenticated"].as_bool(),
        Some(false),
        "is_authenticated debe ser false sin credenciales"
    );
    assert!(
        body["user"].is_null(),
        "user debe ser null cuando no autenticado"
    );
}

/// Con API key válida → 200 con `is_authenticated: true` y datos de usuario.
#[tokio::test(flavor = "multi_thread")]
async fn get_session_authenticated_returns_user_data() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("eve@plane.test").await;

    let res = app.get_authed(&api_key, "/users/session").await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /users/session con auth debe devolver 200, obtuvo {}",
        res.status
    );

    let body = res.json();
    assert_eq!(
        body["is_authenticated"].as_bool(),
        Some(true),
        "is_authenticated debe ser true con credenciales válidas"
    );
    assert!(
        !body["user"].is_null(),
        "user no debe ser null cuando autenticado"
    );
    assert_eq!(
        body["user"]["email"].as_str().unwrap_or(""),
        "eve@plane.test"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/settings
// ─────────────────────────────────────────────────────────────────────────────

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn get_settings_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/users/me/settings").await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "GET /users/me/settings sin auth debe devolver 401, obtuvo {}",
        res.status
    );
}

/// Con auth → 200 y respuesta JSON válida.
#[tokio::test(flavor = "multi_thread")]
async fn get_settings_authenticated_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("frank@plane.test").await;

    let res = app.get_authed(&api_key, "/users/me/settings").await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /users/me/settings debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );

    // La respuesta debe ser JSON parseable
    let _ = res.json();
}
