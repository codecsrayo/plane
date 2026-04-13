use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::Redirect,
    Json,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use fred::prelude::{Expiration, KeysInterface};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::{
        email_auth::{
            ensure_profile_exists, ensure_signup_allowed, redirect_error, safe_next_path,
            space_base,
        },
        responses::{AuthError, AuthErrorBody},
        session::{issue_session_cookie, replace_session_cookie, SessionSurface},
    },
    entities::{instances, profiles, users},
    error::AppError,
    utils::{
        instance_config::get_config_value, passwords::make_password, soft_delete::SoftDeleteExt,
    },
    AppState,
};

const MAGIC_CODE_TTL_SECS: i64 = 600;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MagicGenerateRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MagicGenerateResponse {
    pub key: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MagicAuthForm {
    pub email: Option<String>,
    pub code: Option<String>,
    pub next_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MagicCodeData {
    current_attempt: i32,
    email: String,
    token: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/magic-generate",
    tag = "Auth",
    request_body = MagicGenerateRequest,
    responses(
        (status = 200, description = "Magic code generated", body = MagicGenerateResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
    )
)]
pub async fn magic_generate(
    State(state): State<AppState>,
    Json(payload): Json<MagicGenerateRequest>,
) -> Result<Json<MagicGenerateResponse>, AuthError> {
    generate_magic_code(&state, payload).await.map(Json)
}

#[utoipa::path(
    post,
    path = "/api/auth/spaces/magic-generate",
    tag = "Auth",
    request_body = MagicGenerateRequest,
    responses(
        (status = 200, description = "Space magic code generated", body = MagicGenerateResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
    )
)]
pub async fn magic_generate_space(
    State(state): State<AppState>,
    Json(payload): Json<MagicGenerateRequest>,
) -> Result<Json<MagicGenerateResponse>, AuthError> {
    generate_magic_code(&state, payload).await.map(Json)
}

#[utoipa::path(
    post,
    path = "/api/auth/magic-sign-in",
    tag = "Auth",
    request_body(
        content = MagicAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Signed in with magic code"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn magic_sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<MagicAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    complete_magic_auth(
        &state,
        &headers,
        jar,
        form,
        SessionSurface::App,
        false,
        false,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/auth/magic-sign-up",
    tag = "Auth",
    request_body(
        content = MagicAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Signed up with magic code"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn magic_sign_up(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<MagicAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    complete_magic_auth(
        &state,
        &headers,
        jar,
        form,
        SessionSurface::App,
        true,
        false,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/auth/spaces/magic-sign-in",
    tag = "Auth",
    request_body(
        content = MagicAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Space signed in with magic code"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn magic_sign_in_space(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<MagicAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    complete_magic_auth(
        &state,
        &headers,
        jar,
        form,
        SessionSurface::Space,
        false,
        true,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/auth/spaces/magic-sign-up",
    tag = "Auth",
    request_body(
        content = MagicAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Space signed up with magic code"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn magic_sign_up_space(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<MagicAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    complete_magic_auth(
        &state,
        &headers,
        jar,
        form,
        SessionSurface::Space,
        true,
        true,
    )
    .await
}

async fn generate_magic_code(
    state: &AppState,
    payload: MagicGenerateRequest,
) -> Result<MagicGenerateResponse, AuthError> {
    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(|_| AuthError::instance_not_configured())?;
    if !instance.is_some_and(|instance| instance.is_setup_done) {
        return Err(AuthError::instance_not_configured());
    }

    let email = payload.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(AuthError::email_required());
    }
    if email.parse::<lettre::Address>().is_err() {
        return Err(AuthError::invalid_email());
    }

    ensure_magic_enabled(state, &email).await?;

    let redis_key = format!("magic_{email}");
    let existing = state
        .redis
        .get::<Option<String>, _>(&redis_key)
        .await
        .map_err(|_| AuthError::instance_not_configured())?;
    let token = format!(
        "{:06}",
        (uuid::Uuid::new_v4().as_u128() % 900000) as u32 + 100000
    );

    let data = if let Some(raw) = existing {
        let mut data: MagicCodeData =
            serde_json::from_str(&raw).map_err(|_| AuthError::instance_not_configured())?;
        if data.current_attempt > 2 {
            let exists = users::Entity::find()
                .filter(users::Column::Email.eq(Some(email.clone())))
                .one(&state.db)
                .await
                .map_err(|_| AuthError::instance_not_configured())?
                .is_some();
            return Err(if exists {
                AuthError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    5100,
                    "EMAIL_CODE_ATTEMPT_EXHAUSTED_SIGN_IN",
                )
            } else {
                AuthError::new(
                    axum::http::StatusCode::BAD_REQUEST,
                    5102,
                    "EMAIL_CODE_ATTEMPT_EXHAUSTED_SIGN_UP",
                )
            });
        }
        data.current_attempt += 1;
        data.token = token.clone();
        data
    } else {
        MagicCodeData {
            current_attempt: 0,
            email: email.clone(),
            token: token.clone(),
        }
    };

    state
        .redis
        .set::<(), _, _>(
            &redis_key,
            serde_json::to_string(&data).map_err(|_| AuthError::instance_not_configured())?,
            Some(Expiration::EX(MAGIC_CODE_TTL_SECS)),
            None,
            false,
        )
        .await
        .map_err(|_| AuthError::instance_not_configured())?;

    if let Err(error) = send_magic_code_email(state, &email, &token).await {
        tracing::error!(error = %error, email = %email, "Failed to send magic code email");
    }

    Ok(MagicGenerateResponse { key: redis_key })
}

async fn complete_magic_auth(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    form: MagicAuthForm,
    surface: SessionSurface,
    is_signup: bool,
    is_space: bool,
) -> Result<(CookieJar, Redirect), AppError> {
    let Some(code) = form
        .code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            if is_signup { 5055 } else { 5085 },
            if is_signup {
                "MAGIC_SIGN_UP_EMAIL_CODE_REQUIRED"
            } else {
                "MAGIC_SIGN_IN_EMAIL_CODE_REQUIRED"
            },
        ));
    };
    let Some(email) = form
        .email
        .as_deref()
        .map(|value| value.trim().to_lowercase())
    else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            if is_signup { 5055 } else { 5085 },
            if is_signup {
                "MAGIC_SIGN_UP_EMAIL_CODE_REQUIRED"
            } else {
                "MAGIC_SIGN_IN_EMAIL_CODE_REQUIRED"
            },
        ));
    };

    let existing_user = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if is_signup && existing_user.is_some() {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5030,
            "USER_ALREADY_EXIST",
        ));
    }
    if !is_signup && existing_user.is_none() {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5060,
            "USER_DOES_NOT_EXIST",
        ));
    }

    let redis_key = format!("magic_{email}");
    let cached = state
        .redis
        .get::<Option<String>, _>(&redis_key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;
    let Some(raw) = cached else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            if is_signup { 5097 } else { 5095 },
            if is_signup {
                "EXPIRED_MAGIC_CODE_SIGN_UP"
            } else {
                "EXPIRED_MAGIC_CODE_SIGN_IN"
            },
        ));
    };
    let data: MagicCodeData =
        serde_json::from_str(&raw).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if data.token != code {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            if is_signup { 5092 } else { 5090 },
            if is_signup {
                "INVALID_MAGIC_CODE_SIGN_UP"
            } else {
                "INVALID_MAGIC_CODE_SIGN_IN"
            },
        ));
    }

    state
        .redis
        .del::<i64, _>(&redis_key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    let user = if let Some(user) = existing_user {
        ensure_profile_exists(state, &user).await?;
        user
    } else {
        if let Some(url) = ensure_signup_allowed(state, &email, &form.next_path, is_space).await? {
            return Ok((jar, Redirect::to(&url)));
        }
        create_magic_user(state, headers, &email).await?
    };

    let user = update_magic_login_metadata(state, user, headers).await?;
    let jar = if is_signup {
        issue_session_cookie(state, headers, jar, &user, surface).await?
    } else {
        replace_session_cookie(state, headers, jar, &user, surface).await?
    };

    let redirect_to = if matches!(surface, SessionSurface::App) && user.is_password_autoset {
        let profile = profiles::Entity::find()
            .filter(profiles::Column::UserId.eq(user.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;
        if profile.is_some_and(|profile| profile.is_onboarded) {
            format!("{}/", app_base(state).trim_end_matches('/'))
        } else {
            app_success_redirect(state, &user, form.next_path.as_deref()).await?
        }
    } else if matches!(surface, SessionSurface::Space) {
        let path = safe_next_path(form.next_path.as_deref()).unwrap_or_else(|| "/".to_owned());
        format!("{}{}", space_base(state).trim_end_matches('/'), path)
    } else {
        app_success_redirect(state, &user, form.next_path.as_deref()).await?
    };

    Ok((jar, Redirect::to(&redirect_to)))
}

async fn create_magic_user(
    state: &AppState,
    headers: &HeaderMap,
    email: &str,
) -> Result<users::Model, AppError> {
    let now = Utc::now();
    let random_password = uuid::Uuid::new_v4().to_string();
    let password_hash = make_password(&random_password).map_err(AppError::Internal)?;
    let user_id = uuid::Uuid::new_v4();
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );

    users::ActiveModel {
        password: Set(password_hash),
        last_login: Set(Some(now.into())),
        id: Set(user_id),
        username: Set(uuid::Uuid::new_v4().simple().to_string()),
        mobile_number: Set(None),
        email: Set(Some(email.to_owned())),
        first_name: Set(String::new()),
        last_name: Set(String::new()),
        avatar: Set(String::new()),
        date_joined: Set(now.into()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        last_location: Set(String::new()),
        created_location: Set(String::new()),
        is_superuser: Set(false),
        is_managed: Set(false),
        is_password_expired: Set(false),
        is_active: Set(true),
        is_staff: Set(false),
        is_email_verified: Set(true),
        is_password_autoset: Set(true),
        token: Set(token),
        user_timezone: Set("UTC".to_owned()),
        last_active: Set(Some(now.into())),
        last_login_time: Set(Some(now.into())),
        last_logout_time: Set(None),
        last_login_ip: Set(crate::auth::session::extract_client_ip(headers)),
        last_logout_ip: Set(String::new()),
        last_login_medium: Set("magic-code".to_owned()),
        last_login_uagent: Set(headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned()),
        token_updated_at: Set(Some(now.into())),
        is_bot: Set(false),
        cover_image: Set(None),
        display_name: Set(email.split('@').next().unwrap_or("user").to_owned()),
        avatar_asset_id: Set(None),
        cover_image_asset_id: Set(None),
        bot_type: Set(None),
        is_email_valid: Set(false),
        masked_at: Set(None),
        is_password_reset_required: Set(false),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    let user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;
    ensure_profile_exists(state, &user).await?;

    Ok(user)
}

async fn update_magic_login_metadata(
    state: &AppState,
    user: users::Model,
    headers: &HeaderMap,
) -> Result<users::Model, AppError> {
    let now = Utc::now();
    let mut user_model: users::ActiveModel = user.into();
    user_model.last_login = Set(Some(now.into()));
    user_model.last_active = Set(Some(now.into()));
    user_model.last_login_time = Set(Some(now.into()));
    user_model.last_login_ip = Set(crate::auth::session::extract_client_ip(headers));
    user_model.last_login_medium = Set("magic-code".to_owned());
    user_model.last_login_uagent = Set(headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned());
    user_model.updated_at = Set(now.into());
    user_model.is_active = Set(true);
    user_model
        .update(&state.db)
        .await
        .map_err(AppError::Database)
}

async fn ensure_magic_enabled(state: &AppState, email: &str) -> Result<(), AuthError> {
    let smtp_host = get_config_value(
        state,
        "EMAIL_HOST",
        std::env::var("EMAIL_HOST").ok().as_deref(),
    )
    .await
    .map_err(|_| AuthError::instance_not_configured())?;
    if smtp_host.as_deref().is_none_or(str::is_empty) {
        return Err(AuthError::new(
            axum::http::StatusCode::BAD_REQUEST,
            5025,
            "SMTP_NOT_CONFIGURED",
        ));
    }

    let enabled = get_config_value(
        state,
        "ENABLE_MAGIC_LINK_LOGIN",
        std::env::var("ENABLE_MAGIC_LINK_LOGIN")
            .ok()
            .as_deref()
            .or(Some("1")),
    )
    .await
    .map_err(|_| AuthError::instance_not_configured())?
    .is_none_or(|value| value != "0");
    if !enabled {
        return Err(AuthError::new(
            axum::http::StatusCode::BAD_REQUEST,
            5016,
            "MAGIC_LINK_LOGIN_DISABLED",
        ));
    }

    if email.is_empty() {
        return Err(AuthError::email_required());
    }

    Ok(())
}

async fn send_magic_code_email(state: &AppState, email: &str, token: &str) -> Result<(), AppError> {
    let host = get_config_value(
        state,
        "EMAIL_HOST",
        std::env::var("EMAIL_HOST").ok().as_deref(),
    )
    .await?;
    let user = get_config_value(
        state,
        "EMAIL_HOST_USER",
        std::env::var("EMAIL_HOST_USER").ok().as_deref(),
    )
    .await?;
    let password = get_config_value(
        state,
        "EMAIL_HOST_PASSWORD",
        std::env::var("EMAIL_HOST_PASSWORD").ok().as_deref(),
    )
    .await?;
    let port = get_config_value(
        state,
        "EMAIL_PORT",
        std::env::var("EMAIL_PORT").ok().as_deref(),
    )
    .await?
    .unwrap_or_else(|| "587".to_owned());
    let email_from = get_config_value(
        state,
        "EMAIL_FROM",
        std::env::var("EMAIL_FROM").ok().as_deref(),
    )
    .await?
    .unwrap_or_else(|| "no-reply@plane.local".to_owned());

    let body = format!(
        "<p>Your unique Plane login code is <strong>{token}</strong>.</p><p>Email: {email}</p>"
    );
    let message = Message::builder()
        .from(
            email_from
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid EMAIL_FROM: {e}")))?,
        )
        .to(email
            .parse()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid email: {e}")))?)
        .subject(format!("Your unique Plane login code is {token}"))
        .header(ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("email build error: {e}")))?;

    let host = host.ok_or_else(|| AppError::BadRequest("Missing EMAIL_HOST".to_owned()))?;
    let port: u16 = port
        .parse()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid EMAIL_PORT: {e}")))?;
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("smtp relay error: {e}")))?;
    builder = builder.port(port);
    if let (Some(user), Some(password)) = (user, password) {
        builder = builder.credentials(Credentials::new(user, password));
    }

    builder
        .build()
        .send(message)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("smtp send error: {e}")))?;
    Ok(())
}

async fn app_success_redirect(
    state: &AppState,
    user: &users::Model,
    next_path: Option<&str>,
) -> Result<String, AppError> {
    if let Some(path) = safe_next_path(next_path) {
        return Ok(format!("{}{}", app_base(state).trim_end_matches('/'), path));
    }

    let path = crate::auth::email_auth::app_default_path(state, user).await?;
    Ok(format!(
        "{}/{}",
        app_base(state).trim_end_matches('/'),
        path
    ))
}

fn app_base(state: &AppState) -> String {
    state
        .config
        .app_base_url
        .clone()
        .or_else(|| state.config.web_url.clone())
        .unwrap_or_else(|| "/".to_owned())
}
