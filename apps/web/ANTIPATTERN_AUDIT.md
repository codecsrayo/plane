# Auditoría de Antipatrones — `apps/web`

Rama: `feature/integrations-panel-fix-17593507967815292912`

Marcar cada archivo al validar que está libre de antipatrones y patrones inseguros.

## Leyenda

- `- [ ]` — pendiente de revisión
- `- [x]` — revisado, libre de antipatrones
- `- [!]` — revisado, requiere refactor (dejar nota al lado)

## Criterios de revisión

- Sin `any` implícito/explícito innecesario, sin `@ts-ignore` sin justificación.
- Sin stale closures: dependencias de `useEffect`/`useCallback`/`useMemo` completas.
- Sin `useState` con valores derivables (preferir derivación directa o `useMemo`).
- Sin mutación directa de props o estado observable de MobX fuera de acciones.
- Sin `useEffect` que haga fetching sin cleanup o sin cancelación en unmount.
- Sin hardcode de URLs, tokens, o endpoints que deban venir de config.
- Sin `dangerouslySetInnerHTML` sin sanitización explícita.
- Sin event listeners globales sin cleanup.
- Sin `postMessage` sin validación de origen o tipo de mensaje.
- Sin TDZ (temporal dead zone): declaraciones antes de su uso en closures.
- Sin keys de React basadas en índice en listas que se reordenan.
- Sin acoplamiento directo a `window`/`document` sin `typeof window !== 'undefined'` en SSR paths.

**Total de archivos a auditar: 2068**

## `app/` — 190 archivos

### `app/`

- [x] `app/entry.client.tsx`
- [x] `app/layout.tsx` — Clarity `<Script>` ahora interpola con `JSON.stringify(sessionRecorderKey)` (mismo fix que `app/root.tsx`). `parseInt` → `Number.parseInt(..., 10)`. Nota: archivo sigue siendo un layout legacy de Next.js duplicado con `app/root.tsx` — remoción pendiente cuando se confirme que ya no lo usa ninguna ruta.
- [x] `app/not-found.tsx`
- [x] `app/provider.tsx`
- [x] `app/root.tsx` — Clarity `<Script>` ahora interpola con `JSON.stringify(sessionRecorderKey)` (neutraliza breakouts de `</script>`/quotes/`\u2028`). `parseInt` → `Number.parseInt(..., 10)` con radix explícito. Variable extraída para evitar doble lectura de `process.env`.
- [x] `app/routes.ts`

#### `app/(all)/`

- [x] `app/(all)/layout.preload.tsx`
- [x] `app/(all)/layout.tsx`

#### `app/(all)/[workspaceSlug]/`

- [x] `app/(all)/[workspaceSlug]/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/_sidebar.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/extended-project-sidebar.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/extended-sidebar-wrapper.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/extended-sidebar.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/page.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/sidebar.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/star-us-link.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/active-cycles/`

- [x] `app/(all)/[workspaceSlug]/(projects)/active-cycles/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/active-cycles/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/active-cycles/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/analytics/[tabId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/analytics/[tabId]/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/analytics/[tabId]/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/analytics/[tabId]/page.tsx` — Eliminado `useState` + `useEffect` que espejaban `tabId`. `selectedTab` ahora se deriva directamente del URL param (`const selectedTab = tabId || ANALYTICS_TABS[0]?.key`). `handleTabChange` ya no llama `setSelectedTab` — solo `router.push`, y el URL dispara el re-render con el nuevo valor derivado. Elimina la ventana de un frame con el tab desfasado vs el URL.

#### `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/page.tsx` — Resuelto: guards `window &&` redundantes eliminados dentro del `useEffect` (el efecto solo corre client-side). Ramas `< 768` y `>= 768` fusionadas en `if/else` único.
- [x] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/work-item-header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/drafts/`

- [x] `app/(all)/[workspaceSlug]/(projects)/drafts/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/drafts/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/drafts/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/notifications/`

- [x] `app/(all)/[workspaceSlug]/(projects)/notifications/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/notifications/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/layout.tsx` — Resuelto: (1) `isSmallerScreen` renombrado a `isDesktop` (semántica correcta, es `true` cuando la pantalla es ≥768px). (2) Duplicación `isAuthorizedPath`/`isIssuesTab` colapsada — ahora `isAuthorizedPath = isIssuesTab`.
- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/mobile-header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/navbar.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/[profileViewId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/[profileViewId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/activity/`

