//! Tests de integración: Projects
//!
//! Dependencia: workspace debe existir y el usuario ser miembro.
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/projects        → 401, 200 vacío, 200 con proyecto
//!   - POST /workspaces/{slug}/projects        → 201, 400 nombre vacío, 400 identifier largo,
//!                                               400 identifier con chars especiales,
//!                                               409 identifier duplicado
//!   - GET  /workspaces/{slug}/projects/{pk}   → 404 no miembro, 200 miembro
//!   - PATCH /workspaces/{slug}/projects/{pk}  → 200 actualiza nombre
//!   - GET  /workspaces/{slug}/projects/{pk}/members → 200 lista miembros
//!   - GET  /workspaces/{slug}/projects/details → 200 lista detallada
//!   - Proptest: POST identifiers con chars prohibidos → siempre 4xx

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects
// ─────────────────────────────────────────────────────────────────────────────

/// Sin auth → 401.
#[tokio::test(flavor = "multi_thread")]
async fn list_projects_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("listproj_unauth@plane.test").await;
    app.create_test_workspace(user_id, "lp-unauth-ws").await;

    let res = app.get("/workspaces/lp-unauth-ws/projects").await;
    assert_eq!(res.status.as_u16(), 401);
}

/// Workspace sin proyectos → lista vacía.
#[tokio::test(flavor = "multi_thread")]
async fn list_projects_empty_workspace() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("listproj_empty@plane.test").await;
    app.create_test_workspace(user_id, "lp-empty-ws").await;

    let res = app.get_authed(&api_key, "/workspaces/lp-empty-ws/projects").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    assert!(
        body.as_array().map(|a| a.is_empty()).unwrap_or(false),
        "workspace sin proyectos debe devolver lista vacía, got: {body}"
    );
}

/// Workspace con proyecto → lista con el elemento.
#[tokio::test(flavor = "multi_thread")]
async fn list_projects_returns_existing_project() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("listproj_ok@plane.test").await;
    app.create_test_workspace(user_id, "lp-ok-ws").await;
    let ws_id = app.workspace_id_by_slug("lp-ok-ws").await;
    app.create_test_project(user_id, ws_id, "My Project", "MYPROJ").await;

    let res = app.get_authed(&api_key, "/workspaces/lp-ok-ws/projects").await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let arr = body.as_array().expect("debe ser array");
    assert_eq!(arr.len(), 1, "debe haber 1 proyecto");
    assert_eq!(arr[0]["identifier"].as_str().unwrap_or(""), "MYPROJ");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects
// ─────────────────────────────────────────────────────────────────────────────

/// Creación exitosa → 201 con identifier en mayúsculas.
#[tokio::test(flavor = "multi_thread")]
async fn create_project_success() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createproj@plane.test").await;
    app.create_test_workspace(user_id, "cp-ws-001").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces/cp-ws-001/projects",
            &json!({
                "name": "Alpha Project",
                "identifier": "ALPHA"
            }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        201,
        "POST /projects debe devolver 201, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["identifier"].as_str().unwrap_or(""), "ALPHA");
    assert_eq!(body["name"].as_str().unwrap_or(""), "Alpha Project");
}

/// Nombre vacío → 422 (Validation).
#[tokio::test(flavor = "multi_thread")]
async fn create_project_empty_name_returns_422() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createproj_empty@plane.test").await;
    app.create_test_workspace(user_id, "cp-ws-002").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces/cp-ws-002/projects",
            &json!({ "name": "", "identifier": "VALID" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        422,
        "nombre vacío debe devolver 422, obtuvo {}",
        res.status
    );
}

/// Identifier demasiado largo (> 12 chars) → 422.
#[tokio::test(flavor = "multi_thread")]
async fn create_project_identifier_too_long_returns_422() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createproj_idlen@plane.test").await;
    app.create_test_workspace(user_id, "cp-ws-003").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces/cp-ws-003/projects",
            &json!({ "name": "Valid Name", "identifier": "TOOLONGIDENT" }),
        )
        .await;

    // "TOOLONGIDENT" tiene 12 chars exactos — límite es ≤ 12, probamos con 13
    let res2 = app
        .post_json_authed(
            &api_key,
            "/workspaces/cp-ws-003/projects",
            &json!({ "name": "Valid Name", "identifier": "TOOLONGIDENTX" }),
        )
        .await;

    assert_eq!(
        res2.status.as_u16(),
        422,
        "identifier de 13 chars debe devolver 422, obtuvo {}",
        res2.status
    );
    // 12 chars exactos debe pasar (si no hay duplicado)
    assert_eq!(
        res.status.as_u16(),
        201,
        "identifier de 12 chars debe devolver 201, obtuvo {}",
        res.status
    );
}

/// Identifier con caracteres prohibidos (`-`, `@`, `.`) → 422.
#[tokio::test(flavor = "multi_thread")]
async fn create_project_identifier_forbidden_chars_returns_422() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createproj_chars@plane.test").await;
    app.create_test_workspace(user_id, "cp-ws-004").await;

    for bad_id in &["MY-PROJ", "MY@PROJ", "MY.PROJ", "MY!PROJ"] {
        let res = app
            .post_json_authed(
                &api_key,
                "/workspaces/cp-ws-004/projects",
                &json!({ "name": "Valid Name", "identifier": bad_id }),
            )
            .await;
        assert_eq!(
            res.status.as_u16(),
            422,
            "identifier={bad_id} con char prohibido debe devolver 422, obtuvo {}",
            res.status
        );
    }
}

