use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{
    auth::{api_key::ApiKeyUser, session::SessionUser},
    entities::users,
    error::AppError,
    AppState,
};

pub struct AnyAuth(pub users::Model);

impl<S> FromRequestParts<S> for AnyAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(SessionUser(user)) = SessionUser::from_request_parts(parts, state).await {
            return Ok(AnyAuth(user));
        }

        if let Ok(ApiKeyUser(ctx)) = ApiKeyUser::from_request_parts(parts, state).await {
            return Ok(AnyAuth(ctx.user));
        }

        Err(AppError::Unauthorized)
    }
}

/// Optional authentication extractor.
///
/// Necessary because axum only generates `Option<T>` automatically when
/// `T::Rejection = Infallible`. Since `AnyAuth::Rejection = AppError`, the orphan
/// rule prevents implementing `FromRequestParts` for `Option<AnyAuth>` (foreign type).
/// This local newtype resolves both restrictions without modifying `AnyAuth`.
///
/// Usage in handler: `OptionalAnyAuth(user_opt): OptionalAnyAuth`
pub struct OptionalAnyAuth(pub Option<users::Model>);

impl<S> FromRequestParts<S> for OptionalAnyAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AnyAuth::from_request_parts(parts, state).await.ok().map(|a| a.0);
        Ok(OptionalAnyAuth(user))
    }
}
