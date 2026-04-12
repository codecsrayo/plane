---
titulo: Dominio — Búsqueda (Search)
aliases:
  - search
  - busqueda
  - dominio-busqueda
tags:
  - search
  - dominio
  - rust
  - axum
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-proyectos]]"
  - "[[impl-appstate-repository]]"
  - "[[plan-fases]]"
estado: activo
---

# Dominio — Búsqueda (Search)

> [!NOTE] Búsqueda ILIKE contra PostgreSQL — sin Elasticsearch
> Plane CE usa búsqueda textual simple con `ILIKE` sobre PostgreSQL. No requiere infraestructura adicional. Los resultados se ordenan por relevancia básica (coincidencia en nombre > descripción).

---

## Endpoints a implementar

| Método | URL                                               | Guard                       | Fase |
| ------ | ------------------------------------------------- | --------------------------- | ---- |
| `GET`  | `/workspaces/{slug}/search/`                      | `WorkspaceMemberGuard (≥5)` | 4    |
| `GET`  | `/workspaces/{slug}/projects/{id}/search-issues/` | `ProjectMemberGuard (≥5)`   | 4    |
| `GET`  | `/workspaces/{slug}/entity-search/`               | `WorkspaceMemberGuard (≥5)` | 4    |

---

## `GET /workspaces/{slug}/search/` — búsqueda global

Busca en múltiples entidades del workspace y devuelve resultados agrupados:

```rust
#[derive(Deserialize, ToSchema)]
pub struct GlobalSearchParams {
    pub query:      String,       // texto a buscar
    pub entity:     Option<String>, // filtrar por tipo: "issue"|"project"|"cycle"|"module"|"page"
    pub project_id: Option<Uuid>,   // acotar al proyecto
    pub workspace_slug: String,
}

#[derive(Serialize, ToSchema)]
pub struct GlobalSearchResponse {
    pub results: SearchResults,
    pub query:   String,
}

#[derive(Serialize, ToSchema)]
pub struct SearchResults {
    pub issues:   Vec<SearchResultItem>,
    pub projects: Vec<SearchResultItem>,
    pub cycles:   Vec<SearchResultItem>,
    pub modules:  Vec<SearchResultItem>,
    pub pages:    Vec<SearchResultItem>,
}

#[derive(Serialize, ToSchema)]
pub struct SearchResultItem {
    pub id:          Uuid,
    pub name:        String,
    pub entity_type: String,      // "issue"|"project"|"cycle"|"module"|"page"
    pub project_id:  Option<Uuid>,
    pub workspace_id: Uuid,
    // Campos opcionales por tipo:
    pub sequence_id: Option<i64>, // para issues: el #ID
    pub state_id:    Option<Uuid>,
    pub priority:    Option<String>,
}
```

---

## Implementación — búsqueda global

```rust
pub async fn global_search(
    State(state): State<AppState>,
    WorkspaceMemberGuard { user, workspace, .. }: WorkspaceMemberGuard,
    Query(params): Query<GlobalSearchParams>,
) -> Result<Json<GlobalSearchResponse>, AppError> {
    let db = &state.db;
    let q = format!("%{}%", params.query.to_lowercase());
    let limit: u64 = 5; // máximo 5 resultados por entidad

    // Obtener proyectos a los que tiene acceso el usuario
    let accessible_project_ids = get_accessible_project_ids(db, workspace.id, user.id).await
        .map_err(AppError::Database)?;

    // Ejecutar búsquedas en paralelo con tokio::join!
    let (issues_res, projects_res, cycles_res, modules_res, pages_res) = tokio::join!(
        search_issues(db, &q, &accessible_project_ids, params.project_id, limit),
        search_projects(db, &q, workspace.id, &accessible_project_ids, limit),
        search_cycles(db, &q, &accessible_project_ids, params.project_id, limit),
        search_modules(db, &q, &accessible_project_ids, params.project_id, limit),
        search_pages(db, &q, &accessible_project_ids, params.project_id, limit),
    );

    Ok(Json(GlobalSearchResponse {
        query: params.query,
        results: SearchResults {
            issues:   issues_res.unwrap_or_default(),
            projects: projects_res.unwrap_or_default(),
            cycles:   cycles_res.unwrap_or_default(),
            modules:  modules_res.unwrap_or_default(),
            pages:    pages_res.unwrap_or_default(),
        },
    }))
}
```

