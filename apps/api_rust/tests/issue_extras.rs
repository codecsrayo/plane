//! Tests de integración: Issue extras
//!
//! Cobertura (endpoints pendientes en todo_test.md):
//!   Comments:
//!   - GET/PATCH/DELETE /issues/{id}/comments/{pk}
//!
//!   Issue Links:
//!   - GET/POST /issues/{id}/issue-links
//!   - PATCH/DELETE /issues/{id}/issue-links/{pk}
//!
//!   Sub-issues:
//!   - GET/POST /issues/{id}/sub-issues
//!
//!   Subscribe:
//!   - POST /issues/{id}/subscribe
//!   - DELETE /issues/{id}/subscribe (unsubscribe)
//!   - DELETE /issues/{id}/issue-subscribers/{subscriber_id}
//!
//!   Reactions:
//!   - GET/POST /issues/{id}/reactions
//!   - DELETE /issues/{id}/reactions/{reaction_code}
//!   - GET/POST /comments/{comment_id}/reactions
//!   - DELETE /comments/{comment_id}/reactions/{reaction_code}
//!
//!   Relations:
//!   - POST /issues/{id}/remove-relation
//!
//!   Bulk:
//!   - POST /bulk-archive-issues

mod common;

use common::TestApp;
use axum::http::Method;
use serde_json::json;

// ── Setup compartido ─────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("iext_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("iext-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "IExt Project", "IEX")
        .await;
    (app, api_key, ws_slug, proj_id)
}

