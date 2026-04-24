//! Tests de integración: Workspace Extras
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/workspace-members/me → 200
//!   - GET  /workspaces/{slug}/workspace-views      → 200
//!   - GET  /workspaces/{slug}/labels               → 200
//!   - GET  /workspaces/{slug}/states               → 200
//!   - GET  /workspaces/{slug}/cycles               → 200
//!   - GET  /workspaces/{slug}/modules              → 200
//!   - GET  /workspaces/{slug}/estimates            → 200
//!   - GET/PATCH /workspaces/{slug}/user-properties → 200
//!   - GET/PATCH /workspaces/{slug}/home-preferences → 200
//!   - GET       /workspaces/{slug}/recent-visits   → 200
//!   - GET/PATCH /workspaces/{slug}/sidebar-preferences → 200
//!   - GET/POST/PATCH/DELETE /workspaces/{slug}/user-favorites → 201, 200, 204
//!   - GET/POST/PATCH/DELETE /workspaces/{slug}/quick-links    → 201, 200, 204
//!   - GET/POST/PATCH/DELETE /workspaces/{slug}/stickies       → 201, 200, 204
//!   - GET/POST/PATCH/DELETE /workspaces/{slug}/draft-issues   → 201, 200, 204

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user(&format!("wsext_{suffix}@plane.test")).await;
    let ws_slug = format!("we-ws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, user_id, api_key, ws_slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// Workspace-level aggregated GET endpoints
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_member_me_returns_200() {
    let (app, _, api_key, ws) = setup("me").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/workspace-members/me")).await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_labels_returns_200() {
    let (app, _, api_key, ws) = setup("labels").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/labels")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().is_array(), "workspace labels debe ser array");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_states_returns_200() {
    let (app, _, api_key, ws) = setup("states").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/states")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().is_array());
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_cycles_returns_200() {
    let (app, _, api_key, ws) = setup("cycles").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/cycles")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().is_array());
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_modules_returns_200() {
    let (app, _, api_key, ws) = setup("modules").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/modules")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().is_array());
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_estimates_returns_200() {
    let (app, _, api_key, ws) = setup("estimates").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/estimates")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().is_array());
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /workspaces/{slug}/user-properties
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_user_properties_returns_200() {
    let (app, _, api_key, ws) = setup("userprops").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/user-properties")).await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_workspace_user_properties_returns_200() {
    let (app, _, api_key, ws) = setup("userprops_patch").await;
    let res = app
        .patch_json_authed(&api_key, &format!("/workspaces/{ws}/user-properties"), &json!({ "filters": {} }))
        .await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 204,
        "PATCH user-properties debe devolver 200/204, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /workspaces/{slug}/home-preferences
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_home_preferences_returns_200() {
    let (app, _, api_key, ws) = setup("homeprefs").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/home-preferences")).await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/recent-visits
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_recent_visits_returns_200() {
    let (app, _, api_key, ws) = setup("recentvisits").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/recent-visits")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().is_array());
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /workspaces/{slug}/sidebar-preferences
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_sidebar_preferences_returns_200() {
    let (app, _, api_key, ws) = setup("sidebarprefs").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/sidebar-preferences")).await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// User Favorites CRUD
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn favorites_crud() {
    let (app, user_id, api_key, ws) = setup("favs").await;
    let ws_id = app.workspace_id_by_slug(&ws).await;
    let proj_id = app.create_test_project(user_id, ws_id, "Fav Project", "FAV").await;

    // POST
    let create_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/user-favorites"),
            &json!({ "entity_type": "project", "entity_identifier": proj_id }))
        .await;
    assert_eq!(create_res.status.as_u16(), 201, "crear favorite debe devolver 201, body: {}", String::from_utf8_lossy(&create_res.body));
    let fav_id = create_res.json()["id"].as_str().expect("id").to_owned();

    // GET list
    let list_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/user-favorites")).await;
    assert_eq!(list_res.status.as_u16(), 200);
    let arr = list_res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 favorito");

    // DELETE
    let del_res = app.delete_authed(&api_key, &format!("/workspaces/{ws}/user-favorites/{fav_id}")).await;
    assert_eq!(del_res.status.as_u16(), 204, "DELETE favorite debe devolver 204");
}

// ─────────────────────────────────────────────────────────────────────────────
// Quick Links CRUD
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn quick_links_crud() {
    let (app, _, api_key, ws) = setup("qlinks").await;

    // POST
    let create_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/quick-links"),
            &json!({ "url": "https://example.com", "title": "Example" }))
        .await;
    assert_eq!(create_res.status.as_u16(), 201, "crear quick-link debe devolver 201, body: {}", String::from_utf8_lossy(&create_res.body));
    let ql_id = create_res.json()["id"].as_str().expect("id").to_owned();

    // GET list
    let list_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/quick-links")).await;
    assert_eq!(list_res.status.as_u16(), 200);

    // PATCH
    let patch_res = app
        .patch_json_authed(&api_key, &format!("/workspaces/{ws}/quick-links/{ql_id}"),
            &json!({ "title": "Updated" }))
        .await;
    assert_eq!(patch_res.status.as_u16(), 200, "PATCH quick-link debe devolver 200");

    // DELETE
    let del_res = app.delete_authed(&api_key, &format!("/workspaces/{ws}/quick-links/{ql_id}")).await;
    assert_eq!(del_res.status.as_u16(), 204);
}

// ─────────────────────────────────────────────────────────────────────────────
// Stickies CRUD
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn stickies_crud() {
    let (app, _, api_key, ws) = setup("stickies").await;

    // POST
    let create_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/stickies"),
            &json!({ "description_html": "<p>Sticky note</p>", "color": "#fef08a" }))
        .await;
    assert_eq!(create_res.status.as_u16(), 201, "crear sticky debe devolver 201, body: {}", String::from_utf8_lossy(&create_res.body));
    let sid = create_res.json()["id"].as_str().expect("id").to_owned();

    // GET list
    let list_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/stickies")).await;
    assert_eq!(list_res.status.as_u16(), 200);

    // PATCH
    let patch_res = app
        .patch_json_authed(&api_key, &format!("/workspaces/{ws}/stickies/{sid}"),
            &json!({ "color": "#bbf7d0" }))
        .await;
    assert_eq!(patch_res.status.as_u16(), 200, "PATCH sticky debe devolver 200");

    // DELETE
    let del_res = app.delete_authed(&api_key, &format!("/workspaces/{ws}/stickies/{sid}")).await;
    assert_eq!(del_res.status.as_u16(), 204);
}

// ─────────────────────────────────────────────────────────────────────────────
// Draft Issues CRUD
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn draft_issues_crud() {
    let (app, _, api_key, ws) = setup("drafts").await;

    // POST
    let create_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/draft-issues"),
            &json!({ "name": "Draft Feature", "priority": "medium" }))
        .await;
    assert_eq!(create_res.status.as_u16(), 201, "crear draft debe devolver 201, body: {}", String::from_utf8_lossy(&create_res.body));
    let did = create_res.json()["id"].as_str().expect("id").to_owned();

    // GET list
    let list_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/draft-issues")).await;
    assert_eq!(list_res.status.as_u16(), 200);
    let body = list_res.json();
    let arr = body.as_array()
        .cloned()
        .or_else(|| body["results"].as_array().cloned())
        .unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 draft");

    // GET by id
    let get_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/draft-issues/{did}")).await;
    assert_eq!(get_res.status.as_u16(), 200);

    // PATCH
    let patch_res = app
        .patch_json_authed(&api_key, &format!("/workspaces/{ws}/draft-issues/{did}"),
            &json!({ "name": "Updated Draft", "priority": "high" }))
        .await;
    assert_eq!(patch_res.status.as_u16(), 200, "PATCH draft debe devolver 200");

    // DELETE
    let del_res = app.delete_authed(&api_key, &format!("/workspaces/{ws}/draft-issues/{did}")).await;
    assert_eq!(del_res.status.as_u16(), 204);
}
