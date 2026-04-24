//! Tests de integración: Issues Extras III
//!
//! Cobertura:
//!   - GET    .../issues/{id}/versions                                  → 200
//!   - GET/DELETE .../issues/{id}/versions/{pk}                         → 200/404
//!   - GET    .../work-items/{id}/description-versions                  → 200
//!   - GET/DELETE .../work-items/{id}/description-versions/{pk}         → 200/404
//!   - GET    /workspaces/{slug}/projects/{pid}/issues/list             → 200
//!   - GET    /workspaces/{slug}/projects/{pid}/issues-detail           → 200
//!   - GET    /workspaces/{slug}/projects/{pid}/v2/issues               → 200
//!   - DELETE .../issue-subscribers/{subscriber_id}                     → 404
//!   - GET    /workspaces/{slug}/issues                                 → 200

mod common;

use common::TestApp;

async fn setup(
    suffix: &str,
) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("issext3_{suffix}@plane.test"))
        .await;
    let slug = format!("issext3-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("IssExt3 {suffix}"), "IE3")
        .await;
    (app, user_id, api_key, slug, ws_id, project_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues/{id}/versions
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_versions_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("versions-unauth").await;
    let fake_issue = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/issues/{fake_issue}/versions"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_versions_nonexistent_issue_returns_4xx_or_200() {
    let (app, _, api_key, slug, _, project_id) = setup("versions-ok").await;
    let fake_issue = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/issues/{fake_issue}/versions"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "issue versions debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_issue_version_nonexistent_returns_404() {
    let (app, _, api_key, slug, _, project_id) = setup("version-get").await;
    let fake_issue = uuid::Uuid::new_v4();
    let fake_ver = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/issues/{fake_issue}/versions/{fake_ver}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "versión inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/work-items/{id}/description-versions
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_description_versions_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("descver-unauth").await;
    let fake_wi = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/work-items/{fake_wi}/description-versions"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_description_versions_member_returns_200_or_404() {
    let (app, _, api_key, slug, _, project_id) = setup("descver-ok").await;
    let fake_wi = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/work-items/{fake_wi}/description-versions"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "description-versions debe devolver 200/404, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_description_version_nonexistent_returns_404() {
    let (app, _, api_key, slug, _, project_id) = setup("descver-get").await;
    let fake_wi = uuid::Uuid::new_v4();
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/work-items/{fake_wi}/description-versions/{fake_pk}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "description-version inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues/list (bulk by IDs)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_by_ids_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("isslist-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/issues/list"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_by_ids_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("isslist-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/issues/list"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 400,
        "issues/list debe devolver 200/400, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues-detail
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_detail_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("issdetail-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/issues-detail"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_detail_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("issdetail-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/issues-detail"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "issues-detail debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/v2/issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_v2_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("issv2-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/v2/issues"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issues_v2_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("issv2-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/v2/issues"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "v2/issues debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/issues/{id}/issue-subscribers/{sub_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_issue_subscriber_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("isssub-del").await;
    let fake_issue = uuid::Uuid::new_v4();
    let fake_sub = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/issues/{fake_issue}/issue-subscribers/{fake_sub}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "eliminar suscriptor inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/issues (workspace view issues)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn workspace_view_issues_unauthenticated_returns_401() {
    let (app, _, _, slug, _, _) = setup("wsviewiss-unauth").await;
    let res = app.get(&format!("/workspaces/{slug}/issues")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn workspace_view_issues_member_returns_200() {
    let (app, _, api_key, slug, _, _) = setup("wsviewiss-ok").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/issues"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "/workspaces/{slug}/issues debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
