//! Tests de integración: Public Space / Anchor endpoints
//!
//! Cobertura (`/api/public/...`):
//!   - GET /api/public/anchor/{anchor}/settings          → 200 / 404
//!   - GET /api/public/anchor/{anchor}/meta              → 200 / 404
//!   - GET /api/public/workspaces/{slug}/projects/{pid}/anchor → 200 / 404
//!   - GET /api/public/anchor/{anchor}/states            → 200 / 404
//!   - GET /api/public/anchor/{anchor}/labels            → 200 / 404
//!   - GET /api/public/anchor/{anchor}/members           → 200 / 404
//!   - GET /api/public/anchor/{anchor}/cycles            → 200 / 404
//!   - GET /api/public/anchor/{anchor}/modules           → 200 / 404
//!   - GET /api/public/anchor/{anchor}/issues            → 200 / 404
//!   - GET /api/public/anchor/{anchor}/issues/{id}       → 200 / 404
//!   - GET /api/public/anchor/{anchor}/issues/{id}/comments     → 200 / 404
//!   - POST /api/public/anchor/{anchor}/issues/{id}/comments    → 401 unauthenticated
//!   - GET /api/public/anchor/{anchor}/issues/{id}/reactions    → 200 / 404
//!   - POST /api/public/anchor/{anchor}/issues/{id}/reactions   → 401 unauthenticated
//!   - GET /api/public/anchor/{anchor}/comments/{id}/reactions  → 200 / 404
//!   - GET /api/public/anchor/{anchor}/issues/{id}/votes        → 200 / 404
//!   - POST /api/public/anchor/{anchor}/issues/{id}/votes       → 401 unauthenticated

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

// ── Test fixture setup ────────────────────────────────────────────────────────

