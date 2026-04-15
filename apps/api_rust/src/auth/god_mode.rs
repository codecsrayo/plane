// src/auth/god_mode.rs
//! God Mode authentication — espejo de `plane/license/api/views/admin.py`.
//!
//! Tres endpoints:
//!   POST /api/instances/admins/sign-up/   — primer admin + is_setup_done
//!   POST /api/instances/admins/sign-in/   — login de admin existente
//!   POST /api/instances/admins/sign-out/  — logout de admin

use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use lettre::Address;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    auth::{
        csrf::{is_valid_csrf, CsrfForm, CSRF_COOKIE_NAME},
        session::{
            extract_client_ip, issue_session_cookie, replace_session_cookie,
            SessionSurface, ADMIN_SESSION_COOKIE_NAME, SESSION_COOKIE_NAME,
        },
    },
    entities::{
        instance_admins, instances, profiles, user_notification_preferences, users,
    },
    error::AppError,
    utils::{
        django_sessions::email_display_name,
        passwords::{make_password, verify_password},
        soft_delete::SoftDeleteExt,
    },
    AppState,
};

// ─── helpers ────────────────────────────────────────────────────────────────

fn admin_base(state: &AppState) -> String {
    state
        .config
        .admin_base_url
        .clone()
        .or_else(|| state.config.app_base_url.clone())
        .or_else(|| state.config.web_url.clone())
        .unwrap_or_else(|| "/god-mode/".to_owned())
}

/// Redirige al panel de God-Mode con query params de error.
fn admin_error(state: &AppState, code: u32, message: &str) -> (CookieJar, Redirect) {
    let base = admin_base(state);
    let url = format!(
        "{}?error_code={}&error_message={}",
        base.trim_end_matches('/'),
        code,
        message,
    );
    (CookieJar::new(), Redirect::to(&url))
}

/// Redirige al panel principal de God-Mode tras login/setup exitoso.
fn admin_success(state: &AppState, path: &str) -> String {
    format!(
        "{}/{}",
        admin_base(state).trim_end_matches('/'),
        path.trim_start_matches('/'),
    )
}

fn is_valid_email(email: &str) -> bool {
    email.parse::<Address>().is_ok()
}

/// Comprueba la fortaleza de la contraseña con zxcvbn (score mínimo: 3).
fn is_password_strong(password: &str) -> bool {
    let estimate = zxcvbn::zxcvbn(password, &[]);
    estimate.score() >= zxcvbn::Score::Three
}

// ─── DTOs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminSignUpForm {
    pub email:                Option<String>,
    pub password:             Option<String>,
    pub first_name:           Option<String>,
    pub last_name:            Option<String>,
    pub company_name:         Option<String>,
    pub is_telemetry_enabled: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminSignInForm {
    pub email:    Option<String>,
    pub password: Option<String>,
}

// ─── POST /api/instances/admins/sign-up/ ─────────────────────────────────────
//
// Flujo (espeja InstanceAdminSignUpEndpoint en Django):
//   1. Instance existe y NO tiene admin aún.
//   2. Crea User + Profile + NotificationPreferences en una transacción.
//   3. Crea InstanceAdmin.
//   4. Setea instance.is_setup_done = true.
//   5. Emite cookie de sesión Admin y redirige a /general/.

