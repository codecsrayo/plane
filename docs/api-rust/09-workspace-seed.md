---
titulo: WorkspaceSeedJob — población inicial
tags:
  - apalis
  - seed
  - workspace
  - jobs
relacionado: [[10-patrones]], [[11-diagramas-secuencia]]
---

## Población de datos iniciales (workspace seed)

### Qué hace el seed en Django

`workspace_seed_task.py` es un **Celery task** disparado con `.delay(workspace_id)`
inmediatamente después de `POST /api/workspaces/`. Crea de forma asíncrona:

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

Los datos de plantilla viven en **8 archivos JSON** en `apps/api/plane/seeds/data/`:

| Archivo          | Descripción                                     |
| ---------------- | ----------------------------------------------- |
| `projects.json`  | 1 proyecto demo con nombre, identifier, logo    |
| `states.json`    | 5 estados con color, grupo (backlog/started/…)  |
| `labels.json`    | 2 labels (admin, concepts)                      |
| `cycles.json`    | 2 ciclos con tipo CURRENT / UPCOMING            |
| `modules.json`   | N módulos con nombre y orden                    |
| `issues.json`    | N issues con description_html, priority, refs   |
| `views.json`     | N vistas con filtros                            |
| `pages.json`     | N páginas con description_html                  |

### Migración a Rust — WorkspaceSeedJob (apalis)

#### Paso 1 — Copiar los JSON al proyecto Rust

```bash
mkdir -p apps/api_rust/seeds/data
cp apps/api/plane/seeds/data/*.json apps/api_rust/seeds/data/
```

Los JSON se incluyen en el binario con `include_str!()` para evitar rutas
en runtime, o se montan como volumen en Docker. Recomendado: `include_str!`
para garantizar que siempre estén presentes.

#### Paso 2 — Estructuras de deserialización

```rust
// src/jobs/workspace_seed/seed_data.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectSeed {
    pub id: i32,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub network: i16,
    pub cover_image: Option<String>,
    pub logo_props: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StateSeed {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub sequence: f64,
    pub group: String,
    pub default: bool,
    pub project_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct LabelSeed {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub sort_order: f64,
    pub project_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CycleSeed {
    pub id: i32,
    pub name: String,
    pub project_id: i32,
    #[serde(rename = "type")]
    pub cycle_type: String, // "CURRENT" | "UPCOMING"
}

#[derive(Debug, Deserialize)]
pub struct IssueSeed {
    pub id: i32,
    pub name: String,
    pub sequence_id: i32,
    pub description_html: Option<String>,
    pub description_stripped: Option<String>,
    pub sort_order: f64,
    pub state_id: i32,
    pub labels: Vec<i32>,
    pub priority: String,
    pub project_id: i32,
    pub cycle_id: Option<i32>,
    pub module_ids: Option<Vec<i32>>,
}
```

#### Paso 3 — El job apalis

```rust
// src/jobs/workspace_seed/mod.rs
use apalis::prelude::*;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSeedJob {
    pub workspace_id: Uuid,
}

pub async fn handle_workspace_seed(
    job: WorkspaceSeedJob,
    ctx: Data<DatabaseConnection>,
) -> Result<(), apalis::prelude::Error> {
    let db = ctx.as_ref();
    seed_workspace(db, job.workspace_id).await
        .map_err(|e| apalis::prelude::Error::Failed(e.to_string().into()))
}

async fn seed_workspace(db: &DatabaseConnection, workspace_id: Uuid) -> anyhow::Result<()> {
    // 1. Verificar idempotencia
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
    // …modules, views, pages igual

    // 3. Crear bot user + agregar a workspace
    let bot_id = create_bot_user(db, workspace_id).await?;
    add_bot_to_workspace(db, workspace_id, bot_id).await?;

    // 4. Obtener workspace_members para propagar al proyecto
    let members = workspace_members::Entity::find()
        .filter(workspace_members::Column::WorkspaceId.eq(workspace_id))
        .all(db).await?;

    // 5. Crear proyecto + miembros + user_properties
    // Mapa seed_id(i32) → real_uuid
    let project_map: HashMap<i32, Uuid> =
        create_project_and_members(db, workspace_id, &projects_tpl, &members, bot_id).await?;

    // 6. Estados, labels, ciclos, módulos — en orden (FKs)
    let state_map  = create_states(db, workspace_id, &states_tpl, &project_map, bot_id).await?;
    let label_map  = create_labels(db, workspace_id, &labels_tpl, &project_map, bot_id).await?;
    let cycle_map  = create_cycles(db, workspace_id, &cycles_tpl, &project_map, bot_id).await?;
    let module_map = create_modules(db, workspace_id, &modules_tpl, &project_map, bot_id).await?;

    // 7. Issues con todas sus relaciones
    create_issues(db, workspace_id, &issues_tpl,
        &project_map, &state_map, &label_map, &cycle_map, &module_map, bot_id).await?;

    // 8. Views y pages
    create_views(db, workspace_id, &views_tpl, &project_map, bot_id).await?;
    create_pages(db, workspace_id, &pages_tpl, &project_map, bot_id).await?;

    tracing::info!("Workspace {workspace_id} seeded successfully");
    Ok(())
}
```

#### Paso 4 — Encolar el job desde el handler de workspaces

```rust
// src/routes/workspaces.rs — POST /api/workspaces/
async fn create_workspace(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    // … crear workspace y workspace_member …

    // Encolar seed job asíncrono — no bloquea la respuesta HTTP
    if let Err(e) = state.job_storage
        .push(WorkspaceSeedJob { workspace_id: new_workspace.id })
        .await
    {
        tracing::warn!("Failed to enqueue workspace seed: {e}");
        // No fallar la request — el seed es best-effort
    }

    Ok(Json(workspace_response))
}
```

#### Mapa de IDs — patrón crítico

Los JSON usan IDs enteros temporales (1, 2, 3…). Al insertar en Postgres
se generan UUIDs reales. `HashMap<i32, Uuid>` resuelve referencias cruzadas:

```rust
let mut state_map: HashMap<i32, Uuid> = HashMap::new();

for seed in &states_tpl {
    let model = states::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(seed.name.clone()),
        // …
    }.insert(db).await?;
    state_map.insert(seed.id, model.id);
}

// Al crear issue: resolver FK
let real_state_id = state_map[&issue_seed.state_id];
```

### Datos globales (estáticos) vs datos de workspace (dinámicos)

| Tipo                        | Dónde                          | Cuándo              |
| --------------------------- | ------------------------------ | ------------------- |
| `integrations` (3 filas)    | migración `m007_seed_data`     | al arrancar DB      |
| `instance_configurations`   | migración `m007_seed_data`     | al arrancar DB      |
| `auth_permission` / grupos  | migración `m001_auth_django`   | al arrancar DB      |
| Proyecto demo + issues      | `WorkspaceSeedJob` (apalis)    | al crear workspace  |
| Bot user por workspace      | `WorkspaceSeedJob` (apalis)    | al crear workspace  |

---

> [!IMPORTANT] Orden de inserción obligatorio
> `workspace → bot_user → project → states → labels → cycles → modules → issues → views → pages`
> Las FKs en Postgres fallarán si se inserta fuera de orden.

> [!NOTE] IDs temporales → UUIDs reales
> Los JSON usan enteros (1, 2, 3…). Usar `HashMap<i32, Uuid>` para resolver las
> referencias cruzadas al insertar. Ver sección "Mapa de IDs" en esta nota.

