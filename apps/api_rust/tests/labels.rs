//! Tests de integración: Labels
//!
//! Dependencias: workspace + project + membership deben existir.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects/{project_id}/labels      → 200 lista
//!   - POST /workspaces/{slug}/projects/{project_id}/labels      → 201, 400 nombre vacío
//!   - GET  /workspaces/{slug}/projects/{project_id}/labels/{pk} → 200 datos
//!   - PATCH /workspaces/{slug}/projects/{project_id}/labels/{pk} → 200 actualiza
//!   - DELETE /workspaces/{slug}/projects/{project_id}/labels/{pk} → 204
//!   - POST .../bulk-create-labels                               → 201

mod common;

use common::TestApp;
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/labels
// ─────────────────────────────────────────────────────────────────────────────

/// Lista vacía en proyecto sin labels.
#[tokio::test(flavor = "multi_thread")]
async fn list_labels_empty_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("listlabels@plane.test").await;
    app.create_test_workspace(user_id, "ll-ws").await;
    let ws_id = app.workspace_id_by_slug("ll-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Label Project", "LBLP").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/ll-ws/projects/{proj_id}/labels"))
        .await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert!(
        body.as_array().map(|a| a.is_empty()).unwrap_or(false),
        "proyecto sin labels debe devolver lista vacía, got: {body}"
    );
}

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn list_labels_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("listlabels_unauth@plane.test").await;
    app.create_test_workspace(user_id, "ll-unauth-ws").await;
    let ws_id = app.workspace_id_by_slug("ll-unauth-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "P", "LLUN").await;

    let res = app
        .get(&format!("/workspaces/ll-unauth-ws/projects/{proj_id}/labels"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{project_id}/labels
// ─────────────────────────────────────────────────────────────────────────────

/// Creación exitosa → 201 con nombre y color.
#[tokio::test(flavor = "multi_thread")]
async fn create_label_success() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createlabel@plane.test").await;
    app.create_test_workspace(user_id, "cl-ws").await;
    let ws_id = app.workspace_id_by_slug("cl-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Create Label P", "CRLB").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/cl-ws/projects/{proj_id}/labels"),
            &json!({ "name": "Bug", "color": "#ef4444" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        201,
        "POST /labels debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "Bug");
    assert_eq!(body["color"].as_str().unwrap_or(""), "#ef4444");
}

/// Color omitido → 201 con color por defecto.
#[tokio::test(flavor = "multi_thread")]
async fn create_label_default_color() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createlabel_dc@plane.test").await;
    app.create_test_workspace(user_id, "cldc-ws").await;
    let ws_id = app.workspace_id_by_slug("cldc-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "DC Label P", "DCLB").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/cldc-ws/projects/{proj_id}/labels"),
            &json!({ "name": "Feature" }),
        )
        .await;

    assert_eq!(res.status.as_u16(), 201);
    let body = res.json();
    // El handler asigna "#6b7280" como default
    assert_eq!(body["color"].as_str().unwrap_or(""), "#6b7280");
}

/// Nombre vacío (solo espacios) → 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_label_empty_name_returns_400() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createlabel_empty@plane.test").await;
    app.create_test_workspace(user_id, "cle-ws").await;
    let ws_id = app.workspace_id_by_slug("cle-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Empty Label P", "ELLB").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/cle-ws/projects/{proj_id}/labels"),
            &json!({ "name": "   " }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "nombre vacío debe devolver 400, obtuvo {}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/labels/{pk}
// ─────────────────────────────────────────────────────────────────────────────

/// GET label por ID → 200 con datos.
#[tokio::test(flavor = "multi_thread")]
async fn get_label_by_id_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("getlabel@plane.test").await;
    app.create_test_workspace(user_id, "gl-ws").await;
    let ws_id = app.workspace_id_by_slug("gl-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Get Label P", "GETL").await;

    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/gl-ws/projects/{proj_id}/labels"),
            &json!({ "name": "Enhancement" }),
        )
        .await;
    let label_id = create_res.json()["id"].as_str().expect("id").to_owned();

    let res = app
        .get_authed(&api_key, &format!("/workspaces/gl-ws/projects/{proj_id}/labels/{label_id}"))
        .await;

    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "Enhancement");
}

/// ID inexistente → 404.
#[tokio::test(flavor = "multi_thread")]
async fn get_label_not_found_returns_404() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("getlabel_404@plane.test").await;
    app.create_test_workspace(user_id, "gl404-ws").await;
    let ws_id = app.workspace_id_by_slug("gl404-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "P", "GL4").await;

    let fake_id = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/gl404-ws/projects/{proj_id}/labels/{fake_id}"))
        .await;

    assert_eq!(res.status.as_u16(), 404);
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}/projects/{project_id}/labels/{pk}
// ─────────────────────────────────────────────────────────────────────────────

/// PATCH → 200 con nombre actualizado.
#[tokio::test(flavor = "multi_thread")]
async fn update_label_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("updatelabel@plane.test").await;
    app.create_test_workspace(user_id, "ul-ws").await;
    let ws_id = app.workspace_id_by_slug("ul-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Update Label P", "UPDL").await;

    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/ul-ws/projects/{proj_id}/labels"),
            &json!({ "name": "Old Label" }),
        )
        .await;
    let label_id = create_res.json()["id"].as_str().expect("id").to_owned();

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/ul-ws/projects/{proj_id}/labels/{label_id}"),
            &json!({ "name": "New Label", "color": "#3b82f6" }),
        )
        .await;

    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New Label");
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{project_id}/labels/{pk}
// ─────────────────────────────────────────────────────────────────────────────

/// DELETE → 204.
#[tokio::test(flavor = "multi_thread")]
async fn delete_label_returns_204() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("deletelabel@plane.test").await;
    app.create_test_workspace(user_id, "dl-ws").await;
    let ws_id = app.workspace_id_by_slug("dl-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Delete Label P", "DELL").await;

    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/dl-ws/projects/{proj_id}/labels"),
            &json!({ "name": "Disposable" }),
        )
        .await;
    let label_id = create_res.json()["id"].as_str().expect("id").to_owned();

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/dl-ws/projects/{proj_id}/labels/{label_id}"))
        .await;

    assert_eq!(
        res.status.as_u16(),
        204,
        "DELETE /labels/{{pk}} debe devolver 204, obtuvo {}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST .../bulk-create-labels
// ─────────────────────────────────────────────────────────────────────────────

/// Bulk create → 201 con lista de labels creados.
#[tokio::test(flavor = "multi_thread")]
async fn bulk_create_labels_returns_201() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("bulklabel@plane.test").await;
    app.create_test_workspace(user_id, "bl-ws").await;
    let ws_id = app.workspace_id_by_slug("bl-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Bulk Label P", "BULK").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/bl-ws/projects/{proj_id}/bulk-create-labels"),
            // Shape Django (apps/api/plane/app/views/issue/label.py:93):
            // body = { "label_data": [ {name, description?, color?}, ... ] }
            // Antes el test enviaba un array bare → 422 al deserializar.
            &json!({
                "label_data": [
                    { "name": "Label A", "color": "#f00" },
                    { "name": "Label B", "color": "#0f0" },
                    { "name": "Label C", "color": "#00f" }
                ]
            }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        201,
        "bulk-create-labels debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    // Shape de respuesta Django (label.py:114-117):
    //   { "labels": [ <LabelSerializer>, ... ] }
    let body = res.json();
    let arr = body["labels"]
        .as_array()
        .expect("response.labels debe ser array");
    assert_eq!(arr.len(), 3, "debe haber 3 labels creados");
}
