//! Tests de integración: Project Summary (v1_router)
//!
//! Cobertura:
//!   - GET /workspaces/{slug}/projects/{project_id}/summary
//!         → 401 sin auth
//!         → 200 con campos de conteo (members, states, labels, etc.)
//!         → 404 proyecto inexistente

mod common;

use common::TestApp;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("ps_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("ps-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Summary Project", "SUM")
        .await;
    (app, api_key, ws_slug, ws_id, proj_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/summary
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_project_summary_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/summary"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_summary_returns_200_with_counts() {
    let (app, api_key, slug, _, proj_id) = setup("counts").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/summary"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project summary debe retornar 200: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    // Verificar que incluye campos de conteo principales
    assert!(
        body.get("members").is_some(),
        "debe incluir campo members: {body}"
    );
    assert!(
        body.get("states").is_some(),
        "debe incluir campo states: {body}"
    );
    assert!(
        body.get("labels").is_some(),
        "debe incluir campo labels: {body}"
    );
    assert!(
        body.get("issues").is_some(),
        "debe incluir campo issues: {body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_summary_nonexistent_project_returns_404() {
    let (app, api_key, slug, _, _) = setup("not_found").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{fake_id}/summary"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "proyecto inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_summary_counts_are_non_negative() {
    let (app, api_key, slug, _, proj_id) = setup("non_neg").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/summary"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();

    // Todos los conteos numéricos deben ser >= 0
    for field in &["members", "states", "labels", "issues", "cycles", "modules"] {
        if let Some(val) = body.get(*field) {
            if let Some(n) = val.as_i64() {
                assert!(n >= 0, "campo {field} no debe ser negativo, obtuvo {n}");
            }
        }
    }
}
