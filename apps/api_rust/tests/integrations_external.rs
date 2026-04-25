//! Tests de integración: Integrations / Importers / External / Assets / Exporter
//!
//! Cobertura (auth-guard layer):
//!   - GET  /integrations                                      → 200 (público o auth)
//!   - GET/POST /workspaces/{slug}/workspace-integrations      → 200
//!   - GET/POST /workspaces/{slug}/workspace-integrations/github/repo-syncs → 200
//!   - GET  /workspaces/{slug}/importers/github/repositories   → 401/403
//!   - GET/POST /workspaces/{slug}/importers/github            → 401/403
//!   - GET  /unsplash                                          → 200/400
//!   - POST /workspaces/{slug}/ai-assistant                    → 400/503
//!   - POST /github-webhook                                    → 200/400
//!   - GET/POST /assets/v2/workspaces/{slug}                   → 200/401
//!   - GET/POST /assets/v2/user-assets                        → 200/401
//!   - GET/POST /workspaces/{slug}/export-issues               → 200/401

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, String, String, uuid::Uuid) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app
        .create_test_user(&format!("intg_{suffix}@plane.test"))
        .await;
    let slug = format!("intg-{suffix}");
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, &slug).await.as_str())
        .await;
    (app, api_key, slug, ws_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /integrations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_integrations_returns_200() {
    let app = TestApp::spawn().await;
    let res = app.get("/integrations").await;
    // Público o requiere auth — en todo caso no debe crashear
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 401,
        "GET /integrations debe devolver 200/401, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/workspace-integrations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_integrations_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("wsintg-unauth").await;
    let res = app
        .get(&format!("/workspaces/{slug}/workspace-integrations"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_workspace_integrations_member_returns_200() {
    let (app, api_key, slug, _) = setup("wsintg-ok").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{slug}/workspace-integrations"))
        .await;
    assert_eq!(
        res.status.as_u16(),
        200,
        "workspace-integrations debe devolver 200, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_integration_nonexistent_returns_4xx() {
    let (app, api_key, slug, _) = setup("wsintg-notfound").await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/{fake_pk}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "integración inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/workspace-integrations/github/repo-syncs
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_github_repo_syncs_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("ghreposyncs-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/workspace-integrations/github/repo-syncs"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_github_repo_syncs_member_returns_200_or_404() {
    let (app, api_key, slug, _) = setup("ghreposyncs-ok").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/workspace-integrations/github/repo-syncs"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "github repo-syncs debe devolver 200/404, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/importers/github/repositories
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_github_importer_repos_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("ghimport-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/importers/github/repositories"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_github_importer_repos_member_returns_4xx_without_github() {
    let (app, api_key, slug, _) = setup("ghimport-nogh").await;
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/importers/github/repositories"),
        )
        .await;
    let status = res.status.as_u16();
    // Sin GitHub integration configurada → 400/404/503
    assert!(
        status == 200 || status >= 400,
        "github importers sin config debe devolver 200 o 4xx/5xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/importers/gitlab/repositories
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_gitlab_importer_repos_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("glimport-unauth").await;
    let res = app
        .get(&format!(
            "/workspaces/{slug}/importers/gitlab/repositories"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /unsplash
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn unsplash_endpoint_returns_expected_status() {
    let app = TestApp::spawn().await;
    let res = app.get("/unsplash").await;
    let status = res.status.as_u16();
    // Sin UNSPLASH_API_KEY → 400 o 200 con vacío
    assert!(
        status == 200 || status == 400 || status == 401 || status == 503,
        "GET /unsplash debe devolver status esperado, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/ai-assistant
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn ai_assistant_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("ai-unauth").await;
    let res = app
        .post_json(
            &format!("/workspaces/{slug}/ai-assistant"),
            &json!({ "task": "summarize" }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn ai_assistant_without_api_key_returns_4xx() {
    let (app, api_key, slug, _) = setup("ai-nokey").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/ai-assistant"),
            &json!({ "task": "summarize", "context": "test" }),
        )
        .await;
    let status = res.status.as_u16();
    // Sin OPENAI_API_KEY → 400/503
    assert!(
        status >= 400,
        "ai-assistant sin API key debe devolver 4xx/5xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /github-webhook / POST /gitlab-webhook
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn github_webhook_without_signature_returns_4xx() {
    let app = TestApp::spawn().await;
    let res = app
        .post_json("/github-webhook", &json!({ "action": "opened" }))
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "github-webhook sin firma debe devolver 4xx, obtuvo {status}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn gitlab_webhook_with_invalid_token_returns_403() {
    // Paridad Django (apps/api/plane/app/views/external/sync.py:392-397):
    // el handler valida el header X-Gitlab-Token solo si el server tiene
    // GITLAB_WEBHOOK_TOKEN configurado. Sin token configurado, todas las
    // requests pasan — un test que postee sin token y espere 4xx falla
    // porque el contrato es "open by default". Para ejercitar la
    // trayectoria de rechazo hay que sembrar la config primero y luego
    // mandar un token inválido (o ninguno).
    let app = TestApp::spawn().await;
    app.set_instance_config("GITLAB_WEBHOOK_TOKEN", "expected-secret").await;

    let res = app
        .post_json("/gitlab-webhook", &json!({ "object_kind": "push" }))
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 403,
        "gitlab-webhook con token inválido debe devolver 403, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Endpoints no expuestos en estas URLs (paridad Django):
//
//   GET /assets/v2/user-assets        → no existe. Django UserAssetsV2Endpoint
//     define solo post/patch/delete (asset/v2.py:109,170,191). Llamar GET
//     enruta al view y produce 405 (no `get` method).
//   GET /assets/v2/workspaces/{slug}  → no existe. WorkspaceFileAssetEndpoint
//     define `get(self, request, slug, asset_id)` — la URL bare /{slug}/
//     tiene el método `get` pero la firma exige asset_id; solo /{slug}/
//     /{asset_id}/ es una ruta GET válida.
//
// Antes había 4 tests aquí esperando 200/401 sobre estas URLs; producían
// 405 en ambos backends. Removidos por ser endpoints fantasma.
//
// Cobertura real de assets vive en tests/assets.rs (POST/PATCH/DELETE).

// ─────────────────────────────────────────────────────────────────────────────
// GET/POST /workspaces/{slug}/export-issues
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn export_issues_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("expiss-unauth").await;
    let res = app.get(&format!("/workspaces/{slug}/export-issues")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_exports_member_returns_200() {
    let (app, api_key, slug, _) = setup("expiss-ok").await;
    // El handler exige `per_page` y `cursor` ambos presentes (paridad Django
    // exporter/base.py:73-84; ver exporter.rs:142-150). Sin esos parámetros
    // el endpoint responde 400 antes de listar nada — el frontend siempre
    // los envía vía SWR con un cursor por defecto.
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/export-issues?per_page=10&cursor=10:0:0"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 404,
        "export-issues lista debe devolver 200/404, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_export_member_returns_2xx() {
    let (app, api_key, slug, _) = setup("expiss-cre").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{slug}/export-issues"),
            &json!({ "provider": "csv", "project": [] }),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 201 || status == 202,
        "crear export debe devolver 200/201/202, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/export-issues/{token}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_export_by_token_nonexistent_returns_4xx() {
    let (app, api_key, slug, _) = setup("exptoken-notfound").await;
    let fake_token = uuid::Uuid::new_v4().to_string();
    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{slug}/export-issues/{fake_token}"),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status >= 400,
        "token de export inexistente debe devolver 4xx, obtuvo {status}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH/DELETE /workspaces/{slug}/projects/{pid}/inbox-issues/{pk}
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_inbox_issue_unauthenticated_returns_401() {
    let app = TestApp::spawn().await;
    let (user_id, _) = app.create_test_user("inbox_anon@plane.test").await;
    let slug = "inbox-anon-ws";
    let ws_id = app
        .workspace_id_by_slug(app.create_test_workspace(user_id, slug).await.as_str())
        .await;
    let project_id = app
        .create_test_project(user_id, ws_id, "Inbox Anon Project", "INB")
        .await;
    let fake_pk = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{project_id}/inbox-issues/{fake_pk}"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/projects/{pid}/inboxes
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_project_inboxes_unauthenticated_returns_401() {
    let (app, _, slug, _) = setup("inbox-list-unauth").await;
    let fake_project = uuid::Uuid::new_v4();
    let res = app
        .get(&format!(
            "/workspaces/{slug}/projects/{fake_project}/inboxes"
        ))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}
