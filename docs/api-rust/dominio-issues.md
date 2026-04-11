---
titulo: Dominio — Issues (Work Items)
aliases:
  - issues
  - work-items
  - dominio-issues
tags:
  - issues
  - dominio
  - rust
  - axum
  - seaorm
relacionado:
  - "[[MOC]]"
  - "[[dominio-proyectos]]"
  - "[[dominio-ciclos]]"
  - "[[dominio-modulos]]"
  - "[[impl-extractores-auth]]"
  - "[[impl-appstate-repository]]"
  - "[[fundamentos-soft-delete]]"
  - "[[plan-fases]]"
estado: activo
---

# Dominio — Issues (Work Items)

> [!NOTE] Entidad central del sistema
> `issues` es la tabla con mayor volumen de tráfico. Prioridad máxima en Fase 2.

---

## Modelo de datos — relaciones

```
issues (tabla principal)
    ├─ issue_assignees         (M2M → users)
    ├─ issue_labels            (M2M → labels)
    ├─ issue_subscribers       (M2M → users)
    ├─ issue_reactions         (M2M → users, emoji)
    ├─ issue_attachments       (1→N, almacenamiento S3)
    ├─ issue_links             (1→N, URLs externas)
    ├─ issue_comments          (1→N)
    │       └─ comment_reactions (M2M → users)
    ├─ issue_activities        (1→N, historial de cambios)
    ├─ issue_relations         (M2M reflexiva: blocks/blocked_by/duplicate/relates_to)
    ├─ issue_sequences         (1→1, genera el #ID legible)
    ├─ issue_mentions          (M2M → users, menciones en descripción)
    ├─ issue_votes             (M2M → users)
    ├─ cycle_issues            (M2M → cycles)
    └─ module_issues           (M2M → modules)
```

---

## Endpoints a implementar

### CRUD principal

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/{id}/issues/` | `ProjectMemberGuard (≥5)` | 2 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/issues/` | `ProjectMemberGuard (≥15)` | 2 |
| `GET` | `/api/workspaces/{slug}/projects/{id}/issues/{pk}/` | `ProjectMemberGuard (≥5)` | 2 |
| `PATCH` | `/api/workspaces/{slug}/projects/{id}/issues/{pk}/` | `ProjectMemberGuard (≥15)` | 2 |
| `DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{pk}/` | `ProjectMemberGuard (≥15)` | 2 |
| `GET` | `/api/workspaces/{slug}/projects/{id}/issues/list/` | `ProjectMemberGuard (≥5)` | 2 |
| `GET` | `/api/workspaces/{slug}/projects/{id}/v2/issues/` | `ProjectMemberGuard (≥5)` | 4 |

### Sub-issues

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/sub-issues/` | `ProjectMemberGuard (≥5)` | 4 |

### Labels

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/issue-labels/` | `ProjectMemberGuard (≥5)` | 2 |
| `GET/PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/issue-labels/{pk}/` | `ProjectMemberGuard (≥15)` | 2 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/bulk-create-labels/` | `ProjectMemberGuard (≥15)` | 4 |

### Operaciones bulk

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `POST` | `/api/workspaces/{slug}/projects/{id}/bulk-delete-issues/` | `ProjectMemberGuard (≥15)` | 4 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/bulk-archive-issues/` | `ProjectMemberGuard (≥15)` | 4 |

### Comentarios

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/comments/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET/PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/comments/{pk}/` | `ProjectMemberGuard (≥5)` | 4 |

### Attachments

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/issue-attachments/` | `ProjectMemberGuard (≥15)` | 4 |
| `DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/issue-attachments/{pk}/` | `ProjectMemberGuard (≥15)` | 4 |
| `POST` | `/assets/v2/workspaces/{slug}/projects/{id}/issues/{issue_id}/attachments/` | `ProjectMemberGuard (≥15)` | 4 |

### Links

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/issue-links/` | `ProjectMemberGuard (≥5)` | 4 |
| `PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/issue-links/{pk}/` | `ProjectMemberGuard (≥15)` | 4 |

### Historial

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/history/` | `ProjectMemberGuard (≥5)` | 4 |

### Suscriptores

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/issue-subscribers/` | `ProjectMemberGuard (≥5)` | 4 |
| `DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/issue-subscribers/{subscriber_id}/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET/POST/DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/subscribe/` | `ProjectMemberGuard (≥5)` | 4 |

### Reacciones

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/reactions/` | `ProjectMemberGuard (≥5)` | 4 |
| `DELETE` | `/api/workspaces/{slug}/projects/{id}/issues/{issue_id}/reactions/{reaction_code}/` | `ProjectMemberGuard (≥5)` | 4 |

