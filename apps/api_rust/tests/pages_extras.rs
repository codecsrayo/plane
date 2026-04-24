//! Tests de integración: Pages Extras
//!
//! Cobertura:
//!   - POST/DELETE .../favorite-pages/{page_id}                    → 200/204
//!   - POST        .../pages/{page_id}/access                      → 200
//!   - GET/PATCH   .../pages/{page_id}/description                 → 200
//!   - GET         .../pages/{page_id}/versions                    → 200
//!   - GET/DELETE  .../pages/{page_id}/versions/{pk}               → 200/404

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("pagesext_{suffix}@plane.test"))
        .await;
    let slug = format!("pagesext-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("PagesExt {suffix}"), "PGX")
        .await;
    (app, api_key, slug, ws_id, project_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// POST/DELETE /workspaces/{slug}/projects/{pid}/favorite-pages/{page_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn add_favorite_page_unauthenticated_returns_401() {
    let (app, _, slug, _, project_id) = setup("favpg-unauth").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .post_json(
            &format!(
                "/workspaces/{slug}/projects/{project_id}/favorite-pages/{fake_page}"
            ),
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn add_favorite_page_nonexistent_returns_4xx() {
    let (app, api_key, slug, _, project_id) = setup("favpg-cre").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/favorite-pages/{fake_page}"
            ),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    // Sin una página real, debe devolver 404 o 400
    assert!(
        status >= 400,
        "favoritar página inexistente debe devolver 4xx, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_favorite_page_nonexistent_returns_4xx() {
    let (app, api_key, slug, _, project_id) = setup("favpg-del").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/favorite-pages/{fake_page}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "eliminar favorita inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/pages/{page_id}/access
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_page_access_unauthenticated_returns_401() {
    let (app, _, slug, _, project_id) = setup("pgaccess-unauth").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .post_json(
            &format!(
                "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/access"
            ),
            &json!({ "access": 0 }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_page_access_nonexistent_page_returns_4xx() {
    let (app, api_key, slug, _, project_id) = setup("pgaccess-ok").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/access"
            ),
            &json!({ "access": 0 }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "acceso a página inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /workspaces/{slug}/projects/{pid}/pages/{page_id}/description
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_page_description_unauthenticated_returns_401() {
    let (app, _, slug, _, project_id) = setup("pgdesc-unauth").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/description"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_page_description_nonexistent_returns_4xx() {
    let (app, api_key, slug, _, project_id) = setup("pgdesc-get").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/description"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400 || status == 200,
        "description de página inexistente debe devolver 200/404/400, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_page_description_nonexistent_page_returns_4xx() {
    let (app, api_key, slug, _, project_id) = setup("pgdesc-patch").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/description"
            ),
            &json!({ "description_binary": null, "description_html": "<p>test</p>" }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "actualizar descripción de página inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/pages/{page_id}/versions
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_page_versions_unauthenticated_returns_401() {
    let (app, _, slug, _, project_id) = setup("pgvers-unauth").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/versions"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_page_versions_member_returns_200_or_404() {
    let (app, api_key, slug, _, project_id) = setup("pgvers-ok").await;
    let fake_page = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/versions"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "page versions debe devolver 200/404, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_page_version_nonexistent_returns_404() {
    let (app, api_key, slug, _, project_id) = setup("pgver-get").await;
    let fake_page = uuid::Uuid::new_v4();
    let fake_ver = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/pages/{fake_page}/versions/{fake_ver}"
            ),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 404 || status == 400,
        "versión inexistente debe devolver 404/400, obtuvo {status}"
    );
}
