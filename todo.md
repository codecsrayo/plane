# Plane CE — Feature Tracker

## Estado general de integraciones

Rama activa: `feature/integrations-panel-fix-17593507967815292912`

### Lo que ya funciona (no tocar)

| Qué | Dónde |
|---|---|
| God Mode: formulario GitHub / GitLab / Slack con toggles | `apps/admin/app/(all)/(dashboard)/integrations/` |
| Variables de instancia en DB para las 3 integraciones | `apps/api/plane/utils/instance_config_variables/core.py` |
| Backend: `WorkspaceIntegrationViewSet` con `provider_install`, `provider_destroy` | `apps/api/plane/app/views/integration/base.py` |
| URLs de integración registradas en el router | `apps/api/plane/app/urls/integration.py` |
| GitHub App popup + callback completo (`/auth/github/callback`, `/auth/github/setup`) | `apps/web/app/(all)/auth/github/` |
| Hook `useIntegrationPopup` para abrir popups OAuth (GitHub, GitLab, Slack) | `apps/web/core/hooks/use-integration-popup.tsx` |
| Servicio `AppInstallationService.addInstallationApp` | `apps/web/core/services/app_installation.service.ts` |
| Card grid en `/settings/integrations` (3 columnas, responsive) | `apps/web/app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/page.tsx` |
| `SingleIntegrationCard` como tarjeta vertical con logo / descripción / botón | `apps/web/core/components/integration/single-integration-card.tsx` |
| Migration GitHub + GitLab en DB | `apps/api/plane/db/migrations/0122_add_github_gitlab_integrations.py` |
| Migration Slack en DB | `apps/api/plane/db/migrations/0123_add_slack_integration.py` |
| Webhook receivers (skeleton) para GitHub y GitLab | `apps/api/plane/app/views/external/sync.py` |

### Bugs corregidos en esta rama

