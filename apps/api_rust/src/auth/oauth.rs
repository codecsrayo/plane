// src/auth/oauth.rs
//! Authentication via OAuth 2.0: GitLab, Google, Gitea.

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{Redirect, IntoResponse},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter,
    TransactionTrait,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::session::{issue_session_cookie, SessionSurface},
    entities::{profiles, user_notification_preferences, users},
    error::AppError,
    utils::{
        django_sessions::email_display_name,
        instance_config::get_instance_config,
        oauth_popup::{postmessage_html, OAuthMessageType},
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

const OAUTH_STATE_COOKIE: &str = "oauth-state";

// ── User Helpers ───────────────────────────────────────────────────────

async fn find_or_create_oauth_user(
    state: &AppState,
    headers: &HeaderMap,
    email: String,
    first_name: String,
    last_name: String,
    avatar_url: String,
    provider: &str,
) -> Result<users::Model, AppError> {
    let email = email.trim().to_lowercase();

    let existing_user = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if let Some(user) = existing_user {
        return update_oauth_login_metadata(state, user, headers, provider).await;
    }

    // Create new user (Simplified SignUp for OAuth)
    let now = Utc::now();
    let user_id = uuid::Uuid::new_v4();
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let display_name = email_display_name(&email).into_owned();

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    users::ActiveModel {
        id: Set(user_id),
        email: Set(Some(email)),
        username: Set(uuid::Uuid::new_v4().simple().to_string()),
        first_name: Set(first_name),
        last_name: Set(last_name),
        avatar: Set(avatar_url),
        password: Set("!OAUTH_USER!".to_owned()), // Marker for users without local password
        date_joined: Set(now.into()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        is_active: Set(true),
        is_superuser: Set(false),
        is_staff: Set(false),
        is_managed: Set(true),
        is_password_autoset: Set(true),
        token: Set(token),
        user_timezone: Set("UTC".to_owned()),
        last_active: Set(Some(now.into())),
        last_login: Set(Some(now.into())),
        last_login_time: Set(Some(now.into())),
        last_login_ip: Set(crate::auth::session::extract_client_ip(headers)),
        last_login_medium: Set(provider.to_owned()),
        last_login_uagent: Set(headers.get(axum::http::header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or_default().to_owned()),
        display_name: Set(display_name),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    profiles::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        onboarding_step: Set(serde_json::json!({"profile_complete": false, "workspace_create": false, "workspace_invite": false, "workspace_join": false})),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    user_notification_preferences::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("User creation failed")))
}

async fn update_oauth_login_metadata(
    state: &AppState,
    user: users::Model,
    headers: &HeaderMap,
    provider: &str,
) -> Result<users::Model, AppError> {
    let now = Utc::now();
    let mut am: users::ActiveModel = user.into();
    am.last_login = Set(Some(now.into()));
    am.last_active = Set(Some(now.into()));
    am.last_login_time = Set(Some(now.into()));
    am.last_login_ip = Set(crate::auth::session::extract_client_ip(headers));
    am.last_login_medium = Set(provider.to_owned());
    am.updated_at = Set(now.into());
    am.update(&state.db).await.map_err(AppError::Database)
}

// ── GitLab ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/gitlab/",
    tag = "Auth",
    responses(
        (status = 302, description = "Redirect to GitLab"),
    ),
)]
pub async fn gitlab_initiate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let client_id = get_instance_config(&state, "GITLAB_CLIENT_ID")
        .await?
        .ok_or_else(|| AppError::BadRequest("GitLab OAuth is not configured".into()))?;

    let gitlab_host = get_instance_config(&state, "GITLAB_HOST")
        .await?
        .unwrap_or_else(|| "https://gitlab.com".to_owned());

    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/gitlab/callback/", base_url.trim_end_matches('/'));

    let oauth_state = Uuid::new_v4().simple().to_string();

    let params = [
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", "read_user api"),
        ("state", oauth_state.as_str()),
    ];

    let query = serde_urlencoded::to_string(params).unwrap();
    let url = format!("{}/oauth/authorize?{}", gitlab_host.trim_end_matches('/'), query);

    let cookie = Cookie::build((OAUTH_STATE_COOKIE, oauth_state))
        .path("/")
        .http_only(true)
        .secure(state.config.is_production)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(10));

    Ok((jar.add(cookie), Redirect::to(&url)))
}

