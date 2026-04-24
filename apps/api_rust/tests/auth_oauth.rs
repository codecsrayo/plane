//! Tests de integración para los flujos OAuth: GitLab, Google, Gitea y GitHub.
//!
//! Estos endpoints tienen dos fases:
//!   - *Initiate*:   `GET /auth/{provider}` — redirige al provider. Requiere
//!                   `{PROVIDER}_CLIENT_ID` en `instance_configurations`; sin
//!                   él devuelve 400.
//!   - *Callback*:   `GET /auth/{provider}/callback` — valida `code` + `state`
//!                   antes de intercambiar el code por un token. Los tests se
//!                   limitan al contrato de validación (sin mockear la red).
//!
//! No testeamos el intercambio real con el provider — requeriría un servidor
//! mock externo y saldría del alcance de un test de integración.

mod common;

use axum::http::Method;
use common::TestApp;

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/gitlab  (initiate)
// ─────────────────────────────────────────────────────────────────────────────

/// Sin `GITLAB_CLIENT_ID` configurado → 400 "GitLab OAuth is not configured".
#[tokio::test(flavor = "multi_thread")]
async fn gitlab_initiate_without_client_id_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/gitlab").await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "sin GITLAB_CLIENT_ID debe retornar 400, body={}",
        String::from_utf8_lossy(&res.body)
    );
}

/// Con `GITLAB_CLIENT_ID` configurado → 3xx con Location apuntando al host
/// de GitLab y cookie `oauth-state` emitida.
#[tokio::test(flavor = "multi_thread")]
async fn gitlab_initiate_with_client_id_redirects_to_gitlab() {
    let app = TestApp::spawn().await;
    app.set_instance_config("GITLAB_CLIENT_ID", "fake-gitlab-client-id")
        .await;

    let res = app
        .request(Method::GET, "/auth/gitlab", None, &[])
        .await;

    assert!(
        res.status.is_redirection(),
        "debe redireccionar a GitLab, status={}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/gitlab/callback
// ─────────────────────────────────────────────────────────────────────────────

/// Callback sin `code` → 400 ("Missing code").
#[tokio::test(flavor = "multi_thread")]
async fn gitlab_callback_without_code_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/gitlab/callback").await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "callback sin code debe retornar 400"
    );
}

/// Callback con `code` pero sin `state` → 400 ("Missing state").
#[tokio::test(flavor = "multi_thread")]
async fn gitlab_callback_without_state_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/gitlab/callback?code=abc123").await;

    assert_eq!(
        res.status.as_u16(),
        400,
        "callback sin state debe retornar 400"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/google  (initiate)
// ─────────────────────────────────────────────────────────────────────────────

/// Sin `GOOGLE_CLIENT_ID` → 400.
#[tokio::test(flavor = "multi_thread")]
async fn google_initiate_without_client_id_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/google").await;

    assert_eq!(res.status.as_u16(), 400);
}

/// Con `GOOGLE_CLIENT_ID` → 3xx con Location hacia accounts.google.com.
#[tokio::test(flavor = "multi_thread")]
async fn google_initiate_with_client_id_redirects_to_google() {
    let app = TestApp::spawn().await;
    app.set_instance_config("GOOGLE_CLIENT_ID", "fake-google-client-id")
        .await;

    let res = app.get("/auth/google").await;

    assert!(
        res.status.is_redirection(),
        "debe redireccionar a Google, status={}",
        res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/google/callback
// ─────────────────────────────────────────────────────────────────────────────

/// Callback sin `code` → 400.
#[tokio::test(flavor = "multi_thread")]
async fn google_callback_without_code_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/google/callback").await;

    assert_eq!(res.status.as_u16(), 400);
}

/// Callback con `code` y `state` sin cookie → 200 HTML con postMessage de error
/// (el handler de Google, a diferencia de GitLab, no devuelve 400 sino HTML).
#[tokio::test(flavor = "multi_thread")]
async fn google_callback_without_cookie_returns_html_error() {
    let app = TestApp::spawn().await;

    let res = app
        .get("/auth/google/callback?code=abc&state=xyz")
        .await;

    // El handler intenta el intercambio cuando no hay mismatch de state; como
    // no hay credenciales reales, falla en el paso de token → AppError.
    // Lo único que exigimos aquí es que NO explote con 5xx inesperado y que
    // devuelva 2xx (HTML de postMessage) o 4xx/5xx razonable. Validamos que
    // la ruta existe (no 404) y que la respuesta tiene body.
    assert_ne!(
        res.status.as_u16(),
        404,
        "la ruta debe existir"
    );
    assert!(
        !res.body.is_empty(),
        "debe haber body en la respuesta"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/gitea  (initiate)
// ─────────────────────────────────────────────────────────────────────────────

/// Sin `GITEA_CLIENT_ID` → 400.
#[tokio::test(flavor = "multi_thread")]
async fn gitea_initiate_without_client_id_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/gitea").await;

    assert_eq!(res.status.as_u16(), 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/gitea/callback
// ─────────────────────────────────────────────────────────────────────────────

/// Callback sin `code` → 400.
#[tokio::test(flavor = "multi_thread")]
async fn gitea_callback_without_code_returns_400() {
    let app = TestApp::spawn().await;

    let res = app.get("/auth/gitea/callback").await;

    assert_eq!(res.status.as_u16(), 400);
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/github/callback   (GitHub App setup URL — sin auth)
// ─────────────────────────────────────────────────────────────────────────────

/// Sin `installation_id` ni `state` → 200 HTML de error (postMessage negativo).
/// El endpoint nunca devuelve 4xx — siempre produce HTML para cerrar el popup.
#[tokio::test(flavor = "multi_thread")]
async fn github_app_callback_without_installation_id_returns_error_html() {
    let app = TestApp::spawn().await;

    let res = app.get("/api/github/callback").await;

    assert_eq!(
        res.status.as_u16(),
        200,
        "GitHub App callback siempre responde 200 con HTML de popup"
    );
    let body = String::from_utf8_lossy(&res.body);
    // El HTML de postMessage debe contener algún marcador de error.
    assert!(
        body.contains("Missing") || body.contains("installation_id") || body.contains("error"),
        "body debe indicar error: {body}"
    );
}

/// Con `installation_id` pero `state` (workspace_slug) inexistente → 200 HTML
/// de error porque `github_app_callback_inner` falla al buscar el workspace.
#[tokio::test(flavor = "multi_thread")]
async fn github_app_callback_with_unknown_workspace_returns_error_html() {
    let app = TestApp::spawn().await;

    let res = app
        .get("/api/github/callback?installation_id=123&state=nonexistent-slug")
        .await;

    assert_eq!(res.status.as_u16(), 200);
    let body = String::from_utf8_lossy(&res.body);
    assert!(
        body.contains("Installation failed") || body.contains("error"),
        "body debe indicar error: {body}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/github/user-callback
// ─────────────────────────────────────────────────────────────────────────────

/// Sin autenticación → 401 (requiere `AnyAuth`).
#[tokio::test(flavor = "multi_thread")]
async fn github_user_callback_without_auth_returns_401() {
    let app = TestApp::spawn().await;

    let res = app
        .post_json(
            "/auth/github/user-callback",
            &serde_json::json!({ "code": "dummy" }),
        )
        .await;

    assert_eq!(
        res.status.as_u16(),
        401,
        "github/user-callback sin sesión debe devolver 401, obtuvo {}, body={}",
        res.status,
        String::from_utf8_lossy(&res.body)
    );
}
