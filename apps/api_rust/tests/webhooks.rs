//! Tests de integración: Webhooks
//!
//! Cobertura:
//!   - GET/POST         /workspaces/{slug}/webhooks
//!   - GET/PATCH/DELETE /workspaces/{slug}/webhooks/{pk}
//!   - POST             /workspaces/{slug}/webhooks/{pk}/regenerate
//!   - GET              /workspaces/{slug}/webhook-logs/{webhook_id}
//!
//! Proptest: urls arbitrarias válidas en POST → siempre 201.

mod common;

use common::TestApp;
use proptest::prelude::*;
use proptest::test_runner::{Config as PropConfig, TestRunner};
use serde_json::json;

// ── Setup ─────────────────────────────────────────────────────────────────────

async fn setup(suffix: &str) -> (TestApp, String, String) {
    let app = TestApp::spawn().await;
    let email = format!("wh_{suffix}@plane.test");
    let (user_id, api_key) = app.create_test_user(&email).await;
    let ws_slug = format!("wh-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, api_key, ws_slug)
}

async fn create_webhook(app: &TestApp, api_key: &str, ws_slug: &str, url: &str) -> String {
    let res = app
        .post_json_authed(
            api_key,
            &format!("/workspaces/{ws_slug}/webhooks"),
            &json!({
                "url": url,
                "is_active": true,
                "issue": true
            }),
        )
        .await;
    assert_eq!(
        res.status.as_u16(),
        201,
        "crear webhook falló (url={url}): {}",
        String::from_utf8_lossy(&res.body)
    );
    res.json()["id"].as_str().unwrap().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════
// LIST / CREATE
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_webhooks_unauthenticated_returns_401() {
    let (app, _, ws_slug) = setup("list_unauth").await;
    let res = app.get(&format!("/workspaces/{ws_slug}/webhooks")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_webhooks_empty_returns_200() {
    let (app, api_key, ws_slug) = setup("list_empty").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let count = res.json().as_array().map(|a| a.len()).unwrap_or(0);
    assert_eq!(count, 0, "nuevo workspace no debe tener webhooks");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_webhook_returns_201() {
    let (app, api_key, ws_slug) = setup("create").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/webhooks"),
            &json!({
                "url": "https://hooks.example.com/plane",
                "is_active": true,
                "issue": true,
                "cycle": false,
                "module": false,
                "project": false
            }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 201, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    assert_eq!(body["url"].as_str().unwrap_or(""), "https://hooks.example.com/plane");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_webhook_invalid_url_returns_400() {
    let (app, api_key, ws_slug) = setup("create_bad").await;
    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/webhooks"),
            &json!({ "url": "not-a-url", "is_active": true }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_webhooks_returns_created_webhook() {
    let (app, api_key, ws_slug) = setup("list_ok").await;
    create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/a").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 webhook");
}

/// Proptest: URLs https arbitrarias → siempre 201.
#[tokio::test(flavor = "multi_thread")]
async fn create_webhook_proptest_valid_urls() {
    let (app, api_key, ws_slug) = setup("pt").await;

    // Estrategia: path con caracteres alfanuméricos y guión
    let strategy = "[a-z]{3,10}".prop_map(|s| format!("https://hooks-{s}.example.com/webhook"));
    let mut runner = TestRunner::new(PropConfig { cases: 8, ..PropConfig::default() });

    runner
        .run(&strategy, |url| {
            let rt = tokio::runtime::Handle::current();
            let status = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    app.post_json_authed(
                        &api_key,
                        &format!("/workspaces/{ws_slug}/webhooks"),
                        &json!({ "url": url, "is_active": true, "issue": true }),
                    )
                    .await
                    .status
                    .as_u16()
                })
            });
            prop_assert_eq!(status, 201, "url {url:?} debe devolver 201, obtuvo {status}");
            Ok(())
        })
        .expect("proptest webhooks falló");
}

// ═════════════════════════════════════════════════════════════════════════════
// GET / PATCH / DELETE
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn get_webhook_returns_200() {
    let (app, api_key, ws_slug) = setup("get").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/b").await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks/{wh_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["url"].as_str().unwrap_or(""), "https://hooks.example.com/b");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_webhook_not_found_returns_404() {
    let (app, api_key, ws_slug) = setup("get_404").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks/{fake_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_webhook_returns_200() {
    let (app, api_key, ws_slug) = setup("patch").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/c").await;

    let res = app
        .patch_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/webhooks/{wh_id}"),
            &json!({ "url": "https://hooks.example.com/c-updated", "is_active": false }),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    assert_eq!(body["url"].as_str().unwrap_or(""), "https://hooks.example.com/c-updated");
    assert_eq!(body["is_active"].as_bool(), Some(false));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_webhook_returns_204() {
    let (app, api_key, ws_slug) = setup("del").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/d").await;

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks/{wh_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 204);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_deleted_webhook_returns_404() {
    let (app, api_key, ws_slug) = setup("del_then_get").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/e").await;

    app.delete_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks/{wh_id}"))
        .await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws_slug}/webhooks/{wh_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

// ═════════════════════════════════════════════════════════════════════════════
// POST /{pk}/regenerate
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn regenerate_webhook_secret_returns_200() {
    let (app, api_key, ws_slug) = setup("regen").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/f").await;

    let res = app
        .post_json_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/webhooks/{wh_id}/regenerate"),
            &json!({}),
        )
        .await;
    let status = res.status.as_u16();
    assert!(
        status == 200 || status == 204,
        "regenerate debe devolver 200/204, obtuvo {status}, body: {}",
        String::from_utf8_lossy(&res.body)
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// GET /webhook-logs/{webhook_id}
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread")]
async fn list_webhook_logs_returns_200() {
    let (app, api_key, ws_slug) = setup("logs").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/g").await;

    let res = app
        .get_authed(
            &api_key,
            &format!("/workspaces/{ws_slug}/webhook-logs/{wh_id}"),
        )
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    // Sin entregas aún → lista vacía
    let body = res.json();
    let count = if let Some(arr) = body.as_array() {
        arr.len()
    } else {
        body["results"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    assert_eq!(count, 0, "webhook sin actividad no debe tener logs");
}

#[tokio::test(flavor = "multi_thread")]
async fn list_webhook_logs_unauthenticated_returns_401() {
    let (app, api_key, ws_slug) = setup("logs_unauth").await;
    let wh_id = create_webhook(&app, &api_key, &ws_slug, "https://hooks.example.com/h").await;

    let res = app
        .get(&format!("/workspaces/{ws_slug}/webhook-logs/{wh_id}"))
        .await;
    assert_eq!(res.status.as_u16(), 401);
}
