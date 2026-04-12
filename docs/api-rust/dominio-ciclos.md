---
titulo: Dominio — Cycles (Sprints)
aliases:
  - cycles
  - sprints
  - dominio-ciclos
tags:
  - cycles
  - dominio
  - rust
  - axum
  - pendiente-implementar
  - todo-rs
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-proyectos]]"
  - "[[impl-extractores-auth]]"
  - "[[plan-fases]]"
estado: activo
---

# Dominio — Cycles (Sprints)

> [!NOTE] Equivalente a Sprints en Scrum
> Un ciclo agrupa issues en un período de tiempo. Un issue puede estar en máximo 1 ciclo activo a la vez.

---

## Modelo de datos

```tree
projects
    └─ cycles
            ├─ cycle_issues           (M2M → issues)
            └─ cycle_user_properties  (preferencias de vista por usuario)
```

**Tipos de ciclo (`status`):**

- `CURRENT` — ciclo activo hoy (start_date ≤ hoy ≤ end_date)
- `UPCOMING` — ciclo futuro (start_date > hoy)
- `COMPLETED` — ciclo pasado (end_date < hoy)
- `DRAFT` — sin fechas asignadas

---

## Endpoints a implementar

### CRUD de ciclo

| Método             | URL                                             | Guard                        | Fase |
| ------------------ | ----------------------------------------------- | ---------------------------- | ---- |
| `GET`              | `/workspaces/{slug}/projects/{id}/cycles/`      | `ProjectMemberGuard (≥5)`    | 2    |
| `POST`             | `/workspaces/{slug}/projects/{id}/cycles/`      | `ProjectMemberGuard (≥15)`   | 2    |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/projects/{id}/cycles/{pk}/` | `ProjectMemberGuard (≥5/15)` | 2    |

### Issues del ciclo

| Método                 | URL                                                                           | Guard                        | Fase |
| ---------------------- | ----------------------------------------------------------------------------- | ---------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/cycle-issues/`            | `ProjectMemberGuard (≥5/15)` | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/cycle-issues/{issue_id}/` | `ProjectMemberGuard (≥15)`   | 4    |

### Operaciones especiales

| Método        | URL                                                                   | Guard                      | Fase |
| ------------- | --------------------------------------------------------------------- | -------------------------- | ---- |
| `POST`        | `/workspaces/{slug}/projects/{id}/cycles/date-check/`                 | `ProjectMemberGuard (≥5)`  | 4    |
| `POST`        | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/transfer-issues/` | `ProjectMemberGuard (≥15)` | 4    |
| `POST/DELETE` | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/archive/`         | `ProjectMemberGuard (≥15)` | 4    |
| `GET`         | `/workspaces/{slug}/projects/{id}/archived-cycles/`                   | `ProjectMemberGuard (≥5)`  | 4    |
| `GET`         | `/workspaces/{slug}/projects/{id}/archived-cycles/{pk}/`              | `ProjectMemberGuard (≥5)`  | 4    |

> [!WARNING] INC-05 corregido
> El endpoint `/archived-cycles/{pk}/` solo acepta `GET`. Para desarchivar (unarchive) se debe usar `DELETE /cycles/{cycle_id}/archive/`. (Igual que INC-10 para módulos).

### Analytics y progreso

| Método | URL                                                             | Guard                     | Fase |
| ------ | --------------------------------------------------------------- | ------------------------- | ---- |
| `GET`  | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/progress/`  | `ProjectMemberGuard (≥5)` | 4    |
| `GET`  | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/analytics/` | `ProjectMemberGuard (≥5)` | 4    |

### Favoritos y preferencias

| Método      | URL                                                                   | Guard                     | Fase |
| ----------- | --------------------------------------------------------------------- | ------------------------- | ---- |
| `GET/POST`  | `/workspaces/{slug}/projects/{id}/user-favorite-cycles/`              | `ProjectMemberGuard (≥5)` | 4    |
| `DELETE`    | `/workspaces/{slug}/projects/{id}/user-favorite-cycles/{cycle_id}/`   | `ProjectMemberGuard (≥5)` | 4    |
| `GET/PATCH` | `/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/user-properties/` | `ProjectMemberGuard (≥5)` | 4    |

---

## Regla crítica — un issue, un ciclo activo

```rust
// Al agregar un issue a un ciclo (POST /cycle-issues/):
// 1. Verificar que el issue no esté ya en otro ciclo ACTIVO
// 2. Si está, removerlo del anterior antes de agregar al nuevo

