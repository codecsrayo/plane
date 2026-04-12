---
titulo: Dominio — Analytics
aliases:
  - analytics
  - metricas
  - dominio-analytics
tags:
  - analytics
  - dominio
  - rust
  - axum
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-ciclos]]"
  - "[[impl-appstate-repository]]"
  - "[[plan-fases]]"
estado: activo
---

# Dominio — Analytics

> [!NOTE] Reportes y métricas sobre issues, ciclos y actividad del workspace
> Analytics no modifica datos — solo lee. Candidato a optimizar con queries raw SQL cuando las consultas agregadas de SeaORM sean insuficientes.

---

## Endpoints a implementar

### Analytics de workspace

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/workspaces/{slug}/analytics/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/default-analytics/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/export-analytics/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/project-stats/` | `WorkspaceMemberGuard (≥5)` | 4 |

### Analytic views (guardadas)

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/workspaces/{slug}/analytic-view/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/analytic-view/{pk}/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `POST` | `/workspaces/{slug}/saved-analytic-view/{analytic_id}/` | `WorkspaceMemberGuard (≥5)` | 4 |

### Advance analytics

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/workspaces/{slug}/advance-analytics/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/advance-analytics-stats/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/advance-analytics-charts/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/projects/{id}/advance-analytics/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/projects/{id}/advance-analytics-stats/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/projects/{id}/advance-analytics-charts/` | `ProjectMemberGuard (≥5)` | 4 |

### Stats de usuario en workspace

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/workspaces/{slug}/user-stats/{user_id}/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/user-activity/{user_id}/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/user-activity/{user_id}/export/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/user-profile/{user_id}/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/user-issues/{user_id}/` | `WorkspaceMemberGuard (≥5)` | 4 |

> [!NOTE] INC-08 — En Django estos endpoints están en `urls/workspace.py` (vistas `WorkspaceUserProfileStatsEndpoint`, `WorkspaceUserActivityEndpoint`, etc.), no en el módulo analytics. En Rust deben vivir en el router de **workspace**, no en analytics.

---

## `GET /analytics/` — analytics principal

Devuelve issues agrupados por la dimensión solicitada (`x_axis`) y contados/sumados por otra (`y_axis`):

```rust
#[derive(Deserialize, ToSchema)]
pub struct AnalyticsQueryParams {
    // Dimensiones de agrupación del eje X
    pub x_axis: String,   // "state_id"|"priority"|"assignees"|"label_ids"|
                          // "estimate_point"|"created_at"|"start_date"|"due_date"|
                          // "cycle_id"|"module_ids"
    // Métricas del eje Y
    pub y_axis: String,   // "issue_count" | "estimate"

    // Filtros opcionales (mismos que GET /issues/)
    pub state_ids:     Option<String>,    // CSV de UUIDs
    pub priority:      Option<String>,    // CSV
    pub assignee_ids:  Option<String>,
    pub label_ids:     Option<String>,
    pub project:       Option<String>,    // CSV de UUIDs (workspace-level)
    pub segment:       Option<String>,    // para donut/pie charts secundarios
}

#[derive(Serialize, ToSchema)]
pub struct AnalyticsResponse {
    pub total_issues:  u64,
    pub total_estimate: Option<f64>,
    pub distribution:  Vec<AnalyticsGroup>,
    pub extras:        Option<SegmentData>,  // si segment != null
}

#[derive(Serialize, ToSchema)]
pub struct AnalyticsGroup {
    pub dimension:    serde_json::Value,  // valor del x_axis (puede ser UUID, string, null)
    pub count:        u64,
    pub estimate:     Option<f64>,
    pub label:        Option<String>,     // nombre legible (state name, user name, etc.)
    pub color:        Option<String>,     // para estado o label
}
```

---

## Query SQL de analytics — ejemplo con SeaORM

Analytics con `x_axis=priority` y `y_axis=issue_count`:

```rust
pub async fn analytics_by_priority(
    db: &DatabaseConnection,
    project_ids: &[Uuid],
    filters: &AnalyticsQueryParams,
) -> anyhow::Result<Vec<AnalyticsGroup>> {
    // SeaORM custom query para GROUP BY dinámico
    let rows: Vec<(Option<String>, i64)> = issues::Entity::find()
        .select_only()
        .column(issues::Column::Priority)
        .column_as(issues::Column::Id.count(), "count")
        .active()
        .filter(issues::Column::ProjectId.is_in(project_ids.to_vec()))
        .group_by(issues::Column::Priority)
        .into_tuple()
        .all(db).await?;

    Ok(rows.into_iter().map(|(priority, count)| AnalyticsGroup {
        dimension: serde_json::Value::String(
            priority.clone().unwrap_or("none".into())
        ),
        count: count as u64,
        estimate: None,
        label: Some(priority.unwrap_or("none".into())),
        color: priority_color(&label),
    }).collect())
}

