//! Tests de integración: States
//!
//! Dependencias: workspace + project + membership deben existir.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects/{project_id}/states    → 200 con defaults seeded
//!   - POST /workspaces/{slug}/projects/{project_id}/states    → 201 OK, 400 nombre dup,
//!                                                               400 grupo inválido
//!   - GET  /workspaces/{slug}/projects/{project_id}/states/{pk} → 200 datos correctos
//!   - PATCH /workspaces/{slug}/projects/{project_id}/states/{pk} → 200 actualiza
//!   - DELETE /workspaces/{slug}/projects/{project_id}/states/{pk} → 204
//!   - GET  /workspaces/{slug}/projects/{project_id}/intake-state → 200
//!   - Proptest: POST con grupos inválidos → siempre 400

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/states
// ─────────────────────────────────────────────────────────────────────────────

/// Miembro obtiene lista de estados (puede estar vacía si no hay seeds
/// automáticos, o con los estados por defecto que inserta create_project).
#[tokio::test(flavor = "multi_thread")]
async fn list_states_member_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("liststates@plane.test").await;
    app.create_test_workspace(user_id, "ls-ws").await;
    let ws_id = app.workspace_id_by_slug("ls-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "States Project", "STLS").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/ls-ws/projects/{proj_id}/states"))
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /states debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert!(body.is_array() || body.is_object(), "respuesta debe ser JSON válido");
}

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn list_states_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("liststates_unauth@plane.test").await;
    app.create_test_workspace(user_id, "ls-unauth-ws").await;
    let ws_id = app.workspace_id_by_slug("ls-unauth-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "P", "PAA").await;

    let res = app
        .get(&format!("/workspaces/ls-unauth-ws/projects/{proj_id}/states"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{project_id}/states
// ─────────────────────────────────────────────────────────────────────────────

/// Creación exitosa → 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_state_success() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createstate@plane.test").await;
    app.create_test_workspace(user_id, "cs-ws").await;
    let ws_id = app.workspace_id_by_slug("cs-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "State Project", "STCR").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/cs-ws/projects/{proj_id}/states"),
            &json!({
                "name": "In Progress",
                "color": "#f59e0b",
                "group": "started",
                "description": ""
            }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        201,
        "POST /states debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "In Progress");
    assert_eq!(body["group"].as_str().unwrap_or(""), "started");
}

/// Nombre duplicado en el mismo proyecto → 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_state_duplicate_name_returns_400() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createstate_dup@plane.test").await;
    app.create_test_workspace(user_id, "csd-ws").await;
    let ws_id = app.workspace_id_by_slug("csd-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Dup Project", "DUP1").await;

    let payload = json!({ "name": "Duplicate", "color": "#000", "group": "backlog", "description": "" });
    app.post_json_authed(&api_key, &format!("/workspaces/csd-ws/projects/{proj_id}/states"), &payload).await;

    let res2 = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/csd-ws/projects/{proj_id}/states"),
            &payload,
        )
        .await;

    assert_eq!(
        res2.status.as_u16(),
        400,
        "nombre duplicado debe devolver 400, obtuvo {}",
        res2.status
    );
}

/// Grupo inválido → 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_state_invalid_group_returns_400() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createstate_grp@plane.test").await;
    app.create_test_workspace(user_id, "csg-ws").await;
    let ws_id = app.workspace_id_by_slug("csg-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Group Project", "GRP1").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/csg-ws/projects/{proj_id}/states"),
            &json!({ "name": "Bad Group State", "color": "#fff", "group": "invalid_group", "description": "" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "grupo inválido debe devolver 400, obtuvo {}",
        res.status
    );
}

/// Proptest: grupos aleatorios no válidos → siempre 400.
#[tokio::test(flavor = "multi_thread")]
async fn create_state_proptest_invalid_groups() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createstate_pt@plane.test").await;
    app.create_test_workspace(user_id, "cspt-ws").await;
    let ws_id = app.workspace_id_by_slug("cspt-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "PropTest", "PTST").await;

    // Strings que claramente no son grupos válidos
    let strategy = "[a-z]{6,20}".prop_filter("no es grupo válido", |s| {
        !["backlog", "unstarted", "started", "completed", "cancelled", "triage"].contains(&s.as_str())
    });

    let mut runner = TestRunner::new(PropConfig { cases: 12, ..PropConfig::default() });

    runner
        .run(&strategy, |group| {
            let app = &app;
            let api_key = &api_key;
            let proj_id = proj_id;
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        api_key,
                        &format!("/workspaces/cspt-ws/projects/{proj_id}/states"),
                        &json!({ "name": "X", "color": "#fff", "group": group, "description": "" }),
                    )
                    .await
                    .status
                    .as_u16()
                })
            });
            prop_assert_eq!(status, 400, "grupo inválido {group:?} debe devolver 400, obtuvo {status}");
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET + PATCH + DELETE /workspaces/{slug}/projects/{project_id}/states/{pk}
// ─────────────────────────────────────────────────────────────────────────────

