---
titulo: Patrones de diseño y arquitectura
tags:
  - rust
  - axum
  - repository-pattern
  - appstate
  - apalis
relacionado:
  - "[[03-stack]]"
  - "[[06-soft-delete]]"
  - "[[09-workspace-seed]]"
---

## Patrones de diseño y arquitectura

> **Referencias externas por patrón:**
>
> | # | Patrón | Documentación oficial | Guía de referencia |
> |---|--------|-----------------------|--------------------|
> | 1 | Repository Pattern | [SeaORM — Queries](https://www.sea-ql.org/SeaORM/docs/basic-crud/select/) | [Rust API Guidelines — Modules](https://rust-lang.github.io/api-guidelines/organization.html) |
> | 2 | AppState (Axum) | [Axum — State](https://docs.rs/axum/latest/axum/extract/struct.State.html) | [Axum examples/todos](https://github.com/tokio-rs/axum/tree/main/examples/todos) |
> | 3 | Extractor Pattern | [Axum — FromRequestParts](https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html) | [Axum — custom extractor](https://github.com/tokio-rs/axum/blob/main/examples/customize-extractor-error/src/main.rs) |
> | 4 | Error unificado | [thiserror crate](https://docs.rs/thiserror/latest/thiserror/) | [Axum — IntoResponse](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html) |
> | 5 | Job Pattern (apalis) | [apalis — Book](https://docs.rs/apalis/latest/apalis/) | [apalis — postgres example](https://github.com/geofmureithi/apalis/tree/main/examples/postgres) |
> | 6 | Cron jobs | [tokio-cron-scheduler](https://docs.rs/tokio-cron-scheduler/latest/tokio_cron_scheduler/) | [cron expression syntax](https://crontab.guru/) |
>
> Diagrama de flujo completo de implementación → **[[16-diagramas-flujo]]**

---

### 1. Repository Pattern — aislar SeaORM de los handlers

Los handlers Axum no deben contener queries SeaORM directamente. El módulo
`src/repositories/` encapsula todo el acceso a datos:

```
src/
├── repositories/
│   ├── mod.rs
│   ├── issues.rs        ← list_issues, get_issue, create_issue, update_issue
│   ├── workspaces.rs    ← get_workspace_by_slug, list_workspaces_for_user
│   ├── projects.rs
│   └── states.rs
├── routes/
│   └── issues.rs        ← solo recibe AppState, llama a repositories::issues::*
```

```rust
// src/repositories/issues.rs
pub async fn list_issues(
    db: &DatabaseConnection,
    project_id: Uuid,
    filters: IssueFilters,
) -> Result<Vec<issues::Model>, DbErr> {
    issues::Entity::find()
        .active()   // WHERE deleted_at IS NULL — via SoftDeleteExt
        .filter(issues::Column::ProjectId.eq(project_id))
        .order_by_asc(issues::Column::SortOrder)
        .all(db)
        .await
}
```

Ventaja principal: los tests pueden mockear el repository sin levantar DB real.

---

### 2. AppState — estado global del servidor

```rust
// src/main.rs
#[derive(Clone)]
pub struct AppState {
    pub db:          DatabaseConnection,       // pool SeaORM (Postgres)
    pub redis:       fred::clients::Pool,      // pool Redis/Valkey
    pub s3:          aws_sdk_s3::Client,       // cliente S3/MinIO
    pub config:      Arc<Config>,              // env vars tipadas (dotenvy)
    pub job_storage: PgPool,                   // apalis backend (Postgres)
}
```

Se registra en Axum con `.with_state(state)`. Los handlers lo reciben
con `State(state): State<AppState>`.

---

### 3. Extractor Pattern — autenticación y permisos

> **Referencias:**
> - [Axum — `FromRequestParts` trait](https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html)
> - [Axum — `FromRequest` trait](https://docs.rs/axum/latest/axum/extract/trait.FromRequest.html) (cuando necesitas leer el body)
> - [Axum — custom extractor error](https://github.com/tokio-rs/axum/blob/main/examples/customize-extractor-error/src/main.rs)
> - [async-trait crate](https://docs.rs/async-trait/latest/async_trait/)

Axum permite extractors personalizados que corren **antes** del handler,
implementando auth + RBAC sin middleware global. Cada extractor implementa
`FromRequestParts` y retorna un `Rejection` automático si falla — el handler
nunca llega a ejecutarse.

---

#### 3.1 `FromRequestParts` vs `FromRequest` — cuándo usar cada uno

| Trait | Lee el body | Cuándo usarlo |
|-------|:-----------:|---------------|
| `FromRequestParts` | ❌ | Auth, headers, path params, query params — **usar siempre que sea posible** |
| `FromRequest` | ✅ | Solo cuando necesitas leer el body (JSON, form data) |

Los extractors de auth **siempre** implementan `FromRequestParts` — nunca
necesitan el body y así pueden componerse con otros extractors libremente.

---

#### 3.2 Implementación completa — `CurrentUser`

Valida el token Bearer contra la tabla `authtoken_token` de Django.

```rust
// src/auth/extractors.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, header::AUTHORIZATION},
};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use crate::{AppState, error::AppError, entities::{authtoken_token, users}};

/// Extrae y valida el token Bearer.
/// Retorna 401 automáticamente si:
///   - No hay header Authorization
///   - El formato no es "Token <key>"
///   - El token no existe en la DB
///   - El usuario asociado tiene is_active = false
pub struct CurrentUser(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // 1. Extraer header Authorization
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // 2. Validar formato "Token <key>"
        //    Django REST Framework usa "Token" en lugar de "Bearer"
        let token_key = auth_header
            .strip_prefix("Token ")
            .ok_or(AppError::Unauthorized)?;

        let db = &state.as_ref().db;

        // 3. Buscar token en authtoken_token
        let token = authtoken_token::Entity::find()
            .filter(authtoken_token::Column::Key.eq(token_key))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        // 4. Buscar usuario y verificar que esté activo
        let user = users::Entity::find_by_id(token.user_id)
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        Ok(CurrentUser(user))
    }
}
```

---

#### 3.3 Implementación completa — `WorkspaceMemberGuard`

Reutiliza `CurrentUser` internamente — no duplica la lógica de token.

```rust
// src/auth/extractors.rs (continuación)

use axum::extract::Path;
use std::collections::HashMap;
use crate::entities::workspace_members;

/// Verifica que el usuario autenticado sea miembro del workspace
/// identificado por el path param :slug.
///
/// Retorna:
///   - 401 si el token es inválido
///   - 404 si el workspace no existe
///   - 403 si el usuario no es miembro del workspace
pub struct WorkspaceMemberGuard {
    pub user:   users::Model,
    pub member: workspace_members::Model,
}

#[async_trait]
impl<S> FromRequestParts<S> for WorkspaceMemberGuard
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // Reutilizar CurrentUser — ya valida token + usuario activo
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;

        // Extraer :slug del path — compatible con rutas /api/workspaces/:slug/**
        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound)?;

        let slug = params.get("slug").ok_or(AppError::NotFound)?;

        let db = &state.as_ref().db;

        // Buscar workspace
        let workspace = workspaces::Entity::find()
            .filter(workspaces::Column::Slug.eq(slug.as_str()))
            .filter(workspaces::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

        // Verificar membresía activa
        let member = workspace_members::Entity::find()
            .filter(workspace_members::Column::WorkspaceId.eq(workspace.id))
            .filter(workspace_members::Column::MemberId.eq(user.id))
            .filter(workspace_members::Column::IsActive.eq(true))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Forbidden)?;

        Ok(WorkspaceMemberGuard { user, member })
    }
}
```

---

#### 3.4 Implementación completa — `ProjectMemberGuard`

Verifica membresía en workspace **y** en el proyecto. Cadena completa de auth.

```rust
// src/auth/extractors.rs (continuación)

use crate::entities::{projects, project_members};

/// Verifica membresía en workspace Y en proyecto.
/// Path params requeridos: :slug (workspace) + :project_id
///
/// Cadena de verificación:
///   Token válido → usuario activo → miembro workspace → miembro proyecto
pub struct ProjectMemberGuard {
    pub user:           users::Model,
    pub workspace_member: workspace_members::Model,
    pub project_member: project_members::Model,
}

#[async_trait]
impl<S> FromRequestParts<S> for ProjectMemberGuard
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // Reutilizar WorkspaceMemberGuard — ya verifica token + workspace membership
        let WorkspaceMemberGuard { user, member: workspace_member } =
            WorkspaceMemberGuard::from_request_parts(parts, state).await?;

        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound)?;

        let project_id: Uuid = params
            .get("project_id")
            .and_then(|s| s.parse().ok())
            .ok_or(AppError::NotFound)?;

        let db = &state.as_ref().db;

        // Verificar que el proyecto existe y pertenece al workspace
        let _project = projects::Entity::find_by_id(project_id)
            .filter(projects::Column::WorkspaceId.eq(workspace_member.workspace_id))
            .filter(projects::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

        // Verificar membresía en el proyecto
        let project_member = project_members::Entity::find()
            .filter(project_members::Column::ProjectId.eq(project_id))
            .filter(project_members::Column::MemberId.eq(user.id))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Forbidden)?;

        Ok(ProjectMemberGuard { user, workspace_member, project_member })
    }
}
```

---

#### 3.5 Check de rol dentro del handler

Los guards solo verifican **membresía** — el check de **rol mínimo** se hace
en el handler cuando la operación lo requiere.

##### God mode — Workspace Admin override

En Django (`apps/api/plane/app/permissions/base.py`) existe un bypass explícito:
un usuario con `role = 20` en el **workspace** puede ejecutar cualquier operación
de proyecto aunque su **project role** sea menor al requerido, siempre que sea
miembro activo del proyecto. Este comportamiento debe replicarse en Rust.

```
┌─────────────────────────────────────────────────────────┐
│  ¿Tiene el rol requerido en el proyecto?  ──── Sí ────► OK
│                    │ No
│                    ▼
│  ¿Es Workspace Admin (role=20) Y miembro del proyecto? ─ Sí ─► OK (god mode)
│                    │ No
│                    ▼
│                  403 Forbidden
└─────────────────────────────────────────────────────────┘
```

##### Implementación Rust

```rust
// src/routes/mod.rs

pub const ROLE_GUEST:  i16 = 5;
pub const ROLE_VIEWER: i16 = 10;
pub const ROLE_MEMBER: i16 = 15;
pub const ROLE_ADMIN:  i16 = 20;

/// Check de rol con Workspace Admin override (god mode).
///
/// Pasa si:
///   1. El project_role >= required_role  (camino normal), O
///   2. El workspace_role == ADMIN (20)   (god mode — replica base.py de Django)
///
/// Retorna 403 solo si ninguna condición se cumple.
pub fn require_role(
    project_role:   i16,
    workspace_role: i16,
    required_role:  i16,
) -> Result<(), AppError> {
    // Camino 1 — rol de proyecto suficiente
    if project_role >= required_role {
        return Ok(());
    }
    // Camino 2 — Workspace Admin override (god mode)
    if workspace_role >= ROLE_ADMIN {
        return Ok(());
    }
    Err(AppError::Forbidden)
}

// Ejemplo de uso en handler
async fn update_issue(
    State(state): State<AppState>,
    ProjectMemberGuard { user, workspace_member, project_member }: ProjectMemberGuard,
    Path((slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<UpdateIssueRequest>,
) -> Result<Json<IssueResponse>, AppError> {
    // Member (15) requerido — pero Workspace Admin siempre pasa
    require_role(project_member.role, workspace_member.role, ROLE_MEMBER)?;

    let issue = repositories::issues::update(&state.db, issue_id, payload).await?;
    Ok(Json(issue.into()))
}
```

##### Casos edge documentados en Django

| Situación | project_role | workspace_role | required | Resultado |
|-----------|:---:|:---:|:---:|:---:|
| Member normal con permiso | 15 | 15 | 15 | ✅ OK |
| Viewer sin permiso | 10 | 10 | 15 | ❌ 403 |
| Viewer pero WS Admin | 10 | 20 | 15 | ✅ OK (god mode) |
| Guest pero WS Admin | 5 | 20 | 20 | ✅ OK (god mode) |
| No miembro del proyecto | — | 20 | 15 | ❌ 403 — el god mode requiere ser miembro del proyecto |

> **Importante:** el god mode NO aplica si el usuario no es miembro del proyecto.
> `ProjectMemberGuard` ya retorna 403 antes de llegar al check de rol si no
> existe fila en `project_members`. El override es de **rol**, no de membresía.

---

#### 3.6 Composición — orden de extractors en la firma

Axum ejecuta los extractors **en orden de izquierda a derecha** en la firma.
El orden correcto es siempre: `State` → `Guard` → `Path` → `Query` → `Json`:

```rust
// ✅ Correcto — State primero, body (Json) último
async fn update_issue(
    State(state): State<AppState>,          // 1. AppState
    ProjectMemberGuard { user, .. }: ProjectMemberGuard, // 2. Auth + RBAC
    Path((slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>, // 3. Path
    Query(params): Query<IssueQueryParams>, // 4. Query string
    Json(payload): Json<UpdateIssueRequest>, // 5. Body — SIEMPRE último
) -> Result<Json<IssueResponse>, AppError> { … }

// ❌ Incorrecto — Json antes del Guard consume el body,
//    el Guard no puede leerlo después (FromRequestParts no accede al body)
async fn update_issue(
    Json(payload): Json<UpdateIssueRequest>, // ← consume el body demasiado pronto
    ProjectMemberGuard { .. }: ProjectMemberGuard,
) -> … { … }
```

> **Regla:** `Json` y cualquier extractor que implemente `FromRequest`
> (no `FromRequestParts`) debe ir **al final** de la firma porque consume
> el body del request — operación que solo puede hacerse una vez.

---

#### 3.7 Diagrama de flujo interno del extractor chain

```mermaid
flowchart TD
    REQ([HTTP Request]) --> AX[Axum Router\nmatch route]
    AX --> E1[Extractor 1: State\nAppState — siempre ok]
    E1 --> E2[Extractor 2: ProjectMemberGuard]

    subgraph PMG[ProjectMemberGuard internamente]
        direction TB
        T1[Leer header Authorization] --> T2{Formato Token XYZ?}
        T2 -- No --> R401A[Reject 401]
        T2 -- Sí --> T3[SELECT authtoken_token WHERE key=?]
        T3 --> T4{Existe?}
        T4 -- No --> R401B[Reject 401]
        T4 -- Sí --> T5[SELECT users WHERE id=?]
        T5 --> T6{is_active?}
        T6 -- No --> R401C[Reject 401]
        T6 -- Sí --> T7[Leer :slug del path]
        T7 --> T8[SELECT workspaces WHERE slug=? AND deleted_at IS NULL]
        T8 --> T9{Existe?}
        T9 -- No --> R404A[Reject 404]
        T9 -- Sí --> T10[SELECT workspace_members\nWHERE workspace_id=? AND member_id=? AND is_active=true]
        T10 --> T11{Es miembro WS?}
        T11 -- No --> R403A[Reject 403]
        T11 -- Sí --> T12[Leer :project_id del path]
        T12 --> T13[SELECT projects WHERE id=? AND workspace_id=? AND deleted_at IS NULL]
        T13 --> T14{Existe?}
        T14 -- No --> R404B[Reject 404]
        T14 -- Sí --> T15[SELECT project_members\nWHERE project_id=? AND member_id=?]
        T15 --> T16{Es miembro proyecto?}
        T16 -- No --> R403B[Reject 403]
        T16 -- Sí --> OK[Ok — ProjectMemberGuard]
    end

    E2 --> PMG
    R401A & R401B & R401C --> RESP401([Response 401\nsin llegar al handler])
    R403A & R403B --> RESP403([Response 403\nsin llegar al handler])
    R404A & R404B --> RESP404([Response 404\nsin llegar al handler])
    OK --> E3[Extractor 3: Path params]
    E3 --> E4[Extractor 4: Json body]
    E4 --> HANDLER[Handler ejecuta\nlógica de negocio]
    HANDLER --> RESP200([Response 200 / 201 / 204])
```

---

#### 3.8 Testing de extractors

Los extractors se testean con `axum-test` levantando un router real contra
una DB de test — no hace falta mockear HTTP:

```rust
// tests/auth_extractors.rs
use axum_test::TestServer;
use sea_orm::Database;

async fn build_test_app() -> TestServer {
    let db = Database::connect("postgres://...test_db").await.unwrap();
    let state = AppState { db, ..Default::default() };
    let app = Router::new()
        .route("/api/workspaces/:slug/projects/:project_id/issues",
               get(list_issues))
        .with_state(state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_no_token_returns_401() {
    let server = build_test_app().await;
    let resp = server.get("/api/workspaces/my-ws/projects/abc/issues").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_non_member_returns_403() {
    let server = build_test_app().await;
    // Token válido pero usuario no es miembro del workspace
    let resp = server
        .get("/api/workspaces/other-ws/projects/abc/issues")
        .add_header("Authorization", "Token valid_token_other_user")
        .await;
    resp.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_member_can_list_issues() {
    let server = build_test_app().await;
    let resp = server
        .get("/api/workspaces/my-ws/projects/abc/issues")
        .add_header("Authorization", "Token valid_token_member")
        .await;
    resp.assert_status(StatusCode::OK);
}
```

---

#### 3.9 Comparación con el enfoque Django

| Aspecto | Django DRF | Axum Extractor Pattern |
|---------|-----------|------------------------|
| Auth | `permission_classes = [IsAuthenticated]` en cada ViewSet | `CurrentUser` extractor en la firma del handler |
| RBAC | `BaseWorkspacePermissions` herencia de clases | `WorkspaceMemberGuard` / `ProjectMemberGuard` composición |
| Error 401/403 | Raises `PermissionDenied` / `NotAuthenticated` | `type Rejection = AppError` retornado automáticamente |
| Reutilización | Herencia — `WorkSpaceAdminPermission(WorkSpaceBasePermission)` | Composición — `WorkspaceMemberGuard` llama a `CurrentUser` internamente |
| Testeo | `self.client.force_authenticate(user)` | `TestServer` con token real en header |
| Middleware global | `DEFAULT_AUTHENTICATION_CLASSES` en settings.py | No existe — cada handler declara lo que necesita |

---

### 4. Error unificado — AppError

```rust
// src/error.rs
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound      => (StatusCode::NOT_FOUND,            self.to_string()),
            AppError::Unauthorized  => (StatusCode::UNAUTHORIZED,         self.to_string()),
            AppError::Forbidden     => (StatusCode::FORBIDDEN,            self.to_string()),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            AppError::Database(_)   => (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()),
            AppError::Internal(_)   => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

Todos los handlers retornan `Result<T, AppError>`. Los errores SeaORM
y `anyhow` se convierten automáticamente via `#[from]`.

---

### 5. Job Pattern — apalis workers

Cada job sigue el mismo patrón de registro. Todos los workers corren en
el **mismo proceso** que Axum (mismo binario Tokio), eliminando RabbitMQ
y los contenedores bgworker/beatworker:

```rust
// src/jobs/mod.rs
pub fn build_monitor(db: DatabaseConnection) -> Monitor {
    let storage = PostgresStorage::new(db.clone());

    Monitor::new()
        .register(
            WorkerBuilder::new("workspace-seed-worker")
                .data(db.clone())
                .build_fn(workspace_seed::handle_workspace_seed),
        )
        .register(
            WorkerBuilder::new("github-sync-worker")
                .data(db.clone())
                .build_fn(github_sync::handle_github_sync),
        )
        .register(
            WorkerBuilder::new("notification-worker")
                .data(db.clone())
                .build_fn(notifications::handle_notification),
        )
}
```

---

### 6. Cron jobs — tokio-cron-scheduler

Reemplaza Celery beatworker. Corre en el mismo proceso:

```rust
// src/jobs/scheduled.rs
pub async fn start_scheduler(db: DatabaseConnection) -> anyhow::Result<()> {
    let scheduler = JobScheduler::new().await?;

    // Limpieza de tokens expirados (diario 3am UTC)
    scheduler.add(
        Job::new_async("0 0 3 * * *", move |_, _| {
            let db = db.clone();
            Box::pin(async move {
                if let Err(e) = cleanup_expired_tokens(&db).await {
                    tracing::error!("Token cleanup failed: {e}");
                }
            })
        })?
    ).await?;

    scheduler.start().await?;
    Ok(())
}
```

---

