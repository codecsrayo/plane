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
  - pendiente-implementar
  - todo-rs
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
    ├─ issue_reactions         (M2M → users, emoji codepoint)
    ├─ issue_attachments       (1→N, almacenamiento S3/MinIO)
    ├─ issue_links             (1→N, URLs externas)
    ├─ issue_comments          (1→N)
    │       └─ comment_reactions (M2M → users, emoji codepoint)
    ├─ issue_activities        (1→N, historial de cambios)
    ├─ issue_relations         (M2M reflexiva: blocks/blocked_by/duplicate/relates_to)
    ├─ issue_sequences         (1→1, genera el #ID legible e.g. WS-42)
    ├─ issue_mentions          (M2M → users, menciones en descripción)
    ├─ issue_votes             (M2M → users, sin endpoint en CE)
    ├─ issue_versions          (1→N, historial de versiones del issue)
    ├─ cycle_issues            (M2M → cycles)
    └─ module_issues           (M2M → modules)
```

---

## Endpoints a implementar

> [!INFO] Fuente
> Extraídos de `apps/api/plane/app/urls/issue.py` y `workspace.py` — Django.
> Los paths en Rust omiten el prefijo `/api/` que ya viene del router global.

### CRUD principal — `IssueViewSet`

| Método      | URL real Django                                         | Guard                      | Fase |
| ----------- | ------------------------------------------------------- | -------------------------- | ---- |
| `GET`       | `/workspaces/{slug}/projects/{project_id}/issues/`      | `ProjectMemberGuard (≥5)`  | 2    |
| `POST`      | `/workspaces/{slug}/projects/{project_id}/issues/`      | `ProjectMemberGuard (≥15)` | 2    |
| `GET`       | `/workspaces/{slug}/projects/{project_id}/issues/{pk}/` | `ProjectMemberGuard (≥5)`  | 2    |
| `PUT/PATCH` | `/workspaces/{slug}/projects/{project_id}/issues/{pk}/` | `ProjectMemberGuard (≥15)` | 2    |
| `DELETE`    | `/workspaces/{slug}/projects/{project_id}/issues/{pk}/` | `ProjectMemberGuard (≥15)` | 2    |

### Listados especializados

| Método | URL real Django                                           | Descripción                                                                                                                                                                      | Fase |
| ------ | --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| `GET`  | `/workspaces/{slug}/projects/{project_id}/issues/list/`   | `IssueListEndpoint` — lista ligera con IDs, sin joins pesados                                                                                                                    | 2    |
| `GET`  | `/workspaces/{slug}/projects/{project_id}/issues-detail/` | `IssueDetailEndpoint` — lista de issues con joins expandidos (assignees, labels, state populados); distinto del retrieve individual `/{pk}/` que devuelve un solo issue (INC-09) | 2    |
| `GET`  | `/workspaces/{slug}/projects/{project_id}/v2/issues/`     | `IssuePaginatedViewSet` — paginación cursor v2                                                                                                                                   | 4    |

### Labels

| Método                 | URL real Django                                                | Guard                           | Fase |
| ---------------------- | -------------------------------------------------------------- | ------------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{project_id}/issue-labels/`       | `ProjectMemberGuard (≥5/≥15)`   | 2    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/issue-labels/{pk}/`  | `ProjectMemberGuard (≥15)`      | 2    |
| `POST`                 | `/workspaces/{slug}/projects/{project_id}/bulk-create-labels/` | `BulkCreateIssueLabelsEndpoint` | 4    |

### Operaciones bulk

| Método  | URL real Django                                                 | Descripción                                              | Fase |
| ------- | --------------------------------------------------------------- | -------------------------------------------------------- | ---- |
| `POST`  | `/workspaces/{slug}/projects/{project_id}/bulk-delete-issues/`  | Elimina lista de issue IDs                               | 4    |
| `POST`  | `/workspaces/{slug}/projects/{project_id}/bulk-archive-issues/` | Archiva lista de issue IDs                               | 4    |
| `PATCH` | `/workspaces/{slug}/projects/{project_id}/issue-dates/`         | `IssueBulkUpdateDateEndpoint` — actualiza fechas en bulk | 4    |

### Sub-issues

| Método | URL real Django                                                          | Guard                                      | Fase |
| ------ | ------------------------------------------------------------------------ | ------------------------------------------ | ---- |
| `GET`  | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues/` | `SubIssuesEndpoint` — lista hijos directos | 4    |

### Archivado y papelera

| Método   | URL real Django                                                 | Descripción                                        | Fase |
| -------- | --------------------------------------------------------------- | -------------------------------------------------- | ---- |
| `GET`    | `/workspaces/{slug}/projects/{project_id}/archived-issues/`     | Lista issues archivados                            | 4    |
| `GET`    | `/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/` | Detalle de issue archivado                         | 4    |
| `POST`   | `/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/` | Archivar issue (sets `archived_at`)                | 4    |
| `DELETE` | `/workspaces/{slug}/projects/{project_id}/issues/{pk}/archive/` | Desarchivar issue (clears `archived_at`)           | 4    |
| `GET`    | `/workspaces/{slug}/projects/{project_id}/deleted-issues/`      | `DeletedIssuesListViewSet` — papelera soft-deleted | 4    |

### Meta e identificador legible

| Método | URL real Django                                                          | Descripción                                                              | Fase |
| ------ | ------------------------------------------------------------------------ | ------------------------------------------------------------------------ | ---- |
| `GET`  | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta/`       | `IssueMetaEndpoint` — conteos: sub-issues, links, attachments, reactions | 4    |
| `GET`  | `/workspaces/{slug}/work-items/{project_identifier}-{issue_identifier}/` | `IssueDetailIdentifierEndpoint` — lookup por slug legible e.g. `WS-42`   | 4    |

> [!WARNING] URL especial de identificador
> El path `/workspaces/{slug}/work-items/{project_identifier}-{issue_identifier}/` usa
> strings separados por guión literal, **no UUIDs**.
> En Axum: capturar como un solo `String` y hacer split por el último `-`.

### Comentarios

| Método                 | URL real Django                                                             | Guard                              | Fase |
| ---------------------- | --------------------------------------------------------------------------- | ---------------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/`      | `ProjectMemberGuard (≥5)`          | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/` | autor o `ProjectMemberGuard (≥15)` | 4    |

### Reacciones a comentarios — `CommentReactionViewSet`

| Método     | URL real Django                                                                             | Nota                                                               | Fase |
| ---------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---- |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/`                 | Path directo bajo `projects/{id}/`, sin `/issues/{id}/` intermedio | 4    |
| `DELETE`   | `/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/` | `reaction_code` = codepoint string                                 | 4    |

### Attachments

| Método     | URL real Django                                                                          | Descripción                               | Fase |
| ---------- | ---------------------------------------------------------------------------------------- | ----------------------------------------- | ---- |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/`          | v1                                        | 4    |
| `DELETE`   | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}/`     | v1                                        | 4    |
| `GET/POST` | `/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/`      | **v2 — prefix `/assets/v2/`, no `/api/`** | 4    |
| `DELETE`   | `/assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}/` | v2                                        | 4    |

> [!WARNING] Router separado para v2
> Los endpoints de attachments v2 usan `/assets/v2/` en lugar de `/api/`.
> En Axum requieren un `nest("/assets/v2", assets_router)` independiente del API router principal.

### Links

| Método                 | URL real Django                                                                | Guard                      | Fase |
| ---------------------- | ------------------------------------------------------------------------------ | -------------------------- | ---- |
| `GET/POST`             | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/`      | `ProjectMemberGuard (≥5)`  | 4    |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}/` | `ProjectMemberGuard (≥15)` | 4    |

### Historial / Actividad

| Método | URL real Django                                                       | Fase |
| ------ | --------------------------------------------------------------------- | ---- |
| `GET`  | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history/` | 4    |

### Versiones

| Método       | URL real Django                                                                                 | Descripción                          | Fase |
| ------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------ | ---- |
| `GET`        | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/`                          | Lista versiones                      | 4    |
| `GET/DELETE` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}/`                     | Detalle / eliminar versión           | 4    |
| `GET`        | `/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/`      | `WorkItemDescriptionVersionEndpoint` | 4    |
| `GET/DELETE` | `/workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}/` |                                      | 4    |

### Suscriptores

| Método     | URL real Django                                                                                 | Descripción                              | Fase |
| ---------- | ----------------------------------------------------------------------------------------------- | ---------------------------------------- | ---- |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/`                 | Lista y añadir suscriptores              | 4    |
| `DELETE`   | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}/` | Eliminar suscriptor por ID               | 4    |
| `GET`      | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/`                         | Estado de suscripción del usuario actual | 4    |
| `POST`     | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/`                         | Suscribirse                              | 4    |
| `DELETE`   | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/`                         | Desuscribirse                            | 4    |

