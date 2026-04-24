//! Tests de integración: Pages
//!
//! Dependencias: workspace + project + membership.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects/{pid}/pages          → 401, 200 vacío
//!   - GET  /workspaces/{slug}/projects/{pid}/pages-summary  → 200
//!   - POST /workspaces/{slug}/projects/{pid}/pages          → 201
//!   - GET  /workspaces/{slug}/projects/{pid}/pages/{id}     → 200, 404
//!   - PATCH /workspaces/{slug}/projects/{pid}/pages/{id}    → 200 actualiza nombre
//!   - DELETE /workspaces/{slug}/projects/{pid}/pages/{id}   → 204
//!   - POST .../pages/{id}/archive + POST .../lock           → 200/204
//!   - POST .../pages/{id}/duplicate                         → 201

mod common;

use common::TestApp;
use serde_json::json;

// ─────────── helpers locales ─────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user(&format!("pages_{suffix}@plane.test")).await;
    let ws_slug = format!("pg-ws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app.create_test_project(user_id, ws_id, "Pages Project", "PGS").await;
    (app, api_key, ws_slug, proj_id)
}

async fn create_page(
    app: &TestApp,
    api_key: &str,
    ws: &str,
    pid: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear page falló: {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().expect("id en page").to_owned()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/pages
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_pages_unauthenticated_returns_401() {
    let (app, _, ws, pid) = setup("list_unauth").await;
    let res = app.get(&format!("/workspaces/{ws}/projects/{pid}/pages")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_pages_empty_returns_200() {
    let (app, api_key, ws, pid) = setup("list_empty").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty(), "proyecto sin pages debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_pages_returns_created_page() {
    let (app, api_key, ws, pid) = setup("list_ok").await;
    create_page(&app, &api_key, &ws, pid, "My Page").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 page");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/pages-summary
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn pages_summary_returns_200() {
    let (app, api_key, ws, pid) = setup("summary").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages-summary"))
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/pages
// ─────────────────────────────────────────────────────────────────────────────

/// Página sin nombre (nombre opcional según el handler) → 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_page_no_name_returns_201() {
    let (app, api_key, ws, pid) = setup("create_no_name").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages"),
            &json!({ "access": 0 }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "page sin name debe ser válido, body: {}", String::from_utf8_lossy(&res.body));
}

/// Página con nombre y contenido HTML → 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_page_with_content_returns_201() {
    let (app, api_key, ws, pid) = setup("create_content").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages"),
            &json!({
                "name": "Tech Spec",
                "description_html": "<h1>Spec</h1><p>Details here</p>",
                "access": 0
            }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Tech Spec");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/pages/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_page_by_id_returns_200() {
    let (app, api_key, ws, pid) = setup("get_ok").await;
    let page_id = create_page(&app, &api_key, &ws, pid, "Readable Page").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Readable Page");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_page_not_found_returns_404() {
    let (app, api_key, ws, pid) = setup("get_404").await;
    let fake = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages/{fake}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}/projects/{pid}/pages/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_page_returns_200() {
    let (app, api_key, ws, pid) = setup("update").await;
    let page_id = create_page(&app, &api_key, &ws, pid, "Old Page Name").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}"),
            &json!({ "name": "New Page Name" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New Page Name");
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/pages/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_page_returns_204() {
    let (app, api_key, ws, pid) = setup("delete").await;
    let page_id = create_page(&app, &api_key, &ws, pid, "Ephemeral Page").await;

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST .../pages/{id}/archive
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn archive_page_returns_200_or_204() {
    let (app, api_key, ws, pid) = setup("archive").await;
    let page_id = create_page(&app, &api_key, &ws, pid, "Archive Me").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}/archive"),
            &json!({}),
        )
        .await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 204,
        "archive-page debe devolver 200/204, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST .../pages/{id}/lock  +  POST .../pages/{id}/lock (DELETE desbloquea)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn lock_and_unlock_page() {
    let (app, api_key, ws, pid) = setup("lock").await;
    let page_id = create_page(&app, &api_key, &ws, pid, "Lockable Page").await;

    // Bloquear
    let lock_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}/lock"),
            &json!({}),
        )
        .await;
    assert!(
        lock_res.status.as_u16() == 200 || lock_res.status.as_u16() == 204,
        "lock-page debe devolver 200/204, obtuvo {}", lock_res.status
    );

    // Desbloquear
    let unlock_res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}/lock"))
        .await;
    assert!(
        unlock_res.status.as_u16() == 200 || unlock_res.status.as_u16() == 204,
        "unlock-page debe devolver 200/204, obtuvo {}", unlock_res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST .../pages/{id}/duplicate
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_page_returns_201() {
    let (app, api_key, ws, pid) = setup("duplicate").await;
    let page_id = create_page(&app, &api_key, &ws, pid, "Original Page").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/pages/{page_id}/duplicate"),
            &json!({}),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "duplicate debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let dup_name = res.json()["name"].as_str().unwrap_or("").to_owned();
    assert!(
        dup_name.contains("Copy") || !dup_name.is_empty(),
        "página duplicada debe tener nombre, got: {dup_name:?}"
    );
}
