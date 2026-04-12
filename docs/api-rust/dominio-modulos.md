---
titulo: Dominio — Modules
aliases:
  - modules
  - modulos
  - dominio-modulos
tags:
  - modules
  - dominio
  - rust
  - axum
  - pendiente-implementar
  - todo-rs
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-ciclos]]"
  - "[[impl-extractores-auth]]"
  - "[[plan-fases]]"
estado: activo
---

# Dominio — Modules

> [!NOTE] Agrupación temática de issues
> Un módulo agrupa issues por feature, épica o área. A diferencia de los ciclos, un issue puede estar en **múltiples módulos** simultáneamente.

---

## Modelo de datos

```tree
projects
    └─ modules
            ├─ module_issues          (M2M → issues, un issue puede estar en varios módulos)
            ├─ module_links           (1→N, URLs de referencia del módulo)
            ├─ module_members         (M2M → users, responsables del módulo)
            └─ module_user_properties (preferencias de vista por usuario)
```

**Diferencia clave vs Cycles:** `module_issues` no tiene la restricción "un módulo activo por issue". Un issue puede pertenecer a N módulos.

---

## Endpoints a implementar

### CRUD de módulo

| Método   | URL                                              | Guard                      | Fase |
| -------- | ------------------------------------------------ | -------------------------- | ---- |
| `GET`    | `/workspaces/{slug}/projects/{id}/modules/`      | `ProjectMemberGuard (≥5)`  | 2    |
| `POST`   | `/workspaces/{slug}/projects/{id}/modules/`      | `ProjectMemberGuard (≥15)` | 2    |
| `GET`    | `/workspaces/{slug}/projects/{id}/modules/{pk}/` | `ProjectMemberGuard (≥5)`  | 2    |
| `PATCH`  | `/workspaces/{slug}/projects/{id}/modules/{pk}/` | `ProjectMemberGuard (≥15)` | 4    |
| `DELETE` | `/workspaces/{slug}/projects/{id}/modules/{pk}/` | `ProjectMemberGuard (≥15)` | 4    |

### Issues del módulo

| Método                 | URL                                                                       | Guard                        | Fase |
| ---------------------- | ------------------------------------------------------------------------- | ---------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{id}/modules/{module_id}/issues/`            | `ProjectMemberGuard (≥5/15)` | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{id}/modules/{module_id}/issues/{issue_id}/` | `ProjectMemberGuard (≥15)`   | 4    |
| `POST`                 | `/workspaces/{slug}/projects/{id}/issues/{issue_id}/modules/`             | `ProjectMemberGuard (≥15)`   | 4    |

> [!WARNING] INC-04 corregido
> El endpoint `/issues/{issue_id}/modules/` solo acepta `POST` en Django. El `DELETE` de un issue de un módulo se hace por la ruta inversa `DELETE /modules/{module_id}/issues/{issue_id}/`.

### Links del módulo

| Método                 | URL                                                                       | Guard                        | Fase |
| ---------------------- | ------------------------------------------------------------------------- | ---------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{id}/modules/{module_id}/module-links/`      | `ProjectMemberGuard (≥5/15)` | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{id}/modules/{module_id}/module-links/{pk}/` | `ProjectMemberGuard (≥15)`   | 4    |

### Archivo y listado archivados

| Método        | URL                                                             | Guard                      | Fase |
| ------------- | --------------------------------------------------------------- | -------------------------- | ---- |
| `POST/DELETE` | `/workspaces/{slug}/projects/{id}/modules/{module_id}/archive/` | `ProjectMemberGuard (≥15)` | 4    |
| `GET`         | `/workspaces/{slug}/projects/{id}/archived-modules/`            | `ProjectMemberGuard (≥5)`  | 4    |
| `GET`         | `/workspaces/{slug}/projects/{id}/archived-modules/{pk}/`       | `ProjectMemberGuard (≥5)`  | 4    |

