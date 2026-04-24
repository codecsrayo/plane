//! Tests de integración: Views
//!
//! Cobertura:
//!   Workspace Views:
//!   - GET/POST   /workspaces/{slug}/views
//!   - GET/PATCH/DELETE /workspaces/{slug}/views/{pk}
//!
//!   Project Views:
//!   - GET/POST   /workspaces/{slug}/projects/{project_id}/views
//!   - GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/views/{pk}
//!
//! Proptest: nombres arbitrarios en POST workspace view → siempre 201.

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("views_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("vws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Views Project", "VWS")
        .await;
    (app, api_key, ws_slug, proj_id)
}

// ═════════════════════════════════════════════════════════════════════════════
// WORKSPACE VIEWS
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_views_unauthenticated_returns_401() {
    let (app, _, ws_slug, _) = setup("ws_unauth").await;
    let res = app.get(&format!("/workspaces/{ws_slug}/views")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_views_empty_returns_200() {
    let (app, api_key, ws_slug, _) = setup("ws_empty").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/views"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let count = body.as_array().map(|a| a.len()).unwrap_or(0);
    assert_eq!(count, 0, "nuevo workspace no debe tener views");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_view_returns_201() {
    let (app, api_key, ws_slug, _) = setup("ws_create").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/views"),
            &json!({ "name": "My Workspace View" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "My Workspace View");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_view_empty_name_returns_400() {
    let (app, api_key, ws_slug, _) = setup("ws_empty_name").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/views"),
            &json!({ "name": "   " }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

/// Proptest: nombres arbitrarios (1-200 chars alfanuméricos) → siempre 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_view_proptest_valid_names() {
    let (app, api_key, ws_slug, _) = setup("ws_pt").await;

    let strategy = "[a-zA-Z0-9 ]{1,100}"
        .prop_filter("no vacío tras trim", |s| !s.trim().is_empty());
    let mut runner = TestRunner::new(PropConfig { cases: 10, ..PropConfig::default() });

    runner
        .run(&strategy, |name| {
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        &api_key,
                        &format!("/workspaces/{ws_slug}/views"),
                        &json!({ "name": name }),
                    )
                    .await
                    .status
                    .as_u16()
                })
            });
            prop_assert_eq!(status, 201, "nombre {:?} debe devolver 201, obtuvo {}", name, status);
            Ok(())
        })
        .expect("proptest workspace views falló");
}

async fn create_ws_view(app: &TestApp, api_key: &str, ws_slug: &str, name: &str) -> String {
    let res = app
        .post_json_authed(api_key, &format!("/workspaces/{ws_slug}/views"), &json!({ "name": name }))
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_view_returns_200() {
    let (app, api_key, ws_slug, _) = setup("ws_get").await;
    let view_id = create_ws_view(&app, &api_key, &ws_slug, "View Alpha").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/views/{view_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "View Alpha");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_view_not_found_returns_404() {
    let (app, api_key, ws_slug, _) = setup("ws_404").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/views/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_workspace_view_returns_200() {
    let (app, api_key, ws_slug, _) = setup("ws_patch").await;
    let view_id = create_ws_view(&app, &api_key, &ws_slug, "Old Name").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/views/{view_id}"),
            &json!({ "name": "New Name" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New Name");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_workspace_view_returns_204() {
    let (app, api_key, ws_slug, _) = setup("ws_del").await;
    let view_id = create_ws_view(&app, &api_key, &ws_slug, "Ephemeral View").await;

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws_slug}/views/{view_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_views_returns_created_view() {
    let (app, api_key, ws_slug, _) = setup("ws_list_ok").await;
    create_ws_view(&app, &api_key, &ws_slug, "Listed View").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/views"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 view");
}

// ═════════════════════════════════════════════════════════════════════════════
// PROJECT VIEWS
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_project_views_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("pv_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/views"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_views_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_empty").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let count = body.as_array().map(|a| a.len()).unwrap_or(0);
    assert_eq!(count, 0, "nuevo proyecto no debe tener views");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_project_view_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_create").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views"),
            &json!({ "name": "Project View Alpha" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Project View Alpha");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_project_view_empty_name_returns_400() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_empty_name").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views"),
            &json!({ "name": "" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

async fn create_proj_view(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_view_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_get").await;
    let view_id = create_proj_view(&app, &api_key, &ws_slug, proj_id, "PView A").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views/{view_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "PView A");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_view_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_404").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_project_view_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_patch").await;
    let view_id = create_proj_view(&app, &api_key, &ws_slug, proj_id, "PView Old").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views/{view_id}"),
            &json!({ "name": "PView New" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "PView New");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_project_view_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_del").await;
    let view_id = create_proj_view(&app, &api_key, &ws_slug, proj_id, "Ephemeral PView").await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views/{view_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_views_returns_created_view() {
    let (app, api_key, ws_slug, proj_id) = setup("pv_list_ok").await;
    create_proj_view(&app, &api_key, &ws_slug, proj_id, "Listed PView").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/views"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 project view");
}
