---
titulo: WorkspaceSeedJob — población inicial del workspace
aliases:
  - workspace-seed
  - WorkspaceSeedJob
  - seed
tags:
  - apalis
  - seed
  - workspace
  - jobs
  - dominio
relacionado:
  - "[[MOC]]"
  - "[[impl-error-jobs-cron]]"
  - "[[ref-diagramas-secuencia]]"
  - "[[plan-riesgos]]"
  - "[[ref-estructura-archivos]]"
estado: activo
---

# WorkspaceSeedJob — población inicial del workspace

---

## Qué hace el seed en Django

`workspace_seed_task.py` es un **Celery task** disparado con `.delay(workspace_id)` inmediatamente después de `POST /api/workspaces/`. Crea de forma asíncrona:

```
workspace creado
    └─ bot_user               (User.is_bot=true, bot_type=WORKSPACE_SEED)
    └─ WorkspaceMember        (rol 20 = admin para el bot)
    └─ Project                (nombre = nombre del workspace)
        ├─ ProjectMember      (todos los workspace_members heredan rol)
        ├─ ProjectUserProperty (display_filters + display_properties por user)
        ├─ States             × 5  (Backlog, Todo, In Progress, Done, Cancelled)
        ├─ Labels             × 2  (admin, concepts)
        ├─ Cycles             × 2  (CURRENT: hoy+14d, UPCOMING: siguiente bloque)
        ├─ Modules            × N
        ├─ Issues             × N  (con IssueSequence + IssueActivity + labels/cycles/modules)
        ├─ IssueViews         × N
        └─ Pages              × N  (globales y de proyecto)
```

---

## Archivos JSON de datos

Los datos de plantilla viven en 8 archivos JSON en `apps/api_rust/seeds/data/`:

| Archivo         | Descripción                                    |
| --------------- | ---------------------------------------------- |
| `projects.json` | 1 proyecto demo con nombre, identifier, logo   |
| `states.json`   | 5 estados con color, grupo (backlog/started/…) |
| `labels.json`   | 2 labels (admin, concepts)                     |
| `cycles.json`   | 2 ciclos con tipo CURRENT / UPCOMING           |
| `modules.json`  | N módulos con nombre y orden                   |
| `issues.json`   | N issues con description_html, priority, refs  |
| `views.json`    | N vistas con filtros                           |
| `pages.json`    | N páginas con description_html                 |

```bash
# Copiar desde el proyecto Django
mkdir -p apps/api_rust/seeds/data
cp apps/api/plane/seeds/data/*.json apps/api_rust/seeds/data/
```

Los JSON se incluyen en el binario con `include_str!()` — no necesitan rutas en runtime.

---

## Estructuras de deserialización

```rust
// src/jobs/workspace_seed/seed_data.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectSeed {
    pub id:          i32,
    pub name:        String,
    pub identifier:  String,
    pub description: Option<String>,
    pub network:     i16,
    pub logo_props:  serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StateSeed {
    pub id:         i32,
    pub name:       String,
    pub color:      String,
    pub sequence:   f64,
    pub group:      String,
    pub default:    bool,
    pub project_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CycleSeed {
    pub id:         i32,
    pub name:       String,
    pub project_id: i32,
    #[serde(rename = "type")]
    pub cycle_type: String, // "CURRENT" | "UPCOMING"
}

#[derive(Debug, Deserialize)]
pub struct IssueSeed {
    pub id:                  i32,
    pub name:                String,
    pub sequence_id:         i32,
    pub description_html:    Option<String>,
    pub sort_order:          f64,
    pub state_id:            i32,
    pub labels:              Vec<i32>,
    pub priority:            String,
    pub project_id:          i32,
    pub cycle_id:            Option<i32>,
    pub module_ids:          Option<Vec<i32>>,
}
```

---

## El job apalis

