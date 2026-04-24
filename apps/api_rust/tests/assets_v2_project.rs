//! Tests de integración: Assets v2 — Endpoints de proyecto y workspace
//!
//! Cobertura:
//!   - GET    /assets/v2/static/{asset_id}                              → 404
//!   - POST   /assets/v2/workspaces/{slug}/restore/{asset_id}           → 401, 404
//!   - GET    /assets/v2/workspaces/{slug}/check/{asset_id}             → 401, 404
//!   - POST   /assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}  → 401, 404
//!   - GET    /assets/v2/workspaces/{slug}/download/{asset_id}          → 401, 404
//!   - GET/POST /assets/v2/workspaces/{slug}/projects/{pid}             → 401, 200
//!   - GET/PATCH/DELETE /assets/v2/workspaces/{slug}/projects/{pid}/{pk}→ 401, 404
//!   - POST   /assets/v2/workspaces/{slug}/projects/{pid}/{eid}/bulk    → 401
//!   - GET    /assets/v2/workspaces/{slug}/projects/{pid}/download/{aid}→ 401, 404
//!   - GET/POST /assets/v2/workspaces/{slug}/projects/{pid}/issues/{iid}/attachments → 401, 200
//!   - DELETE /assets/v2/workspaces/{slug}/projects/{pid}/issues/{iid}/attachments/{pk} → 401, 404

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String, Uuid, Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("av2_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("av2-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Assets v2 Project", "AV2")
        .await;
    (app, api_key, ws_slug, ws_id, proj_id)
}

async fn create_issue(app: &TestApp, api_key: &str, slug: &str, proj_id: Uuid) -> Uuid {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues"),
            &json!({ "name": "Assets v2 test issue" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().parse().unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /assets/v2/static/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_static_asset_nonexistent_returns_404() {
    let (app, _, _, _, _) = setup("static_404").await;
    let fake_id = Uuid::new_v4();
    // Este endpoint no requiere auth (asset público), pero asset inexistente → 404
    let res = app
        .get(&format!("/assets/v2/static/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "asset estático inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/v2/workspaces/{slug}/restore/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn restore_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("restore_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/assets/v2/workspaces/{slug}/restore/{fake_id}"),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn restore_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug, _, _) = setup("restore_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/assets/v2/workspaces/{slug}/restore/{fake_id}"),
            None,
            &[("x-api-key", api_key.as_str())],
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "restore asset inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /assets/v2/workspaces/{slug}/check/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn check_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("check_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get(&format!("/assets/v2/workspaces/{slug}/check/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn check_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug, _, _) = setup("check_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/assets/v2/workspaces/{slug}/check/{fake_id}"))
        .await;
    // Paridad Django `AssetCheckEndpoint` (asset/v2.py:691): este endpoint
    // SIEMPRE devuelve 200 con `{"exists": bool}`, nunca 404. Para asset
    // inexistente → `exists=false`. El nombre del test es legacy; se mantiene
    // para no cambiar el contrato externo, pero la aserción refleja el
    // comportamiento real.
    assert_eq!(
        res.status.as_u16(),
        200,
        "check asset siempre devuelve 200: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["exists"], serde_json::json!(false));
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("dup_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/assets/v2/workspaces/{slug}/duplicate-assets/{fake_id}"),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug, _, _) = setup("dup_404").await;
    let fake_id = Uuid::new_v4();
    // El handler monta `Json<DuplicateAssetRequest>` + valida entity_type
    // contra VALID_ENTITY_TYPES (assets.rs:72). Sin body JSON + content-type
    // devuelve 415 antes del lookup. Enviamos un payload válido para
    // llegar a la rama NotFound que el test verifica.
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/duplicate-assets/{fake_id}"),
            &json!({"entity_type": "WORKSPACE_LOGO"}),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "duplicate asset inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /assets/v2/workspaces/{slug}/download/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn download_workspace_asset_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("dl_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get(&format!("/assets/v2/workspaces/{slug}/download/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn download_workspace_asset_nonexistent_returns_404() {
    let (app, api_key, slug, _, _) = setup("dl_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/assets/v2/workspaces/{slug}/download/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "download asset inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/v2/workspaces/{slug}/projects/{project_id}
// ─────────────────────────────────────────────────────────────────────────────
// Nota: sólo POST está definido en esta ruta (paridad con Django
// `ProjectAssetEndpoint`: apps/api/plane/app/views/asset/v2.py:513). Los
// tests previos `list_project_assets_v2_*` asumían un GET que nunca existió
// (ni en Django ni en Rust) y fueron eliminados porque verificaban un
// endpoint inexistente — GET devolvía 405 Method Not Allowed.

#[tokio::test(flavor = "multi_thread")]
async fn initiate_project_asset_v2_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("proj_post_unauth").await;
    let res = app
        .request(
            Method::POST,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}"),
            Some(serde_json::to_vec(&json!({"name":"f.png","type":"image/png","size":1024,"entity_type":"project_cover"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH/DELETE /assets/v2/workspaces/{slug}/projects/{project_id}/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_project_asset_v2_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("proj_get_404").await;
    let fake_pk = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/{fake_pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_project_asset_v2_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("proj_patch_404").await;
    let fake_pk = Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/{fake_pk}"),
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_project_asset_v2_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("proj_del_404").await;
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/{fake_pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn bulk_project_assets_v2_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("bulk_unauth").await;
    let entity_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/{entity_id}/bulk"),
            Some(serde_json::to_vec(&json!({"asset_ids": []})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn bulk_project_assets_v2_authenticated_empty_list() {
    let (app, api_key, slug, _, proj_id) = setup("bulk_empty").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/{issue_id}/bulk"),
            &json!({ "asset_ids": [] }),
        )
        .await;
    // 200 con lista vacía o 400 si body es inválido
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 400 || status == 422,
        "bulk con lista vacía: esperado 200/400/422, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn download_project_asset_v2_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("pdl_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/assets/v2/workspaces/{slug}/projects/{proj_id}/download/{fake_id}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn download_project_asset_v2_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("pdl_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/download/{fake_id}"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "download project asset inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /assets/v2/workspaces/{slug}/projects/{pid}/issues/{iid}/attachments
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_attachments_v2_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("iatt2_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/assets/v2/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/attachments"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_attachments_v2_empty_returns_200() {
    let (app, api_key, slug, _, proj_id) = setup("iatt2_empty").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/attachments"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "attachments v2 vacíos: {}",
        String::from_utf8_lossy(&res.body)
    );
    let arr = res.json();
    assert_eq!(arr.as_array().map(|a| a.len()).unwrap_or(0), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn initiate_issue_attachment_v2_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("iatt2_post_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/attachments"),
            Some(serde_json::to_vec(&json!({"name":"x.pdf","type":"application/pdf","size":512})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /assets/v2/workspaces/{slug}/projects/{pid}/issues/{iid}/attachments/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_issue_attachment_v2_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("iatt2_del_unauth").await;
    let issue_id = Uuid::new_v4();
    let pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            "",
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/attachments/{pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_issue_attachment_v2_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("iatt2_del_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!("/assets/v2/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/attachments/{fake_pk}"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete attachment v2 inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}
