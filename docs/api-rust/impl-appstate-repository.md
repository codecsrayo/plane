---
titulo: AppState y Repository Pattern
aliases:
  - appstate
  - repository-pattern
  - repositorios
tags:
  - rust
  - axum
  - repository-pattern
  - appstate
  - pendiente-implementar
relacionado:
  - "[[MOC]]"
  - "[[vision-stack]]"
  - "[[impl-extractores-auth]]"
  - "[[impl-error-jobs-cron]]"
  - "[[fundamentos-soft-delete]]"
  - "[[ref-estructura-archivos]]"
estado: activo
---

# AppState y Repository Pattern

> **Documentación oficial:**
>
> - [SeaORM — Queries](https://www.sea-ql.org/SeaORM/docs/basic-crud/select/)
> - [Axum — State](https://docs.rs/axum/latest/axum/extract/struct.State.html)
> - [Axum examples/todos](https://github.com/tokio-rs/axum/tree/main/examples/todos)
> - [Rust API Guidelines — Modules](https://rust-lang.github.io/api-guidelines/organization.html)

---

## 1. AppState — estado global del servidor

```rust
// src/main.rs
use std::sync::Arc;
use crate::auth::rate_limit::RateLimitState;

#[derive(Clone)]
pub struct AppState {
    pub db:          sea_orm::DatabaseConnection,  // pool SeaORM (Postgres)
    pub redis:       fred::clients::Pool,           // pool Redis/Valkey
    pub s3:          aws_sdk_s3::Client,            // cliente S3/MinIO
    pub config:      Arc<Config>,                   // env vars tipadas (dotenvy)
    /// Pool sqlx puro — extraído de `db` vía `get_postgres_connection_pool()`.
    /// apalis-sql lo requiere como `sqlx::PgPool`, no como `DatabaseConnection`.
    /// Se usa para `PostgresStorage::setup()` y para encolar jobs.
    pub pg_pool:     sqlx::PgPool,                 // apalis backend (Postgres)
    pub rate_limit:  Arc<RateLimitState>,           // rate limit API keys (en memoria → Redis Fase 3)
}
```

Se registra en Axum con `.with_state(state)`. Los handlers lo reciben con `State(state): State<AppState>`.

> [!NOTE] Clone es barato
> `AppState` implementa `Clone` porque cada campo es un `Arc` o un pool de conexiones — no se copian datos, solo se incrementan contadores de referencia.

---

## 2. Repository Pattern — aislar SeaORM de los handlers

Los handlers Axum **no deben contener queries SeaORM directamente**. El módulo `src/repositories/` encapsula todo el acceso a datos.

### Estructura de módulos

```tree
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

### Ejemplo — `repositories/issues.rs`

```rust
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;
use crate::{entities::issues, error::AppError, utils::soft_delete::SoftDeleteExt};

pub struct IssueFilters {
    pub state_id:    Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub priority:    Option<String>,
    pub label_id:    Option<Uuid>,
}

pub async fn list_issues(
    db: &DatabaseConnection,
    project_id: Uuid,
    filters: IssueFilters,
) -> Result<Vec<issues::Model>, sea_orm::DbErr> {
    let mut query = issues::Entity::find()
        .active()   // WHERE deleted_at IS NULL — via SoftDeleteExt
        .filter(issues::Column::ProjectId.eq(project_id))
        .order_by_asc(issues::Column::SortOrder);

    if let Some(state_id) = filters.state_id {
        query = query.filter(issues::Column::StateId.eq(state_id));
    }
    if let Some(assignee_id) = filters.assignee_id {
        query = query.filter(issues::Column::AssigneeId.eq(assignee_id));
    }

    query.all(db).await
}

pub async fn get_issue_by_id(
    db: &DatabaseConnection,
    issue_id: Uuid,
) -> Result<Option<issues::Model>, sea_orm::DbErr> {
    issues::Entity::find_by_id(issue_id)
        .filter(issues::Column::DeletedAt.is_null())
        .one(db)
        .await
}

pub async fn create_issue(
    db: &DatabaseConnection,
    payload: CreateIssueInput,
) -> Result<issues::Model, sea_orm::DbErr> {
    issues::ActiveModel {
        id:         Set(Uuid::new_v4()),
        name:       Set(payload.name),
        project_id: Set(payload.project_id),
        state_id:   Set(payload.state_id),
        priority:   Set(payload.priority.unwrap_or("none".into())),
        sort_order: Set(0.0),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn soft_delete_issue(
    db: &DatabaseConnection,
    issue_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let issue = issues::Entity::find_by_id(issue_id)
        .one(db).await?
        .ok_or(sea_orm::DbErr::RecordNotFound("issue".into()))?;

    let mut active: issues::ActiveModel = issue.into();
    active.deleted_at = Set(Some(chrono::Utc::now().into()));
    active.update(db).await?;
    Ok(())
}
```

### Ventaja principal

Los tests pueden **mockear el repository** sin levantar DB real. El handler solo recibe datos ya validados y tipados.

---

## 3. Inicialización de AppState en `main.rs`

```rust
// src/main.rs (fragmento)
use sea_orm::Database;
use std::sync::Arc;

// 1. Conectar a PostgreSQL
let db = Database::connect(&config.database_url).await?;

// 2. (Fase 2) Conectar a Redis
// let redis = fred::Pool::new(RedisConfig::from_url(&config.redis_url)?, None, None, None, 6)?;
// redis.connect();

// 3. (Fase 3) Inicializar apalis PostgreSQL storage
// PostgresStorage::setup(&db).await?;

// 4. Construir AppState
let state = AppState {
    db,
    config:     Arc::new(config),
    rate_limit: Arc::new(RateLimitState::default()),
    // redis, pg_pool, s3 — agregar en sus respectivas Fases
};
```

---

## 4. Uso en handlers

```rust
// src/routes/issues.rs
use axum::{extract::State, Json};
use crate::{AppState, error::AppError, auth::extractors::ProjectMemberGuard, repositories};

pub async fn list_issues(
    State(state): State<AppState>,
    ProjectMemberGuard { user, project_member, .. }: ProjectMemberGuard,
    Path((_slug, project_id)): Path<(String, Uuid)>,
    Query(params): Query<IssueQueryParams>,
) -> Result<Json<Vec<IssueResponse>>, AppError> {
    let issues = repositories::issues::list_issues(
        &state.db,
        project_id,
        IssueFilters {
            state_id:    params.state_id,
            assignee_id: params.assignee_id,
            priority:    params.priority,
            label_id:    params.label_id,
        },
    )
    .await
    .map_err(AppError::Database)?;

    Ok(Json(issues.into_iter().map(IssueResponse::from).collect()))
}
```

---

## 5. DTOs — separar entidades DB de respuestas API

```rust
// src/routes/issues.rs — DTOs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct IssueResponse {
    pub id:          uuid::Uuid,
    pub name:        String,
    pub priority:    String,
    pub state_id:    Option<uuid::Uuid>,
    pub project_id:  uuid::Uuid,
    pub created_at:  chrono::DateTime<chrono::Utc>,
}

impl From<crate::entities::issues::Model> for IssueResponse {
    fn from(m: crate::entities::issues::Model) -> Self {
        Self {
            id:         m.id,
            name:       m.name,
            priority:   m.priority.unwrap_or_else(|| "none".into()),
            state_id:   m.state_id,
            project_id: m.project_id,
            created_at: m.created_at.into(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct IssueQueryParams {
    pub state_id:    Option<uuid::Uuid>,
    pub assignee_id: Option<uuid::Uuid>,
    pub priority:    Option<String>,
    pub label_id:    Option<uuid::Uuid>,
}
```

---

## Plan de implementación

```
Fase 1:
  [ ] src/main.rs             — construir AppState (db + config + http)

Fase 2:
  [ ] src/repositories/       — módulo con un repositorio por dominio (issues, workspaces…)
  [ ] src/main.rs             — añadir redis (fred::Pool) a AppState

Fase 3:
  [ ] src/main.rs             — añadir apalis PostgresStorage a AppState
  [ ] src/auth/rate_limit.rs  — RateLimitState migrar de DashMap a Redis
```

## 🔗 Navegar

← [[impl-bootstrap]] | [[MOC]] | → [[impl-extractores-auth]] | → [[impl-error-jobs-cron]]

**Relacionado:** [[fundamentos-soft-delete]] (soft delete en queries) | [[ref-estructura-archivos]] (dónde van los repositories)
