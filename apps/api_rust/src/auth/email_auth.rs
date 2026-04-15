use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::Redirect,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use lettre::Address;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    auth::{
        responses::AuthErrorBody,
        session::{issue_session_cookie, replace_session_cookie, SessionSurface},
    },
    entities::{
        instances, profiles, user_notification_preferences, users, workspace_member_invites,
        workspace_members, workspaces,
    },
    error::AppError,
    utils::{
        django_sessions::email_display_name,
        instance_config::get_config_value,
        passwords::{make_password, verify_password},
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CredentialAuthForm {
    pub email: Option<String>,
    pub password: Option<String>,
    pub next_path: Option<String>,
}

#[utoipa::path(
    post,
    path = "/auth/sign-in",
    tag = "Auth",
    request_body(
        content = CredentialAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Signed in"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<CredentialAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    authenticate_existing_user(&state, &headers, jar, form, SessionSurface::App, false).await
}

#[utoipa::path(
    post,
    path = "/auth/sign-up",
    tag = "Auth",
    request_body(
        content = CredentialAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Signed up"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn sign_up(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<CredentialAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    create_and_authenticate_user(&state, &headers, jar, form, SessionSurface::App).await
}

#[utoipa::path(
    post,
    path = "/auth/spaces/sign-in",
    tag = "Auth",
    request_body(
        content = CredentialAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Space signed in"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn sign_in_space(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<CredentialAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    authenticate_existing_user(&state, &headers, jar, form, SessionSurface::Space, true).await
}

#[utoipa::path(
    post,
    path = "/auth/spaces/sign-up",
    tag = "Auth",
    request_body(
        content = CredentialAuthForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Space signed up"),
        (status = 302, description = "Redirect with authentication error", body = AuthErrorBody),
    )
)]
pub async fn sign_up_space(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<CredentialAuthForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    create_and_authenticate_user(&state, &headers, jar, form, SessionSurface::Space).await
}

async fn authenticate_existing_user(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    form: CredentialAuthForm,
    surface: SessionSurface,
    is_space: bool,
) -> Result<(CookieJar, Redirect), AppError> {
    if let Some(url) = ensure_instance_ready(state, &form.next_path, is_space).await? {
        return Ok((jar, Redirect::to(&url)));
    }
    if let Some(url) = ensure_email_password_enabled(state, &form.next_path, is_space).await? {
        return Ok((jar, Redirect::to(&url)));
    }

    let Some(email) = form.email.as_deref().map(normalize_email) else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5070,
            "REQUIRED_EMAIL_PASSWORD_SIGN_IN",
        ));
    };
    let Some(password) = form.password.as_deref() else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5070,
            "REQUIRED_EMAIL_PASSWORD_SIGN_IN",
        ));
    };
    if !is_valid_email(&email) {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5075,
            "INVALID_EMAIL_SIGN_IN",
        ));
    }

    let Some(user) = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5060,
            "USER_DOES_NOT_EXIST",
        ));
    };

    ensure_profile_exists(state, &user).await?;
    if !verify_password(password, &user.password) {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5065,
            "AUTHENTICATION_FAILED_SIGN_IN",
        ));
    }

    let user = update_login_metadata(state, user, headers).await?;
    let jar = replace_session_cookie(state, headers, jar, &user, surface).await?;
    let redirect_to = success_redirect(state, &user, surface, form.next_path.as_deref()).await?;

    Ok((jar, Redirect::to(&redirect_to)))
}

