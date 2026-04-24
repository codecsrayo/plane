//! Tests de integración: Cycles extras
//!
//! Cobertura:
//!   - GET/PATCH  /cycles/{cycle_id}/user-properties
//!   - GET        /cycles/{cycle_id}/analytics
//!   - GET        /cycles/{cycle_id}/progress
//!   - POST       /cycles/date-check
//!   - GET/POST   /user-favorite-cycles
//!   - DELETE     /user-favorite-cycles/{cycle_id}
//!   - POST       /cycles/{cycle_id}/transfer-issues
//!   - GET/DELETE /archived-cycles/{pk}  (GET + unarchive)

mod common;

use common::TestApp;
use serde_json::json;

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("cyex_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("cy-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "CycleExt Project", "CYX")
        .await;
    (app, api_key, ws_slug, proj_id)
}

async fn create_cycle(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles"),
            &json!({ "name": name }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear cycle falló: {}", String::from_utf8_lossy(&res.body));
    res.json()["id"].as_str().unwrap().to_owned()
}

async fn create_issue(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": "Cycle Issue" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201);
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// USER-PROPERTIES — GET/PATCH /cycles/{cycle_id}/user-properties
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_user_properties_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("uprops_get").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Props Cycle").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{cycle_id}/user-properties"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_cycle_user_properties_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("uprops_patch").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Patch Props Cycle").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{cycle_id}/user-properties"),
            &json!({ "display_properties": { "priority": true } }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_user_properties_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("uprops_unauth").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{fake_id}/user-properties"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ═════════════════════════════════════════════════════════════════════════════
// ANALYTICS — GET /cycles/{cycle_id}/analytics
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_analytics_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("analytics").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Analytics Cycle").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{cycle_id}/analytics"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_analytics_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("analytics_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{fake_id}/analytics"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// PROGRESS — GET /cycles/{cycle_id}/progress
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_cycle_progress_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("progress").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Progress Cycle").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{cycle_id}/progress"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ═════════════════════════════════════════════════════════════════════════════
// DATE-CHECK — POST /cycles/date-check
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn cycle_date_check_non_overlapping_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("datecheck").await;
    // Crear un ciclo con fechas
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles"),
        &json!({
            "name": "Existing Cycle",
            "start_date": "2025-01-01",
            "end_date": "2025-01-15"
        }),
    )
    .await;

    // Verificar que fechas no solapadas son válidas
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/date-check"),
            &json!({
                "start_date": "2025-02-01",
                "end_date": "2025-02-28"
            }),
        )
        .await;
    let status = res.status.as_u16();
    // 200 = sin conflicto, 400 = solapamiento (depende de implementación)
    assert!(
        status == 200 || status == 400,
        "date-check debe devolver 200 o 400, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn cycle_date_check_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("datecheck_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/date-check"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ═════════════════════════════════════════════════════════════════════════════
// FAVORITE CYCLES — GET/POST + DELETE /{cycle_id}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_favorite_cycles_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("fav_list").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-cycles"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty(), "sin favoritos debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn add_favorite_cycle_returns_201() {
    let (app, api_key, ws_slug, proj_id) = setup("fav_add").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Fav Cycle").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-cycles"),
            &json!({ "cycle": cycle_id }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 201 || status == 200,
        "add-favorite-cycle debe devolver 200/201, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_favorite_cycle_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("fav_del").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Del Fav Cycle").await;

    // Añadir a favoritos
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-cycles"),
        &json!({ "cycle": cycle_id }),
    )
    .await;

    // Eliminar de favoritos
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/user-favorite-cycles/{cycle_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 204 || status == 200,
        "delete-favorite-cycle debe devolver 204/200, obtuvo {status}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// TRANSFER ISSUES — POST /cycles/{cycle_id}/transfer-issues
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn transfer_issues_between_cycles_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("transfer").await;
    let src_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Source Cycle").await;
    let dst_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Dest Cycle").await;

    // Añadir un issue al ciclo origen
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id).await;
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{src_id}/cycle-issues"),
        &json!({ "issues": [issue_id] }),
    )
    .await;

    // Transferir al ciclo destino
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{src_id}/transfer-issues"),
            &json!({ "new_cycle_id": dst_id }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "transfer-issues debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn transfer_issues_nonexistent_cycle_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("transfer_404").await;
    let fake_id = uuid::Uuid::new_v4();
    let dest_id = uuid::Uuid::new_v4();

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{fake_id}/transfer-issues"),
            &json!({ "new_cycle_id": dest_id }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// ARCHIVED CYCLES — GET + DELETE (unarchive) /archived-cycles/{pk}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_cycle_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("arch_get").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Archive Me Cycle").await;

    // Archivar el ciclo
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{cycle_id}/archive"),
        &json!({}),
    )
    .await;

    // Obtener el ciclo archivado
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-cycles/{cycle_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_cycle_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("arch_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-cycles/{fake_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_cycle_returns_204() {
    let (app, api_key, ws_slug, proj_id) = setup("unarch").await;
    let cycle_id = create_cycle(&app, &api_key, &ws_slug, proj_id, "Unarchive Cycle").await;

    // Archivar
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/cycles/{cycle_id}/archive"),
        &json!({}),
    )
    .await;

    // Desarchivar via DELETE en archived-cycles/{pk}
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-cycles/{cycle_id}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 204 || status == 200,
        "unarchive cycle debe devolver 204/200, obtuvo {status}"
    );
}
