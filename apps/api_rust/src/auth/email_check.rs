use std::env;

use axum::{extract::State, Json};
use lettre::Address;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::responses::{AuthError, AuthErrorBody, EmailCheckResponse},
    entities::{instances, users},
    utils::{instance_config::get_config_value, soft_delete::SoftDeleteExt},
    AppState,
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct EmailCheckRequest {
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/auth/email-check",
    tag = "Auth",
    request_body = EmailCheckRequest,
    responses(
        (status = 200, description = "Email status resolved", body = EmailCheckResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
    )
)]
pub async fn email_check(
    State(state): State<AppState>,
    Json(payload): Json<EmailCheckRequest>,
) -> Result<Json<EmailCheckResponse>, AuthError> {
    run_email_check(&state, payload).await.map(Json)
}

#[utoipa::path(
    post,
    path = "/auth/spaces/email-check",
    tag = "Auth",
    request_body = EmailCheckRequest,
    responses(
        (status = 200, description = "Email status resolved", body = EmailCheckResponse),
        (status = 400, description = "Auth validation error", body = AuthErrorBody),
    )
)]
pub async fn email_check_space(
    State(state): State<AppState>,
    Json(payload): Json<EmailCheckRequest>,
) -> Result<Json<EmailCheckResponse>, AuthError> {
    run_email_check(&state, payload).await.map(Json)
}

async fn run_email_check(
    state: &AppState,
    payload: EmailCheckRequest,
) -> Result<EmailCheckResponse, AuthError> {
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

    if email.parse::<Address>().is_err() {
        return Err(AuthError::invalid_email());
    }

    let smtp_configured =
        get_config_value(state, "EMAIL_HOST", env::var("EMAIL_HOST").ok().as_deref())
            .await
            .map_err(|_| AuthError::instance_not_configured())?
            .is_some_and(|value| !value.trim().is_empty());

    let magic_login_enabled = get_config_value(
        state,
        "ENABLE_MAGIC_LINK_LOGIN",
        Some(
            env::var("ENABLE_MAGIC_LINK_LOGIN")
                .unwrap_or_else(|_| "1".to_owned())
                .as_str(),
        ),
    )
    .await
    .map_err(|_| AuthError::instance_not_configured())?
    .is_none_or(|value| value == "1");

    let existing = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(|_| AuthError::instance_not_configured())?;

    let status = if existing
        .as_ref()
        .is_some_and(|user| user.is_password_autoset && smtp_configured && magic_login_enabled)
        || (existing.is_none() && smtp_configured && magic_login_enabled)
    {
        "MAGIC_CODE"
    } else {
        "CREDENTIAL"
    };

    Ok(EmailCheckResponse {
        existing: existing.is_some(),
        status,
    })
}
