//! Tests de integración: Issues extras2
//!
//! Cobertura:
//!   - GET/POST/DELETE /issues/{pk}/archive   (archivar/desarchivar/ver archivado)
//!   - GET             /archived-issues
//!   - GET             /deleted-issues
//!   - GET             /issues/{id}/meta
//!   - GET             /issues/{id}/activities
//!   - GET/PATCH       /projects/{project_id}/user-properties  (ya cubierto en project_extras)
//!   - POST            /bulk-archive-issues (ya cubierto en issue_extras)
//!   - GET             /issue-dates + POST bulk-update-dates

mod common;

use common::TestApp;
use serde_json::json;

// ── Setup ────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("ie2_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("ie2-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "IE2 Project", "IE2")
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

/// Crea un state con `group="completed"` en el proyecto y devuelve su id.
///
/// Helper para los tests de archive: el handler `archive_issue` exige
/// paridad Django (`apps/api/plane/app/views/issue/archive.py:259`):
/// solo issues cuyo state.group sea "completed" o "cancelled" se pueden
/// archivar. `create_test_project` no siembra states por defecto, así
/// que el helper crea uno on-demand.
///
/// Se añade un sufijo aleatorio al nombre para evitar colisión de
/// `unique(project_id, name)` cuando varios tests del mismo módulo
/// piden states sobre el mismo proyecto (no es el caso actual, pero
/// previene bugs futuros si alguien refactoriza setup).
async fn create_completed_state(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
) -> String {
    let unique_name = format!("Done {}", uuid::Uuid::new_v4());
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/states"),
            &json!({
                "name": unique_name,
                "group": "completed",
                "color": "#10b981"
            }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "create state debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().unwrap().to_owned()
}

/// Crea un issue YA en estado completed — listo para archive.
///
/// Composición: crea un state group="completed", crea el issue con
/// `state_id` apuntando a ese state. El handler `create_issue` acepta
/// `state_id` opcional; si no se pasa, queda NULL y el archive falla
/// con 400 ("Can only archive completed or cancelled state group issue").
async fn create_completed_issue(
    app: &TestApp,
    api_key: &str,
    ws_slug: &str,
    proj_id: uuid::Uuid,
    name: &str,
) -> String {
    let state_id = create_completed_state(app, api_key, ws_slug, proj_id).await;
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues"),
            &json!({ "name": name, "state_id": state_id }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "create completed issue debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// ARCHIVE / UNARCHIVE — POST/DELETE /issues/{pk}/archive
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn archive_issue_returns_200_or_204() {
    let (app, api_key, ws_slug, proj_id) = setup("arch").await;
    let issue_id = create_completed_issue(&app, &api_key, &ws_slug, proj_id, "Archive Me").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/archive"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "archive issue debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_archived_issue_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("get_arch").await;
    let issue_id = create_completed_issue(&app, &api_key, &ws_slug, proj_id, "Archived Issue").await;

    // Archivar
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/archive"),
        &json!({}),
    )
    .await;

    // GET del issue archivado
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/archive"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn unarchive_issue_returns_200_or_204() {
    let (app, api_key, ws_slug, proj_id) = setup("unarch").await;
    let issue_id = create_completed_issue(&app, &api_key, &ws_slug, proj_id, "Unarchive Me").await;

    // Archivar primero
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/archive"),
        &json!({}),
    )
    .await;

    // Desarchivar
    let res = app
        .delete_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/archive"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "unarchive issue debe devolver 200/204, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn archive_nonexistent_issue_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("arch_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{fake_id}/archive"),
            &json!({}),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /archived-issues
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_archived_issues_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("arclist_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-issues"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_archived_issues_empty_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("arclist_empty").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-issues"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let count = if let Some(arr) = body.as_array() {
        arr.len()
    } else {
        body["results"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    assert_eq!(count, 0, "sin issues archivados debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_archived_issues_shows_archived_issue() {
    let (app, api_key, ws_slug, proj_id) = setup("arclist_ok").await;
    let issue_id = create_completed_issue(&app, &api_key, &ws_slug, proj_id, "Listed Archived").await;

    // Archivar
    app.post_json_authed(
        &api_key,
        &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/archive"),
        &json!({}),
    )
    .await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/archived-issues"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let count = if let Some(arr) = body.as_array() {
        arr.len()
    } else {
        body["results"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    assert!(count > 0, "debe haber al menos 1 issue archivado");
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /deleted-issues
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_deleted_issues_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("del_list").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/deleted-issues"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    let count = body.as_array().map(|a| a.len()).unwrap_or(0);
    assert_eq!(count, 0, "sin issues borrados debe devolver lista vacía");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_deleted_issues_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("del_unauth").await;
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/deleted-issues"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /issues/{id}/meta
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_issue_meta_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("meta").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Meta Issue").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/meta"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_issue_meta_not_found_returns_404() {
    let (app, api_key, ws_slug, proj_id) = setup("meta_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{fake_id}/meta"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /issues/{id}/activities
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_activities_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("act").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Activity Issue").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{issue_id}/activities"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    // Puede contener la actividad de creación o estar vacío
    assert!(res.json().is_array() || res.json().is_object());
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_activities_unauthenticated_returns_401() {
    let (app, _, ws_slug, proj_id) = setup("act_unauth").await;
    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get(&format!("/workspaces/{ws_slug}/projects/{proj_id}/issues/{fake_id}/activities"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ═════════════════════════════════════════════════════════════════════════════
// POST /issue-dates (bulk update dates)
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn bulk_update_issue_dates_returns_200() {
    let (app, api_key, ws_slug, proj_id) = setup("dates").await;
    let issue_id = create_issue(&app, &api_key, &ws_slug, proj_id, "Date Issue").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issue-dates"),
            // Shape Django (apps/api/plane/app/views/issue/base.py:1115):
            // body = { "updates": [ { "id", "start_date", "target_date" } ] }.
            // El campo es "id", no "issue_id" — paridad con
            // `update["id"]` en el handler Python.
            &json!({
                "updates": [{
                    "id": issue_id,
                    "start_date": "2025-01-01",
                    "target_date": "2025-12-31"
                }]
            }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "bulk-update-dates debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bulk_update_issue_dates_empty_list_returns_400_or_200() {
    let (app, api_key, ws_slug, proj_id) = setup("dates_empty").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/projects/{proj_id}/issue-dates"),
            // Shape Django: { "updates": [...] }. Lista vacía dentro del
            // wrapper para no fallar al deserializar.
            &json!({ "updates": [] }),
        )
        .await;
    let status = res.status.as_u16();
    // Lista vacía: puede ser 400 (validación) o 200 (operación vacía exitosa)
    assert!(
        status == 400 || status == 200 || status == 204,
        "lista vacía debe devolver 400 o 200/204, obtuvo {status}"
    );
}
