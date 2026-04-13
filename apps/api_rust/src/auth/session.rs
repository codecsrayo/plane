use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    entities::{sessions, users},
    error::AppError,
    AppState,
};

pub const SESSION_COOKIE_NAME: &str = "session-id";
pub const ADMIN_SESSION_COOKIE_NAME: &str = "admin-session-id";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    App,
    Admin,
}

impl SessionKind {
    pub fn from_path(path: &str) -> Self {
        if path.contains("instances") {
            Self::Admin
        } else {
            Self::App
        }
    }

    pub fn primary_cookie(&self) -> &'static str {
        match self {
            Self::App => SESSION_COOKIE_NAME,
            Self::Admin => ADMIN_SESSION_COOKIE_NAME,
        }
    }

    pub fn fallback_cookie(&self) -> &'static str {
        match self {
            Self::App => ADMIN_SESSION_COOKIE_NAME,
            Self::Admin => SESSION_COOKIE_NAME,
        }
    }
}

pub struct SessionUser(pub users::Model);

impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let jar = CookieJar::from_headers(&parts.headers);
        let kind = SessionKind::from_path(parts.uri.path());

        let session_key = jar
            .get(kind.primary_cookie())
            .or_else(|| jar.get(kind.fallback_cookie()))
            .map(|c| c.value().to_owned())
            .ok_or(AppError::Unauthorized)?;

        if session_key.len() > 128 {
            return Err(AppError::Unauthorized);
        }

        let session = sessions::Entity::find_by_id(&session_key)
            .filter(sessions::Column::ExpireDate.gt(Utc::now()))
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        let user_id: uuid::Uuid = session
            .user_id
            .as_deref()
            .ok_or(AppError::Unauthorized)?
            .parse()
            .map_err(|_| AppError::Unauthorized)?;

        let user = users::Entity::find_by_id(user_id)
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        Ok(SessionUser(user))
    }
}

#[cfg(test)]
mod tests {
    use super::{SessionKind, ADMIN_SESSION_COOKIE_NAME, SESSION_COOKIE_NAME};

    #[test]
    fn session_kind_uses_admin_cookie_for_instances_paths() {
        let kind = SessionKind::from_path("/api/instances/acme/settings");

        assert_eq!(kind.primary_cookie(), ADMIN_SESSION_COOKIE_NAME);
        assert_eq!(kind.fallback_cookie(), SESSION_COOKIE_NAME);
    }

    #[test]
    fn session_kind_uses_app_cookie_elsewhere() {
        let kind = SessionKind::from_path("/api/workspaces/demo");

        assert_eq!(kind.primary_cookie(), SESSION_COOKIE_NAME);
        assert_eq!(kind.fallback_cookie(), ADMIN_SESSION_COOKIE_NAME);
    }
}
