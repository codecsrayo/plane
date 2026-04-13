use axum::{
    extract::State,
    http::{StatusCode, Uri},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use sea_orm::EntityTrait;

use crate::{
    auth::session::{SessionKind, SessionUser, ADMIN_SESSION_COOKIE_NAME, SESSION_COOKIE_NAME},
    entities::sessions,
    error::AppError,
    AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/sign-out",
    tag = "Auth",
    responses(
        (status = 204, description = "Sesión cerrada"),
        (status = 401, description = "No autenticado"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    uri: Uri,
    jar: CookieJar,
    SessionUser(_user): SessionUser,
) -> Result<(CookieJar, StatusCode), AppError> {
    let kind = SessionKind::from_path(uri.path());
    let session_key = jar
        .get(kind.primary_cookie())
        .or_else(|| jar.get(kind.fallback_cookie()))
        .map(|c| c.value().to_owned());

    if let Some(key) = session_key {
        sessions::Entity::delete_by_id(&key)
            .exec(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    let expired = |name: &str| {
        Cookie::build((name.to_owned(), String::new()))
            .max_age(time::Duration::ZERO)
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .build()
    };

    let new_jar = jar
        .remove(expired(SESSION_COOKIE_NAME))
        .remove(expired(ADMIN_SESSION_COOKIE_NAME));

    Ok((new_jar, StatusCode::NO_CONTENT))
}
