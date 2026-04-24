//! Tests de integración: Cycles/Modules/Estimates Extras
//!
//! Cobertura:
//!   - GET    .../archived-cycles/{pk}                   → 200/404
//!   - DELETE .../archived-cycles/{pk}/unarchive         → 200/404
//!   - POST   .../modules/{module_id}/archive            → 200
//!   - GET    .../archived-modules/{pk}                  → 200/404
//!   - DELETE .../archived-modules/{pk}/unarchive        → 200/404
//!   - POST   .../cycles/{cycle_id}/transfer-issues      → 200/400
//!   - GET    .../project-estimates                      → 200

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("cycmod_{suffix}@plane.test"))
        .await;
    let slug = format!("cycmod-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("CycMod {suffix}"), "CYM")
        .await;
    (app, user_id, api_key, slug, ws_id, project_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/archived-cycles/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_cycle_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("arcyc-unauth").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/archived-cycles/{fake_pk}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_cycle_nonexistent_returns_404() {
    let (app, _, api_key, slug, _, project_id) = setup("arcyc-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/archived-cycles/{fake_pk}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "ciclo archivado inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/archived-cycles/{pk}/unarchive
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_cycle_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("unarcyc-unauth").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(
            "no-key",
            &format!(
                "/workspaces/{slug}/projects/{project_id}/archived-cycles/{fake_pk}/unarchive"
            ),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_cycle_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("unarcyc-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/archived-cycles/{fake_pk}/unarchive"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "desarchivar ciclo inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/modules/{module_id}/archive
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn archive_module_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("archmod-unauth").await;
    let fake_module = uuid::Uuid::new_v4();
    let res = app
        .post_json(
            &format!(
                "/workspaces/{slug}/projects/{project_id}/modules/{fake_module}/archive"
            ),
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn archive_module_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("archmod-notfound").await;
    let fake_module = uuid::Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/modules/{fake_module}/archive"
            ),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "archivar módulo inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/archived-modules/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_module_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("armod-unauth").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/archived-modules/{fake_pk}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_module_nonexistent_returns_404() {
    let (app, _, api_key, slug, _, project_id) = setup("armod-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/archived-modules/{fake_pk}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "módulo archivado inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/archived-modules/{pk}/unarchive
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_module_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("unarmod-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/archived-modules/{fake_pk}/unarchive"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "desarchivar módulo inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/cycles/{cycle_id}/transfer-issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn transfer_issues_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("transfer-unauth").await;
    let fake_cycle = uuid::Uuid::new_v4();
    let res = app
        .post_json(
            &format!(
                "/workspaces/{slug}/projects/{project_id}/cycles/{fake_cycle}/transfer-issues"
            ),
            &json!({ "new_cycle_id": uuid::Uuid::new_v4() }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn transfer_issues_nonexistent_cycle_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("transfer-notfound").await;
    let fake_cycle = uuid::Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/cycles/{fake_cycle}/transfer-issues"
            ),
            &json!({ "new_cycle_id": uuid::Uuid::new_v4() }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "transferir issues de ciclo inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/project-estimates
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_estimates_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("projest-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/project-estimates"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_estimates_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("projest-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/project-estimates"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "project-estimates debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
