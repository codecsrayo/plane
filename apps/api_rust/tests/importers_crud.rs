//! Tests de integración: Importer detail-URL — DELETE /{importer_id}
//!
//! Paridad con Django (`apps/api/plane/app/urls/importer.py`):
//!   GithubImporterEndpoint y GitlabImporterEndpoint exponen `get`/`post`/
//!   `delete` pero la URL `…/importers/{provider}/{importer_id}/` solo enruta
//!   a `delete(self, request, slug, importer_id)`. Llamar GET/PATCH sobre esa
//!   URL en Django resulta en TypeError (firma sin `importer_id`) o 405. El
//!   Rust API replica el contrato exponiendo únicamente DELETE en esa ruta,
//!   así que los tests previos para GET/PATCH (que esperaban 401/404) eran
//!   endpoints fantasma y se removieron.
//!
//! Cobertura:
//!   - DELETE /workspaces/{slug}/importers/github/{importer_id}  → 401, 404
//!   - DELETE /workspaces/{slug}/importers/gitlab/{importer_id}  → 401, 404

mod common;

use common::TestApp;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String) {
    let app = TestApp::spawn().await;
    let email = format!("imp_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("imp-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, api_key, ws_slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// GitHub importer — DELETE /{importer_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_github_importer_unauthenticated_returns_401() {
    let (app, _, slug) = setup("gh_del_unauth").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed("", &format!("/workspaces/{slug}/importers/github/{fake_id}"))
        .await;
    // Sin api_key → esperamos 401 (no 403)
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_github_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gh_del_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/importers/github/{fake_id}"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete importer inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GitLab importer — DELETE /{importer_id}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_gitlab_importer_nonexistent_returns_404() {
    let (app, api_key, slug) = setup("gl_del_404").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{slug}/importers/gitlab/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}
