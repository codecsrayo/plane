---
titulo: AppError, Job Pattern (apalis) y Cron
aliases:
  - apperror
  - jobs
  - apalis
  - cron
  - tokio-cron
tags:
  - rust
  - axum
  - apalis
  - error
  - jobs
  - cron
  - pendiente-implementar
  - todo-rs
relacionado:
  - "[[MOC]]"
  - "[[impl-appstate-repository]]"
  - "[[impl-extractores-auth]]"
  - "[[dominio-workspace-seed]]"
  - "[[vision-stack]]"
  - "[[plan-riesgos]]"
estado: activo
---

# AppError, Job Pattern (apalis) y Cron

> **Documentación oficial:**
>
> - [thiserror crate](https://docs.rs/thiserror/latest/thiserror/)
> - [Axum — IntoResponse](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html)
> - [apalis — Book](https://docs.rs/apalis/latest/apalis/)
> - [apalis — postgres example](https://github.com/geofmureithi/apalis/tree/main/examples/postgres)
> - [tokio-cron-scheduler](https://docs.rs/tokio-cron-scheduler/latest/tokio_cron_scheduler/)

---

## 1. AppError — error unificado

```rust
// src/error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;

/// Error central de la aplicación.
/// Todos los handlers retornan `Result<T, AppError>`.
/// Equivalente a las excepciones DRF en Django.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    /// HTTP 429 — rate limit superado para este API key.
    /// Retornado por `apply_rate_limit()` en `src/auth/api_key.rs`.
    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Convierte AppError en respuesta HTTP con body JSON.
/// Axum llama esto automáticamente cuando un handler retorna Err(AppError).
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound      => (StatusCode::NOT_FOUND,            "Not found".into()),
            AppError::Unauthorized  => (StatusCode::UNAUTHORIZED,         "Unauthorized".into()),
            AppError::Forbidden     => (StatusCode::FORBIDDEN,            "Forbidden".into()),
            AppError::RateLimited   => (StatusCode::TOO_MANY_REQUESTS,    "Rate limit exceeded".into()),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            AppError::Database(e)   => {
                // ⚠️ Loguear internamente — NUNCA exponer `e` al cliente:
                // puede contener connection strings, schema names u otros detalles internos.
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            AppError::Internal(e)   => {
                tracing::error!(error = %e, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

Todos los handlers retornan `Result<T, AppError>`. Los errores SeaORM y `anyhow` se convierten automáticamente via `#[from]`.

---

## 2. Job Pattern — apalis workers

Cada job sigue el mismo patrón de registro. Todos los workers corren en el **mismo proceso** que Axum (mismo binario Tokio), eliminando RabbitMQ y los contenedores bgworker/beatworker.

### Patrón de un job

```rust
// src/jobs/notifications.rs
use apalis::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationJob {
    pub user_id:    Uuid,
    pub issue_id:   Uuid,
    pub event_type: String,
}

pub async fn handle_notification(
    job: NotificationJob,
    ctx: Data<sea_orm::DatabaseConnection>,
) -> Result<(), apalis::prelude::Error> {
    let db = ctx.as_ref();
    // lógica de notificación...
    tracing::info!("Notification sent to user {}", job.user_id);
    Ok(())
}
```

### Registro de todos los workers — `src/jobs/mod.rs`

```rust
use apalis::{prelude::*, layers::TraceLayer};
use apalis_sql::postgres::PostgresStorage;

pub fn build_monitor(db: sea_orm::DatabaseConnection) -> Monitor {
    // ✅ PostgresStorage::new() requiere sqlx::PgPool, NO sea_orm::DatabaseConnection.
    // SeaORM expone el pool interno vía get_postgres_connection_pool().
    let pool = db.get_postgres_connection_pool().clone();
    let storage = PostgresStorage::new(pool);

    Monitor::new()
        .register(
            WorkerBuilder::new("workspace-seed-worker")
                .layer(TraceLayer::new())
                .data(db.clone())
                .build_fn(workspace_seed::handle_workspace_seed),
        )
        .register(
            WorkerBuilder::new("github-sync-worker")
                .layer(TraceLayer::new())
                .data(db.clone())
                .build_fn(github_sync::handle_github_initial_sync),
        )
        .register(
            WorkerBuilder::new("notification-worker")
                .layer(TraceLayer::new())
                .data(db.clone())
                .build_fn(notifications::handle_notification),
        )
        .register(
            WorkerBuilder::new("export-worker")
                .layer(TraceLayer::new())
                .data(db.clone())
                .build_fn(export::handle_export),
        )
        .register(
            WorkerBuilder::new("webhook-delivery-worker")
                .layer(TraceLayer::new())
                .data(db.clone())
                .build_fn(webhooks::handle_webhook_delivery),
        )
}
```

### Tabla de Celery tasks → apalis jobs

| Celery task               | Equivalente apalis          | Trigger                                              |
| ------------------------- | --------------------------- | ---------------------------------------------------- |
| `workspace_seed_task`     | `WorkspaceSeedJob`          | Workspace creado — ver [[dominio-workspace-seed]]    |
| `github_sync_task`        | `GithubInitialIssueSyncJob` | Repo sync creado — ver [[dominio-integraciones]]     |
| `notification_task`       | `NotificationJob`           | Cola de eventos                                      |
| `email_notification_task` | `EmailJob`                  | Cola de eventos                                      |
| `webhook_task`            | `WebhookDeliveryJob`        | Post-mutación — ver [[dominio-workspace-settings]]   |
| `export_task`             | `ExportJob`                 | Request usuario — ver [[dominio-workspace-settings]] |
| `cleanup_task`            | `CleanupCron`               | tokio-cron-scheduler (Fase 3)                        |
| `issue_automation_task`   | `AutomationCron`            | Cron diario (Fase 4)                                 |
| `magic_link_code_task`    | —                           | Pendiente — depende de auth Rust completo            |

---

## 3. Inicialización de apalis en `main.rs`

> [!IMPORTANT] `PostgresStorage::setup` es obligatorio antes del primer job
> Sin esto el primer push/pull falla con "table not found". Ver [[plan-riesgos]] punto 10.

```rust
// src/main.rs (fragmento)
use apalis_sql::postgres::PostgresStorage;

// Al arrancar, antes de registrar workers:
// ✅ setup() también necesita sqlx::PgPool — extraer antes de construir AppState
let pg_pool = db.get_postgres_connection_pool().clone();
PostgresStorage::setup(&pg_pool).await?;

// Construir y arrancar el monitor en una tarea Tokio paralela
let monitor = jobs::build_monitor(db.clone());
tokio::spawn(async move {
    if let Err(e) = monitor.run().await {
        tracing::error!("Job monitor error: {e}");
    }
});
```

### Encolar un job desde un handler

> Ejemplo completo con contexto de workspace: [[dominio-workspace-seed#Encolar el job desde el handler de workspaces]].

```rust
// Patrón genérico — en cualquier handler POST que dispara un job
let mut storage = PostgresStorage::<MiJob>::new(state.pg_pool.clone());
if let Err(e) = storage.push(MiJob { ... }).await {
    tracing::warn!("Failed to enqueue job: {e}");
    // No fallar la request — jobs son best-effort salvo indicación contraria
}
```

---

## 4. Cron jobs — tokio-cron-scheduler

Reemplaza Celery beatworker. Corre en el **mismo proceso** que Axum:

```rust
// src/jobs/scheduled.rs
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start_scheduler(db: sea_orm::DatabaseConnection) -> anyhow::Result<()> {
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

    // Digest de notificaciones (diario 8am UTC)
    scheduler.add(
        Job::new_async("0 0 8 * * *", move |_, _| {
            let db = db.clone();
            Box::pin(async move {
                if let Err(e) = send_notification_digests(&db).await {
                    tracing::error!("Notification digest failed: {e}");
                }
            })
        })?
    ).await?;

    scheduler.start().await?;
    Ok(())
}
```

### Cron tasks de Django → Rust

| Celery beat task         | Equivalente Rust            | Cron expression                |
| ------------------------ | --------------------------- | ------------------------------ |
| Limpiar tokens expirados | `cleanup_expired_tokens`    | `0 0 3 * * *` (3am UTC diario) |
| Digest notificaciones    | `send_notification_digests` | `0 0 8 * * *` (8am UTC diario) |
| Limpieza de assets       | `cleanup_expired_assets`    | `0 0 4 * * *`                  |
| Issue automation         | `run_issue_automation`      | `0 0 0 * * *` (medianoche)     |

> Usar [crontab.guru](https://crontab.guru/) para verificar expresiones cron.

---

## Plan de implementación

```
Fase 1:
  [ ] src/error.rs            — AppError enum completo + IntoResponse

Fase 3:
  [ ] src/jobs/mod.rs         — build_monitor() con todos los workers registrados
  [ ] src/jobs/notifications.rs — NotificationJob handler
  [ ] src/jobs/email.rs         — EmailJob handler (lettre)
  [ ] src/jobs/scheduled.rs     — crons: deadline, snooze intake, cycle progress
```

## 🔗 Navegar

← [[impl-extractores-auth]] | [[MOC]] | → [[dominio-workspace-seed]]

**Relacionado:** Workspace seed (ejemplo completo): [[dominio-workspace-seed]] | Integraciones (GithubSyncJob): [[dominio-integraciones]] | Riesgos: [[plan-riesgos]]