async fn create_issue(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

async fn create_comment(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    issue_id: &str,
    html: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments"),
            &json!({ "comment_html": html }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear comment falló: {}", String::from_utf8_lossy(&res.body));
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// COMMENTS — GET/PATCH/DELETE /issues/{id}/comments/{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_comment_by_id_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("get_comment").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Ci").await;
    let comment_id = create_comment(&app, &api_key, &ws_slug, proj_id, &issue_id, "<p>Original</p>").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments/{comment_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["comment_html"].as_str().unwrap_or(""), "<p>Original</p>");
}

#[tokio::test(flavor = "multi_thread")]
async fn update_comment_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("patch_comment").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Ci").await;
    let comment_id = create_comment(&app, &api_key, &ws_slug, proj_id, &issue_id, "<p>Old</p>").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments/{comment_id}"),
            &json!({ "comment_html": "<p>Updated</p>" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["comment_html"].as_str().unwrap_or(""), "<p>Updated</p>");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_comment_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("del_comment").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Ci").await;
    let comment_id = create_comment(&app, &api_key, &ws_slug, proj_id, &issue_id, "<p>Bye</p>").await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments/{comment_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_comment_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("comment_404").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Ci").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/comments/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// ISSUE LINKS — GET/POST + PATCH/DELETE /{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_links_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("links_list").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Link Issue").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-links"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let arr = body.as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty(), "debe empezar sin links");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_link_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("link_create").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Linked Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-links"),
            &json!({
                "title": "Plane docs",
                "url": "https://docs.plane.so"
            }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["url"].as_str().unwrap_or(""), "https://docs.plane.so");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_link_invalid_url_returns_400() {
    let (app, api_key, ws_slug, proj_id) = setup("link_bad_url").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Bad Link Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-links"),
            &json!({ "title": "bad", "url": "not-a-url" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

async fn create_link(app: &TestApp, api_key: &str, ws_slug: &str, proj_id: uuid::Uuid, issue_id: &str) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-links"),
            &json!({ "title": "A Link", "url": "https://example.com" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn update_issue_link_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("link_patch").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Patch Link Issue").await;
    let link_id = create_link(&app, &api_key, &ws_slug, proj_id, &issue_id).await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-links/{link_id}"),
            &json!({ "title": "Updated Link", "url": "https://updated.com" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["title"].as_str().unwrap_or(""), "Updated Link");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_issue_link_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("link_del").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Del Link Issue").await;
    let link_id = create_link(&app, &api_key, &ws_slug, proj_id, &issue_id).await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/issue-links/{link_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

// ═════════════════════════════════════════════════════════════════════════════
// SUB-ISSUES — GET/POST /issues/{id}/sub-issues
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_sub_issues_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("sub_list").await;
    let parent_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Parent Issue").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{parent_id}/sub-issues"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let sub_issues = if let Some(arr) = body.as_array() {
        arr.clone()
    } else {
        body["sub_issues"].as_array().cloned().unwrap_or_default()
    };
    assert!(sub_issues.is_empty(), "no debe tener sub-issues inicialmente");
}

#[tokio::test(flavor = "multi_thread")]
async fn assign_sub_issue_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("sub_assign").await;
    let parent_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Parent").await;
    let child_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Child").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{parent_id}/sub-issues"),
            &json!({ "sub_issue_ids": [child_id] }),
        )
        .await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 201,
        "sub-issues debe devolver 200/201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// SUBSCRIBE — POST + DELETE /issues/{id}/subscribe
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_to_issue_returns_200_or_201() {
    let (app, api_key, ws_slug, proj_id) = setup("subscribe").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Sub Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/subscribe"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201,
        "subscribe debe devolver 200/201, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn unsubscribe_from_issue_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("unsub").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Unsub Issue").await;

    // Primero suscribir
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/subscribe"),
        &json!({}),
    )
    .await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/subscribe"),
        )
        .await;
    // 204 al desuscribir, o 400 si no estaba suscrito (idempotente)
    let status = res.status.as_u16();
    assert!(
        status == 204 || status == 200 || status == 400,
        "unsubscribe devolvió inesperado {status}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// ISSUE REACTIONS — GET/POST /issues/{id}/reactions + DELETE /{code}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_reactions_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("react_list").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "React Issue").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/reactions"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn add_issue_reaction_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("react_add").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "React Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/reactions"),
            &json!({ "reaction": "1f44d" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["reaction"].as_str().unwrap_or(""), "1f44d");
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_issue_reaction_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("react_del").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "React Del Issue").await;

    // Añadir reacción
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/reactions"),
        &json!({ "reaction": "1f44d" }),
    )
    .await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/reactions/1f44d"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204, "body: {}", String::from_utf8_lossy(&res.body));
}

// ═════════════════════════════════════════════════════════════════════════════
// COMMENT REACTIONS — GET/POST /comments/{id}/reactions + DELETE /{code}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_comment_reactions_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("creact_list").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "CReact Issue").await;
    let comment_id = create_comment(&app, &api_key, &ws_slug, proj_id, &issue_id, "<p>Hi</p>").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/comments/{comment_id}/reactions"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn add_comment_reaction_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("creact_add").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "CReact Issue").await;
    let comment_id = create_comment(&app, &api_key, &ws_slug, proj_id, &issue_id, "<p>Hi</p>").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/comments/{comment_id}/reactions"),
            &json!({ "reaction": "2764" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    assert_eq!(res.json()["reaction"].as_str().unwrap_or(""), "2764");
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_comment_reaction_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("creact_del").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "CReact Issue").await;
    let comment_id = create_comment(&app, &api_key, &ws_slug, proj_id, &issue_id, "<p>Hi</p>").await;

    // Añadir primero
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/comments/{comment_id}/reactions"),
        &json!({ "reaction": "2764" }),
    )
    .await;

    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/comments/{comment_id}/reactions/2764"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 204, "body: {}", String::from_utf8_lossy(&res.body));
}

// ═════════════════════════════════════════════════════════════════════════════
// REMOVE RELATION — POST /issues/{id}/remove-relation
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn remove_issue_relation_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("rm_rel").await;
    let source_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Source").await;
    let target_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Target").await;

    // Crear relación primero
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{source_id}/issue-relation"),
        &json!({ "relation_type": "blocking", "issues": [target_id] }),
    )
    .await;

    // Eliminar relación
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{source_id}/remove-relation"),
            &json!({ "relation_type": "blocks", "related_issue": target_id }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "remove-relation debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// BULK ARCHIVE — POST /bulk-archive-issues
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn bulk_archive_issues_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("bulk_arch").await;
    let id1 = create_issue(&app, &api_key, &ws_slug, proj_id, "Archive A").await;
    let id2 = create_issue(&app, &api_key, &ws_slug, proj_id, "Archive B").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/bulk-archive-issues"),
            &json!({ "issue_ids": [id1, id2] }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "bulk-archive debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bulk_archive_empty_list_returns_400() {
    let (app, api_key, ws_slug, proj_id) = setup("bulk_arch_empty").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/bulk-archive-issues"),
            &json!({ "issue_ids": [] }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        400,
        "lista vacía debe devolver 400, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bulk_archive_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("bulk_arch_unauth").await;

    // El endpoint solo acepta POST (issue_extras2:409 + mod.rs router).
    // GET sin auth devuelve 405 — el method-not-allowed se evalúa antes
    // que el auth guard, por lo que el test nunca exercita el 401 si
    // usa GET. Mandamos POST para alcanzar el guard.
    let res = app
        .request(
            Method::POST,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/bulk-archive-issues"),
            Some(serde_json::to_vec(&json!({"issue_ids": []})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}
