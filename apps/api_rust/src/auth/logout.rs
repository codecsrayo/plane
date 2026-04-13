use axum::{
    extract::Form,
    extract::State,
    response::Redirect,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use sea_orm::EntityTrait;

use crate::{
    auth::{
        csrf::{is_valid_csrf, safe_redirect_target, CsrfForm, CSRF_COOKIE_NAME},
        session::{SessionKind, SessionUser, ADMIN_SESSION_COOKIE_NAME, SESSION_COOKIE_NAME},
    },
    entities::sessions,
    error::AppError,
    AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/sign-out",
    tag = "Auth",
    responses(
        (status = 303, description = "Sesión cerrada"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "CSRF inválido"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    SessionUser(_user): SessionUser,
    form: Form<CsrfForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    perform_logout(state, jar, form).await
}

#[utoipa::path(
    post,
    path = "/api/auth/spaces/sign-out",
    tag = "Auth",
    responses(
        (status = 303, description = "Sesión cerrada"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "CSRF inválido"),
    )
)]
pub async fn logout_space(
    State(state): State<AppState>,
    jar: CookieJar,
    SessionUser(_user): SessionUser,
    form: Form<CsrfForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    perform_logout(state, jar, form).await
}

async fn perform_logout(
    state: AppState,
    jar: CookieJar,
    form: Form<CsrfForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    let csrf_cookie = jar
        .get(CSRF_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned())
        .ok_or(AppError::Forbidden)?;

    if !is_valid_csrf(&form.csrfmiddlewaretoken, &csrf_cookie) {
        return Err(AppError::Forbidden);
    }

    let kind = SessionKind::App;
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
        .remove(expired(ADMIN_SESSION_COOKIE_NAME))
        .remove(expired(CSRF_COOKIE_NAME));

    let redirect_to = safe_redirect_target(form.next_path.as_deref());
    Ok((new_jar, Redirect::to(&redirect_to)))
}
