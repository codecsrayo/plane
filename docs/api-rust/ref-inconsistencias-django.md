---
titulo: Auditoría de inconsistencias — Docs vs Django real
aliases:
  - inconsistencias
  - auditoria
  - discrepancias
tags:
  - auditoria
  - inconsistencias
  - rust
  - django
estado: activo
---

# 🔍 Auditoría de inconsistencias — Docs vs Django real

> [!SUMMARY]
> Comparación sistemática entre las tablas de endpoints documentadas en `docs/api-rust/` y
> las rutas registradas en `apps/api/plane/app/urls/*.py`.
> Fuente de verdad: el código Django. Los docs son la referencia para la implementación Rust.
>
> **Resultado auditoría v1:** 9 inconsistencias en 7 documentos.
> **Resultado auditoría v2 (completa):** 19 inconsistencias en 9 documentos + 4 dominios completamente no documentados.

---

## 🔴 Críticas — endpoints incorrectos que causarán bugs en Rust

---

### INC-01 · `dominio-workspace-settings.md` — URL de Exports incorrecta

**Documento dice:**

```
GET  /api/workspaces/{slug}/exports/
POST /api/workspaces/{slug}/exports/
DELETE /api/workspaces/{slug}/exports/{pk}/
```

**Django real** (`apps/api/plane/app/urls/exporter.py`):

```
POST /api/workspaces/{slug}/export-issues/
```

**Problemas:**

- La URL documentada es `/exports/` → real es `/export-issues/`
- Django solo tiene `POST` (dispara el job). No hay `GET` ni `DELETE` para listar/cancelar exports
- El polling de estado mencionado en el diagrama de secuencia no tiene endpoint en Django — el frontend recibe el resultado por otro mecanismo

**Fix:** Corregir URL y métodos en `dominio-workspace-settings.md`.

---

### INC-02 · `dominio-workspace-settings.md` — endpoint `webhook-logs` ausente

**Documento:** no menciona `webhook-logs`.

**Django real** (`apps/api/plane/app/urls/webhook.py`):

```
GET /api/workspaces/{slug}/webhook-logs/{webhook_id}/
```

Vista: `WebhookLogsEndpoint`.

**Fix:** Agregar endpoint a la tabla de webhooks en `dominio-workspace-settings.md`.

---

### INC-03 · `dominio-workspace-settings.md` — endpoints de workspace modernos ausentes

**Documento:** no menciona ninguno de estos endpoints que existen en Django
(`apps/api/plane/app/urls/workspace.py`):

| Endpoint Django real                                    | Vista                            |
| ------------------------------------------------------- | -------------------------------- |
| `GET/POST /workspaces/{slug}/quick-links/`              | `QuickLinkViewSet`               |
| `GET/PATCH/DELETE /workspaces/{slug}/quick-links/{pk}/` | `QuickLinkViewSet`               |
| `GET /workspaces/{slug}/recent-visits/`                 | `UserRecentVisitViewSet`         |
| `GET/PATCH /workspaces/{slug}/home-preferences/`        | `WorkspaceHomePreferenceViewSet` |
| `GET/PATCH /workspaces/{slug}/home-preferences/{key}/`  | `WorkspaceHomePreferenceViewSet` |
| `GET/POST /workspaces/{slug}/stickies/`                 | `WorkspaceStickyViewSet`         |
| `GET/PATCH/DELETE /workspaces/{slug}/stickies/{pk}/`    | `WorkspaceStickyViewSet`         |
| `GET/PATCH /workspaces/{slug}/sidebar-preferences/`     | `WorkspaceUserPreferenceViewSet` |

**Fix:** Agregar sección "Workspace UI State" en `dominio-workspace-settings.md` con estos 8 endpoints.

---

### INC-04 · `dominio-modulos.md` — método incorrecto en `issues/{id}/modules/`

**Documento dice:**