```rust
// src/jobs/workspace_seed/mod.rs
use apalis::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSeedJob {
    pub workspace_id: Uuid,
}

pub async fn handle_workspace_seed(
    job: WorkspaceSeedJob,
    ctx: Data<sea_orm::DatabaseConnection>,
) -> Result<(), apalis::prelude::Error> {
    let db = ctx.as_ref();
    seed_workspace(db, job.workspace_id).await
        .map_err(|e| apalis::prelude::Error::Failed(e.to_string().into()))
}

async fn seed_workspace(
    db: &sea_orm::DatabaseConnection,
    workspace_id: Uuid,
) -> anyhow::Result<()> {
    // 1. Idempotencia — no re-sembrar si ya existe un proyecto
    let existing = projects::Entity::find()
        .filter(projects::Column::WorkspaceId.eq(workspace_id))
        .count(db).await?;
    if existing > 0 {
        tracing::info!("Workspace {workspace_id} already seeded, skipping");
        return Ok(());
    }

    // 2. Cargar seeds desde JSON embebido en el binario
    let projects_tpl: Vec<ProjectSeed> = serde_json::from_str(
        include_str!("../../seeds/data/projects.json"))?;
    let states_tpl: Vec<StateSeed> = serde_json::from_str(
        include_str!("../../seeds/data/states.json"))?;
    let labels_tpl: Vec<LabelSeed> = serde_json::from_str(
        include_str!("../../seeds/data/labels.json"))?;
    let cycles_tpl: Vec<CycleSeed> = serde_json::from_str(
        include_str!("../../seeds/data/cycles.json"))?;
    let issues_tpl: Vec<IssueSeed> = serde_json::from_str(
        include_str!("../../seeds/data/issues.json"))?;
    // ...modules, views, pages igual

    // 3. Bot user
    let bot_id = create_bot_user(db, workspace_id).await?;
    add_bot_to_workspace(db, workspace_id, bot_id).await?;

    // 4. Workspace members actuales
    let members = workspace_members::Entity::find()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
        .all(db).await?;

    // 5. Proyecto + miembros — devuelve mapa seed_id(i32) → real_uuid
    let project_map = create_project_and_members(
        db, workspace_id, &projects_tpl, &members, bot_id
    ).await?;

    // 6. Resto en orden estricto (respeta FKs)
    let state_map  = create_states(db, workspace_id, &states_tpl, &project_map, bot_id).await?;
    let label_map  = create_labels(db, workspace_id, &labels_tpl, &project_map, bot_id).await?;
    let cycle_map  = create_cycles(db, workspace_id, &cycles_tpl, &project_map, bot_id).await?;
    let module_map = create_modules(db, workspace_id, &modules_tpl, &project_map, bot_id).await?;

    // 7. Issues con relaciones
    create_issues(db, workspace_id, &issues_tpl,
        &project_map, &state_map, &label_map, &cycle_map, &module_map, bot_id).await?;

    // 8. Views y pages
    create_views(db, workspace_id, &views_tpl, &project_map, bot_id).await?;
    create_pages(db, workspace_id, &pages_tpl, &project_map, bot_id).await?;

    tracing::info!("Workspace {workspace_id} seeded successfully");
    Ok(())
}
```

---

## Mapa de IDs — patrón crítico

Los JSON usan IDs enteros temporales (1, 2, 3…). Al insertar en Postgres se generan UUIDs reales. `HashMap<i32, Uuid>` resuelve referencias cruzadas:

```rust
let mut state_map: HashMap<i32, Uuid> = HashMap::new();

for seed in &states_tpl {
    let model = states::ActiveModel {
        id:   Set(Uuid::new_v4()),
        name: Set(seed.name.clone()),
        // ...
    }.insert(db).await?;
    state_map.insert(seed.id, model.id);
}

// Al crear issue: resolver FK
let real_state_id = state_map[&issue_seed.state_id];
```

---

## Encolar el job desde el handler de workspaces

```rust
// src/routes/workspaces.rs — POST /api/workspaces/
async fn create_workspace(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    // ... crear workspace y workspace_member ...

    // Best-effort — no bloquear la respuesta HTTP
    if let Err(e) = state.job_storage
        .push(WorkspaceSeedJob { workspace_id: new_workspace.id })
        .await
    {
        tracing::warn!("Failed to enqueue workspace seed: {e}");
    }

    Ok(Json(workspace_response))
}
```

---

## Datos globales vs datos de workspace

| Tipo                       | Dónde                       | Cuándo             |
| -------------------------- | --------------------------- | ------------------ |
| `integrations` (3 filas)   | Migración `m007_seed_data`  | Al arrancar DB     |
| `instance_configurations`  | Migración `m007_seed_data`  | Al arrancar DB     |
| `auth_permission` / grupos | Migración `m001_baseline`   | Al arrancar DB     |
| Proyecto demo + issues     | `WorkspaceSeedJob` (apalis) | Al crear workspace |
| Bot user por workspace     | `WorkspaceSeedJob` (apalis) | Al crear workspace |

---

## Puntos críticos específicos del seed

> [!WARNING] Ver [[plan-riesgos]] para la lista completa de riesgos

1. **Orden obligatorio:** `workspace → bot_user → project → states → labels → cycles → modules → issues → views → pages`
2. **IDs temporales → UUIDs:** usar `HashMap<i32, Uuid>` para todas las referencias cruzadas
3. **`IssueSequence`:** crear por cada issue (genera el `#` de referencia)
4. **`ProjectIdentifier`:** crear al crear el proyecto (genera el prefijo `WS-`)
5. **`bot_type` enum:** verificar que `'WORKSPACE_SEED'` existe en el enum Postgres
6. **Ciclos:** calcular fechas en runtime, no usar fechas del JSON
7. **`description_html`:** insertar verbatim, sin sanitizar

---

## 🔗 Navegar

← [[impl-error-jobs-cron]] | [[MOC]] | → [[dominio-integraciones]]

**Relacionado:** Diagrama de secuencia: [[ref-diagramas-secuencia#1. Creación de workspace + seed asíncrono]] | Riesgos: [[plan-riesgos]] | Estructura de archivos: [[ref-estructura-archivos]]
