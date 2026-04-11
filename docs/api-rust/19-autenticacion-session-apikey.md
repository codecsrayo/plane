---
titulo: Autenticación — Session Cookie y API Key
tags:
  - rust
  - axum
  - auth
  - cookies
  - session
  - api-key
relacionado:
  - "[[10-patrones]]"
  - "[[18-implementacion-api-inicial]]"
  - "[[17-estructura-django]]"
estado: activo
---

# Autenticación — Session Cookie y API Key

> [!INFO] Estado actual en Rust
> **Cero implementado.** `src/auth/middleware.todo.rs` y `src/auth/permissions.todo.rs`
> están vacíos. Este documento cubre la implementación completa de los dos
> mecanismos de autenticación que Django tiene hoy.

---

## Contexto — cómo autentifica Django hoy

Django usa **dos mecanismos paralelos**, no uno:

| Mecanismo | Módulo Django | Quién lo usa | Header / Cookie |
|-----------|---------------|-------------|-----------------|
| **Session cookie** | `plane/app/` + `plane/authentication/` | Frontend Next.js (web app) | Cookie `session-id` |
| **API Key** | `plane/api/` | Integraciones externas, CLI, scripts | Header `X-Api-Key` |

```python
# plane/settings/common.py
DEFAULT_AUTHENTICATION_CLASSES = ("rest_framework.authentication.SessionAuthentication",)
```

La app web vive **100% en session cookies** — no usa JWT ni Bearer tokens.
La API pública añade `APIKeyAuthentication` por encima vía `authentication_classes`
en cada ViewSet.

---

## Dependencias a agregar al `Cargo.toml`

Axum no maneja cookies nativamente. Hay que agregar `axum-extra`:

```toml
# apps/api_rust/Cargo.toml
axum-extra = { version = "0.10", features = ["cookie"] }
chrono     = { version = "0.4.44", features = ["serde"] }  # ya presente
```

> `axum-extra 0.10` es compatible con `axum 0.8`. El feature `cookie`
> incluye `CookieJar` y `PrivateCookieJar` — se usa `CookieJar` porque
> las cookies de sesión de Django no están cifradas por el servidor
> (la clave de sesión es un random de 128 chars, no un payload cifrado).

---

## Parte 1 — Session Cookie

### Cómo funciona en Django

```
Browser                Django                  PostgreSQL
  │                      │                          │
  │  GET /api/me/        │                          │
  │  Cookie: session-id=abc123  ──────────────────► │
  │                      │  SELECT * FROM sessions  │
  │                      │  WHERE session_key='abc123'
  │                      │  AND expire_date > NOW() │
  │                      │ ◄────────────────────────│
  │                      │  user_id = "uuid-del-user"
  │                      │  SELECT * FROM users     │
  │                      │  WHERE id = 'uuid-del-user'
  │                      │ ◄────────────────────────│
  │ ◄────────────────────│                          │
  │  200 OK              │                          │
```

**Puntos clave del modelo Django:**

- `session_key` es una cadena aleatoria de 128 chars (no JWT, no encriptado)
- La tabla `sessions` tiene `user_id` como columna directa — Rust **no necesita
  decodificar `session_data`** (que está en formato base64+pickle de Python)
- `expire_date` controla si la sesión sigue válida
- Dos cookies distintas según el path del request:
  - `session-id` → rutas normales (7 días de TTL)
  - `admin-session-id` → rutas con `instances` en el path (1 hora de TTL)

### `src/auth/session.rs` — extractor completo

