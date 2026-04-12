---
titulo: Dominio — Issue Views
aliases:
  - views
  - vistas
  - dominio-vistas
tags:
  - views
  - dominio
  - rust
  - axum
  - pendiente-implementar
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-workspace-settings]]"
estado: activo
---

# Dominio — Issue Views

> [!NOTE] INC-16 — Dominio de vistas
> Este dominio no estaba documentado previamente.

> [!NOTE] Vistas personalizadas de issues
> Las vistas permiten guardar configuraciones de filtros, agrupación y ordenamiento para issues, ya sea a nivel de proyecto o a nivel global del workspace.

---

## Modelo de datos

```
workspaces
    ├─ views (Workspace Views — Globales)
    └─ projects
            ├─ views (Project Views)
            └─ user_favorite_views (M2M → views)
```

Las vistas guardan un objeto `query` (JSON) que contiene los mismos parámetros de filtrado que el endpoint de issues.

---

## Endpoints a implementar

> [!INFO] Fuente
> `apps/api/plane/app/urls/views.py` — Django.

### Vistas de proyecto (Project Views)

| Método                 | URL                                                    | Guard                        | Fase |
| ---------------------- | ------------------------------------------------------ | ---------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{project_id}/views/`      | `ProjectMemberGuard (≥5/15)` | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/views/{pk}/` | `ProjectMemberGuard (≥5/15)` | 4    |

### Vistas globales (Workspace Views)

| Método                 | URL                              | Guard                          | Fase |
| ---------------------- | -------------------------------- | ------------------------------ | ---- |
| `GET/POST`             | `/workspaces/{slug}/views/`      | `WorkspaceMemberGuard (≥5/15)` | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/views/{pk}/` | `WorkspaceMemberGuard (≥5/15)` | 4    |

### Listado de issues de vista global

| Método | URL                          | Descripción                                                                                             | Fase |
| ------ | ---------------------------- | ------------------------------------------------------------------------------------------------------- | ---- |
| `GET`  | `/workspaces/{slug}/issues/` | `WorkspaceViewIssuesViewSet` — lista issues aplicando filtros de una vista global o parámetros directos | 4    |

### Favoritos de vistas

| Método     | URL                                                                       | Guard                     | Fase |
| ---------- | ------------------------------------------------------------------------- | ------------------------- | ---- |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/user-favorite-views/`           | `ProjectMemberGuard (≥5)` | 4    |
| `DELETE`   | `/workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}/` | `ProjectMemberGuard (≥5)` | 4    |

---

## Handler — `GET /workspaces/{slug}/issues/`

Este endpoint es crucial para el "All Issues" o vistas globales del workspace. Debe agregar issues de múltiples proyectos.

```rust
pub async fn list_workspace_view_issues(
    State(state): State<AppState>,
    WorkspaceMemberGuard { user, workspace, .. }: WorkspaceMemberGuard,
    Query(params): Query<IssueQueryParams>,
) -> Result<Json<Vec<IssueResponse>>, AppError> {
    let db = &state.db;

    // 1. Obtener IDs de proyectos donde el usuario es miembro
    let project_ids = project_members::Entity::find()
        .filter(project_members::Column::WorkspaceId.eq(workspace.id))
        .filter(project_members::Column::MemberId.eq(user.id))
        .filter(project_members::Column::IsActive.eq(true))
        .select_only()
        .column(project_members::Column::ProjectId)
        .into_tuple::<Uuid>()
        .all(db).await.map_err(AppError::Database)?;

    // 2. Ejecutar query de issues filtrada por esos proyectos
    let mut query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.is_in(project_ids));

    // ... aplicar filtros de params (mismo helper que dominio-issues)

    let issues = query.all(db).await.map_err(AppError::Database)?;

    Ok(Json(issues.into_iter().map(IssueResponse::from).collect()))
}
```

---

## DTOs

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateViewRequest {
    pub name:        String,             // ⚠️ FIX-33: validar máx 255 chars
    pub description: Option<String>,     // validar máx 1 000 chars
    pub query:       serde_json::Value,  // ⚠️ FIX-33: JSON arbitrario — validar tamaño máx 64 KB antes de persistir
    pub access:      Option<String>,     // "public" | "private" — validar contra allowlist
}

// ── Fix-33: Guards requeridos en handler ──────────────────────────────────
// const MAX_VIEW_NAME: usize  = 255;
// const MAX_VIEW_QUERY: usize = 64 * 1024; // 64 KB
//
// if payload.name.len() > MAX_VIEW_NAME {
//     return Err(AppError::bad_request("name exceeds 255 characters"));
// }
// let query_bytes = serde_json::to_vec(&payload.query)
//     .map_err(|_| AppError::bad_request("invalid query JSON"))?;
// if query_bytes.len() > MAX_VIEW_QUERY {
//     return Err(AppError::bad_request("query exceeds 64 KB"));
// }
// if !matches!(payload.access.as_deref(), None | Some("public") | Some("private")) {
//     return Err(AppError::bad_request("access must be 'public' or 'private'"));
// }
//
// Riesgo sin validación:
//   • query sin límite: atacante persiste JSON de 10 MB → lento en reads y
//     tabla de vistas se infla sin control.
// ──────────────────────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct ViewResponse {
    pub id:          Uuid,
    pub name:        String,
    pub description: Option<String>,
    pub query:       serde_json::Value,
    pub access:      String,
    pub project_id:  Option<Uuid>,      // null para vistas globales
    pub workspace_id: Uuid,
    pub created_by:  Uuid,
    pub is_favorite: bool,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}
```

---

## Entidades SeaORM involucradas ✅

| Entidad             | Tabla            | Notas                                                  |
| ------------------- | ---------------- | ------------------------------------------------------ |
| `issue_views.rs`    | `issue_views`    | Contiene el campo `query` (jsonb)                      |
| `user_favorites.rs` | `user_favorites` | Para marcar vistas como favoritas (entity_type='view') |

---

## Plan de implementación

```
Fase 3:
  [ ] src/routes/views.rs          — CRUD workspace-views + project-views
                                     POST/DELETE /favorite-views/{id}/
                                     GET /issues/ (issues filtrados por view.query)
```

## 🔗 Navegar

← [[dominio-issues]] | [[MOC]] | → [[dominio-workspace-settings]]
