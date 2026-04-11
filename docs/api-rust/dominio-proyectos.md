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
            ├─ labels                   (labels propios + heredados del workspace)
            ├─ estimates                (sistema de puntos)
            ├─ deploy_boards            (tableros públicos)
            └─ project_user_properties  (preferencias por usuario)
```

---

## Endpoints a implementar

### CRUD de proyecto

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/` | `WorkspaceMemberGuard (≥5)` | 2 |
| `POST` | `/api/workspaces/{slug}/projects/` | `WorkspaceMemberGuard (≥15)` | 2 |
| `GET` | `/api/workspaces/{slug}/projects/{pk}/` | `ProjectMemberGuard (≥5)` | 2 |
| `PATCH` | `/api/workspaces/{slug}/projects/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |
| `DELETE` | `/api/workspaces/{slug}/projects/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |
| `GET` | `/api/workspaces/{slug}/projects/details/` | `WorkspaceMemberGuard (≥5)` | 2 |

### Identificadores

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/project-identifiers/` | `WorkspaceMemberGuard (≥5)` | 2 |
| `DELETE` | `/api/workspaces/{slug}/project-identifiers/` | `WorkspaceMemberGuard (≥20)` | 4 |

### Miembros

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/{id}/members/` | `ProjectMemberGuard (≥5)` | 2 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/members/` | `ProjectMemberGuard (≥20)` | 2 |
| `GET/PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/members/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/members/leave/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET` | `/api/workspaces/{slug}/projects/{id}/project-members/me/` | `ProjectMemberGuard (≥5)` | 2 |

### Invitaciones

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/invitations/` | `ProjectMemberGuard (≥20)` | 4 |
| `GET/DELETE` | `/api/workspaces/{slug}/projects/{id}/invitations/{pk}/` | `ProjectMemberGuard (≥20)` | 4 |
| `GET` | `/api/users/me/workspaces/{slug}/projects/invitations/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/join/{pk}/` | Sin auth (token en URL) | 4 |
| `GET` | `/api/users/me/workspaces/{slug}/project-roles/` | `WorkspaceMemberGuard (≥5)` | 4 |

### Favoritos

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/user-favorite-projects/` | `WorkspaceMemberGuard (≥5)` | 4 |
| `DELETE` | `/api/workspaces/{slug}/user-favorite-projects/{project_id}/` | `WorkspaceMemberGuard (≥5)` | 4 |

### Archivar

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `POST/DELETE` | `/api/workspaces/{slug}/projects/{id}/archive/` | `ProjectMemberGuard (≥20)` | 4 |

### Deploy boards

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/project-deploy-boards/` | `ProjectMemberGuard (≥20)` | 4 |
| `GET/PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/project-deploy-boards/{pk}/` | `ProjectMemberGuard (≥20)` | 4 |

### Views de proyecto (preferencias de usuario)

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/{id}/project-views/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET/PATCH` | `/api/workspaces/{slug}/projects/{id}/preferences/member/{member_id}/` | `ProjectMemberGuard (≥5)` | 4 |

---

## Handler — `POST /projects/`

```rust
pub async fn create_project(
    State(state): State<AppState>,
    WorkspaceMemberGuard { user, workspace, workspace_member, .. }: WorkspaceMemberGuard,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    let db = &state.db;

    // 1. Verificar que el identifier no exista en este workspace
    let identifier_exists = project_identifiers::Entity::find()
        .filter(project_identifiers::Column::WorkspaceId.eq(workspace.id))
        .filter(project_identifiers::Column::Identifier.eq(&payload.identifier))
        .count(db).await.map_err(AppError::Database)? > 0;
    if identifier_exists {
        return Err(AppError::Conflict("identifier already in use".into()));
    }

    // 2. Crear proyecto
    let project_id = Uuid::new_v4();
    let project = projects::ActiveModel {
        id:           Set(project_id),
        name:         Set(payload.name),
        identifier:   Set(payload.identifier.to_uppercase()),
        description:  Set(payload.description.unwrap_or_default()),
        network:      Set(payload.network.unwrap_or(2)), // 2 = secret, 0 = public
        workspace_id: Set(workspace.id),
        created_by_id: Set(Some(user.id)),
        ..Default::default()
    }.insert(db).await.map_err(AppError::Database)?;

    // 3. Registrar identifier
    project_identifiers::ActiveModel {
        id:           Set(Uuid::new_v4()),
        identifier:   Set(payload.identifier.to_uppercase()),
        project_id:   Set(project_id),
        workspace_id: Set(workspace.id),
        ..Default::default()
    }.insert(db).await.map_err(AppError::Database)?;

    // 4. Agregar al creador como Admin del proyecto
    project_members::ActiveModel {
        id:           Set(Uuid::new_v4()),
        project_id:   Set(project_id),
        member_id:    Set(user.id),
        role:         Set(20), // 20 = Admin
        workspace_id: Set(workspace.id),
        ..Default::default()
    }.insert(db).await.map_err(AppError::Database)?;

    // 5. ProjectIdentifier ya creado — emitir proyecto
    Ok((StatusCode::CREATED, Json(ProjectResponse::from(project))))
}
```

---

## Roles de proyecto

| Valor | Nombre | Puede crear issues | Puede editar issues | Puede configurar |
|-------|--------|:-:|:-:|:-:|
| 20 | Admin | ✅ | ✅ | ✅ |
| 15 | Member | ✅ | ✅ | ❌ |
| 10 | Viewer | ❌ | ❌ | ❌ |
| 5 | Guest | ❌ | ❌ | ❌ |

> [!NOTE] Herencia workspace → project
> Un workspace Admin (rol 20) tiene acceso a todos los proyectos aunque no sea miembro explícito. El extractor `ProjectMemberGuard` debe verificar primero membresía de proyecto, y si no existe, buscar el rol del workspace.

---

## States — estados del proyecto

Los estados son por proyecto. Hay 5 grupos fijos, con múltiples estados posibles por grupo.

| Grupo | Descripción | Color típico |
|-------|-------------|--------------|
| `backlog` | No iniciado (default al crear) | `#526B9A` |
| `unstarted` | Pendiente de iniciar | `#3F76FF` |
| `started` | En progreso | `#F4C430` |
| `completed` | Terminado | `#17B26A` |
| `cancelled` | Cancelado | `#EF4444` |

