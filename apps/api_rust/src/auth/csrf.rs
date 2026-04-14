use axum::{extract::State, http::HeaderMap, Json};
use openssl::memcmp;
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;

pub const CSRF_COOKIE_NAME: &str = "csrftoken";

#[derive(Debug, Deserialize, ToSchema)]
pub struct CsrfForm {
    pub csrfmiddlewaretoken: String,
    pub next_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CsrfTokenResponse {
    pub csrf_token: String,
}

#[utoipa::path(
    get,
    path = "/api/auth/get-csrf-token",
    tag = "Auth",
    responses(
        (status = 200, description = "CSRF token issued", body = CsrfTokenResponse),
    )
)]
pub async fn get_csrf_token(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Json<CsrfTokenResponse>) {
    let token = Uuid::new_v4().to_string();
    let cookie = Cookie::build((CSRF_COOKIE_NAME.to_owned(), token.clone()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.config.is_production)
        .path("/")
        .build();

    (
        jar.add(cookie),
        Json(CsrfTokenResponse { csrf_token: token }),
    )
}

/// Validates a CSRF token using constant-time comparison to prevent timing attacks.
///
/// Direct string equality (`==`) short-circuits on the first differing byte,
/// leaking timing information that could be exploited to brute-force the token.
/// `openssl::memcmp::eq` always compares all bytes in constant time.
pub fn is_valid_csrf(form_token: &str, cookie_token: &str) -> bool {
    if form_token.is_empty() {
        return false;
    }
    // Length check is not secret — both tokens are UUID v4 (36 bytes).
    // Returning early on length mismatch is safe and avoids padding concerns.
    if form_token.len() != cookie_token.len() {
        return false;
    }
    memcmp::eq(form_token.as_bytes(), cookie_token.as_bytes())
}

pub fn is_valid_csrf_header(headers: &HeaderMap, cookie_token: &str) -> bool {
    headers
        .get("x-csrftoken")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|header_token| is_valid_csrf(header_token, cookie_token))
}

pub fn safe_redirect_target(next_path: Option<&str>) -> String {
    match next_path {
        Some(path) if path.starts_with('/') && !path.starts_with("//") => path.to_owned(),
        _ => "/".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_valid_csrf, safe_redirect_target};

    #[test]
    fn csrf_tokens_must_match() {
        assert!(is_valid_csrf("abc", "abc"));
        assert!(!is_valid_csrf("abc", "def"));
        assert!(!is_valid_csrf("", "abc"));
    }

    #[test]
    fn redirect_target_stays_local() {
        assert_eq!(
            safe_redirect_target(Some("/workspace/demo")),
            "/workspace/demo"
        );
        assert_eq!(safe_redirect_target(Some("//evil.com")), "/");
        assert_eq!(safe_redirect_target(Some("https://evil.com")), "/");
        assert_eq!(safe_redirect_target(None), "/");
    }
}
