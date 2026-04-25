//! Tests de integración: Notifications
//!
//! Cobertura:
//!   - GET  /workspaces/{slug}/users/notifications        → 401, 200 vacío
//!   - GET  /workspaces/{slug}/users/notifications/unread → 200 contadores
//!   - POST /workspaces/{slug}/users/notifications/mark-all-read → 200
//!   - GET/PATCH/DELETE /notifications/{pk}               → 200, 200, 204
//!   - POST /notifications/{pk}/read + /archive           → 200
//!   - GET/PATCH /users/me/notification-preferences       → 200

mod common;

use common::TestApp;
use serde_json::json;

async fn setup(suffix: &str) -> (TestApp, String, String) {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user(&format!("notif_{suffix}@plane.test")).await;
    let ws_slug = format!("notif-ws-{suffix}");
    app.create_test_workspace(user_id, &ws_slug).await;
    (app, api_key, ws_slug)
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/users/notifications
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_notifications_unauthenticated_returns_401() {
    let (app, _, ws) = setup("list_unauth").await;
    let res = app.get(&format!("/workspaces/{ws}/users/notifications")).await;
    assert_eq!(res.status.as_u16(), 401);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_notifications_empty_returns_200() {
    let (app, api_key, ws) = setup("list_empty").await;
    let res = app.get_authed(&api_key, &format!("/workspaces/{ws}/users/notifications")).await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(arr.is_empty(), "usuario sin notificaciones debe recibir lista vacía");
}

/// Filtros opcionales no rompen el endpoint.
#[tokio::test(flavor = "multi_thread")]
async fn list_notifications_with_filters_returns_200() {
    let (app, api_key, ws) = setup("list_filters").await;

    for query in &[
        "?read=false",
        "?read=true",
        "?archived=true",
        "?snoozed=false",
        "?type=all",
    ] {
        let res = app
            .get_authed(&api_key, &format!("/workspaces/{ws}/users/notifications{query}"))
            .await;
        assert_eq!(
            res.status.as_u16(), 200,
            "GET /notifications{query} debe devolver 200"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /workspaces/{slug}/users/notifications/unread
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn unread_count_returns_200_with_counts() {
    let (app, api_key, ws) = setup("unread").await;
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/users/notifications/unread"))
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
    let body = res.json();
    // Shape Django (apps/api/plane/app/views/notification/base.py:223-229):
    //   { "total_unread_notifications_count": int,
    //     "mention_unread_notifications_count": int }
    // El frontend (web/.../use-notification.ts) consume estos nombres con
    // sufijo _count; el test antes asertaba contra "total_unread_notifications"
    // (sin _count) que no existe en ningún lado.
    assert!(
        body["total_unread_notifications_count"].is_number(),
        "debe tener total_unread_notifications_count numérico, got: {body}"
    );
    assert!(
        body["mention_unread_notifications_count"].is_number(),
        "debe tener mention_unread_notifications_count numérico, got: {body}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /workspaces/{slug}/users/notifications/mark-all-read
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn mark_all_read_returns_200() {
    let (app, api_key, ws) = setup("mark_all").await;
    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/{ws}/users/notifications/mark-all-read"), &json!({}))
        .await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 204,
        "mark-all-read debe devolver 200 o 204, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH/DELETE  /notifications/{pk}  (con notificación seeded en DB)
// ─────────────────────────────────────────────────────────────────────────────

/// Seed directo de una notificación para poder testear GET/PATCH/DELETE.
async fn seed_notification(
    app: &TestApp,
    user_id: uuid::Uuid,
    workspace_id: uuid::Uuid,
) -> uuid::Uuid {
    use api_rust::entities::notifications;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};
    use chrono::Utc;

    let notif_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    let am = notifications::ActiveModel {
        id: Set(notif_id),
        receiver_id: Set(user_id),
        workspace_id: Set(workspace_id),
        title: Set("Test Notification".to_owned()),
        message: Set(None),
        message_html: Set(String::new()),
        message_stripped: Set(None),
        data: Set(None),
        entity_identifier: Set(None),
        entity_name: Set("issue".to_owned()),
        sender: Set(String::new()),
        triggered_by_id: Set(None),
        project_id: Set(None),
        read_at: Set(None),
        archived_at: Set(None),
        snoozed_till: Set(None),
        created_by_id: Set(None),
        updated_by_id: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        deleted_at: Set(None),
    };
    am.insert(&app.state.db).await.expect("seed notification");
    notif_id
}

#[tokio::test(flavor = "multi_thread")]
async fn get_notification_by_id_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("getnotif@plane.test").await;
    app.create_test_workspace(user_id, "getnotif-ws").await;
    let ws_id = app.workspace_id_by_slug("getnotif-ws").await;
    let nid = seed_notification(&app, user_id, ws_id).await;

    let res = app
        .get_authed(&api_key, &format!("/workspaces/getnotif-ws/users/notifications/{nid}"))
        .await;
    assert_eq!(res.status.as_u16(), 200);
    assert_eq!(res.json()["title"].as_str().unwrap_or(""), "Test Notification");
}

#[tokio::test(flavor = "multi_thread")]
async fn get_notification_not_found_returns_404() {
    let (app, api_key, ws) = setup("getnotif_404").await;
    let fake = uuid::Uuid::new_v4();
    let res = app
        .get_authed(&api_key, &format!("/workspaces/{ws}/users/notifications/{fake}"))
        .await;
    assert_eq!(res.status.as_u16(), 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn mark_notification_read_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("readnotif@plane.test").await;
    app.create_test_workspace(user_id, "readnotif-ws").await;
    let ws_id = app.workspace_id_by_slug("readnotif-ws").await;
    let nid = seed_notification(&app, user_id, ws_id).await;

    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/readnotif-ws/users/notifications/{nid}/read"), &json!({}))
        .await;
    assert_eq!(res.status.as_u16(), 200, "mark-read debe devolver 200");
}

#[tokio::test(flavor = "multi_thread")]
async fn archive_notification_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("archnotif@plane.test").await;
    app.create_test_workspace(user_id, "archnotif-ws").await;
    let ws_id = app.workspace_id_by_slug("archnotif-ws").await;
    let nid = seed_notification(&app, user_id, ws_id).await;

    let res = app
        .post_json_authed(&api_key, &format!("/workspaces/archnotif-ws/users/notifications/{nid}/archive"), &json!({}))
        .await;
    assert_eq!(res.status.as_u16(), 200, "archive-notification debe devolver 200");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_notification_returns_204() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("delnotif@plane.test").await;
    app.create_test_workspace(user_id, "delnotif-ws").await;
    let ws_id = app.workspace_id_by_slug("delnotif-ws").await;
    let nid = seed_notification(&app, user_id, ws_id).await;

    let res = app
        .delete_authed(&api_key, &format!("/workspaces/delnotif-ws/users/notifications/{nid}"))
        .await;
    assert_eq!(res.status.as_u16(), 204, "DELETE /notifications/{{pk}} debe devolver 204");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /users/me/notification-preferences
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_notification_preferences_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("notifprefs@plane.test").await;

    let res = app.get_authed(&api_key, "/users/me/notification-preferences").await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_notification_preferences_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("notifprefspatch@plane.test").await;

    let res = app
        .patch_json_authed(&api_key, "/users/me/notification-preferences", &json!({ "email": false }))
        .await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 204,
        "PATCH /notification-preferences debe devolver 200/204, obtuvo {}", res.status
    );
}