| Bug | Commit |
|---|---|
| `GitHub OAuth button visible sin config` — filtro `!== false` mostraba el botón cuando `enabled=undefined` (config no cargada o provider no configurado) | ✅ corregido (commit `992ae8751`) |
| `&amp;` literal en URL OAuth de Slack | `7f6abcf38` |
| Dead code: GitHub installation access token sin JWT (siempre fallaba) | `7f6abcf38` |
| `next/navigation` usado en 6 archivos de integración (runtime crash en react-router) | `29adc41ab` |
| Import duplicado de `react-router` en `single-integration-card.tsx` | `29adc41ab` |
| `is_github_enabled` en `SingleIntegrationCard.isEnabled` — mostraba "Disabled by admin" aunque IS_GITHUB_INTEGRATION_ENABLED=1 | ✅ corregido (commit `13e1d6932`) |
| `is_github_enabled` usado para filtrar card en /settings/integrations — usaba el campo OAuth en lugar del campo de integración, card invisible aunque la integración estuviera habilitada | ✅ corregido (commit `8912173e2`) |
| `IS_GITLAB_ENABLED` compartido — el toggle de GitLab integration en /integrations activaba también el OAuth login de GitLab | ✅ corregido (commit `d32862a8e`) |
| `IS_GITHUB_ENABLED` compartido — el toggle de GitHub App integration en /integrations activaba también el OAuth login de GitHub | ✅ corregido (commit `299003c81`) |
| `Combobox Fragment prop passthrough` — `Combobox.Button as={Fragment}` en 5 archivos causaba crash en flujo de creación de proyecto | ✅ corregido (commit `3cb7681f7`) |
| `workspace=workspace` redundante en `get_or_create` de `GithubRepository` — filtro incorrecto causaba duplicados | ✅ corregido |
| **Soft-delete + re-registro 400 Bad Request:** `GithubRepository`/`GithubRepositorySync` usan soft delete; al re-crear tras eliminar, `get_or_create` ignoraba las filas soft-deleted e intentaba INSERT → violación de la OneToOneField unique constraint a nivel DB → 400. Fix: `all_objects` + resurrección del registro existente en lugar de crear uno nuevo | ✅ corregido (`e030f50`) |
| **`GithubRepository` huérfano tras destroy:** `destroy` solo borraba el `GithubRepositorySync` pero no el `GithubRepository` asociado, dejando filas que bloqueaban futuros re-registros | ✅ corregido (`e030f50`) |
| **Issues existentes no importados:** el sync era 100% basado en webhooks → solo recibía issues nuevos; issues ya existentes en GitHub nunca aparecían en Plane. Fix: nuevo task Celery `github_initial_issue_sync_task` disparado automáticamente al crear el sync, pagina todos los issues con GitHub App token respetando los estados configurados | ✅ corregido (`938a9de`) |
| `github_client_id` no expuesto en la API de instancia — UI de conexión personal sin datos | ✅ corregido |
| UI trigger para OAuth personal de GitHub presente en detail page con `user-callback` popup | ✅ corregido |
| `GithubPRStateMapping.github_pr_state` solo tenía 3 choices (open/merged/closed) — frontend enviaba 6 (draft_open, review_requested, ready_for_merge incluidos) → 400 en API | ✅ corregido (`80b7aee`) |
| `prevent_regression` ausente en el modelo `GithubPRStateMapping` — el frontend lo enviaba, Django lo ignoraba silenciosamente | ✅ corregido (`80b7aee`) — migración `0125` |
| `GithubRepositoriesEndpoint` llamaba `/user/repos` con installation token → 401; debe usar `/installation/repositories` | ✅ corregido (`72adb2d`) |
| `getGithubRepositories` servicio retornaba el objeto wrapper `{repositories, total_count, page}` en lugar del array — lista de repos siempre vacía en modal de Issue Sync | ✅ corregido (`72adb2d`) |
| `select-repository.tsx` llamaba URL `/workspace-integrations/{id}/github-repositories/` que no existía en el backend (404 silencioso) — repos no cargaban en Project Issue Sync | ✅ corregido — ruta agregada en `integration.py`, vista actualizada con `wi_id` opcional |
| GitHub App installation token: repos no accesibles no aparecen en el filtro (comportamiento de la API de GitHub) — se agrega enlace "Manage GitHub App access" en el footer del selector cuando aplica | ✅ corregido — `is_installation_token` + `manage_installation_url` en respuesta, UI actualizada |
| Spacing faltante en grid de cards de integración — faltaba `mt-6` en el `<div>` del grid en `/settings/integrations` | ✅ corregido |
| `handleCoverImageChange` no retornaba `asset_url` tras upload — hacía el upload y devolvía `undefined`, `coverImagePayload` quedaba vacío | ✅ corregido |
| `entityIdentifier: ""` al subir cover image en proyecto nuevo — upstream ya usa `res.id` post-creación, fix obsoleto | ✅ upstream ya corregido |
| **`GithubAppCallbackEndpoint` — handshake no se guardaba en BD (silent failure):** cuando `actor=None` (ningún admin activo en el workspace), `update_defaults` omitía `actor`/`api_token` → `update_or_create` lanzaba `IntegrityError` (campos NOT NULL) → el handler de IntegrityError hacía `.filter().update()` con 0 filas → devolvía `success=True` mintiendo → `installation_id` NUNCA se guardaba en BD | ✅ corregido — retorno temprano con `success=False` cuando no hay admin, `actor`/`api_token` siempre incluidos en `update_defaults` |
| **`GithubAppCallbackEndpoint` — `WorkspaceMember` importado lazy dentro del `try` block** — si fallaba el import, `except Exception` lo silenciaba completamente sin log útil | ✅ corregido — movido al top-level import |
| **`GithubAppCallbackEndpoint` — `except IntegrityError` (race condition) usaba `update(**update_defaults)`** que podía incluir `actor`/`api_token` en el UPDATE provocando un override innecesario; ahora solo actualiza `metadata` y `config` | ✅ corregido |
| **`GithubAppCallbackEndpoint` — `except Exception` broad** capturaba `Workspace.DoesNotExist` e `Integration.DoesNotExist` sin distinción; ahora hay un `except (Workspace.DoesNotExist, Integration.DoesNotExist)` específico con log de warning, y el broad `except Exception` queda solo para errores inesperados | ✅ corregido |
| **`api_rust/issues::IssueResponse` — shape mismatch con el frontend:** DTO único emitía `created_by_id`/`updated_by_id`/`estimate_point_id` (Django usa `created_by`/`updated_by`/`estimate_point`); omitía `cycle_id`, `module_ids`, `sub_issues_count`, `attachment_count`, `link_count`, `is_subscribed`, `is_intake`; incluía `workspace_id` y `type_id` que Django no expone. Resultado: panel de detail roto (autores vacíos, counts como NaN, cycle siempre "sin asignar"). Split en `IssueDetailResponse` (GET) + `IssueCreateResponse` (POST) mirrando `IssueDetailSerializer` y la proyección `.values(...)` de `base.py:427-454` respectivamente. | ✅ corregido (`56c7484`) |
| **`api_rust/issues` PATCH devolvía body — divergencia con Django 204 No Content:** `update_issue` retornaba `Json<IssueResponse>`; Django responde `HTTP_204_NO_CONTENT` (`base.py:700`). Frontend resolvía estado por optimistic update sin consumir el body. Fix: retorno a `StatusCode::NO_CONTENT`. | ✅ corregido (`56c7484`) |
| **`api_rust/issues` enrichment stubs → load_enrichment real:** commit 1 dejó `cycle_id`/`module_ids`/counts/`is_subscribed`/`is_intake` en defaults (shape correcto, data stub). Commit 2 reemplaza el helper local `enrich_issues` por el helper compartido `load_enrichment` (`issue_pagination.rs`) y agrega queries específicas para `is_subscribed` (mirror de `Exists(IssueSubscriber...)` en `base.py:566-574`) y `is_intake` (mirror del pattern `status ∈ (-2, 0)` en `base.py:1305-1313`). | ✅ corregido |

