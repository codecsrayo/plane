//! Tests de integración: Analytics
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/analytics                    → 200
//!   - GET  /workspaces/{slug}/default-analytics            → 200
//!   - GET  /workspaces/{slug}/project-stats                → 200
//!   - POST /workspaces/{slug}/export-analytics             → 200/202
//!   - GET/POST /workspaces/{slug}/analytic-view            → 200/201
//!   - GET/PATCH/DELETE /workspaces/{slug}/analytic-view/{pk} → 200/204
//!   - GET  /workspaces/{slug}/saved-analytic-view/{id}     → 200/404
//!   - GET  /workspaces/{slug}/advance-analytics            → 200
//!   - GET  /workspaces/{slug}/advance-analytics-stats      → 200
//!   - GET  /workspaces/{slug}/advance-analytics-charts     → 200
//!   - GET  /workspaces/{slug}/projects/{id}/advance-analytics       → 200
//!   - GET  /workspaces/{slug}/projects/{id}/advance-analytics-stats → 200
//!   - GET  /workspaces/{slug}/projects/{id}/advance-analytics-charts → 200

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("analytics_owner_{suffix}@plane.test"))
        .await;
    let slug = format!("analytics-ws-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("Analytics Proj {suffix}"), "ANA")
        .await;
    (app, api_key, slug, ws_id, project_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/analytics
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn workspace_analytics_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (uid, _) = app.create_test_user("anl_anon@plane.test").await;
    app.create_test_workspace(uid, "anl-anon-ws").await;
    let res = app.get("/workspaces/anl-anon-ws/analytics").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn workspace_analytics_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("anl").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/analytics"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "analytics debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_object() || body.is_array(), "respuesta debe ser objeto o array");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/default-analytics
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn default_analytics_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (uid, _) = app.create_test_user("def_anl_anon@plane.test").await;
    app.create_test_workspace(uid, "def-anl-anon-ws").await;
    let res = app.get("/workspaces/def-anl-anon-ws/default-analytics").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn default_analytics_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("defanl").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/default-analytics"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "default-analytics debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/project-stats
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn project_stats_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (uid, _) = app.create_test_user("pstats_anon@plane.test").await;
    app.create_test_workspace(uid, "pstats-anon-ws").await;
    let res = app.get("/workspaces/pstats-anon-ws/project-stats").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn project_stats_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("pstats").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/project-stats"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project-stats debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/analytic-view
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_analytic_views_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (uid, _) = app.create_test_user("av_anon@plane.test").await;
    app.create_test_workspace(uid, "av-anon-ws").await;
    let res = app.get("/workspaces/av-anon-ws/analytic-view").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_analytic_views_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("avlst").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/analytic-view"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "analytic-view lista debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_array(), "lista de vistas debe ser array, got: {body}");
}

#[tokio::test(flavor = "multi_thread")]
async fn analytic_views_crud() {
    let (app, api_key, slug, _, _) = setup("avcrud").await;

    // CREATE
    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/analytic-view"),
            &json!({
                "name": "Test Analytic View",
                "description": "Integration test view",
                "filters": {},
                "query": {},
                "query_dict": {}
            }),
        )
        .await;
    let status = create_res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "crear analytic-view debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&create_res.body)
    );
    let created = create_res.json();
    let view_id = created["id"].as_str().expect("vista debe tener id");

    // GET
    let get_res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/analytic-view/{view_id}"))
        .await;
    assert_eq!(get_res.status.as_u16(), 200, "GET analytic-view debe devolver 200");

    // PATCH
    let patch_res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/analytic-view/{view_id}"),
            &json!({ "name": "Updated Analytic View" }),
        )
        .await;
    assert_eq!(
        patch_res.status.as_u16(),
        200,
        "PATCH analytic-view debe devolver 200"
    );

    // DELETE
    let del_res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/analytic-view/{view_id}"))
        .await;
    let del_status = del_res.status.as_u16();
    assert!(
        del_status == 200 || del_status == 204,
        "DELETE analytic-view debe devolver 200/204, obtuvo {del_status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/saved-analytic-view/{analytic_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_saved_analytic_view_nonexistent_returns_404() {
    let (app, api_key, slug, _, _) = setup("savedav").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/saved-analytic-view/{fake_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "vista inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/advance-analytics
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn advance_analytics_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (uid, _) = app.create_test_user("advanl_anon@plane.test").await;
    app.create_test_workspace(uid, "advanl-anon-ws").await;
    let res = app.get("/workspaces/advanl-anon-ws/advance-analytics").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn advance_analytics_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("advanl").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/advance-analytics"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "advance-analytics debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/advance-analytics-stats
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn advance_analytics_stats_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("advanlstats").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/advance-analytics-stats"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "advance-analytics-stats debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/advance-analytics-charts
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn advance_analytics_charts_member_returns_200() {
    let (app, api_key, slug, _, _) = setup("advanlcharts").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/advance-analytics-charts"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "advance-analytics-charts debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/advance-analytics
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn project_advance_analytics_unauthenticated_returns_401() {
    let (app, _, slug, _, project_id) = setup("projadvanl-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/advance-analytics"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn project_advance_analytics_member_returns_200() {
    let (app, api_key, slug, _, project_id) = setup("projadvanl").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/advance-analytics"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project advance-analytics debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn project_advance_analytics_stats_member_returns_200() {
    let (app, api_key, slug, _, project_id) = setup("projadvanlstats").await;
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/advance-analytics-stats"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project advance-analytics-stats debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn project_advance_analytics_charts_member_returns_200() {
    let (app, api_key, slug, _, project_id) = setup("projadvanlcharts").await;
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/advance-analytics-charts"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project advance-analytics-charts debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/export-analytics
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn export_analytics_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (uid, _) = app.create_test_user("expanl_anon@plane.test").await;
    app.create_test_workspace(uid, "expanl-anon-ws").await;
    let res = app
        .post_json(
            "/workspaces/expanl-anon-ws/export-analytics",
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn export_analytics_member_returns_2xx() {
    let (app, api_key, slug, _, _) = setup("expanl").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/export-analytics"),
            &json!({
                "x_axis": "state__group",
                "y_axis": "issue_count"
            }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 202,
        "export-analytics debe devolver 200/202, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
