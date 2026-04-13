use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, HeaderMap},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;

use crate::{
    entities::{sessions, users},
    error::AppError,
    utils::django_sessions::{decode_session, encode_session, session_auth_hash, AUTH_BACKEND},
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

        let payload = decode_session(&session.session_data, &app_state.config.secret_key)
            .map_err(|_| AppError::Unauthorized)?;
        let user_id: uuid::Uuid = payload
            .get("_auth_user_id")
            .and_then(|value| value.as_str())
            .ok_or(AppError::Unauthorized)?
            .parse()
            .map_err(|_| AppError::Unauthorized)?;
        let backend = payload
            .get("_auth_user_backend")
            .and_then(|value| value.as_str())
            .ok_or(AppError::Unauthorized)?;
        if backend != AUTH_BACKEND {
            return Err(AppError::Unauthorized);
        }

        let user = users::Entity::find_by_id(user_id)
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        let expected_hash = session_auth_hash(&user.password, &app_state.config.secret_key)
            .map_err(AppError::Internal)?;
        let session_hash = payload
            .get("_auth_user_hash")
            .and_then(|value| value.as_str())
            .ok_or(AppError::Unauthorized)?;
        if session_hash != expected_hash {
            return Err(AppError::Unauthorized);
        }

        Ok(SessionUser(user))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSurface {
    App,
    Space,
    Admin,
}

impl SessionSurface {
    fn cookie_name(self) -> &'static str {
        match self {
            Self::Admin => ADMIN_SESSION_COOKIE_NAME,
            Self::App | Self::Space => SESSION_COOKIE_NAME,
        }
    }

    fn max_age_seconds(self, state: &AppState) -> i64 {
        match self {
            Self::Admin => state.config.admin_session_cookie_age,
            Self::App | Self::Space => state.config.session_cookie_age,
        }
    }

    fn device_domain(self, state: &AppState) -> String {
        match self {
            Self::App => state
                .config
                .app_base_url
                .clone()
                .or_else(|| state.config.web_url.clone())
                .unwrap_or_else(|| "/".to_owned()),
            Self::Space => {
                let base = state
                    .config
                    .space_base_url
                    .clone()
                    .or_else(|| state.config.web_url.clone())
                    .or_else(|| state.config.app_base_url.clone())
                    .unwrap_or_else(|| "/spaces/".to_owned());
                if base.ends_with("/spaces/") {
                    base
                } else if base.ends_with("/spaces") {
                    format!("{base}/")
                } else {
                    format!("{}/spaces/", base.trim_end_matches('/'))
                }
            }
            Self::Admin => state
                .config
                .admin_base_url
                .clone()
                .or_else(|| state.config.web_url.clone())
                .or_else(|| state.config.app_base_url.clone())
                .unwrap_or_else(|| "/god-mode/".to_owned()),
        }
    }
}

pub async fn issue_session_cookie(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    user: &users::Model,
    surface: SessionSurface,
) -> Result<CookieJar, AppError> {
    let session_key = new_session_key();
    let expire_at = Utc::now() + Duration::seconds(surface.max_age_seconds(state));
    let auth_hash =
        session_auth_hash(&user.password, &state.config.secret_key).map_err(AppError::Internal)?;
    let mut payload = serde_json::Map::new();
    payload.insert("_auth_user_id".to_owned(), json!(user.id.to_string()));
    payload.insert("_auth_user_backend".to_owned(), json!(AUTH_BACKEND));
    payload.insert("_auth_user_hash".to_owned(), json!(auth_hash));
    payload.insert(
        "device_info".to_owned(),
        json!({
            "user_agent": headers
                .get(axum::http::header::USER_AGENT)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default(),
            "ip_address": extract_client_ip(headers),
            "domain": surface.device_domain(state),
        }),
    );
    let session_data =
        encode_session(&payload, &state.config.secret_key).map_err(AppError::Internal)?;

    sessions::ActiveModel {
        session_data: Set(session_data),
        expire_date: Set(expire_at.into()),
        device_info: Set(Some(
            payload
                .get("device_info")
                .cloned()
                .unwrap_or_else(|| json!({})),
        )),
        session_key: Set(session_key.clone()),
        user_id: Set(Some(user.id.to_string())),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    let mut builder = Cookie::build((surface.cookie_name().to_owned(), session_key))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.config.is_production)
        .path("/");

    if let Some(domain) = state.config.cookie_domain.clone() {
        builder = builder.domain(domain);
    }

    Ok(jar.add(
        builder
            .max_age(time::Duration::seconds(surface.max_age_seconds(state)))
            .build(),
    ))
}

pub async fn replace_session_cookie(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    user: &users::Model,
    surface: SessionSurface,
) -> Result<CookieJar, AppError> {
    let current_key = jar
        .get(surface.cookie_name())
        .or_else(|| {
            jar.get(match surface {
                SessionSurface::Admin => SESSION_COOKIE_NAME,
                SessionSurface::App | SessionSurface::Space => ADMIN_SESSION_COOKIE_NAME,
            })
        })
        .map(|cookie| cookie.value().to_owned());

    if let Some(session_key) = current_key {
        sessions::Entity::delete_by_id(session_key)
            .exec(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    issue_session_cookie(state, headers, jar, user, surface).await
}

pub fn extract_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default()
        .to_owned()
}

fn new_session_key() -> String {
    [
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    ]
    .into_iter()
    .map(|value| value.simple().to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{extract_client_ip, SessionKind, ADMIN_SESSION_COOKIE_NAME, SESSION_COOKIE_NAME};
    use axum::http::{HeaderMap, HeaderValue};

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

    #[test]
    fn client_ip_prefers_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.10, 10.0.0.1"),
        );

        assert_eq!(extract_client_ip(&headers), "203.0.113.10");
    }
}