---

## Búsqueda de issues — implementación

```rust
async fn search_issues(
    db: &DatabaseConnection,
    query: &str,
    project_ids: &[Uuid],
    project_filter: Option<Uuid>,
    limit: u64,
) -> Result<Vec<SearchResultItem>, sea_orm::DbErr> {
    let mut q = issues::Entity::find()
        .active()
        .filter(
            // Buscar en name O en description_html
            Condition::any()
                .add(Expr::col(issues::Column::Name)
                    .ilike(query))
                .add(Expr::col(issues::Column::DescriptionHtml)
                    .ilike(query))
        )
        .filter(issues::Column::ProjectId.is_in(project_ids.to_vec()))
        .limit(limit);

    if let Some(pid) = project_filter {
        q = q.filter(issues::Column::ProjectId.eq(pid));
    }

    // Join con issue_sequences para obtener sequence_id
    let issues = q
        .find_also_related(issue_sequences::Entity)
        .all(db).await?;

    Ok(issues.into_iter().map(|(issue, seq)| SearchResultItem {
        id:           issue.id,
        name:         issue.name,
        entity_type:  "issue".into(),
        project_id:   Some(issue.project_id),
        workspace_id: issue.workspace_id,
        sequence_id:  seq.map(|s| s.sequence_id),
        state_id:     issue.state_id,
        priority:     issue.priority,
    }).collect())
}
```

---

## `GET /search-issues/` — búsqueda dentro de un proyecto

Más específico que `/search/`, solo busca issues en un proyecto y es usado por el frontend para:

- Autocomplete al asignar issues relacionados (`issue_relations`)
- Autocomplete al asignar padre (`parent_id`)
- Selector de issues al añadir a ciclo/módulo

```rust
#[derive(Deserialize, ToSchema)]
pub struct SearchIssuesParams {
    pub query:      String,
    pub priority:   Option<String>,
    pub state_id:   Option<Uuid>,
    pub exclude:    Option<Vec<Uuid>>,  // excluir IDs específicos (e.g. el propio issue)
    pub limit:      Option<u64>,
}

pub async fn search_project_issues(
    State(state): State<AppState>,
    ProjectMemberGuard { project, workspace, .. }: ProjectMemberGuard,
    Path((_slug, project_id)): Path<(String, Uuid)>,
    Query(params): Query<SearchIssuesParams>,
) -> Result<Json<Vec<SearchResultItem>>, AppError> {
    let q = format!("%{}%", params.query.to_lowercase());
    let limit = params.limit.unwrap_or(20).min(50); // máximo 50

    let mut query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(
            Condition::any()
                .add(Expr::col(issues::Column::Name).ilike(&q))
        )
        .limit(limit);

    if let Some(exclude_ids) = params.exclude {
        query = query.filter(issues::Column::Id.is_not_in(exclude_ids));
    }

    let results = query.find_also_related(issue_sequences::Entity)
        .all(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(results.into_iter().map(|(issue, seq)| SearchResultItem {
        id:           issue.id,
        name:         issue.name,
        entity_type:  "issue".into(),
        project_id:   Some(project_id),
        workspace_id: workspace.id,
        sequence_id:  seq.map(|s| s.sequence_id),
        state_id:     issue.state_id,
        priority:     issue.priority,
    }).collect()))
}
```

---

## `GET /entity-search/` — búsqueda multi-entidad filtrada

Versión más configurable de `/search/`. El cliente especifica qué entidades buscar:

```rust
#[derive(Deserialize, ToSchema)]
pub struct EntitySearchParams {
    pub query:    String,
    pub entities: Option<Vec<String>>,  // default: todos. Ej: ["issue", "page"]
    pub count:    Option<u64>,          // resultados por entidad, default: 5
}
```

---

## Estrategia de relevancia

Sin full-text search, aplicar un orden de relevancia básico:

```rust
// Prioridad 1: coincidencia exacta en nombre (case-insensitive)
// Prioridad 2: coincidencia en nombre con ILIKE (starts with)
// Prioridad 3: coincidencia en descripción

// Implementación: ORDER BY
//   CASE WHEN lower(name) = lower(query) THEN 1
//        WHEN lower(name) LIKE lower(query || '%') THEN 2
//        ELSE 3
//   END ASC,
//   updated_at DESC
```