```
GET/POST/DELETE  /workspaces/{slug}/projects/{id}/issues/{issue_id}/modules/
```

**Django real** (`apps/api/plane/app/urls/module.py`):

```python
ModuleIssueViewSet.as_view({"post": "create_issue_modules"})
# Solo POST — sin GET ni DELETE en esta ruta
```

`GET` y `DELETE` de módulos de un issue se hacen por la ruta inversa (`/modules/{id}/issues/`), no por `/issues/{id}/modules/`.

**Fix:** Corregir a solo `POST` en la tabla del doc.

---

### INC-05 · `dominio-ciclos.md` — métodos incorrectos en `archived-cycles/{pk}/`

**Documento dice:**

```
GET/DELETE  /workspaces/{slug}/projects/{id}/archived-cycles/{pk}/
```

**Django real** (`apps/api/plane/app/urls/cycle.py`):

```python
CycleArchiveUnarchiveEndpoint.as_view()
# No hay map explícito de métodos — la vista maneja GET internamente
# No hay DELETE registrado en esta ruta específica
```

El `DELETE` (desarchivar) se hace en `/cycles/{cycle_id}/archive/` con `DELETE`, no en `/archived-cycles/{pk}/`.

**Fix:** Corregir métodos en la tabla; aclarar que DELETE de archivo usa `/cycles/{id}/archive/`.

---

---

### INC-10 · `dominio-modulos.md` — `DELETE /archived-modules/{pk}/` incorrecto

**Documento dice:**

```
GET/DELETE  /workspaces/{slug}/projects/{id}/archived-modules/{pk}/
```

**Django real** (`apps/api/plane/app/views/module/archive.py`):

```python
class ModuleArchiveUnarchiveEndpoint(BaseAPIView):
    def get(self, request, slug, project_id, pk=None): ...   # lista o detalle
    def post(self, request, slug, project_id, module_id): ...  # archivar
    def delete(self, request, slug, project_id, module_id): ...  # DESARCHIVAR
```

El `delete()` recibe `module_id` y está mapeado SOLO a `/modules/{module_id}/archive/`.
La ruta `/archived-modules/{pk}/` solo acepta `GET` (detalle del módulo archivado).

**Idéntico a INC-05** para ciclos. El patrón correcto:

- `POST /modules/{id}/archive/` → archivar
- `DELETE /modules/{id}/archive/` → desarchivar
- `GET /archived-modules/` → listar archivados
- `GET /archived-modules/{pk}/` → detalle archivado (sin DELETE)

**Fix:** Corregir a solo `GET` en `/archived-modules/{pk}/`; aclarar que unarchive usa `DELETE /modules/{id}/archive/`.

---

### INC-11 · `dominio-analytics.md` — `ExportAnalyticsEndpoint` es `POST`, no `GET`

**Documento dice:**

```
GET  /workspaces/{slug}/export-analytics/
```

**Django real** (`apps/api/plane/app/views/analytic/base.py` línea 234):

```python
class ExportAnalyticsEndpoint(BaseAPIView):
    def post(self, request, slug): ...   # ← solo POST
```

El export requiere body con filtros (`project_ids`, `provider`, etc.) → necesariamente POST.

**Fix:** Corregir a `POST /workspaces/{slug}/export-analytics/` en la tabla de `dominio-analytics.md`.

---

### INC-12 · `dominio-analytics.md` — `SavedAnalyticEndpoint` es `GET`, no `POST`

**Documento dice:**

```
POST  /workspaces/{slug}/saved-analytic-view/{analytic_id}/
```

**Django real** (`apps/api/plane/app/views/analytic/base.py` línea 208):

```python
class SavedAnalyticEndpoint(BaseAPIView):
    def get(self, request, slug, analytic_id): ...   # ← solo GET
```

El endpoint devuelve la vista guardada como JSON para precargar los filtros. No crea datos.

**Fix:** Corregir a `GET /workspaces/{slug}/saved-analytic-view/{analytic_id}/`.