### Reacciones al issue

| Método     | URL real Django                                                                         | Descripción                                                 | Fase |
| ---------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------- | ---- |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/`                 | Listar / añadir reacción                                    | 4    |
| `DELETE`   | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/` | `reaction_code` = codepoint Unicode string (e.g. `"1F44D"`) | 4    |

### Relaciones entre issues

| Método     | URL real Django                                                               | Descripción                                                     | Fase |
| ---------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------- | ---- |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation/`  | Listar / crear relación                                         | 4    |
| `POST`     | `/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation/` | **Eliminar relación — `POST` con body JSON, NO `DELETE /{id}`** | 4    |

> [!WARNING] Patrón atípico — remove-relation
> Django usa `POST /remove-relation/` con body `{"relation_type": "blocks", "related_issue_id": "uuid"}`.
> No existe endpoint `DELETE /issue-relation/{pk}/`.

### Propiedades de display del usuario

| Método      | URL real Django                                             | Descripción                                                                                                                                  | Fase |
| ----------- | ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| `GET/PATCH` | `/workspaces/{slug}/projects/{project_id}/user-properties/` | `ProjectUserDisplayPropertyEndpoint` — preferencias de filtros, agrupación, orden por usuario+proyecto (upsert — sin POST separado) (INC-06) | 4    |

### Workspace-level issues

| Método             | URL real Django                                 | Descripción                                            | Fase |
| ------------------ | ----------------------------------------------- | ------------------------------------------------------ | ---- |
| `GET`              | `/workspaces/{slug}/user-issues/{user_id}/`     | Issues asignados a un usuario específico del workspace | 4    |
| `GET/POST`         | `/workspaces/{slug}/draft-issues/`              | Borradores del workspace                               | 4    |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/draft-issues/{pk}/`         | Detalle / editar / eliminar borrador                   | 4    |
| `POST`             | `/workspaces/{slug}/draft-to-issue/{draft_id}/` | Promover borrador a issue real                         | 4    |