Endpoints de states:

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/states/` | `ProjectMemberGuard (≥5)` | 2 |
| `GET/PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/states/{pk}/` | `ProjectMemberGuard (≥20)` | 2 |

> [!WARNING] No se puede eliminar un state con issues asociados
> Verificar `issues WHERE state_id = pk AND deleted_at IS NULL` antes del DELETE → 400 si hay issues.

---

## Estimates — puntos de estimación

Cada proyecto puede tener un sistema de estimación personalizado (Fibonacci, T-shirt sizes, etc.).

```rust
// Estructura: Estimate tiene N EstimatePoints (valores posibles)
// EstimatePoint: { value: "1", key: 0 }
// Al asignar estimate_point_id a un issue, se guarda el ID del punto
```

Endpoints:

| Método | URL | Guard | Fase |
|--------|-----|-------|------|
| `GET` | `/api/workspaces/{slug}/projects/{id}/estimates/` | `ProjectMemberGuard (≥5)` | 4 |
| `GET` | `/api/workspaces/{slug}/projects/{id}/project-estimates/` | `ProjectMemberGuard (≥5)` | 4 |
| `POST` | `/api/workspaces/{slug}/projects/{id}/estimates/` | `ProjectMemberGuard (≥20)` | 4 |
| `GET/PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/estimates/{estimate_id}/` | `ProjectMemberGuard (≥20)` | 4 |
| `GET/POST` | `/api/workspaces/{slug}/projects/{id}/estimates/{id}/estimate-points/` | `ProjectMemberGuard (≥20)` | 4 |
| `PATCH/DELETE` | `/api/workspaces/{slug}/projects/{id}/estimates/{id}/estimate-points/{pk}/` | `ProjectMemberGuard (≥20)` | 4 |

---

## DTOs

```rust
#[derive(Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name:         String,
    pub identifier:   String,       // 1-12 chars, solo letras/números
    pub description:  Option<String>,
    pub network:      Option<i16>,  // 0=public, 2=secret
    pub emoji:        Option<String>,
    pub icon_prop:    Option<serde_json::Value>,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id:           Uuid,
    pub name:         String,
    pub identifier:   String,
    pub description:  String,
    pub network:      i16,
    pub workspace_id: Uuid,
    pub member_role:  Option<i16>,  // rol del usuario actual en este proyecto
    pub is_member:    bool,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}
```

---

## Puntos críticos

> [!WARNING] 5 puntos críticos

1. **`identifier` único por workspace** — verificar antes de INSERT y retornar 409 Conflict si ya existe.
2. **Al crear proyecto** — crear `ProjectIdentifier` en la misma transacción. Si falla la transacción, el identifier queda huérfano.
3. **ProjectMemberGuard con fallback a workspace** — si el usuario no es miembro de proyecto pero sí Admin del workspace, se le asigna rol efectivo 20.
4. **`network` valores** — `0` = publicado (acceso sin login), `2` = secreto (solo miembros). Los deploy boards dependen de `network = 0`.
5. **No último admin** — al `DELETE /members/{pk}/`, verificar que quedará al menos 1 Admin. Si es el último → 400.

---

## Entidades SeaORM involucradas ✅

| Entidad | Tabla |
|---------|-------|
| `projects.rs` | `projects` |
| `project_members.rs` | `project_members` |
| `project_member_invites.rs` | `project_member_invites` |
| `project_identifiers.rs` | `project_identifiers` |
| `project_user_properties.rs` | `project_user_properties` |
| `project_deploy_boards.rs` | `project_deploy_boards` |
| `states.rs` | `states` |
| `labels.rs` | `labels` |
| `estimates.rs` | `estimates` |
| `estimate_points.rs` | `estimate_points` |

---

## 🔗 Navegar

← [[dominio-workspace-settings]] | [[MOC]] | → [[dominio-issues]]

**Relacionado:** Issues: [[dominio-issues]] | Cycles: [[dominio-ciclos]] | Modules: [[dominio-modulos]] | Guards: [[impl-extractores-auth]]