---

## Trabajo pendiente — integraciones GitHub / GitLab / Slack

> **Para el siguiente LLM:** leer esta sección de arriba hacia abajo en orden de prioridad.
> Cada tarea incluye archivos relevantes, contexto de diseño y los edge cases conocidos.

---

### TAREA 1 — Página de detalle por integración (la más importante)

**Qué es:** Al hacer click en una card instalada debería navegar a `/settings/integrations/github`
(o gitlab / slack) mostrando la pantalla de la imagen de referencia guardada en `image.png`.

**Estado actual:** No existe ninguna ruta ni componente para esto.

**Archivos a crear / modificar:**

```
apps/web/app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/
  [provider]/
    page.tsx          ← NUEVO: página de detalle por provider
```

```
apps/web/app/routes/core.ts   ← agregar la ruta nueva
```

**Lo que debe renderizar la página:**

1. Header: logo + nombre + botón "Back to integrations"
2. Sección "Connected account" — muestra el usuario de GitHub/GitLab/Slack conectado con botón Disconnect
   - Datos disponibles en `WorkspaceIntegration.metadata` (tiene `installation_id` para GitHub)
   - Para mostrar el avatar/username hay que hacer un GET a la API del provider
3. Sección "Pull Request State Mapping" — ver TAREA 3
4. Sección "Project Issue Sync" — ver TAREA 4

**Referencia visual:** ver `image.png` en la raíz del repo.

**Cómo obtener el `WorkspaceIntegration`:**
```typescript
// GET /api/workspaces/{slug}/workspace-integrations/
// filtrar por integration_detail.provider === "github"
```

**Actualizar `SingleIntegrationCard`:** cuando `isInstalled === true` el botón debe cambiar
de "Uninstall" a "Configure →" y navegar a la detail page. Mantener un botón de "Uninstall"
dentro de la detail page.

---

### TAREA 2 — Conectar cuenta personal GitHub (avatar en detail page)

**Qué es:** En la imagen de referencia se ve el avatar del usuario (codecsrayo) y un botón
"Disconnect" separado del workspace. Es una conexión OAuth personal, distinta de la
instalación del GitHub App en el workspace.

**Estado actual:** No hay modelo `UserSocialAuth` ni endpoint para esto.

**Archivos a crear / modificar:**

Backend:
```
apps/api/plane/db/models/   ← nuevo modelo UserGithubConnection
apps/api/plane/app/views/   ← nuevo endpoint GET/DELETE
apps/api/plane/app/urls/    ← registrar el endpoint
```

Frontend:
```
apps/web/core/services/integrations/github.service.ts
apps/web/app/(all)/auth/github/user-callback/page.tsx  ← callback OAuth personal
                                                          (scope read:user,user:email)
```

