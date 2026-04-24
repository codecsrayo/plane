//! Tests de integración: Users (endpoints extendidos)
//!
//! Cobertura:
//!   - GET/PATCH /users/me/profile                     → 200
//!   - GET       /users/me/accounts                    → 200 lista
//!   - PATCH     /users/me/onboard                     → 200
//!   - PATCH     /users/me/tour-completed              → 200
//!   - GET       /users/me/activities                  → 200
//!   - GET       /users/me/workspaces                  → 200
//!   - GET       /users/last-visited-workspace         → 200/404
//!   - GET       /users/me/workspaces/{slug}/dashboard → 200
//!   - GET       /users/me/workspaces/{slug}/activity-graph → 200
//!   - GET       /users/me/workspaces/{slug}/issues-completed-graph → 200

mod common;

use common::TestApp;
use serde_json::json;

// ─────────── helpers ─────────────────────────────────────────────────────────

/// Seed del profile para que update_onboard/tour_completed no fallen con 404.
async fn seed_profile(app: &TestApp, user_id: uuid::Uuid) {
    use api_rust::entities::profiles;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
    use chrono::Utc;

    let exists = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user_id))
        .one(&app.state.db)
        .await
        .expect("query profile");

    if exists.is_none() {
        let now = Utc::now();
        profiles::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(user_id),
            is_onboarded: Set(false),
            is_tour_completed: Set(false),
            is_mobile_onboarded: Set(false),
            mobile_timezone_auto_set: Set(false),
            is_smooth_cursor_enabled: Set(false),
            is_app_rail_docked: Set(false),
            start_of_the_week: Set(0),
            billing_address_country: Set("US".into()),
            has_billing_address: Set(false),
            company_name: Set(String::new()),
            language: Set("en".into()),
            background_color: Set(String::new()),
            theme: Set(serde_json::json!({})),
            onboarding_step: Set(serde_json::json!({})),
            goals: Set(serde_json::json!({})),
            mobile_onboarding_step: Set(serde_json::json!({})),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        }
        .insert(&app.state.db)
        .await
        .expect("insert profile");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GET/PATCH /users/me/profile
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_profile_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("getprofile@plane.test").await;
    seed_profile(&app, user_id).await;

    let res = app.get_authed(&api_key, "/users/me/profile").await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_profile_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("updateprofile@plane.test").await;
    seed_profile(&app, user_id).await;

    let res = app
        .patch_json_authed(&api_key, "/users/me/profile", &json!({ "role": "developer" }))
        .await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 204,
        "PATCH /users/me/profile debe devolver 200/204, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/accounts
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_accounts_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("listaccounts@plane.test").await;

    let res = app.get_authed(&api_key, "/users/me/accounts").await;
    assert_eq!(res.status.as_u16(), 200);
    // Usuario de test creado sin OAuth → lista vacía
    assert!(res.json().is_array(), "respuesta debe ser array");
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /users/me/onboard
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_onboard_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("onboard@plane.test").await;
    seed_profile(&app, user_id).await;

    let res = app
        .patch_json_authed(&api_key, "/users/me/onboard", &json!({ "is_onboarded": true }))
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /users/me/tour-completed
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn update_tour_completed_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("tour@plane.test").await;
    seed_profile(&app, user_id).await;

    let res = app
        .patch_json_authed(&api_key, "/users/me/tour-completed", &json!({ "is_tour_completed": true }))
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/activities
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_my_activities_returns_200() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("activities@plane.test").await;

    let res = app.get_authed(&api_key, "/users/me/activities").await;
    assert_eq!(res.status.as_u16(), 200);
    // Sin actividades → lista/paginación vacía
    let body = res.json();
    assert!(body.is_array() || body.is_object(), "debe ser JSON válido");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn list_user_workspaces_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("userws@plane.test").await;
    app.create_test_workspace(user_id, "userws-test-001").await;

    let res = app.get_authed(&api_key, "/users/me/workspaces").await;
    assert_eq!(res.status.as_u16(), 200);
    let arr = res.json().as_array().cloned().unwrap_or_default();
    assert!(!arr.is_empty(), "debe haber al menos 1 workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/last-visited-workspace
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_last_workspace_returns_200_or_404() {
    let app = TestApp::spawn().await;
    let (_, api_key) = app.create_test_user("lastwsvisit@plane.test").await;

    let res = app.get_authed(&api_key, "/users/last-visited-workspace").await;
    assert!(
        res.status.as_u16() == 200 || res.status.as_u16() == 404,
        "last-visited-workspace debe devolver 200 o 404, obtuvo {}", res.status
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces/{slug}/dashboard
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_workspace_dashboard_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("dashboard@plane.test").await;
    app.create_test_workspace(user_id, "dash-ws-001").await;

    let res = app
        .get_authed(&api_key, "/users/me/workspaces/dash-ws-001/dashboard")
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces/{slug}/activity-graph
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_activity_graph_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("actgraph@plane.test").await;
    app.create_test_workspace(user_id, "actgraph-ws-001").await;

    let res = app
        .get_authed(&api_key, "/users/me/workspaces/actgraph-ws-001/activity-graph")
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /users/me/workspaces/{slug}/issues-completed-graph
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn get_issues_completed_graph_returns_200() {
    let app = TestApp::spawn().await;
    let (user_id, api_key) = app.create_test_user("icgraph@plane.test").await;
    app.create_test_workspace(user_id, "icgraph-ws-001").await;

    let res = app
        .get_authed(&api_key, "/users/me/workspaces/icgraph-ws-001/issues-completed-graph")
        .await;
    assert_eq!(res.status.as_u16(), 200, "body: {}", String::from_utf8_lossy(&res.body));
}
