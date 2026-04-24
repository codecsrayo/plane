//! Tests de integración: Importer CRUD — GET/PATCH/DELETE /{importer_id}
//!
//! Cobertura:
//!   - GET    /workspaces/{slug}/importers/github/{importer_id}  → 401, 404
//!   - PATCH  /workspaces/{slug}/importers/github/{importer_id}  → 401, 404
//!   - DELETE /workspaces/{slug}/importers/github/{importer_id}  → 401, 404
//!   - GET    /workspaces/{slug}/importers/gitlab/{importer_id}  → 401, 404
//!   - PATCH  /workspaces/{slug}/importers/gitlab/{importer_id}  → 401, 404
//!   - DELETE /workspaces/{slug}/importers/gitlab/{importer_id}  → 401, 404

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String) {
    let app = TestApp::spawn().await;
    let email = format!("imp_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("imp-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, api_key, ws_slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// GitHub importer — GET/PATCH/DELETE /{importer_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_github_importer_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gh_get_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get(&format!("/workspaces/{slug}/importers/github/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_github_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gh_get_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/importers/github/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "importer inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_github_importer_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gh_patch_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .request(
            Method::PATCH,
            &format!("/workspaces/{slug}/importers/github/{fake_id}"),
            Some(serde_json::to_vec(&json!({"status": "processing"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_github_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gh_patch_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/importers/github/{fake_id}"),
            &json!({ "status": "processing" }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "patch importer inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_github_importer_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gh_del_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed("", &format!("/workspaces/{slug}/importers/github/{fake_id}"))
        .await;
    // Sin api_key → esperamos 401 (no 403)
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_github_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gh_del_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/importers/github/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete importer inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GitLab importer — GET/PATCH/DELETE /{importer_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_gitlab_importer_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gl_get_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get(&format!("/workspaces/{slug}/importers/gitlab/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_gitlab_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gl_get_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/importers/gitlab/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "gitlab importer inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_gitlab_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gl_patch_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/importers/gitlab/{fake_id}"),
            &json!({ "status": "failed" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_gitlab_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gl_del_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/importers/gitlab/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}