> [!WARNING] INC-10 corregido
> El endpoint `/archived-modules/{pk}/` solo acepta `GET`. Para desarchivar (unarchive) se debe usar `DELETE /modules/{module_id}/archive/`.

### Favoritos y preferencias de usuario

| Método      | URL                                                                     | Guard                     | Fase |
| ----------- | ----------------------------------------------------------------------- | ------------------------- | ---- |
| `GET/POST`  | `/workspaces/{slug}/projects/{id}/user-favorite-modules/`               | `ProjectMemberGuard (≥5)` | 4    |
| `DELETE`    | `/workspaces/{slug}/projects/{id}/user-favorite-modules/{module_id}/`   | `ProjectMemberGuard (≥5)` | 4    |
| `GET/PATCH` | `/workspaces/{slug}/projects/{id}/modules/{module_id}/user-properties/` | `ProjectMemberGuard (≥5)` | 4    |

---

## Handler — `POST /modules/{id}/issues/` (bulk add)

El frontend envía una lista de `issue_ids` para agregar al módulo en un solo request:

```rust
pub async fn add_issues_to_module(
    State(state): State<AppState>,
    ProjectMemberGuard { user, project, workspace, .. }: ProjectMemberGuard,
    Path((_slug, project_id, module_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<AddModuleIssuesRequest>,
) -> Result<Json<Vec<ModuleIssueResponse>>, AppError> {
    let db = &state.db;
    let mut created = vec![];

    for issue_id in payload.issues {
        // Verificar que no exista ya (evitar duplicados)
        let exists = module_issues::Entity::find()
            .filter(module_issues::Column::ModuleId.eq(module_id))
            .filter(module_issues::Column::IssueId.eq(issue_id))
            .filter(module_issues::Column::DeletedAt.is_null())
            .count(db).await.map_err(AppError::Database)? > 0;

        if exists { continue; }

        let mi = module_issues::ActiveModel {
            id:           Set(Uuid::new_v4()),
            module_id:    Set(module_id),
            issue_id:     Set(issue_id),
            project_id:   Set(project_id),
            workspace_id: Set(workspace.id),
            created_by_id: Set(Some(user.id)),
            ..Default::default()
        }.insert(db).await.map_err(AppError::Database)?;

        created.push(ModuleIssueResponse::from(mi));
    }

    Ok(Json(created))
}

#[derive(Deserialize, ToSchema)]
pub struct AddModuleIssuesRequest {
    pub issues: Vec<Uuid>,
}
```

---

## Handler — issue-centric module assignment

El endpoint `/issues/{issue_id}/modules/` solo acepta `POST` (Django: `create_issue_modules`).
El `DELETE` de un issue de un módulo se hace por la ruta inversa `DELETE /modules/{module_id}/issues/{issue_id}/`.

> [!WARNING] INC-04 corregido — Django solo mapea `POST` en esta ruta, no `GET` ni `DELETE`.

```rust
// POST /issues/{issue_id}/modules/ — asignar issue a uno o varios módulos
pub async fn assign_issue_modules(
    State(state): State<AppState>,
    ProjectMemberGuard { user, workspace, .. }: ProjectMemberGuard,
    Path((_slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(payload): Json<IssueModulesRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = &state.db;

    for module_id in &payload.modules {
        // Idempotente: solo insertar si no existe
        let exists = module_issues::Entity::find()
            .filter(module_issues::Column::ModuleId.eq(*module_id))
            .filter(module_issues::Column::IssueId.eq(issue_id))
            .filter(module_issues::Column::DeletedAt.is_null())
            .count(db).await.map_err(AppError::Database)? > 0;

        if !exists {
            module_issues::ActiveModel {
                id:           Set(Uuid::new_v4()),
                module_id:    Set(*module_id),
                issue_id:     Set(issue_id),
                project_id:   Set(project_id),
                workspace_id: Set(workspace.id),
                created_by_id: Set(Some(user.id)),
                ..Default::default()
            }.insert(db).await.map_err(AppError::Database)?;
        }
    }

    Ok(Json(serde_json::json!({ "message": "success" })))
}
```

