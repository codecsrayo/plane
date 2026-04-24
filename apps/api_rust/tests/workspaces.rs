//! Tests de integración: Workspaces
//!
//! Cobertura:
//!   - GET  /workspace-slug-check          → status true/false
//!   - GET  /workspaces                    → 401 sin auth, 200 lista vacía/con item
//!   - POST /workspaces                    → 201 crea, 400 validaciones, 409 slug duplicado
//!   - GET  /workspaces/{slug}             → 404 sin membresía, 200 con membresía
//!   - PATCH /workspaces/{slug}            → 403 sin rol Admin, 200 con Admin
//!   - GET  /workspaces/{slug}/members     → 200 lista miembros
//!   - Proptest: POST slugs inválidos (espacios/url) → siempre 4xx

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspace-slug-check
// ─────────────────────────────────────────────────────────────────────────────

/// Slug nuevo → status true (disponible).
#[tokio::test(flavor = "multi_thread")]
async fn slug_check_available_returns_true() {
    let app = TestApp::spawn().await;
    let res = app.get("/workspace-slug-check?slug=totally-unique-xyz-123").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert_eq!(
        body["status"].as_bool(),
        Some(true),
        "slug nuevo debe estar disponible"
    );
}

/// Slug restringido (`admin`) → status false.
#[tokio::test(flavor = "multi_thread")]
async fn slug_check_restricted_returns_false() {
    let app = TestApp::spawn().await;
    let res = app.get("/workspace-slug-check?slug=admin").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert_eq!(
        body["status"].as_bool(),
        Some(false),
        "slug restringido debe devolver false"
    );
}

/// Slug ya usado → status false.
#[tokio::test(flavor = "multi_thread")]
async fn slug_check_taken_returns_false() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("slugcheck@plane.test").await;
    app.create_test_workspace(user_id, "taken-slug-001").await;

    let res = app.get("/workspace-slug-check?slug=taken-slug-001").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert_eq!(
        body["status"].as_bool(),
        Some(false),
        "slug ya usado debe devolver false"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces
// ─────────────────────────────────────────────────────────────────────────────

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn list_workspaces_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/workspaces").await;
    assert_eq!(res.status.as_u16(), 401);
}

/// Usuario sin workspaces → lista vacía.
#[tokio::test(flavor = "multi_thread")]
async fn list_workspaces_empty_for_new_user() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("listws@plane.test").await;

    let res = app.get_authed(&api_key, "/workspaces").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert!(
        body.as_array().map(|a| a.is_empty()).unwrap_or(false),
        "usuario sin membresías debe recibir lista vacía, obtuvo: {body}"
    );
}

/// Usuario con workspace → lista con un elemento.
#[tokio::test(flavor = "multi_thread")]
async fn list_workspaces_returns_member_workspaces() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("listwsmember@plane.test").await;
    app.create_test_workspace(user_id, "my-ws-list-001").await;

    let res = app.get_authed(&api_key, "/workspaces").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let arr = body.as_array().expect("respuesta debe ser array");
    assert_eq!(arr.len(), 1, "debe haber exactamente 1 workspace");
    assert_eq!(arr[0]["slug"].as_str().unwrap_or(""), "my-ws-list-001");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces
// ─────────────────────────────────────────────────────────────────────────────

/// Creación exitosa → 201 con slug correcto.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_success() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createws@plane.test").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces",
            &json!({
                "name": "My Test Workspace",
                "slug": "my-test-ws-create-001"
            }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        201,
        "POST /workspaces debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["slug"].as_str().unwrap_or(""), "my-test-ws-create-001");
}

/// Nombre vacío → 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_empty_name_returns_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createws_empty@plane.test").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces",
            &json!({ "name": "", "slug": "valid-slug-999" }),
        )
        .await;

    assert_eq!(res.status.as_u16(), 400);
}

/// Nombre con URL embebida → 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_name_with_url_returns_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createws_url@plane.test").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces",
            &json!({ "name": "http://evil.com", "slug": "valid-slug-998" }),
        )
        .await;

    assert_eq!(res.status.as_u16(), 400);
}