- [x] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/activity/page.tsx` — Resuelto: `key={i}` reemplazado por `key={cursor}` donde `cursor = \`${PER_PAGE}:${i}:0\`` (estable y único por página).

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/cycles/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/cycles/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/cycles/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/[archivedIssueId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/[archivedIssueId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/modules/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/modules/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/modules/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/mobile-header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/[cycleId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/[cycleId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/mobile-header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/intake/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/intake/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/intake/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(detail)/[issueId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(detail)/[issueId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/mobile-header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/mobile-header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/[moduleId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/[moduleId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/mobile-header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/[pageId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/[pageId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/header.tsx` — Resuelto: `catch (err: any)` → `catch (err: unknown)` con narrowing explícito antes de acceder a `err.data.error`.
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/[viewId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/[viewId]/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/[viewId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/mobile-header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/archives/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/archives/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/archives/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(list)/`

- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(list)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/projects/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/stickies/`

- [x] `app/(all)/[workspaceSlug]/(projects)/stickies/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/stickies/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/stickies/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/workspace-views/`

- [x] `app/(all)/[workspaceSlug]/(projects)/workspace-views/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/workspace-views/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/workspace-views/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/workspace-views/[globalViewId]/`

- [x] `app/(all)/[workspaceSlug]/(projects)/workspace-views/[globalViewId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/`

- [x] `app/(all)/[workspaceSlug]/(settings)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/billing/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/billing/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/billing/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/exports/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/exports/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/exports/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/[provider]/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/[provider]/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/members/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/members/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/members/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/[webhookId]/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/[webhookId]/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/[webhookId]/page.tsx` — Resuelto: `catch (error: any)` → `catch (error: unknown)` con narrowing explícito. Se eliminó el `eslint-disable`.

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/estimates/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/estimates/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/estimates/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/cycles/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/cycles/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/cycles/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/intake/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/intake/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/intake/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/modules/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/modules/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/modules/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/pages/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/pages/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/pages/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/views/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/views/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/views/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/labels/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/labels/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/labels/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/members/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/members/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/members/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/states/`

- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/states/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/states/page.tsx`

#### `app/(all)/accounts/forgot-password/`

- [x] `app/(all)/accounts/forgot-password/layout.tsx`
- [x] `app/(all)/accounts/forgot-password/page.tsx`

#### `app/(all)/accounts/reset-password/`

- [x] `app/(all)/accounts/reset-password/layout.tsx`
- [x] `app/(all)/accounts/reset-password/page.tsx`

#### `app/(all)/accounts/set-password/`

- [x] `app/(all)/accounts/set-password/layout.tsx`
- [x] `app/(all)/accounts/set-password/page.tsx`

#### `app/(all)/auth/github/callback/`

- [x] `app/(all)/auth/github/callback/page.tsx`

#### `app/(all)/auth/github/setup/`

- [x] `app/(all)/auth/github/setup/page.tsx`

#### `app/(all)/auth/github/user-callback/`

- [x] `app/(all)/auth/github/user-callback/page.tsx`

#### `app/(all)/auth/gitlab/callback/`

- [x] `app/(all)/auth/gitlab/callback/page.tsx`

#### `app/(all)/auth/slack/callback/`

- [x] `app/(all)/auth/slack/callback/page.tsx`

#### `app/(all)/create-workspace/`

- [x] `app/(all)/create-workspace/layout.tsx`
- [x] `app/(all)/create-workspace/page.tsx`

#### `app/(all)/invitations/`

- [x] `app/(all)/invitations/layout.tsx`
- [x] `app/(all)/invitations/page.tsx`

#### `app/(all)/onboarding/`

- [x] `app/(all)/onboarding/layout.tsx`
- [x] `app/(all)/onboarding/page.tsx`

#### `app/(all)/settings/profile/`

- [x] `app/(all)/settings/profile/layout.tsx`

#### `app/(all)/settings/profile/[profileTabId]/`

- [x] `app/(all)/settings/profile/[profileTabId]/page.tsx`

#### `app/(all)/sign-up/`

- [x] `app/(all)/sign-up/layout.tsx`
- [x] `app/(all)/sign-up/page.tsx`

#### `app/(all)/workspace-invitations/`

- [x] `app/(all)/workspace-invitations/layout.tsx`
- [x] `app/(all)/workspace-invitations/page.tsx`

#### `app/(home)/`

- [x] `app/(home)/layout.tsx`
- [x] `app/(home)/page.tsx`

#### `app/compat/next/`

- [x] `app/compat/next/helper.ts`
- [x] `app/compat/next/image.tsx`
- [x] `app/compat/next/link.tsx`
- [x] `app/compat/next/navigation.ts` — Timers ahora se trackean en `useRef<Set<TimeoutID>>` y se cancelan todos en el cleanup del `useEffect` de unmount → elimina navegaciones fantasma después de unmount y leak de timers pendientes. `location.reload()` → `window.location.reload()` con guard `typeof window !== "undefined"`. Comentario documenta que el bug raíz (actualizar estado durante render) debe arreglarse en los callers; este shim es puente migratorio.
- [x] `app/compat/next/script.tsx` — Reescrito: (1) `[key: string]: any` reemplazado por whitelist tipada (`Pick<ScriptHTMLAttributes, AllowedExtraAttr>` con mapa camelCase→kebab-case para `setAttribute`); (2) dep `rest` (nuevo objeto cada render) sustituido por `restKey = useMemo(JSON.stringify(rest))` → efecto ya no re-crea el `<script>` en cada render del parent; (3) `setAttribute` solo copia atributos en la whitelist (drop silencioso de desconocidos) — elimina vector XSS si un caller pasa event handlers como `onerror`; (4) branches `src`/`children` unificados; `document.body.removeChild` reemplazado por `script.remove()` (no-throw si el parent cambió).

#### `app/error/`

- [x] `app/error/dev.tsx`
- [x] `app/error/index.tsx`
- [x] `app/error/prod.tsx`

#### `app/routes/`

- [x] `app/routes/core.ts`
- [x] `app/routes/extended.ts`
- [x] `app/routes/helper.ts`

#### `app/routes/redirects/`

- [x] `app/routes/redirects/index.ts`

#### `app/routes/redirects/core/`

- [x] `app/routes/redirects/core/accounts-signup.tsx`
- [x] `app/routes/redirects/core/analytics.tsx`
- [x] `app/routes/redirects/core/api-tokens.tsx`
- [x] `app/routes/redirects/core/inbox.tsx`
- [x] `app/routes/redirects/core/index.ts`
- [x] `app/routes/redirects/core/login.tsx`
- [x] `app/routes/redirects/core/profile-settings.tsx`
- [x] `app/routes/redirects/core/project-settings.tsx`
- [x] `app/routes/redirects/core/register.tsx`
- [x] `app/routes/redirects/core/sign-in.tsx`
- [x] `app/routes/redirects/core/signin.tsx`
- [x] `app/routes/redirects/core/workspace-account-settings.tsx`

#### `app/routes/redirects/extended/`

- [x] `app/routes/redirects/extended/index.ts`

## `ce/` — 279 archivos

#### `ce/components/active-cycles/`

- [x] `ce/components/active-cycles/index.ts`
- [x] `ce/components/active-cycles/root.tsx`
- [x] `ce/components/active-cycles/workspace-active-cycles-upgrade.tsx`

#### `ce/components/analytics/`

- [x] `ce/components/analytics/tabs.tsx`
- [x] `ce/components/analytics/use-analytics-tabs.tsx`

#### `ce/components/app-rail/`

- [x] `ce/components/app-rail/app-rail-hoc.tsx`
- [x] `ce/components/app-rail/index.ts`

#### `ce/components/automations/`

- [x] `ce/components/automations/root.tsx`

#### `ce/components/automations/list/`

- [x] `ce/components/automations/list/wrapper.tsx`

#### `ce/components/breadcrumbs/`

- [x] `ce/components/breadcrumbs/common.tsx`
- [x] `ce/components/breadcrumbs/project-feature.tsx`
- [x] `ce/components/breadcrumbs/project.tsx`

#### `ce/components/browse/`

- [x] `ce/components/browse/workItem-detail.tsx`

#### `ce/components/command-palette/`

- [x] `ce/components/command-palette/helpers.tsx` — `TCommandGroups` pasó de index signature uniforme con `(item: any) => ReactNode` a objeto tipado por clave con generic `TCommandGroup<T>`, donde cada entrada (`cycle`/`issue`/`issue_view`/`module`/`page`/`project`/`workspace`) se ancla al shape real (`IWorkspaceIssueSearchResult`, `IWorkspacePageSearchResult`, etc.). Los parámetros `cycle`/`issue`/etc. en cada callback ahora inferidos automáticamente — sin anotaciones redundantes ni `any`. El archivo no tiene importers externos todavía (probablemente punto de extensión EE) pero el tipado correcto en el punto de definición previene bugs si/cuando se consume.
- [x] `ce/components/command-palette/index.ts`

#### `ce/components/command-palette/actions/`

- [x] `ce/components/command-palette/actions/index.ts`

#### `ce/components/command-palette/actions/work-item-actions/`

- [x] `ce/components/command-palette/actions/work-item-actions/change-state-list.tsx`
- [x] `ce/components/command-palette/actions/work-item-actions/index.ts`

#### `ce/components/command-palette/modals/`

- [x] `ce/components/command-palette/modals/project-level.tsx`
- [x] `ce/components/command-palette/modals/work-item-level.tsx`
- [x] `ce/components/command-palette/modals/workspace-level.tsx`

#### `ce/components/command-palette/power-k/`

- [x] `ce/components/command-palette/power-k/constants.ts`
- [x] `ce/components/command-palette/power-k/context-detector.ts`
- [x] `ce/components/command-palette/power-k/types.ts`

#### `ce/components/command-palette/power-k/hooks/`

- [x] `ce/components/command-palette/power-k/hooks/use-extended-context-indicator.ts`

#### `ce/components/command-palette/power-k/pages/context-based/`

- [x] `ce/components/command-palette/power-k/pages/context-based/index.ts`
- [x] `ce/components/command-palette/power-k/pages/context-based/root.tsx`

#### `ce/components/command-palette/power-k/pages/context-based/work-item/`

- [x] `ce/components/command-palette/power-k/pages/context-based/work-item/state-menu-item.tsx`

#### `ce/components/command-palette/power-k/search/`

- [x] `ce/components/command-palette/power-k/search/no-results-command.tsx`
- [x] `ce/components/command-palette/power-k/search/search-results-map.tsx`

#### `ce/components/comments/`

- [x] `ce/components/comments/comment-block.tsx`
- [x] `ce/components/comments/index.ts`

#### `ce/components/common/`

- [x] `ce/components/common/extended-app-header.tsx`
- [x] `ce/components/common/quick-actions-factory.tsx`

#### `ce/components/common/modal/`

- [x] `ce/components/common/modal/global.tsx`

#### `ce/components/common/subscription/`

- [x] `ce/components/common/subscription/subscription-pill.tsx`

#### `ce/components/cycles/`

- [x] `ce/components/cycles/additional-actions.tsx`
- [x] `ce/components/cycles/index.ts`

#### `ce/components/cycles/active-cycle/`

- [x] `ce/components/cycles/active-cycle/index.ts`
- [x] `ce/components/cycles/active-cycle/root.tsx` — `handleFiltersUpdate: (filters: any) => void` reemplazado por `(conditions: TWorkItemFilterCondition[]) => void`, alineado con el tipo real que retorna `useCyclesDetails` (y que los consumidores downstream `active-cycle/progress.tsx` y `cycle-stats.tsx` ya declaraban correctamente — este era el único eslabón con `any` en la cadena). Import de `TWorkItemFilterCondition` agregado desde `@plane/shared-state`. El parámetro también estaba mal nombrado (`filters`); la signature ahora refleja que son condiciones de filtro, no un objeto de filtros.

#### `ce/components/cycles/analytics-sidebar/`

- [x] `ce/components/cycles/analytics-sidebar/base.tsx`
- [x] `ce/components/cycles/analytics-sidebar/index.ts`
- [x] `ce/components/cycles/analytics-sidebar/root.tsx`

#### `ce/components/cycles/end-cycle/`

- [x] `ce/components/cycles/end-cycle/index.ts`
- [x] `ce/components/cycles/end-cycle/modal.tsx`

#### `ce/components/de-dupe/`

- [x] `ce/components/de-dupe/de-dupe-button.tsx`

#### `ce/components/de-dupe/duplicate-modal/`

- [x] `ce/components/de-dupe/duplicate-modal/index.ts`
- [x] `ce/components/de-dupe/duplicate-modal/root.tsx`

#### `ce/components/de-dupe/duplicate-popover/`

- [x] `ce/components/de-dupe/duplicate-popover/index.ts`
- [x] `ce/components/de-dupe/duplicate-popover/root.tsx`

#### `ce/components/de-dupe/issue-block/`

- [x] `ce/components/de-dupe/issue-block/button-label.tsx`

#### `ce/components/desktop/`

- [x] `ce/components/desktop/helper.ts`
- [x] `ce/components/desktop/index.ts`
- [x] `ce/components/desktop/sidebar-workspace-menu.tsx`

#### `ce/components/editor/embeds/mentions/`

- [x] `ce/components/editor/embeds/mentions/index.ts`
- [x] `ce/components/editor/embeds/mentions/root.tsx`

#### `ce/components/epics/epic-modal/`

- [x] `ce/components/epics/epic-modal/index.ts`
- [x] `ce/components/epics/epic-modal/modal.tsx`

#### `ce/components/estimates/`

- [x] `ce/components/estimates/estimate-list-item-buttons.tsx`
- [x] `ce/components/estimates/helper.tsx`
- [x] `ce/components/estimates/index.ts`

#### `ce/components/estimates/inputs/`

- [x] `ce/components/estimates/inputs/index.ts`
- [x] `ce/components/estimates/inputs/time-input.tsx`

#### `ce/components/estimates/points/`

- [x] `ce/components/estimates/points/delete.tsx`
- [x] `ce/components/estimates/points/index.ts`

#### `ce/components/estimates/update/`

- [x] `ce/components/estimates/update/index.ts`
- [x] `ce/components/estimates/update/modal.tsx`

#### `ce/components/gantt-chart/`

- [x] `ce/components/gantt-chart/index.ts`

#### `ce/components/gantt-chart/blocks/`

- [x] `ce/components/gantt-chart/blocks/block-row-list.tsx` — Resuelto: `blockUpdateHandler: (block: any, ...)` → `(block: unknown, ...)`. La rigidez `any` queda empujada a la capa `core/` (auditada como flag separado) pero la interfaz pública del CE ya no propaga `any`.
- [x] `ce/components/gantt-chart/blocks/blocks-list.tsx` — Resuelto: `blockToRender: (data: any) => ReactNode` → `(data: unknown) => ReactNode`.

#### `ce/components/gantt-chart/dependency/`

- [x] `ce/components/gantt-chart/dependency/dependency-paths.tsx`
- [x] `ce/components/gantt-chart/dependency/draggable-dependency-path.tsx`
- [x] `ce/components/gantt-chart/dependency/index.ts`

#### `ce/components/gantt-chart/dependency/blockDraggables/`

- [x] `ce/components/gantt-chart/dependency/blockDraggables/index.ts`
- [x] `ce/components/gantt-chart/dependency/blockDraggables/left-draggable.tsx`
- [x] `ce/components/gantt-chart/dependency/blockDraggables/right-draggable.tsx`

#### `ce/components/gantt-chart/layers/`

- [x] `ce/components/gantt-chart/layers/additional-layers.tsx`
- [x] `ce/components/gantt-chart/layers/index.ts`

#### `ce/components/global/`

- [x] `ce/components/global/index.ts`
- [x] `ce/components/global/version-number.tsx`

#### `ce/components/global/product-updates/`

- [x] `ce/components/global/product-updates/changelog.tsx`
- [x] `ce/components/global/product-updates/header.tsx`

#### `ce/components/home/`

- [x] `ce/components/home/header.tsx`
- [x] `ce/components/home/index.ts`
- [x] `ce/components/home/peek-overviews.tsx`

#### `ce/components/inbox/`

- [x] `ce/components/inbox/source-pill.tsx`

#### `ce/components/instance/`

- [x] `ce/components/instance/index.ts`
- [x] `ce/components/instance/maintenance-message.tsx`

#### `ce/components/issues/`

- [x] `ce/components/issues/header.tsx`

#### `ce/components/issues/bulk-operations/`

- [x] `ce/components/issues/bulk-operations/index.ts`
- [x] `ce/components/issues/bulk-operations/root.tsx`

#### `ce/components/issues/filters/`

- [x] `ce/components/issues/filters/issue-types.tsx`
- [x] `ce/components/issues/filters/team-project.tsx`

#### `ce/components/issues/filters/applied-filters/`

- [x] `ce/components/issues/filters/applied-filters/issue-types.tsx`

#### `ce/components/issues/issue-detail-widgets/`

- [x] `ce/components/issues/issue-detail-widgets/action-buttons.tsx`
- [x] `ce/components/issues/issue-detail-widgets/collapsibles.tsx`
- [x] `ce/components/issues/issue-detail-widgets/modals.tsx`

#### `ce/components/issues/issue-details/`

- [x] `ce/components/issues/issue-details/additional-activity-root.tsx`
- [x] `ce/components/issues/issue-details/additional-properties.tsx`
- [x] `ce/components/issues/issue-details/index.ts`
- [x] `ce/components/issues/issue-details/issue-creator.tsx`
- [x] `ce/components/issues/issue-details/issue-identifier.tsx`
- [x] `ce/components/issues/issue-details/issue-type-activity.tsx`
- [x] `ce/components/issues/issue-details/issue-type-switcher.tsx`
- [x] `ce/components/issues/issue-details/parent-select-root.tsx`

#### `ce/components/issues/issue-details/issue-properties-activity/`

- [x] `ce/components/issues/issue-details/issue-properties-activity/index.ts`
- [x] `ce/components/issues/issue-details/issue-properties-activity/root.tsx`

#### `ce/components/issues/issue-details/sidebar/`

- [x] `ce/components/issues/issue-details/sidebar/date-alert.tsx`
- [x] `ce/components/issues/issue-details/sidebar/transfer-hop-info.tsx`

#### `ce/components/issues/issue-layouts/`

- [x] `ce/components/issues/issue-layouts/additional-properties.tsx`
- [x] `ce/components/issues/issue-layouts/issue-stats.tsx`
- [x] `ce/components/issues/issue-layouts/utils.tsx`

#### `ce/components/issues/issue-layouts/empty-states/`

- [x] `ce/components/issues/issue-layouts/empty-states/index.ts`
- [x] `ce/components/issues/issue-layouts/empty-states/team-issues.tsx`
- [x] `ce/components/issues/issue-layouts/empty-states/team-project.tsx`
- [x] `ce/components/issues/issue-layouts/empty-states/team-view-issues.tsx`

#### `ce/components/issues/issue-layouts/quick-action-dropdowns/`

- [x] `ce/components/issues/issue-layouts/quick-action-dropdowns/copy-menu-helper.tsx`
- [x] `ce/components/issues/issue-layouts/quick-action-dropdowns/duplicate-modal.tsx`
- [x] `ce/components/issues/issue-layouts/quick-action-dropdowns/index.ts`

#### `ce/components/issues/issue-modal/`

- [x] `ce/components/issues/issue-modal/index.ts`
- [x] `ce/components/issues/issue-modal/issue-type-select.tsx`
- [x] `ce/components/issues/issue-modal/modal-additional-properties.tsx`
- [x] `ce/components/issues/issue-modal/provider.tsx`
- [x] `ce/components/issues/issue-modal/template-select.tsx`

#### `ce/components/issues/quick-add/`

- [x] `ce/components/issues/quick-add/index.ts`
- [x] `ce/components/issues/quick-add/root.tsx`

#### `ce/components/issues/worklog/activity/`

- [x] `ce/components/issues/worklog/activity/filter-root.tsx`
- [x] `ce/components/issues/worklog/activity/index.ts`
- [x] `ce/components/issues/worklog/activity/root.tsx`
- [x] `ce/components/issues/worklog/activity/worklog-create-button.tsx`

#### `ce/components/issues/worklog/property/`

- [x] `ce/components/issues/worklog/property/index.ts`
- [x] `ce/components/issues/worklog/property/root.tsx`

#### `ce/components/license/`

- [x] `ce/components/license/index.ts`

#### `ce/components/license/modal/`

- [x] `ce/components/license/modal/index.ts`
- [x] `ce/components/license/modal/upgrade-modal.tsx`

#### `ce/components/navigations/`

- [x] `ce/components/navigations/index.ts`
- [x] `ce/components/navigations/top-navigation-root.tsx`
- [x] `ce/components/navigations/use-navigation-items.ts`

#### `ce/components/onboarding/tour/`

- [x] `ce/components/onboarding/tour/root.tsx`
- [x] `ce/components/onboarding/tour/sidebar.tsx`

#### `ce/components/pages/`

- [x] `ce/components/pages/extra-actions.tsx`
- [x] `ce/components/pages/index.ts`

#### `ce/components/pages/editor/`

- [x] `ce/components/pages/editor/index.ts`

#### `ce/components/pages/editor/ai/`

- [x] `ce/components/pages/editor/ai/ask-pi-menu.tsx`
- [x] `ce/components/pages/editor/ai/index.ts`
- [x] `ce/components/pages/editor/ai/menu.tsx`

#### `ce/components/pages/editor/embed/`

- [x] `ce/components/pages/editor/embed/index.ts`
- [x] `ce/components/pages/editor/embed/issue-embed-upgrade-card.tsx` — `props: any` reemplazado por `IssueEmbedUpgradeCardProps = { selected?: boolean }` destructurado en el parámetro. Único uso interno (`props.selected`) confirmó que no hay más props; tipado mínimo y preciso.

#### `ce/components/pages/header/`

- [x] `ce/components/pages/header/collaborators-list.tsx`
- [x] `ce/components/pages/header/lock-control.tsx`
- [x] `ce/components/pages/header/move-control.tsx`
- [x] `ce/components/pages/header/share-control.tsx`

#### `ce/components/pages/modals/`

- [x] `ce/components/pages/modals/index.ts`
- [x] `ce/components/pages/modals/modals.tsx`
- [x] `ce/components/pages/modals/move-page-modal.tsx`

#### `ce/components/pages/navigation-pane/`

- [x] `ce/components/pages/navigation-pane/index.ts`

#### `ce/components/pages/navigation-pane/tab-panels/`

- [x] `ce/components/pages/navigation-pane/tab-panels/assets.tsx`
- [x] `ce/components/pages/navigation-pane/tab-panels/root.tsx`

#### `ce/components/pages/navigation-pane/tab-panels/empty-states/`

- [x] `ce/components/pages/navigation-pane/tab-panels/empty-states/assets.tsx`
- [x] `ce/components/pages/navigation-pane/tab-panels/empty-states/outline.tsx`

#### `ce/components/preferences/`

- [x] `ce/components/preferences/theme-switcher.tsx`

#### `ce/components/projects/`

- [x] `ce/components/projects/header.tsx`
- [x] `ce/components/projects/mobile-header.tsx`
- [x] `ce/components/projects/page.tsx`

#### `ce/components/projects/create/`

- [x] `ce/components/projects/create/attributes.tsx`
- [x] `ce/components/projects/create/root.tsx`
- [x] `ce/components/projects/create/template-select.tsx`
- [x] `ce/components/projects/create/utils.ts`

#### `ce/components/projects/navigation/`

- [x] `ce/components/projects/navigation/helper.tsx`

#### `ce/components/projects/settings/`

- [x] `ce/components/projects/settings/features-list.tsx`
- [x] `ce/components/projects/settings/useProjectColumns.tsx`

#### `ce/components/projects/settings/intake/`

- [x] `ce/components/projects/settings/intake/header.tsx`

#### `ce/components/projects/teamspaces/`

- [x] `ce/components/projects/teamspaces/teamspace-list.tsx`

#### `ce/components/relations/`

- [x] `ce/components/relations/activity.ts`
- [x] `ce/components/relations/index.tsx`

#### `ce/components/rich-filters/filter-value-input/`

- [x] `ce/components/rich-filters/filter-value-input/root.tsx`

#### `ce/components/sidebar/`

- [x] `ce/components/sidebar/app-switcher.tsx`
- [x] `ce/components/sidebar/index.ts`
- [x] `ce/components/sidebar/project-navigation-root.tsx`

#### `ce/components/views/`

- [x] `ce/components/views/access-controller.tsx` — Stub CE reescrito con genérico `<T extends FieldValues>` y `control: Control<T>` de `react-hook-form`. El CE renderiza `<></>` (feature EE) pero el prop tipado preserva la inferencia del form type en los callers (`form.tsx` con `IProjectView`, `workspace/views/form.tsx` con `IWorkspaceView`). Sin `any`; sin `eslint-disable`.
- [x] `ce/components/views/helper.tsx`

#### `ce/components/views/filters/`

- [x] `ce/components/views/filters/access-filter.tsx` — Resuelto: `props: any` → `FilterByAccessProps` con shape explícito (`appliedFilters`, `handleUpdate`, `searchQuery`, `accessFilters`) basado en el único caller en `filter-selection.tsx`. Props renombrado a `_props` para silenciar `@typescript-eslint/no-unused-vars`.

#### `ce/components/views/publish/`

- [x] `ce/components/views/publish/index.ts`
- [x] `ce/components/views/publish/modal.tsx`
- [x] `ce/components/views/publish/use-view-publish.tsx`

#### `ce/components/workflow/`

- [x] `ce/components/workflow/index.ts`
- [x] `ce/components/workflow/state-option.tsx`
- [x] `ce/components/workflow/use-workflow-drag-n-drop.ts`
- [x] `ce/components/workflow/workflow-disabled-message.tsx`
- [x] `ce/components/workflow/workflow-disabled-overlay.tsx`
- [x] `ce/components/workflow/workflow-group-tree.tsx`

#### `ce/components/workspace/`

- [x] `ce/components/workspace/app-switcher.tsx`
- [x] `ce/components/workspace/content-wrapper.tsx`
- [x] `ce/components/workspace/delete-workspace-modal.tsx`
- [x] `ce/components/workspace/delete-workspace-section.tsx`
- [x] `ce/components/workspace/edition-badge.tsx`
- [x] `ce/components/workspace/upgrade-badge.tsx`

#### `ce/components/workspace/billing/`

- [x] `ce/components/workspace/billing/billing-actions-button.tsx`
- [x] `ce/components/workspace/billing/index.ts`
- [x] `ce/components/workspace/billing/root.tsx`

#### `ce/components/workspace/billing/comparison/`

- [x] `ce/components/workspace/billing/comparison/frequency-toggle.tsx`
- [x] `ce/components/workspace/billing/comparison/plan-detail.tsx`
- [x] `ce/components/workspace/billing/comparison/root.tsx`

#### `ce/components/workspace/members/`

- [x] `ce/components/workspace/members/index.ts`
- [x] `ce/components/workspace/members/invite-modal.tsx`
- [x] `ce/components/workspace/members/members-activity-button.tsx`

#### `ce/components/workspace/settings/`

- [x] `ce/components/workspace/settings/useMemberColumns.tsx`

#### `ce/components/workspace/sidebar/`

- [x] `ce/components/workspace/sidebar/extended-sidebar-item.tsx` — Resuelto: `item.access as any` eliminado. `item.access` es `EUserWorkspaceRoles[]` y `allowPermissions` acepta `ETempUserRole[] = TUserPermissions | EUserWorkspaceRoles | EUserProjectRoles` — el array es estructuralmente compatible sin cast.
- [x] `ce/components/workspace/sidebar/helper.tsx`
- [x] `ce/components/workspace/sidebar/sidebar-item.tsx`
- [x] `ce/components/workspace/sidebar/teams-sidebar-list.tsx`

#### `ce/components/workspace-notifications/`

- [x] `ce/components/workspace-notifications/index.ts`
- [x] `ce/components/workspace-notifications/list-root.tsx`

#### `ce/components/workspace-notifications/notification-card/`

- [x] `ce/components/workspace-notifications/notification-card/content.ts`
- [x] `ce/components/workspace-notifications/notification-card/root.tsx`

#### `ce/hooks/`

- [x] `ce/hooks/use-additional-editor-mention.tsx`
- [x] `ce/hooks/use-additional-favorite-item-details.ts`
- [x] `ce/hooks/use-bulk-operation-status.ts`
- [x] `ce/hooks/use-debounced-duplicate-issues.tsx`
- [x] `ce/hooks/use-editor-flagging.ts`
- [x] `ce/hooks/use-file-size.ts`
- [x] `ce/hooks/use-issue-embed.tsx`
- [x] `ce/hooks/use-issue-properties.tsx`
- [x] `ce/hooks/use-notification-preview.tsx`
- [x] `ce/hooks/use-page-flag.ts`
- [x] `ce/hooks/use-timeline-chart.ts`
- [x] `ce/hooks/use-workspace-issue-properties-extended.tsx`

#### `ce/hooks/app-rail/`

- [x] `ce/hooks/app-rail/index.ts`
- [x] `ce/hooks/app-rail/provider.tsx`

#### `ce/hooks/editor/`

- [x] `ce/hooks/editor/use-extended-editor-config.ts`

#### `ce/hooks/pages/`

- [x] `ce/hooks/pages/index.ts`
- [x] `ce/hooks/pages/use-extended-editor-extensions.ts`
- [x] `ce/hooks/pages/use-pages-pane-extensions.ts`

#### `ce/hooks/rich-filters/`

- [x] `ce/hooks/rich-filters/use-filters-operator-configs.ts`

#### `ce/hooks/store/`

- [x] `ce/hooks/store/index.ts`
- [x] `ce/hooks/store/use-page-store.ts`
- [x] `ce/hooks/store/use-page.ts`

#### `ce/hooks/work-item-filters/`

- [x] `ce/hooks/work-item-filters/use-work-item-filters-config.tsx`

#### `ce/store/`

- [x] `ce/store/analytics.store.ts`
- [x] `ce/store/command-palette.store.ts`
- [x] `ce/store/global-view.store.ts`
- [x] `ce/store/power-k.store.ts`
- [x] `ce/store/project-inbox.store.ts`
- [x] `ce/store/project-view.store.ts`
- [x] `ce/store/root.store.ts`
- [x] `ce/store/state.store.ts`

#### `ce/store/cycle/`

- [x] `ce/store/cycle/index.ts`

#### `ce/store/estimates/`

- [x] `ce/store/estimates/estimate.ts`

#### `ce/store/issue/epic/`

- [x] `ce/store/issue/epic/filter.store.ts` — Comentarios `// @ts-nocheck` eran no-ops (TS solo los reconoce al inicio del archivo). Stub reescrito: body vacío — el parent `ProjectIssuesFilter` ya asigna `rootIssueStore`, y `implements IProjectEpicsFilter` se satisface automáticamente porque `IProjectEpicsFilter = IProjectIssuesFilter` y el parent ya lo implementa. El comentario anterior "this class will never be used" era incorrecto: sí se usa como CE fallback vía module resolver (`@/plane-web/store/issue/epic` → `ce/store/issue/epic/` en builds CE).
- [x] `ce/store/issue/epic/index.ts`
- [x] `ce/store/issue/epic/issue.store.ts` — Mismo patrón que epic/filter.store.ts — `@ts-nocheck` removido, body vacío, CE fallback documentado.

#### `ce/store/issue/helpers/`

- [x] `ce/store/issue/helpers/base-issue-store.ts`
- [x] `ce/store/issue/helpers/base-issue.store.ts`
- [x] `ce/store/issue/helpers/filter-utils.ts`

#### `ce/store/issue/issue-details/`

- [x] `ce/store/issue/issue-details/activity.store.ts`
- [x] `ce/store/issue/issue-details/root.store.ts`

#### `ce/store/issue/team/`

- [x] `ce/store/issue/team/filter.store.ts` — Mismo patrón que epic/filter.store.ts — `@ts-nocheck` removido, CE fallback documentado. `implements IProjectIssuesFilter` cambiado a `implements ITeamIssuesFilter` (tipo alias local, más expresivo).
- [x] `ce/store/issue/team/index.ts`
- [x] `ce/store/issue/team/issue.store.ts` — Mismo patrón. `implements IProjectIssues` → `implements ITeamIssues`.

#### `ce/store/issue/team-project/`

- [x] `ce/store/issue/team-project/filter.store.ts` — Mismo patrón. `@ts-nocheck` removido, CE fallback documentado.
- [x] `ce/store/issue/team-project/index.ts`
- [x] `ce/store/issue/team-project/issue.store.ts` — Mismo patrón. `implements IProjectIssues` → `implements ITeamProjectWorkItems`.

#### `ce/store/issue/team-views/`

- [x] `ce/store/issue/team-views/filter.store.ts` — Mismo patrón. `implements IProjectViewIssuesFilter` → `implements ITeamViewIssuesFilter`.
- [x] `ce/store/issue/team-views/index.ts`
- [x] `ce/store/issue/team-views/issue.store.ts` — Mismo patrón. `implements IProjectViewIssues` → `implements ITeamViewIssues`.

#### `ce/store/issue/workspace/`

- [x] `ce/store/issue/workspace/issue.store.ts`

#### `ce/store/member/`

- [x] `ce/store/member/project-member.store.ts`

#### `ce/store/pages/`

- [x] `ce/store/pages/extended-base-page.ts`

#### `ce/store/timeline/`

- [x] `ce/store/timeline/base-timeline.store.ts` — Nuevo tipo `TGanttRenderPayload = IWeekBlock[] | IMonthView | IMonthBlock[]` reemplaza `any` en el observable y en `updateRenderView`. `updatedBlockMaps` pasa de `{ path: string[]; value: any }[]` a `{ path: [string, keyof IGanttBlock]; value: IGanttBlock[keyof IGanttBlock] }[]`. Loop sobre `Object.keys(block)` ahora cast a `(keyof IGanttBlock)[]` una sola vez (no por iteración). Consumidores `week/month/quarter.tsx` actualizados con narrowing explícito (`as IWeekBlock[]` / `as IMonthView` / `as IMonthBlock[]`) y comentario del invariante runtime (la vista sólo se monta cuando `currentView` coincide con su tipo). `root.tsx` ya usaba la unión completa en su cast de `mergeRenderPayloads` → compatible sin cambios.
- [x] `ce/store/timeline/index.ts`

#### `ce/store/user/`

- [x] `ce/store/user/permission.store.ts`

#### `ce/store/workspace/`

- [x] `ce/store/workspace/index.ts`

#### `ce/types/`

- [x] `ce/types/gantt-chart.ts`
- [x] `ce/types/index.ts`

#### `ce/types/issue-types/`

- [x] `ce/types/issue-types/index.ts`

#### `ce/types/pages/`

- [x] `ce/types/pages/pane-extensions.ts`

#### `ce/types/projects/`

- [x] `ce/types/projects/index.ts`
- [x] `ce/types/projects/project-activity.ts`
- [x] `ce/types/projects/projects.ts`

## `core/` — 1589 archivos

#### `core/components/account/`

- [x] `core/components/account/deactivate-account-modal.tsx` — Resuelto: `.catch((err: any))` → `.catch((err: unknown))` con narrowing antes de acceder a `err.error`.
- [x] `core/components/account/terms-and-conditions.tsx`

#### `core/components/account/auth-forms/`

- [x] `core/components/account/auth-forms/auth-banner.tsx`
- [x] `core/components/account/auth-forms/auth-header.tsx`
- [x] `core/components/account/auth-forms/auth-root.tsx`
- [x] `core/components/account/auth-forms/email.tsx`
- [x] `core/components/account/auth-forms/forgot-password-popover.tsx`
- [x] `core/components/account/auth-forms/forgot-password.tsx` — Resuelto: `catch (err: any)` → `catch (err: unknown)` con narrowing antes de acceder a `err.error`.
- [x] `core/components/account/auth-forms/form-root.tsx` — Resuelto: `catch (error: any)` → `catch (error: unknown)` con narrowing antes de acceder a `error.error_code`.
- [x] `core/components/account/auth-forms/index.ts`
- [x] `core/components/account/auth-forms/password.tsx`
- [x] `core/components/account/auth-forms/reset-password.tsx`
- [x] `core/components/account/auth-forms/set-password.tsx`
- [x] `core/components/account/auth-forms/unique-code.tsx`

#### `core/components/account/auth-forms/common/`

- [x] `core/components/account/auth-forms/common/container.tsx`
- [x] `core/components/account/auth-forms/common/header.tsx`

#### `core/components/analytics/`

- [x] `core/components/analytics/analytics-filter-actions.tsx`
- [x] `core/components/analytics/analytics-section-wrapper.tsx`
- [x] `core/components/analytics/analytics-wrapper.tsx`
- [x] `core/components/analytics/empty-state.tsx`
- [x] `core/components/analytics/export.ts`
- [x] `core/components/analytics/insight-card.tsx`
- [x] `core/components/analytics/loaders.tsx`
- [x] `core/components/analytics/total-insights.tsx`
- [x] `core/components/analytics/trend-piece.tsx`

#### `core/components/analytics/insight-table/`

- [x] `core/components/analytics/insight-table/data-table.tsx` — Resuelto: eliminados los dos casts `as any` sobre `flexRender(...)`. La función ya retorna `React.ReactNode`, que es válido como JSX child directo sin cast.
- [x] `core/components/analytics/insight-table/index.ts`
- [x] `core/components/analytics/insight-table/loader.tsx`
- [x] `core/components/analytics/insight-table/root.tsx`

#### `core/components/analytics/overview/`

- [x] `core/components/analytics/overview/active-project-item.tsx`
- [x] `core/components/analytics/overview/active-projects.tsx`
- [x] `core/components/analytics/overview/index.ts`
- [x] `core/components/analytics/overview/project-insights.tsx`
- [x] `core/components/analytics/overview/root.tsx`

#### `core/components/analytics/select/`

- [x] `core/components/analytics/select/analytics-params.tsx`
- [x] `core/components/analytics/select/duration.tsx`
- [x] `core/components/analytics/select/project.tsx`
- [x] `core/components/analytics/select/select-x-axis.tsx`
- [x] `core/components/analytics/select/select-y-axis.tsx`

#### `core/components/analytics/work-items/`

- [x] `core/components/analytics/work-items/created-vs-resolved.tsx`
- [x] `core/components/analytics/work-items/customized-insights.tsx`
- [x] `core/components/analytics/work-items/index.ts`
- [x] `core/components/analytics/work-items/priority-chart.tsx`
- [x] `core/components/analytics/work-items/root.tsx`
- [x] `core/components/analytics/work-items/utils.ts`
- [x] `core/components/analytics/work-items/workitems-insight-table.tsx`

#### `core/components/analytics/work-items/modal/`

- [x] `core/components/analytics/work-items/modal/content.tsx`
- [x] `core/components/analytics/work-items/modal/header.tsx`
- [x] `core/components/analytics/work-items/modal/index.tsx`

#### `core/components/api-token/`

- [x] `core/components/api-token/delete-token-modal.tsx`
- [x] `core/components/api-token/empty-state.tsx`
- [x] `core/components/api-token/token-list-item.tsx`

#### `core/components/api-token/modal/`

- [x] `core/components/api-token/modal/create-token-modal.tsx` — Resuelto: `catch (err: any)` → `catch (err: unknown)` con narrowing antes de acceder a `err.message` / `err.detail`. `throw err` preservado para re-propagación.
- [x] `core/components/api-token/modal/form.tsx`
- [x] `core/components/api-token/modal/generated-token-details.tsx`

#### `core/components/appearance/`

- [x] `core/components/appearance/index.ts`
- [x] `core/components/appearance/theme-switcher.tsx`

#### `core/components/archives/`

- [x] `core/components/archives/archive-tabs-list.tsx`
- [x] `core/components/archives/index.ts`

#### `core/components/auth-screens/`

- [x] `core/components/auth-screens/auth-base.tsx`
- [x] `core/components/auth-screens/footer.tsx`
- [x] `core/components/auth-screens/header.tsx`
- [x] `core/components/auth-screens/not-authorized-view.tsx`

#### `core/components/auth-screens/project/`

- [x] `core/components/auth-screens/project/project-access-restriction.tsx`

#### `core/components/auth-screens/workspace/`

- [x] `core/components/auth-screens/workspace/not-a-member.tsx`

#### `core/components/automation/`

- [x] `core/components/automation/auto-archive-automation.tsx`
- [x] `core/components/automation/auto-close-automation.tsx`
- [x] `core/components/automation/index.ts`
- [x] `core/components/automation/select-month-modal.tsx`

#### `core/components/base-layouts/`

- [x] `core/components/base-layouts/constants.ts`
- [x] `core/components/base-layouts/layout-switcher.tsx`

#### `core/components/base-layouts/gantt/`

- [x] `core/components/base-layouts/gantt/index.ts`
- [x] `core/components/base-layouts/gantt/layout.tsx` — Resuelto: `sidebarProps: any` → `Omit<React.ComponentProps<typeof BaseGanttSidebar<T>>, "items" | "renderItem" | "loadMoreItems">` (los campos que este wrapper inyecta). Removido el `eslint-disable`.
- [x] `core/components/base-layouts/gantt/sidebar.tsx`

#### `core/components/base-layouts/hooks/`

- [x] `core/components/base-layouts/hooks/use-group-drop-target.ts`
- [x] `core/components/base-layouts/hooks/use-layout-state.ts`

#### `core/components/base-layouts/kanban/`

- [x] `core/components/base-layouts/kanban/group-header.tsx`
- [x] `core/components/base-layouts/kanban/group.tsx`
- [x] `core/components/base-layouts/kanban/item.tsx`
- [x] `core/components/base-layouts/kanban/layout.tsx`

#### `core/components/base-layouts/list/`

- [x] `core/components/base-layouts/list/group-header.tsx`
- [x] `core/components/base-layouts/list/group.tsx`
- [x] `core/components/base-layouts/list/item.tsx`
- [x] `core/components/base-layouts/list/layout.tsx`

#### `core/components/base-layouts/loaders/`

- [x] `core/components/base-layouts/loaders/layout-loader.tsx`

#### `core/components/chart/`

- [x] `core/components/chart/utils.ts`

#### `core/components/comments/`

- [x] `core/components/comments/comment-create.tsx`
- [x] `core/components/comments/comment-reaction.tsx`
- [x] `core/components/comments/comments.tsx`
- [x] `core/components/comments/index.ts`
- [x] `core/components/comments/quick-actions.tsx`

#### `core/components/comments/card/`

- [x] `core/components/comments/card/display.tsx`
- [x] `core/components/comments/card/edit-form.tsx`
- [x] `core/components/comments/card/root.tsx`

#### `core/components/common/`

- [x] `core/components/common/access-field.tsx`
- [x] `core/components/common/breadcrumb-link.tsx`
- [x] `core/components/common/count-chip.tsx`
- [x] `core/components/common/cover-image.tsx`
- [x] `core/components/common/empty-state.tsx` — Resuelto: `image: any` → `string | undefined` (verificado: todos los callers usan `import ... from "...svg?url"`, que resuelve a string URL). `icon?: any` → `React.ReactElement` (consistente con el prop `prependIcon` del Button de propel).
- [x] `core/components/common/latest-feature-block.tsx`
- [x] `core/components/common/logo-spinner.tsx`
- [x] `core/components/common/new-empty-state.tsx` — Resuelto: `image: any` → `string | undefined`; `icon?: any` → `React.ReactElement`. Mismo patrón aplicado al gemelo `empty-state.tsx`.
- [x] `core/components/common/page-access-icon.tsx`
- [x] `core/components/common/pro-icon.tsx`
- [x] `core/components/common/quick-actions-factory.tsx`
- [x] `core/components/common/quick-actions-helper.tsx`
- [x] `core/components/common/switcher-label.tsx`

#### `core/components/common/activity/`

- [x] `core/components/common/activity/activity-block.tsx`
- [x] `core/components/common/activity/activity-item.tsx`
- [x] `core/components/common/activity/helper.tsx`
- [x] `core/components/common/activity/user.tsx`

#### `core/components/common/applied-filters/`

- [x] `core/components/common/applied-filters/date.tsx`
- [x] `core/components/common/applied-filters/members.tsx`

#### `core/components/common/filters/`

- [x] `core/components/common/filters/created-at.tsx`
- [x] `core/components/common/filters/created-by.tsx`

#### `core/components/common/layout/sidebar/`

- [x] `core/components/common/layout/sidebar/property-list-item.tsx`

#### `core/components/core/`

- [x] `core/components/core/activity.tsx`
- [x] `core/components/core/app-header.tsx`
- [x] `core/components/core/content-overflow-HOC.tsx`
- [x] `core/components/core/content-wrapper.tsx`
- [x] `core/components/core/image-picker-popover.tsx`
- [x] `core/components/core/page-title.tsx`
- [x] `core/components/core/render-if-visible-HOC.tsx`

#### `core/components/core/description-versions/`

- [x] `core/components/core/description-versions/dropdown-item.tsx`
- [x] `core/components/core/description-versions/dropdown.tsx`
- [x] `core/components/core/description-versions/index.ts`
- [x] `core/components/core/description-versions/modal.tsx`
- [x] `core/components/core/description-versions/root.tsx`

#### `core/components/core/filters/`

- [x] `core/components/core/filters/date-filter-modal.tsx`
- [x] `core/components/core/filters/date-filter-select.tsx` — Resuelto: `icon: any` → `React.ReactElement`. En la data, los iconos son JSX pre-renderizado (`<CalendarBeforeIcon className="h-4 w-4" />`), no componentes — por eso `ReactElement` es la tipificación correcta (no `ComponentType`).

#### `core/components/core/list/`

- [x] `core/components/core/list/index.ts`
- [x] `core/components/core/list/list-item.tsx`
- [x] `core/components/core/list/list-root.tsx`

#### `core/components/core/modals/`

- [x] `core/components/core/modals/bulk-delete-issues-modal-item.tsx`
- [x] `core/components/core/modals/bulk-delete-issues-modal.tsx`
- [x] `core/components/core/modals/change-email-modal.tsx`
- [x] `core/components/core/modals/existing-issues-list-modal.tsx`
- [x] `core/components/core/modals/gpt-assistant-popover.tsx` — Resuelto: (1) `onResponse: (response: any)` → `(response: string)` — la `response` state es `useState<string>`. (2) `onError?: (error: any)` → `(error: unknown)`. (3) `handleServiceError(err: any)` → `(err: unknown)` con narrowing antes de acceder a `err.data.error` y `err.status`.
- [x] `core/components/core/modals/issue-search-modal-empty-state.tsx`
- [x] `core/components/core/modals/user-image-upload-modal.tsx` — Resuelto: `console.log` → `console.error`. Mensaje corregido: el catch está dentro del path de `delete` (no `upload`), así que se cambió a "Error removing user asset:".
- [x] `core/components/core/modals/workspace-image-upload-modal.tsx` — Resuelto: (1) `catch (error: any)` → `catch (error: unknown)` con narrowing. (2) Ambos `console.log` → `console.error` con mensajes corregidos ("Error uploading workspace asset:" y "Error removing workspace asset:").

#### `core/components/core/multiple-select/`

- [x] `core/components/core/multiple-select/entity-select-action.tsx`
- [x] `core/components/core/multiple-select/group-select-action.tsx`
- [x] `core/components/core/multiple-select/index.ts`
- [x] `core/components/core/multiple-select/select-group.tsx`

#### `core/components/core/sidebar/`

- [x] `core/components/core/sidebar/progress-chart.tsx`
- [x] `core/components/core/sidebar/sidebar-menu-hamburger-toggle.tsx`
- [x] `core/components/core/sidebar/single-progress-stats.tsx` — Resuelto: `title: any` → `React.ReactNode`.

#### `core/components/core/sidebar/progress-stats/`

- [x] `core/components/core/sidebar/progress-stats/assignee.tsx`
- [x] `core/components/core/sidebar/progress-stats/label.tsx`
- [x] `core/components/core/sidebar/progress-stats/shared.ts`
- [x] `core/components/core/sidebar/progress-stats/state_group.tsx`

#### `core/components/core/theme/`

- [x] `core/components/core/theme/color-inputs.tsx`
- [x] `core/components/core/theme/custom-theme-selector.tsx`
- [x] `core/components/core/theme/download-config-button.tsx`
- [x] `core/components/core/theme/import-config-button.tsx`
- [x] `core/components/core/theme/theme-mode-selector.tsx`
- [x] `core/components/core/theme/theme-switch.tsx`

#### `core/components/cycles/`

- [x] `core/components/cycles/cycle-peek-overview.tsx`
- [x] `core/components/cycles/cycles-view-header.tsx`
- [x] `core/components/cycles/cycles-view.tsx`
- [x] `core/components/cycles/delete-modal.tsx` — Resuelto: `catch (errors: any)` → `catch (err: unknown)` con narrowing antes de leer `err.error`. Se unificó el nombre de variable (`errors` → `err`).
- [x] `core/components/cycles/form.tsx`
- [x] `core/components/cycles/modal.tsx` — Resuelto: ambos `catch (err: any)` (create cycle y update cycle) → `catch (err: unknown)` con narrowing antes de leer `err.detail`.
- [x] `core/components/cycles/quick-actions.tsx`
- [x] `core/components/cycles/transfer-issues-modal.tsx`
- [x] `core/components/cycles/transfer-issues.tsx`

#### `core/components/cycles/active-cycle/`

- [x] `core/components/cycles/active-cycle/cycle-stats.tsx`
- [x] `core/components/cycles/active-cycle/productivity.tsx`
- [x] `core/components/cycles/active-cycle/progress.tsx` — Resuelto: `const groupedIssues: any = ...` → `const groupedIssues: Record<string, number> = ...`. Los valores del objeto (completed_issues, started_issues, etc.) son todos `number` en `ICycle`, y el uso posterior (`groupedIssues[group] > 0`) asume `number`.
- [x] `core/components/cycles/active-cycle/use-cycles-details.ts`

#### `core/components/cycles/analytics-sidebar/`

- [x] `core/components/cycles/analytics-sidebar/index.ts`
- [x] `core/components/cycles/analytics-sidebar/issue-progress.tsx` — Resuelto: `const updatedCycleDetails: any = { ...cycleDetails }` → `const updatedCycleDetails: ICycle = { ...cycleDetails }`. El spread preserva todas las claves de `ICycle`; la asignación dinámica por `keyof TProgressSnapshot` mantiene soundness porque esas keys son un subset de `keyof ICycle`. (Nota del audit sugería `Partial<ICycle>` pero eso quebraría la asignación indexada; `ICycle` es más apropiado aquí.)
- [x] `core/components/cycles/analytics-sidebar/progress-stats.tsx`
- [x] `core/components/cycles/analytics-sidebar/root.tsx`
- [x] `core/components/cycles/analytics-sidebar/sidebar-details.tsx`
- [x] `core/components/cycles/analytics-sidebar/sidebar-header.tsx` — Resuelto: `dateChecker(payload: any)` → `dateChecker(payload: CycleDateCheckData)`. Import añadido desde `@plane/types`. Tipo consistente con la firma de `cycleService.cycleDateCheck()`.

#### `core/components/cycles/applied-filters/`

- [x] `core/components/cycles/applied-filters/date.tsx`
- [x] `core/components/cycles/applied-filters/index.ts`
- [x] `core/components/cycles/applied-filters/root.tsx`
- [x] `core/components/cycles/applied-filters/status.tsx`

#### `core/components/cycles/archived-cycles/`

- [x] `core/components/cycles/archived-cycles/header.tsx`
- [x] `core/components/cycles/archived-cycles/index.ts`
- [x] `core/components/cycles/archived-cycles/modal.tsx`
- [x] `core/components/cycles/archived-cycles/root.tsx`
- [x] `core/components/cycles/archived-cycles/view.tsx`

#### `core/components/cycles/dropdowns/`

- [x] `core/components/cycles/dropdowns/estimate-type-dropdown.tsx`
- [x] `core/components/cycles/dropdowns/index.ts`

#### `core/components/cycles/dropdowns/filters/`

- [x] `core/components/cycles/dropdowns/filters/end-date.tsx`
- [x] `core/components/cycles/dropdowns/filters/index.ts`
- [x] `core/components/cycles/dropdowns/filters/root.tsx`
- [x] `core/components/cycles/dropdowns/filters/start-date.tsx`
- [x] `core/components/cycles/dropdowns/filters/status.tsx`

#### `core/components/cycles/list/`

- [x] `core/components/cycles/list/cycle-list-group-header.tsx`
- [x] `core/components/cycles/list/cycle-list-item-action.tsx`
- [x] `core/components/cycles/list/cycle-list-project-group-header.tsx`
- [x] `core/components/cycles/list/cycles-list-item.tsx`
- [x] `core/components/cycles/list/cycles-list-map.tsx`
- [x] `core/components/cycles/list/index.ts`
- [x] `core/components/cycles/list/root.tsx`

#### `core/components/dropdowns/`

- [x] `core/components/dropdowns/buttons.tsx`
- [x] `core/components/dropdowns/constants.ts`
- [x] `core/components/dropdowns/date-range.tsx`
- [x] `core/components/dropdowns/date.tsx`
- [x] `core/components/dropdowns/estimate.tsx` — Resuelto: `displayValue={(assigned: any) => assigned?.name}` → `(assigned: { name?: string } | null | undefined) => assigned?.name ?? ""`. Fallback a string vacío ya que la firma de headless UI espera `string`. Patrón unificado con los otros 6 dropdowns en esta tanda.
- [x] `core/components/dropdowns/layout.tsx` — Resuelto: `keyExtractor = useCallback((option: any) => option.value, [])` → `(option: { value: EIssueLayoutTypes }) => option.value`. El tipo refleja la forma de `options` (mapeado desde `ISSUE_LAYOUT_MAP`).
- [x] `core/components/dropdowns/merged-date.tsx`
- [x] `core/components/dropdowns/priority.tsx` — Resuelto: mismo patrón `displayValue: any` → `{ name?: string } | null | undefined` con fallback `?? ""`.

#### `core/components/dropdowns/cycle/`

- [x] `core/components/dropdowns/cycle/cycle-options.tsx`
- [x] `core/components/dropdowns/cycle/index.tsx`

#### `core/components/dropdowns/intake-state/`

- [x] `core/components/dropdowns/intake-state/base.tsx` — Resuelto: mismo patrón `displayValue: any` → `{ name?: string } | null | undefined` con fallback `?? ""`. Tanda unifica los 7 sitios con el mismo shape.
- [x] `core/components/dropdowns/intake-state/dropdown.tsx`

#### `core/components/dropdowns/member/`

- [x] `core/components/dropdowns/member/avatar.tsx`
- [x] `core/components/dropdowns/member/base.tsx`
- [x] `core/components/dropdowns/member/dropdown.tsx`
- [x] `core/components/dropdowns/member/member-options.tsx` — Resuelto: mismo patrón `displayValue: any` → `{ name?: string } | null | undefined` con fallback `?? ""`.

#### `core/components/dropdowns/module/`

- [x] `core/components/dropdowns/module/base.tsx` — Resuelto: `onChange as any` → `onChange as unknown as (val: string[]) => void`. Comentario explica que el child (`ModuleButtonContent`) solo invoca `onChange` en la rama `Array.isArray(value)`, nunca en single-select, así que la coerción es safe at runtime. Launder via `unknown` evita `any`.
- [x] `core/components/dropdowns/module/button-content.tsx`
- [x] `core/components/dropdowns/module/dropdown.tsx`
- [x] `core/components/dropdowns/module/module-options.tsx` — Resuelto: mismo patrón `displayValue: any` → `{ name?: string } | null | undefined` con fallback `?? ""`.

#### `core/components/dropdowns/project/`

- [x] `core/components/dropdowns/project/base.tsx` — Resuelto: mismo patrón `displayValue: any` → `{ name?: string } | null | undefined` con fallback `?? ""`. Patrón unificado en todos los 7 sitios flagged en `dropdowns/`.
- [x] `core/components/dropdowns/project/dropdown.tsx`

#### `core/components/dropdowns/state/`

- [x] `core/components/dropdowns/state/base.tsx` — Resuelto: mismo patrón `displayValue: any` → `{ name?: string } | null | undefined` con fallback `?? ""`.
- [x] `core/components/dropdowns/state/dropdown.tsx`

#### `core/components/editor/document/`

- [x] `core/components/editor/document/editor.tsx`

#### `core/components/editor/embeds/mentions/`

- [x] `core/components/editor/embeds/mentions/index.ts`
- [x] `core/components/editor/embeds/mentions/root.tsx`
- [x] `core/components/editor/embeds/mentions/user.tsx`

#### `core/components/editor/lite-text/`

- [x] `core/components/editor/lite-text/editor.tsx`
- [x] `core/components/editor/lite-text/index.ts`
- [x] `core/components/editor/lite-text/lite-toolbar.tsx`
- [x] `core/components/editor/lite-text/toolbar.tsx`

#### `core/components/editor/pdf/`

- [x] `core/components/editor/pdf/document.tsx`
- [x] `core/components/editor/pdf/index.ts`

#### `core/components/editor/rich-text/`

- [x] `core/components/editor/rich-text/editor.tsx`
- [x] `core/components/editor/rich-text/index.ts`

#### `core/components/editor/rich-text/description-input/`

- [x] `core/components/editor/rich-text/description-input/index.ts`
- [x] `core/components/editor/rich-text/description-input/loader.tsx`
- [x] `core/components/editor/rich-text/description-input/root.tsx` — Resuelto: `console.log("Error in uploading asset:", error)` → `console.error(...)`. El `throw new Error(..., { cause: error })` se mantiene intacto para re-propagación.

#### `core/components/editor/sticky-editor/`

- [x] `core/components/editor/sticky-editor/color-palette.tsx`
- [x] `core/components/editor/sticky-editor/editor.tsx`
- [x] `core/components/editor/sticky-editor/index.ts`
- [x] `core/components/editor/sticky-editor/toolbar.tsx`

#### `core/components/empty-state/`

- [x] `core/components/empty-state/comic-box-button.tsx` — Resuelto: `icon?: any` → `icon?: ReactNode`. Import `ReactNode` añadido (tree-shakeable `type`-only import).
- [x] `core/components/empty-state/detailed-empty-state-root.tsx`
- [x] `core/components/empty-state/helper.tsx`
- [x] `core/components/empty-state/section-empty-state-root.tsx`
- [x] `core/components/empty-state/simple-empty-state-root.tsx`

#### `core/components/estimates/`

- [x] `core/components/estimates/empty-screen.tsx`
- [x] `core/components/estimates/estimate-disable-switch.tsx`
- [x] `core/components/estimates/estimate-list-item.tsx`
- [x] `core/components/estimates/estimate-list.tsx`
- [x] `core/components/estimates/estimate-search.tsx`
- [x] `core/components/estimates/index.ts`
- [x] `core/components/estimates/loader-screen.tsx`
- [x] `core/components/estimates/radio-select.tsx`
- [x] `core/components/estimates/root.tsx`

#### `core/components/estimates/create/`

- [x] `core/components/estimates/create/modal.tsx`
- [x] `core/components/estimates/create/stage-one.tsx`

#### `core/components/estimates/delete/`

- [x] `core/components/estimates/delete/modal.tsx`

#### `core/components/estimates/inputs/`

- [x] `core/components/estimates/inputs/index.ts`
- [x] `core/components/estimates/inputs/number-input.tsx`
- [x] `core/components/estimates/inputs/root.tsx`
- [x] `core/components/estimates/inputs/text-input.tsx`

#### `core/components/estimates/points/`

- [x] `core/components/estimates/points/create-root.tsx`
- [x] `core/components/estimates/points/create.tsx`
- [x] `core/components/estimates/points/index.ts`
- [x] `core/components/estimates/points/preview.tsx`
- [x] `core/components/estimates/points/update.tsx`

#### `core/components/exporter/`

- [x] `core/components/exporter/column.tsx`
- [x] `core/components/exporter/export-form.tsx`
- [x] `core/components/exporter/export-modal.tsx` — Resuelto: `const onChange = (val: any) => setValue(val)` → `(val: string[])` (derivado del target `useState<string[]>([])`).
- [x] `core/components/exporter/guide.tsx`
- [x] `core/components/exporter/prev-exports.tsx`
- [x] `core/components/exporter/single-export.tsx`

#### `core/components/gantt-chart/`

- [x] `core/components/gantt-chart/constants.ts`
- [x] `core/components/gantt-chart/index.ts`
- [x] `core/components/gantt-chart/root.tsx` — `blockUpdateHandler`/`blockToRender`/`sidebarToRender` tipados con `unknown` (tightening del boundary).

#### `core/components/gantt-chart/blocks/`

- [x] `core/components/gantt-chart/blocks/block-row.tsx` — `blockUpdateHandler` `any`→`unknown`.
- [x] `core/components/gantt-chart/blocks/block.tsx` — `blockToRender` `any`→`unknown`.

#### `core/components/gantt-chart/chart/`

- [x] `core/components/gantt-chart/chart/header.tsx` — `VIEWS_LIST.map((chartView: any))` → `(chartView: ChartDataType)`.
- [x] `core/components/gantt-chart/chart/index.ts`
- [x] `core/components/gantt-chart/chart/main-content.tsx` — Cluster 3× `any`→`unknown`.
- [x] `core/components/gantt-chart/chart/root.tsx` — Cluster 3× `any`→`unknown`.
- [x] `core/components/gantt-chart/chart/timeline-drag-helper.tsx`

#### `core/components/gantt-chart/chart/views/`

- [x] `core/components/gantt-chart/chart/views/index.ts`
- [x] `core/components/gantt-chart/chart/views/month.tsx` — `_props: any` eliminado (componente no recibe props).
- [x] `core/components/gantt-chart/chart/views/quarter.tsx` — `_props: any` eliminado.
- [x] `core/components/gantt-chart/chart/views/week.tsx` — `_props: any` eliminado.

#### `core/components/gantt-chart/contexts/`

- [x] `core/components/gantt-chart/contexts/index.tsx`

#### `core/components/gantt-chart/data/`

- [x] `core/components/gantt-chart/data/index.ts`

#### `core/components/gantt-chart/helpers/`

- [x] `core/components/gantt-chart/helpers/add-block.tsx` — Resuelto: `blockUpdateHandler: (block: any, ...)` → `(block: unknown, ...)`. Consistente con el fix aplicado previamente en `ce/components/gantt-chart/blocks/block-row-list.tsx` y `blocks-list.tsx` (tanda 2). La rigidez `any` de `IGanttBlock.data` en `packages/types/src/layout/gantt.ts` es la fuente raíz, pero esta signature ya no la propaga.
- [x] `core/components/gantt-chart/helpers/draggable.tsx` — `blockToRender` `any`→`unknown`.
- [x] `core/components/gantt-chart/helpers/index.ts`

#### `core/components/gantt-chart/helpers/blockResizables/`

- [x] `core/components/gantt-chart/helpers/blockResizables/left-resizable.tsx`
- [x] `core/components/gantt-chart/helpers/blockResizables/right-resizable.tsx`
- [x] `core/components/gantt-chart/helpers/blockResizables/use-gantt-resizable.ts`

#### `core/components/gantt-chart/sidebar/`

- [x] `core/components/gantt-chart/sidebar/gantt-dnd-HOC.tsx`
- [x] `core/components/gantt-chart/sidebar/index.ts`
- [x] `core/components/gantt-chart/sidebar/root.tsx` — 2× `any`→`unknown`.
- [x] `core/components/gantt-chart/sidebar/utils.ts` — `blockUpdateHandler` signature `any`→`unknown`.

#### `core/components/gantt-chart/sidebar/issues/`

- [x] `core/components/gantt-chart/sidebar/issues/block.tsx`
- [x] `core/components/gantt-chart/sidebar/issues/index.ts`
- [x] `core/components/gantt-chart/sidebar/issues/sidebar.tsx` — `blockUpdateHandler` `any`→`unknown`.

#### `core/components/gantt-chart/sidebar/modules/`

- [x] `core/components/gantt-chart/sidebar/modules/block.tsx`
- [x] `core/components/gantt-chart/sidebar/modules/index.ts`
- [x] `core/components/gantt-chart/sidebar/modules/sidebar.tsx` — `blockUpdateHandler` `any`→`unknown`.

#### `core/components/gantt-chart/views/`

- [x] `core/components/gantt-chart/views/helpers.ts`
- [x] `core/components/gantt-chart/views/index.ts`
- [x] `core/components/gantt-chart/views/month-view.ts`
- [x] `core/components/gantt-chart/views/quarter-view.ts`
- [x] `core/components/gantt-chart/views/week-view.ts`

#### `core/components/global/`

- [x] `core/components/global/index.ts`
- [x] `core/components/global/timezone-select.tsx`

#### `core/components/global/product-updates/`

- [x] `core/components/global/product-updates/fallback.tsx`
- [x] `core/components/global/product-updates/footer.tsx`
- [x] `core/components/global/product-updates/index.ts`
- [x] `core/components/global/product-updates/modal.tsx`

#### `core/components/home/`

- [x] `core/components/home/home-dashboard-widgets.tsx`
- [x] `core/components/home/index.ts`
- [x] `core/components/home/root.tsx`
- [x] `core/components/home/user-greetings.tsx`

#### `core/components/home/widgets/`

- [x] `core/components/home/widgets/index.ts`

#### `core/components/home/widgets/empty-states/`

- [x] `core/components/home/widgets/empty-states/index.ts`
- [x] `core/components/home/widgets/empty-states/links.tsx`
- [x] `core/components/home/widgets/empty-states/no-projects.tsx`
- [x] `core/components/home/widgets/empty-states/recents.tsx`
- [x] `core/components/home/widgets/empty-states/stickies.tsx`

#### `core/components/home/widgets/links/`

- [x] `core/components/home/widgets/links/action.tsx`
- [x] `core/components/home/widgets/links/create-update-link-modal.tsx`
- [x] `core/components/home/widgets/links/index.ts`
- [x] `core/components/home/widgets/links/link-detail.tsx`
- [x] `core/components/home/widgets/links/links.tsx`
- [x] `core/components/home/widgets/links/root.tsx`
- [x] `core/components/home/widgets/links/use-links.tsx` — 3× `catch (error: any)` → `unknown`; `error?.data?.error` → `extractApiErrorMessage`.

#### `core/components/home/widgets/loaders/`

- [x] `core/components/home/widgets/loaders/home-loader.tsx`
- [x] `core/components/home/widgets/loaders/index.ts`
- [x] `core/components/home/widgets/loaders/loader.tsx`
- [x] `core/components/home/widgets/loaders/quick-links.tsx`
- [x] `core/components/home/widgets/loaders/recent-activity.tsx`

#### `core/components/home/widgets/manage/`

- [x] `core/components/home/widgets/manage/index.tsx`
- [x] `core/components/home/widgets/manage/widget-item-drag-handle.tsx`
- [x] `core/components/home/widgets/manage/widget-item.tsx`
- [x] `core/components/home/widgets/manage/widget-list.tsx`
- [x] `core/components/home/widgets/manage/widget.helpers.ts`

#### `core/components/home/widgets/recents/`

- [x] `core/components/home/widgets/recents/filters.tsx`
- [x] `core/components/home/widgets/recents/index.tsx`
- [x] `core/components/home/widgets/recents/issue.tsx`
- [x] `core/components/home/widgets/recents/page.tsx`
- [x] `core/components/home/widgets/recents/project.tsx`

#### `core/components/icons/`

- [x] `core/components/icons/index.ts`
- [x] `core/components/icons/locked-component.tsx`

#### `core/components/icons/attachment/`

- [x] `core/components/icons/attachment/attachment-icon.tsx`
- [x] `core/components/icons/attachment/audio-file-icon.tsx`
- [x] `core/components/icons/attachment/css-file-icon.tsx`
- [x] `core/components/icons/attachment/csv-file-icon.tsx`
- [x] `core/components/icons/attachment/default-file-icon.tsx`
- [x] `core/components/icons/attachment/doc-file-icon.tsx`
- [x] `core/components/icons/attachment/document-icon.tsx`
- [x] `core/components/icons/attachment/figma-file-icon.tsx`
- [x] `core/components/icons/attachment/html-file-icon.tsx`
- [x] `core/components/icons/attachment/img-file-icon.tsx`
- [x] `core/components/icons/attachment/index.ts`
- [x] `core/components/icons/attachment/jpg-file-icon.tsx`
- [x] `core/components/icons/attachment/js-file-icon.tsx`
- [x] `core/components/icons/attachment/pdf-file-icon.tsx`
- [x] `core/components/icons/attachment/png-file-icon.tsx`
- [x] `core/components/icons/attachment/rar-file-icon.tsx`
- [x] `core/components/icons/attachment/setting-icon.tsx`
- [x] `core/components/icons/attachment/sheet-file-icon.tsx`
- [x] `core/components/icons/attachment/svg-file-icon.tsx`
- [x] `core/components/icons/attachment/tune-icon.tsx`
- [x] `core/components/icons/attachment/txt-file-icon.tsx`
- [x] `core/components/icons/attachment/video-file-icon.tsx`
- [x] `core/components/icons/attachment/zip-file-icon.tsx`

#### `core/components/inbox/`

- [x] `core/components/inbox/inbox-issue-status.tsx`
- [x] `core/components/inbox/inbox-status-icon.tsx`
- [x] `core/components/inbox/index.ts`
- [x] `core/components/inbox/root.tsx`

#### `core/components/inbox/content/`

- [x] `core/components/inbox/content/inbox-issue-header.tsx`
- [x] `core/components/inbox/content/inbox-issue-mobile-header.tsx`
- [x] `core/components/inbox/content/index.ts`
- [x] `core/components/inbox/content/issue-properties.tsx`
- [x] `core/components/inbox/content/issue-root.tsx` — `console.log` → `console.error` en error handler.
- [x] `core/components/inbox/content/root.tsx`

#### `core/components/inbox/inbox-filter/`

- [x] `core/components/inbox/inbox-filter/index.ts`
- [x] `core/components/inbox/inbox-filter/root.tsx`

#### `core/components/inbox/inbox-filter/applied-filters/`

- [x] `core/components/inbox/inbox-filter/applied-filters/date.tsx`
- [x] `core/components/inbox/inbox-filter/applied-filters/label.tsx`
- [x] `core/components/inbox/inbox-filter/applied-filters/member.tsx`
- [x] `core/components/inbox/inbox-filter/applied-filters/priority.tsx`
- [x] `core/components/inbox/inbox-filter/applied-filters/root.tsx`
- [x] `core/components/inbox/inbox-filter/applied-filters/state.tsx`
- [x] `core/components/inbox/inbox-filter/applied-filters/status.tsx`

#### `core/components/inbox/inbox-filter/filters/`

- [x] `core/components/inbox/inbox-filter/filters/date.tsx`
- [x] `core/components/inbox/inbox-filter/filters/filter-selection.tsx`
- [x] `core/components/inbox/inbox-filter/filters/labels.tsx`
- [x] `core/components/inbox/inbox-filter/filters/members.tsx`
- [x] `core/components/inbox/inbox-filter/filters/priority.tsx`
- [x] `core/components/inbox/inbox-filter/filters/state.tsx`
- [x] `core/components/inbox/inbox-filter/filters/status.tsx`

#### `core/components/inbox/inbox-filter/sorting/`

- [x] `core/components/inbox/inbox-filter/sorting/order-by.tsx`

#### `core/components/inbox/modals/`

- [x] `core/components/inbox/modals/decline-issue-modal.tsx`
- [x] `core/components/inbox/modals/delete-issue-modal.tsx` — `catch (errors: any)` → `unknown`; `errors?.error` → `extractApiErrorMessage`.
- [x] `core/components/inbox/modals/select-duplicate.tsx`
- [x] `core/components/inbox/modals/snooze-issue-modal.tsx`

#### `core/components/inbox/modals/create-modal/`

- [x] `core/components/inbox/modals/create-modal/create-root.tsx`
- [x] `core/components/inbox/modals/create-modal/index.ts`
- [x] `core/components/inbox/modals/create-modal/issue-description.tsx` — (1) `console.log` → `console.error`. (2) `onEnterKeyPress?: (e?: any) => void` pendiente tipado.
- [x] `core/components/inbox/modals/create-modal/issue-properties.tsx`
- [x] `core/components/inbox/modals/create-modal/issue-title.tsx`
- [x] `core/components/inbox/modals/create-modal/modal.tsx`

#### `core/components/inbox/sidebar/`

- [x] `core/components/inbox/sidebar/inbox-list-item.tsx`
- [x] `core/components/inbox/sidebar/inbox-list.tsx`
- [x] `core/components/inbox/sidebar/index.ts`
- [x] `core/components/inbox/sidebar/root.tsx`

#### `core/components/instance/`

- [x] `core/components/instance/index.ts`
- [x] `core/components/instance/maintenance-view.tsx`
- [x] `core/components/instance/not-ready-view.tsx`

#### `core/components/integration/`

- [x] `core/components/integration/confirm-action-modal.tsx`
- [x] `core/components/integration/connected-account-details.tsx`
- [x] `core/components/integration/oauth-callback-page.tsx`
- [x] `core/components/integration/single-integration-card.tsx`
- [x] `core/components/integration/utils.ts`

#### `core/components/integration/github/`

- [x] `core/components/integration/github/personal-connect-card.tsx`
- [x] `core/components/integration/github/pr-state-mapping-modal.tsx`
- [x] `core/components/integration/github/pr-state-mapping.tsx`
- [x] `core/components/integration/github/project-issue-sync-modal.tsx`
- [x] `core/components/integration/github/project-issue-sync.tsx`
- [x] `core/components/integration/github/select-repository.tsx`

#### `core/components/integration/slack/`

- [x] `core/components/integration/slack/select-channel.tsx`

#### `core/components/issues/`

- [x] `core/components/issues/archive-issue-modal.tsx`
- [x] `core/components/issues/archived-issues-header.tsx`
- [x] `core/components/issues/confirm-issue-discard.tsx`
- [x] `core/components/issues/create-issue-toast-action-items.tsx`
- [x] `core/components/issues/delete-issue-modal.tsx`
- [x] `core/components/issues/filters.tsx`
- [x] `core/components/issues/issue-update-status.tsx`
- [!] `core/components/issues/label.tsx` — `labelDetails: any[]` (línea 12) — tipar con `ILabel[]`.
- [x] `core/components/issues/layout-quick-actions.tsx`
- [!] `core/components/issues/parent-issues-list-modal.tsx` — `value?: any` (línea 35) en prop type.
- [x] `core/components/issues/title-input.tsx`

#### `core/components/issues/attachment/`

- [x] `core/components/issues/attachment/attachment-detail.tsx`
- [x] `core/components/issues/attachment/attachment-item-list.tsx`
- [x] `core/components/issues/attachment/attachment-list-item.tsx`
- [x] `core/components/issues/attachment/attachment-list-upload-item.tsx`
- [x] `core/components/issues/attachment/attachment-upload-details.tsx`
- [x] `core/components/issues/attachment/attachment-upload.tsx`
- [x] `core/components/issues/attachment/attachments-list.tsx`
- [x] `core/components/issues/attachment/delete-attachment-modal.tsx`
- [x] `core/components/issues/attachment/index.ts`
- [x] `core/components/issues/attachment/root.tsx`

#### `core/components/issues/bulk-operations/`

- [x] `core/components/issues/bulk-operations/upgrade-banner.tsx`

#### `core/components/issues/issue-detail/`

- [x] `core/components/issues/issue-detail/cycle-select.tsx`
- [x] `core/components/issues/issue-detail/identifier-text.tsx`
- [x] `core/components/issues/issue-detail/index.ts`
- [x] `core/components/issues/issue-detail/issue-detail-quick-actions.tsx`
- [x] `core/components/issues/issue-detail/main-content.tsx`
- [x] `core/components/issues/issue-detail/module-select.tsx`
- [!] `core/components/issues/issue-detail/parent-select.tsx` — `onChange={(selectedIssue: any) => ...}` (línea 76).
- [x] `core/components/issues/issue-detail/relation-select.tsx`
- [x] `core/components/issues/issue-detail/root.tsx` — 5× `console.log` → `console.error` en catch handlers.
- [x] `core/components/issues/issue-detail/sidebar.tsx`
- [x] `core/components/issues/issue-detail/subscription.tsx`

#### `core/components/issues/issue-detail/issue-activity/`

- [x] `core/components/issues/issue-detail/issue-activity/activity-comment-root.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity-filter.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/helper.tsx` — `console.log` → `console.error`.
- [x] `core/components/issues/issue-detail/issue-activity/index.ts`
- [x] `core/components/issues/issue-detail/issue-activity/loader.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/root.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/sort-root.tsx`

#### `core/components/issues/issue-detail/issue-activity/activity/`

- [x] `core/components/issues/issue-detail/issue-activity/activity/activity-list.tsx`

#### `core/components/issues/issue-detail/issue-activity/activity/actions/`

- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/archived-at.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/assignee.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/attachment.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/cycle.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/default.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/description.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/estimate.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/inbox.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/index.ts`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/label-activity-chip.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/label.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/link.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/module.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/name.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/parent.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/priority.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/relation.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/start_date.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/state.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/target_date.tsx`

#### `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/`

- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/activity-block.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/issue-link.tsx`
- [x] `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/issue-user.tsx`

#### `core/components/issues/issue-detail/label/`

- [x] `core/components/issues/issue-detail/label/create-label.tsx`
- [x] `core/components/issues/issue-detail/label/index.ts`
- [x] `core/components/issues/issue-detail/label/label-list-item.tsx`
- [x] `core/components/issues/issue-detail/label/label-list.tsx`
- [!] `core/components/issues/issue-detail/label/root.tsx` — Cast `error as any` (línea 90) para leer `.error`. Tipar como `unknown` y narrowing con `typeof error === 'object' && 'error' in error`.

#### `core/components/issues/issue-detail/label/select/`

- [!] `core/components/issues/issue-detail/label/select/label-select.tsx` — `displayValue={(assigned: any) => ...}` (línea 160).
- [x] `core/components/issues/issue-detail/label/select/root.tsx`

#### `core/components/issues/issue-detail/links/`

- [x] `core/components/issues/issue-detail/links/create-update-link-modal.tsx`
- [x] `core/components/issues/issue-detail/links/index.ts`
- [x] `core/components/issues/issue-detail/links/link-detail.tsx`
- [x] `core/components/issues/issue-detail/links/link-item.tsx`
- [x] `core/components/issues/issue-detail/links/link-list.tsx`
- [x] `core/components/issues/issue-detail/links/links.tsx`
- [x] `core/components/issues/issue-detail/links/root.tsx` — `catch (error: any)` → `unknown`; `error?.data?.error` → `extractApiErrorMessage`.

#### `core/components/issues/issue-detail/parent/`

- [x] `core/components/issues/issue-detail/parent/index.ts`
- [x] `core/components/issues/issue-detail/parent/root.tsx`
- [x] `core/components/issues/issue-detail/parent/sibling-item.tsx`
- [x] `core/components/issues/issue-detail/parent/siblings.tsx`

#### `core/components/issues/issue-detail/reactions/`

- [x] `core/components/issues/issue-detail/reactions/index.ts`
- [x] `core/components/issues/issue-detail/reactions/issue-comment.tsx`
- [x] `core/components/issues/issue-detail/reactions/issue.tsx`

#### `core/components/issues/issue-detail-widgets/`

- [x] `core/components/issues/issue-detail-widgets/action-buttons.tsx`
- [x] `core/components/issues/issue-detail-widgets/index.ts`
- [x] `core/components/issues/issue-detail-widgets/issue-detail-widget-collapsibles.tsx`
- [x] `core/components/issues/issue-detail-widgets/issue-detail-widget-modals.tsx`
- [x] `core/components/issues/issue-detail-widgets/root.tsx`
- [x] `core/components/issues/issue-detail-widgets/widget-button.tsx`

#### `core/components/issues/issue-detail-widgets/attachments/`

- [x] `core/components/issues/issue-detail-widgets/attachments/content.tsx`
- [x] `core/components/issues/issue-detail-widgets/attachments/helper.tsx`
- [x] `core/components/issues/issue-detail-widgets/attachments/index.ts`
- [x] `core/components/issues/issue-detail-widgets/attachments/quick-action-button.tsx`
- [x] `core/components/issues/issue-detail-widgets/attachments/root.tsx`
- [x] `core/components/issues/issue-detail-widgets/attachments/title.tsx`

#### `core/components/issues/issue-detail-widgets/links/`

- [x] `core/components/issues/issue-detail-widgets/links/content.tsx`
- [x] `core/components/issues/issue-detail-widgets/links/helper.tsx` — 2× `catch (error: any)` → `unknown`; `error?.data?.error` → `extractApiErrorMessage`.
- [x] `core/components/issues/issue-detail-widgets/links/index.ts`
- [x] `core/components/issues/issue-detail-widgets/links/quick-action-button.tsx`
- [x] `core/components/issues/issue-detail-widgets/links/root.tsx`
- [x] `core/components/issues/issue-detail-widgets/links/title.tsx`

#### `core/components/issues/issue-detail-widgets/relations/`

- [x] `core/components/issues/issue-detail-widgets/relations/content.tsx`
- [x] `core/components/issues/issue-detail-widgets/relations/helper.tsx`
- [x] `core/components/issues/issue-detail-widgets/relations/index.ts`
- [x] `core/components/issues/issue-detail-widgets/relations/quick-action-button.tsx`
- [x] `core/components/issues/issue-detail-widgets/relations/root.tsx`
- [x] `core/components/issues/issue-detail-widgets/relations/title.tsx`

#### `core/components/issues/issue-detail-widgets/sub-issues/`

- [x] `core/components/issues/issue-detail-widgets/sub-issues/content.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/display-filters.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/filters.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/helper.ts`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/index.ts`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/quick-action-button.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/root.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/title-actions.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/title.tsx`

#### `core/components/issues/issue-detail-widgets/sub-issues/issues-list/`

- [x] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/list-group.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/list-item.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/properties.tsx`
- [x] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/root.tsx`

#### `core/components/issues/issue-layouts/`

- [x] `core/components/issues/issue-layouts/group-drag-overlay.tsx`
- [x] `core/components/issues/issue-layouts/issue-layout-HOC.tsx`
- [x] `core/components/issues/issue-layouts/layout-icon.tsx`
- [!] `core/components/issues/issue-layouts/utils.tsx` — 2× `let groupValue: any` (línea 549) y `subGroupValue: any` (línea 569).

#### `core/components/issues/issue-layouts/calendar/`

- [x] `core/components/issues/issue-layouts/calendar/base-calendar-root.tsx`
- [x] `core/components/issues/issue-layouts/calendar/calendar.tsx`
- [x] `core/components/issues/issue-layouts/calendar/day-tile.tsx`
- [x] `core/components/issues/issue-layouts/calendar/header.tsx`
- [x] `core/components/issues/issue-layouts/calendar/issue-block-root.tsx`
- [x] `core/components/issues/issue-layouts/calendar/issue-block.tsx`
- [x] `core/components/issues/issue-layouts/calendar/issue-blocks.tsx`
- [x] `core/components/issues/issue-layouts/calendar/quick-add-issue-actions.tsx`
- [x] `core/components/issues/issue-layouts/calendar/utils.ts`
- [x] `core/components/issues/issue-layouts/calendar/week-days.tsx`
- [x] `core/components/issues/issue-layouts/calendar/week-header.tsx`

#### `core/components/issues/issue-layouts/calendar/dropdowns/`

- [x] `core/components/issues/issue-layouts/calendar/dropdowns/index.ts`
- [x] `core/components/issues/issue-layouts/calendar/dropdowns/months-dropdown.tsx`
- [!] `core/components/issues/issue-layouts/calendar/dropdowns/options-dropdown.tsx` — `closePopover: any` (línea 69) — tipar con `() => void`.

#### `core/components/issues/issue-layouts/calendar/roots/`

- [x] `core/components/issues/issue-layouts/calendar/roots/cycle-root.tsx`
- [x] `core/components/issues/issue-layouts/calendar/roots/module-root.tsx`
- [x] `core/components/issues/issue-layouts/calendar/roots/project-root.tsx`
- [x] `core/components/issues/issue-layouts/calendar/roots/project-view-root.tsx`

#### `core/components/issues/issue-layouts/empty-states/`

- [x] `core/components/issues/issue-layouts/empty-states/archived-issues.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/cycle.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/global-view.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/index.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/module.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/profile-view.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/project-epic.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/project-issues.tsx`
- [x] `core/components/issues/issue-layouts/empty-states/project-view.tsx`

#### `core/components/issues/issue-layouts/filters/`

- [x] `core/components/issues/issue-layouts/filters/index.ts`

#### `core/components/issues/issue-layouts/filters/applied-filters/`

- [x] `core/components/issues/issue-layouts/filters/applied-filters/cycle.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/date.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/index.ts`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/label.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/members.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/module.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/priority.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/project.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/state-group.tsx`
- [x] `core/components/issues/issue-layouts/filters/applied-filters/state.tsx`

#### `core/components/issues/issue-layouts/filters/header/`

- [x] `core/components/issues/issue-layouts/filters/header/index.ts`
- [x] `core/components/issues/issue-layouts/filters/header/layout-selection.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/mobile-layout-selection.tsx`

#### `core/components/issues/issue-layouts/filters/header/display-filters/`

- [x] `core/components/issues/issue-layouts/filters/header/display-filters/display-filters-selection.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/display-filters/display-properties.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/display-filters/extra-options.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/display-filters/group-by.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/display-filters/index.ts`
- [x] `core/components/issues/issue-layouts/filters/header/display-filters/order-by.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/display-filters/sub-group-by.tsx`

#### `core/components/issues/issue-layouts/filters/header/filters/`

- [x] `core/components/issues/issue-layouts/filters/header/filters/assignee.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/created-by.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/cycle.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/due-date.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/index.ts`
- [x] `core/components/issues/issue-layouts/filters/header/filters/labels.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/mentions.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/module.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/priority.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/project.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/start-date.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/state-group.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/filters/state.tsx`

#### `core/components/issues/issue-layouts/filters/header/helpers/`

- [x] `core/components/issues/issue-layouts/filters/header/helpers/dropdown.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/helpers/filter-header.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/helpers/filter-option.tsx`
- [x] `core/components/issues/issue-layouts/filters/header/helpers/index.ts`

#### `core/components/issues/issue-layouts/gantt/`

- [!] `core/components/issues/issue-layouts/gantt/base-gantt-root.tsx` — `const payload: any = { ...data }` (línea 87) — réplica del patrón en `modules/gantt-chart/modules-list-layout.tsx`.
- [!] `core/components/issues/issue-layouts/gantt/blocks.tsx` — `handleIssuePeekOverview = (e: any) => ...` (línea 130) — tipar como `React.MouseEvent`.
- [x] `core/components/issues/issue-layouts/gantt/index.ts`

#### `core/components/issues/issue-layouts/kanban/`

- [x] `core/components/issues/issue-layouts/kanban/base-kanban-root.tsx`
- [x] `core/components/issues/issue-layouts/kanban/block.tsx`
- [x] `core/components/issues/issue-layouts/kanban/blocks-list.tsx`
- [x] `core/components/issues/issue-layouts/kanban/default.tsx`
- [x] `core/components/issues/issue-layouts/kanban/kanban-group.tsx`
- [x] `core/components/issues/issue-layouts/kanban/swimlanes.tsx`

#### `core/components/issues/issue-layouts/kanban/headers/`

- [x] `core/components/issues/issue-layouts/kanban/headers/group-by-card.tsx`
- [x] `core/components/issues/issue-layouts/kanban/headers/sub-group-by-card.tsx`

#### `core/components/issues/issue-layouts/kanban/roots/`

- [x] `core/components/issues/issue-layouts/kanban/roots/cycle-root.tsx`
- [x] `core/components/issues/issue-layouts/kanban/roots/module-root.tsx`
- [x] `core/components/issues/issue-layouts/kanban/roots/profile-issues-root.tsx`
- [x] `core/components/issues/issue-layouts/kanban/roots/project-root.tsx`
- [x] `core/components/issues/issue-layouts/kanban/roots/project-view-root.tsx`

#### `core/components/issues/issue-layouts/list/`

- [x] `core/components/issues/issue-layouts/list/base-list-root.tsx`
- [x] `core/components/issues/issue-layouts/list/block-root.tsx`
- [x] `core/components/issues/issue-layouts/list/block.tsx`
- [x] `core/components/issues/issue-layouts/list/blocks-list.tsx`
- [x] `core/components/issues/issue-layouts/list/default.tsx`
- [!] `core/components/issues/issue-layouts/list/list-group.tsx` — `prePopulateQuickAddData(groupByKey, value: any)` (línea 151).

#### `core/components/issues/issue-layouts/list/headers/`

- [x] `core/components/issues/issue-layouts/list/headers/group-by-card.tsx`

#### `core/components/issues/issue-layouts/list/roots/`

- [x] `core/components/issues/issue-layouts/list/roots/archived-issue-root.tsx`
- [x] `core/components/issues/issue-layouts/list/roots/cycle-root.tsx`
- [x] `core/components/issues/issue-layouts/list/roots/module-root.tsx`
- [x] `core/components/issues/issue-layouts/list/roots/profile-issues-root.tsx`
- [x] `core/components/issues/issue-layouts/list/roots/project-root.tsx`
- [x] `core/components/issues/issue-layouts/list/roots/project-view-root.tsx`

#### `core/components/issues/issue-layouts/properties/`

- [x] `core/components/issues/issue-layouts/properties/all-properties.tsx`
- [x] `core/components/issues/issue-layouts/properties/index.ts`
- [!] `core/components/issues/issue-layouts/properties/label-dropdown.tsx` — (1) `defaultOptions?: any` (línea 36). (2) `displayValue={(assigned: any) => ...}` (línea 274).
- [x] `core/components/issues/issue-layouts/properties/labels.tsx`
- [x] `core/components/issues/issue-layouts/properties/with-display-properties-HOC.tsx`

#### `core/components/issues/issue-layouts/quick-action-dropdowns/`

- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/all-issue.tsx`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/archived-issue.tsx`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/cycle-issue.tsx`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/helper.tsx`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/index.ts`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/issue-detail.tsx`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/module-issue.tsx`
- [x] `core/components/issues/issue-layouts/quick-action-dropdowns/project-issue.tsx`

#### `core/components/issues/issue-layouts/quick-add/`

- [x] `core/components/issues/issue-layouts/quick-add/index.ts`
- [x] `core/components/issues/issue-layouts/quick-add/root.tsx`

#### `core/components/issues/issue-layouts/quick-add/button/`

- [x] `core/components/issues/issue-layouts/quick-add/button/gantt.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/button/index.ts`
- [x] `core/components/issues/issue-layouts/quick-add/button/kanban.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/button/list.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/button/spreadsheet.tsx`

#### `core/components/issues/issue-layouts/quick-add/form/`

- [x] `core/components/issues/issue-layouts/quick-add/form/calendar.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/form/gantt.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/form/index.ts`
- [x] `core/components/issues/issue-layouts/quick-add/form/kanban.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/form/list.tsx`
- [x] `core/components/issues/issue-layouts/quick-add/form/spreadsheet.tsx`

#### `core/components/issues/issue-layouts/roots/`

- [x] `core/components/issues/issue-layouts/roots/all-issue-layout-root.tsx`
- [x] `core/components/issues/issue-layouts/roots/archived-issue-layout-root.tsx`
- [x] `core/components/issues/issue-layouts/roots/cycle-layout-root.tsx`
- [x] `core/components/issues/issue-layouts/roots/module-layout-root.tsx`
- [x] `core/components/issues/issue-layouts/roots/project-layout-root.tsx`
- [x] `core/components/issues/issue-layouts/roots/project-view-layout-root.tsx`

#### `core/components/issues/issue-layouts/spreadsheet/`

- [x] `core/components/issues/issue-layouts/spreadsheet/base-spreadsheet-root.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/issue-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/issue-row.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-header-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-header.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-table.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-view.tsx`

#### `core/components/issues/issue-layouts/spreadsheet/columns/`

- [x] `core/components/issues/issue-layouts/spreadsheet/columns/assignee-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange` (shared type).
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/attachment-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/created-on-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/cycle-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/due-date-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange`.
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/estimate-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange`.
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/header-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/index.ts`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/label-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange`.
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/link-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/module-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/priority-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange`.
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/start-date-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange`.
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/state-column.tsx` — `onChange` usa `TSpreadsheetColumnOnChange`.
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/sub-issue-column.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/columns/updated-on-column.tsx`

#### `core/components/issues/issue-layouts/spreadsheet/roots/`

- [x] `core/components/issues/issue-layouts/spreadsheet/roots/cycle-root.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/roots/module-root.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/roots/project-root.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/roots/project-view-root.tsx`
- [x] `core/components/issues/issue-layouts/spreadsheet/roots/workspace-root.tsx`

#### `core/components/issues/issue-modal/`

- [x] `core/components/issues/issue-modal/base.tsx` — 2× `catch (error: any)` → `unknown`.
- [x] `core/components/issues/issue-modal/draft-issue-layout.tsx`
- [!] `core/components/issues/issue-modal/form.tsx` — `dataResetProperties?: any[]` (línea 75).
- [x] `core/components/issues/issue-modal/modal.tsx`

#### `core/components/issues/issue-modal/components/`

- [x] `core/components/issues/issue-modal/components/default-properties.tsx`
- [x] `core/components/issues/issue-modal/components/description-editor.tsx` — `console.log` → `console.error`.
- [x] `core/components/issues/issue-modal/components/index.ts`
- [x] `core/components/issues/issue-modal/components/parent-tag.tsx`
- [x] `core/components/issues/issue-modal/components/project-select.tsx`
- [x] `core/components/issues/issue-modal/components/title-input.tsx`

#### `core/components/issues/issue-modal/context/`

- [x] `core/components/issues/issue-modal/context/index.ts`
- [x] `core/components/issues/issue-modal/context/issue-modal-context.tsx`

#### `core/components/issues/peek-overview/`

- [x] `core/components/issues/peek-overview/error.tsx`
- [x] `core/components/issues/peek-overview/header.tsx` — `icon: any`→`FC<SVGAttributes<SVGElement>>`; `onChange val: any`→`TPeekModes`.
- [x] `core/components/issues/peek-overview/index.ts`
- [x] `core/components/issues/peek-overview/issue-detail.tsx`
- [x] `core/components/issues/peek-overview/loader.tsx`
- [x] `core/components/issues/peek-overview/properties.tsx`
- [x] `core/components/issues/peek-overview/root.tsx`
- [x] `core/components/issues/peek-overview/view.tsx`

#### `core/components/issues/preview-card/`

- [x] `core/components/issues/preview-card/date.tsx`
- [x] `core/components/issues/preview-card/index.ts`
- [x] `core/components/issues/preview-card/root.tsx`

#### `core/components/issues/relations/`

- [x] `core/components/issues/relations/issue-list-item.tsx`
- [x] `core/components/issues/relations/issue-list.tsx`
- [x] `core/components/issues/relations/properties.tsx`

#### `core/components/issues/select/`

- [!] `core/components/issues/select/base.tsx` — `displayValue={(assigned: any) => assigned?.name}` (línea 211) — patrón repetido en dropdowns.
- [x] `core/components/issues/select/dropdown.tsx`
- [x] `core/components/issues/select/index.ts`

#### `core/components/issues/workspace-draft/`

- [x] `core/components/issues/workspace-draft/delete-modal.tsx` — `catch (errors: any)` → `unknown`; `errors?.error` → `extractApiErrorMessage`.
- [x] `core/components/issues/workspace-draft/draft-issue-block.tsx`
- [x] `core/components/issues/workspace-draft/draft-issue-properties.tsx`
- [x] `core/components/issues/workspace-draft/empty-state.tsx`
- [x] `core/components/issues/workspace-draft/index.ts`
- [x] `core/components/issues/workspace-draft/loader.tsx`
- [x] `core/components/issues/workspace-draft/quick-action.tsx`
- [x] `core/components/issues/workspace-draft/root.tsx`

#### `core/components/labels/`

- [!] `core/components/labels/create-update-label-inline.tsx` — `getErrorMessage = (error: any, operation) => string` (línea 73) — tipar `error` como `unknown` + narrowing.
- [x] `core/components/labels/delete-label-modal.tsx` — `catch (err: any)` → `unknown`; `err?.error` → `extractApiErrorMessage`.
- [x] `core/components/labels/index.ts`
- [x] `core/components/labels/label-drag-n-drop-HOC.tsx`
- [x] `core/components/labels/label-utils.ts`
- [x] `core/components/labels/project-setting-label-group.tsx`
- [x] `core/components/labels/project-setting-label-item.tsx`
- [x] `core/components/labels/project-setting-label-list.tsx`

#### `core/components/labels/label-block/`

- [x] `core/components/labels/label-block/label-item-block.tsx`
- [x] `core/components/labels/label-block/label-name.tsx`

#### `core/components/license/`

- [x] `core/components/license/index.ts`

#### `core/components/license/modal/`

- [x] `core/components/license/modal/index.ts`

#### `core/components/license/modal/card/`

- [x] `core/components/license/modal/card/base-paid-plan-card.tsx`
- [x] `core/components/license/modal/card/checkout-button.tsx`
- [x] `core/components/license/modal/card/discount-info.tsx`
- [x] `core/components/license/modal/card/free-plan.tsx`
- [x] `core/components/license/modal/card/index.ts`
- [x] `core/components/license/modal/card/plan-upgrade.tsx`
- [x] `core/components/license/modal/card/talk-to-sales.tsx`

#### `core/components/modules/`

- [x] `core/components/modules/delete-module-modal.tsx`
- [!] `core/components/modules/form.tsx` — `handleFormSubmit: (values, dirtyFields: any) => ...` (línea 26) — tipar `dirtyFields` con el tipo de react-hook-form.
- [x] `core/components/modules/index.ts`
- [x] `core/components/modules/modal.tsx` — 2× `catch (err: any)` → `unknown`.
- [x] `core/components/modules/module-card-item.tsx`
- [x] `core/components/modules/module-layout-icon.tsx`
- [x] `core/components/modules/module-list-item-action.tsx` — `catch (error: any)` → `unknown`.
- [x] `core/components/modules/module-list-item.tsx`
- [x] `core/components/modules/module-peek-overview.tsx`
- [x] `core/components/modules/module-status-dropdown.tsx`
- [x] `core/components/modules/module-view-header.tsx`
- [x] `core/components/modules/modules-list-view.tsx`
- [x] `core/components/modules/quick-actions.tsx`

#### `core/components/modules/analytics-sidebar/`

- [x] `core/components/modules/analytics-sidebar/index.ts`
- [x] `core/components/modules/analytics-sidebar/issue-progress.tsx`
- [x] `core/components/modules/analytics-sidebar/progress-stats.tsx`
- [x] `core/components/modules/analytics-sidebar/root.tsx` — `onChange={(nextValue: any)}` → `TModuleStatus`.

#### `core/components/modules/applied-filters/`

- [x] `core/components/modules/applied-filters/date.tsx`
- [x] `core/components/modules/applied-filters/index.ts`
- [x] `core/components/modules/applied-filters/members.tsx`
- [x] `core/components/modules/applied-filters/root.tsx`
- [x] `core/components/modules/applied-filters/status.tsx`

#### `core/components/modules/archived-modules/`

- [x] `core/components/modules/archived-modules/header.tsx`
- [x] `core/components/modules/archived-modules/index.ts`
- [x] `core/components/modules/archived-modules/modal.tsx`
- [x] `core/components/modules/archived-modules/root.tsx`
- [x] `core/components/modules/archived-modules/view.tsx`

#### `core/components/modules/dropdowns/`

- [x] `core/components/modules/dropdowns/index.ts`
- [x] `core/components/modules/dropdowns/order-by.tsx`

#### `core/components/modules/dropdowns/filters/`

- [x] `core/components/modules/dropdowns/filters/index.ts`
- [x] `core/components/modules/dropdowns/filters/lead.tsx`
- [x] `core/components/modules/dropdowns/filters/members.tsx`
- [x] `core/components/modules/dropdowns/filters/root.tsx`
- [x] `core/components/modules/dropdowns/filters/start-date.tsx`
- [x] `core/components/modules/dropdowns/filters/status.tsx`
- [x] `core/components/modules/dropdowns/filters/target-date.tsx`

#### `core/components/modules/gantt-chart/`

- [x] `core/components/modules/gantt-chart/blocks.tsx`
- [x] `core/components/modules/gantt-chart/index.ts`
- [!] `core/components/modules/gantt-chart/modules-list-layout.tsx` — `const payload: any = { ...data }` (línea 35).

#### `core/components/modules/links/`

- [x] `core/components/modules/links/create-update-modal.tsx` — `catch (error: any)` → `unknown`.
- [x] `core/components/modules/links/index.ts`
- [x] `core/components/modules/links/list-item.tsx`
- [x] `core/components/modules/links/list.tsx`

#### `core/components/modules/select/`

- [x] `core/components/modules/select/index.ts`
- [x] `core/components/modules/select/status.tsx`

#### `core/components/modules/sidebar-select/`

- [x] `core/components/modules/sidebar-select/index.ts`
- [x] `core/components/modules/sidebar-select/select-status.tsx` — `onChange={(nextValue: any)}` → `TModuleStatus`.

#### `core/components/navigation/`

- [x] `core/components/navigation/app-rail-root.tsx`
- [x] `core/components/navigation/customize-navigation-dialog.tsx`
- [x] `core/components/navigation/index.ts`
- [x] `core/components/navigation/items-root.tsx`
- [x] `core/components/navigation/project-actions-menu.tsx`
- [x] `core/components/navigation/project-header-button.tsx`
- [x] `core/components/navigation/project-header.tsx`
- [x] `core/components/navigation/tab-navigation-overflow-menu.tsx`
- [x] `core/components/navigation/tab-navigation-root.tsx`
- [x] `core/components/navigation/tab-navigation-utils.ts`
- [x] `core/components/navigation/tab-navigation-visible-item.tsx`
- [x] `core/components/navigation/top-nav-power-k.tsx`
- [x] `core/components/navigation/use-active-tab.ts`
- [x] `core/components/navigation/use-project-actions.ts`
- [x] `core/components/navigation/use-responsive-tab-layout.ts`
- [x] `core/components/navigation/use-tab-preferences.ts`

#### `core/components/onboarding/`

- [x] `core/components/onboarding/create-or-join-workspaces.tsx`
- [x] `core/components/onboarding/create-workspace.tsx`
- [x] `core/components/onboarding/header.tsx`
- [x] `core/components/onboarding/index.ts`
- [x] `core/components/onboarding/invitations.tsx`
- [x] `core/components/onboarding/invite-members.tsx` — `errors: any` → `FieldErrors<FormValues>`; `catch (err: any)` → `unknown`.
- [x] `core/components/onboarding/profile-setup.tsx`
- [x] `core/components/onboarding/root.tsx`
- [x] `core/components/onboarding/step-indicator.tsx`
- [x] `core/components/onboarding/switch-account-dropdown.tsx`
- [x] `core/components/onboarding/switch-account-modal.tsx`

#### `core/components/onboarding/steps/`

- [x] `core/components/onboarding/steps/index.ts`
- [x] `core/components/onboarding/steps/root.tsx`

#### `core/components/onboarding/steps/common/`

- [x] `core/components/onboarding/steps/common/header.tsx`
- [x] `core/components/onboarding/steps/common/index.ts`

#### `core/components/onboarding/steps/profile/`

- [x] `core/components/onboarding/steps/profile/consent.tsx`
- [x] `core/components/onboarding/steps/profile/index.ts`
- [x] `core/components/onboarding/steps/profile/root.tsx`
- [x] `core/components/onboarding/steps/profile/set-password.tsx`

#### `core/components/onboarding/steps/role/`

- [x] `core/components/onboarding/steps/role/index.ts`
- [x] `core/components/onboarding/steps/role/root.tsx`

#### `core/components/onboarding/steps/team/`

- [x] `core/components/onboarding/steps/team/index.ts`
- [x] `core/components/onboarding/steps/team/root.tsx` — `errors: any` → `FieldErrors<FormValues>`; `catch (err: any)` → `unknown`.

#### `core/components/onboarding/steps/usecase/`

- [x] `core/components/onboarding/steps/usecase/index.ts`
- [x] `core/components/onboarding/steps/usecase/root.tsx`

#### `core/components/onboarding/steps/workspace/`

- [x] `core/components/onboarding/steps/workspace/create.tsx`
- [x] `core/components/onboarding/steps/workspace/index.ts`
- [x] `core/components/onboarding/steps/workspace/join-invites.tsx` — `catch (error: any)` → `unknown`.
- [x] `core/components/onboarding/steps/workspace/root.tsx`

#### `core/components/pages/`

- [x] `core/components/pages/pages-list-main-content.tsx`
- [x] `core/components/pages/pages-list-view.tsx`

#### `core/components/pages/dropdowns/`

- [x] `core/components/pages/dropdowns/actions.tsx`
- [x] `core/components/pages/dropdowns/index.ts`

#### `core/components/pages/editor/`

- [x] `core/components/pages/editor/content-limit-banner.tsx`
- [x] `core/components/pages/editor/editor-body.tsx`
- [x] `core/components/pages/editor/page-root.tsx`
- [x] `core/components/pages/editor/title.tsx`

#### `core/components/pages/editor/header/`

- [x] `core/components/pages/editor/header/index.ts`
- [x] `core/components/pages/editor/header/logo-picker.tsx`
- [x] `core/components/pages/editor/header/root.tsx`

#### `core/components/pages/editor/summary/`

- [x] `core/components/pages/editor/summary/content-browser.tsx`
- [x] `core/components/pages/editor/summary/heading-components.tsx`
- [x] `core/components/pages/editor/summary/index.ts`

#### `core/components/pages/editor/toolbar/`

- [x] `core/components/pages/editor/toolbar/color-dropdown.tsx`
- [x] `core/components/pages/editor/toolbar/index.ts`
- [x] `core/components/pages/editor/toolbar/options-dropdown.tsx`
- [x] `core/components/pages/editor/toolbar/root.tsx`
- [x] `core/components/pages/editor/toolbar/toolbar.tsx`

#### `core/components/pages/header/`

- [x] `core/components/pages/header/actions.tsx`
- [x] `core/components/pages/header/archived-badge.tsx`
- [x] `core/components/pages/header/copy-link-control.tsx`
- [x] `core/components/pages/header/favorite-control.tsx`
- [x] `core/components/pages/header/index.ts`
- [x] `core/components/pages/header/offline-badge.tsx`
- [x] `core/components/pages/header/root.tsx`
- [x] `core/components/pages/header/syncing-badge.tsx`

#### `core/components/pages/list/`

- [x] `core/components/pages/list/block-item-action.tsx`
- [x] `core/components/pages/list/block.tsx`
- [x] `core/components/pages/list/index.ts`
- [x] `core/components/pages/list/order-by.tsx`
- [x] `core/components/pages/list/root.tsx`
- [x] `core/components/pages/list/search-input.tsx`
- [x] `core/components/pages/list/tab-navigation.tsx`

#### `core/components/pages/list/applied-filters/`

- [x] `core/components/pages/list/applied-filters/index.ts`
- [x] `core/components/pages/list/applied-filters/root.tsx`

#### `core/components/pages/list/filters/`

- [x] `core/components/pages/list/filters/index.ts`
- [x] `core/components/pages/list/filters/root.tsx`

#### `core/components/pages/loaders/`

- [x] `core/components/pages/loaders/page-content-loader.tsx`
- [x] `core/components/pages/loaders/page-loader.tsx`

#### `core/components/pages/modals/`

- [x] `core/components/pages/modals/create-page-modal.tsx`
- [x] `core/components/pages/modals/delete-page-modal.tsx`
- [x] `core/components/pages/modals/export-page-modal.tsx`
- [x] `core/components/pages/modals/page-form.tsx` — `onChange={(val: any)}` → `TChangeHandlerProps` (propel emoji-icon-picker).

#### `core/components/pages/navigation-pane/`

- [x] `core/components/pages/navigation-pane/index.ts`
- [x] `core/components/pages/navigation-pane/root.tsx`
- [x] `core/components/pages/navigation-pane/tabs-list.tsx`

#### `core/components/pages/navigation-pane/tab-panels/`

- [x] `core/components/pages/navigation-pane/tab-panels/assets.tsx`
- [x] `core/components/pages/navigation-pane/tab-panels/outline.tsx`
- [x] `core/components/pages/navigation-pane/tab-panels/root.tsx`

#### `core/components/pages/navigation-pane/tab-panels/info/`

- [x] `core/components/pages/navigation-pane/tab-panels/info/actors-info.tsx`
- [x] `core/components/pages/navigation-pane/tab-panels/info/document-info.tsx`
- [x] `core/components/pages/navigation-pane/tab-panels/info/root.tsx`
- [x] `core/components/pages/navigation-pane/tab-panels/info/version-history.tsx`

#### `core/components/pages/navigation-pane/types/`

- [x] `core/components/pages/navigation-pane/types/extensions.ts`
- [x] `core/components/pages/navigation-pane/types/index.ts`

#### `core/components/pages/version/`

- [x] `core/components/pages/version/editor.tsx`
- [x] `core/components/pages/version/index.ts`
- [x] `core/components/pages/version/main-content.tsx`
- [x] `core/components/pages/version/root.tsx`

#### `core/components/power-k/`

- [x] `core/components/power-k/global-shortcuts.tsx`
- [x] `core/components/power-k/projects-app-provider.tsx`

#### `core/components/power-k/actions/`

- [x] `core/components/power-k/actions/helper.ts`

#### `core/components/power-k/config/`

- [x] `core/components/power-k/config/account-commands.ts`
- [x] `core/components/power-k/config/commands.ts`
- [x] `core/components/power-k/config/help-commands.ts`
- [x] `core/components/power-k/config/miscellaneous-commands.ts`
- [x] `core/components/power-k/config/preferences-commands.ts`

#### `core/components/power-k/config/creation/`

- [x] `core/components/power-k/config/creation/command.ts`
- [x] `core/components/power-k/config/creation/root.ts`

#### `core/components/power-k/config/navigation/`

- [x] `core/components/power-k/config/navigation/commands.ts`
- [x] `core/components/power-k/config/navigation/root.ts`

#### `core/components/power-k/core/`

- [x] `core/components/power-k/core/context-detector.ts`
- [x] `core/components/power-k/core/registry.ts`
- [x] `core/components/power-k/core/shortcut-handler.ts`
- [x] `core/components/power-k/core/types.ts`

#### `core/components/power-k/hooks/`

- [x] `core/components/power-k/hooks/use-context-indicator.ts`

#### `core/components/power-k/menus/`

- [x] `core/components/power-k/menus/builder.tsx`
- [x] `core/components/power-k/menus/cycles.tsx`
- [x] `core/components/power-k/menus/empty-state.tsx`
- [x] `core/components/power-k/menus/labels.tsx`
- [x] `core/components/power-k/menus/members.tsx`
- [x] `core/components/power-k/menus/modules.tsx`
- [x] `core/components/power-k/menus/projects.tsx`
- [x] `core/components/power-k/menus/settings.tsx`
- [x] `core/components/power-k/menus/views.tsx`
- [x] `core/components/power-k/menus/workspaces.tsx`

#### `core/components/power-k/ui/modal/`

- [x] `core/components/power-k/ui/modal/command-item-shortcut-badge.tsx`
- [x] `core/components/power-k/ui/modal/command-item.tsx`
- [x] `core/components/power-k/ui/modal/commands-list.tsx`
- [x] `core/components/power-k/ui/modal/constants.ts`
- [x] `core/components/power-k/ui/modal/context-indicator.tsx`
- [x] `core/components/power-k/ui/modal/footer.tsx`
- [x] `core/components/power-k/ui/modal/header.tsx`
- [x] `core/components/power-k/ui/modal/search-menu.tsx`
- [!] `core/components/power-k/ui/modal/search-results-map.tsx` — 2× `any` en props `itemName: (item: any) => ReactNode` y `path: (item: any, ...) => string` (replica del patrón en `ce/components/command-palette/helpers.tsx`).
- [x] `core/components/power-k/ui/modal/search-results.tsx`
- [x] `core/components/power-k/ui/modal/shortcuts-root.tsx`
- [x] `core/components/power-k/ui/modal/wrapper.tsx`

#### `core/components/power-k/ui/pages/`

- [x] `core/components/power-k/ui/pages/default.tsx`
- [x] `core/components/power-k/ui/pages/index.ts`
- [x] `core/components/power-k/ui/pages/root.tsx`
- [x] `core/components/power-k/ui/pages/work-item-selection-page.tsx`

#### `core/components/power-k/ui/pages/context-based/`

- [x] `core/components/power-k/ui/pages/context-based/index.ts`
- [x] `core/components/power-k/ui/pages/context-based/root.tsx`

#### `core/components/power-k/ui/pages/context-based/cycle/`

- [x] `core/components/power-k/ui/pages/context-based/cycle/commands.ts`

#### `core/components/power-k/ui/pages/context-based/module/`

- [x] `core/components/power-k/ui/pages/context-based/module/commands.tsx`
- [x] `core/components/power-k/ui/pages/context-based/module/index.ts`
- [x] `core/components/power-k/ui/pages/context-based/module/root.tsx`
- [x] `core/components/power-k/ui/pages/context-based/module/status-menu.tsx`

#### `core/components/power-k/ui/pages/context-based/page/`

- [x] `core/components/power-k/ui/pages/context-based/page/commands.ts`

#### `core/components/power-k/ui/pages/context-based/work-item/`

- [x] `core/components/power-k/ui/pages/context-based/work-item/commands.ts`
- [x] `core/components/power-k/ui/pages/context-based/work-item/cycles-menu.tsx`
- [x] `core/components/power-k/ui/pages/context-based/work-item/estimates-menu.tsx`
- [x] `core/components/power-k/ui/pages/context-based/work-item/index.ts`
- [x] `core/components/power-k/ui/pages/context-based/work-item/labels-menu.tsx`
- [x] `core/components/power-k/ui/pages/context-based/work-item/modules-menu.tsx`
- [x] `core/components/power-k/ui/pages/context-based/work-item/priorities-menu.tsx`
- [x] `core/components/power-k/ui/pages/context-based/work-item/root.tsx`
- [x] `core/components/power-k/ui/pages/context-based/work-item/states-menu.tsx`

#### `core/components/power-k/ui/pages/open-entity/`

- [x] `core/components/power-k/ui/pages/open-entity/project-cycles-menu.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/project-modules-menu.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/project-settings-menu.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/project-views-menu.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/projects-menu.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/root.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/shared.ts`
- [x] `core/components/power-k/ui/pages/open-entity/workspace-settings-menu.tsx`
- [x] `core/components/power-k/ui/pages/open-entity/workspaces-menu.tsx`

#### `core/components/power-k/ui/pages/preferences/`

- [x] `core/components/power-k/ui/pages/preferences/index.ts`
- [x] `core/components/power-k/ui/pages/preferences/languages-menu.tsx`
- [x] `core/components/power-k/ui/pages/preferences/root.tsx`
- [x] `core/components/power-k/ui/pages/preferences/start-of-week-menu.tsx`
- [x] `core/components/power-k/ui/pages/preferences/themes-menu.tsx`
- [x] `core/components/power-k/ui/pages/preferences/timezone-menu.tsx`

#### `core/components/power-k/ui/renderer/`

- [x] `core/components/power-k/ui/renderer/command.tsx`
- [x] `core/components/power-k/ui/renderer/shared.ts`
- [x] `core/components/power-k/ui/renderer/shortcut.tsx`

#### `core/components/power-k/utils/`

- [x] `core/components/power-k/utils/navigation.ts`

#### `core/components/profile/`

- [x] `core/components/profile/profile-issues-filter.tsx`
- [x] `core/components/profile/profile-issues.tsx`
- [x] `core/components/profile/profile-setting-content-wrapper.tsx`
- [x] `core/components/profile/sidebar.tsx`
- [x] `core/components/profile/start-of-week-preference.tsx`
- [x] `core/components/profile/time.tsx`

#### `core/components/profile/activity/`

- [x] `core/components/profile/activity/activity-list.tsx`
- [x] `core/components/profile/activity/download-button.tsx`
- [!] `core/components/profile/activity/profile-activity-list.tsx` — `.map((activityItem: any) => ...)` (línea 65) — tipar con el shape de actividad.
- [x] `core/components/profile/activity/workspace-activity-list.tsx`

#### `core/components/profile/overview/`

- [x] `core/components/profile/overview/activity.tsx`
- [!] `core/components/profile/overview/priority-distribution.tsx` — `fill: (payload: any) => ...` (línea 48) con comment `// TODO: fix types` — deuda documentada, resolver.
- [x] `core/components/profile/overview/state-distribution.tsx`
- [x] `core/components/profile/overview/stats.tsx`
- [x] `core/components/profile/overview/workload.tsx`

#### `core/components/project/`

- [x] `core/components/project/archive-restore-modal.tsx`
- [x] `core/components/project/card-list.tsx`
- [x] `core/components/project/card.tsx`
- [x] `core/components/project/confirm-project-member-remove.tsx`
- [x] `core/components/project/create-project-modal.tsx`
- [x] `core/components/project/delete-project-modal.tsx`
- [!] `core/components/project/empty-state.tsx` — `image: any` + `icon?: any` — mismo patrón de `common/empty-state.tsx`.
- [x] `core/components/project/filters.tsx`
- [x] `core/components/project/form-loader.tsx`
- [x] `core/components/project/form.tsx` — `onChange={(val: any)}` → `TChangeHandlerProps`. Elimina TODO de tipado.
- [x] `core/components/project/header.tsx`
- [x] `core/components/project/integration-card.tsx`
- [x] `core/components/project/join-project-modal.tsx`
- [!] `core/components/project/leave-project-modal.tsx` — `onSubmit = async (data: any) => ...` (línea 56).
- [x] `core/components/project/member-header-column.tsx`
- [!] `core/components/project/member-list-item.tsx` — (1) 2× `catch (err: any)` (líneas 51, 63). (2) Cast `as any` (línea 85) sobre `memberDetails?.filter(...)` — arreglar el tipo de `data` prop del hijo.
- [x] `core/components/project/member-list.tsx`
- [!] `core/components/project/member-select.tsx` — `value: any` (línea 20) en prop type.
- [x] `core/components/project/multi-select-modal.tsx`
- [x] `core/components/project/project-feature-update.tsx`
- [x] `core/components/project/project-network-icon.tsx`
- [x] `core/components/project/project-settings-member-defaults.tsx`
- [x] `core/components/project/root.tsx`
- [x] `core/components/project/search-projects.tsx`
- [x] `core/components/project/send-project-invitation-modal.tsx`

#### `core/components/project/applied-filters/`

- [x] `core/components/project/applied-filters/access.tsx`
- [x] `core/components/project/applied-filters/date.tsx`
- [x] `core/components/project/applied-filters/index.ts`
- [x] `core/components/project/applied-filters/members.tsx`
- [x] `core/components/project/applied-filters/project-display-filters.tsx`
- [x] `core/components/project/applied-filters/root.tsx`

#### `core/components/project/create/`

- [x] `core/components/project/create/common-attributes.tsx`
- [x] `core/components/project/create/header.tsx` — `onChange={(val: any)}` → `TChangeHandlerProps`.
- [x] `core/components/project/create/project-create-buttons.tsx`

#### `core/components/project/dropdowns/`

- [x] `core/components/project/dropdowns/order-by.tsx`

#### `core/components/project/dropdowns/filters/`

- [x] `core/components/project/dropdowns/filters/access.tsx`
- [x] `core/components/project/dropdowns/filters/created-at.tsx`
- [x] `core/components/project/dropdowns/filters/index.ts`
- [x] `core/components/project/dropdowns/filters/lead.tsx`
- [x] `core/components/project/dropdowns/filters/member-list.tsx`
- [x] `core/components/project/dropdowns/filters/members.tsx`
- [x] `core/components/project/dropdowns/filters/root.tsx`

#### `core/components/project/publish-project/`

- [x] `core/components/project/publish-project/modal.tsx`

#### `core/components/project/settings/`

- [x] `core/components/project/settings/control-section.tsx`
- [x] `core/components/project/settings/features-list.tsx`
- [!] `core/components/project/settings/helper.tsx` — `featureItem: any` (línea 17).
- [x] `core/components/project/settings/member-columns.tsx` — debug `console.log(err, 'err')` → `console.error` con mensaje descriptivo.

#### `core/components/project-states/`

- [x] `core/components/project-states/group-item.tsx`
- [x] `core/components/project-states/group-list.tsx`
- [x] `core/components/project-states/index.ts`
- [x] `core/components/project-states/loader.tsx`
- [x] `core/components/project-states/root.tsx`
- [x] `core/components/project-states/state-delete-modal.tsx`
- [x] `core/components/project-states/state-item-title.tsx`
- [x] `core/components/project-states/state-item.tsx`
- [x] `core/components/project-states/state-list.tsx`

#### `core/components/project-states/create-update/`

- [x] `core/components/project-states/create-update/create.tsx`
- [x] `core/components/project-states/create-update/form.tsx` — `console.log` → `console.error`.
- [x] `core/components/project-states/create-update/index.ts`
- [x] `core/components/project-states/create-update/update.tsx`

#### `core/components/project-states/options/`

- [x] `core/components/project-states/options/delete.tsx`
- [x] `core/components/project-states/options/index.ts`
- [x] `core/components/project-states/options/mark-as-default.tsx`

#### `core/components/readonly/`

- [x] `core/components/readonly/cycle.tsx`
- [x] `core/components/readonly/date.tsx`
- [x] `core/components/readonly/estimate.tsx`
- [x] `core/components/readonly/index.tsx`
- [x] `core/components/readonly/labels.tsx`
- [x] `core/components/readonly/member.tsx`
- [x] `core/components/readonly/module.tsx`
- [x] `core/components/readonly/priority.tsx`
- [x] `core/components/readonly/state.tsx`

#### `core/components/rich-filters/`

- [x] `core/components/rich-filters/filters-row.tsx`
- [x] `core/components/rich-filters/filters-toggle.tsx`
- [x] `core/components/rich-filters/shared.ts`

#### `core/components/rich-filters/add-filters/`

- [x] `core/components/rich-filters/add-filters/button.tsx`
- [x] `core/components/rich-filters/add-filters/dropdown.tsx`

#### `core/components/rich-filters/filter-item/`

- [x] `core/components/rich-filters/filter-item/close-button.tsx`
- [x] `core/components/rich-filters/filter-item/container.tsx`
- [x] `core/components/rich-filters/filter-item/invalid.tsx`
- [x] `core/components/rich-filters/filter-item/loader.tsx`
- [x] `core/components/rich-filters/filter-item/property.tsx`
- [x] `core/components/rich-filters/filter-item/root.tsx`

#### `core/components/rich-filters/filter-value-input/`

- [x] `core/components/rich-filters/filter-value-input/root.tsx`

#### `core/components/rich-filters/filter-value-input/date/`

- [x] `core/components/rich-filters/filter-value-input/date/range.tsx`
- [x] `core/components/rich-filters/filter-value-input/date/single.tsx`

#### `core/components/rich-filters/filter-value-input/select/`

- [x] `core/components/rich-filters/filter-value-input/select/multi.tsx`
- [x] `core/components/rich-filters/filter-value-input/select/selected-options-display.tsx`
- [x] `core/components/rich-filters/filter-value-input/select/shared.tsx`
- [x] `core/components/rich-filters/filter-value-input/select/single.tsx`

#### `core/components/settings/`

- [x] `core/components/settings/boxed-control-item.tsx`
- [x] `core/components/settings/content-wrapper.tsx`
- [x] `core/components/settings/control-item.tsx`
- [x] `core/components/settings/heading.tsx`
- [!] `core/components/settings/helper.ts` — `Record<..., Array<{ ...; [key: string]: any }>>` (línea 9) — quitar el index signature `any`.
- [x] `core/components/settings/layout.tsx`
- [x] `core/components/settings/page-header.tsx`

#### `core/components/settings/mobile/`

- [x] `core/components/settings/mobile/nav.tsx`

#### `core/components/settings/profile/`

- [x] `core/components/settings/profile/heading.tsx`
- [x] `core/components/settings/profile/modal.tsx`

#### `core/components/settings/profile/content/`

- [x] `core/components/settings/profile/content/index.ts`
- [x] `core/components/settings/profile/content/root.tsx`

#### `core/components/settings/profile/content/pages/`

- [x] `core/components/settings/profile/content/pages/api-tokens.tsx`
- [x] `core/components/settings/profile/content/pages/index.ts`
- [x] `core/components/settings/profile/content/pages/security.tsx`

#### `core/components/settings/profile/content/pages/activity/`

- [!] `core/components/settings/profile/content/pages/activity/activity-list.tsx` — `.map((activityItem: any) => ...)` (línea 65) — réplica del patrón en `profile/activity/profile-activity-list.tsx`.
- [x] `core/components/settings/profile/content/pages/activity/index.ts`
- [!] `core/components/settings/profile/content/pages/activity/root.tsx` — `key={i}` (línea 50) sobre lista paginada (mismo patrón que en app/profile/activity). Aceptable hoy (sólo append) pero frágil — usar el cursor como key.

#### `core/components/settings/profile/content/pages/general/`

- [!] `core/components/settings/profile/content/pages/general/form.tsx` — `cover_image_asset: any` (línea 37).
- [x] `core/components/settings/profile/content/pages/general/index.ts`
- [x] `core/components/settings/profile/content/pages/general/root.tsx`

#### `core/components/settings/profile/content/pages/notifications/`

- [x] `core/components/settings/profile/content/pages/notifications/email-notification-form.tsx`
- [x] `core/components/settings/profile/content/pages/notifications/index.ts`
- [x] `core/components/settings/profile/content/pages/notifications/root.tsx`

#### `core/components/settings/profile/content/pages/preferences/`

- [x] `core/components/settings/profile/content/pages/preferences/default-list.tsx`
- [x] `core/components/settings/profile/content/pages/preferences/index.ts`
- [x] `core/components/settings/profile/content/pages/preferences/language-and-timezone-list.tsx`
- [x] `core/components/settings/profile/content/pages/preferences/root.tsx`

#### `core/components/settings/profile/sidebar/`

- [x] `core/components/settings/profile/sidebar/header.tsx`
- [x] `core/components/settings/profile/sidebar/index.ts`
- [x] `core/components/settings/profile/sidebar/item-categories.tsx`
- [x] `core/components/settings/profile/sidebar/root.tsx`
- [x] `core/components/settings/profile/sidebar/workspace-options.tsx`

#### `core/components/settings/project/content/`

- [x] `core/components/settings/project/content/feature-control-item.tsx`

#### `core/components/settings/project/sidebar/`

- [x] `core/components/settings/project/sidebar/header.tsx`
- [x] `core/components/settings/project/sidebar/index.ts`
- [x] `core/components/settings/project/sidebar/item-categories.tsx`
- [x] `core/components/settings/project/sidebar/item-icon.tsx`
- [x] `core/components/settings/project/sidebar/root.tsx`

#### `core/components/settings/sidebar/`

- [x] `core/components/settings/sidebar/item.tsx`

#### `core/components/settings/workspace/sidebar/`

- [x] `core/components/settings/workspace/sidebar/header.tsx`
- [x] `core/components/settings/workspace/sidebar/index.ts`
- [x] `core/components/settings/workspace/sidebar/item-categories.tsx`
- [x] `core/components/settings/workspace/sidebar/item-icon.tsx`
- [x] `core/components/settings/workspace/sidebar/root.tsx`

#### `core/components/sidebar/`

- [x] `core/components/sidebar/add-button.tsx`
- [!] `core/components/sidebar/resizable-sidebar.tsx` — Firma `= {} as any)` (línea 52) al default de props — reemplazar con default real tipado.
- [x] `core/components/sidebar/search-button.tsx`
- [x] `core/components/sidebar/sidebar-item.tsx`
- [x] `core/components/sidebar/sidebar-navigation.tsx`
- [x] `core/components/sidebar/sidebar-toggle-button.tsx`
- [x] `core/components/sidebar/sidebar-wrapper.tsx`

#### `core/components/stickies/`

- [x] `core/components/stickies/action-bar.tsx`
- [x] `core/components/stickies/delete-modal.tsx`
- [x] `core/components/stickies/widget.tsx`

#### `core/components/stickies/layout/`

- [x] `core/components/stickies/layout/stickies-infinite.tsx`
- [x] `core/components/stickies/layout/stickies-list.tsx`
- [x] `core/components/stickies/layout/stickies-loader.tsx`
- [x] `core/components/stickies/layout/stickies-truncated.tsx`
- [x] `core/components/stickies/layout/sticky-dnd-wrapper.tsx`
- [x] `core/components/stickies/layout/sticky.helpers.ts`

#### `core/components/stickies/modal/`

- [x] `core/components/stickies/modal/index.tsx`
- [x] `core/components/stickies/modal/search.tsx`
- [x] `core/components/stickies/modal/stickies.tsx`

#### `core/components/stickies/sticky/`

- [x] `core/components/stickies/sticky/index.ts`
- [x] `core/components/stickies/sticky/inputs.tsx`
- [x] `core/components/stickies/sticky/root.tsx`
- [x] `core/components/stickies/sticky/sticky-item-drag-handle.tsx`
- [x] `core/components/stickies/sticky/use-operations.tsx` — `catch (error: any)` → `unknown`.

#### `core/components/ui/`

- [!] `core/components/ui/empty-space.tsx` — 3× `any` en `children: any` (15), `Icon?: any` (16), `Icon: any` (53).
- [x] `core/components/ui/integration-and-import-export-banner.tsx`
- [x] `core/components/ui/labels-list.tsx`
- [!] `core/components/ui/markdown-to-component.tsx` — `options?: any` (línea 39).
- [!] `core/components/ui/profile-empty-state.tsx` — `image: any` (línea 12).

#### `core/components/ui/loader/`

- [x] `core/components/ui/loader/cycle-module-board-loader.tsx`
- [x] `core/components/ui/loader/cycle-module-list-loader.tsx`
- [x] `core/components/ui/loader/notification-loader.tsx`
- [x] `core/components/ui/loader/pages-loader.tsx`
- [x] `core/components/ui/loader/projects-loader.tsx`
- [x] `core/components/ui/loader/utils.tsx`
- [x] `core/components/ui/loader/view-list-loader.tsx`

#### `core/components/ui/loader/layouts/`

- [x] `core/components/ui/loader/layouts/calendar-layout-loader.tsx`
- [x] `core/components/ui/loader/layouts/gantt-layout-loader.tsx`
- [x] `core/components/ui/loader/layouts/kanban-layout-loader.tsx`
- [x] `core/components/ui/loader/layouts/list-layout-loader.tsx`
- [x] `core/components/ui/loader/layouts/members-layout-loader.tsx`
- [x] `core/components/ui/loader/layouts/spreadsheet-layout-loader.tsx`

#### `core/components/ui/loader/layouts/project-inbox/`

- [x] `core/components/ui/loader/layouts/project-inbox/inbox-layout-loader.tsx`
- [x] `core/components/ui/loader/layouts/project-inbox/inbox-sidebar-loader.tsx`

#### `core/components/ui/loader/settings/`

- [x] `core/components/ui/loader/settings/activity.tsx`
- [x] `core/components/ui/loader/settings/api-token.tsx`
- [x] `core/components/ui/loader/settings/email.tsx`
- [x] `core/components/ui/loader/settings/import-and-export.tsx`
- [x] `core/components/ui/loader/settings/integration.tsx`
- [x] `core/components/ui/loader/settings/members.tsx`
- [x] `core/components/ui/loader/settings/web-hook.tsx`

#### `core/components/user/`

- [x] `core/components/user/index.ts`
- [x] `core/components/user/user-greetings.tsx`

#### `core/components/views/`

- [x] `core/components/views/delete-view-modal.tsx`
- [x] `core/components/views/form.tsx` — `onChange={(val: any)}` → `TChangeHandlerProps`. Elimina TODO de tipado.
- [x] `core/components/views/helper.tsx`
- [x] `core/components/views/modal.tsx`
- [x] `core/components/views/quick-actions.tsx`
- [x] `core/components/views/view-list-header.tsx`
- [x] `core/components/views/view-list-item-action.tsx`
- [x] `core/components/views/view-list-item.tsx`
- [x] `core/components/views/views-list.tsx`

#### `core/components/views/applied-filters/`

- [x] `core/components/views/applied-filters/access.tsx`
- [x] `core/components/views/applied-filters/index.tsx`
- [x] `core/components/views/applied-filters/root.tsx`

#### `core/components/views/filters/`

- [x] `core/components/views/filters/filter-selection.tsx`
- [x] `core/components/views/filters/order-by.tsx`

#### `core/components/web-hooks/`

- [x] `core/components/web-hooks/create-webhook-modal.tsx`
- [x] `core/components/web-hooks/delete-webhook-modal.tsx`
- [x] `core/components/web-hooks/empty-state.tsx`
- [x] `core/components/web-hooks/generated-hook-details.tsx`
- [x] `core/components/web-hooks/index.ts`
- [x] `core/components/web-hooks/utils.ts`
- [x] `core/components/web-hooks/webhooks-list-item.tsx`
- [x] `core/components/web-hooks/webhooks-list.tsx`

#### `core/components/web-hooks/form/`

- [x] `core/components/web-hooks/form/delete-section.tsx`
- [x] `core/components/web-hooks/form/event-types.tsx`
- [x] `core/components/web-hooks/form/form.tsx`
- [x] `core/components/web-hooks/form/index.ts`
- [x] `core/components/web-hooks/form/individual-event-options.tsx`
- [x] `core/components/web-hooks/form/input.tsx`
- [!] `core/components/web-hooks/form/secret-key.tsx` — `key={index}` (línea 118) sobre un array de puntos decorativos — aceptable visualmente pero inconsistente con el resto del codebase; preferir un id estable o fragment.
- [x] `core/components/web-hooks/form/toggle.tsx`

#### `core/components/work-item-filters/`

- [x] `core/components/work-item-filters/filters-row.tsx`
- [x] `core/components/work-item-filters/filters-toggle.tsx`

#### `core/components/work-item-filters/filters-hoc/`

- [x] `core/components/work-item-filters/filters-hoc/base.tsx`
- [x] `core/components/work-item-filters/filters-hoc/project-level.tsx`
- [x] `core/components/work-item-filters/filters-hoc/shared.ts`
- [x] `core/components/work-item-filters/filters-hoc/workspace-level.tsx`

#### `core/components/workspace/`

- [x] `core/components/workspace/ConfirmWorkspaceMemberRemove.tsx`
- [x] `core/components/workspace/confirm-workspace-member-remove.tsx`
- [x] `core/components/workspace/create-workspace-form.tsx`
- [x] `core/components/workspace/delete-workspace-form.tsx`
- [x] `core/components/workspace/logo.tsx`

#### `core/components/workspace/billing/comparison/`

- [x] `core/components/workspace/billing/comparison/base.tsx`
- [x] `core/components/workspace/billing/comparison/feature-detail.tsx`
- [x] `core/components/workspace/billing/comparison/index.ts`

#### `core/components/workspace/invite-modal/`

- [x] `core/components/workspace/invite-modal/actions.tsx`
- [x] `core/components/workspace/invite-modal/fields.tsx`
- [x] `core/components/workspace/invite-modal/form.tsx`

#### `core/components/workspace/settings/`

- [x] `core/components/workspace/settings/invitations-list-item.tsx`
- [x] `core/components/workspace/settings/member-columns.tsx`
- [x] `core/components/workspace/settings/members-list-item.tsx`
- [x] `core/components/workspace/settings/members-list.tsx`
- [x] `core/components/workspace/settings/workspace-details.tsx`

#### `core/components/workspace/sidebar/`

- [x] `core/components/workspace/sidebar/dropdown-item.tsx`
- [x] `core/components/workspace/sidebar/project-navigation.tsx`
- [x] `core/components/workspace/sidebar/projects-list-item.tsx`
- [x] `core/components/workspace/sidebar/projects-list.tsx` — Migrado de `localStorage.setItem` directo al hook `useLocalStorage` para consistencia + SSR safety.
- [x] `core/components/workspace/sidebar/quick-actions.tsx`
- [x] `core/components/workspace/sidebar/sidebar-item.tsx`
- [x] `core/components/workspace/sidebar/sidebar-menu-items.tsx`
- [!] `core/components/workspace/sidebar/user-menu-item.tsx` — (1) Cast `item.access as any` (línea 49) — misma deuda. (2) `Icon: any` (línea 28).
- [x] `core/components/workspace/sidebar/user-menu-root.tsx`
- [x] `core/components/workspace/sidebar/user-menu.tsx`
- [!] `core/components/workspace/sidebar/workspace-menu-header.tsx` — Cast `[EUserWorkspaceRoles.ADMIN] as any` (línea 46) en `allowPermissions`. Arreglar la signature de `allowPermissions` para aceptar `EUserWorkspaceRoles[]`.
- [!] `core/components/workspace/sidebar/workspace-menu-item.tsx` — (1) Cast `item.access as any` (línea 52) en `allowPermissions` — replica del patrón en `ce/components/workspace/sidebar/extended-sidebar-item.tsx`. (2) `Icon: any` (línea 28) en el tipo del prop.
- [x] `core/components/workspace/sidebar/workspace-menu-root.tsx`
- [x] `core/components/workspace/sidebar/workspace-menu.tsx`

#### `core/components/workspace/sidebar/favorites/`

- [x] `core/components/workspace/sidebar/favorites/favorite-folder.tsx`
- [x] `core/components/workspace/sidebar/favorites/favorites-menu.tsx` — debug `console.log({ sourceId })` eliminado.
- [x] `core/components/workspace/sidebar/favorites/favorites.helpers.ts`
- [x] `core/components/workspace/sidebar/favorites/new-fav-folder.tsx`

#### `core/components/workspace/sidebar/favorites/favorite-items/`

- [x] `core/components/workspace/sidebar/favorites/favorite-items/index.ts`
- [x] `core/components/workspace/sidebar/favorites/favorite-items/root.tsx`

#### `core/components/workspace/sidebar/favorites/favorite-items/common/`

- [x] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-drag-handle.tsx`
- [x] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-quick-action.tsx`
- [x] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-title.tsx`
- [x] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-wrapper.tsx`
- [x] `core/components/workspace/sidebar/favorites/favorite-items/common/helper.tsx`
- [x] `core/components/workspace/sidebar/favorites/favorite-items/common/index.ts`

#### `core/components/workspace/sidebar/help-section/`

- [x] `core/components/workspace/sidebar/help-section/index.ts`
- [x] `core/components/workspace/sidebar/help-section/root.tsx`

#### `core/components/workspace/views/`

- [x] `core/components/workspace/views/default-view-list-item.tsx`
- [x] `core/components/workspace/views/default-view-quick-action.tsx`
- [x] `core/components/workspace/views/delete-view-modal.tsx` — `localStorage.removeItem` envuelto con `typeof window !== "undefined"`.
- [x] `core/components/workspace/views/form.tsx`
- [x] `core/components/workspace/views/header.tsx`
- [x] `core/components/workspace/views/modal.tsx`
- [x] `core/components/workspace/views/quick-action.tsx`
- [x] `core/components/workspace/views/view-list-item.tsx`
- [x] `core/components/workspace/views/views-list.tsx`

#### `core/components/workspace-notifications/`

- [x] `core/components/workspace-notifications/index.ts`
- [x] `core/components/workspace-notifications/notification-app-sidebar-option.tsx`
- [x] `core/components/workspace-notifications/root.tsx`

#### `core/components/workspace-notifications/sidebar/`

- [x] `core/components/workspace-notifications/sidebar/empty-state.tsx`
- [x] `core/components/workspace-notifications/sidebar/index.ts`
- [!] `core/components/workspace-notifications/sidebar/loader.tsx` — `key={i}` (línea 13) en un loop de skeletons — aceptable para UI placeholder (no hay datos), documentar o usar constante.
- [x] `core/components/workspace-notifications/sidebar/root.tsx`

#### `core/components/workspace-notifications/sidebar/filters/`

- [x] `core/components/workspace-notifications/sidebar/filters/applied-filter.tsx`

#### `core/components/workspace-notifications/sidebar/filters/menu/`

- [x] `core/components/workspace-notifications/sidebar/filters/menu/index.ts`
- [x] `core/components/workspace-notifications/sidebar/filters/menu/menu-option-item.tsx`
- [x] `core/components/workspace-notifications/sidebar/filters/menu/root.tsx`

#### `core/components/workspace-notifications/sidebar/header/`

- [x] `core/components/workspace-notifications/sidebar/header/index.ts`
- [x] `core/components/workspace-notifications/sidebar/header/root.tsx`

#### `core/components/workspace-notifications/sidebar/header/options/`

- [x] `core/components/workspace-notifications/sidebar/header/options/index.ts`
- [x] `core/components/workspace-notifications/sidebar/header/options/root.tsx`

#### `core/components/workspace-notifications/sidebar/header/options/menu-option/`

- [x] `core/components/workspace-notifications/sidebar/header/options/menu-option/index.ts`
- [x] `core/components/workspace-notifications/sidebar/header/options/menu-option/menu-item.tsx`
- [x] `core/components/workspace-notifications/sidebar/header/options/menu-option/root.tsx`

#### `core/components/workspace-notifications/sidebar/notification-card/`

- [x] `core/components/workspace-notifications/sidebar/notification-card/content.tsx`
- [x] `core/components/workspace-notifications/sidebar/notification-card/item.tsx`

#### `core/components/workspace-notifications/sidebar/notification-card/options/`

- [x] `core/components/workspace-notifications/sidebar/notification-card/options/archive.tsx`
- [x] `core/components/workspace-notifications/sidebar/notification-card/options/button.tsx`
- [x] `core/components/workspace-notifications/sidebar/notification-card/options/index.ts`
- [x] `core/components/workspace-notifications/sidebar/notification-card/options/read.tsx`
- [x] `core/components/workspace-notifications/sidebar/notification-card/options/root.tsx`

#### `core/components/workspace-notifications/sidebar/notification-card/options/snooze/`

- [x] `core/components/workspace-notifications/sidebar/notification-card/options/snooze/index.ts`
- [x] `core/components/workspace-notifications/sidebar/notification-card/options/snooze/modal.tsx`
- [x] `core/components/workspace-notifications/sidebar/notification-card/options/snooze/root.tsx`

#### `core/constants/`

- [x] `core/constants/ai.ts`
- [x] `core/constants/calendar.ts`
- [!] `core/constants/editor.ts` — `icon: any` (línea 210) en tipo/struct de constantes — tipar como `LucideIcon | React.ComponentType<...>`.
- [!] `core/constants/fetch-keys.ts` — `paramsToKey(params: any)` (línea 9) + `CYCLE_ISSUES_WITH_PARAMS(..., params?: any)` (línea 98) — tipar como `Record<string, unknown>` o el shape real.
- [x] `core/constants/gantt-chart.ts`
- [x] `core/constants/plans.tsx`
- [x] `core/constants/sidebar-favorites.ts`

#### `core/custom-events/`

- [x] `core/custom-events/chat-support.ts`

#### `core/hooks/`

- [x] `core/hooks/use-app-router.tsx`
- [x] `core/hooks/use-auto-save.tsx`
- [x] `core/hooks/use-auto-scroller.tsx`
- [x] `core/hooks/use-chat-support.ts`
- [x] `core/hooks/use-collaborative-page-actions.tsx`
- [x] `core/hooks/use-current-time.tsx`
- [x] `core/hooks/use-debounce.tsx`
- [x] `core/hooks/use-dropdown-key-down.tsx`
- [x] `core/hooks/use-dropdown.ts`
- [x] `core/hooks/use-expandable-search.ts`
- [x] `core/hooks/use-extended-sidebar-overview-outside-click.tsx`
- [x] `core/hooks/use-favorite-item-details.tsx`
- [x] `core/hooks/use-group-dragndrop.ts`
- [x] `core/hooks/use-integration-popup.tsx`
- [x] `core/hooks/use-intersection-observer.ts`
- [x] `core/hooks/use-issue-layout-store.ts`
- [x] `core/hooks/use-issue-peek-overview-redirection.tsx`
- [x] `core/hooks/use-issues-actions.tsx`
- [x] `core/hooks/use-keypress.tsx`
- [x] `core/hooks/use-local-storage.tsx` — `getValueFromLocalStorage` y `setValueIntoLocalStorage` convertidos a genéricos `<T>` eliminando `any` en firmas.
- [x] `core/hooks/use-multiple-select.ts` — debug `console.log("force adding")` eliminado.
- [x] `core/hooks/use-navigation-preferences.ts`
- [x] `core/hooks/use-online-status.ts`
- [x] `core/hooks/use-page-fallback.ts` — `catch (error: any)` → `unknown`.
- [x] `core/hooks/use-page-filters.ts`
- [x] `core/hooks/use-page-operations.ts`
- [x] `core/hooks/use-parse-editor-content.ts`
- [x] `core/hooks/use-peek-overview-outside-click.tsx`
- [x] `core/hooks/use-platform-os.tsx`
- [x] `core/hooks/use-project-issue-properties.ts`
- [x] `core/hooks/use-query-params.ts`
- [x] `core/hooks/use-realtime-page-events.tsx`
- [x] `core/hooks/use-reload-confirmation.tsx`
- [x] `core/hooks/use-stickies.tsx`
- [x] `core/hooks/use-table-keyboard-navigation.tsx`
- [x] `core/hooks/use-timeline-chart.ts`
- [x] `core/hooks/use-timer.tsx`
- [x] `core/hooks/use-timezone-converter.tsx`
- [x] `core/hooks/use-timezone.tsx`
- [x] `core/hooks/use-window-size.tsx`
- [x] `core/hooks/use-workspace-invitation.tsx`
- [x] `core/hooks/use-workspace-issue-properties.ts`
- [x] `core/hooks/use-workspace-paths.ts`

#### `core/hooks/context/`

- [x] `core/hooks/context/use-issue-modal.tsx`

#### `core/hooks/editor/`

- [x] `core/hooks/editor/index.ts`
- [x] `core/hooks/editor/use-editor-config.ts`
- [x] `core/hooks/editor/use-editor-mention.tsx`

#### `core/hooks/oauth/`

- [x] `core/hooks/oauth/core.tsx`
- [x] `core/hooks/oauth/extended.tsx`
- [x] `core/hooks/oauth/index.ts`

#### `core/hooks/store/`

- [x] `core/hooks/store/use-analytics.ts`
- [x] `core/hooks/store/use-app-theme.ts`
- [x] `core/hooks/store/use-calendar-view.ts`
- [x] `core/hooks/store/use-command-palette.ts`
- [x] `core/hooks/store/use-cycle-filter.ts`
- [x] `core/hooks/store/use-cycle.ts`
- [x] `core/hooks/store/use-dashboard.ts`
- [x] `core/hooks/store/use-editor-asset.ts`
- [x] `core/hooks/store/use-favorite.ts`
- [x] `core/hooks/store/use-global-view.ts`
- [x] `core/hooks/store/use-home.ts`
- [x] `core/hooks/store/use-inbox-issues.ts`
- [x] `core/hooks/store/use-instance.ts`
- [x] `core/hooks/store/use-issue-detail.ts`
- [x] `core/hooks/store/use-issues.ts`
- [x] `core/hooks/store/use-kanban-view.ts`
- [x] `core/hooks/store/use-label.ts`
- [x] `core/hooks/store/use-member.ts`
- [x] `core/hooks/store/use-module-filter.ts`
- [x] `core/hooks/store/use-module.ts`
- [x] `core/hooks/store/use-multiple-select-store.ts`
- [x] `core/hooks/store/use-power-k.ts`
- [x] `core/hooks/store/use-project-filter.ts`
- [x] `core/hooks/store/use-project-inbox.ts`
- [x] `core/hooks/store/use-project-publish.ts`
- [x] `core/hooks/store/use-project-state.ts`
- [x] `core/hooks/store/use-project-view.ts`
- [x] `core/hooks/store/use-project.ts`
- [x] `core/hooks/store/use-router-params.ts`
- [x] `core/hooks/store/use-webhook.ts`
- [x] `core/hooks/store/use-workspace.ts`

#### `core/hooks/store/estimates/`

- [x] `core/hooks/store/estimates/index.ts`
- [x] `core/hooks/store/estimates/use-estimate-point.ts`
- [x] `core/hooks/store/estimates/use-estimate.ts`
- [x] `core/hooks/store/estimates/use-project-estimate.ts`

#### `core/hooks/store/notifications/`

- [x] `core/hooks/store/notifications/index.ts`
- [x] `core/hooks/store/notifications/use-notification.ts`
- [x] `core/hooks/store/notifications/use-workspace-notifications.ts`

#### `core/hooks/store/user/`

- [x] `core/hooks/store/user/index.ts`
- [x] `core/hooks/store/user/user-permissions.ts`
- [x] `core/hooks/store/user/user-user-profile.ts`
- [x] `core/hooks/store/user/user-user-settings.ts`
- [x] `core/hooks/store/user/user-user.ts`

#### `core/hooks/store/work-item-filters/`

- [x] `core/hooks/store/work-item-filters/use-work-item-filter-instance.ts`
- [x] `core/hooks/store/work-item-filters/use-work-item-filters.ts`

#### `core/hooks/store/workspace-draft/`

- [x] `core/hooks/store/workspace-draft/index.ts`
- [x] `core/hooks/store/workspace-draft/use-workspace-draft-issue-filters.ts`
- [x] `core/hooks/store/workspace-draft/use-workspace-draft-issue.ts`

#### `core/layouts/auth-layout/`

- [x] `core/layouts/auth-layout/project-wrapper.tsx`
- [x] `core/layouts/auth-layout/workspace-wrapper.tsx`

#### `core/layouts/default-layout/`

- [x] `core/layouts/default-layout/index.tsx`

#### `core/lib/`

- [x] `core/lib/local-storage.ts`
- [x] `core/lib/store-context.tsx`

#### `core/lib/app-rail/`

- [x] `core/lib/app-rail/context.tsx`
- [x] `core/lib/app-rail/index.ts`
- [x] `core/lib/app-rail/provider.tsx`
- [x] `core/lib/app-rail/types.ts`

#### `core/lib/b-progress/`

- [x] `core/lib/b-progress/AppProgressBar.tsx`

#### `core/lib/polyfills/`

- [x] `core/lib/polyfills/index.ts`

#### `core/lib/wrappers/`

- [x] `core/lib/wrappers/authentication-wrapper.tsx`
- [x] `core/lib/wrappers/instance-wrapper.tsx`
- [x] `core/lib/wrappers/store-wrapper.tsx`

#### `core/services/`

- [x] `core/services/ai.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/analytics.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/api.service.ts`
- [x] `core/services/app_config.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/app_installation.service.ts`
- [x] `core/services/auth.service.ts` — Raw rethrow → `ApiError`/`toApiError`. DOM manipulation en `signOut` (document.createElement+appendChild) sin guard SSR sigue documentada — aceptable porque corre post-click en browser con reload inmediato.
- [x] `core/services/cycle.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/cycle_archive.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/dashboard.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/estimate.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/file-upload.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/file.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/instance.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/issue_filter.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/module.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/module_archive.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/sticky.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/timezone.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/user.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/view.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/webhook.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/workspace-notification.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.
- [x] `core/services/workspace.service.ts` — Migrado a `ApiError`: `throw error?.response(?.data)` → `throw new ApiError(error?.response)`. Bare `throw error` → `throw toApiError(error)`.

#### `core/services/favorite/`

- [x] `core/services/favorite/favorite.service.ts` — Migrado a `ApiError`.
- [x] `core/services/favorite/index.ts`

#### `core/services/inbox/`

- [x] `core/services/inbox/inbox-issue.service.ts` — Migrado a `ApiError`.
- [x] `core/services/inbox/index.ts`
- [x] `core/services/inbox/intake-work_item_version.service.ts` — Migrado a `ApiError`.

#### `core/services/integrations/`

- [x] `core/services/integrations/github-user-connection.service.ts`
- [x] `core/services/integrations/github.service.ts`
- [x] `core/services/integrations/gitlab.service.ts`
- [x] `core/services/integrations/index.ts`
- [x] `core/services/integrations/integration.service.ts`
- [x] `core/services/integrations/jira.service.ts`

#### `core/services/issue/`

- [x] `core/services/issue/index.ts`
- [x] `core/services/issue/issue.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_activity.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_archive.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_attachment.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_comment.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_label.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_reaction.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/issue_relation.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/work_item_version.service.ts` — Migrado a `ApiError`.
- [x] `core/services/issue/workspace_draft.service.ts` — Migrado a `ApiError`.

#### `core/services/page/`

- [x] `core/services/page/index.ts`
- [x] `core/services/page/project-page-version.service.ts` — Migrado a `ApiError`.
- [x] `core/services/page/project-page.service.ts` — Migrado a `ApiError`.

#### `core/services/project/`

- [x] `core/services/project/index.ts`
- [x] `core/services/project/project-archive.service.ts` — Migrado a `ApiError`.
- [x] `core/services/project/project-export.service.ts` — Migrado a `ApiError`.
- [x] `core/services/project/project-member.service.ts` — Migrado a `ApiError`.
- [x] `core/services/project/project-publish.service.ts` — Migrado a `ApiError`.
- [x] `core/services/project/project-state.service.ts` — Migrado a `ApiError`.
- [x] `core/services/project/project.service.ts` — Migrado a `ApiError`.

#### `core/store/`

- [!] `core/store/analytics.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/base-command-palette.store.ts`
- [x] `core/store/base-power-k.store.ts`
- [x] `core/store/cycle.store.ts` — `console.log` → `console.error`. eslint-disable y raw rethrow pendientes.
- [x] `core/store/cycle_filter.store.ts`
- [!] `core/store/dashboard.store.ts` — Raw error rethrow + `as unknown as T` en línea 140 (type laundering en widgetStats) + `: any` inferido en callback `.then((res: any) =>` línea 186.
- [!] `core/store/favorite.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/global-view.store.ts` — Raw error rethrow + `: any` en anotaciones.
- [!] `core/store/instance.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/label.store.ts` — `console.log` → `console.error`. Raw rethrow pendiente.
- [!] `core/store/module.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/module_filter.store.ts`
- [x] `core/store/multiple_select.store.ts`
- [!] `core/store/project-view.store.ts` — `: any` en anotaciones.
- [!] `core/store/root.store.ts` — 12× `as unknown as RootStore` (type laundering por arquitectura modular CE/EE) + 2× `localStorage.setItem` en `resetOnSignOut` sin guard SSR (aceptable por contexto de uso client-only)
- [x] `core/store/router.store.ts`
- [!] `core/store/state.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/theme.store.ts` — 9× `localStorage.setItem` wrapped con `if (typeof window !== "undefined")` para SSR safety.

#### `core/store/editor/`

- [!] `core/store/editor/asset.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/estimates/`

- [!] `core/store/estimates/estimate-point.ts` — `eslint-disable` (revisar justificación) + raw error rethrow.
- [!] `core/store/estimates/project-estimate.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/inbox/`

- [x] `core/store/inbox/inbox-issue.store.ts`
- [!] `core/store/inbox/project-inbox.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/`

- [x] `core/store/issue/issue.store.ts`
- [!] `core/store/issue/issue_calendar_view.store.ts` — `eslint-disable` + `: any`.
- [x] `core/store/issue/issue_gantt_view.store.ts` — Mismo refactor que `ce/store/timeline/base-timeline.store.ts`: `renderView: any[]` → `TGanttRenderPayload` (union local: `IWeekBlock[] | IMonthView | IMonthBlock[]`); `updateRenderView(data: any[])` → `updateRenderView(data: TGanttRenderPayload)`. El tipo se define localmente (no re-exportado desde `ce/`) para no invertir la dirección de dependencia core→ce.
- [x] `core/store/issue/issue_kanban_view.store.ts`
- [x] `core/store/issue/root.store.ts`

#### `core/store/issue/archived/`

- [x] `core/store/issue/archived/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/archived/index.ts`
- [!] `core/store/issue/archived/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/cycle/`

- [x] `core/store/issue/cycle/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/cycle/index.ts`
- [!] `core/store/issue/cycle/issue.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).

#### `core/store/issue/helpers/`

- [!] `core/store/issue/helpers/base-issues-utils.ts` — `: any` + `console.log` (detectado en scan previo). 2× non-null assertions (`!.`).
- [!] `core/store/issue/helpers/base-issues.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/issue/helpers/issue-filter-helper.store.ts`

#### `core/store/issue/issue-details/`

- [!] `core/store/issue/issue-details/attachment.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/issue/issue-details/comment.store.ts` — Raw error rethrow + `: any`.
- [x] `core/store/issue/issue-details/comment_reaction.store.ts` — 2× `console.log` → `console.error`. `: any` pendiente.
- [x] `core/store/issue/issue-details/issue.store.ts`
- [!] `core/store/issue/issue-details/link.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/issue/issue-details/reaction.store.ts` — `: any` en anotaciones.
- [!] `core/store/issue/issue-details/relation.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/issue/issue-details/root.store.ts`
- [x] `core/store/issue/issue-details/sub_issues.store.ts`
- [x] `core/store/issue/issue-details/sub_issues_filter.store.ts`
- [!] `core/store/issue/issue-details/subscription.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/module/`

- [x] `core/store/issue/module/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/module/index.ts`
- [!] `core/store/issue/module/issue.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).

#### `core/store/issue/profile/`

- [x] `core/store/issue/profile/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/profile/index.ts`
- [!] `core/store/issue/profile/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/project/`

- [x] `core/store/issue/project/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/project/index.ts`
- [!] `core/store/issue/project/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/project-views/`

- [x] `core/store/issue/project-views/filter.store.ts` — 2× `console.log` → `console.error`.
- [x] `core/store/issue/project-views/index.ts`
- [!] `core/store/issue/project-views/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/workspace/`

- [x] `core/store/issue/workspace/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/workspace/index.ts`
- [!] `core/store/issue/workspace/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/workspace-draft/`

- [x] `core/store/issue/workspace-draft/filter.store.ts` — `console.log` → `console.error`.
- [x] `core/store/issue/workspace-draft/index.ts`
- [!] `core/store/issue/workspace-draft/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/member/`

- [x] `core/store/member/index.ts`
- [x] `core/store/member/utils.ts`

#### `core/store/member/project/`

- [!] `core/store/member/project/base-project-member.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/member/project/project-member-filters.store.ts`

#### `core/store/member/workspace/`

- [x] `core/store/member/workspace/workspace-member-filters.store.ts`
- [!] `core/store/member/workspace/workspace-member.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/notifications/`

- [!] `core/store/notifications/notification.ts` — `eslint-disable` + raw error rethrow.
- [!] `core/store/notifications/workspace-notifications.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/pages/`

- [!] `core/store/pages/base-page.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/pages/page-editor-info.ts`
- [!] `core/store/pages/project-page.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/pages/project-page.ts`

#### `core/store/project/`

- [x] `core/store/project/index.ts`
- [!] `core/store/project/project-publish.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/project/project.store.ts` — (1) 11× `console.log(...)` → `console.error(..., error)` con el `error` pasado en todos (antes se perdía en la mayoría) y mensajes copy-paste duplicados corregidos (`removeProjectFromFavorites` decía "Failed to add"; `updateProject` decía "Failed to create"). (2) Interface: `addProjectToFavorites` → `Promise<IFavorite | undefined>`; `removeProjectFromFavorites` → `Promise<void>`; `updateProjectView(viewProps: any) => Promise<any>` → `(viewProps: { sort_order: number }) => Promise<IProjectUserPropertiesResponse>`. (3) Impl `createProject(data: any)` → `data: Partial<TProject>` (alineado con la interface). Raw error rethrows se mantienen como legacy consistente con el resto de stores (refactor separado a `ApiError` en tanda dedicada).
- [x] `core/store/project/project_filter.store.ts`

#### `core/store/sticky/`

- [x] `core/store/sticky/sticky.store.ts` — (1) 2× `console.error(e)` sin contexto → `console.error("Failed to fetch ...", e)` con mensaje identificativo. (2) `console.log(e)` en `deleteSticky` → `console.error("Failed to delete sticky", e)`. (3) `console.error("Failed to move sticky")` en `updateStickyPosition` ahora pasa `error` como segundo arg. (4) **Bug sustantivo**: `throw new Error("", { cause: error })` en `updateSticky` → `throw new Error("Failed to update sticky", { cause: error })`. Un mensaje top-level vacío rompe toasts UI (muestran string vacío) y filtrado en aggregadores — el `cause` preserva la original.

#### `core/store/timeline/`

- [x] `core/store/timeline/issues-timeline.store.ts`
- [x] `core/store/timeline/modules-timeline.store.ts`

#### `core/store/user/`

- [!] `core/store/user/account.store.ts` — `: any` en anotaciones.
- [!] `core/store/user/base-permissions.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/user/index.ts` — `console.log(error)` en `changePassword` → `console.error("Failed to change password from user store", error)` (mensaje contextual, nivel de log correcto). Raw rethrows se mantienen como legacy consistente con el resto de stores.
- [!] `core/store/user/profile.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/user/settings.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/workspace/`

- [x] `core/store/workspace/api-token.store.ts`
- [!] `core/store/workspace/home.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/workspace/index.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/workspace/link.store.ts`
- [x] `core/store/workspace/webhook.store.ts`

#### `core/types/`

- [x] `core/types/navigation-preferences.ts`

## `helpers/` — 8 archivos

### `helpers/`

- [x] `helpers/authentication.helper.tsx`
- [x] `helpers/cover-image.helper.ts`
- [x] `helpers/dashboard.helper.ts`
- [x] `helpers/emoji.helper.tsx`
- [x] `helpers/graph.helper.ts`
- [x] `helpers/issue-filter.helper.ts`
- [x] `helpers/react-hook-form.helper.ts`
- [x] `helpers/views.helper.ts`

## `./` — 2 archivos

#### `./`

- [x] `react-router.config.ts`
- [x] `vite.config.ts`

## Declaration files (`.d.ts`) — 11 archivos

Archivos de declaración de tipos (sin código runtime). Revisión enfocada en: `any` injustificado, shims demasiado permisivos, index signatures laxos.

### `app/types/`

- [x] `app/types/next-link.d.ts`
- [x] `app/types/next-navigation.d.ts`
- [!] `app/types/next-script.d.ts` — `[key: string]: any;` en `ScriptProps` abre el tipo a cualquier prop. Justificable para shim de compat pero relaja la seguridad de tipos para todos los consumidores de `next/script`. Considerar enumerar props concretas que se usan (o al menos `unknown` en vez de `any`).
- [!] `app/types/react-router-virtual.d.ts` — `const build: any;` con `eslint-disable`. Shim necesario para módulo virtual de react-router pero elimina toda seguridad de tipos en el build handler. Documentado con comentario, aceptable como deuda técnica conocida.

### `ce/types/issue-types/`

- [x] `ce/types/issue-types/issue-property-values.d.ts`

### `core/components/dropdowns/`

- [x] `core/components/dropdowns/types.d.ts`
- [x] `core/components/dropdowns/member/types.d.ts`

### `core/components/icons/`

- [x] `core/components/icons/types.d.ts`

### `core/components/issues/issue-layouts/list/`

- [x] `core/components/issues/issue-layouts/list/list-view-types.d.ts`

### `./`

- [!] `google.d.ts` — `(...args: any[]) => void` en `native_callback` e `intermediate_iframe_close_callback` de `IdConfiguration`. Son callbacks de Google Identity Services donde los args dependen de la lib externa; aceptable como shim pero idealmente `(...args: unknown[]) => void` con narrowing en el callsite.
- [x] `use-font-face-observer.d.ts`

