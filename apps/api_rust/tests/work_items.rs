//! Tests de integración: Work Items (prefijo `/work-items/`)
//!
//! Los work-items son el nuevo prefijo semántico del v1 router de Plane.
//! Usan los mismos handlers que `/issues/` pero con path `/work-items/`.
//!
//! Cobertura:
//!   - GET/POST         /projects/{project_id}/work-items
//!   - GET/PATCH/DELETE /projects/{project_id}/work-items/{pk}
//!   - GET/POST         /projects/{project_id}/work-items/{id}/links
//!   - PATCH/DELETE     /projects/{project_id}/work-items/{id}/links/{pk}
//!   - GET/POST         /projects/{project_id}/work-items/{id}/comments
//!   - GET/PATCH/DELETE /projects/{project_id}/work-items/{id}/comments/{pk}
//!   - GET              /projects/{project_id}/work-items/{id}/activities
//!   - GET/POST         /projects/{project_id}/work-items/{id}/relations

mod common;

use common::TestApp;
use serde_json::json;

// Los work-items están montados bajo /api/v1 — el TestApp usa paths sin
// prefijo pero el router los sirve bajo /api/v1. Los tests siguen el
// mismo patrón que el resto de la suite (path sin /api/v1).
const V1: &str = "/api/v1";

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("wi_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("wi-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "WorkItems Project", "WIT")
        .await;
    (app, api_key, ws_slug, proj_id)
}

async fn create_work_item(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear work-item falló: {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// WORK-ITEMS CRUD
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_work_items_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("list_unauth").await;
    let res = app
        .get(&format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_work_items_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("list_empty").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    let count = if let Some(arr) = body.as_array() {
        arr.len()
    } else {
        body["results"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    assert_eq!(count, 0, "proyecto vacío debe devolver 0 work-items");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_work_item_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("create").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items"),
            &json!({ "name": "First Work Item" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "First Work Item");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_work_item_empty_name_returns_400() {
    let (app, api_key, ws_slug, proj_id) = setup("create_empty").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items"),
            &json!({ "name": "   " }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_work_item_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("get").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Get WI").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Get WI");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_work_item_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("get_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_work_item_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("patch").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Old WI Name").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}"),
            &json!({ "name": "New WI Name", "priority": "high" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New WI Name");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_work_item_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("delete").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Delete WI").await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// LINKS — GET/POST + PATCH/DELETE /{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_work_item_links_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("links_list").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Link WI").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/links"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert!(res.json().as_array().map(|a| a.is_empty()).unwrap_or(true));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_work_item_link_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("link_create").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Linked WI").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/links"),
            &json!({ "title": "Plane", "url": "https://plane.so" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_work_item_link_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("link_del").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Del Link WI").await;

    // Crear link
    let link_res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/links"),
            &json!({ "title": "A", "url": "https://example.com" }),
        )
        .await;
    assert_eq!(link_res.status.as_u16(), 201);
    let link_id = link_res.json()["id"].as_str().unwrap().to_owned();

    let res = app
        .delete_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/links/{link_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// COMMENTS — GET/POST + GET/PATCH/DELETE /{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_work_item_comments_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("cmt_list").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Comment WI").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/comments"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_work_item_comment_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("cmt_create").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Commented WI").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/comments"),
            &json!({ "comment_html": "<p>A comment</p>" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_work_item_comment_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("cmt_patch").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Patch Comment WI").await;

    let cmt_res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/comments"),
            &json!({ "comment_html": "<p>Old</p>" }),
        )
        .await;
    let cmt_id = cmt_res.json()["id"].as_str().unwrap().to_owned();

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/comments/{cmt_id}"),
            &json!({ "comment_html": "<p>New</p>" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["comment_html"].as_str().unwrap_or(""), "<p>New</p>");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_work_item_comment_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("cmt_del").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Del Comment WI").await;

    let cmt_res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/comments"),
            &json!({ "comment_html": "<p>Bye</p>" }),
        )
        .await;
    let cmt_id = cmt_res.json()["id"].as_str().unwrap().to_owned();

    let res = app
        .delete_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/comments/{cmt_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// ACTIVITIES — GET /work-items/{id}/activities
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_work_item_activities_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("activities").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Activity WI").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/activities"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert!(res.json().is_array() || res.json().is_object());
}

// ═════════════════════════════════════════════════════════════════════════════
// RELATIONS — GET/POST /work-items/{id}/relations
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_work_item_relations_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("rel_list").await;
    let wi_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Relation WI").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{wi_id}/relations"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_work_item_relation_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("rel_create").await;
    let src_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Source WI").await;
    let tgt_id = create_work_item(&app, &api_key, &ws_slug, proj_id, "Target WI").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("{V1}/workspaces/{ws_slug}/projects/{proj_id}/work-items/{src_id}/relations"),
            &json!({ "relation_type": "blocking", "issues": [tgt_id] }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "create-relation debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
