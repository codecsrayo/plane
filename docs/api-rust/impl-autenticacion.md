---
titulo: Autenticación — Session Cookie y API Key
aliases:
  - autenticacion
  - session-cookie
  - api-key
  - SessionUser
  - ApiKeyUser
tags:
  - rust
  - axum
  - auth
  - cookies
  - session
  - api-key
  - pendiente-implementar
relacionado:
  - "[[MOC]]"
  - "[[impl-extractores-auth]]"
  - "[[impl-bootstrap]]"
  - "[[ref-estructura-django]]"
  - "[[plan-riesgos]]"
estado: activo
---

# Autenticación — Session Cookie y API Key

> [!INFO] Estado actual en Rust
> **Cero implementado.** `src/auth/middleware.todo.rs` y `src/auth/permissions.todo.rs` están vacíos. Este documento cubre la implementación completa.

---

## Contexto — cómo autentifica Django hoy

| Mecanismo          | Módulo Django                          | Quién lo usa                         | Header / Cookie     |
| ------------------ | -------------------------------------- | ------------------------------------ | ------------------- |
| **Session cookie** | `plane/app/` + `plane/authentication/` | Frontend Next.js (web app)           | Cookie `session-id` |
| **API Key**        | `plane/api/`                           | Integraciones externas, CLI, scripts | Header `X-Api-Key`  |

La app web vive **100% en session cookies** — no usa JWT ni Bearer tokens.

---

## Dependencias a agregar

```toml
# apps/api_rust/Cargo.toml
axum-extra = { version = "0.10", features = ["cookie"] }
time = "0.3"   # requerido por Cookie::max_age
```

---

## Parte 1 — Session Cookie

### Cómo funciona en Django

```
Browser              Django               PostgreSQL
  │  Cookie: session-id=abc123  ──────────► │
  │                    │  SELECT FROM sessions WHERE session_key='abc123'
  │                    │  AND expire_date > NOW()
  │                    │ ◄─────────────────────│
  │                    │  user_id = "uuid"
  │                    │  SELECT FROM users WHERE id = 'uuid'
  │ ◄──────────────────│
  │  200 OK
```

**Puntos clave:**

- `session_key` es una cadena aleatoria de 128 chars — **no JWT, no encriptado**
- La tabla `sessions` tiene `user_id` como columna directa — Rust **no necesita decodificar `session_data`** (pickle Python)
- Dos cookies distintas según el path:
  - `session-id` → rutas normales (7 días TTL)
  - `admin-session-id` → rutas con `instances` en el path (1 hora TTL)

### `src/auth/session.rs`

```rust
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::{AppState, entities::{sessions, users}, error::AppError};

pub const SESSION_COOKIE_NAME: &str       = "session-id";
pub const ADMIN_SESSION_COOKIE_NAME: &str = "admin-session-id";

#[derive(Debug, Clone, PartialEq)]
pub enum SessionKind {
    App,
    Admin,
}

impl SessionKind {
    pub fn from_path(path: &str) -> Self {
        if path.contains("instances") { Self::Admin } else { Self::App }
    }
    pub fn primary_cookie(&self) -> &'static str {
        match self { Self::App => SESSION_COOKIE_NAME, Self::Admin => ADMIN_SESSION_COOKIE_NAME }
    }
    pub fn fallback_cookie(&self) -> &'static str {
        match self { Self::App => ADMIN_SESSION_COOKIE_NAME, Self::Admin => SESSION_COOKIE_NAME }
    }
}

pub struct SessionUser(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for SessionUser
where S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let jar  = CookieJar::from_headers(&parts.headers);
        let kind = SessionKind::from_path(parts.uri.path());

        let session_key = jar
            .get(kind.primary_cookie())
            .or_else(|| jar.get(kind.fallback_cookie()))
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized)?;

        // Django genera session_key de 128 chars. Valores más largos son inválidos
        // y causarían una query DB innecesaria con input potencialmente enorme.
        if session_key.len() > 128 {
            return Err(AppError::Unauthorized);
        }

        let session = sessions::Entity::find_by_id(&session_key)
            .filter(sessions::Column::ExpireDate.gt(Utc::now()))
            .one(&state.as_ref().db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        // ✅ Usar user_id directo — NO decodificar session_data (pickle Python)
        let user_id: uuid::Uuid = session.user_id
            .ok_or(AppError::Unauthorized)?
            .parse()
            .map_err(|_| AppError::Unauthorized)?;

        let user = users::Entity::find_by_id(user_id)
            .one(&state.as_ref().db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active { return Err(AppError::Unauthorized); }
        Ok(SessionUser(user))
    }
}
```

