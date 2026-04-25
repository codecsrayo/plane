//! Tests de integración: Intake / Inbox
//!
//! Cobertura:
//!   - GET/POST         /workspaces/{slug}/projects/{project_id}/intakes
//!   - GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intakes/{pk}
//!   - GET/POST         /workspaces/{slug}/projects/{project_id}/intake-issues
//!   - GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intake-issues/{pk}
//!   - Alias legacy: /inboxes y /inbox-issues

mod common;

use common::TestApp;
use serde_json::json;

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("intk_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("ik-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Intake Project", "ITK")
        .await;
    (app, api_key, ws_slug, proj_id)
}

async fn create_intake(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear intake falló: {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// INTAKES — GET/POST
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_intakes_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("list_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_intakes_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("list").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_intake_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("create").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes"),
            &json!({ "name": "My Intake" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "My Intake");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_intake_empty_name_returns_400() {
    let (app, api_key, ws_slug, proj_id) = setup("create_empty").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes"),
            &json!({ "name": "   " }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

// ═════════════════════════════════════════════════════════════════════════════
// INTAKE — GET/PATCH/DELETE /{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_intake_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("get").await;
    let intake_id = create_intake(&app, &api_key, &ws_slug, proj_id, "Get Intake").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes/{intake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Get Intake");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_intake_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("get_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_intake_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("patch").await;
    let intake_id = create_intake(&app, &api_key, &ws_slug, proj_id, "Old Intake").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes/{intake_id}"),
            &json!({ "name": "Updated Intake" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Updated Intake");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_intake_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("del").await;
    let intake_id = create_intake(&app, &api_key, &ws_slug, proj_id, "Delete Intake").await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intakes/{intake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// INTAKE-ISSUES — GET/POST
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_intake_issues_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("iss_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_intake_issues_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("iss_empty").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let count = res.json().as_array().map(|a| a.len()).unwrap_or(0);
    assert_eq!(count, 0, "sin intake issues debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_intake_issue_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("iss_create").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues"),
            &json!({
                "issue": {
                    "name": "Intake Issue Alpha",
                    "priority": "none"
                }
            }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
}

async fn create_intake_issue(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues"),
            &json!({ "issue": { "name": name } }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear intake-issue falló: {}", String::from_utf8_lossy(&res.body));
    // Django interpreta el `pk` en /intake-issues/{pk}/ como el issue_id del
    // issue subyacente, NO como el id del intake_issue (Django base.py:499,
    // 525; frontend project-inbox.store.ts:430,467 también envía
    // `response.issue.id`). Los handlers Rust (get/update/delete) replican
    // esto vía `intake_issues::Column::IssueId.eq(pk)`. El helper antes
    // devolvía `res.json()["id"]` (intake_issue.id), provocando 404 en
    // get/update/delete; lo corrijo para devolver el issue_id real.
    res.json()["issue_id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// INTAKE-ISSUES — GET/PATCH/DELETE /{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_intake_issue_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("iss_get").await;
    let iss_id = create_intake_issue(&app, &api_key, &ws_slug, proj_id, "Get Intake Issue").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues/{iss_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_intake_issue_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("iss_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_intake_issue_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("iss_patch").await;
    let iss_id = create_intake_issue(&app, &api_key, &ws_slug, proj_id, "Old Intake Issue").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues/{iss_id}"),
            &json!({ "status": -2 }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "update intake-issue debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_intake_issue_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("iss_del").await;
    let iss_id = create_intake_issue(&app, &api_key, &ws_slug, proj_id, "Delete Intake Issue").await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/intake-issues/{iss_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// ALIAS LEGACY: /inbox-issues (mismo handler que intake-issues)
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_inbox_issues_alias_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("inbox_alias").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/inbox-issues"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "alias /inbox-issues debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_inbox_issue_alias_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("inbox_create").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/inbox-issues"),
            &json!({ "issue": { "name": "Inbox Alias Issue" } }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "alias /inbox-issues POST debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