---

## Module status — calculado en runtime

Al igual que los ciclos, el `status` se calcula comparando fechas con hoy:

```rust
pub fn compute_module_status(
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
) -> &'static str {
    let today = chrono::Local::now().date_naive();
    match (start_date, end_date) {
        (None, None)                              => "backlog",
        (Some(s), _) if s > today               => "backlog",
        (_, Some(e)) if e < today               => "completed",
        (Some(s), Some(e)) if s <= today && today <= e => "in_progress",
        _                                         => "backlog",
    }
}
```

**Valores posibles de status:** `"backlog"`, `"in_progress"`, `"paused"`, `"completed"`, `"cancelled"`.

---

## Module Links

Links son URLs de referencia del módulo (documentación, PRs, etc.):

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateModuleLinkRequest {
    pub title: Option<String>,
    pub url:   String,
}

#[derive(Serialize, ToSchema)]
pub struct ModuleLinkResponse {
    pub id:         Uuid,
    pub title:      Option<String>,
    pub url:        String,
    pub module_id:  Uuid,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
```

---

## DTOs

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateModuleRequest {
    pub name:         String,
    pub description:  Option<String>,
    pub status:       Option<String>,
    pub start_date:   Option<NaiveDate>,
    pub target_date:  Option<NaiveDate>,   // end_date en la DB
    pub lead_id:      Option<Uuid>,        // responsable principal
    pub member_ids:   Option<Vec<Uuid>>,   // miembros del módulo
}

#[derive(Serialize, ToSchema)]
pub struct ModuleResponse {
    pub id:               Uuid,
    pub name:             String,
    pub description:      Option<String>,
    pub status:           String,          // calculado en runtime
    pub start_date:       Option<NaiveDate>,
    pub target_date:      Option<NaiveDate>,
    pub lead_id:          Option<Uuid>,
    pub member_ids:       Vec<Uuid>,
    pub project_id:       Uuid,
    pub workspace_id:     Uuid,
    pub is_favorite:      bool,
    pub total_issues:     u64,
    pub completed_issues: u64,
    pub cancelled_issues: u64,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}
```

---

## Puntos críticos

> [!WARNING] 4 puntos críticos

1. **Múltiples módulos por issue** — no hay restricción de unicidad. Un issue puede estar en 0, 1 o N módulos al mismo tiempo.
2. **Soft delete de `module_issues`** — igual que `cycle_issues`, usar `deleted_at`. No DELETE físico.
3. **`target_date` vs `end_date`** — en el frontend se llama `target_date`, en la DB puede ser `end_date`. Mapear en el DTO.
4. **Bulk add idempotente** — si el issue ya está en el módulo (soft-delete IS NULL), ignorar silenciosamente, no retornar error.

---

## Entidades SeaORM involucradas ✅

| Entidad                     | Tabla                    |
| --------------------------- | ------------------------ |
| `modules.rs`                | `modules`                |
| `module_issues.rs`          | `module_issues`          |
| `module_links.rs`           | `module_links`           |
| `module_members.rs`         | `module_members`         |
| `module_user_properties.rs` | `module_user_properties` |

---

## Plan de implementación

```
Fase 3:
  [ ] src/routes/modules.rs        — CRUD + ModuleListEndpoint + ModuleDetailEndpoint
  [ ] src/routes/module_issues.rs  — añadir/quitar issues de módulo
  [ ] src/routes/module_links.rs   — CRUD links de módulo
```

## 🔗 Navegar

← [[dominio-ciclos]] | [[MOC]] | → [[dominio-paginas]]

**Relacionado:** Issues: [[dominio-issues]] | Cycles: [[dominio-ciclos]] | Seed: [[dominio-workspace-seed]]