fn priority_color(priority: &str) -> Option<String> {
    match priority {
        "urgent" => Some("#EF4444".into()),
        "high"   => Some("#F97316".into()),
        "medium" => Some("#EAB308".into()),
        "low"    => Some("#22C55E".into()),
        _        => Some("#A1A1AA".into()),
    }
}
```

---

## `GET /default-analytics/` — métricas del dashboard

Devuelve un conjunto predefinido de métricas para el workspace dashboard:

```rust
#[derive(Serialize, ToSchema)]
pub struct DefaultAnalytics {
    pub total_issues:          u64,
    pub completed_issues:      u64,
    pub pending_issues:        u64,
    pub issues_by_priority:    Vec<PriorityCount>,
    pub issues_by_state_group: Vec<StateGroupCount>,
    pub open_estimate_sum:     Option<f64>,
    pub completed_estimate_sum: Option<f64>,
    pub last_week_issues:      u64,  // creados en los últimos 7 días
    pub this_week_issues:      u64,  // pendientes para esta semana
    pub overdue_issues:        u64,  // due_date < hoy AND no completados
}
```

---

## `GET /user-stats/{user_id}/` — estadísticas de usuario

```rust
#[derive(Serialize, ToSchema)]
pub struct UserStats {
    pub assigned_issues:   u64,
    pub completed_issues:  u64,
    pub pending_issues:    u64,
    pub created_issues:    u64,
    pub overdue_issues:    u64,
    pub subscribed_issues: u64,
    // Gráficas de actividad (últimos 30 días)
    pub completion_graph:  Vec<DailyCompletionPoint>,
    pub issue_graph:       Vec<DailyIssuePoint>,
}
```

---

## Analytic Views — guardar configuraciones

Los usuarios pueden guardar configuraciones de analytics para reutilizar:

```rust
// src/entities/analytic_views.rs — ya generado ✅
// Campos clave: query (JSON con todos los params), name, workspace_id, created_by_id

#[derive(Deserialize, ToSchema)]
pub struct CreateAnalyticViewRequest {
    pub name:  String,
    pub query: AnalyticsQueryParams,  // serializado como JSON en DB
}
```

---

## Advance Analytics — métricas avanzadas

El conjunto "advance" incluye métricas de equipo más elaboradas. Requiere queries más complejas:

```rust
#[derive(Serialize, ToSchema)]
pub struct AdvanceAnalytics {
    // Issues creados vs completados en un período
    pub created_vs_completed:  Vec<TimeSeriesPoint>,
    // Distribución de issues por miembro
    pub by_assignee:           Vec<AssigneeStats>,
    // Tiempo promedio de resolución por prioridad
    pub avg_resolution_time:   Vec<ResolutionTime>,
    // Tasa de completado de ciclos
    pub cycle_completion_rate: Vec<CycleCompletionRate>,
}

#[derive(Serialize, ToSchema)]
pub struct TimeSeriesPoint {
    pub date:      NaiveDate,
    pub created:   u64,
    pub completed: u64,
}
```

---

## Export analytics — CSV

`GET /export-analytics/` devuelve un CSV de los datos del analytics actual:

```rust
pub async fn export_analytics(
    State(state): State<AppState>,
    WorkspaceMemberGuard { workspace, .. }: WorkspaceMemberGuard,
    Query(params): Query<AnalyticsQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let data = compute_analytics(&state.db, workspace.id, &params).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut csv = String::from("Dimension,Count,Estimate\n");
    for row in &data.distribution {
        csv.push_str(&format!("{},{},{}\n",
            row.label.as_deref().unwrap_or(""),
            row.count,
            row.estimate.map(|e| e.to_string()).unwrap_or_default()
        ));
    }

    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/csv"),
         (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=\"analytics.csv\"")],
        csv,
    ))
}
```

---

## Estrategia de implementación

> [!TIP] Analytics puede mantenerse en Django más tiempo
> Analytics no es crítico para el flujo de trabajo diario. Ver [[plan-fases#Fase 4]]. Puede ser de las últimas rutas en migrar — Django sigue sirviendo analytics mientras Rust cubre issues/workspaces/projects.

**Orden de prioridad interno (Fase 4):**
1. `GET /default-analytics/` — usado en el dashboard principal
2. `GET /user-stats/{user_id}/` — usado en el perfil de usuario
3. `GET /analytics/` — analytics configurable
4. Advance analytics (más complejo, menos crítico)

---

## Puntos críticos

> [!WARNING] 4 puntos críticos

1. **Queries dinámicas** — `x_axis` determina el `GROUP BY`. Implementar con `match` exhaustivo para cada dimensión posible, no inyección de strings en SQL.
2. **`segment` doble agrupación** — si `segment` está presente, el resultado es una matriz 2D (group × segment). La respuesta cambia de `Vec<Group>` a `HashMap<String, Vec<Group>>`.
3. **Permisos de proyecto en workspace analytics** — filtrar solo issues de proyectos a los que el usuario tiene acceso (es miembro).
4. **Export CSV no usa S3** — el CSV se genera en memoria y se devuelve como streaming response. No pasar por S3/MinIO para exports pequeños.

---

## Entidades SeaORM involucradas ✅

| Entidad | Tabla |
|---------|-------|
| `analytic_views.rs` | `analytic_views` |
| `issues.rs` | `issues` (queries agregadas) |
| `issue_activities.rs` | `issue_activities` (user activity) |
| `cycles.rs` | `cycles` (cycle completion rate) |

---

## 🔗 Navegar

← [[dominio-intake]] | [[MOC]] | → [[dominio-importadores]]

**Relacionado:** Issues: [[dominio-issues]] | Cycles: [[dominio-ciclos]] | Plan de fases: [[plan-fases]]