**Nota importante:** El callback actual `/auth/github/callback` instala el GitHub App en el
workspace. El flujo personal usa OAuth estándar (no Apps), con scope `read:user,user:email`.
Son dos flujos distintos.

---

### TAREA 3 — Pull Request State Mapping

**Qué es:** Permite mapear estados de Plane (To Do, In Progress, Done…) a estados de PR de
GitHub (open, merged, closed). Se muestra como una lista de filas en la detail page.

**Estado actual:** No hay modelo ni API para esto.

**Modelo Django a crear:**

```python
# apps/api/plane/db/models/integration/github_pr_state.py
class GithubPRStateMapping(BaseModel):
    workspace_integration = models.ForeignKey(
        WorkspaceIntegration, on_delete=models.CASCADE, related_name="pr_state_mappings"
    )
    project = models.ForeignKey("db.Project", on_delete=models.CASCADE)
    state = models.ForeignKey("db.State", on_delete=models.CASCADE)
    # "open" | "merged" | "closed"
    github_pr_state = models.CharField(max_length=20)

    class Meta:
        unique_together = [("workspace_integration", "project", "github_pr_state")]
```

**Migración:** `0124_githubprstatemapping_usergithubconnection.py` ✅ ya existe

> ⚠️ **LO QUE FALTA:** el `GithubPRStateMappingViewSet` y su URL en `integration.py` no fueron implementados. El frontend llama a `/pr-state-mappings/` pero el backend no tiene ese endpoint registrado.

**Endpoints:**
```
GET/POST /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
DELETE   /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{id}/
```

**Frontend:** componente nuevo en `apps/web/core/components/integration/github/pr-state-mapping.tsx`
con un select de State de Plane (usar el store de estados) y otro de PR state (hardcoded: open/merged/closed).

---

### TAREA 4 — Project Issue Sync

**Qué es:** Conecta un proyecto de Plane a un repositorio de GitHub para sincronizar issues.
En la imagen de referencia se ve: `knime_api (codecsrayo/knime_api) ↔ test-0457`.

**Estado actual:** El modelo `GithubRepositorySync` ya existe en el backend pero el frontend
de la detail page no está construido. El importer (importar issues una sola vez) sí existe.

**Modelos existentes a usar:**
```python
GithubRepository          # apps/api/plane/db/models/importer.py
GithubRepositorySync
GithubIssueSync
GithubCommentSync
```

**Endpoints a agregar:**
```
GET  /api/workspaces/{slug}/workspace-integrations/{provider}/repo-syncs/
POST /api/workspaces/{slug}/workspace-integrations/{provider}/repo-syncs/
       body: { repo_id, repo_full_name, project_id }
DELETE /api/workspaces/{slug}/workspace-integrations/{provider}/repo-syncs/{id}/
```

**Para listar repos disponibles:**
```
GET /api/workspaces/{slug}/importers/github/repositories/
```
Este endpoint ya existe (`GithubRepositoriesEndpoint`) pero usa un PAT. Con GitHub App
hay que usar el token de instalación — depende de TAREA 5.

**Frontend:**
- Reutilizar `apps/web/core/components/integration/github/select-repository.tsx` (ya existe)
- Crear `apps/web/core/components/integration/github/project-issue-sync.tsx`

---

### TAREA 5 — GitHub App JWT + Installation Access Tokens

**Qué es:** Para que el backend llame a la API de GitHub en nombre de la instalación necesita
un installation access token. Se obtiene firmando un JWT con la private key del App.

**Estado actual:** El dead code fue removido (commit 7f6abcf38). El `installation_id` se guarda
correctamente en `WorkspaceIntegration.metadata`.

**Implementar helper:**
```python
# apps/api/plane/utils/github_app.py  ← NUEVO archivo
import base64, os, time
import jwt  # PyJWT
import requests
from cryptography.hazmat.primitives import serialization

def get_installation_access_token(installation_id: str) -> str | None:
    app_id = os.environ.get("GITHUB_APP_ID")
    private_key_b64 = os.environ.get("GITHUB_APP_PRIVATE_KEY")
    if not app_id or not private_key_b64:
        return None

    pem = base64.b64decode(private_key_b64)
    private_key = serialization.load_pem_private_key(pem, password=None)

    now = int(time.time())
    payload = {"iat": now - 60, "exp": now + 600, "iss": app_id}
    app_jwt = jwt.encode(payload, private_key, algorithm="RS256")

    resp = requests.post(
        f"https://api.github.com/app/installations/{installation_id}/access_tokens",
        headers={
            "Authorization": f"Bearer {app_jwt}",
            "Accept": "application/vnd.github+json",
        },
        timeout=10,
    )
    return resp.json().get("token") if resp.ok else None
```

