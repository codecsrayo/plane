---
titulo: Dominio — Projects
aliases:
  - proyectos
  - projects
  - dominio-proyectos
tags:
  - projects
  - dominio
  - rust
  - axum
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-ciclos]]"
  - "[[dominio-modulos]]"
  - "[[impl-extractores-auth]]"
  - "[[plan-fases]]"
estado: activo
---

# Dominio — Projects

> [!NOTE] Recurso principal tras workspaces
> Un proyecto vive dentro de un workspace. Tiene sus propios miembros, roles, estados, labels, cycles y módulos.

---

## Modelo de datos

```
workspaces
    └─ projects
            ├─ project_members          (M2M → users, rol propio)
            ├─ project_member_invites   (invitaciones pendientes)
            ├─ project_identifiers      (prefijo único e.g. "WS", "PROJ")
            ├─ states                   (estados propios del proyecto)
            ├─ labels                   (labels propios, ver dominio-issues)
            ├─ estimates                (sistema de puntos)
            │       └─ estimate_points  (valores posibles del sistema)
            ├─ deploy_boards            (tableros públicos)
            └─ project_user_properties  (preferencias por usuario)
```

---

## Endpoints a implementar

> [!INFO] Fuente
> `apps/api/plane/app/urls/project.py`, `state.py`, `estimate.py` — Django.
> Los paths NO incluyen el prefijo `/api/` (ya viene del router global).

### CRUD de proyecto — `ProjectViewSet`

| Método | URL real Django | Guard | Fase |
|--------|----------------|-------|------|
| `GET` | `/workspaces/{slug}/projects/` | `WorkspaceMemberGuard (≥5)` | 2 |
| `POST` | `/workspaces/{slug}/projects/` | `WorkspaceMemberGuard (≥15)` | 2 |
| `GET` | `/workspaces/{slug}/projects/details/` | `WorkspaceMemberGuard (≥5)` — `list_detail` con joins | 2 |
| `GET` | `/workspaces/{slug}/projects/{pk}/` | `ProjectMemberGuard (≥5)` | 2 |
| `PUT/PATCH` | `/workspaces/{slug}/projects/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |
| `DELETE` | `/workspaces/{slug}/projects/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |

### Identificadores de proyecto — `ProjectIdentifierEndpoint`

| Método | URL real Django | Guard | Fase |
|--------|----------------|-------|------|
| `GET` | `/workspaces/{slug}/project-identifiers/` | `WorkspaceMemberGuard (≥5)` | 2 |
| `DELETE` | `/workspaces/{slug}/project-identifiers/` | `WorkspaceMemberGuard (≥20)` | 4 |

> [!NOTE] Validación de identifier
> Al crear proyecto, el `identifier` (e.g. `"WS"`) debe ser único por workspace. Se almacena en `project_identifiers` en la misma transacción que `projects`.

### Miembros — `ProjectMemberViewSet`

| Método | URL real Django | Guard | Fase |
|--------|----------------|-------|------|
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/members/` | `ProjectMemberGuard (≥5/≥20)` | 2 |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/members/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |
| `POST` | `/workspaces/{slug}/projects/{project_id}/members/leave/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET` | `/workspaces/{slug}/projects/{project_id}/project-members/me/` | `ProjectMemberGuard (≥5)` — info del usuario actual | 2 |

### Invitaciones — `ProjectInvitationsViewset`

| Método | URL real Django | Guard | Fase |
|--------|----------------|-------|------|
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/invitations/` | `ProjectMemberGuard (≥20)` | 4 |
| `GET/DELETE` | `/workspaces/{slug}/projects/{project_id}/invitations/{pk}/` | `ProjectMemberGuard (≥20)` | 4 |
| `POST` | `/workspaces/{slug}/projects/{project_id}/join/{pk}/` | Sin auth — token en URL | 4 |
| `GET/POST` | `/users/me/workspaces/{slug}/projects/invitations/` | Usuario autenticado | 4 |
| `GET` | `/users/me/workspaces/{slug}/project-roles/` | Usuario autenticado — `UserProjectRolesEndpoint` | 4 |

### Favoritos — `ProjectFavoritesViewSet`

| Método | URL real Django | Fase |
|--------|----------------|------|
| `GET/POST` | `/workspaces/{slug}/user-favorite-projects/` | 4 |
| `DELETE` | `/workspaces/{slug}/user-favorite-projects/{project_id}/` | 4 |

### Archivar — `ProjectArchiveUnarchiveEndpoint`

| Método | URL real Django | Descripción | Fase |
|--------|----------------|-------------|------|
| `POST` | `/workspaces/{slug}/projects/{project_id}/archive/` | Archivar proyecto | 4 |
| `DELETE` | `/workspaces/{slug}/projects/{project_id}/archive/` | Desarchivar proyecto | 4 |

