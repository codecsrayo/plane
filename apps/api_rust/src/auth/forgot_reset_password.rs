// src/auth/forgot_reset_password.rs
//
// Implementa:
//   POST  /api/auth/forgot-password
//   POST  /api/auth/reset-password/:uidb64/:token
//   POST  /api/auth/spaces/forgot-password
//   POST  /api/auth/spaces/reset-password/:uidb64/:token
//
// Equivalente a las vistas Django:
//   - authentication/views/app/password_management.py   → ForgotPasswordEndpoint / ResetPasswordEndpoint
//   - authentication/views/space/password_management.py → ForgotPasswordSpaceEndpoint / ResetPasswordSpaceEndpoint
//
// Diferencia clave vs Django:
//   Django usa PasswordResetTokenGenerator (HMAC stateless ligado al hash de contraseña).
//   Rust usa Redis con TTL de 24 h — más seguro (revocable) y sin dependencia de la clave secreta Django.

use axum::{
    extract::{Form, Path, State},
    http::HeaderMap,
    response::Redirect,
    Json,
};
use axum_extra::extract::CookieJar;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use fred::prelude::{Expiration, KeysInterface};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use zxcvbn::{Score, zxcvbn};

use crate::{
    auth::{
        email_auth::{redirect_error, safe_next_path, space_base},
        responses::{AuthError, AuthErrorBody, PasswordMessageResponse},
        session::SessionSurface,
    },
    entities::{instances, users},
    error::AppError,
    utils::{instance_config::get_config_value, passwords::make_password, soft_delete::SoftDeleteExt},
    AppState,
};

/// TTL del token de reset — 24 horas (Django default = 3 días; reducimos por seguridad)
const RESET_TOKEN_TTL_SECS: i64 = 86_400;

// ─── Structs públicos ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordForm {
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResetTokenData {
    token: String,
    user_id: uuid::Uuid,
    email: String,
}

// ─── Handlers app ─────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset email sent", body = PasswordMessageResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<PasswordMessageResponse>, AuthError> {
    handle_forgot_password(&state, payload, SessionSurface::App).await
}

#[utoipa::path(
    post,
    path = "/api/auth/reset-password/{uidb64}/{token}",
    tag = "Auth",
    params(
        ("uidb64" = String, Path, description = "Base64url-encoded user UUID"),
        ("token"  = String, Path, description = "Reset token from email link"),
    ),
    request_body(
        content = ResetPasswordForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Password reset — redirect to sign-in"),
        (status = 302, description = "Redirect with error"),
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Path((uidb64, token)): Path<(String, String)>,
    _headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<ResetPasswordForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    handle_reset_password(&state, uidb64, token, form, SessionSurface::App).await
}

// ─── Handlers spaces ──────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/auth/spaces/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset email sent (space)", body = PasswordMessageResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
    )
)]
pub async fn forgot_password_space(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<PasswordMessageResponse>, AuthError> {
    handle_forgot_password(&state, payload, SessionSurface::Space).await
}

#[utoipa::path(
    post,
    path = "/api/auth/spaces/reset-password/{uidb64}/{token}",
    tag = "Auth",
    params(
        ("uidb64" = String, Path, description = "Base64url-encoded user UUID"),
        ("token"  = String, Path, description = "Reset token from email link"),
    ),
    request_body(
        content = ResetPasswordForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Password reset (space)"),
        (status = 302, description = "Redirect with error (space)"),
    )
)]
pub async fn reset_password_space(
    State(state): State<AppState>,
    Path((uidb64, token)): Path<(String, String)>,
    _headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<ResetPasswordForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    handle_reset_password(&state, uidb64, token, form, SessionSurface::Space).await
}

// ─── Lógica compartida ────────────────────────────────────────────────────────

async fn handle_forgot_password(
    state: &AppState,
    payload: ForgotPasswordRequest,
    _surface: SessionSurface,
) -> Result<Json<PasswordMessageResponse>, AuthError> {
    // 1. Instancia configurada
    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(|_| AuthError::instance_not_configured())?;
    if !instance.is_some_and(|i| i.is_setup_done) {
        return Err(AuthError::instance_not_configured());
    }

    // 2. SMTP configurado
    let smtp_host = get_config_value(
        state,
        "EMAIL_HOST",
        std::env::var("EMAIL_HOST").ok().as_deref(),
    )
    .await
    .map_err(|_| AuthError::instance_not_configured())?;
    if smtp_host.as_deref().is_none_or(str::is_empty) {
        return Err(AuthError::smtp_not_configured());
    }

    // 3. Email válido
    let email = payload.email.trim().to_lowercase();
    if email.parse::<lettre::Address>().is_err() {
        return Err(AuthError::invalid_email());
    }

    // 4. Usuario existe
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(|_| AuthError::instance_not_configured())?;
    let Some(user) = user else {
        return Err(AuthError::user_does_not_exist());
    };

    // 5. Generar uidb64 y token aleatorio
    let uidb64 = URL_SAFE_NO_PAD.encode(user.id.to_string());
    let token = uuid::Uuid::new_v4().simple().to_string();

    // 6. Persistir en Redis
    let redis_key = format!("pwreset_{uidb64}");
    let data = ResetTokenData {
        token: token.clone(),
        user_id: user.id,
        email: email.clone(),
    };
    state
        .redis
        .set::<(), _, _>(
            &redis_key,
            serde_json::to_string(&data).map_err(|_| AuthError::instance_not_configured())?,
            Some(Expiration::EX(RESET_TOKEN_TTL_SECS)),
            None,
            false,
        )
        .await
        .map_err(|_| AuthError::instance_not_configured())?;

    // 7. Enviar email (best-effort — no expone error al cliente)
    let base = app_base(state);
    let reset_url = format!(
        "{}/accounts/reset-password/?uidb64={}&token={}&email={}",
        base.trim_end_matches('/'),
        uidb64,
        token,
        urlencoding::encode(&email),
    );
    if let Err(err) = send_reset_email(state, &user, &email, &reset_url).await {
        tracing::error!(error = %err, email = %email, "Failed to send password reset email");
    }

    Ok(Json(PasswordMessageResponse {
        message: "Check your email to reset your password".to_owned(),
    }))
}