### Workspace-level issues

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/issues/` | `WorkspaceMemberGuard (≥5)` | 4 |

---

## Modelo de filtros — `GET /issues/`

Django soporta filtrado extenso por query params. Replicar en Rust:

```rust
#[derive(Deserialize, ToSchema)]
pub struct IssueQueryParams {
    // Paginación
    pub cursor:      Option<String>,   // cursor-based pagination
    pub per_page:    Option<u32>,      // default 100

    // Filtros básicos
    pub state_id:         Option<Vec<Uuid>>,      // ?state_id=uuid1,uuid2
    pub assignee_id:      Option<Vec<Uuid>>,
    pub label_id:         Option<Vec<Uuid>>,
    pub cycle_id:         Option<Uuid>,
    pub module_id:        Option<Uuid>,
    pub priority:         Option<Vec<String>>,    // "urgent","high","medium","low","none"
    pub mention:          Option<Uuid>,            // filtrar por menciones del user
    pub subscriber:       Option<Uuid>,
    pub created_by:       Option<Vec<Uuid>>,

    // Filtros de fecha
    pub created_at__gt:   Option<DateTime<Utc>>,
    pub updated_at__gt:   Option<DateTime<Utc>>,
    pub due_date__gt:     Option<NaiveDate>,
    pub due_date__lt:     Option<NaiveDate>,

    // Ordenamiento
    pub order_by:   Option<String>,  // "created_at","-created_at","priority","sort_order"

    // Estado especial
    pub state_group: Option<Vec<String>>, // "backlog","unstarted","started","completed","cancelled"
    pub archived:    Option<bool>,
    pub draft:       Option<bool>,
    pub sub_issue:   Option<bool>,    // incluir/excluir sub-issues
}
```

---

## Handler — `POST /issues/`

```rust
// src/routes/issues.rs
pub async fn create_issue(
    State(state): State<AppState>,
    ProjectMemberGuard { user, project_member, project, workspace, .. }: ProjectMemberGuard,
    Path((_slug, project_id)): Path<(String, Uuid)>,
    Json(payload): Json<CreateIssueRequest>,
) -> Result<Json<IssueResponse>, AppError> {
    let db = &state.db;

    // 1. Determinar sort_order (max + 10000 o 65535 si no hay issues)
    let max_sort = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .select_only()
        .column_as(issues::Column::SortOrder.max(), "max_sort")
        .into_tuple::<Option<f64>>()
        .one(db).await?
        .flatten()
        .unwrap_or(0.0);
    let sort_order = max_sort + 10000.0;

    // 2. Crear issue
    let issue_id = Uuid::new_v4();
    let issue = issues::ActiveModel {
        id:           Set(issue_id),
        name:         Set(payload.name),
        description:  Set(payload.description.unwrap_or_default()),
        description_html: Set(payload.description_html),
        priority:     Set(payload.priority.unwrap_or("none".into())),
        state_id:     Set(payload.state_id),
        parent_id:    Set(payload.parent_id),
        project_id:   Set(project_id),
        workspace_id: Set(workspace.id),
        sort_order:   Set(sort_order),
        created_by_id: Set(Some(user.id)),
        ..Default::default()
    }.insert(db).await.map_err(AppError::Database)?;

    // 3. IssueSequence — genera el #ID legible (e.g. WS-42)
    let sequence = issue_sequences::ActiveModel {
        id:         Set(Uuid::new_v4()),
        issue_id:   Set(issue_id),
        project_id: Set(project_id),
        workspace_id: Set(workspace.id),
        sequence_id: Set(payload.sequence_id.unwrap_or(0)), // auto si no viene
        ..Default::default()
    }.insert(db).await.map_err(AppError::Database)?;

    // 4. Asignar labels si vienen en el payload
    for label_id in payload.label_ids.unwrap_or_default() {
        issue_labels::ActiveModel {
            id: Set(Uuid::new_v4()),
            issue_id: Set(issue_id),
            label_id: Set(label_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace.id),
            ..Default::default()
        }.insert(db).await.map_err(AppError::Database)?;
    }

    // 5. Asignar assignees
    for assignee_id in payload.assignee_ids.unwrap_or_default() {
        issue_assignees::ActiveModel {
            id: Set(Uuid::new_v4()),
            issue_id: Set(issue_id),
            assignee_id: Set(assignee_id),
            project_id: Set(project_id),
            workspace_id: Set(workspace.id),
            ..Default::default()
        }.insert(db).await.map_err(AppError::Database)?;
    }

    // 6. IssueActivity — registrar creación
    // Ver sección "IssueActivity" más abajo

    // 7. WebhookDeliveryJob (best-effort, Fase 3)
    // state.job_storage.push(WebhookDeliveryJob { ... }).await.ok();

    Ok(Json(IssueResponse::from_model(issue, sequence)))
}
```

---

## IssueActivity — registro de cambios

Cada `PATCH` debe registrar en `issue_activities` los campos que cambiaron:

```rust
// src/utils/issue_activity.rs
pub struct FieldChange {
    pub field:     String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub verb:      String,  // "created", "updated", "deleted", "added", "removed"
}

