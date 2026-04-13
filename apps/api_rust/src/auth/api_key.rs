use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};

use crate::{
    auth::rate_limit::{apply_rate_limit, RateLimitState},
    entities::{api_tokens, users},
    error::AppError,
    AppState,
};

pub const API_KEY_HEADER: &str = "x-api-key";
pub const RATE_LIMIT_HUMAN: u32 = 60;
pub const RATE_LIMIT_SERVICE: u32 = 300;

pub struct ApiKeyContext {
    pub user: users::Model,
    pub token: api_tokens::Model,
}

pub struct ApiKeyUser(pub ApiKeyContext);

impl<S> FromRequestParts<S> for ApiKeyUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let raw_key = parts
            .headers
            .get(API_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .ok_or(AppError::Unauthorized)?;

        if raw_key.len() > 128 {
            return Err(AppError::Unauthorized);
        }

        let now = Utc::now();
        let token = api_tokens::Entity::find()
            .filter(api_tokens::Column::Token.eq(&raw_key))
            .filter(api_tokens::Column::IsActive.eq(true))
            .filter(api_tokens::Column::DeletedAt.is_null())
            .filter(
                Condition::any()
                    .add(api_tokens::Column::ExpiredAt.is_null())
                    .add(api_tokens::Column::ExpiredAt.gt(now)),
            )
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        let limit = if token.is_service {
            RATE_LIMIT_SERVICE
        } else {
            RATE_LIMIT_HUMAN
        };
        apply_rate_limit(rate_limit_state(&app_state), &raw_key, limit)?;

        let user = users::Entity::find_by_id(token.user_id)
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        let mut active: api_tokens::ActiveModel = token.clone().into();
        active.last_used = Set(Some(now.into()));
        if let Err(e) = active.update(&app_state.db).await {
            tracing::warn!(error = %e, "Failed to update last_used for api token");
        }

        Ok(ApiKeyUser(ApiKeyContext { user, token }))
    }
}

fn rate_limit_state(state: &AppState) -> &RateLimitState {
    state.rate_limit.as_ref()
}
