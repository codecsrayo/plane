//! Tests de integración: Work Items Extras + Legacy Issues
//!
//! Cobertura:
//!   - GET /workspaces/{slug}/work-items/search         → 401, 200
//!   - GET /workspaces/{slug}/issues/search             → 401, 200 (legacy)
//!   - GET /workspaces/{slug}/issues/{combined}         → 401, 404 (legacy)
//!   - GET /workspaces/{slug}/work-items/{combined}     → 401, 404
//!   - GET /workspaces/{slug}/projects/{pid}/work-items/{iid}/activities/{pk}
//!         → 401, 404 actividad inexistente
//!   - GET/POST /workspaces/{slug}/projects/{pid}/issues/{iid}/links  (legacy)
//!         → 401, 200 vacío, 201 creado
//!   - PATCH/DELETE /workspaces/{slug}/projects/{pid}/issues/{iid}/links/{pk}
//!         → 401, 404 inexistente

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

// ── Setup ─────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("wie_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("wie-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "WI Extras", "WIE")
        .await;
    (app, api_key, ws_slug, ws_id, proj_id)
}

async fn create_issue(app: &TestApp, api_key: &str, slug: &str, proj_id: Uuid) -> Uuid {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues"),
            &json!({ "name": "WI extras issue" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().parse().unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/work-items/search
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn work_items_search_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("wi_search_unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/work-items/search"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn work_items_search_authenticated_returns_200() {
    let (app, api_key, slug, _, _) = setup("wi_search_ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/work-items/search?search=nonexistent"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "work-items/search debe retornar 200: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/issues/search  (legacy prefix)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn legacy_issues_search_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("iss_search_unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/issues/search"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn legacy_issues_search_authenticated_returns_200() {
    let (app, api_key, slug, _, _) = setup("iss_search_ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/issues/search?search=test"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "issues/search legacy debe retornar 200: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/issues/{combined}  (legacy)
// GET /workspaces/{slug}/work-items/{combined}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn legacy_issues_combined_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("iss_combined_unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/issues/WIE-1"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn legacy_issues_combined_not_found_returns_404() {
    let (app, api_key, slug, _, _) = setup("iss_combined_404").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/issues/WIE-9999"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "combined inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn work_items_combined_unauthenticated_returns_401() {
    let (app, _, slug, _, _) = setup("wi_combined_unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/work-items/WIE-1"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn work_items_combined_not_found_returns_404() {
    let (app, api_key, slug, _, _) = setup("wi_combined_404").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/work-items/WIE-9999"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/work-items/{iid}/activities/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_work_item_activity_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("wi_act_pk_unauth").await;
    let issue_id = Uuid::new_v4();
    let act_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/work-items/{issue_id}/activities/{act_id}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_work_item_activity_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("wi_act_pk_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_act_id = Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/work-items/{issue_id}/activities/{fake_act_id}"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "actividad inexistente debe devolver 404"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_legacy_issue_activity_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("iss_act_pk_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_act_id = Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/activities/{fake_act_id}"
            ),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/projects/{pid}/issues/{iid}/links  (legacy)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_links_legacy_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("iss_links_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_links_legacy_empty_returns_200() {
    let (app, api_key, slug, _, proj_id) = setup("iss_links_empty").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "links vacíos legacy: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body.as_array().map(|a| a.len()).unwrap_or(0), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_issue_link_legacy_creates_and_list_reflects() {
    let (app, api_key, slug, _, proj_id) = setup("iss_link_create").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    // Crear link
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links"),
            &json!({
                "title": "Documentation",
                "url": "https://docs.plane.so"
            }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear link legacy: {}",
        String::from_utf8_lossy(&res.body)
    );
    let link_id = res.json()["id"].as_str().expect("id en link").to_owned();

    // Listar y verificar que aparece
    let list_res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links"),
        )
        .await;
    assert_eq!(list_res.status.as_u16(), 200);
    let links = list_res.json();
    let arr = links.as_array().expect("array de links");
    assert!(
        arr.iter().any(|l| l["id"].as_str() == Some(&link_id)),
        "el link creado debe aparecer en la lista"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH / DELETE /workspaces/{slug}/projects/{pid}/issues/{iid}/links/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn patch_issue_link_legacy_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("iss_link_patch_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_pk = Uuid::new_v4();

    let res = app
        .patch_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links/{fake_pk}"
            ),
            &json!({ "title": "Updated" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_issue_link_legacy_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("iss_link_del_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_pk = Uuid::new_v4();

    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links/{fake_pk}"
            ),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_then_delete_issue_link_legacy_full_cycle() {
    let (app, api_key, slug, _, proj_id) = setup("iss_link_cycle").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    // Crear
    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links"),
            &json!({ "title": "Original", "url": "https://plane.so" }),
        )
        .await;
    assert_eq!(create_res.status.as_u16(), 201);
    let pk = create_res.json()["id"].as_str().unwrap().to_owned();

    // PATCH
    let patch_res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links/{pk}"),
            &json!({ "title": "Updated Title" }),
        )
        .await;
    assert_eq!(
        patch_res.status.as_u16(),
        200,
        "patch link: {}",
        String::from_utf8_lossy(&patch_res.body)
    );
    assert_eq!(
        patch_res.json()["title"].as_str(),
        Some("Updated Title"),
        "título debe actualizarse"
    );

    // DELETE
    let del_res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/links/{pk}"),
        )
        .await;
    assert!(
        del_res.status.as_u16() == 204 || del_res.status.as_u16() == 200,
        "delete link debe retornar 204 o 200, obtuvo {}",
        del_res.status.as_u16()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/issues/{iid}/activities  (legacy)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_activities_legacy_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("iss_act_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/activities"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_activities_legacy_returns_200() {
    let (app, api_key, slug, _, proj_id) = setup("iss_act_ok").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/activities"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "activities legacy: {}",
        String::from_utf8_lossy(&res.body)
    );
}