pub async fn record_activity(
    db: &DatabaseConnection,
    issue_id: Uuid,
    actor_id: Uuid,
    project_id: Uuid,
    workspace_id: Uuid,
    changes: Vec<FieldChange>,
) -> anyhow::Result<()> {
    for change in changes {
        issue_activities::ActiveModel {
            id:           Set(Uuid::new_v4()),
            issue_id:     Set(issue_id),
            actor_id:     Set(actor_id),
            project_id:   Set(project_id),
            workspace_id: Set(workspace_id),
            field:        Set(Some(change.field)),
            old_value:    Set(change.old_value),
            new_value:    Set(change.new_value),
            verb:         Set(change.verb),
            ..Default::default()
        }.insert(db).await?;
    }
    Ok(())
}
```

**Campos que generan actividad:** `state_id`, `priority`, `assignees`, `labels`, `name`, `description`, `due_date`, `start_date`, `estimate_point`, `parent_id`, `cycle`, `module`, `link`, `attachment`.

---

## IssueRelations — tipos de relación

```rust
// Enum para el tipo de relación entre issues
pub enum IssueRelationType {
    Blocks,      // issue A bloquea a issue B
    BlockedBy,   // issue A está bloqueada por B (inversa de Blocks)
    Duplicate,   // issue A duplica a B
    RelatesTo,   // relación general
}
```

> [!WARNING] Relaciones simétricas
> Al crear `A blocks B`, se debe crear también la relación inversa `B blocked_by A`. Al eliminar una, eliminar la otra.

---

## Attachments — flujo S3/MinIO

```rust
// src/routes/issue_attachments.rs
pub async fn upload_attachment(
    State(state): State<AppState>,
    ProjectMemberGuard { user, .. }: ProjectMemberGuard,
    Path((_slug, project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentResponse>, AppError> {
    // 1. Extraer archivo del multipart
    let field = multipart.next_field().await?.ok_or(AppError::BadRequest("no file".into()))?;
    let filename = field.file_name().unwrap_or("file").to_string();
    let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
    let bytes = field.bytes().await.map_err(|_| AppError::BadRequest("read error".into()))?;

    // 2. Generar key en S3
    let key = format!("attachments/{project_id}/{issue_id}/{}", Uuid::new_v4());

    // 3. Upload a S3/MinIO
    state.s3.put_object()
        .bucket(&state.config.s3_bucket)
        .key(&key)
        .body(bytes.into())
        .content_type(&content_type)
        .send().await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 4. Registrar en issue_attachments
    let attachment = issue_attachments::ActiveModel {
        id:           Set(Uuid::new_v4()),
        issue_id:     Set(issue_id),
        project_id:   Set(project_id),
        file_name:    Set(filename),
        storage_key:  Set(key),
        content_type: Set(content_type),
        file_size:    Set(bytes.len() as i64),
        created_by_id: Set(Some(user.id)),
        ..Default::default()
    }.insert(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(AttachmentResponse::from(attachment)))
}
```

---

## DTOs principales

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateIssueRequest {
    pub name:             String,
    pub description:      Option<String>,
    pub description_html: Option<String>,
    pub priority:         Option<String>,   // "urgent"|"high"|"medium"|"low"|"none"
    pub state_id:         Option<Uuid>,
    pub parent_id:        Option<Uuid>,
    pub due_date:         Option<NaiveDate>,
    pub start_date:       Option<NaiveDate>,
    pub estimate_point:   Option<i32>,
    pub sort_order:       Option<f64>,
    pub label_ids:        Option<Vec<Uuid>>,
    pub assignee_ids:     Option<Vec<Uuid>>,
    pub cycle_id:         Option<Uuid>,
    pub module_ids:       Option<Vec<Uuid>>,
    pub sequence_id:      Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct IssueResponse {
    pub id:               Uuid,
    pub sequence_id:      i64,   // #ID legible (de issue_sequences)
    pub name:             String,
    pub description_html: Option<String>,
    pub priority:         String,
    pub state_id:         Option<Uuid>,
    pub parent_id:        Option<Uuid>,
    pub project_id:       Uuid,
    pub workspace_id:     Uuid,
    pub assignee_ids:     Vec<Uuid>,
    pub label_ids:        Vec<Uuid>,
    pub cycle_id:         Option<Uuid>,   // del join con cycle_issues activo
    pub module_ids:       Vec<Uuid>,
    pub sort_order:       f64,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
    pub created_by_id:    Option<Uuid>,
}
```

---

## Paginación — cursor-based

Issues usa paginación por cursor para evitar OFFSET en tablas grandes:

```rust
// src/utils/pagination.rs
pub struct CursorPage<T> {
    pub results:     Vec<T>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub count:       u64,
}

// El cursor codifica: sort_order + id (base64)
// Ejemplo: cursor="65535.0:uuid-xxxx"
pub fn decode_cursor(cursor: &str) -> Option<(f64, Uuid)> {
    let decoded = base64::engine::general_purpose::STANDARD.decode(cursor).ok()?;
    let s = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    let sort_order = parts[0].parse::<f64>().ok()?;
    let id = Uuid::parse_str(parts[1]).ok()?;
    Some((sort_order, id))
}
```

---

## Puntos críticos — issues

> [!WARNING] 8 puntos críticos específicos

1. **`sequence_id`** — se obtiene de `issue_sequences`, NO de `issues`. Siempre crear la fila en `issue_sequences` al crear un issue.
2. **`sort_order`** — tipo `float8` (f64). Al crear: `max(sort_order) + 10000`. Al reordenar: media aritmética entre los dos vecinos.
3. **Sub-issues** — `parent_id` en la misma tabla `issues`. No hay límite de profundidad en DB, pero el frontend solo muestra 1 nivel.
4. **Soft delete** — issues usa `deleted_at`. Siempre usar `.active()` en queries. Al `PATCH archived=true`, actualizar `archived_at` (no `deleted_at`).
5. **Assignees / Labels** — son M2M. `PATCH` con `label_ids: [...]` hace DELETE de todos + INSERT de los nuevos. Mismo patrón para assignees.
6. **IssueActivity** — cada campo cambiado genera una fila. Comparar old vs new antes del UPDATE.
7. **Reactions** — `reaction_code` es el codepoint Unicode como string (e.g. `"1F44D"` para 👍). Constraint unique por (issue_id, actor_id, reaction_code).
8. **Búsqueda** — `GET /issues/?search=texto` hace `ILIKE` en `name` y `description_html`. Ver [[dominio-busqueda]].

---

## Entidades SeaORM involucradas ✅

| Entidad | Tabla | Notas |
|---------|-------|-------|
| `issues.rs` | `issues` | `deleted_at`, `archived_at`, `parent_id` |
| `issue_sequences.rs` | `issue_sequences` | sequence_id legible |
| `issue_assignees.rs` | `issue_assignees` | M2M |
| `issue_labels.rs` | `issue_labels` | M2M |
| `issue_comments.rs` | `issue_comments` | soft delete |
| `comment_reactions.rs` | `comment_reactions` | |
| `issue_activities.rs` | `issue_activities` | audit trail |
| `issue_attachments.rs` | `issue_attachments` | + S3 |
| `issue_links.rs` | `issue_links` | URLs externas |
| `issue_reactions.rs` | `issue_reactions` | emoji por codepoint |
| `issue_relations.rs` | `issue_relations` | blocks/blocked_by/etc |
| `issue_subscribers.rs` | `issue_subscribers` | M2M |
| `issue_mentions.rs` | `issue_mentions` | M2M |
| `issue_votes.rs` | `issue_votes` | |

---

## Plan de implementación

```
Fase 2 (alta frecuencia):
  [ ] src/routes/issues.rs           — CRUD básico + labels CRUD
  [ ] src/repositories/issues.rs     — list_issues con filtros + cursor pagination
  [ ] src/utils/pagination.rs        — cursor encoder/decoder

Fase 4 (completar dominio):
  [ ] src/routes/issue_comments.rs   — CRUD comentarios + reacciones
  [ ] src/routes/issue_attachments.rs — upload S3 + registro DB
  [ ] src/utils/issue_activity.rs    — record_activity helper
  [ ] src/routes/issue_links.rs      — CRUD links
  [ ] src/routes/issue_relations.rs  — CRUD relaciones (con simetría)
  [ ] src/routes/issue_subscribers.rs — suscripciones
```

---

## 🔗 Navegar

← [[dominio-proyectos]] | [[MOC]] | → [[dominio-ciclos]]

**Relacionado:** Cycles: [[dominio-ciclos]] | Modules: [[dominio-modulos]] | Extractores: [[impl-extractores-auth]] | Búsqueda: [[dominio-busqueda]]
