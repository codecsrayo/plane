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

/// Permite usar `Option<AnyAuth>` como extractor en handlers de axum.
///
/// axum solo genera `Option<T>` automáticamente cuando `T::Rejection = Infallible`.
/// Como `AnyAuth::Rejection = AppError`, se requiere esta implementación explícita
/// para que el extractor sea opcional sin rechazar la solicitud.
impl<S> FromRequestParts<S> for Option<AnyAuth>
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(AnyAuth::from_request_parts(parts, state).await.ok())
    }
}