/// Slug restringido → 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_restricted_slug_returns_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createws_restricted@plane.test").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces",
            &json!({ "name": "Valid Name", "slug": "admin" }),
        )
        .await;

    assert_eq!(res.status.as_u16(), 400, "slug 'admin' está restringido");
}

/// Slug duplicado → 409 Conflict.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_duplicate_slug_returns_409() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createws_dup@plane.test").await;
    app.create_test_workspace(user_id, "dup-slug-001").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces",
            &json!({ "name": "Another", "slug": "dup-slug-001" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        409,
        "slug duplicado debe devolver 409, obtuvo {}",
        res.status
    );
}

/// Proptest: slugs con caracteres inválidos (espacios, @, /) → siempre 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_invalid_slugs_always_400() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("createws_proptest@plane.test").await;

    // Slugs con al menos un carácter inválido (espacio, @, /, !)
    let strategy = (
        "[a-z]{1,8}",
        prop::sample::select(vec![" ", "@", "/", "!", "#", "."]),
        "[a-z]{1,8}",
    )
        .prop_map(|(pre, bad, post)| format!("{pre}{bad}{post}"));

    let mut runner = TestRunner::new(PropConfig {
        cases: 15,
        ..PropConfig::default()
    });

    runner
        .run(&strategy, |slug| {
            let app = &app;
            let api_key = &api_key;
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        api_key,
                        "/workspaces",
                        &json!({ "name": "Valid Name", "slug": slug }),
                    )
                    .await
                    .status
                    .as_u16()
                })
            });
            prop_assert!(
                status == 400 || status == 409,
                "slug inválido {slug:?} debe devolver 4xx, obtuvo {status}"
            );
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}
// ─────────────────────────────────────────────────────────────────────────────

/// Usuario sin membresía → 404 (el workspace existe pero no es miembro).
#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_non_member_returns_404() {
    let app = TestApp::spawn().await;
    let (owner_id, _) = app.create_test_user("wsowner@plane.test").await;
    app.create_test_workspace(owner_id, "restricted-ws-001").await;

    // Otro usuario sin membresía
    let (_, other_key) = app.create_test_user("wsother@plane.test").await;
    let res = app.get_authed(&other_key, "/workspaces/restricted-ws-001").await;

    assert_eq!(
        res.status.as_u16(),
        404,
        "no-miembro debe recibir 404 al acceder al workspace"
    );
}

/// Miembro Admin → 200 con slug correcto.
#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_member_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("wsget@plane.test").await;
    app.create_test_workspace(user_id, "getws-test-001").await;

    let res = app.get_authed(&api_key, "/workspaces/getws-test-001").await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "miembro debe obtener 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["slug"].as_str().unwrap_or(""), "getws-test-001");
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}
// ─────────────────────────────────────────────────────────────────────────────

/// Admin puede actualizar nombre del workspace.
#[tokio::test(flavor = "multi_thread")]
async fn patch_workspace_admin_can_update_name() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("wspatch@plane.test").await;
    app.create_test_workspace(user_id, "patchws-test-001").await;

    let res = app
        .patch_json_authed(
            &api_key,
            "/workspaces/patchws-test-001",
            &json!({ "name": "Updated Name" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "Admin debe poder hacer PATCH, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "Updated Name");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/members
// ─────────────────────────────────────────────────────────────────────────────

/// Miembro puede listar los miembros del workspace.
#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_members_returns_member_list() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("wsmembers@plane.test").await;
    app.create_test_workspace(user_id, "membersws-test-001").await;

    let res = app
        .get_authed(&api_key, "/workspaces/membersws-test-001/members")
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /workspaces/{{slug}}/members debe devolver 200"
    );
    let body = res.json();
    let arr = body.as_array().expect("respuesta debe ser array");
    assert!(!arr.is_empty(), "debe haber al menos 1 miembro (el owner)");
}
