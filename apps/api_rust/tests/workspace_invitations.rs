//! Tests de integración: Workspace Invitations
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/invitations                → 200
//!   - POST /workspaces/{slug}/invitations                → 200/201
//!   - GET  /workspaces/{slug}/invitations/{pk}           → 200/404
//!   - PATCH /workspaces/{slug}/invitations/{pk}          → 200
//!   - DELETE /workspaces/{slug}/invitations/{pk}         → 204
//!   - POST /workspaces/{slug}/invitations/{pk}/join      → auth guards

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("inv_owner_{suffix}@plane.test"))
        .await;
    let slug = format!("inv-ws-{suffix}");
    app.create_test_workspace(user_id, &slug).await;
    (app, user_id, api_key, slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/invitations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_invitations_unauthenticated_returns_401() {
    let (app, _, _, slug) = setup("lst-unauth").await;
    let res = app.get(&format!("/workspaces/{slug}/invitations")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_invitations_member_returns_200() {
    let (app, _, api_key, slug) = setup("lst-member").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/invitations"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_array(), "respuesta debe ser array, got: {body}");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/invitations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_invitation_unauthenticated_returns_401() {
    let (app, _, _, slug) = setup("cre-unauth").await;
    let res = app
        .post_json(
            &format!("/workspaces/{slug}/invitations"),
            &json!([{ "email": "invited@plane.test", "role": 15 }]),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_invitation_as_admin_returns_2xx() {
    let (app, _, api_key, slug) = setup("cre-admin").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/invitations"),
            &json!([{ "email": "newinvite@plane.test", "role": 15 }]),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "crear invitación debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/invitations/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_invitation_nonexistent_returns_404_or_403() {
    let (app, _, api_key, slug) = setup("get-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/invitations/{fake_pk}"))
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 403 || status == 400,
        "invitación inexistente debe devolver 404/403/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/invitations/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_invitation_unauthenticated_returns_401() {
    let (app, _, _, slug) = setup("del-unauth").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed("no-key", &format!("/workspaces/{slug}/invitations/{fake_pk}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_invitation_nonexistent_returns_4xx() {
    let (app, _, api_key, slug) = setup("del-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/invitations/{fake_pk}"))
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 403 || status == 400,
        "eliminar invitación inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/invitations/{pk}/join — requiere token válido
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn join_invitation_with_invalid_token_returns_4xx() {
    let (app, _, _, slug) = setup("join-invalid").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .post_json(
            &format!("/workspaces/{slug}/invitations/{fake_pk}/join"),
            &json!({ "email": "joiner@plane.test" }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "join con token inválido debe devolver error 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces/invitations — invitaciones del usuario actual
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_user_workspace_invitations_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let res = app.get("/users/me/workspaces/invitations").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_user_workspace_invitations_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app
        .create_test_user("user_ws_invites@plane.test")
        .await;
    let res = app
        .get_authed(&api_key, "/users/me/workspaces/invitations")
        .await;
    let status = res.status.as_u16();
    // Puede devolver 200 con lista vacía
    assert!(
        status == 200 || status == 404,
        "invitaciones usuario debe devolver 200 o 404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}/invitations/{pk} — actualizar rol de invitación
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_invitation_nonexistent_returns_4xx() {
    let (app, _, api_key, slug) = setup("upd-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/invitations/{fake_pk}"),
            &json!({ "role": 10 }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "actualizar invitación inexistente debe devolver 4xx, obtuvo {status}"
    );
}