**Variables de entorno a agregar** en `.env.example` y `deployments/`:
```
GITHUB_APP_ID=""             # ID numérico (Settings → About)
GITHUB_APP_PRIVATE_KEY=""    # PEM en base64: base64 -w0 private-key.pem
```

**Usar en:**
- `GithubRepositoriesEndpoint` (reemplazar PAT por installation token)
- Registro automático de webhooks al crear `GithubRepositorySync`

---

### TAREA 6 — GitLab: OAuth callback en frontend

**Qué es:** El popup de GitLab ya se abre correctamente pero no hay página que reciba el
redirect tras la autorización.

**Estado actual:**
- `useIntegrationPopup` genera la URL OAuth de GitLab con `redirect_uri={origin}/auth/gitlab/callback`
- El backend `provider_install` acepta `{ code }` para GitLab
- **NO existe** `/auth/gitlab/callback` en el frontend

**Archivos a crear:**
```
apps/web/app/(all)/auth/gitlab/callback/page.tsx
```

El componente es casi idéntico a `apps/web/app/(all)/auth/github/callback/page.tsx`.
Cambios:
- Leer `code` en lugar de `installation_id` del query string
- POST `{ code }` a `/workspace-integrations/gitlab/install/`
- `postMessage` con `type: "gitlab-integration"`

**Actualizar `use-integration-popup.tsx`** para escuchar también `gitlab-integration`:
```typescript
if (!["github-integration", "gitlab-integration", "slack-integration"].includes(event.data?.type)) return;
```

**Registrar la ruta** en `apps/web/app/routes/core.ts`.

---

### TAREA 7 — Slack: OAuth callback en frontend

**Qué es:** El mismo problema que GitLab — falta la página de callback.

**Estado actual:**
- El popup se abre (URL en `useIntegrationPopup`)
- El backend ya intercambia el `code` por un access token via `slack.com/api/oauth.v2.access`
  y guarda el `team_id`, `team_name`, `access_token` en `config`
- **NO existe** `/auth/slack/callback` en el frontend

**Archivos a crear:**
```
apps/web/app/(all)/auth/slack/callback/page.tsx
```

Mismo patrón: leer `code`, POST a `/workspace-integrations/slack/install/`, postMessage con
`type: "slack-integration"`, cerrar popup.

**Nota sobre la detail page de Slack:** a diferencia de GitHub/GitLab no hay repos ni issues.
La detail page debería mostrar el workspace de Slack conectado (del `config.team_name`)
y la configuración de canales por proyecto (el modelo `SlackProjectSync` ya existe).

---

### TAREA 8 — Webhooks completos para sincronización en tiempo real

**Estado actual:** Skeleton en `apps/api/plane/app/views/external/sync.py`.
Maneja `issues` y `issue_comment` de GitHub pero está incompleto.

**Lo que falta:**

GitHub:
- Handler para `pull_request` event → actualizar estado según PR State Mapping (TAREA 3)
- Bug: `hmac.new` debe ser `hmac.new` → verificar firma correctamente
- Registro automático del webhook al crear `GithubRepositorySync` (TAREA 4 + TAREA 5)

GitLab:
- Handler para `merge_request` event
- Handler para `note` event (comentarios)

**Registrar webhook en GitHub al crear sync:**
```python
requests.post(
    f"https://api.github.com/repos/{owner}/{repo}/hooks",
    headers={"Authorization": f"Bearer {installation_token}"},
    json={
        "name": "web",
        "config": {
            "url": f"{settings.WEB_URL}/api/github-webhook/",
            "content_type": "json",
            "secret": os.environ.get("GITHUB_WEBHOOK_SECRET", ""),
        },
        "events": ["issues", "pull_request", "issue_comment"],
        "active": True,
    },
)
```

---

## Variables de entorno necesarias (resumen)

