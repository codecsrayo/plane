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
    pub name:        String,
    pub description: Option<String>,
    pub query:       serde_json::Value, // Objeto IssueQueryParams
    pub access:      Option<String>,    // "public" | "private"
}

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

## 🔗 Navegar

← [[dominio-issues]] | [[MOC]] | → [[dominio-workspace-settings]]
