//! Tests de integración: External / AI endpoints
//!
//! Cobertura:
//!   - POST /workspaces/{slug}/projects/{project_id}/ai-assistant
//!         → 401 sin auth
//!         → 400/402/503 sin API key LLM configurada
//!   - POST /workspaces/{slug}/rephrase-grammar
//!         → 401 sin auth
//!         → 400/402/503 sin API key LLM configurada

mod common;

use common::TestApp;
use serde_json::json;
use uuid::Uuid;

async fn setup(suffix: &str) -> (TestApp, String, String, Uuid) {
    let app = TestApp::spawn().await;
    let email = format!("ai_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("ai-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    let ws_id = app.workspace_id_by_slug(&ws_slug).await;
    let proj_id = app
        .create_test_project(user_id, ws_id, "AI Project", "AIP")
        .await;
    (app, api_key, ws_slug, proj_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/projects/{project_id}/ai-assistant
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn project_ai_assistant_unauthenticated_returns_401() {
    let (app, _, slug, proj_id) = setup("ai_proj_unauth").await;
    let res = app
        .request(
            axum::http::Method::POST,
            &format!("/workspaces/{slug}/projects/{proj_id}/ai-assistant"),
            Some(serde_json::to_vec(&json!({"prompt": "Summarize"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn project_ai_assistant_without_llm_key_returns_error() {
    // Sin llm_api_key configurada → el servidor debe rechazar la petición
    // antes de intentar llamar al LLM externo.
    let (app, api_key, slug, proj_id) = setup("ai_proj_nokey").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{proj_id}/ai-assistant"),
            &json!({
                "prompt": "Write a description for this issue",
                "task": "ISSUE_DESCRIPTION"
            }),
        )
        .await;
    // Sin API key LLM: 400 (bad request), 402, o 503/500 (servicio no disponible)
    // No debe ser 401/403 (auth correcta) ni 200 (sin LLM key no puede responder)
    let status = res.status.as_u16();
    assert!(
        status == 400 || status == 402 || status == 500 || status == 503,
        "sin LLM key debe retornar error (400/402/500/503), obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn project_ai_assistant_nonexistent_project_returns_404_or_error() {
    let (app, api_key, slug, _) = setup("ai_proj_404").await;
    let fake_proj = Uuid::new_v4();
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/projects/{fake_proj}/ai-assistant"),
            &json!({ "prompt": "test" }),
        )
        .await;
    let status = res.status.as_u16();
    // 403 (no member), 404, o error de LLM (400/500) son todos aceptables
    assert!(
        status == 403 || status == 404 || status == 400 || status == 500 || status == 503,
        "proyecto inexistente: esperado 403/404/400/500, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/rephrase-grammar
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn rephrase_grammar_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("rephrase_unauth").await;
    let res = app
        .request(
            axum::http::Method::POST,
            &format!("/workspaces/{slug}/rephrase-grammar"),
            Some(serde_json::to_vec(&json!({"text": "This are wrong grammar"})).unwrap()),
            &[("content-type", "application/json")],
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn rephrase_grammar_without_llm_key_returns_error() {
    let (app, api_key, slug, _) = setup("rephrase_nokey").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/rephrase-grammar"),
            &json!({ "text": "This sentence have some grammar error in it." }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 400 || status == 402 || status == 500 || status == 503,
        "sin LLM key rephrase debe fallar (400/402/500/503), obtuvo {status}: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rephrase_grammar_empty_text_returns_400() {
    let (app, api_key, slug, _) = setup("rephrase_empty").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/rephrase-grammar"),
            &json!({ "text": "" }),
        )
        .await;
    // Texto vacío → 400 o error de validación
    let status = res.status.as_u16();
    assert!(
        status == 400 || status == 422 || status == 500,
        "texto vacío debe ser 400/422/500, obtuvo {status}"
    );
}
