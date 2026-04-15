use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::CookieJar;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use zxcvbn::{zxcvbn, Score};

use crate::{
    auth::{
        csrf::{is_valid_csrf_header, CSRF_COOKIE_NAME},
        responses::{AuthError, AuthErrorBody, PasswordMessageResponse},
        session::{replace_session_cookie, SessionSurface, SessionUser},
    },
    entities::users,
    error::AppError,
    utils::passwords::{make_password, verify_password},
    AppState,
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SetPasswordRequest {
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/auth/change-password",
    tag = "Auth",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password updated", body = PasswordMessageResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Invalid CSRF token"),
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    SessionUser(user): SessionUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<(CookieJar, Json<PasswordMessageResponse>), Response> {
    validate_csrf_header(&headers, &jar).map_err(IntoResponse::into_response)?;

    if payload.new_password.trim().is_empty() {
        return Err(AuthError::missing_password().into_response());
    }

    if !user.is_password_autoset {
        let Some(old_password) = payload.old_password.as_deref() else {
            return Err(AuthError::missing_password().into_response());
        };

        if !verify_password(old_password, &user.password) {
            return Err(AuthError::incorrect_old_password().into_response());
        }
    }

    validate_password_strength(&payload.new_password).map_err(IntoResponse::into_response)?;

    let password_hash = make_password(&payload.new_password)
        .map_err(AppError::Internal)
        .map_err(IntoResponse::into_response)?;
    let mut user_model: users::ActiveModel = user.into();
    user_model.password = Set(password_hash);
    user_model.is_password_autoset = Set(false);
    let user = user_model
        .update(&state.db)
        .await
        .map_err(AppError::Database)
        .map_err(IntoResponse::into_response)?;
    let jar = replace_session_cookie(&state, &headers, jar, &user, SessionSurface::App)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok((
        jar,
        Json(PasswordMessageResponse {
            message: "Password updated successfully".to_owned(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/set-password",
    tag = "Auth",
    request_body = SetPasswordRequest,
    responses(
        (status = 200, description = "Password set", body = PasswordMessageResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Invalid CSRF token"),
    )
)]
pub async fn set_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    SessionUser(user): SessionUser,
    Json(payload): Json<SetPasswordRequest>,
) -> Result<(CookieJar, Json<PasswordMessageResponse>), Response> {
    validate_csrf_header(&headers, &jar).map_err(IntoResponse::into_response)?;

    if !user.is_password_autoset {
        return Err(AuthError::password_already_set().into_response());
    }

    if payload.password.trim().is_empty() {
        return Err(AuthError::invalid_password().into_response());
    }

    validate_password_strength(&payload.password).map_err(IntoResponse::into_response)?;

    let password_hash = make_password(&payload.password)
        .map_err(AppError::Internal)
        .map_err(IntoResponse::into_response)?;
    let mut user_model: users::ActiveModel = user.into();
    user_model.password = Set(password_hash);
    user_model.is_password_autoset = Set(false);
    let user = user_model
        .update(&state.db)
        .await
        .map_err(AppError::Database)
        .map_err(IntoResponse::into_response)?;
    let jar = replace_session_cookie(&state, &headers, jar, &user, SessionSurface::App)
        .await
        .map_err(IntoResponse::into_response)?;

    Ok((
        jar,
        Json(PasswordMessageResponse {
            message: "Password set successfully".to_owned(),
        }),
    ))
}

fn validate_csrf_header(headers: &HeaderMap, jar: &CookieJar) -> Result<(), AppError> {
    let csrf_cookie = jar
        .get(CSRF_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned())
        .ok_or(AppError::Forbidden)?;

    if is_valid_csrf_header(headers, &csrf_cookie) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn validate_password_strength(password: &str) -> Result<(), AuthError> {
    let estimate = zxcvbn(password, &[]);
    if estimate.score() < Score::Three {
        return Err(AuthError::password_too_weak());
    }

    Ok(())
}