---

## Modelo de filtros — `GET /issues/`

```rust
#[derive(Deserialize, ToSchema)]
pub struct IssueQueryParams {
    // Paginación cursor-based
    pub cursor:       Option<String>,   // sort_order + id en base64
    pub per_page:     Option<u32>,      // default 100

    // Filtros básicos
    pub state_id:     Option<Vec<Uuid>>,
    pub assignee_id:  Option<Vec<Uuid>>,
    pub label_id:     Option<Vec<Uuid>>,
    pub cycle_id:     Option<Uuid>,
    pub module_id:    Option<Uuid>,
    pub priority:     Option<Vec<String>>, // "urgent","high","medium","low","none"
    pub mention:      Option<Uuid>,
    pub subscriber:   Option<Uuid>,
    pub created_by:   Option<Vec<Uuid>>,

    // Filtros de fecha
    pub created_at__gt:  Option<DateTime<Utc>>,
    pub updated_at__gt:  Option<DateTime<Utc>>,
    pub due_date__gt:    Option<NaiveDate>,
    pub due_date__lt:    Option<NaiveDate>,
    pub start_date__gt:  Option<NaiveDate>,
    pub start_date__lt:  Option<NaiveDate>,

    // Ordenamiento
    pub order_by:  Option<String>, // "created_at","-created_at","priority","sort_order","-due_date"

    // Estado
    pub state_group: Option<Vec<String>>, // "backlog","unstarted","started","completed","cancelled"
    pub archived:    Option<bool>,
    pub draft:       Option<bool>,
    pub sub_issue:   Option<bool>,
    pub search:      Option<String>,      // ILIKE en name
}
```

---

## Handler — `POST /issues/`