pub async fn add_issue_to_cycle(
    db: &DatabaseConnection,
    cycle_id: Uuid,
    issue_id: Uuid,
    project_id: Uuid,
    workspace_id: Uuid,
    actor_id: Uuid,
) -> anyhow::Result<()> {
    // Buscar ciclo actual (si lo hay)
    let current_cycle_issue = cycle_issues::Entity::find()
        .inner_join(cycles::Entity)
        .filter(cycle_issues::Column::IssueId.eq(issue_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        // Solo ciclos activos (status: CURRENT o UPCOMING que ya empezó)
        .filter(cycle_issues::Column::DeletedAt.is_null())
        .one(db).await?;

    if let Some(existing) = current_cycle_issue {
        // Soft delete del cycle_issue anterior
        let mut active: cycle_issues::ActiveModel = existing.into();
        active.deleted_at = Set(Some(chrono::Utc::now().into()));
        active.update(db).await?;
    }

    // Agregar al nuevo ciclo
    cycle_issues::ActiveModel {
        id:           Set(Uuid::new_v4()),
        cycle_id:     Set(cycle_id),
        issue_id:     Set(issue_id),
        project_id:   Set(project_id),
        workspace_id: Set(workspace_id),
        created_by_id: Set(Some(actor_id)),
        ..Default::default()
    }.insert(db).await?;

    Ok(())
}
```

---

## Date check — `/cycles/date-check/`

Verifica que las fechas de un nuevo ciclo no se solapen con ciclos existentes:

```rust
pub async fn date_check(
    State(state): State<AppState>,
    ProjectMemberGuard { project, .. }: ProjectMemberGuard,
    Query(params): Query<DateCheckParams>,
) -> Result<Json<DateCheckResponse>, AppError> {
    let overlapping = cycles::Entity::find()
        .active()
        .filter(cycles::Column::ProjectId.eq(project.id))
        .filter(
            // Solapamiento: new_start < existing_end AND new_end > existing_start
            Condition::all()
                .add(cycles::Column::StartDate.lt(params.end_date))
                .add(cycles::Column::EndDate.gt(params.start_date))
        )
        .count(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(DateCheckResponse { is_overlap: overlapping > 0 }))
}
```

---

## Transfer issues — `/transfer-issues/`

Mueve todos los issues incompletos del ciclo origen al ciclo destino:

```rust
pub async fn transfer_issues(
    State(state): State<AppState>,
    ProjectMemberGuard { workspace, .. }: ProjectMemberGuard,
    Path((_slug, project_id, cycle_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<TransferIssuesRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Obtener todos los issues del ciclo actual que NO están en estado "completed"
    let incomplete_issues = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::CycleId.eq(cycle_id))
        .inner_join(issues::Entity)
        .inner_join(states::Entity)
        .filter(states::Column::Group.ne("completed"))
        .filter(states::Column::Group.ne("cancelled"))
        .all(&state.db).await.map_err(AppError::Database)?;

    // 2. Mover cada issue al ciclo destino
    for ci in incomplete_issues {
        add_issue_to_cycle(&state.db, payload.new_cycle_id, ci.issue_id,
                           project_id, workspace.id, /* actor */ Uuid::nil()).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "message": "success" })))
}

