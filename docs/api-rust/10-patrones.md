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

Axum permite extractors personalizados que corren **antes** del handler,
implementando auth + RBAC sin middleware global:

```rust
// src/auth/middleware.rs

/// Extrae y valida el token desde la tabla authtoken_token.
/// Si falla → 401 automático antes de entrar al handler.
pub struct CurrentUser(pub users::Model);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts)?;
        let user = validate_token(&state.as_ref().db, &token).await?;
        Ok(CurrentUser(user))
    }
}

/// Extrae workspace_member con su rol.
/// Falla con 403 si el usuario no es miembro del workspace.
pub struct WorkspaceMemberGuard {
    pub user:   users::Model,
    pub member: workspace_members::Model,
}

/// Extrae project_member. Verifica membership en workspace Y proyecto.
pub struct ProjectMemberGuard {
    pub user:           users::Model,
    pub project_member: project_members::Model,
}
```

Uso en handlers — los guards se componen directamente en la firma:

```rust
// Solo autenticación
async fn list_workspaces(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,  // ← 401 si token inválido
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> { … }

// Auth + membership en workspace
async fn get_project(
    State(state): State<AppState>,
    WorkspaceMemberGuard { user, member }: WorkspaceMemberGuard, // ← 403 si no es miembro
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<ProjectResponse>, AppError> { … }
```

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