#[utoipa::path(
    post,
    path = "/api/instances/admins/sign-up/",
    tag = "Instance",
    request_body(
        content = AdminSignUpForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Admin created, redirects to God Mode"),
        (status = 302, description = "Redirect with error params"),
    )
)]
pub async fn admin_sign_up(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<AdminSignUpForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    // 1. Verificar que la instancia existe
    let Some(instance) = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    else {
        return Ok(admin_error(&state, 5000, "INSTANCE_NOT_CONFIGURED"));
    };

    // 2. Solo se puede registrar un admin una vez
    let admin_exists = instance_admins::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();
    if admin_exists {
        return Ok(admin_error(&state, 5150, "ADMIN_ALREADY_EXIST"));
    }

    // 3. Validar campos obligatorios
    let email = form.email.as_deref().unwrap_or("").trim().to_lowercase();
    let password = form.password.as_deref().unwrap_or("");
    let first_name = form.first_name.as_deref().unwrap_or("").trim().to_owned();
    let last_name = form.last_name.as_deref().unwrap_or("").trim().to_owned();
    let company_name = form.company_name.as_deref().unwrap_or("").trim().to_owned();
    let telemetry_enabled = form
        .is_telemetry_enabled
        .as_deref()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

    if email.is_empty() || password.is_empty() || first_name.is_empty() {
        return Ok(admin_error(
            &state,
            5155,
            "REQUIRED_ADMIN_EMAIL_PASSWORD_FIRST_NAME",
        ));
    }

    if !is_valid_email(&email) {
        return Ok(admin_error(&state, 5160, "INVALID_ADMIN_EMAIL"));
    }

    // 4. El correo no debe existir ya como usuario
    let user_exists = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();
    if user_exists {
        return Ok(admin_error(&state, 5180, "ADMIN_USER_ALREADY_EXIST"));
    }

    // 5. Fortaleza de contraseña (zxcvbn score >= 3)
    if !is_password_strong(password) {
        return Ok(admin_error(&state, 5021, "PASSWORD_TOO_WEAK"));
    }

    // 6. Crear User + Profile + NotificationPreferences en transacción
    let now = Utc::now();
    let user_id = uuid::Uuid::new_v4();
    let profile_id = uuid::Uuid::new_v4();
    let preference_id = uuid::Uuid::new_v4();
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let bg_color = format!("#{}", &uuid::Uuid::new_v4().simple().to_string()[..6]);
    let display_name = email_display_name(&email).into_owned();
    let password_hash = make_password(password).map_err(AppError::Internal)?;

    let txn = state.db.begin().await.map_err(AppError::Database)?;

    users::ActiveModel {
        id: Set(user_id),
        username: Set(uuid::Uuid::new_v4().simple().to_string()),
        first_name: Set(first_name.clone()),
        last_name: Set(last_name),
        email: Set(Some(email.clone())),
        password: Set(password_hash),
        is_superuser: Set(false),
        is_staff: Set(false),
        is_active: Set(true),
        is_managed: Set(false),
        is_password_expired: Set(false),
        is_email_verified: Set(false),
        is_password_autoset: Set(false),
        is_bot: Set(false),
        is_email_valid: Set(false),
        token: Set(token),
        user_timezone: Set("UTC".to_owned()),
        display_name: Set(display_name),
        avatar: Set(String::new()),
        mobile_number: Set(None),
        last_location: Set(String::new()),
        created_location: Set(String::new()),
        last_active: Set(Some(now.into())),
        last_login_time: Set(Some(now.into())),
        last_login_ip: Set(extract_client_ip(&headers)),
        last_login_medium: Set("email".to_owned()),
        last_login_uagent: Set(headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned()),
        last_logout_time: Set(None),
        last_logout_ip: Set(String::new()),
        token_updated_at: Set(Some(now.into())),
        last_login: Set(Some(now.into())),
        date_joined: Set(now.into()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        cover_image: Set(None),
        avatar_asset_id: Set(None),
        cover_image_asset_id: Set(None),
        bot_type: Set(None),
        masked_at: Set(None),
        is_password_reset_required: Set(false),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    profiles::ActiveModel {
        id: Set(profile_id),
        user_id: Set(user_id),
        company_name: Set(company_name.clone()),
        is_onboarded: Set(false),
        is_tour_completed: Set(false),
        is_mobile_onboarded: Set(false),
        is_navigation_tour_completed: Set(false),
        is_subscribed_to_changelog: Set(false),
        is_smooth_cursor_enabled: Set(false),
        is_app_rail_docked: Set(true),
        has_marketing_email_consent: Set(false),
        has_billing_address: Set(false),
        mobile_timezone_auto_set: Set(false),
        start_of_the_week: Set(0),
        theme: Set(json!({})),
        onboarding_step: Set(json!({
            "profile_complete": false,
            "workspace_create": false,
            "workspace_invite": false,
            "workspace_join": false
        })),
        mobile_onboarding_step: Set(json!({
            "profile_complete": false,
            "workspace_create": false,
            "workspace_join": false
        })),
        product_tour: Set(json!({
            "work_items": false,
            "cycles": false,
            "modules": false,
            "intake": false,
            "pages": false
        })),
        goals: Set(json!({})),
        use_case: Set(None),
        role: Set(None),
        last_workspace_id: Set(None),
        billing_address: Set(None),
        billing_address_country: Set("INDIA".to_owned()),
        background_color: Set(bg_color),
        language: Set("en".to_owned()),
        notification_view_mode: Set("full".to_owned()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    user_notification_preferences::ActiveModel {
        id: Set(preference_id),
        user_id: Set(user_id),
        property_change: Set(true),
        state_change: Set(true),
        comment: Set(true),
        mention: Set(true),
        issue_completed: Set(true),
        project_id: Set(None),
        workspace_id: Set(None),
        created_by_id: Set(None),
        updated_by_id: Set(None),
        deleted_at: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&txn)
    .await
    .map_err(AppError::Database)?;

    txn.commit().await.map_err(AppError::Database)?;

    // 7. Crear InstanceAdmin
    let admin_id = uuid::Uuid::new_v4();
    instance_admins::ActiveModel {
        id: Set(admin_id),
        instance_id: Set(instance.id),
        user_id: Set(Some(user_id)),
        role: Set(20),
        is_verified: Set(false),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // 8. Marcar instancia como configurada
    let mut active_instance: instances::ActiveModel = instance.into();
    active_instance.is_setup_done = Set(true);
    active_instance.is_telemetry_enabled = Set(telemetry_enabled);
    if !company_name.is_empty() {
        active_instance.instance_name = Set(company_name);
    }
    active_instance.updated_at = Set(now.into());
    active_instance
        .update(&state.db)
        .await
        .map_err(AppError::Database)?;

    // 9. Emitir sesión de admin y redirigir
    let user = users::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

    let jar = issue_session_cookie(&state, &headers, jar, &user, SessionSurface::Admin).await?;
    let redirect_to = admin_success(&state, "general/");
    Ok((jar, Redirect::to(&redirect_to)))
}

// ─── POST /api/instances/admins/sign-in/ ─────────────────────────────────────
//
// Flujo (espeja InstanceAdminSignInEndpoint en Django):
//   1. Valida instance, email, password.
//   2. Verifica que el usuario sea InstanceAdmin activo.
//   3. Emite cookie de sesión Admin.

#[utoipa::path(
    post,
    path = "/api/instances/admins/sign-in/",
    tag = "Instance",
    request_body(
        content = AdminSignInForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (status = 303, description = "Signed in, redirects to God Mode"),
        (status = 302, description = "Redirect with error params"),
    )
)]
pub async fn admin_sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(form): Form<AdminSignInForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    // 1. Instance existe
    let instance = instances::Entity::find()
        .active()
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound)?;

    // 2. Campos obligatorios
    let email = form.email.as_deref().unwrap_or("").trim().to_lowercase();
    let password = form.password.as_deref().unwrap_or("");

    if email.is_empty() || password.is_empty() {
        return Ok(admin_error(&state, 5170, "REQUIRED_ADMIN_EMAIL_PASSWORD"));
    }

    if !is_valid_email(&email) {
        return Ok(admin_error(&state, 5160, "INVALID_ADMIN_EMAIL"));
    }

    // 3. Usuario existe y está activo
    let Some(user) = users::Entity::find()
        .filter(users::Column::Email.eq(Some(email.clone())))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
    else {
        return Ok(admin_error(&state, 5185, "ADMIN_USER_DOES_NOT_EXIST"));
    };

    if !user.is_active {
        return Ok(admin_error(&state, 5190, "ADMIN_USER_DEACTIVATED"));
    }

    // 4. Verificar contraseña
    if !verify_password(password, &user.password) {
        return Ok(admin_error(&state, 5175, "ADMIN_AUTHENTICATION_FAILED"));
    }

    // 5. El usuario debe ser InstanceAdmin de esta instancia
    let is_admin = user.is_superuser
        || instance_admins::Entity::find()
            .active()
            .filter(instance_admins::Column::InstanceId.eq(instance.id))
            .filter(instance_admins::Column::UserId.eq(user.id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .is_some();

    if !is_admin {
        return Ok(admin_error(&state, 5175, "ADMIN_AUTHENTICATION_FAILED"));
    }

    // 6. Actualizar metadatos de login
    let now = Utc::now();
    let mut user_model: users::ActiveModel = user.into();
    user_model.last_active = Set(Some(now.into()));
    user_model.last_login_time = Set(Some(now.into()));
    user_model.last_login_ip = Set(extract_client_ip(&headers));
    user_model.last_login_uagent = Set(headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned());
    user_model.token_updated_at = Set(Some(now.into()));
    user_model.last_login = Set(Some(now.into()));
    user_model.is_active = Set(true);
    user_model.updated_at = Set(now.into());
    let user = user_model.update(&state.db).await.map_err(AppError::Database)?;

    // 7. Emitir cookie de sesión admin
    let jar =
        replace_session_cookie(&state, &headers, jar, &user, SessionSurface::Admin).await?;
    let redirect_to = admin_success(&state, "general/");
    Ok((jar, Redirect::to(&redirect_to)))
}

// ─── POST /api/instances/admins/sign-out/ ────────────────────────────────────
//
// Invalida la sesión del admin y redirige al inicio de God Mode.
// Requiere token CSRF para prevenir CSRF attacks (igual que el logout normal).

#[utoipa::path(
    post,
    path = "/api/instances/admins/sign-out/",
    tag = "Instance",
    responses(
        (status = 303, description = "Signed out"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Invalid CSRF token"),
    )
)]
pub async fn admin_sign_out(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    form: Form<CsrfForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    // Validar CSRF
    let csrf_cookie = jar
        .get(CSRF_COOKIE_NAME)
        .map(|c| c.value().to_owned())
        .ok_or(AppError::Forbidden)?;

    if !is_valid_csrf(&form.csrfmiddlewaretoken, &csrf_cookie) {
        return Err(AppError::Forbidden);
    }

    // Intentar actualizar metadatos de logout del usuario activo.
    // Si no hay sesión activa simplemente redirigimos igual.
    if let Ok(session_key) = jar
        .get(ADMIN_SESSION_COOKIE_NAME)
        .or_else(|| jar.get(SESSION_COOKIE_NAME))
        .map(|c| c.value().to_owned())
        .ok_or(())
    {
        use sea_orm::EntityTrait;
        use crate::entities::sessions;

        // Eliminar sesión de la BD
        let _ = sessions::Entity::delete_by_id(&session_key)
            .exec(&state.db)
            .await;
    }

    // Actualizar last_logout si podemos identificar al usuario por el header
    let user_ip = extract_client_ip(&headers);
    let now = Utc::now();
    // Best-effort: no bloqueamos el logout si falla
    if let Ok(Some(session)) = {
        use sea_orm::EntityTrait;
        use crate::entities::sessions;
        let key = jar
            .get(ADMIN_SESSION_COOKIE_NAME)
            .or_else(|| jar.get(SESSION_COOKIE_NAME))
            .map(|c| c.value().to_owned());
        match key {
            Some(k) => sessions::Entity::find_by_id(&k)
                .one(&state.db)
                .await
                .map_err(AppError::Database),
            None => Ok(None),
        }
    } {
        use crate::utils::django_sessions::decode_session;
        if let Ok(data) = decode_session(&session.session_data, &state.config.secret_key) {
            if let Some(uid_str) = data.get("_auth_user_id").and_then(|v| v.as_str()) {
                if let Ok(uid) = uid_str.parse::<uuid::Uuid>() {
                    if let Ok(Some(user)) = users::Entity::find_by_id(uid)
                        .one(&state.db)
                        .await
                    {
                        let mut um: users::ActiveModel = user.into();
                        um.last_logout_ip = Set(user_ip);
                        um.last_logout_time = Set(Some(now.into()));
                        let _ = um.update(&state.db).await;
                    }
                }
            }
        }
    }

    // Expirar cookies de sesión y CSRF
    let expired = |name: &str| {
        Cookie::build((name.to_owned(), String::new()))
            .max_age(time::Duration::ZERO)
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .build()
    };

    let new_jar = jar
        .remove(expired(ADMIN_SESSION_COOKIE_NAME))
        .remove(expired(SESSION_COOKIE_NAME))
        .remove(expired(CSRF_COOKIE_NAME));

    let redirect_to = admin_base(&state);
    Ok((new_jar, Redirect::to(&redirect_to)))
}
