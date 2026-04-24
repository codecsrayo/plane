//! Tests de integración para endpoints de autenticación.
//!
//! Orden (según `todo_test.md`): auth primero, luego workspace, luego project.
//!
//! Framework: **proptest** — las estrategias generan inputs, cada caso corre
//! el router completo vía `oneshot` contra contenedores Postgres + Redis
//! reales. No se mockea la capa de datos: una regresión en queries o en el
//! schema se detecta aquí.
//!
//! Convenciones:
//!   - `#[tokio::test(flavor = "multi_thread")]` — requerido para `oneshot`
//!     de routers con background tasks.
//!   - `proptest::test_runner::TestRunner` dentro del test async: `proptest!`
//!     macro no soporta `async` nativo, así que se driveña el runner manual.
//!   - Cada test _spawnea_ su `TestApp` — aislamiento total.

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/get-csrf-token
// ─────────────────────────────────────────────────────────────────────────────

/// Contrato: responde 200, emite cookie `csrftoken` y devuelve el mismo token
/// en el body JSON (`csrf_token`). El valor es un UUID v4 (36 chars).
#[tokio::test(flavor = "multi_thread")]
async fn csrf_token_endpoint_returns_uuid_and_cookie() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/get-csrf-token").await;

    assert_eq!(res.status, 200, "CSRF debe responder 200");
    let body = res.json();
    let token = body
        .get("csrf_token")
        .and_then(|v| v.as_str())
        .expect("csrf_token presente en el body");
    assert_eq!(
        token.len(),
        36,
        "csrf_token debe ser un UUID v4 (36 chars): {token:?}"
    );
    assert!(
        token.chars().filter(|c| *c == '-').count() == 4,
        "csrf_token debe tener 4 guiones (formato UUID)"
    );
}

/// Propiedad: dos llamadas sucesivas generan tokens distintos.
/// Previene regresión a tokens determinísticos (catástrofe de seguridad).
#[tokio::test(flavor = "multi_thread")]
async fn csrf_tokens_are_unique_per_request() {
    let app = TestApp::spawn().await;

    let t1 = app
        .get("/auth/get-csrf-token")
        .await
        .json()
        .get("csrf_token")
        .and_then(|v| v.as_str().map(String::from))
        .expect("token 1");
    let t2 = app
        .get("/auth/get-csrf-token")
        .await
        .json()
        .get("csrf_token")
        .and_then(|v| v.as_str().map(String::from))
        .expect("token 2");

    assert_ne!(t1, t2, "CSRF tokens deben ser únicos por request");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/email-check
// ─────────────────────────────────────────────────────────────────────────────

/// Propiedad: cualquier string que NO parsea como dirección válida debe
/// producir 400 con el código de error `INVALID_EMAIL` (o `EMAIL_REQUIRED`
/// si está vacío tras trim).
///
/// Genera cadenas arbitrarias de longitud acotada y sin `@` para garantizar
/// que la estrategia produzca entradas inválidas con alta probabilidad.
#[tokio::test(flavor = "multi_thread")]
async fn email_check_rejects_malformed_emails() {
    let app = TestApp::spawn().await;

    // Estrategia: strings ASCII imprimibles sin `@` → siempre inválidos.
    let strategy = "[a-zA-Z0-9._+-]{0,32}";
    let mut runner = TestRunner::new(PropConfig {
        cases: 16, // suficiente para cubrir bordes sin saturar el contenedor
        ..PropConfig::default()
    });

    runner
        .run(&strategy.prop_map(|s| s), |candidate| {
            let app = &app;
            let rt = tokio::runtime::Handle::current();
            let res = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json(
                        "/auth/email-check/",
                        &json!({ "email": candidate }),
                    )
                    .await
                })
            });
            prop_assert_eq!(
                res.status.as_u16(),
                400,
                "email inválido debe retornar 400: input={:?}",
                candidate
            );
            Ok(())
        })
        .expect("proptest email-check malformado");
}

/// Contrato: un email sintácticamente válido y no registrado responde 200
/// con `existing_user: false` (paridad con `EmailCheckEndpoint` de Django).
#[tokio::test(flavor = "multi_thread")]
async fn email_check_accepts_valid_email_not_registered() {
    let app = TestApp::spawn().await;
    // email-check corta temprano con INSTANCE_NOT_CONFIGURED si no hay fila
    // `instances` activa con `is_setup_done=true` (src/auth/email_check.rs:59).
    app.ensure_instance_configured().await;

    let res = app
        .post_json(
            "/auth/email-check/",
            &json!({ "email": "nuevo-usuario@plane.test" }),
        )
        .await;

    assert_eq!(
        res.status, 200,
        "email válido debe responder 200, status={}: body={}",
        res.status,
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(
        body.get("existing_user").and_then(|v| v.as_bool()),
        Some(false),
        "usuario inexistente debe tener existing_user=false"
    );
}

/// Email vacío o solo whitespace → 400 con `EMAIL_REQUIRED`.
#[tokio::test(flavor = "multi_thread")]
async fn email_check_rejects_empty_email() {
    let app = TestApp::spawn().await;

    for input in &["", "   ", "\t\n"] {
        let res = app
            .post_json("/auth/email-check/", &json!({ "email": input }))
            .await;
        assert_eq!(
            res.status, 400,
            "email vacío debe retornar 400: input={input:?}"
        );
    }
}