async fn handle_reset_password(
    state: &AppState,
    uidb64: String,
    token: String,
    form: ResetPasswordForm,
    surface: SessionSurface,
) -> Result<(CookieJar, Redirect), AppError> {
    let error_base = match surface {
        SessionSurface::Space => space_base(state),
        _ => app_base(state),
    };
    let reset_page = format!(
        "{}/accounts/reset-password/",
        error_base.trim_end_matches('/')
    );

    // 1. Decodificar uidb64 → user_id
    let user_id = match decode_uidb64(&uidb64) {
        Some(id) => id,
        None => {
            return Ok((
                CookieJar::new(),
                Redirect::to(&format!(
                    "{}?error_code=5125&error_message=INVALID_PASSWORD_TOKEN",
                    reset_page
                )),
            ));
        }
    };

    // 2. Validar token contra Redis
    let redis_key = format!("pwreset_{uidb64}");
    let cached = state
        .redis
        .get::<Option<String>, _>(&redis_key)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    let Some(raw) = cached else {
        return Ok((
            CookieJar::new(),
            Redirect::to(&format!(
                "{}?error_code=5130&error_message=EXPIRED_PASSWORD_TOKEN",
                reset_page
            )),
        ));
    };

    let data: ResetTokenData =
        serde_json::from_str(&raw).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // Comparación constante para evitar timing attacks
    if !constant_time_eq(&data.token, &token) || data.user_id != user_id {
        return Ok((
            CookieJar::new(),
            Redirect::to(&format!(
                "{}?error_code=5125&error_message=INVALID_PASSWORD_TOKEN",
                reset_page
            )),
        ));
    }

    // 3. Validar contraseña
    let password = match form.password.as_deref().filter(|p| !p.trim().is_empty()) {
        Some(p) => p,
        None => {
            return Ok((
                CookieJar::new(),
                Redirect::to(&format!(
                    "{}?error_code=5020&error_message=INVALID_PASSWORD",
                    reset_page
                )),
            ));
        }
    };

    let estimate = zxcvbn(password, &[]);
    if estimate.score() < Score::Three {
        return Ok((
            CookieJar::new(),
            Redirect::to(&format!(
                "{}?error_code=5021&error_message=PASSWORD_TOO_WEAK",
                reset_page
            )),
        ));
    }

    // 4. Obtener usuario y actualizar contraseña
    let user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

    let password_hash = make_password(password).map_err(AppError::Internal)?;
    let mut user_model: users::ActiveModel = user.into();
    user_model.password = Set(password_hash);
    user_model.is_password_autoset = Set(false);
    user_model
        .update(&state.db)
        .await
        .map_err(AppError::Database)?;

    // 5. Invalidar token en Redis
    let _ = state.redis.del::<i64, _>(&redis_key).await;

    // 6. Redirect al login
    let success_url = match surface {
        SessionSurface::Space => space_base(state),
        _ => format!("{}/sign-in/?success=true", app_base(state).trim_end_matches('/')),
    };

    Ok((CookieJar::new(), Redirect::to(&success_url)))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn decode_uidb64(uidb64: &str) -> Option<uuid::Uuid> {
    let bytes = URL_SAFE_NO_PAD.decode(uidb64).ok()?;
    let uuid_str = std::str::from_utf8(&bytes).ok()?;
    uuid_str.parse::<uuid::Uuid>().ok()
}

/// Comparación en tiempo constante para tokens string.
/// Previene timing attacks en la validación del token de reset.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn app_base(state: &AppState) -> String {
    state
        .config
        .app_base_url
        .clone()
        .or_else(|| state.config.web_url.clone())
        .unwrap_or_else(|| "/".to_owned())
}

async fn send_reset_email(
    state: &AppState,
    user: &users::Model,
    email: &str,
    reset_url: &str,
) -> Result<(), AppError> {
    let host = get_config_value(
        state,
        "EMAIL_HOST",
        std::env::var("EMAIL_HOST").ok().as_deref(),
    )
    .await?;
    let smtp_user = get_config_value(
        state,
        "EMAIL_HOST_USER",
        std::env::var("EMAIL_HOST_USER").ok().as_deref(),
    )
    .await?;
    let smtp_password = get_config_value(
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
    let from = get_config_value(
        state,
        "EMAIL_FROM",
        std::env::var("EMAIL_FROM").ok().as_deref(),
    )
    .await?
    .unwrap_or_else(|| "no-reply@plane.local".to_owned());

    let first_name = if user.first_name.is_empty() {
        "there"
    } else {
        &user.first_name
    };

    let body = format!(
        "<p>Hi {first_name},</p>\
         <p>A password reset has been requested for your Plane account.</p>\
         <p><a href=\"{reset_url}\">Reset your password</a></p>\
         <p>This link expires in 24 hours. If you did not request this, you can ignore this email.</p>",
    );

    let message = Message::builder()
        .from(
            from.parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid EMAIL_FROM: {e}")))?,
        )
        .to(email
            .parse()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid to email: {e}")))?)
        .subject("A new password to your Plane account has been requested")
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
    if let (Some(u), Some(p)) = (smtp_user, smtp_password) {
        builder = builder.credentials(Credentials::new(u, p));
    }

    builder
        .build()
        .send(message)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("smtp send error: {e}")))?;

    Ok(())
}