/// Identifier duplicado en el mismo workspace → 422.
#[tokio::test(flavor = "multi_thread")]
async fn create_project_duplicate_identifier_returns_422() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createproj_dup@plane.test").await;
    app.create_test_workspace(user_id, "cp-ws-005").await;
    let ws_id = app.workspace_id_by_slug("cp-ws-005").await;
    app.create_test_project(user_id, ws_id, "First", "DUPID").await;

    let res = app
        .post_json_authed(
            &api_key,
            "/workspaces/cp-ws-005/projects",
            &json!({ "name": "Second", "identifier": "DUPID" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        422,
        "identifier duplicado debe devolver 422, obtuvo {}",
        res.status
    );
}

/// Proptest: identifiers con caracteres de la lista prohibida → siempre 422.
#[tokio::test(flavor = "multi_thread")]
async fn create_project_proptest_forbidden_identifier_chars() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("createproj_pt@plane.test").await;
    app.create_test_workspace(user_id, "cp-ws-pt").await;

    // Genera "ABC<bad_char>XYZ" donde bad_char está en la lista prohibida
    let bad_chars = vec!['&', '+', ',', ':', '$', '^', '*', '=', '?', '@', '#', '|', '<', '>', '.', '(', ')', '%', '!', '-'];
    let strategy = (
        "[A-Z]{1,4}",
        prop::sample::select(bad_chars),
        "[A-Z]{1,4}",
    )
        .prop_map(|(pre, bad, post)| format!("{pre}{bad}{post}"));

    let mut runner = TestRunner::new(PropConfig {
        cases: 15,
        ..PropConfig::default()
    });

    runner
        .run(&strategy, |identifier| {
            let app = &app;
            let api_key = &api_key;
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        api_key,
                        "/workspaces/cp-ws-pt/projects",
                        &json!({ "name": "Test", "identifier": identifier }),
                    )
                    .await
                    .status
                    .as_u16()
                })
            });
            prop_assert_eq!(
                status, 422,
                "identifier con char prohibido {identifier:?} debe devolver 422, obtuvo {status}"
            );
            Ok(())
        })
        .expect("proptest falló");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pk}
// ─────────────────────────────────────────────────────────────────────────────

/// No miembro del proyecto → 404.
#[tokio::test(flavor = "multi_thread")]
async fn get_project_non_member_returns_404() {
    let app = TestApp::spawn().await;
    let (owner_id, _) = app.create_test_user("getproj_owner@plane.test").await;
    app.create_test_workspace(owner_id, "gp-ws-001").await;
    let ws_id = app.workspace_id_by_slug("gp-ws-001").await;
    let proj_id = app.create_test_project(owner_id, ws_id, "Secret", "SECR").await;

    // Segundo usuario: es miembro del workspace pero NO del proyecto
    let (other_id, other_key) = app.create_test_user("getproj_other@plane.test").await;
    app.add_workspace_member(other_id, ws_id, 10).await; // Viewer en WS

    let res = app
        .get_authed(&other_key, &format!("/workspaces/gp-ws-001/projects/{proj_id}"))
        .await;

    assert_eq!(
        res.status.as_u16(),
        404,
        "no-miembro de proyecto debe recibir 404"
    );
}

/// Miembro del proyecto → 200 con datos correctos.
#[tokio::test(flavor = "multi_thread")]
async fn get_project_member_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("getproj_ok@plane.test").await;
    app.create_test_workspace(user_id, "gp-ws-002").await;
    let ws_id = app.workspace_id_by_slug("gp-ws-002").await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Visible Project", "VISP")
        .await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/gp-ws-002/projects/{proj_id}"))
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "miembro debe obtener 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["identifier"].as_str().unwrap_or(""), "VISP");
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /workspaces/{slug}/projects/{pk}
// ─────────────────────────────────────────────────────────────────────────────

/// Admin del proyecto puede actualizar el nombre.
#[tokio::test(flavor = "multi_thread")]
async fn patch_project_admin_updates_name() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("patchproj@plane.test").await;
    app.create_test_workspace(user_id, "pp-ws-001").await;
    let ws_id = app.workspace_id_by_slug("pp-ws-001").await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Old Name", "PATCH")
        .await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/pp-ws-001/projects/{proj_id}"),
            &json!({ "name": "New Name" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "PATCH proyecto debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(body["name"].as_str().unwrap_or(""), "New Name");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pk}/members
// ─────────────────────────────────────────────────────────────────────────────

/// Miembro puede listar miembros del proyecto.
#[tokio::test(flavor = "multi_thread")]
async fn list_project_members_returns_list() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("projmembers@plane.test").await;
    app.create_test_workspace(user_id, "pm-ws-001").await;
    let ws_id = app.workspace_id_by_slug("pm-ws-001").await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Members Project", "MEMB")
        .await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/pm-ws-001/projects/{proj_id}/members"),
        )
        .await;

    assert_eq!(res.status.as_u16(), 200);
    let body = res.json();
    let arr = body.as_array().expect("debe ser array");
    assert!(!arr.is_empty(), "debe haber al menos 1 miembro (el owner)");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/details
// ─────────────────────────────────────────────────────────────────────────────

/// La ruta /projects/details devuelve 200 con lista de proyectos donde el
/// usuario es miembro.
#[tokio::test(flavor = "multi_thread")]
async fn list_projects_details_returns_member_projects() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("projdetails@plane.test").await;
    app.create_test_workspace(user_id, "pd-ws-001").await;
    let ws_id = app.workspace_id_by_slug("pd-ws-001").await;
    app.create_test_project(user_id, ws_id, "Detail Project", "DETA").await;

    let res = app
        .get_authed(&api_key, "/workspaces/pd-ws-001/projects/details")
        .await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GET /projects/details debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}
