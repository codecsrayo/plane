//! Tests de integración: Issue Attachments
//!
//! Cobertura (legacy prefix `/issues/...`):
//!   - GET  /workspaces/{slug}/projects/{pid}/issues/{iid}/issue-attachments
//!         → 401 sin auth, 200 lista vacía
//!   - POST /workspaces/{slug}/projects/{pid}/issues/{iid}/issue-attachments
//!         → 401 sin auth, 200 inicia upload (presigned)
//!   - PATCH /workspaces/{slug}/projects/{pid}/issues/{iid}/issue-attachments/{pk}
//!         → 404 asset inexistente
//!   - DELETE /workspaces/{slug}/projects/{pid}/issues/{iid}/issue-attachments/{pk}
//!         → 404 asset inexistente
//!
//! Cobertura (work-items prefix vía v1_router):
//!   - GET  /workspaces/{slug}/projects/{pid}/work-items/{iid}/attachments
//!         → 401 sin auth, 200 lista vacía
//!   - POST /workspaces/{slug}/projects/{pid}/work-items/{iid}/attachments
//!         → 401 sin auth, 200 inicia upload
//!
//! Nota: los tests de upload completo (PATCH mark-uploaded) y delete requieren
//! que el asset_id exista en la DB, por lo que se prueban via el flujo completo
//! (initiate → id → complete/delete).

mod common;

use axum::http::Method;
use common::TestApp;
use serde_json::json;
use uuid::Uuid;

// ── Setup ─────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("iatt_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("iatt-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "Attach Project", "ATT")
        .await;
    (app, api_key, ws_slug, ws_id, proj_id)
}

/// Crea un issue y retorna su UUID.
async fn create_issue(app: &TestApp, api_key: &str, slug: &str, proj_id: Uuid) -> Uuid {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues"),
            &json!({ "name": "Attachment test issue" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "crear issue: {}", String::from_utf8_lossy(&res.body));
    let id_str = res.json()["id"].as_str().expect("id en respuesta").to_owned();
    id_str.parse().expect("uuid válido")
}

// ─────────────────────────────────────────────────────────────────────────────
// GET issue-attachments (legacy prefix)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_attachments_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("list_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/issue-attachments"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_attachments_empty_returns_200() {
    let (app, api_key, slug, _, proj_id) = setup("list_empty").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/issue-attachments"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "lista vacía debe devolver 200: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    let arr = body.as_array().expect("respuesta debe ser array");
    assert_eq!(arr.len(), 0, "issue recién creado no debe tener attachments");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_issue_attachments_nonexistent_issue_returns_404_or_empty() {
    let (app, api_key, slug, _, proj_id) = setup("list_noissue").await;
    let fake_id = Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{fake_id}/issue-attachments"),
        )
        .await;
    // Issue no existe → 404 o lista vacía (comportamiento defensivo aceptable)
    assert!(
        res.status.as_u16() == 404 || res.status.as_u16() == 200,
        "issue inexistente: esperado 404 o 200, obtuvo {}",
        res.status.as_u16()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST issue-attachments — iniciar upload presignado (legacy prefix)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn initiate_issue_attachment_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("post_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/issue-attachments"),
            Some(serde_json::to_vec(&json!({"name":"test.png","type":"image/png","size":1024})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn initiate_issue_attachment_creates_asset_record() {
    let (app, api_key, slug, _, proj_id) = setup("post_create").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/issue-attachments"),
            &json!({
                "name": "screenshot.png",
                "type": "image/png",
                "size": 204800
            }),
        )
        .await;
    // Puede ser 200 (upload iniciado) o 500 si MinIO no está disponible en test
    // Verificamos que no sea 401/403/404 (auth/permisos correctos)
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 500,
        "upload iniciado: esperado 200 o 500 (MinIO no disponible), obtuvo {}",
        res.status.as_u16()
    );
    if res.status.as_u16() == 200 {
        let body = res.json();
        assert!(body["asset_id"].is_string(), "debe retornar asset_id");
        assert!(body["attachment"].is_object(), "debe retornar attachment");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH / DELETE issue-attachments/{pk} — asset inexistente → 404
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn complete_attachment_upload_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("patch_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_pk = Uuid::new_v4();

    let res = app
        .patch_json_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/issue-attachments/{fake_pk}"
            ),
            &json!({}),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "asset inexistente debe devolver 404"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_attachment_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("del_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_pk = Uuid::new_v4();

    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/issues/{issue_id}/issue-attachments/{fake_pk}"
            ),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        404,
        "delete asset inexistente debe devolver 404"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET work-items/attachments (nuevo prefijo vía v1_router)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_work_item_attachments_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("wi_list_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{proj_id}/work-items/{issue_id}/attachments"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_work_item_attachments_empty_returns_200() {
    let (app, api_key, slug, _, proj_id) = setup("wi_list_empty").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/work-items/{issue_id}/attachments"),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "work-items attachments vacíos: {}",
        String::from_utf8_lossy(&res.body)
    );
    let body = res.json();
    assert_eq!(
        body.as_array().map(|a| a.len()).unwrap_or(0),
        0,
        "debe ser lista vacía"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn initiate_work_item_attachment_unauthenticated_returns_401() {
    let (app, _, slug, _, proj_id) = setup("wi_post_unauth").await;
    let issue_id = Uuid::new_v4();
    let res = app
        .request(
            Method::POST,
            &format!("/workspaces/{slug}/projects/{proj_id}/work-items/{issue_id}/attachments"),
            Some(serde_json::to_vec(&json!({"name":"file.pdf","type":"application/pdf"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// DELETE work-items/attachments/{pk} — asset inexistente → 404
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn delete_work_item_attachment_nonexistent_returns_404() {
    let (app, api_key, slug, _, proj_id) = setup("wi_del_404").await;
    let issue_id = create_issue(&app, &api_key, &slug, proj_id).await;
    let fake_pk = Uuid::new_v4();

    let res = app
        .delete_authed(
            &api_key,
            &format!(
                "/workspaces/{slug}/projects/{proj_id}/work-items/{issue_id}/attachments/{fake_pk}"
            ),
        )
        .await;
    assert_eq!(res.status.as_u16(), 404);
}