```rust
/// Prioridades válidas — mismos valores que Django (plane/db/models/issue.py)
const VALID_PRIORITIES: &[&str] = &["urgent", "high", "medium", "low", "none"];

pub async fn create_issue(
    State(state): State<AppState>,
    ProjectMemberGuard { user, project, workspace, .. }: ProjectMemberGuard,
    Path((_slug, project_id)): Path<(String, Uuid)>,
    Json(payload): Json<CreateIssueRequest>,
) -> Result<Json<IssueResponse>, AppError> {
    // ── Validaciones de input ──────────────────────────────────────────────

    // [Fix #11] Validar priority contra valores permitidos — evitar strings
    // arbitrarios almacenados en DB que romperían filtros y la UI.
    let priority = payload.priority.as_deref().unwrap_or("none");
    if !VALID_PRIORITIES.contains(&priority) {
        return Err(AppError::Validation(
            format!("priority debe ser uno de: {}", VALID_PRIORITIES.join(", "))
        ));
    }

    // [Fix #12] description_html viene del usuario — NO almacenar verbatim sin sanitizar.
    // El frontend (TipTap/ProseMirror) genera HTML estructurado; el backend
    // debe sanitizarlo para eliminar <script>, event handlers y URLs javascript:.
    // Usar la crate `ammonia` con el allowlist de tags de Tiptap.
    // NOTA: plan-riesgos ítem 7 aplica sólo al seed (fuente confiable), NO a input de usuario.
    //
    // Implementación requerida (Fase 2):
    //   let clean_html = ammonia::Builder::default()
    //       .tags(TIPTAP_ALLOWED_TAGS)
    //       .clean(&raw_html).to_string();
    //
    // Hasta que ammonia esté integrado, rechazar HTML con <script para bloquear
    // el vector más común de XSS almacenado:
    if let Some(ref html) = payload.description_html {
        if html.to_lowercase().contains("<script") {
            return Err(AppError::Validation("description_html contiene contenido no permitido".into()));
        }
    }

    let db = &state.db;

    // ── [Fix #14] Toda la creación en una transacción atómica ─────────────
    // Sin transacción: si falla el INSERT de issue_sequences (paso 3),
    // el issue queda en DB sin sequence_id → huérfano no navegable desde la UI.
    let txn = db.begin().await.map_err(AppError::Database)?;

    let result = async {
        // 1. sort_order = max existente + 10_000 (o 65_535 si vacío)
        let max_sort = issues::Entity::find()
            .active()
            .filter(issues::Column::ProjectId.eq(project_id))
            .select_only()
            .column_as(issues::Column::SortOrder.max(), "max_sort")
            .into_tuple::<Option<f64>>()
            .one(&txn).await?.flatten().unwrap_or(0.0);
        let sort_order = max_sort + 10_000.0;

        // 2. INSERT issue
        let issue_id = Uuid::new_v4();
        let issue = issues::ActiveModel {
            id:               Set(issue_id),
            name:             Set(payload.name),
            description_html: Set(payload.description_html),
            priority:         Set(priority.to_string()),
            state_id:         Set(payload.state_id),
            parent_id:        Set(payload.parent_id),
            project_id:       Set(project_id),
            workspace_id:     Set(workspace.id),
            sort_order:       Set(sort_order),
            created_by_id:    Set(Some(user.id)),
            ..Default::default()
        }.insert(&txn).await?;

        // 3. INSERT issue_sequences — obligatorio: sin esto el issue no tiene #ID legible
        let sequence = issue_sequences::ActiveModel {
            id: Set(Uuid::new_v4()), issue_id: Set(issue_id),
            project_id: Set(project_id), workspace_id: Set(workspace.id),
            ..Default::default()
        }.insert(&txn).await?;

        // 4. M2M labels
        for label_id in payload.label_ids.unwrap_or_default() {
            issue_labels::ActiveModel {
                id: Set(Uuid::new_v4()), issue_id: Set(issue_id), label_id: Set(label_id),
                project_id: Set(project_id), workspace_id: Set(workspace.id),
                ..Default::default()
            }.insert(&txn).await?;
        }

        // 5. M2M assignees
        for assignee_id in payload.assignee_ids.unwrap_or_default() {
            issue_assignees::ActiveModel {
                id: Set(Uuid::new_v4()), issue_id: Set(issue_id), assignee_id: Set(assignee_id),
                project_id: Set(project_id), workspace_id: Set(workspace.id),
                ..Default::default()
            }.insert(&txn).await?;
        }

        Ok::<_, sea_orm::DbErr>((issue, sequence, issue_id))
    }.await;

    match result {
        Ok((issue, sequence, issue_id)) => {
            txn.commit().await.map_err(AppError::Database)?;

            // [Fix #13] IssueActivity — loguear error en lugar de descartarlo con .ok()
            // record_activity es best-effort pero sus fallos deben ser visibles en logs.
            if let Err(e) = record_activity(db, issue_id, user.id, project_id, workspace.id, vec![
                FieldChange { field: "state".into(), old_value: None,
                              new_value: Some("created".into()), verb: "created".into() }
            ]).await {
                tracing::warn!(error = %e, issue_id = %issue_id, "Failed to record issue activity");
            }

            Ok(Json(IssueResponse::from_model(issue, sequence)))
        }
        Err(e) => {
            if let Err(rb_err) = txn.rollback().await { // silence-patterns-ok: rollback en error path, loguear pero no ocultar
                tracing::error!(rollback_error = %rb_err, original_error = %e, "Transaction rollback failed after insert error");
            }
            Err(AppError::Database(e))
        }
    }
}
```