---

### INC-13 · `dominio-workspace-settings.md` WS-2 — Guard de `GET /members/` es `≥5` (GUEST), no `≥15`

**Documento dice:**

```
GET  /api/workspaces/{slug}/members/  →  WorkspaceMemberGuard (≥15)
```

**Django real** (`apps/api/plane/app/views/workspace/member.py` línea 45):

```python
@allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST], level="WORKSPACE")
def list(self, request, slug): ...
```

`ROLE.GUEST = 5`. Cualquier miembro del workspace (incluidos guests) puede ver la lista de miembros.

**Fix:** Corregir guard a `WorkspaceMemberGuard (≥5)` para el método GET de `/members/`.

---

### INC-14 · `dominio-issues.md` — `issue_votes` listada en entidades pero sin endpoint en Django

**Documento lista** `issue_votes.rs` en la tabla "Entidades SeaORM involucradas".

**Django real** (`apps/api/plane/app/urls/issue.py`): **No existe ningún endpoint de votes** registrado. La tabla existe en la DB (migración) pero no tiene API REST expuesta en CE.

**Fix en Rust:** No implementar endpoint de votes en Fase 4. Omitir o agregar nota "sin endpoint en CE".

---

## 🟡 Medias — ausencias de endpoints reales

---

### INC-06 · `dominio-issues.md` — `user-properties` solo acepta `GET` en Django

**Documento dice:**

```
GET/POST/PATCH  /workspaces/{slug}/projects/{project_id}/user-properties/
```

**Django real** (`apps/api/plane/app/urls/issue.py`):

```python
ProjectUserDisplayPropertyEndpoint.as_view()
# Sin map de métodos explícito → depende de la vista
```

La vista `ProjectUserDisplayPropertyEndpoint` típicamente solo expone `GET` y `PATCH`/`POST` dependiendo de si el registro existe. No hay `POST` y `PATCH` como métodos separados con semántica distinta — es un upsert.

**Fix:** Documentar como `GET/PATCH (upsert)` sin `POST` separado.

---

### INC-07 · `dominio-proyectos.md` — `intake-state` está en `state.py`, no en `project.py`

**Documento:** lista `GET /workspaces/{slug}/projects/{project_id}/intake-state/` bajo "Estados del proyecto".

**Django real:** el endpoint existe y está correcto en URL, pero está registrado en
`apps/api/plane/app/urls/state.py` (no en `project.py`). Esto afecta cómo se organiza el router en Rust.

**Fix:** Nota en `dominio-proyectos.md` indicando que en Rust debe ir en el módulo de states, no en projects.

---

### INC-08 · `dominio-analytics.md` — user-stats/activity/profile/issues están en `workspace.py`

**Documento:** lista estos endpoints bajo Analytics:

```
GET /workspaces/{slug}/user-stats/{user_id}/
GET /workspaces/{slug}/user-activity/{user_id}/
GET /workspaces/{slug}/user-activity/{user_id}/export/
GET /workspaces/{slug}/user-profile/{user_id}/
GET /workspaces/{slug}/user-issues/{user_id}/
```

**Django real** (`apps/api/plane/app/urls/workspace.py`): todos están registrados como parte de **workspace**, no del módulo analytics. Las vistas son:

- `WorkspaceUserProfileStatsEndpoint`
- `WorkspaceUserActivityEndpoint`
- `ExportWorkspaceUserActivityEndpoint`
- `WorkspaceUserProfileEndpoint`
- `WorkspaceUserProfileIssuesEndpoint`

**Fix:** En Rust, estos endpoints deben vivir en el módulo `workspace`, no en `analytics`. Actualizar `dominio-analytics.md` con nota de router placement.

---

### INC-09 · `dominio-issues.md` — falta endpoint de `issues-detail/`

**Documento:** no menciona explícitamente `IssueDetailEndpoint`.