#[utoipa::path(
    get,
    path = "/auth/gitlab/callback/",
    tag = "Auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state"),
    ),
    responses(
        (status = 200, description = "Popup closure HTML"),
    ),
)]
pub async fn gitlab_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, AppError> {
    let code = params.code.ok_or_else(|| AppError::BadRequest("Missing code".into()))?;
    let state_val = params.state.ok_or_else(|| AppError::BadRequest("Missing state".into()))?;

    let saved_state = jar.get(OAUTH_STATE_COOKIE).map(|c| c.value().to_owned());
    if saved_state.as_deref() != Some(&state_val) {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GitlabIntegration, Some("Invalid OAuth state (CSRF detected)"), None)).into_response());
    }

    let client_id = get_instance_config(&state, "GITLAB_CLIENT_ID").await?.unwrap_or_default();
    let client_secret = get_instance_config(&state, "GITLAB_CLIENT_SECRET").await?.unwrap_or_default();
    let gitlab_host = get_instance_config(&state, "GITLAB_HOST").await?.unwrap_or_else(|| "https://gitlab.com".to_owned());
    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/gitlab/callback/", base_url.trim_end_matches('/'));

    // Exchange code for token
    let token_resp = state.http.post(format!("{}/oauth/token", gitlab_host.trim_end_matches('/')))
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send().await.map_err(|e| AppError::Internal(anyhow::anyhow!("GitLab token exchange failed: {e}")))?;

    if !token_resp.status().is_success() {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GitlabIntegration, Some("Failed to exchange GitLab code"), None)).into_response());
    }

    let token_data: serde_json::Value = token_resp.json().await.unwrap_or_default();
    let access_token = token_data["access_token"].as_str().unwrap_or_default();

    // Fetch user info
    let user_resp = state.http.get(format!("{}/api/v4/user", gitlab_host.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {access_token}"))
        .send().await.map_err(|e| AppError::Internal(anyhow::anyhow!("GitLab user fetch failed: {e}")))?;

    if !user_resp.status().is_success() {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GitlabIntegration, Some("Failed to fetch GitLab user info"), None)).into_response());
    }

    let gitlab_user: serde_json::Value = user_resp.json().await.unwrap_or_default();
    let email = gitlab_user["email"].as_str().ok_or_else(|| AppError::Internal(anyhow::anyhow!("GitLab user has no email")))?;
    let name = gitlab_user["name"].as_str().unwrap_or("");
    let avatar = gitlab_user["avatar_url"].as_str().unwrap_or("");

    let user = find_or_create_oauth_user(&state, &headers, email.to_owned(), name.to_owned(), String::new(), avatar.to_owned(), "gitlab").await?;

    let jar = issue_session_cookie(&state, &headers, jar.remove(OAUTH_STATE_COOKIE), &user, SessionSurface::App).await?;

    Ok((jar, postmessage_html(true, OAuthMessageType::GitlabIntegration, None, None)).into_response())
}

// ── Google ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/google/",
    tag = "Auth",
    responses(
        (status = 302, description = "Redirect to Google"),
    ),
)]
pub async fn google_initiate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let client_id = get_instance_config(&state, "GOOGLE_CLIENT_ID")
        .await?
        .ok_or_else(|| AppError::BadRequest("Google OAuth is not configured".into()))?;

    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/google/callback/", base_url.trim_end_matches('/'));

    let oauth_state = Uuid::new_v4().simple().to_string();
    let params = [
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", "openid email profile"),
        ("access_type", "offline"),
        ("state", oauth_state.as_str()),
    ];

    let query = serde_urlencoded::to_string(params).unwrap();
    let url = format!("https://accounts.google.com/o/oauth2/v2/auth?{query}");

    let cookie = Cookie::build((OAUTH_STATE_COOKIE, oauth_state))
        .path("/")
        .http_only(true)
        .secure(state.config.is_production)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(10));

    Ok((jar.add(cookie), Redirect::to(&url)))
}