### Atributos de la cookie (réplica exacta de Django)

| Atributo Django                   | Valor                   | Rust (`Cookie::build`)                     |
| --------------------------------- | ----------------------- | ------------------------------------------ |
| `SESSION_COOKIE_HTTPONLY = True`  | No accesible desde JS   | `.http_only(true)`                         |
| `SESSION_COOKIE_SECURE`           | Solo HTTPS en prod      | `.secure(config.is_production)`            |
| `SESSION_COOKIE_SAMESITE`         | `"Lax"`                 | `.same_site(SameSite::Lax)`                |
| `SESSION_COOKIE_DOMAIN`           | `COOKIE_DOMAIN` env var | `.domain(config.cookie_domain.as_deref())` |
| `SESSION_COOKIE_AGE = 604800`     | 7 días                  | `.max_age(time::Duration::days(7))`        |
| `ADMIN_SESSION_COOKIE_AGE = 3600` | 1 hora                  | `.max_age(time::Duration::hours(1))`       |

### Diferencia crítica: `session_data` vs `user_id`

```sql
-- Tabla sessions en PostgreSQL
session_key  TEXT PRIMARY KEY   -- clave aleatoria 128 chars
session_data TEXT               -- base64(pickle(dict)) — SOLO lo lee Python
expire_date  TIMESTAMPTZ
user_id      VARCHAR(50)        -- ← Rust usa SOLO esta columna (indexada)
```

```rust
// ✅ Correcto
let user_id_str = session.user_id.ok_or(AppError::Unauthorized)?;

// ❌ NUNCA intentar esto — pickle de Python no decodificable en Rust
// let data = base64::decode(&session.session_data)?;
```

### Logout — `src/auth/logout.rs`

```rust
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{CookieJar, cookie::{Cookie, SameSite}};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    SessionUser(user): SessionUser,
) -> Result<(CookieJar, StatusCode), AppError> {
    // Borrar sesión de la DB
    let session_key = jar
        .get(SESSION_COOKIE_NAME)
        .or_else(|| jar.get(ADMIN_SESSION_COOKIE_NAME))
        .map(|c| c.value().to_string());

    if let Some(key) = session_key {
        sessions::Entity::delete_by_id(&key)
            .exec(&state.db).await
            .map_err(AppError::Database)?;
    }

    // Expirar cookies (Max-Age=0 elimina la cookie en el browser)
    let expired = |name: &str| Cookie::build((name, ""))
        .max_age(time::Duration::ZERO)
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();

    let new_jar = jar
        .remove(expired(SESSION_COOKIE_NAME))
        .remove(expired(ADMIN_SESSION_COOKIE_NAME));

    Ok((new_jar, StatusCode::NO_CONTENT))
}
```

---

## Parte 2 — API Key (`X-Api-Key`)

### `src/auth/api_key.rs`

