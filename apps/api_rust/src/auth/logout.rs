use axum::{extract::Form, extract::State, http::HeaderMap, response::Redirect};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

use crate::{
    auth::{
        csrf::{is_valid_csrf, safe_redirect_target, CsrfForm, CSRF_COOKIE_NAME},
        session::{SessionKind, SessionUser, ADMIN_SESSION_COOKIE_NAME, SESSION_COOKIE_NAME},
    },
    entities::{sessions, users},
    error::AppError,
    AppState,
};

#[utoipa::path(
    post,
    path = "/auth/sign-out",
    tag = "Auth",
    responses(
        (status = 303, description = "Signed out"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Invalid CSRF token"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    SessionUser(user): SessionUser,
    form: Form<CsrfForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    perform_logout(state, headers, jar, user, form, LogoutTarget::App).await
}

#[utoipa::path(
    post,
    path = "/auth/spaces/sign-out",
    tag = "Auth",
    responses(
        (status = 303, description = "Signed out"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Invalid CSRF token"),
    )
)]
pub async fn logout_space(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    SessionUser(user): SessionUser,
    form: Form<CsrfForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    perform_logout(state, headers, jar, user, form, LogoutTarget::Space).await
}

#[derive(Clone, Copy)]
enum LogoutTarget {
    App,
    Space,
}

async fn perform_logout(
    state: AppState,
    headers: HeaderMap,
    jar: CookieJar,
    user: users::Model,
    form: Form<CsrfForm>,
    target: LogoutTarget,
) -> Result<(CookieJar, Redirect), AppError> {
    let csrf_cookie = jar
        .get(CSRF_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned())
        .ok_or(AppError::Forbidden)?;

    if !is_valid_csrf(&form.csrfmiddlewaretoken, &csrf_cookie) {
        return Err(AppError::Forbidden);
    }

    let user_ip = extract_client_ip(&headers);
    let mut user_model: users::ActiveModel = user.into();
    user_model.last_logout_ip = Set(user_ip);
    user_model.last_logout_time = Set(Some(Utc::now().into()));
    user_model
        .update(&state.db)
        .await
        .map_err(AppError::Database)?;

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

    let redirect_to = match target {
        // Django's SignOutAuthEndpoint redirects to APP_BASE_URL (no path),
        // so after logout the user lands on "/" and not "/app/".
        LogoutTarget::App => state.config.app_base_url_only(),
        LogoutTarget::Space => {
            let next_path = safe_redirect_target(form.next_path.as_deref());
            space_redirect_url(&state, &next_path)
        }
    };
    Ok((new_jar, Redirect::to(&redirect_to)))
}

fn extract_client_ip(headers: &HeaderMap) -> String {
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

fn space_redirect_url(state: &AppState, next_path: &str) -> String {
    let base = state.config.space_base();

    if next_path == "/" {
        base
    } else {
        format!("{}{}", base.trim_end_matches('/'), next_path)
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_client_ip, space_redirect_url};
    use axum::http::{HeaderMap, HeaderValue};
    use std::sync::Arc;

    use crate::{auth::rate_limit::RateLimitState, config::Config, AppState};

    #[test]
    fn extract_client_ip_prefers_forwarded_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.8, 10.0.0.1"),
        );

        assert_eq!(extract_client_ip(&headers), "203.0.113.8");
    }


    #[tokio::test]
    async fn space_redirect_joins_safe_path() {
        let state = AppState {
            db: sea_orm::DatabaseConnection::Disconnected,
            redis: {
                let config = fred::prelude::Config::from_url("redis://127.0.0.1:6379")
                    .expect("redis config");
                fred::prelude::Builder::from_config(config)
                    .build_pool(1)
                    .expect("redis pool")
            },
            config: Arc::new(Config {
                database_url: String::new(),
                redis_url: String::new(),
                host: String::new(),
                port: 0,
                secret_key: String::new(),
                debug: false,
                web_url: None,
                app_base_url: Some("https://app.example.com".to_owned()),
                app_base_path: None,
                space_base_url: None,
                space_base_path: None,
                admin_base_url: None,
                admin_base_path: None,
                aws_s3_bucket: String::new(),
                aws_endpoint: String::new(),
                aws_access_key_id: "access-key".to_owned(),
                aws_secret_access_key: "secret-key".to_owned(),
                aws_region: "us-east-1".to_owned(),
                use_minio: false,
                llm_api_key: None,
                llm_provider: String::new(),
                llm_model: None,
                unsplash_access_key: None,
                cookie_domain: None,
                is_production: true,
                session_cookie_age: 604800,
                admin_session_cookie_age: 3600,
                cors_origins: vec![],
                email_host: None,
                email_port: 587,
                email_host_user: None,
                email_host_password: None,
                email_use_tls: false,
                email_use_ssl: false,
                email_from: String::new(),
                hard_delete_after_days: 30,
                unuploaded_asset_delete_days: 7,
            }),
            rate_limit: Arc::new(RateLimitState::default()),
            http: reqwest::Client::new(),
            pg_pool: sqlx::PgPool::connect_lazy("postgres://localhost/test_dummy")
                .expect("lazy pg pool for test"),
        };

        assert_eq!(
            space_redirect_url(&state, "/workspace/acme"),
            "https://app.example.com/spaces/workspace/acme"
        );
        assert_eq!(
            space_redirect_url(&state, "/"),
            "https://app.example.com/spaces/"
        );
    }
}