#[utoipa::path(
    get,
    path = "/auth/google/callback/",
    tag = "Auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state"),
    ),
    responses(
        (status = 200, description = "Popup closure HTML"),
    ),
)]
pub async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, AppError> {
    let code = params.code.ok_or_else(|| AppError::BadRequest("Missing code".into()))?;
    let state_val = params.state.unwrap_or_default();
    let saved_state = jar.get(OAUTH_STATE_COOKIE).map(|c| c.value().to_owned());

    if saved_state.as_deref() != Some(&state_val) {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GoogleAuth, Some("Invalid OAuth state"), None)).into_response());
    }

    let client_id = get_instance_config(&state, "GOOGLE_CLIENT_ID").await?.unwrap_or_default();
    let client_secret = get_instance_config(&state, "GOOGLE_CLIENT_SECRET").await?.unwrap_or_default();
    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/google/callback/", base_url.trim_end_matches('/'));

    // Exchange code for token
    let token_resp = state.http.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send().await.map_err(|e| AppError::Internal(anyhow::anyhow!("Google token exchange failed: {e}")))?;

    if !token_resp.status().is_success() {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GoogleAuth, Some("Failed to exchange Google code"), None)).into_response());
    }

    let token_data: serde_json::Value = token_resp.json().await.unwrap_or_default();
    let access_token = token_data["access_token"].as_str().unwrap_or_default();

    // Fetch user info
    let user_resp = state.http.get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header("Authorization", format!("Bearer {access_token}"))
        .send().await.map_err(|e| AppError::Internal(anyhow::anyhow!("Google user fetch failed: {e}")))?;

    if !user_resp.status().is_success() {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GoogleAuth, Some("Failed to fetch Google user info"), None)).into_response());
    }

    let google_user: serde_json::Value = user_resp.json().await.unwrap_or_default();
    let email = google_user["email"].as_str().ok_or_else(|| AppError::Internal(anyhow::anyhow!("Google user has no email")))?;
    let name = google_user["name"].as_str().unwrap_or("");
    let avatar = google_user["picture"].as_str().unwrap_or("");

    let user = find_or_create_oauth_user(&state, &headers, email.to_owned(), name.to_owned(), String::new(), avatar.to_owned(), "google").await?;

    let jar = issue_session_cookie(&state, &headers, jar.remove(OAUTH_STATE_COOKIE), &user, SessionSurface::App).await?;

    Ok((jar, postmessage_html(true, OAuthMessageType::GoogleAuth, None, None)).into_response())
}

// ── Gitea ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/gitea/",
    tag = "Auth",
    responses(
        (status = 302, description = "Redirect to Gitea"),
    ),
)]
pub async fn gitea_initiate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let client_id = get_instance_config(&state, "GITEA_CLIENT_ID")
        .await?
        .ok_or_else(|| AppError::BadRequest("Gitea OAuth is not configured".into()))?;

    let gitea_host = get_instance_config(&state, "GITEA_HOST")
        .await?
        .unwrap_or_else(|| "https://gitea.com".to_owned());

    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/gitea/callback/", base_url.trim_end_matches('/'));

    let oauth_state = Uuid::new_v4().simple().to_string();
    let params = [
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("state", oauth_state.as_str()),
    ];

    let query = serde_urlencoded::to_string(params).unwrap();
    let url = format!("{}/login/oauth/authorize?{}", gitea_host.trim_end_matches('/'), query);

    let cookie = Cookie::build((OAUTH_STATE_COOKIE, oauth_state))
        .path("/")
        .http_only(true)
        .secure(state.config.is_production)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(10));

    Ok((jar.add(cookie), Redirect::to(&url)))
}

