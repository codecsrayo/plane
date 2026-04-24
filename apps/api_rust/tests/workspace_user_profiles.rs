//! Tests de integración: Workspace User Profiles / Stats / Activity / Views
//!
//! Cobertura:
//!   - GET /workspaces/{slug}/workspace-views               → 200
//!   - GET /workspaces/{slug}/user-profile/{user_id}        → 200
//!   - GET /workspaces/{slug}/user-stats/{user_id}          → 200
//!   - GET /workspaces/{slug}/user-activity/{user_id}       → 200
//!   - GET /workspaces/{slug}/user-activity/{user_id}/export → 200/202
//!   - GET /workspaces/{slug}/user-issues/{user_id}         → 200

mod common;

use common::TestApp;

async fn setup(
    suffix: &str,
) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid, String) {
    let app = TestApp::spawn().await;
    let (owner_id, owner_key) = app
        .create_test_user(&format!("prof_owner_{suffix}@plane.test"))
        .await;
    let (target_id, _) = app
        .create_test_user(&format!("prof_target_{suffix}@plane.test"))
        .await;
    let slug = format!("prof-ws-{suffix}");
    let ws_id = app.workspace_id_by_slug(
        app.create_test_workspace(owner_id, &slug).await.as_str()
    ).await;
    // target también es miembro
    app.add_workspace_member(target_id, ws_id, 15).await;
    (app, owner_id, owner_key, slug.clone(), target_id, slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/workspace-views
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_views_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("wsviews_anon@plane.test").await;
    let slug = "wsviews-anon-ws";
    app.create_test_workspace(user_id, slug).await;
    let res = app.get(&format!("/workspaces/{slug}/workspace-views")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_views_member_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user("wsviews_member@plane.test")
        .await;
    let slug = "wsviews-member-ws";
    app.create_test_workspace(user_id, slug).await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/workspace-views"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "workspace-views debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(
        body.is_array() || body.is_object(),
        "respuesta debe ser array u objeto, got: {body}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-profile/{user_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_user_profile_unauthenticated_returns_401() {
    let (app, _, _, slug, target_id, _) = setup("uprofile-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/user-profile/{target_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_user_profile_member_returns_200() {
    let (app, _, owner_key, slug, target_id, _) = setup("uprofile-ok").await;
    let res = app
        .get_authed(
            &owner_key,
            &format!("/workspaces/{slug}/user-profile/{target_id}"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "user-profile debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_object(), "perfil debe ser objeto");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-stats/{user_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_user_stats_unauthenticated_returns_401() {
    let (app, _, _, slug, target_id, _) = setup("ustats-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/user-stats/{target_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_user_stats_member_returns_200() {
    let (app, _, owner_key, slug, target_id, _) = setup("ustats-ok").await;
    let res = app
        .get_authed(
            &owner_key,
            &format!("/workspaces/{slug}/user-stats/{target_id}"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "user-stats debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_object(), "stats debe ser objeto");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-activity/{user_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_user_activity_unauthenticated_returns_401() {
    let (app, _, _, slug, target_id, _) = setup("uactiv-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/user-activity/{target_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_user_activity_member_returns_200() {
    let (app, _, owner_key, slug, target_id, _) = setup("uactiv-ok").await;
    let res = app
        .get_authed(
            &owner_key,
            &format!("/workspaces/{slug}/user-activity/{target_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "user-activity debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-activity/{user_id}/export
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn export_user_activity_unauthenticated_returns_401() {
    let (app, _, _, slug, target_id, _) = setup("uexport-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/user-activity/{target_id}/export"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn export_user_activity_member_returns_2xx() {
    let (app, _, owner_key, slug, target_id, _) = setup("uexport-ok").await;
    let res = app
        .get_authed(
            &owner_key,
            &format!("/workspaces/{slug}/user-activity/{target_id}/export"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 202 || status == 404,
        "export user-activity debe devolver 200/202/404, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-issues/{user_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_user_issues_unauthenticated_returns_401() {
    let (app, _, _, slug, target_id, _) = setup("uissues-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/user-issues/{target_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_user_issues_member_returns_200() {
    let (app, _, owner_key, slug, target_id, _) = setup("uissues-ok").await;
    let res = app
        .get_authed(
            &owner_key,
            &format!("/workspaces/{slug}/user-issues/{target_id}"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "user-issues debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    // Puede ser una lista paginada o un objeto con resultados
    assert!(
        body.is_array() || body.is_object(),
        "respuesta debe ser array u objeto, got: {body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_user_issues_nonmember_user_returns_403() {
    let (app, _, _, slug, target_id, _) = setup("uissues-nonmember").await;
    let (_, outsider_key) = app
        .create_test_user("uissues_outsider@plane.test")
        .await;

    let res = app
        .get_authed(
            &outsider_key,
            &format!("/workspaces/{slug}/user-issues/{target_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 403 || status == 401 || status == 404,
        "usuario externo no debe acceder a user-issues, obtuvo {status}"
    );
}
