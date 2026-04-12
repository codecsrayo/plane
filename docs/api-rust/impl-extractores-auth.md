---
titulo: Extractores de autenticación y RBAC
aliases:
  - extractores
  - CurrentUser
  - WorkspaceMemberGuard
  - ProjectMemberGuard
  - RBAC
tags:
  - rust
  - axum
  - auth
  - rbac
  - extractores
  - pendiente-implementar
  - todo-rs
relacionado:
  - "[[MOC]]"
  - "[[impl-autenticacion]]"
  - "[[impl-appstate-repository]]"
  - "[[impl-error-jobs-cron]]"
  - "[[ref-diagramas-flujo]]"
  - "[[ref-diagramas-secuencia]]"
estado: activo
---

# Extractores de autenticación y RBAC (Axum Extractor Pattern)

> **Documentación oficial:**
>
> - [Axum — `FromRequestParts` trait](https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html)
> - [Axum — `FromRequest` trait](https://docs.rs/axum/latest/axum/extract/trait.FromRequest.html)
> - [Axum — custom extractor error](https://github.com/tokio-rs/axum/blob/main/examples/customize-extractor-error/src/main.rs)
> - [async-trait crate](https://docs.rs/async-trait/latest/async_trait/)
>
> Diagrama de flujo completo → [[ref-diagramas-flujo]]

---

## Concepto central

Axum permite extractors personalizados que corren **antes** del handler, implementando auth + RBAC sin middleware global. Cada extractor implementa `FromRequestParts` y retorna un `Rejection` automático si falla — el handler nunca llega a ejecutarse.

### `FromRequestParts` vs `FromRequest`

| Trait              | Lee el body | Cuándo usarlo                                                               |
| ------------------ | :---------: | --------------------------------------------------------------------------- |
| `FromRequestParts` |     ❌      | Auth, headers, path params, query params — **usar siempre que sea posible** |
| `FromRequest`      |     ✅      | Solo cuando necesitas leer el body (JSON, form data)                        |

Los extractores de auth **siempre** implementan `FromRequestParts`.

---

## Extractor 1 — `CurrentUser`

Valida el token Bearer contra la tabla `authtoken_token`.

```rust
// src/auth/extractors.rs

use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
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
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let app_state = AppState::from_ref(state);
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Django REST Framework usa "Token" en lugar de "Bearer"
        let token_key = auth_header
            .strip_prefix("Token ")
            .ok_or(AppError::Unauthorized)?;

        // DRF genera tokens de exactamente 40 hex chars.
        // Rechazar cualquier valor distinto evita queries innecesarias con input inválido.
        if token_key.len() > 40 {
            return Err(AppError::Unauthorized);
        }

        let db = &app_state.db;

        let token = authtoken_token::Entity::find()
            .filter(authtoken_token::Column::Key.eq(token_key))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

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

## Extractor 2 — `WorkspaceMemberGuard`

Reutiliza `CurrentUser` internamente — no duplica la lógica de token.

```rust
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
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // Reutilizar CurrentUser — ya valida token + usuario activo
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;

        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound)?;

        let slug = params.get("slug").ok_or(AppError::NotFound)?;
        let db = &AppState::from_ref(state).db;

        let workspace = workspaces::Entity::find()
            .filter(workspaces::Column::Slug.eq(slug.as_str()))
            .filter(workspaces::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

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

## Extractor 3 — `ProjectMemberGuard`

Verifica membresía en workspace **y** en el proyecto. Cadena completa.

```rust
use crate::entities::{projects, project_members};

/// Cadena de verificación:
///   Token válido → usuario activo → miembro workspace → miembro proyecto
pub struct ProjectMemberGuard {
    pub user:             users::Model,
    pub workspace_member: workspace_members::Model,
    pub project_member:   project_members::Model,
}

#[async_trait]
impl<S> FromRequestParts<S> for ProjectMemberGuard
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        // Reutilizar WorkspaceMemberGuard
        let WorkspaceMemberGuard { user, member: workspace_member } =
            WorkspaceMemberGuard::from_request_parts(parts, state).await?;

        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::NotFound)?;

        let project_id: Uuid = params
            .get("project_id")
            .and_then(|s| s.parse().ok())
            .ok_or(AppError::NotFound)?;

        let db = &AppState::from_ref(state).db;

        let _project = projects::Entity::find_by_id(project_id)
            .filter(projects::Column::WorkspaceId.eq(workspace_member.workspace_id))
            .filter(projects::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)?;

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

## Verificación de rol (god mode)

Los guards solo verifican **membresía** — el check de **rol mínimo** se hace en el handler.

### Constantes de rol

```rust
// src/auth/permissions.rs  ← ubicación correcta (no routes/mod.rs)
// Ver ref-estructura-archivos.md — ROLE_* viven en auth/, no en routes/
pub const ROLE_GUEST:  i16 = 5;
pub const ROLE_VIEWER: i16 = 10;
pub const ROLE_MEMBER: i16 = 15;
pub const ROLE_ADMIN:  i16 = 20;
```

### Workspace Admin override (god mode)

En Django (`apps/api/plane/app/permissions/base.py`) existe un bypass explícito:
un usuario con `role = 20` en el **workspace** puede ejecutar cualquier operación de proyecto aunque su **project role** sea menor al requerido, siempre que sea miembro activo del proyecto.

```mermaid
flowchart TD
    A{¿Tiene el rol requerido\nen el proyecto?} -- Sí --> OK([✅ OK])
    A -- No --> B{¿Es Workspace Admin (20)\nY miembro del proyecto?}
    B -- Sí --> OK
    B -- No --> ERR([❌ 403 Forbidden])
```

```rust
/// Check de rol con Workspace Admin override.
/// Pasa si:
///   1. project_role >= required_role  (camino normal)
///   2. workspace_role >= ROLE_ADMIN   (god mode)
pub fn require_role(
    project_role:   i16,
    workspace_role: i16,
    required_role:  i16,
) -> Result<(), AppError> {
    if project_role >= required_role {
        return Ok(());
    }
    if workspace_role >= ROLE_ADMIN {
        return Ok(());
    }
    Err(AppError::Forbidden)
}

// Uso en handler:
async fn update_issue(
    State(state): State<AppState>,
    ProjectMemberGuard { user, workspace_member, project_member }: ProjectMemberGuard,
    Path((slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<UpdateIssueRequest>,
) -> Result<Json<IssueResponse>, AppError> {
    require_role(project_member.role, workspace_member.role, ROLE_MEMBER)?;
    // ...
}
```

### Tabla de casos edge

| Situación                 | project_role | workspace_role | required |               Resultado                |
| ------------------------- | :----------: | :------------: | :------: | :------------------------------------: |
| Member normal con permiso |      15      |       15       |    15    |                 ✅ OK                  |
| Viewer sin permiso        |      10      |       10       |    15    |                 ❌ 403                 |
| Viewer pero WS Admin      |      10      |       20       |    15    |            ✅ OK (god mode)            |
| Guest pero WS Admin       |      5       |       20       |    20    |            ✅ OK (god mode)            |
| No miembro del proyecto   |      —       |       20       |    15    | ❌ 403 — god mode requiere ser miembro |

> **Importante:** el god mode NO aplica si el usuario no es miembro del proyecto. `ProjectMemberGuard` retorna 403 antes de llegar al check de rol.

---

## Orden de extractors en la firma del handler

Axum ejecuta los extractors en orden de **izquierda a derecha**:

```rust
// ✅ Correcto — State primero, body (Json) último
async fn update_issue(
    State(state): State<AppState>,            // 1. AppState
    ProjectMemberGuard { user, .. }: ProjectMemberGuard, // 2. Auth + RBAC
    Path((slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>, // 3. Path
    Query(params): Query<IssueQueryParams>,   // 4. Query string
    Json(payload): Json<UpdateIssueRequest>,  // 5. Body — SIEMPRE último
) -> Result<Json<IssueResponse>, AppError> { … }

// ❌ Incorrecto — Json antes del Guard consume el body
async fn update_issue(
    Json(payload): Json<UpdateIssueRequest>, // ← consume el body demasiado pronto
    ProjectMemberGuard { .. }: ProjectMemberGuard,
) -> … { … }
```

> **Regla:** `Json` y cualquier extractor que implemente `FromRequest` (no `FromRequestParts`) debe ir **al final** porque consume el body del request — operación que solo puede hacerse una vez.

---

## Diagrama de flujo del extractor chain

```mermaid
flowchart TD
    REQ([HTTP Request]) --> AX[Axum Router\nmatch route]
    AX --> E1[Extractor 1: State\nAppState]
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
        T7 --> T8[SELECT workspaces WHERE slug=?]
        T8 --> T9{Existe?}
        T9 -- No --> R404A[Reject 404]
        T9 -- Sí --> T10[SELECT workspace_members]
        T10 --> T11{Es miembro WS?}
        T11 -- No --> R403A[Reject 403]
        T11 -- Sí --> T12[Leer :project_id]
        T12 --> T13[SELECT projects WHERE id=?]
        T13 --> T14{Existe?}
        T14 -- No --> R404B[Reject 404]
        T14 -- Sí --> T15[SELECT project_members]
        T15 --> T16{Es miembro proyecto?}
        T16 -- No --> R403B[Reject 403]
        T16 -- Sí --> OK[Ok — ProjectMemberGuard]
    end

    R401A & R401B & R401C --> RESP401([Response 401])
    R403A & R403B --> RESP403([Response 403])
    R404A & R404B --> RESP404([Response 404])
    OK --> E3[Extractor 3: Path params]
    E3 --> E4[Extractor 4: Json body]
    E4 --> HANDLER[Handler]
    HANDLER --> RESP200([Response 200/201/204])
```

---

## Testing de extractors

```rust
// tests/auth_extractors.rs
use axum_test::TestServer;

async fn build_test_app() -> TestServer {
    let db = Database::connect("postgres://...test_db").await.unwrap();
    let state = AppState { db, ..Default::default() };
    let app = Router::new()
        .route(
            "/api/workspaces/:slug/projects/:project_id/issues",
            get(list_issues)
        )
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
    let resp = server
        .get("/api/workspaces/other-ws/projects/abc/issues")
        .add_header("Authorization", "Token valid_token_other_user")
        .await;
    resp.assert_status(StatusCode::FORBIDDEN);
}
```

---

## Comparación con Django

| Aspecto           | Django DRF                                               | Axum Extractor Pattern                                     |
| ----------------- | -------------------------------------------------------- | ---------------------------------------------------------- |
| Auth              | `permission_classes = [IsAuthenticated]` en cada ViewSet | `CurrentUser` extractor en la firma del handler            |
| RBAC              | `BaseWorkspacePermissions` herencia de clases            | Composición de guards                                      |
| Error 401/403     | Raises `PermissionDenied`                                | `type Rejection = AppError` automático                     |
| Reutilización     | Herencia                                                 | Composición — `WorkspaceMemberGuard` llama a `CurrentUser` |
| Testeo            | `self.client.force_authenticate(user)`                   | `TestServer` con token real en header                      |
| Middleware global | `DEFAULT_AUTHENTICATION_CLASSES` en settings.py          | No existe — cada handler declara lo que necesita           |

---

## Plan de implementación

```
Fase 1:
  [ ] src/auth/extractors.rs   — CurrentUser (Bearer token)
                                  WorkspaceMemberGuard (slug → workspace + member)
                                  ProjectMemberGuard (project_id → project + member)
  [ ] src/auth/permissions.rs  — constantes ROLE_GUEST/VIEWER/MEMBER/ADMIN
                                  fn require_role() con Workspace Admin override
                                  (NO en routes/mod.rs — auth vive en src/auth/)
```

## 🔗 Navegar

← [[impl-autenticacion]] | [[MOC]] | → [[impl-error-jobs-cron]]

**Relacionado:** Diagramas: [[ref-diagramas-flujo]] | Auth Session/APIKey: [[impl-autenticacion]] | Testing: [[ref-testing]]