---

## IssueActivity — registro de cambios

```rust
// src/utils/issue_activity.rs
pub struct FieldChange {
    pub field:     String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub verb:      String,  // "created","updated","deleted","added","removed"
}

pub async fn record_activity(db, issue_id, actor_id, project_id, workspace_id, changes) {
    for change in changes {
        issue_activities::ActiveModel {
            id: Set(Uuid::new_v4()), issue_id: Set(issue_id),
            actor_id: Set(actor_id), project_id: Set(project_id),
            workspace_id: Set(workspace_id),
            field: Set(Some(change.field)), old_value: Set(change.old_value),
            new_value: Set(change.new_value), verb: Set(change.verb),
            ..Default::default()
        }.insert(db).await?;
    }
}
```

**Campos que generan actividad:** `state_id`, `priority`, `assignees`, `labels`, `name`, `description`, `due_date`, `start_date`, `estimate_point`, `parent_id`, `cycle`, `module`, `link`, `attachment`, `archived`.

---

## IssueRelations — tipos y simetría

```rust
pub enum IssueRelationType {
    Blocks,      // A bloquea B → también crea B BlockedBy A
    BlockedBy,   // inversa
    Duplicate,   // A duplica B → también crea B DuplicateOf A
    RelatesTo,   // simétrica
}
```

> [!WARNING] Doble INSERT / Doble DELETE
> Crear `A blocks B` → 2 filas en `issue_relations`.
> `POST /remove-relation/` con body → eliminar ambas filas.

---

## DTOs principales

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateIssueRequest {
    pub name:             String,
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
}