```rust
// src/auth/session.rs

use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    AppState,
    entities::{sessions, users},
    error::AppError,
};

// ── Nombres de cookies (replica plane/settings/common.py) ─────────────────────

/// Cookie de sesión normal — rutas `/api/*` excepto instancias
pub const SESSION_COOKIE_NAME: &str = "session-id";

/// Cookie de sesión admin — rutas que contienen `instances` en el path
pub const ADMIN_SESSION_COOKIE_NAME: &str = "admin-session-id";

// ── Tipos de sesión ────────────────────────────────────────────────────────────

/// Tipo de sesión detectado a partir del path del request.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionKind {
    /// Sesión normal de la app (cookie `session-id`)
    App,
    /// Sesión de administración de instancia (cookie `admin-session-id`)
    Admin,
}

impl SessionKind {
    /// Detecta el tipo de sesión según el path del request.
    /// Replica la lógica de `SessionMiddleware._get_session_key()` en Django.
    pub fn from_path(path: &str) -> Self {
        if path.contains("instances") {
            Self::Admin
        } else {
            Self::App
        }
    }

    /// Nombre de la cookie principal para este tipo de sesión.
    pub fn primary_cookie(&self) -> &'static str {
        match self {
            Self::App   => SESSION_COOKIE_NAME,
            Self::Admin => ADMIN_SESSION_COOKIE_NAME,
        }
    }

    /// Nombre de la cookie de fallback (Django intenta la otra si la primaria no existe).
    pub fn fallback_cookie(&self) -> &'static str {
        match self {
            Self::App   => ADMIN_SESSION_COOKIE_NAME,
            Self::Admin => SESSION_COOKIE_NAME,
        }
    }
}

// ── Extractor principal ────────────────────────────────────────────────────────

/// Usuario autenticado vía session cookie.
///
/// Equivalente a `request.user` en Django después de que
/// `SessionAuthentication` lo popula.
///
/// Uso en handler:
/// ```rust
/// async fn get_me(
///     State(state): State<AppState>,
///     SessionUser(user): SessionUser,
/// ) -> Result<Json<UserResponse>, AppError> {
///     Ok(Json(user.into()))
/// }
/// ```
pub struct SessionUser(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let app_state = state.as_ref();

        // 1. Leer todas las cookies del request
        let jar = CookieJar::from_headers(&parts.headers);

        // 2. Determinar tipo de sesión según el path (replica SessionMiddleware)
        let kind = SessionKind::from_path(parts.uri.path());

        // 3. Buscar session_key: primero cookie principal, luego fallback
        //    Replica: request.COOKIES.get(primary) or request.COOKIES.get(fallback)
        let session_key = jar
            .get(kind.primary_cookie())
            .or_else(|| jar.get(kind.fallback_cookie()))
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized)?;

        // 4. Buscar sesión en PostgreSQL
        //    Solo trae sesiones no expiradas
        let session = sessions::Entity::find_by_id(&session_key)
            .filter(sessions::Column::ExpireDate.gt(Utc::now()))
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        // 5. Extraer user_id de la columna directa (NO decodificar session_data)
        //    La tabla custom de Django guarda user_id como columna indexada
        let user_id_str = session.user_id.ok_or(AppError::Unauthorized)?;
        let user_id: Uuid = user_id_str
            .parse()
            .map_err(|_| AppError::Unauthorized)?;

        // 6. Buscar usuario y verificar que esté activo
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
```

### `src/auth/logout.rs` — borrar sesión y cookie

```rust
// src/auth/logout.rs

use axum::{extract::State, response::Response, http::StatusCode};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use sea_orm::EntityTrait;

use crate::{AppState, auth::session::{SessionUser, SESSION_COOKIE_NAME, ADMIN_SESSION_COOKIE_NAME}, error::AppError};

/// Cierra la sesión del usuario.
///
/// Equivalente a `django.contrib.auth.logout()`:
/// 1. Borra la fila en la tabla `sessions`
/// 2. Setea Set-Cookie con Max-Age=0 para que el browser descarte la cookie
///
/// Replica: `plane/authentication/views/app/signout.py`
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    SessionUser(user): SessionUser,
) -> Result<(CookieJar, StatusCode), AppError> {
    // Buscar session_key activa en las cookies
    let session_key = jar
        .get(SESSION_COOKIE_NAME)
        .or_else(|| jar.get(ADMIN_SESSION_COOKIE_NAME))
        .map(|c| c.value().to_string());

    // Borrar sesión de la DB si existe
    if let Some(key) = session_key {
        use crate::entities::sessions;
        sessions::Entity::delete_by_id(&key)
            .exec(&state.db)
            .await
            .map_err(AppError::Database)?;
    }

    // Actualizar last_logout en el usuario
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::users;
    let mut user_active: users::ActiveModel = user.into();
    user_active.last_logout_time = Set(Some(chrono::Utc::now().into()));
    user_active.update(&state.db).await.map_err(AppError::Database)?;

    // Construir cookies de expiración (Max-Age=0 elimina la cookie en el browser)
    let expired_session = Cookie::build((SESSION_COOKIE_NAME, ""))
        .max_age(time::Duration::ZERO)
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();

    let expired_admin = Cookie::build((ADMIN_SESSION_COOKIE_NAME, ""))
        .max_age(time::Duration::ZERO)
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();

    let new_jar = jar.remove(expired_session).remove(expired_admin);

    Ok((new_jar, StatusCode::NO_CONTENT))
}
```

### Atributos de seguridad de la cookie (réplica exacta de Django)

| Atributo Django | Valor | Equivalente Rust (`Cookie::build`) |
|----------------|-------|-------------------------------------|
| `SESSION_COOKIE_HTTPONLY = True` | No accesible desde JS | `.http_only(true)` |
| `SESSION_COOKIE_SECURE = secure_origins` | Solo HTTPS en prod | `.secure(config.is_production)` |
| `SESSION_COOKIE_SAMESITE` | `"Lax"` (default Django) | `.same_site(SameSite::Lax)` |
| `SESSION_COOKIE_DOMAIN` | `COOKIE_DOMAIN` env var | `.domain(config.cookie_domain.as_deref())` |
| `SESSION_COOKIE_AGE = 604800` | 7 días | `.max_age(time::Duration::days(7))` |
| `ADMIN_SESSION_COOKIE_AGE = 3600` | 1 hora | `.max_age(time::Duration::hours(1))` |
| `SESSION_COOKIE_PATH` | `"/"` | `.path("/")` |

> **`SameSite::Lax`** es el default de Django cuando no se especifica
> `SESSION_COOKIE_SAMESITE`. Permite que las cookies se envíen en
> navegaciones top-level (links externos) pero no en peticiones cross-site
> iniciadas por terceros (CSRF protection básica).

### Diferencia crítica: `session_data` vs `user_id`

Django guarda la sesión serializada en `session_data` (base64 de un pickle Python).
**Rust no puede ni debe leer ese campo.** La tabla custom de Plane agrega
`user_id` como columna directa indexada precisamente para este caso de uso:

```sql
-- Lo que tiene la tabla sessions en PostgreSQL
session_key  TEXT PRIMARY KEY   -- clave aleatoria 128 chars
session_data TEXT               -- base64(pickle(dict)) — solo lo lee Python
expire_date  TIMESTAMPTZ
device_info  JSONB
user_id      VARCHAR(50)        -- ← Rust usa SOLO esta columna
```

```rust
// ✅ Correcto — usar columna user_id directa
let user_id_str = session.user_id.ok_or(AppError::Unauthorized)?;

// ❌ Incorrecto — NO intentar decodificar session_data
// let data = base64::decode(&session.session_data)?;
// Es pickle de Python — no decodificable en Rust de forma segura
```

---

## Parte 2 — API Key (`X-Api-Key`)

### Cómo funciona en Django

```
Cliente externo           Django                   PostgreSQL
     │                      │                           │
     │  GET /api/issues/     │                           │
     │  X-Api-Key: plane_api_abc123  ─────────────────► │
     │                      │  SELECT * FROM api_tokens │
     │                      │  WHERE token='plane_api_abc123'
     │                      │  AND is_active = true
     │                      │  AND (expired_at IS NULL
     │                      │       OR expired_at > NOW())
     │                      │ ◄─────────────────────────│
     │                      │  UPDATE api_tokens        │
     │                      │  SET last_used = NOW()    │
     │                      │  WHERE id = ...           │
     │ ◄────────────────────│                           │
     │  200 OK              │                           │
     │  X-RateLimit-Remaining: 59                       │
     │  X-RateLimit-Reset: 1735000000                   │
```

**Puntos clave del modelo Django:**

- El token tiene el prefijo `plane_api_` seguido de un UUID hex: `plane_api_<uuid4().hex>`
- `is_service = True` → rate limit de 300/min; `is_service = False` → 60/min
- `expired_at = NULL` significa que el token nunca expira
- Django actualiza `last_used` en cada request exitoso
- Los headers `X-RateLimit-Remaining` y `X-RateLimit-Reset` se agregan a la respuesta

### `src/auth/api_key.rs` — extractor completo

```rust
// src/auth/api_key.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::{
    AppState,
    entities::{api_tokens, users},
    error::AppError,
};

/// Nombre del header de autenticación por API Key.
/// Replica: `APIKeyAuthentication.auth_header_name` en Django.
pub const API_KEY_HEADER: &str = "x-api-key";

// ── Tipos de token ─────────────────────────────────────────────────────────────

/// Información del token API autenticado.
pub struct ApiKeyContext {
    pub user:  users::Model,
    pub token: api_tokens::Model,
}

// ── Límites de rate (replica ApiKeyRateThrottle en Django) ────────────────────

pub const RATE_LIMIT_HUMAN: u32   = 60;    // requests/min para tokens normales
pub const RATE_LIMIT_SERVICE: u32 = 300;   // requests/min para tokens de servicio

// ── Extractor ─────────────────────────────────────────────────────────────────

/// Usuario autenticado vía header `X-Api-Key`.
///
/// Equivalente a `APIKeyAuthentication` en `plane/app/middleware/api_authentication.py`.
///
/// Uso en handler:
/// ```rust
/// async fn list_issues(
///     State(state): State<AppState>,
///     ApiKeyUser(ctx): ApiKeyUser,
/// ) -> Result<Json<Vec<IssueResponse>>, AppError> {
///     // ctx.user  → usuario autenticado
///     // ctx.token → token usado (para auditoría, rate limit, workspace scope)
///     Ok(Json(vec![]))
/// }
/// ```
pub struct ApiKeyUser(pub ApiKeyContext);

#[async_trait]
impl<S> FromRequestParts<S> for ApiKeyUser
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let app_state = state.as_ref();

        // 1. Leer header X-Api-Key (case-insensitive via axum)
        let raw_key = parts
            .headers
            .get(API_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let now = Utc::now();

        // 2. Buscar token en api_tokens
        //    Condiciones (replica validate_api_token en Django):
        //    - token == raw_key
        //    - is_active = true
        //    - deleted_at IS NULL (soft delete)
        //    - expired_at IS NULL OR expired_at > NOW()
        let token = api_tokens::Entity::find()
            .filter(api_tokens::Column::Token.eq(raw_key))
            .filter(api_tokens::Column::IsActive.eq(true))
            .filter(api_tokens::Column::DeletedAt.is_null())
            .filter(
                sea_orm::Condition::any()
                    .add(api_tokens::Column::ExpiredAt.is_null())
                    .add(api_tokens::Column::ExpiredAt.gt(now)),
            )
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        // 3. Actualizar last_used (replica: api_token.last_used = timezone.now())
        //    Se hace en un task separado para no bloquear el request en producción
        //    Por ahora: actualización inline (simplificar en Fase 3 con apalis)
        {
            let mut active: api_tokens::ActiveModel = token.clone().into();
            active.last_used = Set(Some(now.into()));
            // fire-and-forget: si falla no bloqueamos el request
            let _ = active.update(&app_state.db).await;
        }

        // 4. Buscar usuario propietario del token
        let user = users::Entity::find_by_id(token.user_id)
            .one(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        Ok(ApiKeyUser(ApiKeyContext { user, token }))
    }
}
```

### Headers de rate limit en la respuesta

Django agrega `X-RateLimit-Remaining` y `X-RateLimit-Reset` vía
`ApiKeyRateThrottle`. En Rust esto se implementa como un middleware de Tower:

```rust
// src/auth/rate_limit.rs

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use chrono::Utc;

/// Estado del rate limiter por token (en memoria — Fase 1).
/// En Fase 3 migrar a Redis (fred) para consistencia multi-instancia.
#[derive(Default)]
pub struct RateLimitState {
    /// token_key → (request_count, window_start_unix)
    pub buckets: Mutex<HashMap<String, (u32, i64)>>,
}

/// Middleware de rate limit para API Keys.
/// Replica `ApiKeyRateThrottle` y `ServiceTokenRateThrottle` de Django.
///
/// Retorna 429 si se supera el límite.
/// Agrega headers `X-RateLimit-Remaining` y `X-RateLimit-Reset`.
pub async fn rate_limit_middleware(
    State(rl): State<Arc<RateLimitState>>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Si no hay API Key, dejar pasar (la auth lo rechazará si es necesario)
    let Some(key) = api_key else {
        return next.run(req).await;
    };

    // Rate limit: 60/min normal, 300/min service
    // Por ahora usamos 60 — en Fase 2 leer is_service del token via state
    let limit = RATE_LIMIT_HUMAN;
    let window = 60i64; // segundos

    let now = Utc::now().timestamp();
    let window_start = (now / window) * window;
    let reset_at = window_start + window;

    let (count, remaining) = {
        let mut buckets = rl.buckets.lock().await;
        let entry = buckets.entry(key).or_insert((0, window_start));

        // Nueva ventana → resetear contador
        if entry.1 < window_start {
            *entry = (0, window_start);
        }

        entry.0 += 1;
        let remaining = limit.saturating_sub(entry.0);
        (entry.0, remaining)
    };

    if count > limit {
        let mut resp = Response::new(Body::from(
            r#"{"error":"Rate limit exceeded"}"#,
        ));
        *resp.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        resp.headers_mut().insert("X-RateLimit-Remaining", "0".parse().unwrap());
        resp.headers_mut().insert("X-RateLimit-Reset", reset_at.to_string().parse().unwrap());
        return resp;
    }

    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );
    resp.headers_mut().insert(
        "X-RateLimit-Reset",
        reset_at.to_string().parse().unwrap(),
    );
    resp
}
```

---

## Parte 3 — Extractor combinado (`AnyAuth`)

Algunos endpoints en Django aceptan tanto sesión como API Key. Para replicar
esto en Rust sin duplicar handlers, se usa un extractor que prueba ambos:

```rust
// src/auth/any_auth.rs

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use crate::{AppState, auth::{session::SessionUser, api_key::ApiKeyUser}, entities::users, error::AppError};

/// Usuario autenticado por cualquier mecanismo (session cookie OR API Key).
///
/// Prueba primero session cookie, luego API Key.
/// Si ninguno está presente o es válido → 401.
///
/// Uso:
/// ```rust
/// async fn get_workspace(
///     AnyAuth(user): AnyAuth,
///     ...
/// ) -> Result<...> { ... }
/// ```
pub struct AnyAuth(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for AnyAuth
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // Intentar session cookie primero (path normal de la web app)
        if let Ok(SessionUser(user)) = SessionUser::from_request_parts(parts, state).await {
            return Ok(AnyAuth(user));
        }

        // Fallback: API Key (integraciones externas)
        if let Ok(ApiKeyUser(ctx)) = ApiKeyUser::from_request_parts(parts, state).await {
            return Ok(AnyAuth(ctx.user));
        }

        Err(AppError::Unauthorized)
    }
}
```

---

## Parte 4 — Integración en `main.rs` y router

### Agregar `RateLimitState` al `AppState`

```rust
// src/main.rs

use std::sync::Arc;
use crate::auth::rate_limit::RateLimitState;

#[derive(Clone)]
pub struct AppState {
    pub db:         sea_orm::DatabaseConnection,
    pub config:     Arc<Config>,
    pub rate_limit: Arc<RateLimitState>,   // ← agregar
}

// En main():
let state = AppState {
    db,
    config: Arc::new(config.clone()),
    rate_limit: Arc::new(RateLimitState::default()),
};
```

### Módulo `auth` completo

```rust
// src/auth/mod.rs
pub mod session;
pub mod api_key;
pub mod any_auth;
pub mod logout;
pub mod rate_limit;
```

### Router con middleware de rate limit solo en rutas de API pública

```rust
// src/routes/mod.rs

use axum::{middleware, routing::{get, post}, Router};
use crate::{AppState, auth::rate_limit::{rate_limit_middleware, RateLimitState}};

pub fn build_router(state: AppState) -> Router {
    // Rutas web app — auth vía session cookie, sin rate limit de API
    let app_router = Router::new()
        .route("/me", get(users::get_me))
        .route("/workspaces", get(workspaces::list))
        // ... más rutas
        ;

    // Rutas API pública — auth vía X-Api-Key, con rate limit
    let public_api_router = Router::new()
        .route("/workspaces/:slug/projects/:project_id/issues", get(issues::list))
        // ... más rutas
        .layer(middleware::from_fn_with_state(
            state.rate_limit.clone(),
            rate_limit_middleware,
        ));

    // Logout — acepta ambos tipos de auth
    let auth_router = Router::new()
        .route("/sign-out/", post(logout::logout));

    Router::new()
        .nest("/api", app_router)
        .nest("/api", public_api_router)
        .nest("/auth", auth_router)
        .merge(swagger_ui())
        .with_state(state)
}
```

---

## Parte 5 — Tabla comparativa Django vs Rust

### Session Cookie

| Aspecto | Django | Rust |
|---------|--------|------|
| Leer cookie | `request.COOKIES.get("session-id")` | `CookieJar::from_headers(&parts.headers).get("session-id")` |
| Fallback cookie | `or request.COOKIES.get("admin-session-id")` | `.or_else(\|\| jar.get(kind.fallback_cookie()))` |
| Buscar sesión | `SessionStore(session_key)` | `sessions::Entity::find_by_id(&key).filter(expire_date > now)` |
| Obtener user_id | `session["_auth_user_id"]` | `session.user_id` (columna directa) |
| Verificar activo | `user.is_active` | `user.is_active` |
| Logout — DB | `logout(request)` → borra sesión | `sessions::Entity::delete_by_id(&key)` |
| Logout — cookie | `response.delete_cookie(...)` | `Cookie::build(...).max_age(Duration::ZERO)` |
| Cookie HttpOnly | `SESSION_COOKIE_HTTPONLY = True` | `.http_only(true)` |
| Cookie Secure | `SESSION_COOKIE_SECURE = secure_origins` | `.secure(config.is_production)` |
| Cookie SameSite | default `"Lax"` | `.same_site(SameSite::Lax)` |
| Cookie Domain | `COOKIE_DOMAIN` env var | `.domain(config.cookie_domain)` |
| TTL app | 604800 s (7 días) | `Duration::days(7)` |
| TTL admin | 3600 s (1 hora) | `Duration::hours(1)` |

### API Key

| Aspecto | Django | Rust |
|---------|--------|------|
| Leer header | `request.headers.get("X-Api-Key")` | `parts.headers.get("x-api-key")` |
| Validar token | `APIToken.objects.get(token=key, is_active=True, ...)` | `api_tokens::Entity::find().filter(...)` |
| Check expiración | `expired_at__gt=now OR expired_at__isnull=True` | `Condition::any().add(ExpiredAt.is_null()).add(ExpiredAt.gt(now))` |
| Actualizar last_used | `api_token.last_used = timezone.now(); api_token.save()` | `active.last_used = Set(Some(now)); active.update(&db)` |
| Rate limit normal | `60/minute` (Redis cache) | `RATE_LIMIT_HUMAN = 60` (en memoria → Redis en Fase 3) |
| Rate limit service | `300/minute` | `RATE_LIMIT_SERVICE = 300` |
| Headers respuesta | `X-RateLimit-Remaining`, `X-RateLimit-Reset` | middleware Tower agrega los headers |

---

## Parte 6 — Tests

```rust
// tests/auth.rs

use axum::http::StatusCode;
use axum_test::TestServer;

// ── Session Cookie ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_session_no_cookie_returns_401() {
    let server = build_test_server().await;
    let resp = server.get("/api/me").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_session_expired_returns_401() {
    let server = build_test_server().await;
    // Insertar sesión expirada en DB de test
    seed_expired_session(&server, "old-key-123").await;
    let resp = server
        .get("/api/me")
        .add_header("Cookie", "session-id=old-key-123")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_session_valid_returns_200() {
    let server = build_test_server().await;
    let (user, key) = seed_active_session(&server).await;
    let resp = server
        .get("/api/me")
        .add_header("Cookie", format!("session-id={key}"))
        .await;
    resp.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_admin_path_uses_admin_cookie() {
    let server = build_test_server().await;
    // Sesión en cookie normal — ruta instances requiere admin cookie
    let (_user, key) = seed_active_session(&server).await;
    let resp = server
        .get("/api/instances/")
        .add_header("Cookie", format!("session-id={key}"))
        .await;
    // Sin admin-session-id → 401 (session normal no es fallback para admin path)
    // Nota: Django SÍ hace fallback, replicar la lógica correctamente
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_clears_session_and_cookie() {
    let server = build_test_server().await;
    let (_user, key) = seed_active_session(&server).await;
    let resp = server
        .post("/auth/sign-out/")
        .add_header("Cookie", format!("session-id={key}"))
        .await;
    resp.assert_status(StatusCode::NO_CONTENT);
    // Verificar Set-Cookie: session-id=; Max-Age=0
    let set_cookie = resp.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(set_cookie.contains("Max-Age=0") || set_cookie.contains("max-age=0"));
}

// ── API Key ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_api_key_missing_returns_401() {
    let server = build_test_server().await;
    let resp = server.get("/api/workspaces/my-ws/issues").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_key_invalid_returns_401() {
    let server = build_test_server().await;
    let resp = server
        .get("/api/workspaces/my-ws/issues")
        .add_header("X-Api-Key", "plane_api_nonexistent")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_key_expired_returns_401() {
    let server = build_test_server().await;
    seed_expired_api_key(&server, "plane_api_expired123").await;
    let resp = server
        .get("/api/workspaces/my-ws/issues")
        .add_header("X-Api-Key", "plane_api_expired123")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_api_key_valid_returns_200_with_rate_headers() {
    let server = build_test_server().await;
    let key = seed_active_api_key(&server).await;
    let resp = server
        .get("/api/workspaces/my-ws/issues")
        .add_header("X-Api-Key", &key)
        .await;
    resp.assert_status(StatusCode::OK);
    assert!(resp.headers().contains_key("x-ratelimit-remaining"));
    assert!(resp.headers().contains_key("x-ratelimit-reset"));
}

#[tokio::test]
async fn test_api_key_rate_limit_429() {
    let server = build_test_server().await;
    let key = seed_active_api_key(&server).await;
    // Hacer 61 requests en el mismo minuto
    for i in 0..=RATE_LIMIT_HUMAN {
        let resp = server
            .get("/api/workspaces/my-ws/issues")
            .add_header("X-Api-Key", &key)
            .await;
        if i == RATE_LIMIT_HUMAN {
            resp.assert_status(StatusCode::TOO_MANY_REQUESTS);
        }
    }
}
```

---

## Parte 7 — Checklist de implementación

### Session Cookie

- [ ] Agregar `axum-extra = { version = "0.10", features = ["cookie"] }` al `Cargo.toml`
- [ ] Agregar `time = "0.3"` al `Cargo.toml` (requerido por `Cookie::max_age`)
- [ ] Crear `src/auth/session.rs` con `SessionUser` extractor
- [ ] Crear `src/auth/logout.rs` con handler de logout
- [ ] Exponer `SESSION_COOKIE_NAME` y `ADMIN_SESSION_COOKIE_NAME` como constantes en `config.rs`
- [ ] Agregar `cookie_domain` y `is_production` a `Config`
- [ ] Registrar `/auth/sign-out/` en el router

### API Key

- [ ] Crear `src/auth/api_key.rs` con `ApiKeyUser` extractor
- [ ] Crear `src/auth/rate_limit.rs` con middleware Tower
- [ ] Agregar `rate_limit: Arc<RateLimitState>` a `AppState`
- [ ] Aplicar middleware de rate limit en el router de API pública
- [ ] Migrar rate limit de memoria a Redis (`fred`) en Fase 3

### Ambos

- [ ] Crear `src/auth/any_auth.rs` con `AnyAuth` extractor combinado
- [ ] Crear `src/auth/mod.rs` con `pub mod` de todos los submódulos
- [ ] Escribir tests en `tests/auth.rs`

---

*Siguiente paso: implementar los extractors en `src/auth/` eliminando los
archivos `.todo.rs` → ver [[18-implementacion-api-inicial]] para el bootstrap
del servidor donde se integran.*
