---
titulo: Extractores de autenticación y RBAC
aliases:
  - extractores
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

## Extractor 1 — `WorkspaceMemberGuard`

> [!NOTE] Sin `CurrentUser` intermedio
> `authtoken_token` (DRF) no existe — no hay justificación para un wrapper.
> Los guards llaman `ApiKeyUser` directamente. Auth base → [[impl-autenticacion]] Parte 2.

Llama a `ApiKeyUser` directamente.

```rust
// src/auth/extractors.rs
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
        let ApiKeyUser(ctx) = ApiKeyUser::from_request_parts(parts, state).await?;
        let user = ctx.user;

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

## Extractor 2 — `ProjectMemberGuard`

Verifica membresía en workspace **y** en el proyecto. Cadena completa.

```rust
// src/auth/extractors.rs
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
// src/auth/permissions.rs
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
// src/routes/issues.rs — ejemplo de uso
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

> Diagrama completo con las 5 queries secuenciales de `ProjectMemberGuard` → [[ref-diagramas-flujo#Extractor chain — ProjectMemberGuard internamente]]

---

## Testing de extractors

`build_test_app()` — helper que levanta el router con estado real de test:

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
```

Los casos de test (`test_no_token_returns_401`, `test_non_member_returns_403`, `test_member_can_list_issues`) están en → [[ref-testing#Tests de extractores de auth]]

---

## Comparación con Django

| Aspecto           | Django DRF                                               | Axum Extractor Pattern                                    |
| ----------------- | -------------------------------------------------------- | --------------------------------------------------------- |
| Auth              | `permission_classes = [IsAuthenticated]` en cada ViewSet | `ApiKeyUser` directo en guards (X-Api-Key → api_tokens)   |
| RBAC              | `BaseWorkspacePermissions` herencia de clases            | Composición de guards                                     |
| Error 401/403     | Raises `PermissionDenied`                                | `type Rejection = AppError` automático                    |
| Reutilización     | Herencia                                                 | Composición — `WorkspaceMemberGuard` llama a `ApiKeyUser` |
| Testeo            | `self.client.force_authenticate(user)`                   | `TestServer` con token real en header                     |
| Middleware global | `DEFAULT_AUTHENTICATION_CLASSES` en settings.py          | No existe — cada handler declara lo que necesita          |

---

## Plan de implementación

```
Fase 1:
  [ ] src/auth/extractors.rs   — WorkspaceMemberGuard (ApiKeyUser → slug → workspace + member)
                                  ProjectMemberGuard (ApiKeyUser → project_id → project + member)
  [ ] src/auth/permissions.rs  — constantes ROLE_GUEST/VIEWER/MEMBER/ADMIN
                                  fn require_role() con Workspace Admin override
                                  (NO en routes/mod.rs — auth vive en src/auth/)
```

## 🔗 Navegar

← [[impl-autenticacion]] | [[MOC]] | → [[impl-error-jobs-cron]]

**Relacionado:** Diagramas: [[ref-diagramas-flujo]] | Auth Session/APIKey: [[impl-autenticacion]] | Testing: [[ref-testing]]

---

_`docs/api-rust/impl-extractores-auth.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