/// Creates: user, workspace, project, and a deploy board with `anchor`.
async fn setup_with_board(
    suffix: &str,
) -> (TestApp, Uuid, String, String, Uuid, Uuid, String) {
    let app = TestApp::spawn().await;
    app.ensure_instance_configured().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("space_{suffix}@plane.test"))
        .await;
    let slug = format!("space-{suffix}");
    app.create_test_workspace(user_id, &slug).await;
    let ws_id = app.workspace_id_by_slug(&slug).await;
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("Space {suffix}"), "SPC")
        .await;
    let anchor = format!("anchor-{suffix}");
    app.create_deploy_board(user_id, ws_id, project_id, &anchor)
        .await;
    (app, user_id, api_key, slug, ws_id, project_id, anchor)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/settings
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_settings_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/settings"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "anchor inexistente debe retornar 404, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_settings_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("settings-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/settings"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor settings debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["anchor"], anchor, "anchor en respuesta debe coincidir");
    assert!(body["id"].is_string(), "debe tener campo id");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/meta
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_meta_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/meta"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_meta_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("meta-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/meta"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor meta debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body["id"].is_string(), "meta debe tener id del proyecto");
    assert!(body["name"].is_string(), "meta debe tener nombre del proyecto");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/workspaces/{slug}/projects/{pid}/anchor
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn workspace_project_anchor_nonexistent_slug_returns_404() {
    let app = TestApp::spawn().await;
    let fake_pid = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/workspaces/nonexistent-ws/projects/{fake_pid}/anchor"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn workspace_project_anchor_existing_returns_200() {
    let (app, _, _, slug, _, project_id, _) = setup_with_board("ws-anchor-ok").await;
    let res = app
        .get(&format!(
            "/api/public/workspaces/{slug}/projects/{project_id}/anchor"
        ))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "workspace project anchor debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body["anchor"].is_string(), "debe tener campo anchor");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/states
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_states_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/states"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_states_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("states-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/states"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor states debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "states debe ser array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/labels
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_labels_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/labels"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_labels_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("labels-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/labels"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor labels debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "labels debe ser array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/members
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_members_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/members"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_members_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("members-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/members"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor members debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "members debe ser array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/cycles
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_cycles_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/cycles"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_cycles_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("cycles-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/cycles"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor cycles debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "cycles debe ser array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/modules
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_modules_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/modules"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_modules_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("modules-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/modules"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor modules debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "modules debe ser array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_issues_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let res = app
        .get(&format!("/api/public/anchor/{fake_anchor}/issues"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_issues_existing_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("issues-ok").await;
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/issues"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "anchor issues debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    // Paginated response — expects "results" or similar field
    assert!(
        body.is_object() || body.is_array(),
        "issues response debe ser objeto o array"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/issues/{issue_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_get_issue_nonexistent_anchor_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{fake_anchor}/issues/{fake_issue}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_get_issue_nonexistent_issue_returns_404() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("get-issue-notfound").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!("/api/public/anchor/{anchor}/issues/{fake_issue}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "issue inexistente debe retornar 404, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/issues/{issue_id}/comments
// POST /api/public/anchor/{anchor}/issues/{issue_id}/comments
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_comments_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{fake_anchor}/issues/{fake_issue}/comments"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_comments_existing_board_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("comments-list-ok").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{anchor}/issues/{fake_issue}/comments"
        ))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "list comments debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "comments debe ser array");
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_create_comment_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("comment-post-unauth").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/api/public/anchor/{anchor}/issues/{fake_issue}/comments"),
            Some(
                serde_json::to_vec(&json!({
                    "comment_html": "<p>Test comment</p>"
                }))
                .unwrap(),
            ),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        401,
        "create comment sin auth debe retornar 401"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}
// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_update_comment_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("comment-patch-unauth").await;
    let fake_issue = Uuid::new_v4();
    let fake_comment = Uuid::new_v4();
    let res = app
        .request(
            Method::PATCH,
            &format!(
                "/api/public/anchor/{anchor}/issues/{fake_issue}/comments/{fake_comment}"
            ),
            Some(
                serde_json::to_vec(&json!({ "comment_html": "<p>Updated</p>" })).unwrap(),
            ),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_delete_comment_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("comment-del-unauth").await;
    let fake_issue = Uuid::new_v4();
    let fake_comment = Uuid::new_v4();
    let res = app
        .request(
            Method::DELETE,
            &format!(
                "/api/public/anchor/{anchor}/issues/{fake_issue}/comments/{fake_comment}"
            ),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/issues/{issue_id}/reactions
// POST /api/public/anchor/{anchor}/issues/{issue_id}/reactions
// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/reactions/{code}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_issue_reactions_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{fake_anchor}/issues/{fake_issue}/reactions"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_issue_reactions_existing_board_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("reactions-list-ok").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{anchor}/issues/{fake_issue}/reactions"
        ))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "list reactions debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "reactions debe ser array");
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_add_issue_reaction_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("reaction-post-unauth").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/api/public/anchor/{anchor}/issues/{fake_issue}/reactions"),
            Some(serde_json::to_vec(&json!({ "reaction": "128077" })).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_remove_issue_reaction_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("reaction-del-unauth").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .request(
            Method::DELETE,
            &format!("/api/public/anchor/{anchor}/issues/{fake_issue}/reactions/128077"),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/comments/{comment_id}/reactions
// POST /api/public/anchor/{anchor}/comments/{comment_id}/reactions
// DELETE /api/public/anchor/{anchor}/comments/{comment_id}/reactions/{code}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_comment_reactions_nonexistent_anchor_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let fake_comment = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{fake_anchor}/comments/{fake_comment}/reactions"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_comment_reactions_existing_board_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("comment-reactions-ok").await;
    let fake_comment = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{anchor}/comments/{fake_comment}/reactions"
        ))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "list comment reactions debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "comment reactions debe ser array");
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_add_comment_reaction_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("com-react-post-unauth").await;
    let fake_comment = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/api/public/anchor/{anchor}/comments/{fake_comment}/reactions"),
            Some(serde_json::to_vec(&json!({ "reaction": "128077" })).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_remove_comment_reaction_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("com-react-del-unauth").await;
    let fake_comment = Uuid::new_v4();
    let res = app
        .request(
            Method::DELETE,
            &format!(
                "/api/public/anchor/{anchor}/comments/{fake_comment}/reactions/128077"
            ),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/public/anchor/{anchor}/issues/{issue_id}/votes
// POST /api/public/anchor/{anchor}/issues/{issue_id}/votes
// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/votes
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_votes_nonexistent_returns_404() {
    let app = TestApp::spawn().await;
    let fake_anchor = Uuid::new_v4().to_string();
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{fake_anchor}/issues/{fake_issue}/votes"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_list_votes_existing_board_returns_200() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("votes-list-ok").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/api/public/anchor/{anchor}/issues/{fake_issue}/votes"
        ))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "list votes debe retornar 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    assert!(res.json().is_array(), "votes debe ser array");
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_add_vote_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("vote-post-unauth").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/api/public/anchor/{anchor}/issues/{fake_issue}/votes"),
            Some(serde_json::to_vec(&json!({ "vote": 1 })).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_remove_vote_unauthenticated_returns_401() {
    let (app, _, _, _, _, _, anchor) = setup_with_board("vote-del-unauth").await;
    let fake_issue = Uuid::new_v4();
    let res = app
        .request(
            Method::DELETE,
            &format!("/api/public/anchor/{anchor}/issues/{fake_issue}/votes"),
            None,
            &[],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}
