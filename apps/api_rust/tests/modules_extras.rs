//! Tests de integración: Modules extras
//!
//! Cobertura:
//!   - GET/PATCH  /modules/{module_id}/user-properties
//!   - POST       /issues/{issue_id}/modules  (set issue modules)
//!   - GET/POST   /modules/{module_id}/module-links
//!   - GET/PATCH/DELETE /modules/{module_id}/module-links/{pk}
//!   - GET/POST   /user-favorite-modules
//!   - DELETE     /user-favorite-modules/{module_id}
//!   - GET/DELETE /archived-modules/{pk}  (get + unarchive)

mod common;

use common::TestApp;
use serde_json::json;

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("mex_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("mx-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "ModuleExt Project", "MEX")
        .await;
    (app, api_key, ws_slug, proj_id)
}

async fn create_module(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear module falló: {}", String::from_utf8_lossy(&res.body));
    res.json()["id"].as_str().unwrap().to_owned()
}

async fn create_issue(app: &TestApp, api_key: &str, ws_slug: &str, proj_id: uuid::Uuid) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": "Module Issue" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// USER-PROPERTIES — GET/PATCH /modules/{module_id}/user-properties
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_module_user_properties_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("uprops_get").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Props Module").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/user-properties"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_module_user_properties_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("uprops_patch").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Patch Props Module").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/user-properties"),
            &json!({ "display_properties": { "priority": true } }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ═════════════════════════════════════════════════════════════════════════════
// SET ISSUE MODULES — POST /issues/{issue_id}/modules
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn set_issue_modules_returns_200_or_201() {
    let (app, api_key, ws_slug, proj_id) = setup("set_modules").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id).await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Set Modules Module").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/modules"),
            &json!({ "modules": [mod_id] }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "set-issue-modules debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// MODULE LINKS — GET/POST + GET/PATCH/DELETE /{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_module_links_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("links_list").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Links Module").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty(), "sin links debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_module_link_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("link_create").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Link Create Module").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links"),
            &json!({ "title": "Plane Docs", "url": "https://docs.plane.so" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["url"].as_str().unwrap_or(""), "https://docs.plane.so");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_module_link_invalid_url_returns_400() {
    let (app, api_key, ws_slug, proj_id) = setup("link_bad").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Bad Link Module").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links"),
            &json!({ "title": "bad", "url": "not-a-url" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

async fn create_module_link(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    mod_id: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links"),
            &json!({ "title": "A Link", "url": "https://example.com" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn get_module_link_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("link_get").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Get Link Module").await;
    let link_id = create_module_link(&app, &api_key, &ws_slug, proj_id, &mod_id).await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links/{link_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_module_link_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("link_patch").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Patch Link Module").await;
    let link_id = create_module_link(&app, &api_key, &ws_slug, proj_id, &mod_id).await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links/{link_id}"),
            &json!({ "title": "Updated Link", "url": "https://updated.example.com" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["title"].as_str().unwrap_or(""), "Updated Link");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_module_link_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("link_del").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Del Link Module").await;
    let link_id = create_module_link(&app, &api_key, &ws_slug, proj_id, &mod_id).await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/module-links/{link_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// FAVORITE MODULES — GET/POST + DELETE /{module_id}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_favorite_modules_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("fav_list").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-modules"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty(), "sin favoritos debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn add_favorite_module_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("fav_add").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Fav Module").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-modules"),
            &json!({ "module": mod_id }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "add-favorite-module debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_favorite_module_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("fav_del").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Del Fav Module").await;

    // Añadir a favoritos
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-modules"),
        &json!({ "module": mod_id }),
    )
    .await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-modules/{mod_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 204 || status == 200,
        "delete-favorite-module debe devolver 204/200, obtuvo {status}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// ARCHIVED MODULES — GET + DELETE (unarchive) /archived-modules/{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_module_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("arch_get").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Archive Me Module").await;

    // Archivar
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/archive"),
        &json!({}),
    )
    .await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-modules/{mod_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_module_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("arch_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-modules/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_module_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("unarch").await;
    let mod_id = create_module(&app, &api_key, &ws_slug, proj_id, "Unarchive Module").await;

    // Archivar
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/modules/{mod_id}/archive"),
        &json!({}),
    )
    .await;

    // Desarchivar via DELETE en archived-modules/{pk}
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-modules/{mod_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 204 || status == 200,
        "unarchive module debe devolver 204/200, obtuvo {status}"
    );
}