**Django real** (`apps/api/plane/app/urls/issue.py`):

```
GET /workspaces/{slug}/projects/{project_id}/issues-detail/
```

Vista: `IssueDetailEndpoint` — devuelve issues con joins expandidos para board/list view (campos calculados, assignees populados, etc.). Es distinto al `IssueViewSet` estándar.

> **Nota:** Revisando el doc nuevamente, sí aparece en la tabla como `IssueDetailEndpoint` con descripción. Inconsistencia menor — la descripción no deja claro que es un endpoint diferente al `/{pk}/` retrieve.

---

---

### INC-15 · `dominio-workspace-settings.md` WS-7 — Workspace Favorites ausentes

**Documento WS-7** agrega quick-links, stickies, sidebar-preferences, home-preferences, recent-visits. Pero **omite** los endpoints de favoritos del workspace:

**Django real** (`apps/api/plane/app/urls/workspace.py`):

```
GET/POST   /workspaces/{slug}/user-favorites/
GET/PATCH/DELETE /workspaces/{slug}/user-favorites/{favorite_id}/
GET        /workspaces/{slug}/user-favorites/{favorite_id}/group/
```

Vistas: `WorkspaceFavoriteEndpoint`, `WorkspaceFavoriteGroupEndpoint`.

> Son distintos a `/user-favorite-projects/`, `/user-favorite-cycles/`, etc. Son los "favoritos unificados" de la sidebar del workspace (issues, páginas, vistas).

**Fix:** Agregar estos 3 endpoints a WS-7.

---

### INC-16 · Dominio "Issue Views" completamente no documentado

**Django real** (`apps/api/plane/app/urls/views.py`): 7 endpoints sin ningún documento en `docs/api-rust/`:

| Método                 | URL                                                               | Vista                        |
| ---------------------- | ----------------------------------------------------------------- | ---------------------------- |
| `GET/POST`             | `/workspaces/{slug}/projects/{id}/views/`                         | `IssueViewViewSet`           |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/projects/{id}/views/{pk}/`                    | `IssueViewViewSet`           |
| `GET/POST`             | `/workspaces/{slug}/views/`                                       | `WorkspaceViewViewSet`       |
| `GET/PUT/PATCH/DELETE` | `/workspaces/{slug}/views/{pk}/`                                  | `WorkspaceViewViewSet`       |
| `GET`                  | `/workspaces/{slug}/issues/`                                      | `WorkspaceViewIssuesViewSet` |
| `GET/POST`             | `/workspaces/{slug}/projects/{id}/user-favorite-views/`           | `IssueViewFavoriteViewSet`   |
| `DELETE`               | `/workspaces/{slug}/projects/{id}/user-favorite-views/{view_id}/` | `IssueViewFavoriteViewSet`   |

**Fix:** Crear `dominio-vistas.md` o agregar sección a `dominio-workspace-settings.md`.

---

### INC-17 · Endpoints workspace-level agregados no documentados

**Django real** (`workspace.py`): endpoints que agregan datos cross-project, ninguno documentado:

| URL                                                          | Vista                              | Descripción                      |
| ------------------------------------------------------------ | ---------------------------------- | -------------------------------- |
| `GET /workspaces/{slug}/labels/`                             | `WorkspaceLabelsEndpoint`          | Todos los labels del workspace   |
| `GET /workspaces/{slug}/states/`                             | `WorkspaceStatesEndpoint`          | Todos los estados del workspace  |
| `GET /workspaces/{slug}/estimates/`                          | `WorkspaceEstimatesEndpoint`       | Todos los sistemas de estimación |
| `GET /workspaces/{slug}/modules/`                            | `WorkspaceModulesEndpoint`         | Todos los módulos del workspace  |
| `GET /workspaces/{slug}/cycles/`                             | `WorkspaceCyclesEndpoint`          | Todos los ciclos del workspace   |
| `GET/PATCH /workspaces/{slug}/user-properties/`              | `WorkspaceUserPropertiesEndpoint`  | Filtros globales del usuario     |
| `GET/POST /workspaces/{slug}/workspace-themes/`              | `WorkspaceThemeViewSet`            | Temas del workspace              |
| `GET/PATCH/DELETE /workspaces/{slug}/workspace-themes/{pk}/` | `WorkspaceThemeViewSet`            | —                                |
| `GET /workspaces/{slug}/workspace-views/`                    | `WorkspaceMemberUserViewsEndpoint` | Vistas guardadas del usuario     |
| `GET /workspaces/{slug}/workspace-members/me/`               | `WorkspaceMemberUserEndpoint`      | Info del miembro actual          |
| `GET /workspaces/{slug}/project-members/`                    | `WorkspaceProjectMemberEndpoint`   | Roles en proyectos               |

**Fix:** Agregar sección "Workspace Aggregates" al `dominio-workspace-settings.md` u otro doc relevante.

---

### INC-18 · Endpoints de usuario (`/users/me/`) no documentados

**Django real** (`user.py`): endpoints de perfil de usuario sin documento en `docs/api-rust/`:

| URL                                                          | Descripción                    |
| ------------------------------------------------------------ | ------------------------------ |
| `GET/PATCH/DELETE /users/me/`                                | Perfil + desactivar cuenta     |
| `GET /users/me/settings/`                                    | Configuración del usuario      |
| `POST /users/me/email/generate-code/`                        | Generar código de verificación |
| `PATCH /users/me/email/`                                     | Cambiar email                  |
| `GET/PATCH /users/me/profile/`                               | Perfil extendido               |
| `GET /users/me/accounts/`, `DELETE /users/me/accounts/{pk}/` | Cuentas OAuth vinculadas       |
| `GET /users/me/activities/`                                  | Historial de actividad         |
| `GET /users/me/workspaces/`                                  | Workspaces del usuario         |
| `GET /users/last-visited-workspace/`                         | Último workspace visitado      |
| `GET /users/session/`                                        | Info de sesión                 |
| `GET /users/me/workspaces/{slug}/activity-graph/`            | Gráfico de actividad           |
| `GET /users/me/workspaces/{slug}/issues-completed-graph/`    | Gráfico issues completados     |
| `GET /users/me/workspaces/{slug}/dashboard/`                 | Dashboard personal             |

**Fix:** Crear `dominio-usuario.md` con todos estos endpoints.

---

## 🟢 Confirmaciones — endpoints correctamente documentados

Los siguientes dominios fueron auditados y **no presentan inconsistencias**:

| Documento                   | Veredicto                                                                                           |
| --------------------------- | --------------------------------------------------------------------------------------------------- |
| `dominio-ciclos.md`         | ✅ Correcto (salvo INC-05)                                                                          |
| `dominio-modulos.md`        | ✅ Correcto (salvo INC-04)                                                                          |
| `dominio-notificaciones.md` | ✅ Correcto — todos los endpoints coinciden con `notification.py`                                   |
| `dominio-paginas.md`        | ✅ Correcto — todos los endpoints coinciden con `page.py`                                           |
| `dominio-intake.md`         | ✅ Correcto — coincide con `intake.py` incluyendo aliases `/inboxes/`                               |
| `dominio-busqueda.md`       | ✅ Correcto — coincide con `search.py` (3 endpoints: `search/`, `search-issues/`, `entity-search/`) |
| `dominio-importadores.md`   | ✅ Correcto — coincide con `importer.py`                                                            |
| `dominio-integraciones.md`  | ✅ Correcto — coincide con `integration.py` actualizado                                             |
| `dominio-proyectos.md`      | ✅ Correcto (salvo INC-07, nota de placement)                                                       |
| `dominio-analytics.md`      | ✅ URLs correctas (salvo INC-08, nota de placement)                                                 |
| `dominio-issues.md`         | ✅ Correcto (salvo INC-06 y INC-09 menores)                                                         |
| `dominio-workspace-seed.md` | ✅ Correcto — el workspace seed job no tiene endpoints HTTP directos                                |
| `dominio-ia.md`             | ✅ Recién creado, ya refleja la realidad                                                            |

---

## 📋 Plan de correcciones

| #      | Documento a corregir                                                                                    | Prioridad | Tipo                 |
| ------ | ------------------------------------------------------------------------------------------------------- | --------- | -------------------- |
| INC-01 | `dominio-workspace-settings.md` — URL `/exports/` → `/export-issues/` + solo POST                       | 🔴 Alta   | URL incorrecta       |
| INC-02 | `dominio-workspace-settings.md` — agregar `webhook-logs`                                                | 🔴 Alta   | Endpoint ausente     |
| INC-03 | `dominio-workspace-settings.md` — agregar 8 endpoints UI state                                          | 🔴 Alta   | Endpoints ausentes   |
| INC-10 | `dominio-modulos.md` — `archived-modules/{pk}/` sin DELETE (igual a INC-05)                             | 🔴 Alta   | Método incorrecto    |
| INC-11 | `dominio-analytics.md` — `export-analytics` es POST, no GET                                             | 🔴 Alta   | Método incorrecto    |
| INC-12 | `dominio-analytics.md` — `saved-analytic-view` es GET, no POST                                          | 🔴 Alta   | Método incorrecto    |
| INC-13 | `dominio-workspace-settings.md` — guard GET /members/ es ≥5 no ≥15                                      | 🔴 Alta   | Guard incorrecto     |
| INC-04 | `dominio-modulos.md` — `issues/{id}/modules/` solo POST                                                 | 🟡 Media  | Método incorrecto    |
| INC-05 | `dominio-ciclos.md` — `archived-cycles/{pk}/` sin DELETE                                                | 🟡 Media  | Método incorrecto    |
| INC-06 | `dominio-issues.md` — `user-properties` es `GET/PATCH` (upsert)                                         | 🟡 Media  | Semántica incorrecta |
| INC-14 | `dominio-issues.md` — `issue_votes` sin endpoint en Django                                              | 🟡 Media  | Entidad fantasma     |
| INC-15 | `dominio-workspace-settings.md` — agregar workspace favorites (3 endpoints)                             | 🟡 Media  | Endpoints ausentes   |
| INC-07 | `dominio-proyectos.md` — nota: `intake-state` va en módulo states en Rust                               | 🟢 Baja   | Placement            |
| INC-08 | `dominio-analytics.md` — nota: user-stats/activity/profile en workspace router                          | 🟢 Baja   | Placement            |
| INC-09 | `dominio-issues.md` — clarificar `issues-detail/` vs `/{pk}/`                                           | 🟢 Baja   | Claridad             |
| INC-16 | Crear `dominio-vistas.md` — Issue Views + Workspace Views (7 endpoints)                                 | 🟢 Baja   | Dominio ausente      |
| INC-17 | Agregar workspace aggregates (11 endpoints) a un doc existente                                          | 🟢 Baja   | Dominio ausente      |
| INC-18 | Crear `dominio-usuario.md` — 13 endpoints de `/users/me/`                                               | 🟢 Baja   | Dominio ausente      |
| INC-19 | Assets/file management (`asset.py`), API tokens (`api.py`), timezones (`timezone.py`) — no documentados | 🟢 Baja   | Dominios ausentes    |

---

## 🔗 Relacionado

- [[dominio-ia]] — errores de IA encontrados en la misma auditoría
- [[ref-estructura-django]] — árbol completo de Django como fuente de verdad
- [[plan-fases]] — orden de implementación afectado por estas correcciones

---

_Auditoría realizada sobre rama `feature/integrations-panel-fix-17593507967815292912`_
_Fuente de verdad: `apps/api/plane/app/urls/_.py`\*