```rust
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use crate::{AppState, entities::{api_tokens, users}, error::AppError};

pub const API_KEY_HEADER:    &str = "x-api-key";
pub const RATE_LIMIT_HUMAN:  u32  = 60;    // requests/min
pub const RATE_LIMIT_SERVICE: u32 = 300;   // requests/min (is_service=true)

pub struct ApiKeyContext {
    pub user:  users::Model,
    pub token: api_tokens::Model,
}

pub struct ApiKeyUser(pub ApiKeyContext);

#[async_trait]
impl<S> FromRequestParts<S> for ApiKeyUser
where S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let raw_key = parts.headers
            .get(API_KEY_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Validación de longitud — evita queries DB con inputs gigantes
        if raw_key.len() > 128 {
            return Err(AppError::Unauthorized);
        }

        let now = Utc::now();

        let token = api_tokens::Entity::find()
            .filter(api_tokens::Column::Token.eq(raw_key))
            .filter(api_tokens::Column::IsActive.eq(true))
            .filter(api_tokens::Column::DeletedAt.is_null())
            .filter(
                sea_orm::Condition::any()
                    .add(api_tokens::Column::ExpiredAt.is_null())
                    .add(api_tokens::Column::ExpiredAt.gt(now)),
            )
            .one(&state.as_ref().db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        // ✅ Rate limit AQUÍ — tenemos token.is_service, aplicamos el límite correcto.
        // El middleware de rate limit genérico no tiene acceso al modelo del token
        // y siempre aplicaría RATE_LIMIT_HUMAN (bug silencioso para tokens de servicio).
        let limit = if token.is_service { RATE_LIMIT_SERVICE } else { RATE_LIMIT_HUMAN };
        let app_state = state.as_ref();
        apply_rate_limit(&app_state.rate_limit, raw_key, limit)?;

        // Actualizar last_used fire-and-forget — log errores en lugar de descartarlos
        {
            let mut active: api_tokens::ActiveModel = token.clone().into();
            active.last_used = Set(Some(now.into()));
            if let Err(e) = active.update(&state.as_ref().db).await {
                tracing::warn!(error = %e, "Failed to update last_used for api token");
            }
        }

        let user = users::Entity::find_by_id(token.user_id)
            .one(&state.as_ref().db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active { return Err(AppError::Unauthorized); }
        Ok(ApiKeyUser(ApiKeyContext { user, token }))
    }
}

/// Aplica el rate limit en memoria al bucket del token dado.
/// Retorna `Err(AppError::RateLimited)` si se superó el límite.
/// Separar la lógica facilita el test unitario sin levantar un servidor.
pub fn apply_rate_limit(
    state: &RateLimitState,
    raw_key: &str,
    limit: u32,
) -> Result<(), AppError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let window  = 60u64;
    let now     = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let window_start = (now / window) * window;

    let bucket = bucket_key(raw_key);
    // NOTA: en producción con múltiples réplicas migrar a Redis (Fase 3).
    // Este mutex es per-proceso — correcto para instancia única.
    let mut buckets = state.buckets.blocking_lock();
    let entry = buckets.entry(bucket).or_insert((0, window_start));
    if entry.1 < window_start { *entry = (0, window_start); }
    entry.0 += 1;
    if entry.0 > limit {
        Err(AppError::RateLimited)
    } else {
        Ok(())
    }
}
```

### Rate limit middleware — `src/auth/rate_limit.rs`

> [!IMPORTANT] Arquitectura actualizada
> El **enforcement** del rate limit (contar requests, retornar 429) ocurre dentro de `ApiKeyUser::from_request_parts` porque en ese punto el extractor ya tiene `token.is_service` — lo que permite elegir el límite correcto (`RATE_LIMIT_HUMAN=60` vs `RATE_LIMIT_SERVICE=300`).
>
> El middleware Tower de abajo es solo un **inyector de headers de respuesta** (`X-RateLimit-*`) y no toma decisiones de bloqueo.

```rust
use axum::{body::Body, http::{Request, Response, StatusCode, HeaderValue}, middleware::Next};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use sha2::{Sha256, Digest};

/// Genera un bucket key opaco a partir del raw token.
/// ⚠️ NUNCA almacenar el raw token como key del HashMap:
///    cualquier memory dump / heap profiler expondría todas las claves activas.
pub fn bucket_key(raw_token: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw_token.as_bytes());
    format!("{:x}", h.finalize())
}

#[derive(Default)]
pub struct RateLimitState {
    /// sha256(token) → (request_count, window_start_unix_secs)
    pub buckets: Mutex<HashMap<String, (u32, u64)>>,
}

/// Middleware Tower — inyecta headers X-RateLimit-* en la respuesta.
/// No bloquea — el bloqueo ocurre en ApiKeyUser::from_request_parts.
pub async fn rate_limit_headers_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let window  = 60u64;
    let now     = chrono::Utc::now().timestamp() as u64;
    let reset_at = ((now / window) + 1) * window;

    let reset_hv = HeaderValue::from_str(&reset_at.to_string())
        .unwrap_or_else(|_| HeaderValue::from_static("0"));

    let mut resp = next.run(req).await;
    // Los headers de remaining los añade el extractor vía Extension si es necesario.
    resp.headers_mut().insert("X-RateLimit-Reset", reset_hv);
    resp
}
```