Con SeaORM se puede usar `order_by_expr` con una expresión CASE:

```rust
use sea_orm::sea_query::Expr;

let lower_name = Expr::col(issues::Column::Name).cast_as(Alias::new("text")).binary(
    BinOper::Like, Expr::value(format!("{}%", query.to_lowercase()))
);
// Simplificación: ordenar solo por updated_at desc cuando ILIKE coincide
query.order_by_desc(issues::Column::UpdatedAt)
```

---

## Acceso a proyectos en búsqueda global

```rust
/// Devuelve los project_ids a los que el usuario tiene acceso en el workspace.
/// Incluye proyectos donde es miembro explícito + proyectos públicos del workspace.
async fn get_accessible_project_ids(
    db: &DatabaseConnection,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<Uuid>, sea_orm::DbErr> {
    // Proyectos donde es miembro
    let member_projects: Vec<Uuid> = project_members::Entity::find()
        .filter(project_members::Column::WorkspaceId.eq(workspace_id))
        .filter(project_members::Column::MemberId.eq(user_id))
        .filter(project_members::Column::DeletedAt.is_null())
        .select_only()
        .column(project_members::Column::ProjectId)
        .into_tuple()
        .all(db).await?;

    // Proyectos públicos del workspace (network=0)
    let public_projects: Vec<Uuid> = projects::Entity::find()
        .filter(projects::Column::WorkspaceId.eq(workspace_id))
        .filter(projects::Column::Network.eq(0))
        .filter(projects::Column::DeletedAt.is_null())
        .select_only()
        .column(projects::Column::Id)
        .into_tuple()
        .all(db).await?;

    // Unión sin duplicados
    let mut all: std::collections::HashSet<Uuid> =
        member_projects.into_iter().chain(public_projects).collect();
    Ok(all.into_iter().collect())
}
```

---

## Puntos críticos

> [!WARNING] 4 puntos críticos

1. **Acceso a proyectos** — la búsqueda global solo debe devolver resultados de proyectos a los que el usuario tiene acceso (miembro explícito o proyectos públicos). Nunca exponer issues de proyectos secretos donde no es miembro.
2. **ILIKE es case-insensitive pero lento** — sin índices de texto no escala bien. Para tablas grandes (>100k issues), considerar `pg_trgm` extension con índice GIN. Documentar como mejora futura.
3. **Límite estricto** — máximo 5–10 resultados por entidad en búsqueda global. El endpoint `/search-issues/` puede devolver hasta 50. Evitar queries sin límite.
4. **`query` mínimo 2 chars** — validar que el término de búsqueda tiene al menos 2 caracteres antes de ejecutar las queries. Evitar búsquedas de 1 char que devuelven casi todo.

---

## Mejora futura — `pg_trgm`

Para escalar la búsqueda sin Elasticsearch:

```sql
-- Activar extensión (en migración futura)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Índice GIN para búsqueda de texto en issues
CREATE INDEX CONCURRENTLY idx_issues_name_trgm
    ON issues USING GIN (name gin_trgm_ops)
    WHERE deleted_at IS NULL;

-- Query con operador % (similitud) en lugar de ILIKE
SELECT * FROM issues
WHERE name % 'search term'
ORDER BY similarity(name, 'search term') DESC
LIMIT 10;
```

Documentar como opción de Fase 4+ cuando el volumen de issues supere 50k por workspace.

---

## Entidades SeaORM involucradas ✅

| Entidad              | Tabla             | Campos buscados                       |
| -------------------- | ----------------- | ------------------------------------- |
| `issues.rs`          | `issues`          | `name`, `description_html`            |
| `projects.rs`        | `projects`        | `name`, `identifier`                  |
| `cycles.rs`          | `cycles`          | `name`                                |
| `modules.rs`         | `modules`         | `name`                                |
| `pages.rs`           | `pages`           | `name`, `description_html`            |
| `issue_sequences.rs` | `issue_sequences` | `sequence_id` (join para obtener #ID) |

---

## 🔗 Navegar

← [[dominio-importadores]] | [[MOC]] | → [[dominio-issues]]

**Relacionado:** Issues: [[dominio-issues]] | Proyectos: [[dominio-proyectos]] | Repository Pattern: [[impl-appstate-repository]]
