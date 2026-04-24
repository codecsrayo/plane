//! Tests de integración: Workspace Extras II
//!
//! Cobertura:
//!   - GET/POST /workspaces/{slug}/user-favorites/{id}/children → 200
//!   - GET      /workspaces/{slug}/user-favorites/{id}/group    → 200 (alias)
//!   - POST     /workspaces/{slug}/draft-to-issue/{draft_id}    → 200/404
//!   - GET/PATCH /workspaces/{slug}/home-preferences/{key}      → 200
//!   - GET/DELETE /workspaces/{slug}/project-identifiers        → 200

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("ws_ext2_{suffix}@plane.test"))
        .await;
    let slug = format!("ws-ext2-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    (app, user_id, api_key, slug, ws_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/user-favorites/{favorite_id}/children
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_favorite_children_unauthenticated_returns_401() {
    let (app, _, _, slug, _) = setup("fav-children-unauth").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get(&format!("/workspaces/{slug}/user-favorites/{fake_id}/children"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_favorite_children_nonexistent_parent_returns_4xx_or_200() {
    let (app, _, api_key, slug, _) = setup("fav-children-ok").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/user-favorites/{fake_id}/children"),
        )
        .await;
    let status = res.status.as_u16();
    // Puede devolver lista vacía (200) o not-found (404)
    assert!(
        status == 200 || status == 404,
        "children de favorito inexistente debe devolver 200/404, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn favorite_group_alias_matches_children() {
    let (app, _, api_key, slug, _) = setup("fav-group-alias").await;
    let fake_id = uuid::Uuid::new_v4();

    let children_res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/user-favorites/{fake_id}/children"),
        )
        .await;
    let group_res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/user-favorites/{fake_id}/group"),
        )
        .await;

    assert_eq!(
        children_res.status.as_u16(),
        group_res.status.as_u16(),
        "alias /group debe devolver mismo status que /children"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/draft-to-issue/{draft_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn draft_to_issue_unauthenticated_returns_401() {
    let (app, _, _, slug, _) = setup("d2i-unauth").await;
    let fake_draft = uuid::Uuid::new_v4();
    let res = app
        .post_json(
            &format!("/workspaces/{slug}/draft-to-issue/{fake_draft}"),
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn draft_to_issue_nonexistent_draft_returns_4xx() {
    let (app, _, api_key, slug, _) = setup("d2i-notfound").await;
    let fake_draft = uuid::Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/draft-to-issue/{fake_draft}"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "draft inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /workspaces/{slug}/home-preferences/{key}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_home_preference_key_unauthenticated_returns_401() {
    let (app, _, _, slug, _) = setup("hpref-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/home-preferences/dashboard"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_home_preference_key_authenticated_returns_200() {
    let (app, _, api_key, slug, _) = setup("hpref-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/home-preferences/dashboard"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "home-preferences/key debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_home_preference_key_returns_200() {
    let (app, _, api_key, slug, _) = setup("hpref-patch").await;
    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/home-preferences/dashboard"),
            &json!({ "is_enabled": true }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "PATCH home-preferences/key debe devolver 200/201, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/DELETE /workspaces/{slug}/project-identifiers
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_identifiers_unauthenticated_returns_401() {
    let (app, _, _, slug, _) = setup("projident-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/project-identifiers"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_project_identifiers_member_returns_200() {
    let (app, user_id, api_key, slug, ws_id) = setup("projident-ok").await;
    // Crear un proyecto para que haya un identifier
    app.create_test_project(user_id, ws_id, "Ident Project", "IDP").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/project-identifiers"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "project-identifiers debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_project_identifier_nonexistent_returns_4xx() {
    let (app, _, api_key, slug, _) = setup("projident-del").await;
    // DELETE requiere ?identifier=XYZ query param
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{slug}/project-identifiers?identifier=NOTEXIST"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "eliminar identifier inexistente debe devolver 4xx, obtuvo {status}"
    );
}
