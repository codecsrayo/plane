//! Tests de integración: Modules
//!
//! Dependencias: workspace + project + membership + issues.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects/{pid}/modules               → 401, 200 vacío, 200 con módulo
//!   - POST /workspaces/{slug}/projects/{pid}/modules               → 201, 400 nombre vacío,
//!                                                                     400 status inválido
//!   - GET  /workspaces/{slug}/projects/{pid}/modules/{id}          → 200, 404
//!   - PATCH /workspaces/{slug}/projects/{pid}/modules/{id}         → 200 actualiza nombre
//!   - DELETE /workspaces/{slug}/projects/{pid}/modules/{id}        → 204
//!   - GET/POST .../modules/{id}/issues                             → 200, 200 añade
//!   - DELETE .../modules/{id}/issues/{issue_id}                    → 204
//!   - POST .../modules/{id}/archive + GET /archived-modules        → 200/204, 200
//!   - Proptest: status inválidos → siempre 400

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────── helpers locales ─────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user(&format!("mods_{suffix}@plane.test")).await;
    let ws_slug = format!("mod-ws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app.create_test_project(user_id, ws_id, "Modules Project", "MOD").await;
    (app, user_id, api_key, ws_slug, proj_id)
}

async fn create_module(app: &TestApp, api_key: &str, ws: &str, pid: uuid::Uuid, name: &str) -> String {
    let res = app
        .post_json_authed(api_key, &format!("/workspaces/{ws}/projects/{pid}/modules"), &json!({ "name": name }))
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear módulo falló: {}", String::from_utf8_lossy(&res.body));
    res.json()["id"].as_str().expect("id").to_owned()
}

async fn create_issue(app: &TestApp, api_key: &str, ws: &str, pid: uuid::Uuid, name: &str) -> String {
    let res = app
        .post_json_authed(api_key, &format!("/workspaces/{ws}/projects/{pid}/issues"), &json!({ "name": name }))
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear issue falló");
    res.json()["id"].as_str().expect("id").to_owned()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/modules
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_modules_unauthenticated_returns_401() {
    let (app, _, _, ws, pid) = setup("list_unauth").await;
    let res = app.get(&format!("/workspaces/{ws}/projects/{pid}/modules")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_modules_empty_returns_200() {
    let (app, _, api_key, ws, pid) = setup("list_empty").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules")).await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert!(body.as_array().map(|a| a.is_empty()).unwrap_or(false),
        "proyecto sin módulos debe devolver lista vacía, got: {body}");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_modules_returns_created_module() {
    let (app, _, api_key, ws, pid) = setup("list_ok").await;
    create_module(&app, &api_key, &ws, pid, "Feature Module").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules")).await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 módulo");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/modules
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_module_success() {
    let (app, _, api_key, ws, pid) = setup("create_ok").await;
    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules"), &json!({ "name": "Auth Module" }))
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "Auth Module");
    assert_eq!(body["status"].as_str().unwrap_or(""), "backlog");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_module_empty_name_returns_400() {
    let (app, _, api_key, ws, pid) = setup("create_empty").await;
    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules"), &json!({ "name": "   " }))
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_module_invalid_status_returns_400() {
    let (app, _, api_key, ws, pid) = setup("create_bad_status").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/modules"),
            &json!({ "name": "Bad Status Module", "status": "invalid_status" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400, "status inválido debe devolver 400");
}

/// Todos los status válidos → siempre 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_module_all_valid_statuses() {
    let (app, _, api_key, ws, pid) = setup("valid_statuses").await;
    for status in &["backlog", "in-progress", "paused", "completed", "cancelled"] {
        let res = app
            .post_json_authed(
                &api_key,
                &format!("/workspaces/{ws}/projects/{pid}/modules"),
                &json!({ "name": format!("Module {status}"), "status": status }),
            )
            .await;
        assert_eq!(res.status.as_u16(), 201, "status={status} debe ser válido");
    }
}

/// Proptest: strings de status aleatorios → siempre 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_module_proptest_invalid_statuses() {
    let (app, _, api_key, ws, pid) = setup("proptest_status").await;
    let valid = &["backlog", "in-progress", "paused", "completed", "cancelled"];
    let strategy = "[a-z_]{3,20}"
        .prop_filter("no es status válido", move |s| !valid.contains(&s.as_str()));
    let mut runner = TestRunner::new(PropConfig { cases: 12, ..PropConfig::default() });
    runner
        .run(&strategy, |status| {
            let rt = tokio::runtime::Handle::current();
            let status_code = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        &api_key,
                        &format!("/workspaces/{ws}/projects/{pid}/modules"),
                        &json!({ "name": "Test", "status": status }),
                    )
                    .await.status.as_u16()
                })
            });
            prop_assert_eq!(status_code, 400, "status inválido {status:?} debe dar 400, obtuvo {status_code}");
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET + PATCH + DELETE /modules/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_module_returns_200() {
    let (app, _, api_key, ws, pid) = setup("get_ok").await;
    let mid = create_module(&app, &api_key, &ws, pid, "Readable Module").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Readable Module");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_module_not_found_returns_404() {
    let (app, _, api_key, ws, pid) = setup("get_404").await;
    let fake = uuid::Uuid::new_v4();
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{fake}")).await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_module_returns_200() {
    let (app, _, api_key, ws, pid) = setup("update").await;
    let mid = create_module(&app, &api_key, &ws, pid, "Old Module").await;
    let res = app
        .patch_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}"), &json!({ "name": "New Module" }))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New Module");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_module_returns_204() {
    let (app, _, api_key, ws, pid) = setup("delete").await;
    let mid = create_module(&app, &api_key, &ws, pid, "Ephemeral Module").await;
    let res = app.delete_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}")).await;
    assert_eq!(res.status.as_u16(), 204);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST .../modules/{id}/issues  +  DELETE .../issues/{issue_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_module_issues_empty_returns_200() {
    let (app, _, api_key, ws, pid) = setup("mi_list").await;
    let mid = create_module(&app, &api_key, &ws, pid, "Empty Module").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}/issues")).await;
    assert_eq!(res.status.as_u16(), 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn add_issue_to_module_and_remove() {
    let (app, _, api_key, ws, pid) = setup("mi_add_rm").await;
    let mid = create_module(&app, &api_key, &ws, pid, "Active Module").await;
    let iid = create_issue(&app, &api_key, &ws, pid, "Module Issue").await;

    // Añadir
    let add_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}/issues"), &json!({ "issues": [iid] }))
        .await;
    assert_eq!(add_res.status.as_u16(), 200, "add_issues_to_module debe devolver 200, body: {}", String::from_utf8_lossy(&add_res.body));

    // Eliminar
    let rm_res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}/issues/{iid}"))
        .await;
    assert_eq!(rm_res.status.as_u16(), 204, "remove_from_module debe devolver 204");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST .../archive  +  GET /archived-modules
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn archive_module_and_list_archived() {
    let (app, _, api_key, ws, pid) = setup("archive").await;
    let mid = create_module(&app, &api_key, &ws, pid, "To Archive").await;

    let arc_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/modules/{mid}/archive"), &json!({}))
        .await;
    assert!(
        arc_res.status.as_u16() == 200 || arc_res.status.as_u16() == 204,
        "archive debe devolver 200 o 204, obtuvo {}", arc_res.status
    );

    let list_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/archived-modules")).await;
    assert_eq!(list_res.status.as_u16(), 200);
    let arr = list_res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 módulo archivado");
}