### Deploy boards — `DeployBoardViewSet`

| Método | URL real Django | Fase |
|--------|----------------|------|
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/project-deploy-boards/` | 4 |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}/` | 4 |

### Views / preferencias de usuario por proyecto

| Método | URL real Django | Descripción | Fase |
|--------|----------------|-------------|------|
| `GET/PATCH` | `/workspaces/{slug}/projects/{project_id}/project-views/` | `ProjectUserViewsEndpoint` — filtros y vistas guardadas | 4 |
| `GET/PATCH` | `/workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}/` | `ProjectMemberPreferenceEndpoint` — notificaciones y prefs del miembro | 4 |

---

## States — estados del proyecto

### Endpoints — `StateViewSet` + `IntakeStateEndpoint`

| Método | URL real Django | Descripción | Fase |
|--------|----------------|-------------|------|
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/states/` | Listar / crear estado | 2 |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/states/{pk}/` | Detalle / editar / eliminar | 2 |
| `POST` | `/workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default/` | Marcar estado como default del grupo | 4 |
| `GET` | `/workspaces/{slug}/projects/{project_id}/intake-state/` | `IntakeStateEndpoint` — estado especial de Intake/Triage | 4 |

> [!WARNING] Django expone `PATCH` en states, no `PUT`
> El `StateViewSet` usa `partial_update` (PATCH), no `update` (PUT). En Axum solo registrar PATCH.

> [!WARNING] No DELETE si hay issues activos
> Antes de eliminar un state: verificar `COUNT(*) FROM issues WHERE state_id = pk AND deleted_at IS NULL`.
> Si > 0 → retornar 400 con mensaje descriptivo.

### Grupos de estado (fijos en Django)

| Grupo | Descripción | Color default |
|-------|-------------|---------------|
| `backlog` | No iniciado | `#526B9A` |
| `unstarted` | Pendiente | `#3F76FF` |
| `started` | En progreso | `#F4C430` |
| `completed` | Terminado | `#17B26A` |
| `cancelled` | Cancelado | `#EF4444` |

---

## Estimates — puntos de estimación

### Endpoints

> [!INFO] Dos endpoints distintos para estimates
> `ProjectEstimatePointEndpoint` en `/project-estimates/` (operaciones a nivel de proyecto, e.g. activar/desactivar sistema)
> `BulkEstimatePointEndpoint` en `/estimates/` (CRUD del esquema de estimación)

| Método | URL real Django | Descripción | Fase |
|--------|----------------|-------------|------|
| `GET/PATCH` | `/workspaces/{slug}/projects/{project_id}/project-estimates/` | `ProjectEstimatePointEndpoint` — gestión del sistema activo | 4 |
| `GET/POST` | `/workspaces/{slug}/projects/{project_id}/estimates/` | `BulkEstimatePointEndpoint` — CRUD del esquema | 4 |
| `GET/PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/` | Detalle del esquema | 4 |
| `POST` | `/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/` | Crear punto de estimación | 4 |
| `PATCH/DELETE` | `/workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{estimate_point_id}/` | Editar / eliminar punto | 4 |

> [!WARNING] `estimate_point_id` NO es UUID
> En Django: `<estimate_point_id>` sin tipo explícito → captura string arbitrario.
> En Axum: usar `Path<(String, Uuid, String)>` o `Path<(String, Uuid, Uuid)>` según lo que retorna la DB (verificar entidad SeaORM).

---

## Handler — `POST /projects/`

```rust
pub async fn create_project(
    State(state): State<AppState>,
    WorkspaceMemberGuard { user, workspace, .. }: WorkspaceMemberGuard,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    let db = &state.db;

    // 1. Verificar identifier único en el workspace
    let exists = project_identifiers::Entity::find()
        .filter(project_identifiers::Column::WorkspaceId.eq(workspace.id))
        .filter(project_identifiers::Column::Identifier.eq(&payload.identifier))
        .count(db).await? > 0;
    if exists {
        return Err(AppError::Conflict("identifier already in use".into()));
    }

    // 2. INSERT project
    let project_id = Uuid::new_v4();
    let project = projects::ActiveModel {
        id:            Set(project_id),
        name:          Set(payload.name),
        identifier:    Set(payload.identifier.to_uppercase()),
        description:   Set(payload.description.unwrap_or_default()),
        network:       Set(payload.network.unwrap_or(2)), // 0=public, 2=secret
        workspace_id:  Set(workspace.id),
        created_by_id: Set(Some(user.id)),
        ..Default::default()
    }.insert(db).await?;

    // 3. INSERT project_identifiers (misma transacción lógica)
    project_identifiers::ActiveModel {
        id:           Set(Uuid::new_v4()),
        identifier:   Set(payload.identifier.to_uppercase()),
        project_id:   Set(project_id),
        workspace_id: Set(workspace.id),
        ..Default::default()
    }.insert(db).await?;

    // 4. Creador → Admin del proyecto (rol 20)
    project_members::ActiveModel {
        id:           Set(Uuid::new_v4()),
        project_id:   Set(project_id),
        member_id:    Set(user.id),
        role:         Set(20),
        workspace_id: Set(workspace.id),
        ..Default::default()
    }.insert(db).await?;

    Ok((StatusCode::CREATED, Json(ProjectResponse::from(project))))
}
```

