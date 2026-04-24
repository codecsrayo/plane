//! Tests de integración: Workspace Themes
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/workspace-themes             → 200 (lista)
//!   - POST /workspaces/{slug}/workspace-themes             → 200/201
//!   - GET  /workspaces/{slug}/workspace-themes/{pk}        → 200/404
//!   - PATCH /workspaces/{slug}/workspace-themes/{pk}       → 200
//!   - DELETE /workspaces/{slug}/workspace-themes/{pk}      → 204

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, uuid::Uuid, String, String) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("theme_owner_{suffix}@plane.test"))
        .await;
    let slug = format!("theme-ws-{suffix}");
    app.create_test_workspace(user_id, &slug).await;
    (app, user_id, api_key, slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/workspace-themes
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_themes_unauthenticated_returns_401() {
    let (app, _, _, slug) = setup("lst-unauth").await;
    let res = app.get(&format!("/workspaces/{slug}/workspace-themes")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_themes_member_returns_200() {
    let (app, _, api_key, slug) = setup("lst-member").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/workspace-themes"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_array(), "temas deben ser array, got: {body}");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/workspace-themes
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_theme_unauthenticated_returns_401() {
    let (app, _, _, slug) = setup("cre-unauth").await;
    let res = app
        .post_json(
            &format!("/workspaces/{slug}/workspace-themes"),
            &json!({ "name": "Dark Theme", "theme_data": {} }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_workspace_theme_returns_2xx() {
    let (app, _, api_key, slug) = setup("cre-ok").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-themes"),
            &json!({
                "name": "Custom Dark",
                "actor": "#0f172a",
                "pointer": "#334155",
                "background": "#1e293b",
                "text": "#f1f5f9",
                "sidebarBackground": "#0f172a",
                "sidebarText": "#cbd5e1",
                "sidebarHighlight": "#1e3a5f"
            }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "crear tema debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_object(), "respuesta debe ser objeto");
}

// ─────────────────────────────────────────────────────────────────────────────
// CRUD completo: GET/PATCH/DELETE /workspace-themes/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn workspace_themes_crud() {
    let (app, _, api_key, slug) = setup("crud").await;

    // CREATE
    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-themes"),
            &json!({
                "name": "CRUD Theme",
                "actor": "#ffffff",
                "pointer": "#ffffff",
                "background": "#000000",
                "text": "#ffffff",
                "sidebarBackground": "#111111",
                "sidebarText": "#eeeeee",
                "sidebarHighlight": "#222222"
            }),
        )
        .await;
    let status = create_res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "crear tema debe devolver 200/201, obtuvo {status}"
    );
    let created = create_res.json();
    let theme_id = created["id"]
        .as_str()
        .expect("tema creado debe tener id");

    // GET
    let get_res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/workspace-themes/{theme_id}"))
        .await;
    assert_eq!(
        get_res.status.as_u16(),
        200,
        "GET tema debe devolver 200"
    );

    // PATCH
    let patch_res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-themes/{theme_id}"),
            &json!({ "name": "CRUD Theme Updated" }),
        )
        .await;
    assert_eq!(
        patch_res.status.as_u16(),
        200,
        "PATCH tema debe devolver 200"
    );
    let patched = patch_res.json();
    assert_eq!(
        patched["name"].as_str(),
        Some("CRUD Theme Updated"),
        "nombre debe actualizarse"
    );

    // DELETE
    let del_res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/workspace-themes/{theme_id}"))
        .await;
    let del_status = del_res.status.as_u16();
    assert!(
        del_status == 200 || del_status == 204,
        "DELETE tema debe devolver 200/204, obtuvo {del_status}"
    );

    // GET post-delete → 404
    let get_after = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/workspace-themes/{theme_id}"))
        .await;
    assert_eq!(
        get_after.status.as_u16(),
        404,
        "tema eliminado debe devolver 404"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Tema inexistente
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_nonexistent_theme_returns_404() {
    let (app, _, api_key, slug) = setup("notfound").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/workspace-themes/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404, "tema inexistente debe devolver 404");
}