> [!NOTE] Migración a Redis en Fase 3
> El rate limit en memoria no es consistente entre múltiples instancias. Migrar a `fred` (Redis) en la Fase 3 para entornos con múltiples réplicas.

---

## Parte 3 — Extractor combinado `AnyAuth`

Para endpoints que aceptan tanto sesión como API Key:

```rust
// src/auth/any_auth.rs
pub struct AnyAuth(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for AnyAuth
where S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // Intentar session cookie primero
        if let Ok(SessionUser(user)) = SessionUser::from_request_parts(parts, state).await {
            return Ok(AnyAuth(user));
        }
        // Fallback: API Key
        if let Ok(ApiKeyUser(ctx)) = ApiKeyUser::from_request_parts(parts, state).await {
            return Ok(AnyAuth(ctx.user));
        }
        Err(AppError::Unauthorized)
    }
}
```

---

## Tabla comparativa Django vs Rust

### Session Cookie

| Aspecto         | Django                              | Rust                                                           |
| --------------- | ----------------------------------- | -------------------------------------------------------------- |
| Leer cookie     | `request.COOKIES.get("session-id")` | `CookieJar::from_headers(&parts.headers)`                      |
| Buscar sesión   | `SessionStore(session_key)`         | `sessions::Entity::find_by_id(&key).filter(expire_date > now)` |
| Obtener user_id | `session["_auth_user_id"]`          | `session.user_id` (columna directa)                            |
| Logout — DB     | `logout(request)`                   | `sessions::Entity::delete_by_id(&key)`                         |
| Logout — cookie | `response.delete_cookie(...)`       | `Cookie::build(...).max_age(Duration::ZERO)`                   |

### API Key

| Aspecto           | Django                                          | Rust                                                               |
| ----------------- | ----------------------------------------------- | ------------------------------------------------------------------ |
| Leer header       | `request.headers.get("X-Api-Key")`              | `parts.headers.get("x-api-key")`                                   |
| Check expiración  | `expired_at__gt=now OR expired_at__isnull=True` | `Condition::any().add(ExpiredAt.is_null()).add(ExpiredAt.gt(now))` |
| Rate limit normal | 60/min (Redis cache)                            | `RATE_LIMIT_HUMAN = 60` (memoria → Redis Fase 3)                   |

---

## Checklist de implementación

- [ ] Agregar `axum-extra` y `time` a `Cargo.toml`
- [ ] `src/auth/session.rs` — `SessionUser` extractor
- [ ] `src/auth/logout.rs` — handler POST `/auth/sign-out/`
- [ ] `src/auth/api_key.rs` — `ApiKeyUser` extractor
- [ ] `src/auth/rate_limit.rs` — middleware Tower
- [ ] `src/auth/any_auth.rs` — `AnyAuth` combinado
- [ ] Agregar `rate_limit: Arc<RateLimitState>` a `AppState`
- [ ] Registrar `/auth/sign-out/` en el router
- [ ] Tests en `tests/auth.rs`

---

## Plan de implementación

```
Fase 1:
  [ ] src/auth/middleware.rs    — session cookie middleware (tower Layer)
  [ ] src/auth/rate_limit.rs   — RateLimitState (DashMap en memoria, migrar a Redis Fase 3)
  [ ] src/auth/extractors.rs   — CurrentUser + guards (ver impl-extractores-auth)

Fase 3:
  [ ] src/auth/rate_limit.rs   — migrar RateLimitState a Redis (fred) para multi-réplica
```

## 🔗 Navegar

← [[impl-bootstrap]] | [[MOC]] | → [[impl-extractores-auth]]

**Relacionado:** Extractores Bearer Token: [[impl-extractores-auth]] | Riesgos: [[plan-riesgos]] | Estructura Django auth: [[ref-estructura-django]]