---

## Roles de proyecto

| Valor | Nombre | Crear issues | Editar issues | Configurar proyecto |
|-------|--------|:---:|:---:|:---:|
| 20 | Admin | ✅ | ✅ | ✅ |
| 15 | Member | ✅ | ✅ | ❌ |
| 10 | Viewer | ❌ | ❌ | ❌ |
| 5 | Guest | ❌ | ❌ | ❌ |

> [!NOTE] Fallback workspace Admin
> Si el usuario no es miembro explícito del proyecto pero tiene rol 20 en el workspace,
> `ProjectMemberGuard` le asigna rol efectivo 20. Ver [[impl-extractores-auth]].

---

## DTOs

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name:        String,
    pub identifier:  String,        // 1-12 chars, solo letras y números, se convierte a UPPERCASE
    pub description: Option<String>,
    pub network:     Option<i16>,   // 0=public, 2=secret (default)
    pub emoji:       Option<String>,
    pub icon_prop:   Option<serde_json::Value>,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id:          Uuid,
    pub name:        String,
    pub identifier:  String,
    pub description: String,
    pub network:     i16,
    pub workspace_id: Uuid,
    pub member_role: Option<i16>,   // rol del usuario actual en este proyecto
    pub is_member:   bool,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct StateResponse {
    pub id:          Uuid,
    pub name:        String,
    pub color:       String,        // hex e.g. "#526B9A"
    pub group:       String,        // "backlog"|"unstarted"|"started"|"completed"|"cancelled"
    pub default:     bool,          // estado default del grupo
    pub project_id:  Uuid,
    pub sequence:    f64,           // orden dentro del grupo
}
```

---

## Puntos críticos

> [!WARNING] 6 puntos críticos

1. **`identifier` único por workspace** — verificar antes de INSERT; retornar 409 si ya existe.
2. **Transacción project + identifier** — si el INSERT de `project_identifiers` falla, hacer rollback del proyecto. Usar transacción DB explícita.
3. **ProjectMemberGuard con fallback** — si el usuario no es miembro del proyecto pero es workspace Admin (rol 20), se le asigna rol efectivo 20. Sin esta lógica, un workspace Admin no puede acceder a ningún proyecto.
4. **`network` valores** — `0` = público (acceso sin auth via Space API), `2` = secreto (solo miembros). Los deploy boards solo funcionan con `network = 0`.
5. **No último admin** — `DELETE /members/{pk}/` o `POST /members/leave/` deben verificar que queda ≥ 1 Admin. Si es el último → 400.
6. **`estimate_point_id` — tipo en URL** — verificar en entidades SeaORM si el PK es UUID o int. Django no aplica tipo en el path (`<estimate_point_id>` sin `uuid:`). Puede ser un int según la migración.

---

## Entidades SeaORM involucradas

| Entidad | Tabla |
|---------|-------|
| `projects.rs` | `projects` |
| `project_members.rs` | `project_members` |
| `project_member_invites.rs` | `project_member_invites` |
| `project_identifiers.rs` | `project_identifiers` |
| `project_user_properties.rs` | `project_user_properties` |
| `project_deploy_boards.rs` | `project_deploy_boards` |
| `states.rs` | `states` |
| `labels.rs` | `labels` (ver [[dominio-issues]] para endpoints de issue-labels) |
| `estimates.rs` | `estimates` |
| `estimate_points.rs` | `estimate_points` |

---

## Plan de implementación

```
Fase 2:
  [ ] src/routes/projects.rs       — CRUD + list_detail + members CRUD + me
  [ ] src/routes/states.rs         — CRUD + mark-default

Fase 4:
  [ ] src/routes/project_invites.rs  — invitaciones + join por token
  [ ] src/routes/project_archive.rs  — archive/unarchive
  [ ] src/routes/estimates.rs        — CRUD estimates + estimate-points
  [ ] src/routes/deploy_boards.rs    — deploy boards
  [ ] src/routes/project_prefs.rs    — project-views + preferences/member
```

---

## 🔗 Navegar

← [[dominio-workspace-settings]] | [[MOC]] | → [[dominio-issues]]

**Relacionado:** Issues: [[dominio-issues]] | Cycles: [[dominio-ciclos]] | Modules: [[dominio-modulos]] | Guards: [[impl-extractores-auth]]