> ⚠️ **Las credenciales de auth e integración ya NO se leen desde el .env.**
> Todo debe configurarse desde God Mode (panel de administración).
> El .env solo es necesario para variables de infraestructura (DB, Redis, SECRET_KEY, etc.).



```bash
# GitHub App
GITHUB_APP_NAME=""           # slug del App (aparece en la URL de instalación)
GITHUB_APP_ID=""             # ID numérico del App
GITHUB_APP_PRIVATE_KEY=""    # PEM en base64: base64 -w0 private-key.pem
GITHUB_CLIENT_ID=""
GITHUB_CLIENT_SECRET=""
GITHUB_WEBHOOK_SECRET=""
IS_GITHUB_ENABLED=1

# GitLab
GITLAB_HOST="https://gitlab.com"
GITLAB_CLIENT_ID=""
GITLAB_CLIENT_SECRET=""
IS_GITLAB_ENABLED=1

# Slack
SLACK_CLIENT_ID=""
SLACK_CLIENT_SECRET=""
IS_SLACK_ENABLED=1
```

---

## Estado de implementación

| Tarea | Estado |
|---|---|
| TAREA 1 — Detail page por integración | ✅ Hecho |
| TAREA 2 — Cuenta personal GitHub | ✅ Hecho (UI trigger agregado + `github_client_id` expuesto en instancia API) |
| TAREA 3 — PR State Mapping | ✅ Hecho (ViewSet + serializer + URLs registradas en commit `542f78433`) |
| TAREA 4 — Project Issue Sync | ✅ Hecho + bugs adicionales corregidos (commits `e030f50`, `938a9de`): soft-delete resurrection (400 en re-registro), cleanup de GithubRepository huérfano en destroy, importación inicial automática de issues existentes al conectar repo |
| TAREA 5 — GitHub App JWT | ✅ Hecho |
| TAREA 6 — GitLab callback | ✅ Hecho |
| TAREA 7 — Slack callback | ✅ Hecho |
| TAREA 8 — Webhooks completos | ✅ Hecho (GitHub `pull_request` + GitLab `merge_request` handlers completos: 6 estados, linking por description, prevent_regression) |

---

## Features generales (no relacionados con integraciones)

| Feature | Estado |
|---|---|
| Dashboards + Reports (básico) | ✅ Parcial |
| Intake Forms | ✅ Existe |
| Pages / Wikis | ✅ Existe |
| Full Time Tracking | ✅ Existe |
| Teamspaces | ❌ Pendiente |
| Workflows + Approvals | ❌ Pendiente |
| Decision + Loops Automation | ❌ Pendiente |
| Custom Reports | ❌ Pendiente |
| Nested Pages | ❌ Pendiente |
| Project Templates | ❌ Pendiente |
| LDAP / GAC / Databases | ❌ Pendiente |

---

## Bugs generales (no integraciones)

| Bug | Archivo | Commit |
|---|---|---|
| `ReferenceError: Cannot access 'cycleIds' before initialization` en CycleOptions — temporal dead zone: `cycleIds` referenciado en `useCallback` antes de ser declarado | `apps/web/core/components/dropdowns/cycle/cycle-options.tsx` | `5be1ba3` |

---

## Documentación IA — dominio-ia.md (docs/api-rust/)

### Qué se documentó
Arquitectura completa de la integración IA de Plane: asistente Pi, reformulación de texto, multi-proveedor LLM.

### Errores encontrados en el código IA

| Error | Severidad | Detalle |
|---|---|---|
| **Endpoint `/rephrase-grammar/` ausente en Django** | 🔴 Crítico | El frontend llama `POST /api/workspaces/{slug}/rephrase-grammar/` desde el editor de páginas (Ask Pi + reformulación) pero la URL **no está registrada** en `apps/api/plane/app/urls/external.py` → 404 en producción |
| **`ASK_ANYTHING` incompleto en EditorAIMenu** | 🟡 Medio | El flujo retorna temprano sin llamar al backend; el input de `AskPiMenu` actualiza `query` en estado local pero no lo envía — botón de submit sin `onClick` |
| **Modelos Anthropic desactualizados** | 🟡 Medio | Solo incluye Claude 3 (2024); Claude 3.5, 3.7, Claude 4 serían rechazados por la validación de `model not in provider.models` |
| **Gemini vía OpenAI SDK sin proxy documentado** | 🟡 Medio | Requiere proxy LiteLLM externo no documentado ni validado en el código |