#[derive(Deserialize)]
pub struct TransferIssuesRequest {
    pub new_cycle_id: Uuid,
}
```

---

## Analytics de ciclo

El endpoint `GET /cycles/{id}/analytics/` devuelve métricas de burndown:

```rust
#[derive(Serialize, ToSchema)]
pub struct CycleAnalytics {
    pub total_issues:     u64,
    pub completed_issues: u64,
    pub cancelled_issues: u64,
    pub started_issues:   u64,
    pub unstarted_issues: u64,
    pub backlog_issues:   u64,
    // Series temporales para burndown chart
    pub completion_chart: Vec<BurndownPoint>,
    pub issue_activities: Vec<CycleActivity>,
}

#[derive(Serialize, ToSchema)]
pub struct BurndownPoint {
    pub date:      NaiveDate,
    pub completed: u64,
    pub pending:   u64,
}
```

---

## DTOs

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateCycleRequest {
    pub name:        String,
    pub description: Option<String>,
    pub start_date:  Option<NaiveDate>,
    pub end_date:    Option<NaiveDate>,
    pub status:      Option<String>,  // "DRAFT" si no hay fechas
}

#[derive(Serialize, ToSchema)]
pub struct CycleResponse {
    pub id:              Uuid,
    pub name:            String,
    pub description:     Option<String>,
    pub status:          String,         // calculado en runtime
    pub start_date:      Option<NaiveDate>,
    pub end_date:        Option<NaiveDate>,
    pub project_id:      Uuid,
    pub workspace_id:    Uuid,
    pub is_favorite:     bool,
    pub total_issues:    u64,
    pub completed_issues: u64,
    pub cancelled_issues: u64,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}
```

> [!NOTE] `status` se calcula en runtime
> No se almacena en DB. Se deriva de `start_date`, `end_date` vs la fecha actual:
>
> - Sin fechas → `"DRAFT"`
> - start_date > hoy → `"UPCOMING"`
> - end_date < hoy → `"COMPLETED"`
> - else → `"CURRENT"`

---

## Puntos críticos

1. **Un issue, un ciclo activo** — al agregar, remover del ciclo anterior si existe.
2. **`status` calculado** — no persiste en DB. Calcular al serializar.
3. **Soft delete de `cycle_issues`** — usar `deleted_at`, no DELETE físico.
4. **Archivar/Desarchivar** — `POST /cycles/{id}/archive/` setea `archived_at = NOW()`. `DELETE /cycles/{id}/archive/` limpia `archived_at`. El endpoint `/archived-cycles/{pk}/` solo sirve para `GET` — no hay `DELETE` en esa ruta. No afecta a los issues.
5. **Date check antes de crear** — el frontend llama a `date-check` antes de `POST /cycles/`. El backend debe validar también (no confiar solo en frontend).

---

## Entidades SeaORM involucradas ✅

| Entidad                    | Tabla                   |
| -------------------------- | ----------------------- |
| `cycles.rs`                | `cycles`                |
| `cycle_issues.rs`          | `cycle_issues`          |
| `cycle_user_properties.rs` | `cycle_user_properties` |

---

## Plan de implementación

```
Fase 2:
  [ ] src/routes/cycles.rs         — CRUD + CycleListEndpoint + CycleDetailEndpoint
  [ ] src/routes/cycle_issues.rs   — añadir/quitar issues de ciclo + transfer issues

Fase 3:
  [ ] src/jobs/scheduled.rs        — cron de cálculo de progreso de ciclos (completadas en doc)
  [ ] src/utils/cycle_progress.rs  — helper: completedIssues / totalIssues → porcentaje
```

## 🔗 Navegar

← [[dominio-issues]] | [[MOC]] | → [[dominio-modulos]]

**Relacionado:** Issues: [[dominio-issues]] | Módulos: [[dominio-modulos]] | Analytics: [[dominio-analytics]]

---

_`docs/api-rust/dominio-ciclos.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
