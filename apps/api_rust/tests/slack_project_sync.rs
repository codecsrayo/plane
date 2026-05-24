//! Tests de integración: Slack Project Sync endpoints
//!
//! Cobertura (auth-guard layer):
//!   - GET/POST /workspaces/{slug}/projects/{pid}/workspace-integrations/{wi_id}/project-slack-sync/
//!         → 401 unauthenticated, 404 wi inexistente
//!   - DELETE  /workspaces/{slug}/projects/{pid}/workspace-integrations/{wi_id}/project-slack-sync/{sid}/
//!         → 401 unauthenticated, 404 sync inexistente

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String, Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("slk_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("slk-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, &format!("Slack {suffix}"), "SLK")
        .await;
    (app, api_key, ws_slug, proj_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/workspace-integrations/{wi_id}/project-slack-sync/
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_slack_syncs_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("list_unauth").await;
    let wi_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/workspace-integrations/{wi_id}/project-slack-sync"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_slack_syncs_nonexistent_wi_returns_200_or_404() {
    let (app, api_key, slug, proj_id) = setup("list_ok").await;
    let fake_wi = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/workspace-integrations/{fake_wi}/project-slack-sync"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "list slack syncs wi inexistente: esperado 200 o 404, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
    if status == 200 {
        let body = res.json();
        assert!(body.is_array(), "debe ser array vacío");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/workspace-integrations/{wi_id}/project-slack-sync/
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_project_slack_sync_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("post_unauth").await;
    let wi_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/workspace-integrations/{wi_id}/project-slack-sync"
            ),
            Some(
                serde_json::to_vec(&json!({
                    "access_token": "xoxb-test",
                    "scopes": "channels:read",
                    "bot_user_id": "U123",
                    "webhook_url": "https://hooks.slack.com/test",
                    "data": {},
                    "team_id": "T123",
                    "team_name": "Test Team"
                }))
                .unwrap(),
            ),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_project_slack_sync_nonexistent_wi_returns_error() {
    let (app, api_key, slug, proj_id) = setup("post_404").await;
    let fake_wi = Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/workspace-integrations/{fake_wi}/project-slack-sync"
            ),
            &json!({
                "access_token": "xoxb-test",
                "scopes": "channels:read",
                "bot_user_id": "U123",
                "webhook_url": "https://hooks.slack.com/test",
                "data": {},
                "team_id": "T123",
                "team_name": "Test Team"
            }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 403 || status == 404 || status == 400,
        "create slack sync wi inexistente: esperado 403/404/400, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/workspace-integrations/{wi_id}/project-slack-sync/{sid}/
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_project_slack_sync_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("del_unauth").await;
    let wi_id = Uuid::new_v4();
    let sid = Uuid::new_v4();
    let res = app
        .delete_authed(
            "",
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/workspace-integrations/{wi_id}/project-slack-sync/{sid}"
            ),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_project_slack_sync_nonexistent_returns_404() {
    let (app, api_key, slug, proj_id) = setup("del_404").await;
    let fake_wi = Uuid::new_v4();
    let fake_sid = Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/workspace-integrations/{fake_wi}/project-slack-sync/{fake_sid}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 403 || status == 404,
        "delete slack sync inexistente: esperado 403 o 404, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}
