//! Tests de integración: Cycles
//!
//! Dependencias: workspace + project + membership + issues.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects/{pid}/cycles           → 401, 200 vacío, 200 con ciclo
//!   - POST /workspaces/{slug}/projects/{pid}/cycles           → 201, 400 nombre vacío,
//!                                                               400 start >= end
//!   - GET  /workspaces/{slug}/projects/{pid}/cycles/{id}      → 200, 404
//!   - PATCH /workspaces/{slug}/projects/{pid}/cycles/{id}     → 200 actualiza nombre
//!   - DELETE /workspaces/{slug}/projects/{pid}/cycles/{id}    → 204
//!   - GET/POST /workspaces/{slug}/projects/{pid}/cycles/{id}/cycle-issues → 200, 200 añade
//!   - DELETE .../cycle-issues/{issue_id}                      → 204
//!   - POST .../archive + GET /archived-cycles                 → 200
//!   - Proptest: nombres válidos → siempre 201

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────── helpers locales ─────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user(&format!("cycles_{suffix}@plane.test")).await;
    let ws_slug = format!("cyc-ws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app.create_test_project(user_id, ws_id, "Cycles Project", "CYC").await;
    (app, user_id, api_key, ws_slug, proj_id)
}

async fn create_cycle(app: &TestApp, api_key: &str, ws: &str, pid: uuid::Uuid, name: &str) -> String {
    let res = app
        .post_json_authed(api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles"), &json!({ "name": name }))
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear ciclo falló: {}", String::from_utf8_lossy(&res.body));
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
// GET /workspaces/{slug}/projects/{pid}/cycles
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_cycles_unauthenticated_returns_401() {
    let (app, _, _, ws, pid) = setup("list_unauth").await;
    let res = app.get(&format!("/workspaces/{ws}/projects/{pid}/cycles")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_cycles_empty_returns_200() {
    let (app, _, api_key, ws, pid) = setup("list_empty").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles")).await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert!(body.as_array().map(|a| a.is_empty()).unwrap_or(false),
        "proyecto sin ciclos debe devolver lista vacía, got: {body}");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_cycles_returns_created_cycle() {
    let (app, _, api_key, ws, pid) = setup("list_ok").await;
    create_cycle(&app, &api_key, &ws, pid, "Sprint 1").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles")).await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 ciclo");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/cycles
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_cycle_success() {
    let (app, _, api_key, ws, pid) = setup("create_ok").await;
    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles"), &json!({ "name": "Q1 Sprint" }))
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Q1 Sprint");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_cycle_empty_name_returns_400() {
    let (app, _, api_key, ws, pid) = setup("create_empty").await;
    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles"), &json!({ "name": "  " }))
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_cycle_start_gte_end_returns_400() {
    let (app, _, api_key, ws, pid) = setup("create_dates").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws}/projects/{pid}/cycles"),
            &json!({
                "name": "Bad Dates",
                "start_date": "2025-05-10T00:00:00Z",
                "end_date":   "2025-05-09T00:00:00Z"
            }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400, "start >= end debe devolver 400");
}

/// Proptest: nombres alfanuméricos válidos → siempre 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_cycle_proptest_valid_names() {
    let (app, _, api_key, ws, pid) = setup("proptest").await;
    let strategy = "[a-zA-Z0-9 ]{1,80}"
        .prop_filter("no vacío tras trim", |s| !s.trim().is_empty());
    let mut runner = TestRunner::new(PropConfig { cases: 12, ..PropConfig::default() });
    runner
        .run(&strategy, |name| {
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles"), &json!({ "name": name }))
                        .await.status.as_u16()
                })
            });
            prop_assert_eq!(status, 201, "nombre válido {name:?} debe dar 201, obtuvo {status}");
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET + PATCH + DELETE /cycles/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_returns_200() {
    let (app, _, api_key, ws, pid) = setup("get_ok").await;
    let cid = create_cycle(&app, &api_key, &ws, pid, "Readable Cycle").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}")).await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Readable Cycle");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_not_found_returns_404() {
    let (app, _, api_key, ws, pid) = setup("get_404").await;
    let fake = uuid::Uuid::new_v4();
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{fake}")).await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_cycle_returns_200() {
    let (app, _, api_key, ws, pid) = setup("update").await;
    let cid = create_cycle(&app, &api_key, &ws, pid, "Old Cycle").await;
    let res = app
        .patch_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}"), &json!({ "name": "New Cycle" }))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New Cycle");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_cycle_returns_204() {
    let (app, _, api_key, ws, pid) = setup("delete").await;
    let cid = create_cycle(&app, &api_key, &ws, pid, "Ephemeral Cycle").await;
    let res = app.delete_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}")).await;
    assert_eq!(res.status.as_u16(), 204);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /cycles/{id}/cycle-issues  +  DELETE .../cycle-issues/{issue_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_cycle_issues_empty_returns_200() {
    let (app, _, api_key, ws, pid) = setup("ci_list").await;
    let cid = create_cycle(&app, &api_key, &ws, pid, "Empty Cycle").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}/cycle-issues")).await;
    assert_eq!(res.status.as_u16(), 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn add_issue_to_cycle_and_remove() {
    let (app, _, api_key, ws, pid) = setup("ci_add_rm").await;
    let cid = create_cycle(&app, &api_key, &ws, pid, "Active Cycle").await;
    let iid = create_issue(&app, &api_key, &ws, pid, "Cycle Issue").await;

    // Añadir
    let add_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}/cycle-issues"), &json!({ "issues": [iid] }))
        .await;
    assert_eq!(add_res.status.as_u16(), 200, "add_issues_to_cycle debe devolver 200");

    // Eliminar
    let rm_res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}/cycle-issues/{iid}"))
        .await;
    assert_eq!(rm_res.status.as_u16(), 204, "remove_from_cycle debe devolver 204");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST .../archive  +  GET /archived-cycles
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn archive_cycle_and_list_archived() {
    let (app, _, api_key, ws, pid) = setup("archive").await;
    let cid = create_cycle(&app, &api_key, &ws, pid, "To Archive").await;

    let arc_res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/cycles/{cid}/archive"), &json!({}))
        .await;
    assert!(
        arc_res.status.as_u16() == 200 || arc_res.status.as_u16() == 204,
        "archive debe devolver 200 o 204, obtuvo {}", arc_res.status
    );

    let list_res = app.get_authed(&api_key, &format!("/workspaces/{ws}/projects/{pid}/archived-cycles")).await;
    assert_eq!(list_res.status.as_u16(), 200);
    let arr = list_res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 ciclo archivado");
}
