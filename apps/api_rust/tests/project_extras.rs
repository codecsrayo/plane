//! Tests de integración: Project extras
//!
//! Cobertura:
//!   - POST/DELETE  /workspaces/{slug}/projects/{project_id}/archive
//!   - GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/members/{pk}
//!   - POST         /workspaces/{slug}/projects/{project_id}/members/leave
//!   - GET          /workspaces/{slug}/projects/{project_id}/project-members/me
//!   - POST         /workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default
//!   - GET/PATCH    /workspaces/{slug}/projects/{project_id}/user-properties

mod common;

use common::TestApp;
use serde_json::json;

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("pext_{suffix}@plane.test");
    let (owner_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("pe-{suffix}");
    app.create_test_workspace(owner_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(owner_id, ws_id, "PExt Project", "PEX")
        .await;
    (app, owner_id, api_key, ws_slug, proj_id)
}

// ═════════════════════════════════════════════════════════════════════════════
// PROJECT ARCHIVE / UNARCHIVE
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn archive_project_returns_200_or_204() {
    let (app, _, api_key, ws_slug, proj_id) = setup("arch").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archive"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "archive project debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_project_returns_200_or_204() {
    let (app, _, api_key, ws_slug, proj_id) = setup("unarch").await;

    // Archivar primero
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/archive"),
        &json!({}),
    )
    .await;

    // Desarchivar
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archive"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "unarchive project debe devolver 200/204, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn archive_project_unauthenticated_returns_401() {
    let (app, _, _, ws_slug, proj_id) = setup("arch_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/archive"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ═════════════════════════════════════════════════════════════════════════════
// PROJECT MEMBERS: GET/PATCH/DELETE /{pk} + leave + me
// ═════════════════════════════════════════════════════════════════════════════

async fn get_owner_project_member_pk(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
) -> String {
    let res = app
        .get_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/project-members/me"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "project-members/me: {}", String::from_utf8_lossy(&res.body));
    res.json()["id"].as_str().unwrap_or("").to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_me_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("me").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/project-members/me"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    // Debe contener rol (20 = Admin para el owner)
    assert!(body["role"].as_i64().is_some(), "project-members/me debe devolver role");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_me_unauthenticated_returns_401() {
    let (app, _, _, ws_slug, proj_id) = setup("me_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/project-members/me"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_by_pk_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("mem_get").await;
    let pk = get_owner_project_member_pk(&app, &api_key, &ws_slug, proj_id).await;
    if pk.is_empty() { return; }

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/members/{pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_not_found_returns_404() {
    let (app, _, api_key, ws_slug, proj_id) = setup("mem_404").await;
    let fake_pk = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/members/{fake_pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_project_member_role_returns_200() {
    let (app, owner_id, owner_key, ws_slug, proj_id) = setup("mem_patch").await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;

    // Crear segundo usuario y añadirlo al workspace + proyecto
    let (member_id, member_key) = app.create_test_user("pext_patch_m@plane.test").await;
    app.add_workspace_member(member_id, ws_id, 10).await;
    app.post_json_authed(
        &owner_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/members"),
        &json!({ "member_ids": [member_id], "role": 10 }),
    )
    .await;

    // Obtener pk del miembro en el proyecto
    let me_res = app
        .get_authed(
            &member_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/project-members/me"),
        )
        .await;
    let member_pk = me_res.json()["id"].as_str().unwrap_or("").to_owned();
    if member_pk.is_empty() { let _ = owner_id; return; }

    // Admin actualiza rol
    let res = app
        .patch_json_authed(
            &owner_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/members/{member_pk}"),
            &json!({ "role": 5 }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let _ = owner_id;
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_project_member_returns_204() {
    let (app, owner_id, owner_key, ws_slug, proj_id) = setup("mem_del").await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;

    let (member_id, member_key) = app.create_test_user("pext_del_m@plane.test").await;
    app.add_workspace_member(member_id, ws_id, 10).await;
    app.post_json_authed(
        &owner_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/members"),
        &json!({ "member_ids": [member_id], "role": 10 }),
    )
    .await;

    let me_res = app
        .get_authed(
            &member_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/project-members/me"),
        )
        .await;
    let member_pk = me_res.json()["id"].as_str().unwrap_or("").to_owned();
    if member_pk.is_empty() { let _ = owner_id; return; }

    let res = app
        .delete_authed(
            &owner_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/members/{member_pk}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204, "body: {}", String::from_utf8_lossy(&res.body));
    let _ = owner_id;
}

#[tokio::test(flavor = "multi_thread")]
async fn leave_project_as_member_returns_204() {
    let (app, owner_id, owner_key, ws_slug, proj_id) = setup("leave").await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;

    let (member_id, member_key) = app.create_test_user("pext_leave_m@plane.test").await;
    app.add_workspace_member(member_id, ws_id, 10).await;
    app.post_json_authed(
        &owner_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/members"),
        &json!({ "member_ids": [member_id], "role": 10 }),
    )
    .await;

    let res = app
        .post_json_authed(
            &member_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/members/leave"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 204 || status == 200,
        "leave project debe devolver 204/200, obtuvo {status}"
    );
    let _ = owner_id;
}

// ═════════════════════════════════════════════════════════════════════════════
// STATES: POST /{pk}/mark-default
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn mark_state_as_default_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("mark_def").await;

    // Crear un estado
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/states"),
            &json!({ "name": "Backlog", "color": "#a8c1e0", "group": "backlog" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    let state_id = res.json()["id"].as_str().unwrap().to_owned();

    // Marcar como default
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/states/{state_id}/mark-default"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "mark-default debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mark_default_nonexistent_state_returns_404() {
    let (app, _, api_key, ws_slug, proj_id) = setup("mark_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/states/{fake_id}/mark-default"),
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// USER-PROPERTIES — GET/PATCH /projects/{project_id}/user-properties
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_project_user_properties_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("user_props_get").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-properties"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_project_user_properties_returns_200() {
    let (app, _, api_key, ws_slug, proj_id) = setup("user_props_patch").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-properties"),
            &json!({ "display_properties": { "priority": true } }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_project_user_properties_unauthenticated_returns_401() {
    let (app, _, _, ws_slug, proj_id) = setup("user_props_unauth").await;

    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/user-properties"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}