#[utoipa::path(
    get,
    path = "/auth/gitea/callback/",
    tag = "Auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state"),
    ),
    responses(
        (status = 200, description = "Popup closure HTML"),
    ),
)]
pub async fn gitea_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, AppError> {
    let code = params.code.ok_or_else(|| AppError::BadRequest("Missing code".into()))?;
    let state_val = params.state.unwrap_or_default();
    let saved_state = jar.get(OAUTH_STATE_COOKIE).map(|c| c.value().to_owned());

    if saved_state.as_deref() != Some(&state_val) {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GiteaAuth, Some("Invalid OAuth state"), None)).into_response());
    }

    let client_id = get_instance_config(&state, "GITEA_CLIENT_ID").await?.unwrap_or_default();
    let client_secret = get_instance_config(&state, "GITEA_CLIENT_SECRET").await?.unwrap_or_default();
    let gitea_host = get_instance_config(&state, "GITEA_HOST").await?.unwrap_or_else(|| "https://gitea.com".to_owned());
    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/gitea/callback/", base_url.trim_end_matches('/'));

    // Exchange code for token
    let token_resp = state.http.post(format!("{}/login/oauth/access_token", gitea_host.trim_end_matches('/')))
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send().await.map_err(|e| AppError::Internal(anyhow::anyhow!("Gitea token exchange failed: {e}")))?;

    if !token_resp.status().is_success() {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GiteaAuth, Some("Failed to exchange Gitea code"), None)).into_response());
    }

    let token_data: serde_json::Value = token_resp.json().await.unwrap_or_default();
    let access_token = token_data["access_token"].as_str().unwrap_or_default();

    // Fetch user info
    let user_resp = state.http.get(format!("{}/api/v1/user", gitea_host.trim_end_matches('/')))
        .header("Authorization", format!("token {access_token}"))
        .send().await.map_err(|e| AppError::Internal(anyhow::anyhow!("Gitea user fetch failed: {e}")))?;

    if !user_resp.status().is_success() {
        return Ok((jar.remove(OAUTH_STATE_COOKIE), postmessage_html(false, OAuthMessageType::GiteaAuth, Some("Failed to fetch Gitea user info"), None)).into_response());
    }

    let gitea_user: serde_json::Value = user_resp.json().await.unwrap_or_default();
    let email = gitea_user["email"].as_str().ok_or_else(|| AppError::Internal(anyhow::anyhow!("Gitea user has no email")))?;
    let name = gitea_user["full_name"].as_str().unwrap_or(gitea_user["username"].as_str().unwrap_or(""));
    let avatar = gitea_user["avatar_url"].as_str().unwrap_or("");

    let user = find_or_create_oauth_user(&state, &headers, email.to_owned(), name.to_owned(), String::new(), avatar.to_owned(), "gitea").await?;

    let jar = issue_session_cookie(&state, &headers, jar.remove(OAUTH_STATE_COOKIE), &user, SessionSurface::App).await?;

    Ok((jar, postmessage_html(true, OAuthMessageType::GiteaAuth, None, None)).into_response())
}

// ── GitHub user OAuth (login) ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/github",
    tag = "Auth",
    responses(
        (status = 302, description = "Redirect to GitHub for authentication"),
    ),
)]
pub async fn github_initiate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let client_id = get_instance_config(&state, "GITHUB_CLIENT_ID")
        .await?
        .ok_or_else(|| AppError::BadRequest("GitHub OAuth is not configured".into()))?;

    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/github/callback/", base_url.trim_end_matches('/'));

    let oauth_state = Uuid::new_v4().simple().to_string();
    let params = [
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("scope", "user:email"),
        ("state", oauth_state.as_str()),
    ];

    let query = serde_urlencoded::to_string(params).unwrap();
    let url = format!("https://github.com/login/oauth/authorize?{query}");

    let cookie = Cookie::build((OAUTH_STATE_COOKIE, oauth_state))
        .path("/")
        .http_only(true)
        .secure(state.config.is_production)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(10));

    Ok((jar.add(cookie), Redirect::to(&url)))
}

