// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Validation error in DRF-compatible field-keyed format.
    /// Body shape: `{"field_name": ["ERROR_CODE", ...], ...}` — status 400.
    /// Use this when the frontend expects specific error codes per field
    /// (e.g. `PROJECT_IDENTIFIER_ALREADY_EXIST`).
    #[error("Validation error")]
    Validation(serde_json::Value),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody { error: self.to_string() }),
            )
                .into_response(),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody { error: self.to_string() }),
            )
                .into_response(),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(ErrorBody { error: self.to_string() }),
            )
                .into_response(),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ErrorBody { error: self.to_string() }),
            )
                .into_response(),
            AppError::BadRequest(m) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody { error: m.clone() }),
            )
                .into_response(),
            AppError::Validation(body) => {
                // 422 Unprocessable Entity — body bien formado pero contenido
                // inválido a nivel de campo. Permite que el cliente discrimine
                // entre malformed JSON (400 BadRequest) y errores de validación.
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body.clone())).into_response()
            }
            AppError::Conflict(m) => (
                StatusCode::CONFLICT,
                Json(ErrorBody { error: m.clone() }),
            )
                .into_response(),
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody { error: "Database error".into() }),
                )
                    .into_response()
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody { error: "Internal server error".into() }),
                )
                    .into_response()
            }
        }
    }
}
