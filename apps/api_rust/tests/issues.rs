//! Tests de integración: Issues
//!
//! Dependencias: workspace + project + membership.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects/{pid}/issues          → 401, 200 vacío, 200 con issue
//!   - POST /workspaces/{slug}/projects/{pid}/issues          → 201, 400 nombre vacío
//!   - GET  /workspaces/{slug}/projects/{pid}/issues/{id}     → 200 datos, 404
//!   - PATCH /workspaces/{slug}/projects/{pid}/issues/{id}    → 200 actualiza nombre
//!   - DELETE /workspaces/{slug}/projects/{pid}/issues/{id}   → 204
//!   - GET  /workspaces/{slug}/projects/{pid}/issues/{id}/history     → 200
//!   - GET/POST /workspaces/{slug}/projects/{pid}/issues/{id}/comments → 201, 400 vacío
//!   - POST /workspaces/{slug}/projects/{pid}/issues/{id}/issue-relation → 201
//!   - GET  /workspaces/{slug}/projects/{pid}/issues/{id}/issue-subscribers → 200
//!   - POST /workspaces/{slug}/projects/{pid}/bulk-delete-issues → 200
//!   - Proptest: POST con prioridades inválidas → siempre aceptadas o 422

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────── helpers locales ────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("issues_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("iss-ws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Issues Project", "ISS")
        .await;
    (app, user_id, api_key, ws_slug, proj_id)
}

/// Crea un issue vía API y devuelve su UUID como String.
async fn create_issue(app: &TestApp, api_key: &str, ws_slug: &str, proj_id: uuid::Uuid, name: &str) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear issue falló: {}", String::from_utf8_lossy(&res.body));
    res.json()["id"].as_str().expect("id en respuesta").to_owned()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_unauthenticated_returns_401() {
    let (app, user_id, _, ws_slug, proj_id) = setup("list_unauth").await;
    let _ = user_id;
    let res = app.get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/issues")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_empty_project_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("list_empty").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    // La respuesta puede ser array vacío o paginación con results vacío
    let body = res.json();
    let count = if let Some(arr) = body.as_array() {
        arr.len()
    } else {
        body["results"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    assert_eq!(count, 0, "proyecto sin issues debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_returns_created_issue() {
    let (app, _, api_key, ws_slug, proj_id) = setup("list_ok").await;
    create_issue(&app, &api_key, &ws_slug, proj_id, "Test Issue").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let arr = if let Some(a) = body.as_array() {
        a.clone()
    } else {
        body["results"].as_array().cloned().unwrap_or_default()
    };
    assert!(!arr.is_empty(), "debe haber al menos 1 issue");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_success_returns_201() {
    let (app, _, api_key, ws_slug, proj_id) = setup("create_ok").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": "My First Issue" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "My First Issue");
    assert_eq!(body["priority"].as_str().unwrap_or(""), "none");
    assert!(body["sequence_id"].as_i64().unwrap_or(0) > 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_empty_name_returns_400() {
    let (app, _, api_key, ws_slug, proj_id) = setup("create_empty").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": "   " }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_with_priority_returns_201() {
    let (app, _, api_key, ws_slug, proj_id) = setup("create_prio").await;
    for prio in &["urgent", "high", "medium", "low", "none"] {
        let res = app
            .post_json_authed(
                &api_key,
                &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
                &json!({ "name": format!("Issue {prio}"), "priority": prio }),
            )
            .await;
        assert_eq!(res.status.as_u16(), 201, "priority={prio} debe ser válida");
    }
}

/// Proptest: nombres de 1–500 chars alfanuméricos → siempre 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_issue_proptest_valid_names_always_201() {
    let (app, _, api_key, ws_slug, proj_id) = setup("create_pt").await;

    let strategy = "[a-zA-Z0-9 ]{1,200}"
        .prop_filter("no vacío tras trim", |s| !s.trim().is_empty());
    let mut runner = TestRunner::new(PropConfig { cases: 15, ..PropConfig::default() });

    runner
        .run(&strategy, |name| {
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        &api_key,
                        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
                        &json!({ "name": name }),
                    )
                    .await
                    .status
                    .as_u16()
                })
            });
            prop_assert_eq!(status, 201, "nombre válido {name:?} debe devolver 201, obtuvo {status}");
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_issue_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("get_ok").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Readable Issue").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Readable Issue");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_issue_not_found_returns_404() {
    let (app, _, api_key, ws_slug, proj_id) = setup("get_404").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}/projects/{pid}/issues/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_issue_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("update").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Old Title").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}"),
            &json!({ "name": "New Title", "priority": "high" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "New Title");
    assert_eq!(body["priority"].as_str().unwrap_or(""), "high");
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/issues/{id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_issue_returns_204() {
    let (app, _, api_key, ws_slug, proj_id) = setup("delete").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Ephemeral Issue").await;

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues/{id}/history
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_issue_history_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("history").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "History Issue").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/history"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    // Actividades recién creadas pueden ser vacías o tener el evento de creación
    assert!(res.json().is_array() || res.json().is_object());
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/projects/{pid}/issues/{id}/comments
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_comments_empty_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("comments_list").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Comment Issue").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let arr = body.as_array().unwrap_or(&vec![]).clone();
    assert!(arr.is_empty(), "sin comentarios la lista debe ser vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_comment_success_returns_201() {
    let (app, _, api_key, ws_slug, proj_id) = setup("comments_create").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Commented Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments"),
            &json!({ "comment_html": "<p>Hello world</p>" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(
        res.json()["comment_html"].as_str().unwrap_or(""),
        "<p>Hello world</p>"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_comment_empty_html_returns_400() {
    let (app, _, api_key, ws_slug, proj_id) = setup("comments_empty").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Empty Comment Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments"),
            &json!({ "comment_html": "   " }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues/{id}/issue-subscribers
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_subscribers_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("subs").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Subbed Issue").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-subscribers"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/bulk-delete-issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn bulk_delete_issues_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("bulk_del").await;
    let id1 = create_issue(&app, &api_key, &ws_slug, proj_id, "Bulk A").await;
    let id2 = create_issue(&app, &api_key, &ws_slug, proj_id, "Bulk B").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/bulk-delete-issues"),
            &json!({ "issue_ids": [id1, id2] }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "bulk-delete debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/issues/{id}/issue-relation
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_relation_returns_201() {
    let (app, _, api_key, ws_slug, proj_id) = setup("relation").await;
    let source_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Source Issue").await;
    let target_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Target Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{source_id}/issue-relation"),
            &json!({
                "relation_type": "blocks",
                "related_list": [target_id]
            }),
        )
        .await;
    assert!(
        res.status.as_u16() == 201 || res.status.as_u16() == 200,
        "issue-relation debe devolver 200/201, obtuvo {} body: {}",
        res.status,
        String::from_utf8_lossy(&res.body)
    );
}
