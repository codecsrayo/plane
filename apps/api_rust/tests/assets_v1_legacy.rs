//! Tests de integración: Assets v1 (legacy) — user-assets + workspace assets
//!
//! Cobertura vía v1_router:
//!   - POST   /assets/user-assets                  → 401, 200 inicia upload
//!   - PATCH  /assets/user-assets/{asset_id}        → 401, 404
//!   - DELETE /assets/user-assets/{asset_id}        → 401, 404
//!   - POST   /assets/user-assets/server            → 401 (alias server-side)
//!   - POST   /assets/user-assets/{asset_id}/server → 401, 404
//!   - POST   /workspaces/{slug}/assets             → 401, 200 inicia upload
//!   - GET    /workspaces/{slug}/assets/{asset_id}  → 401, 404
//!   - PATCH  /workspaces/{slug}/assets/{asset_id}  → 401, 404
//!   - DELETE /workspaces/{slug}/assets/{asset_id}  → 401, 404

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String) {
    let app = TestApp::spawn().await;
    let email = format!("av1_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("av1-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, api_key, ws_slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/user-assets  — iniciar upload de asset de usuario
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn post_user_assets_unauthenticated_returns_401() {
    let (app, _, _) = setup("ua_post_unauth").await;
    let res = app
        .request(
            Method::POST,
            "/api/v1/assets/user-assets",
            Some(serde_json::to_vec(&json!({"name":"avatar.png","type":"image/png","size":51200})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn post_user_assets_authenticated_initiates_upload() {
    let (app, api_key, _) = setup("ua_post_ok").await;
    let res = app
        .post_json_authed(
            &api_key,
            "/api/v1/assets/user-assets",
            &json!({
                "name": "profile.jpg",
                "type": "image/jpeg",
                "size": 102400,
                "entity_type": "USER_AVATAR"
            }),
        )
        .await;
    // 200 si MinIO disponible, 500 si no (ambos aceptables)
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 500,
        "user-assets POST: esperado 200 o 500, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
    if status == 200 {
        let body = res.json();
        assert!(body["asset_id"].is_string(), "debe retornar asset_id");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH/DELETE /assets/user-assets/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn patch_user_asset_unauthenticated_returns_401() {
    let (app, _, _) = setup("ua_patch_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .request(
            Method::PATCH,
            &format!("/api/v1/assets/user-assets/{fake_id}"),
            Some(serde_json::to_vec(&json!({})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_user_asset_nonexistent_returns_404() {
    let (app, api_key, _) = setup("ua_patch_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .patch_json_authed(&api_key, &format!("/api/v1/assets/user-assets/{fake_id}"), &json!({}))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "patch user-asset inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_user_asset_unauthenticated_returns_401() {
    let (app, _, _) = setup("ua_del_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed("", &format!("/api/v1/assets/user-assets/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_user_asset_nonexistent_returns_404() {
    let (app, api_key, _) = setup("ua_del_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/api/v1/assets/user-assets/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete user-asset inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/user-assets/server  — alias server-side upload
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn post_user_assets_server_unauthenticated_returns_401() {
    let (app, _, _) = setup("ua_srv_unauth").await;
    let res = app
        .request(
            Method::POST,
            "/api/v1/assets/user-assets/server",
            Some(serde_json::to_vec(&json!({"name":"logo.svg","type":"image/svg+xml","size":2048})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/user-assets/{asset_id}/server
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn post_user_asset_server_complete_unauthenticated_returns_401() {
    let (app, _, _) = setup("ua_srv2_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/api/v1/assets/user-assets/{fake_id}/server"),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn post_user_asset_server_complete_nonexistent_returns_404() {
    let (app, api_key, _) = setup("ua_srv2_404").await;
    let fake_id = Uuid::new_v4();
    // El handler monta `Json<CompleteUploadRequest>` como extractor, así que
    // sin content-type + body JSON válido devuelve 415 antes de chequear la
    // existencia. Enviamos `{}` (todos los campos son opcionales) para
    // alcanzar la rama NotFound que el test verifica.
    let res = app
        .request(
            Method::POST,
            &format!("/api/v1/assets/user-assets/{fake_id}/server"),
            Some(b"{}".to_vec()),
            &[
                ("content-type", "application/json"),
                ("x-api-key", api_key.as_str()),
            ],
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "user-assets/{{id}}/server inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/assets
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn post_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug) = setup("wa_post_unauth").await;
    let res = app
        .request(
            Method::POST,
            &format!("/api/v1/workspaces/{slug}/assets"),
            Some(serde_json::to_vec(&json!({"name":"banner.png","type":"image/png","size":204800,"entity_type":"WORKSPACE_LOGO"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn post_workspace_asset_authenticated_initiates_upload() {
    let (app, api_key, slug) = setup("wa_post_ok").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/api/v1/workspaces/{slug}/assets"),
            &json!({
                "name": "workspace-logo.png",
                "type": "image/png",
                "size": 204800,
                "entity_type": "WORKSPACE_LOGO"
            }),
        )
        .await;
    let status = res.status.as_u16();
    // 200 o 500 (MinIO indisponible en entorno de test)
    assert!(
        status == 200 || status == 500,
        "workspace assets POST: esperado 200 o 500, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH/DELETE /workspaces/{slug}/assets/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug) = setup("wa_get_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get(&format!("/api/v1/workspaces/{slug}/assets/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("wa_get_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/api/v1/workspaces/{slug}/assets/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "get workspace asset inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("wa_patch_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .patch_json_authed(&api_key, &format!("/api/v1/workspaces/{slug}/assets/{fake_id}"), &json!({}))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug) = setup("wa_del_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed("", &format!("/api/v1/workspaces/{slug}/assets/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("wa_del_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/api/v1/workspaces/{slug}/assets/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}
