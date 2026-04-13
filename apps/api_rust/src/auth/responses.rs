use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthErrorBody {
    pub error_code: i32,
    pub error_message: &'static str,
}

pub struct AuthError {
    pub status: StatusCode,
    pub error_code: i32,
    pub error_message: &'static str,
}

impl AuthError {
    pub const fn new(status: StatusCode, error_code: i32, error_message: &'static str) -> Self {
        Self {
            status,
            error_code,
            error_message,
        }
    }

    pub const fn instance_not_configured() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5000, "INSTANCE_NOT_CONFIGURED")
    }

    pub const fn invalid_email() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5005, "INVALID_EMAIL")
    }

    pub const fn email_required() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5010, "EMAIL_REQUIRED")
    }

    pub const fn invalid_password() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5020, "INVALID_PASSWORD")
    }

    pub const fn password_too_weak() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5021, "PASSWORD_TOO_WEAK")
    }

    pub const fn incorrect_old_password() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5135, "INCORRECT_OLD_PASSWORD")
    }

    pub const fn missing_password() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5138, "MISSING_PASSWORD")
    }

    pub const fn password_already_set() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5145, "PASSWORD_ALREADY_SET")
    }

    pub const fn user_does_not_exist() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5060, "USER_DOES_NOT_EXIST")
    }

    pub const fn smtp_not_configured() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5025, "SMTP_NOT_CONFIGURED")
    }

    pub const fn invalid_password_token() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5125, "INVALID_PASSWORD_TOKEN")
    }

    pub const fn expired_password_token() -> Self {
        Self::new(StatusCode::BAD_REQUEST, 5130, "EXPIRED_PASSWORD_TOKEN")
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(AuthErrorBody {
                error_code: self.error_code,
                error_message: self.error_message,
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmailCheckResponse {
    pub existing: bool,
    pub status: &'static str,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PasswordMessageResponse {
    pub message: String,
}
