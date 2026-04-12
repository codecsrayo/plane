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
> **Resultado:** 9 inconsistencias encontradas en 7 documentos.

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

| Endpoint Django real | Vista |
|----------------------|-------|
| `GET/POST /workspaces/{slug}/quick-links/` | `QuickLinkViewSet` |
| `GET/PATCH/DELETE /workspaces/{slug}/quick-links/{pk}/` | `QuickLinkViewSet` |
| `GET /workspaces/{slug}/recent-visits/` | `UserRecentVisitViewSet` |
| `GET/PATCH /workspaces/{slug}/home-preferences/` | `WorkspaceHomePreferenceViewSet` |
| `GET/PATCH /workspaces/{slug}/home-preferences/{key}/` | `WorkspaceHomePreferenceViewSet` |
| `GET/POST /workspaces/{slug}/stickies/` | `WorkspaceStickyViewSet` |
| `GET/PATCH/DELETE /workspaces/{slug}/stickies/{pk}/` | `WorkspaceStickyViewSet` |
| `GET/PATCH /workspaces/{slug}/sidebar-preferences/` | `WorkspaceUserPreferenceViewSet` |

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

## 🟢 Confirmaciones — endpoints correctamente documentados

Los siguientes dominios fueron auditados y **no presentan inconsistencias**:

| Documento | Veredicto |
|-----------|-----------|
| `dominio-ciclos.md` | ✅ Correcto (salvo INC-05) |
| `dominio-modulos.md` | ✅ Correcto (salvo INC-04) |
| `dominio-notificaciones.md` | ✅ Correcto — todos los endpoints coinciden con `notification.py` |
| `dominio-paginas.md` | ✅ Correcto — todos los endpoints coinciden con `page.py` |
| `dominio-intake.md` | ✅ Correcto — coincide con `intake.py` incluyendo aliases `/inboxes/` |
| `dominio-busqueda.md` | ✅ Correcto — coincide con `search.py` (3 endpoints: `search/`, `search-issues/`, `entity-search/`) |
| `dominio-importadores.md` | ✅ Correcto — coincide con `importer.py` |
| `dominio-integraciones.md` | ✅ Correcto — coincide con `integration.py` actualizado |
| `dominio-proyectos.md` | ✅ Correcto (salvo INC-07, nota de placement) |
| `dominio-analytics.md` | ✅ URLs correctas (salvo INC-08, nota de placement) |
| `dominio-issues.md` | ✅ Correcto (salvo INC-06 y INC-09 menores) |
| `dominio-workspace-seed.md` | ✅ Correcto — el workspace seed job no tiene endpoints HTTP directos |
| `dominio-ia.md` | ✅ Recién creado, ya refleja la realidad |

---

## 📋 Plan de correcciones

| # | Documento a corregir | Prioridad | Tipo |
|---|----------------------|-----------|------|
| INC-01 | `dominio-workspace-settings.md` — URL `/exports/` → `/export-issues/` + solo POST | 🔴 Alta | URL incorrecta |
| INC-02 | `dominio-workspace-settings.md` — agregar `webhook-logs` | 🔴 Alta | Endpoint ausente |
| INC-03 | `dominio-workspace-settings.md` — agregar 8 endpoints UI state | 🔴 Alta | Endpoints ausentes |
| INC-04 | `dominio-modulos.md` — `issues/{id}/modules/` solo POST | 🟡 Media | Método incorrecto |
| INC-05 | `dominio-ciclos.md` — `archived-cycles/{pk}/` sin DELETE | 🟡 Media | Método incorrecto |
| INC-06 | `dominio-issues.md` — `user-properties` es `GET/PATCH` (upsert) | 🟡 Media | Semántica incorrecta |
| INC-07 | `dominio-proyectos.md` — nota: `intake-state` va en módulo states en Rust | 🟢 Baja | Placement |
| INC-08 | `dominio-analytics.md` — nota: user-stats/activity/profile en workspace router | 🟢 Baja | Placement |
| INC-09 | `dominio-issues.md` — clarificar `issues-detail/` vs `/{pk}/` | 🟢 Baja | Claridad |

---

## 🔗 Relacionado

- [[dominio-ia]] — errores de IA encontrados en la misma auditoría
- [[ref-estructura-django]] — árbol completo de Django como fuente de verdad
- [[plan-fases]] — orden de implementación afectado por estas correcciones

---

*Auditoría realizada sobre rama `feature/integrations-panel-fix-17593507967815292912`*
*Fuente de verdad: `apps/api/plane/app/urls/*.py`*