#[derive(Deserialize, ToSchema)]
pub struct RemoveRelationRequest {
    pub relation_type:    String,   // "blocks"|"blocked_by"|"duplicate_of"|"relates_to"
    pub related_issue_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct IssueResponse {
    pub id:               Uuid,
    pub sequence_id:      i64,          // de issue_sequences, NO de issues
    pub name:             String,
    pub description_html: Option<String>,
    pub priority:         String,
    pub state_id:         Option<Uuid>,
    pub parent_id:        Option<Uuid>,
    pub project_id:       Uuid,
    pub workspace_id:     Uuid,
    pub assignee_ids:     Vec<Uuid>,
    pub label_ids:        Vec<Uuid>,
    pub cycle_id:         Option<Uuid>, // join activo con cycle_issues
    pub module_ids:       Vec<Uuid>,
    pub sort_order:       f64,
    pub archived_at:      Option<DateTime<Utc>>,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
    pub created_by_id:    Option<Uuid>,
}
```

---

## Paginación — cursor-based

```rust
// cursor = base64( sort_order + ":" + uuid )
pub fn decode_cursor(cursor: &str) -> Option<(f64, Uuid)> {
    // Cursor válido = base64( f64_str + ":" + uuid ) ≈ máximo ~60 chars en base64.
    // Rechazar inputs gigantes antes de decodificar para evitar asignación innecesaria.
    if cursor.len() > 128 {
        return None;
    }
    let decoded = base64::engine::general_purpose::STANDARD.decode(cursor).ok()?;
    let s = String::from_utf8(decoded).ok()?;
    let (sort_str, id_str) = s.splitn(2, ':').collect_tuple()?;
    Some((sort_str.parse().ok()?, Uuid::parse_str(id_str).ok()?))
}
```

---

## Puntos críticos

> [!WARNING] 10 puntos críticos

1. **`sequence_id`** — de `issue_sequences`, nunca de `issues`. Siempre INSERT en ambas tablas al crear.
2. **`sort_order`** — `float8`. Crear: `max + 10_000`. Reordenar: media aritmética entre vecinos.
3. **Sub-issues** — `parent_id` en la misma tabla. DB no limita profundidad; frontend solo 1 nivel.
4. **Soft delete vs archive** — `deleted_at` para borrado lógico; `archived_at` para archivado. Son estados independientes; siempre usar `.active()` (filtra `deleted_at IS NULL`).
5. **M2M Assignees/Labels** — en `PATCH`: DELETE todos los existentes + INSERT los nuevos. Registrar en IssueActivity.
6. **IssueActivity** — comparar old vs new antes del UPDATE; una fila por campo cambiado.
7. **Reactions** — `reaction_code` = codepoint Unicode string (e.g. `"1F44D"`). Unique `(issue_id, actor_id, reaction_code)`.
8. **Relations simétricas** — crear/eliminar siempre opera en ambas direcciones.
9. **`remove-relation` usa POST** — el body JSON identifica la relación. No existe `DELETE /issue-relation/{pk}/`.
10. **Attachments v2** — prefix `/assets/v2/`, router Axum separado del API principal.

---

## Entidades SeaORM involucradas

| Entidad                | Tabla               | Notas                                    |
| ---------------------- | ------------------- | ---------------------------------------- |
| `issues.rs`            | `issues`            | `deleted_at`, `archived_at`, `parent_id` |
| `issue_sequences.rs`   | `issue_sequences`   | auto-increment per-project               |
| `issue_assignees.rs`   | `issue_assignees`   | M2M                                      |
| `issue_labels.rs`      | `issue_labels`      | M2M                                      |
| `issue_comments.rs`    | `issue_comments`    | soft delete                              |
| `comment_reactions.rs` | `comment_reactions` | emoji codepoint                          |
| `issue_activities.rs`  | `issue_activities`  | audit trail                              |
| `issue_attachments.rs` | `issue_attachments` | S3/MinIO key                             |
| `issue_links.rs`       | `issue_links`       | URLs externas                            |
| `issue_reactions.rs`   | `issue_reactions`   | emoji codepoint                          |
| `issue_relations.rs`   | `issue_relations`   | M2M reflexiva                            |
| `issue_subscribers.rs` | `issue_subscribers` | M2M                                      |
| `issue_mentions.rs`    | `issue_mentions`    | M2M                                      |
| `issue_votes.rs`       | `issue_votes`       | Sin endpoint en CE (INC-14)              |
| `issue_versions.rs`    | `issue_versions`    | historial                                |

---

## Plan de implementación

```
Fase 2:
  [ ] src/routes/issues.rs              — CRUD + IssueListEndpoint + IssueDetailEndpoint
  [ ] src/repositories/issues.rs        — list_issues con filtros + cursor pagination
  [ ] src/utils/pagination.rs           — cursor encoder/decoder
  [ ] src/routes/labels.rs              — CRUD labels de proyecto

Fase 4:
  [ ] src/routes/issue_comments.rs      — CRUD comentarios
  [ ] src/routes/comment_reactions.rs   — reacciones a comentarios
  [ ] src/routes/issue_attachments.rs   — upload S3 v1 + router /assets/v2/
  [ ] src/utils/issue_activity.rs       — record_activity helper
  [ ] src/routes/issue_links.rs         — CRUD links
  [ ] src/routes/issue_relations.rs     — CRUD + POST remove-relation (simétrico)
  [ ] src/routes/issue_subscribers.rs   — suscripciones
  [ ] src/routes/issue_reactions.rs     — reacciones por reaction_code
  [ ] src/routes/issue_archive.rs       — archive/unarchive + deleted list
  [ ] src/routes/issue_versions.rs      — versiones + description-versions
  [ ] src/routes/draft_issues.rs        — borradores workspace-level
  [ ] src/routes/issue_meta.rs          — meta endpoint + identifier lookup
```

---

## 🔗 Navegar

← [[dominio-proyectos]] | [[MOC]] | → [[dominio-ciclos]]

**Relacionado:** Cycles: [[dominio-ciclos]] | Modules: [[dominio-modulos]] | Extractores: [[impl-extractores-auth]] | Búsqueda: [[dominio-busqueda]]
