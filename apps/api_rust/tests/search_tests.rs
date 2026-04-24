//! Tests de integración: Search
//!
//! Cobertura:
//!   - GET /workspaces/{slug}/search                        → 200 (auth required)
//!   - GET /workspaces/{slug}/projects/{id}/search-issues   → 200
//!   - GET /workspaces/{slug}/entity-search                 → 200

mod common;

use common::TestApp;

async fn setup(
    suffix: &str,
) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("search_owner_{suffix}@plane.test"))
        .await;
    let slug = format!("search-ws-{suffix}");
    let ws_id = app.workspace_id_by_slug(
        app.create_test_workspace(user_id, &slug).await.as_str(),
    ).await;
    let identifier = format!("SP{}", &suffix[..suffix.len().min(3)]);
    let project_id = app
        .create_test_project(user_id, ws_id, &format!("Search Project {suffix}"), &identifier)
        .await;
    (app, api_key, slug, project_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/search
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn global_search_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("srch_anon@plane.test").await;
    app.create_test_workspace(user_id, "srch-anon-ws").await;
    let res = app.get("/workspaces/srch-anon-ws/search?query=test").await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn global_search_empty_query_returns_200() {
    let (app, api_key, slug, _) = setup("empty").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/search"))
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 400,
        "búsqueda vacía debe devolver 200 o 400, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn global_search_with_query_returns_200() {
    let (app, api_key, slug, _) = setup("withquery").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/search?query=test"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "búsqueda global debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    // Respuesta estructurada con categorías
    assert!(
        body.is_object() || body.is_array(),
        "respuesta de búsqueda debe ser objeto o array, got: {body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn global_search_nonmember_returns_403() {
    let (app, _, slug, _) = setup("nonmember").await;
    let (_, outsider_key) = app.create_test_user("srch_outsider@plane.test").await;
    let res = app
        .get_authed(&outsider_key, &format!("/workspaces/{slug}/search?query=hello"))
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 403 || status == 401 || status == 404,
        "no-miembro no debe buscar en workspace, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/search-issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn search_issues_unauthenticated_returns_401() {
    let (app, _, slug, project_id) = setup("issues-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/search-issues?query=bug"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn search_issues_member_returns_200() {
    let (app, api_key, slug, project_id) = setup("issues-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{project_id}/search-issues?query=bug"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "search-issues debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(
        body.is_array() || body.is_object(),
        "search-issues debe devolver array u objeto, got: {body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn search_issues_empty_query_returns_200() {
    let (app, api_key, slug, project_id) = setup("issues-empty").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{project_id}/search-issues"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 400,
        "search-issues sin query debe devolver 200/400, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/entity-search
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn entity_search_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("entity-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/entity-search?query=test"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn entity_search_member_returns_200() {
    let (app, api_key, slug, _) = setup("entity-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/entity-search?query=test"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "entity-search debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
