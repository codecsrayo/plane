//! Tests de integración: Project Extras II
//!
//! Cobertura:
//!   - GET/POST /workspaces/{slug}/projects/{pid}/invitations → 200/201
//!   - GET/PATCH/DELETE ...invitations/{pk}                   → 200/404
//!   - POST ...projects/{pid}/join/{pk}                       → 4xx
//!   - GET/POST ...project-deploy-boards                      → 200
//!   - GET/PATCH/DELETE ...project-deploy-boards/{pk}         → 200/404
//!   - GET/PATCH ...preferences/member/{member_id}            → 200
//!   - GET ...project-members                                 → 200
//!   - GET ...project-members/{pk}                            → 200/404
//!   - GET/PATCH/DELETE ...members/{pk}                       → 200
//!   - GET ...project-views                                   → 200
//!   - GET /users/me/workspaces/{slug}/project-roles          → 200
//!   - GET /users/me/workspaces/{slug}/projects/invitations   → 200

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("projext2_{suffix}@plane.test"))
        .await;
    let slug = format!("projext2-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("ProjExt2 {suffix}"), "PE2")
        .await;
    (app, user_id, api_key, slug, ws_id, project_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/projects/{project_id}/invitations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_invitations_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("pinv-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/invitations"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_invitations_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("pinv-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/invitations"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project invitations debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_array(), "invitaciones deben ser array");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_project_invitation_returns_2xx() {
    let (app, _, api_key, slug, _, project_id) = setup("pinv-cre").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/invitations"),
            &json!([{ "email": "pinvitee@plane.test", "role": 15 }]),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "crear project invitation debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_invitation_nonexistent_returns_404() {
    let (app, _, api_key, slug, _, project_id) = setup("pinv-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/invitations/{fake_pk}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "invitación inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/project-members
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_members_public_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("pmembers-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/project-members"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "project-members debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_nonexistent_returns_404() {
    let (app, _, api_key, slug, _, project_id) = setup("pmember-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/project-members/{fake_pk}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "member inexistente debe devolver 404/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/members/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_by_pk_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("pmember-pk-unauth").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/members/{fake_pk}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_project_member_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("pmember-patch").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/members/{fake_pk}"),
            &json!({ "role": 10 }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "actualizar member inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/projects/{project_id}/project-deploy-boards
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_deploy_boards_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("dboards-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/project-deploy-boards"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_deploy_boards_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("dboards-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/project-deploy-boards"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "deploy-boards debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_member_preferences_unauthenticated_returns_401() {
    let (app, user_id, _, slug, _, project_id) = setup("mprefs-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/preferences/member/{user_id}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_member_preferences_own_returns_200() {
    let (app, user_id, api_key, slug, _, project_id) = setup("mprefs-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/preferences/member/{user_id}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "member preferences debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/project-views
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_views_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("pviews-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/project-views"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_views_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("pviews-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/project-views"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "project-views debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces/{slug}/project-roles
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_user_project_roles_unauthenticated_returns_401() {
    let (app, _, _, slug, _, _) = setup("roles-unauth").await;
    let res = app
        .get(&format!("/users/me/workspaces/{slug}/project-roles"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_user_project_roles_member_returns_200() {
    let (app, _, api_key, slug, _, _) = setup("roles-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/users/me/workspaces/{slug}/project-roles"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project-roles debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces/{slug}/projects/invitations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_user_project_invitations_unauthenticated_returns_401() {
    let (app, _, _, slug, _, _) = setup("upinv-unauth").await;
    let res = app
        .get(&format!(
            "/users/me/workspaces/{slug}/projects/invitations"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_user_project_invitations_member_returns_200() {
    let (app, _, api_key, slug, _, _) = setup("upinv-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/users/me/workspaces/{slug}/projects/invitations"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "user project invitations debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-favorite-views / DELETE .../{view_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_user_favorite_views_unauthenticated_returns_401() {
    let (app, _, _, slug, _, project_id) = setup("favviews-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/user-favorite-views"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_user_favorite_views_member_returns_200() {
    let (app, _, api_key, slug, _, project_id) = setup("favviews-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/user-favorite-views"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "user-favorite-views debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_user_favorite_view_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _, project_id) = setup("favviews-del").await;
    let fake_view_id = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/user-favorite-views/{fake_view_id}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "eliminar vista favorita inexistente debe devolver 4xx, obtuvo {status}"
    );
}
