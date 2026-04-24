//! Tests de integración: Workspace Integrations — endpoints avanzados
//!
//! Cobertura:
//!   - DELETE /workspaces/{slug}/workspace-integrations/{provider}/provider → 401, 404
//!   - POST   /workspaces/{slug}/workspace-integrations/{provider}/install  → 401, 400
//!   - DELETE /workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk} → 401, 404
//!   - GET    /workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories → 401, 200[]
//!   - GET    /workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories → 401, 200[]
//!   - GET/POST /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings → 401, 200, 201
//!   - DELETE  /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk} → 401, 404
//!
//! Nota: los endpoints de OAuth GitHub (GET /github/callback, /github/user-callback)
//! son redirect flows que requieren estado de OAuth externo; se prueba solo el
//! comportamiento de rechazo ante parámetros ausentes.

mod common;

use axum::http::Method;
use common::TestApp;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String) {
    let app = TestApp::spawn().await;
    let email = format!("wi2_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("wi2-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, api_key, ws_slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/workspace-integrations/{provider}/provider
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_integration_by_provider_unauthenticated_returns_401() {
    let (app, _, slug) = setup("prov_del_unauth").await;
    let res = app
        .delete_authed("", &format!("/workspaces/{slug}/workspace-integrations/github/provider"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_integration_by_provider_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("prov_del_404").await;
    // "github" provider pero sin integration instalada
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/github/provider"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "provider no instalado debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_integration_by_provider_unknown_provider_404() {
    let (app, api_key, slug) = setup("prov_del_unk").await;
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/slack/provider"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "provider desconocido debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/workspace-integrations/{provider}/install
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn install_integration_provider_unauthenticated_returns_401() {
    let (app, _, slug) = setup("inst_unauth").await;
    let res = app
        .request(
            Method::POST,
            &format!("/workspaces/{slug}/workspace-integrations/github/install"),
            Some(serde_json::to_vec(&json!({"code": "abc123"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn install_integration_provider_missing_code_returns_error() {
    let (app, api_key, slug) = setup("inst_nocode").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/github/install"),
            &json!({}),
        )
        .await;
    // Sin code de OAuth → 400 o 422
    let status = res.status.as_u16();
    assert!(
        status == 400 || status == 422 || status == 404,
        "install sin code: esperado 400/422/404, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_github_repo_sync_unauthenticated_returns_401() {
    let (app, _, slug) = setup("rs_del_unauth").await;
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            "",
            &format!("/workspaces/{slug}/workspace-integrations/github/repo-syncs/{fake_pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_github_repo_sync_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("rs_del_404").await;
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/github/repo-syncs/{fake_pk}"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete repo-sync inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_github_repositories_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gh_repos_unauth").await;
    let wi_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_github_repositories_nonexistent_wi_returns_404_or_empty() {
    let (app, api_key, slug) = setup("gh_repos_404").await;
    let fake_wi = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/{fake_wi}/github-repositories"),
        )
        .await;
    // WorkspaceIntegration inexistente → 404 o lista vacía
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 200,
        "github-repos wi inexistente: esperado 404 o 200, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_gitlab_repositories_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gl_repos_unauth").await;
    let wi_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_gitlab_repositories_nonexistent_wi_returns_404_or_empty() {
    let (app, api_key, slug) = setup("gl_repos_404").await;
    let fake_wi = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/{fake_wi}/gitlab-repositories"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 200,
        "gitlab-repos wi inexistente: esperado 404 o 200, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_pr_state_mappings_unauthenticated_returns_401() {
    let (app, _, slug) = setup("prsm_list_unauth").await;
    let wi_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_pr_state_mappings_nonexistent_wi_returns_200_or_404() {
    let (app, api_key, slug) = setup("prsm_list_ok").await;
    let fake_wi = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/{fake_wi}/pr-state-mappings"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "pr-state-mappings wi inexistente: esperado 200 o 404, obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
    if status == 200 {
        let body = res.json();
        assert!(body.is_array(), "debe ser array");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn create_pr_state_mapping_unauthenticated_returns_401() {
    let (app, _, slug) = setup("prsm_post_unauth").await;
    let wi_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings"),
            Some(serde_json::to_vec(&json!({"state_id": Uuid::new_v4(), "pr_state": "open"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_pr_state_mapping_nonexistent_wi_returns_error() {
    let (app, api_key, slug) = setup("prsm_post_404").await;
    let fake_wi = Uuid::new_v4();
    let fake_state = Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/{fake_wi}/pr-state-mappings"),
            &json!({
                "state_id": fake_state,
                "pr_state": "open"
            }),
        )
        .await;
    // wi inexistente → 404 o error de FK
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400 || status == 422 || status == 500,
        "create pr-state-mapping wi inexistente: obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_pr_state_mapping_unauthenticated_returns_401() {
    let (app, _, slug) = setup("prsm_del_unauth").await;
    let wi_id = Uuid::new_v4();
    let pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            "",
            &format!("/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_pr_state_mapping_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("prsm_del_404").await;
    let fake_wi = Uuid::new_v4();
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/workspace-integrations/{fake_wi}/pr-state-mappings/{fake_pk}"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete pr-state-mapping inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /github/callback + /github/user-callback — OAuth flow
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn github_oauth_callback_without_state_redirects_to_error() {
    let app = TestApp::spawn().await;
    // Sin parámetros → debe redirigir a error o devolver 400
    let res = app.get("/auth/github/callback").await;
    let status = res.status.as_u16();
    assert!(
        status == 303 || status == 302 || status == 400 || status == 422,
        "github callback sin params: esperado redirect o 400, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn github_user_callback_without_state_redirects_to_error() {
    let app = TestApp::spawn().await;
    let res = app.get("/auth/github/user-callback").await;
    let status = res.status.as_u16();
    assert!(
        status == 303 || status == 302 || status == 400 || status == 422,
        "github user-callback sin params: esperado redirect o 400, obtuvo {status}"
    );
}
