//! Tests de integración: API Tokens + Timezones
//!
//! API Tokens: CRUD completo sobre /api-tokens y alias /users/api-tokens.
//! Timezones:  GET /timezones — endpoint público sin auth.

mod common;

use common::TestApp;
use serde_json::json;

// ═════════════════════════════════════════════════════════════════════════════
// GET /timezones
// ═════════════════════════════════════════════════════════════════════════════

/// Devuelve 200 con una lista no vacía de zonas horarias.
#[tokio::test(flavor = "multi_thread")]
async fn list_timezones_returns_200_with_data() {
    let app = TestApp::spawn().await;
    let res = app.get("/timezones").await;

    assert_eq!(res.status.as_u16(), 200, "GET /timezones debe devolver 200");
    let body = res.json();
    // Paridad con Django TimezoneEndpoint (apps/api/.../timezone/base.py): la
    // respuesta es `{"timezones": [...]}`, no un array al tope.
    let arr = body["timezones"]
        .as_array()
        .expect("body.timezones debe ser array");
    assert!(!arr.is_empty(), "debe haber al menos una timezone");
    // TimezoneEntry tiene campos {utc_offset, gmt_offset, label, value}.
    // `utc_offset` siempre es `"UTC±HH:MM"`, así que al menos una entrada debe
    // contener "UTC" ahí. Dejamos los otros caminos como fallback defensivo
    // por si Django cambia el shape.
    let has_utc = arr.iter().any(|tz| {
        tz["utc_offset"].as_str().map(|s| s.contains("UTC")).unwrap_or(false)
            || tz.as_str().map(|s| s == "UTC" || s.contains("UTC")).unwrap_or(false)
            || tz["value"].as_str().map(|s| s.contains("UTC")).unwrap_or(false)
    });
    assert!(has_utc, "al menos una timezone debe reportar offset UTC");
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /api-tokens
// ═════════════════════════════════════════════════════════════════════════════

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn list_api_tokens_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/api-tokens").await;
    assert_eq!(res.status.as_u16(), 401);
}

/// Usuario recién creado tiene al menos 1 token (el de test).
#[tokio::test(flavor = "multi_thread")]
async fn list_api_tokens_returns_own_tokens() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("listtokens@plane.test").await;

    let res = app.get_authed(&api_key, "/api-tokens").await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos el token de test");
}

// ═════════════════════════════════════════════════════════════════════════════
// POST /api-tokens
// ═════════════════════════════════════════════════════════════════════════════

/// Creación sin label → 201, label auto-generado.
#[tokio::test(flavor = "multi_thread")]
async fn create_api_token_no_label_returns_201() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createtoken@plane.test").await;

    let res = app
        .post_json_authed(&api_key, "/api-tokens", &json!({}))
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    // El token generado es el raw value (se expone solo al crear)
    assert!(body["token"].as_str().map(|t| t.len() >= 32).unwrap_or(false),
        "token generado debe tener ≥32 chars");
}

/// Creación con label explícito → 201, label preservado.
#[tokio::test(flavor = "multi_thread")]
async fn create_api_token_with_label_returns_201() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createtoken_lbl@plane.test").await;

    let res = app
        .post_json_authed(&api_key, "/api-tokens", &json!({ "label": "CI Token", "description": "Used in CI" }))
        .await;
    assert_eq!(res.status.as_u16(), 201);
    let body = res.json();
    assert_eq!(body["label"].as_str().unwrap_or(""), "CI Token");
    assert_eq!(body["description"].as_str().unwrap_or(""), "Used in CI");
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /api-tokens/{pk}
// ═════════════════════════════════════════════════════════════════════════════

/// GET token por ID → 200 con label correcto.
#[tokio::test(flavor = "multi_thread")]
async fn get_api_token_by_id_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("gettoken@plane.test").await;

    let created = app
        .post_json_authed(&api_key, "/api-tokens", &json!({ "label": "My Token" }))
        .await;
    let token_id = created.json()["id"].as_str().expect("id").to_owned();

    let res = app.get_authed(&api_key, &format!("/api-tokens/{token_id}")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["label"].as_str().unwrap_or(""), "My Token");
}

/// ID inexistente → 404.
#[tokio::test(flavor = "multi_thread")]
async fn get_api_token_not_found_returns_404() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("gettoken404@plane.test").await;
    let fake = uuid::Uuid::new_v4();
    let res = app.get_authed(&api_key, &format!("/api-tokens/{fake}")).await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// PATCH /api-tokens/{pk}
// ═════════════════════════════════════════════════════════════════════════════

/// PATCH label → 200 con label actualizado.
#[tokio::test(flavor = "multi_thread")]
async fn update_api_token_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("updatetoken@plane.test").await;

    let created = app
        .post_json_authed(&api_key, "/api-tokens", &json!({ "label": "Old Label" }))
        .await;
    let token_id = created.json()["id"].as_str().expect("id").to_owned();

    let res = app
        .patch_json_authed(&api_key, &format!("/api-tokens/{token_id}"), &json!({ "label": "New Label" }))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["label"].as_str().unwrap_or(""), "New Label");
}

// ═════════════════════════════════════════════════════════════════════════════
// DELETE /api-tokens/{pk}
// ═════════════════════════════════════════════════════════════════════════════

/// DELETE → 204.
#[tokio::test(flavor = "multi_thread")]
async fn delete_api_token_returns_204() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("deletetoken@plane.test").await;

    let created = app
        .post_json_authed(&api_key, "/api-tokens", &json!({ "label": "Ephemeral" }))
        .await;
    let token_id = created.json()["id"].as_str().expect("id").to_owned();

    let res = app.delete_authed(&api_key, &format!("/api-tokens/{token_id}")).await;
    assert_eq!(res.status.as_u16(), 204);

    // Verificar que ya no es accesible
    let after = app.get_authed(&api_key, &format!("/api-tokens/{token_id}")).await;
    assert_eq!(after.status.as_u16(), 404, "token borrado no debe ser accesible");
}

// ═════════════════════════════════════════════════════════════════════════════
// /users/api-tokens alias
// ═════════════════════════════════════════════════════════════════════════════

/// El alias /users/api-tokens devuelve los mismos tokens.
#[tokio::test(flavor = "multi_thread")]
async fn users_api_tokens_alias_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("aliastoken@plane.test").await;

    let res = app.get_authed(&api_key, "/users/api-tokens").await;
    assert_eq!(res.status.as_u16(), 200);
}