#[utoipa::path(
    get,
    path = "/auth/github/callback",
    tag = "Auth",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state"),
    ),
    responses(
        (status = 200, description = "Popup closure HTML with postMessage"),
    ),
)]
pub async fn github_auth_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, AppError> {
    let code = params.code.ok_or_else(|| AppError::BadRequest("Missing code".into()))?;
    let state_val = params.state.unwrap_or_default();
    let saved_state = jar.get(OAUTH_STATE_COOKIE).map(|c| c.value().to_owned());

    if saved_state.as_deref() != Some(&state_val) {
        return Ok((
            jar.remove(OAUTH_STATE_COOKIE),
            postmessage_html(false, OAuthMessageType::GithubAuth, Some("Invalid OAuth state"), None),
        )
            .into_response());
    }

    let client_id = get_instance_config(&state, "GITHUB_CLIENT_ID")
        .await?
        .unwrap_or_default();
    let client_secret = get_instance_config(&state, "GITHUB_CLIENT_SECRET")
        .await?
        .unwrap_or_default();
    let base_url = state.config.app_base();
    let redirect_uri = format!("{}/auth/github/callback/", base_url.trim_end_matches('/'));

    // Exchange code for access token
    let token_resp = state
        .http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub token exchange failed: {e}")))?;

    if !token_resp.status().is_success() {
        return Ok((
            jar.remove(OAUTH_STATE_COOKIE),
            postmessage_html(false, OAuthMessageType::GithubAuth, Some("Failed to exchange GitHub code"), None),
        )
            .into_response());
    }

    let token_data: serde_json::Value = token_resp.json().await.unwrap_or_default();
    let access_token = token_data["access_token"].as_str().unwrap_or_default();
    if access_token.is_empty() {
        return Ok((
            jar.remove(OAUTH_STATE_COOKIE),
            postmessage_html(false, OAuthMessageType::GithubAuth, Some("GitHub denied access"), None),
        )
            .into_response());
    }

    // Fetch user info
    let user_resp = state
        .http
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "plane-api")
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub user fetch failed: {e}")))?;

    if !user_resp.status().is_success() {
        return Ok((
            jar.remove(OAUTH_STATE_COOKIE),
            postmessage_html(false, OAuthMessageType::GithubAuth, Some("Failed to fetch GitHub user info"), None),
        )
            .into_response());
    }

    let github_user: serde_json::Value = user_resp.json().await.unwrap_or_default();

    // GitHub may not expose email publicly; fetch from /user/emails
    let email = if let Some(e) = github_user["email"].as_str().filter(|e| !e.is_empty()) {
        e.to_owned()
    } else {
        let emails_resp = state
            .http
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {access_token}"))
            .header("User-Agent", "plane-api")
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("GitHub emails fetch failed: {e}")))?;

        let emails: serde_json::Value = emails_resp.json().await.unwrap_or_default();
        emails
            .as_array()
            .and_then(|arr| {
                arr.iter()
                    .find(|e| e["primary"].as_bool().unwrap_or(false))
                    .and_then(|e| e["email"].as_str())
                    .map(str::to_owned)
            })
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("GitHub user has no email")))?
    };

    let name = github_user["name"].as_str().unwrap_or("");
    let avatar = github_user["avatar_url"].as_str().unwrap_or("");

    let user = find_or_create_oauth_user(
        &state,
        &headers,
        email,
        name.to_owned(),
        String::new(),
        avatar.to_owned(),
        "github",
    )
    .await?;

    let jar = issue_session_cookie(
        &state,
        &headers,
        jar.remove(OAUTH_STATE_COOKIE),
        &user,
        SessionSurface::App,
    )
    .await?;

    Ok((jar, postmessage_html(true, OAuthMessageType::GithubAuth, None, None)).into_response())
}
