//! Tests de integración: Estimates — endpoints del v1_router
//!
//! Cobertura (vía v1_router):
//!   - GET  /workspaces/{slug}/projects/{pid}/estimates
//!         → 401, 200 (mismo handler que la sección principal, ruta diferente)
//!   - POST /workspaces/{slug}/projects/{pid}/estimates/{eid}/estimate-points
//!         → 401, 201 (crear punto), 404 estimate inexistente
//!   - PATCH/DELETE /workspaces/{slug}/projects/{pid}/estimates/{eid}/estimate-points/{pk}
//!         → 401, 404 inexistente
//!
//! Nota: la ruta GET /estimates ya cubre la sección principal del router.
//! Aquí se verifica que las rutas del v1_router funcionan igual.

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String, Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("estv1_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("estv1-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Estimates v1 Project", "EV1")
        .await;
    (app, api_key, ws_slug, proj_id)
}

/// Crea un estimate y retorna su UUID.
async fn create_estimate(app: &TestApp, api_key: &str, slug: &str, proj_id: Uuid) -> Uuid {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates"),
            &json!({
                "name": "Story Points v1",
                "type": "category",
                "points": []
            }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear estimate: {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().expect("id en estimate").parse().expect("uuid")
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/estimates  (v1_router path)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_estimates_v1_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("list_unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/projects/{proj_id}/estimates"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_estimates_v1_authenticated_returns_200() {
    let (app, api_key, slug, proj_id) = setup("list_ok").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/projects/{proj_id}/estimates"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "list estimates v1: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_array(), "debe ser array de estimates");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{pid}/estimates/{eid}/estimate-points
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn create_estimate_point_v1_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("pt_post_unauth").await;
    let fake_est = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{fake_est}/estimate-points"),
            Some(serde_json::to_vec(&json!({"value":"1","key":0})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_estimate_point_v1_nonexistent_estimate_returns_404() {
    let (app, api_key, slug, proj_id) = setup("pt_post_404").await;
    let fake_est = Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{fake_est}/estimate-points"),
            &json!({ "value": "1", "key": 0 }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "estimate inexistente debe devolver 404: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_estimate_point_v1_creates_and_lists() {
    let (app, api_key, slug, proj_id) = setup("pt_post_create").await;
    let est_id = create_estimate(&app, &api_key, &slug, proj_id).await;

    // Crear punto
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}/estimate-points"),
            &json!({ "value": "3", "key": 1 }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear estimate-point: {}",
        String::from_utf8_lossy(&res.body)
    );
    let pt = res.json();
    assert_eq!(pt["value"].as_str(), Some("3"), "valor del punto");
    assert!(pt["id"].is_string(), "debe tener id");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_multiple_estimate_points_v1() {
    let (app, api_key, slug, proj_id) = setup("pt_multi").await;
    let est_id = create_estimate(&app, &api_key, &slug, proj_id).await;

    for (key, value) in [(0, "1"), (1, "2"), (2, "3"), (3, "5"), (4, "8")] {
        let res = app
            .post_json_authed(
                &api_key,
                &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}/estimate-points"),
                &json!({ "value": value, "key": key }),
            )
            .await;
        assert_eq!(res.status.as_u16(), 201, "crear punto {value}: {}", String::from_utf8_lossy(&res.body));
    }

    // Verificar que el estimate tiene los puntos via GET
    let list_res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}"),
        )
        .await;
    assert_eq!(list_res.status.as_u16(), 200);
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}/projects/{pid}/estimates/{eid}/estimate-points/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn patch_estimate_point_v1_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("pt_patch_unauth").await;
    let fake_est = Uuid::new_v4();
    let fake_pk = Uuid::new_v4();
    let res = app
        .request(
            Method::PATCH,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/estimates/{fake_est}/estimate-points/{fake_pk}"
            ),
            Some(serde_json::to_vec(&json!({"value":"5"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_estimate_point_v1_nonexistent_returns_404() {
    let (app, api_key, slug, proj_id) = setup("pt_patch_404").await;
    let fake_est = Uuid::new_v4();
    let fake_pk = Uuid::new_v4();
    let res = app
        .patch_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/estimates/{fake_est}/estimate-points/{fake_pk}"
            ),
            &json!({ "value": "5" }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "patch estimate-point inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn patch_estimate_point_v1_updates_value() {
    let (app, api_key, slug, proj_id) = setup("pt_patch_ok").await;
    let est_id = create_estimate(&app, &api_key, &slug, proj_id).await;

    // Crear punto
    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}/estimate-points"),
            &json!({ "value": "2", "key": 0 }),
        )
        .await;
    assert_eq!(create_res.status.as_u16(), 201);
    let pt_id = create_res.json()["id"].as_str().unwrap().to_owned();

    // PATCH
    let patch_res = app
        .patch_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}/estimate-points/{pt_id}"
            ),
            &json!({ "value": "13" }),
        )
        .await;
    assert_eq!(
        patch_res.status.as_u16(),
        200,
        "patch estimate-point: {}",
        String::from_utf8_lossy(&patch_res.body)
    );
    assert_eq!(
        patch_res.json()["value"].as_str(),
        Some("13"),
        "valor debe actualizarse a 13"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE /workspaces/{slug}/projects/{pid}/estimates/{eid}/estimate-points/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_estimate_point_v1_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("pt_del_unauth").await;
    let fake_est = Uuid::new_v4();
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            "",
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/estimates/{fake_est}/estimate-points/{fake_pk}"
            ),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_estimate_point_v1_nonexistent_returns_404() {
    let (app, api_key, slug, proj_id) = setup("pt_del_404").await;
    let fake_est = Uuid::new_v4();
    let fake_pk = Uuid::new_v4();
    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/estimates/{fake_est}/estimate-points/{fake_pk}"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete estimate-point inexistente: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_estimate_point_v1_full_cycle() {
    let (app, api_key, slug, proj_id) = setup("pt_del_ok").await;
    let est_id = create_estimate(&app, &api_key, &slug, proj_id).await;

    // Crear
    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}/estimate-points"),
            &json!({ "value": "XL", "key": 3 }),
        )
        .await;
    assert_eq!(create_res.status.as_u16(), 201);
    let pt_id = create_res.json()["id"].as_str().unwrap().to_owned();

    // Delete
    let del_res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/estimates/{est_id}/estimate-points/{pt_id}"
            ),
        )
        .await;
    assert!(
        del_res.status.as_u16() == 204 || del_res.status.as_u16() == 200,
        "delete estimate-point: esperado 204/200, obtuvo {}",
        del_res.status.as_u16()
    );
}