### Documento generado
`docs/api-rust/dominio-ia.md` — incluye: arquitectura, mapa de archivos, flujos mermaid, config de proveedores, errores, plan de implementación Rust.

---

## Correcciones — Patrones de error silencioso (docs/api-rust/)

> Rama: `feature/integrations-panel-fix-17593507967815292912`
> Completado: 2026-04-12

Todos los archivos modificados llevan la tag `// silence-patterns-ok` en cada línea corregida.

| # | Archivo | Patrón corregido | Fix aplicado | Commit |
|---|---------|-----------------|--------------|--------|
| 1 | `dominio-importadores.md` | `serde_json::to_value(...).unwrap()` | `map_err(AppError::Internal)?` | `cbdfa3f` |
| 2 | `dominio-importadores.md` | `update_importer_status(...).await.ok()` | `if let Err(e) + tracing::warn!` | `cbdfa3f` |
| 3 | `dominio-importadores.md` | `update_importer_progress(...).await.ok()` | `if let Err(e) + tracing::warn!` | `cbdfa3f` |
| 4 | `dominio-importadores.md` | `update_importer_status("completed").await.ok()` | `if let Err(e) + tracing::warn!` | `cbdfa3f` |
| 5-7 | `dominio-importadores.md` | `importer.project_id.unwrap()` ×3 | `ok_or_else(anyhow!)?` al inicio de función | `cbdfa3f` |
| 8 | `dominio-importadores.md` | `.insert(db).await.ok()` en labels | `if let Err(e) + tracing::warn!` | `cbdfa3f` |
| 9 | `dominio-intake.md` | `Job::new_async(...).unwrap()` | `map_err(anyhow!)?` | `d41eef4` |
| 10 | `dominio-intake.md` | `scheduler.add(...).await.unwrap()` | `map_err(anyhow!)?` | `d41eef4` |
| 11 | `dominio-issues.md` | `let _ = txn.rollback().await` | `if let Err(rb_err) + tracing::error!` | `6c244e2` |
| 12 | `dominio-integraciones.md` | `job_storage.push(...).await.ok()` | `if let Err(e) + tracing::warn!` | `9056ac1` |
| 13 | `dominio-integraciones.md` | `let _ = register_github_webhook(...).await` | `if let Err(e) + tracing::warn!` (nota de diseño) | `9056ac1` |
| 14 | `dominio-workspace-settings.md` | `.expect("failed to build HTTP client")` | `map_err(anyhow!)?` propagando en startup | `0ffe3bd` |
| 15 | `dominio-workspace-seed.md` | `.try_into().unwrap()` | `.expect()` con comentario `SAFETY:` (invariante estático) | `8fad804` |
| 16 | `impl-bootstrap.md` | `dotenv().ok()` | `match dotenv()` con `tracing::debug!/warn!` por caso | `4c388ab` |
| 17 | `dominio-notificaciones.md` | `job_storage.push(NotificationJob{...}).await.ok()` | `if let Err(e) + tracing::warn!` | `db1d637` |

---

## 📝 Historial de cambios en documentación

| Fecha | Acción | Detalle |
|---|---|---|
| 2026-04-12 | Merge `docs/restructure-api-rust` (8ccb7f8) | Reemplazó 18 árboles ASCII con mermaid mindmaps; eliminó tabla de rutas en `ref-diagramas-flujo.md`; introdujo `ref-guia-estilo.md` |
| 2026-04-12 | **Restauración post-merge** | Restaurados 18 archivos con árboles ASCII completos; restaurada tabla de rutas; fijado `dominio-ia.md` (nota ⚠️ NO REGISTRADO recuperada); fijado `dominio-workspace-settings.md` (conexiones incompletas en graph mermaid); `ref-guia-estilo.md` y `audit-report-2026-04-12.md` conservados |
| 2026-04-12 | **Path comments — cobertura total** | Agregados `// path` comments como primera línea en todos los bloques de código (`rust`, `python`, `typescript`, `toml`, `sql`) en los 40 archivos de `docs/api-rust/`. Commits `017874ee1` (batch 1) y `1aa9e77a3` (batch 2). Total ~62 bloques corregidos. |