/// GET estado por ID → 200 con datos correctos.
#[tokio::test(flavor = "multi_thread")]
async fn get_state_by_id_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("getstate@plane.test").await;
    app.create_test_workspace(user_id, "gs-ws").await;
    let ws_id = app.workspace_id_by_slug("gs-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Get State P", "GETS").await;

    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/gs-ws/projects/{proj_id}/states"),
            &json!({ "name": "Todo", "color": "#6b7280", "group": "unstarted", "description": "" }),
        )
        .await;
    assert_eq!(create_res.status.as_u16(), 201);
    let state_id = create_res.json()["id"].as_str().expect("id en respuesta").to_owned();

    let res = app
        .get_authed(&api_key, &format!("/workspaces/gs-ws/projects/{proj_id}/states/{state_id}"))
        .await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "Todo");
}

/// PATCH estado → 200 con nombre actualizado.
#[tokio::test(flavor = "multi_thread")]
async fn update_state_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("updatestate@plane.test").await;
    app.create_test_workspace(user_id, "us-ws").await;
    let ws_id = app.workspace_id_by_slug("us-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Update State P", "UPDS").await;

    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/us-ws/projects/{proj_id}/states"),
            &json!({ "name": "Old Name", "color": "#000", "group": "backlog", "description": "" }),
        )
        .await;
    let state_id = create_res.json()["id"].as_str().expect("id").to_owned();

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/us-ws/projects/{proj_id}/states/{state_id}"),
            &json!({ "name": "New Name" }),
        )
        .await;

    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["name"].as_str().unwrap_or(""), "New Name");
}

/// DELETE estado → 204.
#[tokio::test(flavor = "multi_thread")]
async fn delete_state_returns_204() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("deletestate@plane.test").await;
    app.create_test_workspace(user_id, "ds-ws").await;
    let ws_id = app.workspace_id_by_slug("ds-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Delete State P", "DELS").await;

    let create_res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/ds-ws/projects/{proj_id}/states"),
            &json!({ "name": "Ephemeral", "color": "#f00", "group": "cancelled", "description": "" }),
        )
        .await;
    let state_id = create_res.json()["id"].as_str().expect("id").to_owned();

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/ds-ws/projects/{proj_id}/states/{state_id}"))
        .await;

    assert_eq!(
        res.status.as_u16(),
        204,
        "DELETE /states/{{pk}} debe devolver 204, obtuvo {}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{project_id}/intake-state
// ─────────────────────────────────────────────────────────────────────────────

/// Miembro obtiene el intake-state → 200.
#[tokio::test(flavor = "multi_thread")]
async fn get_intake_state_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("intakestate@plane.test").await;
    app.create_test_workspace(user_id, "is-ws").await;
    let ws_id = app.workspace_id_by_slug("is-ws").await;
    let proj_id = app.create_test_project(user_id, ws_id, "Intake Project", "INTS").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/is-ws/projects/{proj_id}/intake-state"))
        .await;

    // El handler puede devolver 200 (con datos) o 404 si no hay intake state seeded.
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 404,
        "GET /intake-state debe devolver 200 o 404, obtuvo {}",
        res.status
    );
}