async fn create_and_authenticate_user(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    form: CredentialAuthForm,
    surface: SessionSurface,
) -> Result<(CookieJar, Redirect), AppError> {
    let is_space = matches!(surface, SessionSurface::Space);
    if let Some(url) = ensure_instance_ready(state, &form.next_path, is_space).await? {
        return Ok((jar, Redirect::to(&url)));
    }
    if let Some(url) = ensure_email_password_enabled(state, &form.next_path, is_space).await? {
        return Ok((jar, Redirect::to(&url)));
    }

    let Some(email) = form.email.as_deref().map(normalize_email) else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5040,
            "REQUIRED_EMAIL_PASSWORD_SIGN_UP",
        ));
    };
    let Some(password) = form.password.as_deref() else {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5040,
            "REQUIRED_EMAIL_PASSWORD_SIGN_UP",
        ));
    };
    if !is_valid_email(&email) {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5045,
            "INVALID_EMAIL_SIGN_UP",
        ));
    }

    if users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some()
    {
        return Ok(redirect_error(
            state,
            surface,
            form.next_path.as_deref(),
            5030,
            "USER_ALREADY_EXIST",
        ));
    }

    if let Some(url) = ensure_signup_allowed(state, &email, &form.next_path, is_space).await? {
        return Ok((jar, Redirect::to(&url)));
    }

    let now = Utc::now();
    let password_hash = make_password(password).map_err(AppError::Internal)?;
    let user_id = uuid::Uuid::new_v4();
    let profile_id = uuid::Uuid::new_v4();
    let preference_id = uuid::Uuid::new_v4();
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let color = format!("#{}", &uuid::Uuid::new_v4().simple().to_string()[..6]);
    let display_name = email_display_name(&email).into_owned();

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    users::ActiveModel {
        password: Set(password_hash),
        last_login: Set(Some(now.into())),
        id: Set(user_id),
        username: Set(uuid::Uuid::new_v4().simple().to_string()),
        mobile_number: Set(None),
        email: Set(Some(email.clone())),
        first_name: Set(String::new()),
        last_name: Set(String::new()),
        avatar: Set(String::new()),
        date_joined: Set(now.into()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        last_location: Set(String::new()),
        created_location: Set(String::new()),
        is_superuser: Set(false),
        is_managed: Set(false),
        is_password_expired: Set(false),
        is_active: Set(true),
        is_staff: Set(false),
        is_email_verified: Set(false),
        is_password_autoset: Set(false),
        token: Set(token),
        user_timezone: Set("UTC".to_owned()),
        last_active: Set(Some(now.into())),
        last_login_time: Set(Some(now.into())),
        last_logout_time: Set(None),
        last_login_ip: Set(crate::auth::session::extract_client_ip(headers)),
        last_logout_ip: Set(String::new()),
        last_login_medium: Set("email".to_owned()),
        last_login_uagent: Set(headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned()),
        token_updated_at: Set(Some(now.into())),
        is_bot: Set(false),
        cover_image: Set(None),
        display_name: Set(display_name),
        avatar_asset_id: Set(None),
        cover_image_asset_id: Set(None),
        bot_type: Set(None),
        is_email_valid: Set(false),
        masked_at: Set(None),
        is_password_reset_required: Set(false),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    profiles::ActiveModel {
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        id: Set(profile_id),
        theme: Set(json!({})),
        is_tour_completed: Set(false),
        onboarding_step: Set(json!({
            "profile_complete": false,
            "workspace_create": false,
            "workspace_invite": false,
            "workspace_join": false
        })),
        use_case: Set(None),
        role: Set(None),
        is_onboarded: Set(false),
        last_workspace_id: Set(None),
        billing_address_country: Set("INDIA".to_owned()),
        billing_address: Set(None),
        has_billing_address: Set(false),
        company_name: Set(String::new()),
        user_id: Set(user_id),
        is_mobile_onboarded: Set(false),
        mobile_onboarding_step: Set(json!({
            "profile_complete": false,
            "workspace_create": false,
            "workspace_join": false
        })),
        mobile_timezone_auto_set: Set(false),
        language: Set("en".to_owned()),
        is_smooth_cursor_enabled: Set(false),
        start_of_the_week: Set(0),
        is_app_rail_docked: Set(true),
        background_color: Set(color),
        goals: Set(json!({})),
        has_marketing_email_consent: Set(false),
        is_navigation_tour_completed: Set(false),
        is_subscribed_to_changelog: Set(false),
        notification_view_mode: Set("full".to_owned()),
        product_tour: Set(json!({
            "work_items": false,
            "cycles": false,
            "modules": false,
            "intake": false,
            "pages": false
        })),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    user_notification_preferences::ActiveModel {
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        id: Set(preference_id),
        property_change: Set(true),
        state_change: Set(true),
        comment: Set(true),
        mention: Set(true),
        issue_completed: Set(true),
        created_by_id: Set(None),
        project_id: Set(None),
        updated_by_id: Set(None),
        user_id: Set(user_id),
        workspace_id: Set(None),
        deleted_at: Set(None),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    let user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;
    let jar = issue_session_cookie(state, headers, jar, &user, surface).await?;
    let redirect_to = success_redirect(state, &user, surface, form.next_path.as_deref()).await?;

    Ok((jar, Redirect::to(&redirect_to)))
}

async fn update_login_metadata(
    state: &AppState,
    user: users::Model,
    headers: &HeaderMap,
) -> Result<users::Model, AppError> {
    let now = Utc::now();
    let mut user_model: users::ActiveModel = user.into();
    user_model.last_login = Set(Some(now.into()));
    user_model.last_active = Set(Some(now.into()));
    user_model.last_login_time = Set(Some(now.into()));
    user_model.last_login_ip = Set(crate::auth::session::extract_client_ip(headers));
    user_model.last_login_medium = Set("email".to_owned());
    user_model.last_login_uagent = Set(headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned());
    user_model.token_updated_at = Set(Some(now.into()));
    user_model.token = Set(format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    ));
    user_model.is_active = Set(true);
    user_model.updated_at = Set(now.into());
    user_model
        .update(&state.db)
        .await
        .map_err(AppError::Database)
}

async fn ensure_instance_ready(
    state: &AppState,
    next_path: &Option<String>,
    is_space: bool,
) -> Result<Option<String>, AppError> {
    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;
    if !instance.is_some_and(|instance| instance.is_setup_done) {
        let surface = if is_space {
            SessionSurface::Space
        } else {
            SessionSurface::App
        };
        return Ok(Some(redirect_url(
            state,
            surface,
            next_path.as_deref(),
            Some((5000, "INSTANCE_NOT_CONFIGURED")),
        )));
    }
    Ok(None)
}

async fn ensure_email_password_enabled(
    state: &AppState,
    next_path: &Option<String>,
    is_space: bool,
) -> Result<Option<String>, AppError> {
    let enabled = get_config_value(
        state,
        "ENABLE_EMAIL_PASSWORD",
        std::env::var("ENABLE_EMAIL_PASSWORD").ok().as_deref(),
    )
    .await?
    .is_none_or(|value| value != "0");
    if enabled {
        Ok(None)
    } else {
        let surface = if is_space {
            SessionSurface::Space
        } else {
            SessionSurface::App
        };
        Ok(Some(redirect_url(
            state,
            surface,
            next_path.as_deref(),
            Some((5056, "EMAIL_PASSWORD_AUTHENTICATION_DISABLED")),
        )))
    }
}

pub(crate) async fn ensure_signup_allowed(
    state: &AppState,
    email: &str,
    next_path: &Option<String>,
    is_space: bool,
) -> Result<Option<String>, AppError> {
    let enabled = get_config_value(
        state,
        "ENABLE_SIGNUP",
        std::env::var("ENABLE_SIGNUP").ok().as_deref().or(Some("1")),
    )
    .await?
    .is_none_or(|value| value != "0");
    if enabled {
        return Ok(None);
    }

    let invited = workspace_member_invites::Entity::find()
        .active()
        .filter(workspace_member_invites::Column::Email.eq(email.to_owned()))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();
    if invited {
        Ok(None)
    } else {
        let surface = if is_space {
            SessionSurface::Space
        } else {
            SessionSurface::App
        };
        Ok(Some(redirect_url(
            state,
            surface,
            next_path.as_deref(),
            Some((5015, "SIGNUP_DISABLED")),
        )))
    }
}

pub(crate) fn redirect_error(
    state: &AppState,
    surface: SessionSurface,
    next_path: Option<&str>,
    code: i32,
    message: &'static str,
) -> (CookieJar, Redirect) {
    (
        CookieJar::new(),
        Redirect::to(&redirect_url(
            state,
            surface,
            next_path,
            Some((code, message)),
        )),
    )
}

async fn success_redirect(
    state: &AppState,
    user: &users::Model,
    surface: SessionSurface,
    next_path: Option<&str>,
) -> Result<String, AppError> {
    match surface {
        SessionSurface::Space => {
            let path = safe_next_path(next_path).unwrap_or_else(|| "/".to_owned());
            Ok(format!(
                "{}{}",
                space_base(state).trim_end_matches('/'),
                path
            ))
        }
        SessionSurface::App | SessionSurface::Admin => match safe_next_path(next_path) {
            Some(path) => Ok(format!("{}{}", app_base(state).trim_end_matches('/'), path)),
            None => Ok(format!(
                "{}/{}",
                app_base(state).trim_end_matches('/'),
                app_default_path(state, user).await?
            )),
        },
    }
}

fn redirect_url(
    state: &AppState,
    surface: SessionSurface,
    next_path: Option<&str>,
    error: Option<(i32, &'static str)>,
) -> String {
    let base = match surface {
        SessionSurface::Space => space_base(state),
        SessionSurface::App | SessionSurface::Admin => app_base(state),
    };
    let mut query = Vec::new();
    if let Some(path) = safe_next_path(next_path) {
        query.push(format!("next_path={path}"));
    }
    if let Some((code, message)) = error {
        query.push(format!("error_code={code}"));
        query.push(format!("error_message={message}"));
    }
    if query.is_empty() {
        base
    } else {
        format!("{}/?{}", base.trim_end_matches('/'), query.join("&"))
    }
}

fn app_base(state: &AppState) -> String {
    state
        .config
        .app_base_url
        .clone()
        .or_else(|| state.config.web_url.clone())
        .unwrap_or_else(|| "/".to_owned())
}

pub(crate) fn space_base(state: &AppState) -> String {
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

pub(crate) fn safe_next_path(next_path: Option<&str>) -> Option<String> {
    let path = next_path?.trim();
    if path.is_empty() {
        return None;
    }
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains("..")
        || path.contains('\\')
    {
        return None;
    }
    Some(path.to_owned())
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn is_valid_email(email: &str) -> bool {
    email.parse::<Address>().is_ok()
}

pub(crate) async fn app_default_path(
    state: &AppState,
    user: &users::Model,
) -> Result<String, AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(profile) = profile else {
        return Ok("onboarding".to_owned());
    };

    if !profile.is_onboarded {
        return Ok("onboarding".to_owned());
    }

    if let Some(last_workspace_id) = profile.last_workspace_id {
        let membership = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(last_workspace_id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .filter(workspace_members::Column::IsActive.eq(true))
            .filter(workspace_members::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;

        if membership.is_some() {
            let workspace = workspaces::Entity::find_by_id(last_workspace_id)
                .filter(workspaces::Column::DeletedAt.is_null())
                .one(&state.db)
                .await
                .map_err(AppError::Database)?;
            if let Some(workspace) = workspace {
                return Ok(workspace.slug);
            }
        }
    }

    let fallback_member = workspace_members::Entity::find()
        .filter(workspace_members::Column::MemberId.eq(user.id))
        .filter(workspace_members::Column::IsActive.eq(true))
        .filter(workspace_members::Column::DeletedAt.is_null())
        .order_by_asc(workspace_members::Column::CreatedAt)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;
    if let Some(member) = fallback_member {
        let workspace = workspaces::Entity::find_by_id(member.workspace_id)
            .filter(workspaces::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .map_err(AppError::Database)?;
        if let Some(workspace) = workspace {
            return Ok(workspace.slug);
        }
    }

    if let Some(email) = user.email.as_deref() {
        let has_invites = workspace_member_invites::Entity::find()
            .active()
            .filter(workspace_member_invites::Column::Email.eq(email))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .is_some();
        if has_invites {
            return Ok("invitations".to_owned());
        }
    }

    Ok("create-workspace".to_owned())
}

pub(crate) async fn ensure_profile_exists(
    state: &AppState,
    user: &users::Model,
) -> Result<(), AppError> {
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;
    if profile.is_some() {
        return Ok(());
    }

    let now = Utc::now();
    profiles::ActiveModel {
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        id: Set(uuid::Uuid::new_v4()),
        theme: Set(json!({})),
        is_tour_completed: Set(false),
        onboarding_step: Set(json!({
            "profile_complete": false,
            "workspace_create": false,
            "workspace_invite": false,
            "workspace_join": false
        })),
        use_case: Set(None),
        role: Set(None),
        is_onboarded: Set(false),
        last_workspace_id: Set(None),
        billing_address_country: Set("INDIA".to_owned()),
        billing_address: Set(None),
        has_billing_address: Set(false),
        company_name: Set(String::new()),
        user_id: Set(user.id),
        is_mobile_onboarded: Set(false),
        mobile_onboarding_step: Set(json!({
            "profile_complete": false,
            "workspace_create": false,
            "workspace_join": false
        })),
        mobile_timezone_auto_set: Set(false),
        language: Set("en".to_owned()),
        is_smooth_cursor_enabled: Set(false),
        start_of_the_week: Set(0),
        is_app_rail_docked: Set(true),
        background_color: Set(format!(
            "#{}",
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        )),
        goals: Set(json!({})),
        has_marketing_email_consent: Set(false),
        is_navigation_tour_completed: Set(false),
        is_subscribed_to_changelog: Set(false),
        notification_view_mode: Set("full".to_owned()),
        product_tour: Set(json!({
            "work_items": false,
            "cycles": false,
            "modules": false,
            "intake": false,
            "pages": false
        })),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}
