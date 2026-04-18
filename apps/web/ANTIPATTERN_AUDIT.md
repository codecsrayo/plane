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
- [!] `app/layout.tsx` — Clarity `<Script>` inyecta `${process.env.VITE_SESSION_RECORDER_KEY}` sin escapar dentro de JS inline. Aunque el valor viene de build-env, es defensa-en-profundidad: usar `JSON.stringify(process.env.VITE_SESSION_RECORDER_KEY)` para neutralizar breakouts. Además archivo duplica `<head>`/meta con `app/root.tsx` (es un layout legacy de Next.js).
- [x] `app/not-found.tsx`
- [x] `app/provider.tsx`
- [!] `app/root.tsx` — Mismo issue Clarity `<Script>` (interpolación de env sin `JSON.stringify`). `parseInt(process.env.VITE_ENABLE_SESSION_RECORDER || "0")` sin radix explícito. `HydrateFallback` documenta bien paridad SSR/CSR.
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
- [!] `app/(all)/[workspaceSlug]/(projects)/analytics/[tabId]/page.tsx` — Antipatrón "mirror props in state": `selectedTab` es `useState` derivable directamente de `tabId` (param URL). El `useEffect` (líneas 57-61) que sincroniza `tabId → selectedTab` es innecesario y abre ventana de re-render desfasado con el URL (frame antes del efecto). Refactor: eliminar `useState` + `useEffect` y usar `const selectedTab = tabId || ANALYTICS_TABS[0]?.key` directamente. Derivación pura de la URL (source of truth).

#### `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/layout.tsx`
- [!] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/page.tsx` — Guards `if (window && ...)` redundantes dentro de `useEffect` (el efecto solo corre client-side; `window` siempre está definido). No es bug, es código defensivo innecesario — limpiar para mantener claridad. Listener de resize bien registrado y limpiado.
- [ ] `app/(all)/[workspaceSlug]/(projects)/browse/[workItem]/work-item-header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/drafts/`

- [x] `app/(all)/[workspaceSlug]/(projects)/drafts/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/drafts/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/drafts/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/notifications/`

- [x] `app/(all)/[workspaceSlug]/(projects)/notifications/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/notifications/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/mobile-header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/navbar.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/[profileViewId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/[profileViewId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/activity/`

- [!] `app/(all)/[workspaceSlug]/(projects)/profile/[userId]/activity/page.tsx` — `key={i}` (index como key, línea 42) sobre `WorkspaceActivityListPage` en loop paginado. Aceptable hoy porque las páginas solo se appendan (nunca reordenan/eliminan), pero frágil: si el cursor/paginación cambia, React reusará componentes equivocados. Mejor usar `key={\`\${PER_PAGE}:\${i}:0\`}` (el cursor mismo, que es estable y único).

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/cycles/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/cycles/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/cycles/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/[archivedIssueId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(detail)/[archivedIssueId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(list)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/issues/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/modules/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/modules/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/archives/modules/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/mobile-header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/[cycleId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(detail)/[cycleId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/mobile-header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/cycles/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/intake/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/intake/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/intake/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(detail)/[issueId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(detail)/[issueId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/mobile-header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/issues/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/mobile-header.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/[moduleId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(detail)/[moduleId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/mobile-header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/modules/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/[pageId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(detail)/[pageId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/`

- [!] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/header.tsx` — `catch (err: any)` (línea 60) tipado como `any` en lugar de `unknown` + narrowing. Deuda técnica menor consistente con el resto del código legacy.
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/pages/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/[viewId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/[viewId]/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(detail)/[viewId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/mobile-header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/[projectId]/views/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/archives/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/archives/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(detail)/archives/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/projects/(list)/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(list)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/projects/(list)/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/stickies/`

- [x] `app/(all)/[workspaceSlug]/(projects)/stickies/header.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/stickies/layout.tsx`
- [x] `app/(all)/[workspaceSlug]/(projects)/stickies/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/workspace-views/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/workspace-views/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/workspace-views/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(projects)/workspace-views/page.tsx`

#### `app/(all)/[workspaceSlug]/(projects)/workspace-views/[globalViewId]/`

- [ ] `app/(all)/[workspaceSlug]/(projects)/workspace-views/[globalViewId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/layout.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/billing/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/billing/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/billing/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/exports/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/exports/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/exports/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/[provider]/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/integrations/[provider]/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/members/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/members/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/members/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/[webhookId]/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/[webhookId]/header.tsx`
- [!] `app/(all)/[workspaceSlug]/(settings)/settings/(workspace)/webhooks/[webhookId]/page.tsx` — `catch (error: any)` (línea 71) tipado como `any` (con `eslint-disable` comentado). Preferir `catch (error: unknown)` y narrowing para acceder a `error.error`.

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/layout.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/automations/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/estimates/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/estimates/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/estimates/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/cycles/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/cycles/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/cycles/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/intake/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/intake/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/intake/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/modules/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/modules/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/modules/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/pages/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/pages/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/pages/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/views/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/views/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/features/views/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/labels/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/labels/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/labels/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/members/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/members/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/members/page.tsx`

#### `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/states/`

- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/states/header.tsx`
- [ ] `app/(all)/[workspaceSlug]/(settings)/settings/projects/[projectId]/states/page.tsx`

#### `app/(all)/accounts/forgot-password/`

- [ ] `app/(all)/accounts/forgot-password/layout.tsx`
- [ ] `app/(all)/accounts/forgot-password/page.tsx`

#### `app/(all)/accounts/reset-password/`

- [ ] `app/(all)/accounts/reset-password/layout.tsx`
- [ ] `app/(all)/accounts/reset-password/page.tsx`

#### `app/(all)/accounts/set-password/`

- [ ] `app/(all)/accounts/set-password/layout.tsx`
- [ ] `app/(all)/accounts/set-password/page.tsx`

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

- [ ] `app/(all)/create-workspace/layout.tsx`
- [ ] `app/(all)/create-workspace/page.tsx`

#### `app/(all)/invitations/`

- [ ] `app/(all)/invitations/layout.tsx`
- [ ] `app/(all)/invitations/page.tsx`

#### `app/(all)/onboarding/`

- [ ] `app/(all)/onboarding/layout.tsx`
- [ ] `app/(all)/onboarding/page.tsx`

#### `app/(all)/settings/profile/`

- [ ] `app/(all)/settings/profile/layout.tsx`

#### `app/(all)/settings/profile/[profileTabId]/`

- [ ] `app/(all)/settings/profile/[profileTabId]/page.tsx`

#### `app/(all)/sign-up/`

- [ ] `app/(all)/sign-up/layout.tsx`
- [ ] `app/(all)/sign-up/page.tsx`

#### `app/(all)/workspace-invitations/`

- [ ] `app/(all)/workspace-invitations/layout.tsx`
- [ ] `app/(all)/workspace-invitations/page.tsx`

#### `app/(home)/`

- [ ] `app/(home)/layout.tsx`
- [ ] `app/(home)/page.tsx`

#### `app/compat/next/`

- [x] `app/compat/next/helper.ts`
- [x] `app/compat/next/image.tsx`
- [x] `app/compat/next/link.tsx`
- [!] `app/compat/next/navigation.ts` — `useRouter` shim envuelve cada `push`/`replace`/`back`/`forward` en `setTimeout(..., 0)` sin cleanup: la navegación puede dispararse post-unmount del componente llamante (no se cancela el timer). `refresh` usa `location.reload()` sin `window.` prefix (funciona pero inconsistente). Hack documentado como "defer navigation to avoid state updates during render" — el problema raíz (actualizar estado durante render) debería corregirse en el caller en lugar de parchearlo aquí.
- [!] `app/compat/next/script.tsx` — (1) `[key: string]: any;` en `ScriptProps`; (2) `useEffect` dep array incluye `rest` (objeto nuevo en cada render) → el efecto re-crea y re-inserta el `<script>` en cada render del parent, leak potencial y doble ejecución; solución: serializar `rest` (`JSON.stringify`) o spread de props conocidas; (3) `script.setAttribute(key, rest[key])` sin validación — riesgo XSS si un caller pasa props controladas por usuario (bajo en práctica, pero sin whitelist).

#### `app/error/`

- [x] `app/error/dev.tsx`
- [x] `app/error/index.tsx`
- [x] `app/error/prod.tsx`

#### `app/routes/`

- [ ] `app/routes/core.ts`
- [ ] `app/routes/extended.ts`
- [ ] `app/routes/helper.ts`

#### `app/routes/redirects/`

- [ ] `app/routes/redirects/index.ts`

#### `app/routes/redirects/core/`

- [ ] `app/routes/redirects/core/accounts-signup.tsx`
- [ ] `app/routes/redirects/core/analytics.tsx`
- [ ] `app/routes/redirects/core/api-tokens.tsx`
- [ ] `app/routes/redirects/core/inbox.tsx`
- [ ] `app/routes/redirects/core/index.ts`
- [ ] `app/routes/redirects/core/login.tsx`
- [ ] `app/routes/redirects/core/profile-settings.tsx`
- [ ] `app/routes/redirects/core/project-settings.tsx`
- [ ] `app/routes/redirects/core/register.tsx`
- [ ] `app/routes/redirects/core/sign-in.tsx`
- [ ] `app/routes/redirects/core/signin.tsx`
- [ ] `app/routes/redirects/core/workspace-account-settings.tsx`

#### `app/routes/redirects/extended/`

- [ ] `app/routes/redirects/extended/index.ts`

## `ce/` — 279 archivos

#### `ce/components/active-cycles/`

- [ ] `ce/components/active-cycles/index.ts`
- [ ] `ce/components/active-cycles/root.tsx`
- [ ] `ce/components/active-cycles/workspace-active-cycles-upgrade.tsx`

#### `ce/components/analytics/`

- [ ] `ce/components/analytics/tabs.tsx`
- [ ] `ce/components/analytics/use-analytics-tabs.tsx`

#### `ce/components/app-rail/`

- [ ] `ce/components/app-rail/app-rail-hoc.tsx`
- [ ] `ce/components/app-rail/index.ts`

#### `ce/components/automations/`

- [ ] `ce/components/automations/root.tsx`

#### `ce/components/automations/list/`

- [ ] `ce/components/automations/list/wrapper.tsx`

#### `ce/components/breadcrumbs/`

- [ ] `ce/components/breadcrumbs/common.tsx`
- [ ] `ce/components/breadcrumbs/project-feature.tsx`
- [ ] `ce/components/breadcrumbs/project.tsx`

#### `ce/components/browse/`

- [ ] `ce/components/browse/workItem-detail.tsx`

#### `ce/components/command-palette/`

- [ ] `ce/components/command-palette/helpers.tsx`
- [ ] `ce/components/command-palette/index.ts`

#### `ce/components/command-palette/actions/`

- [ ] `ce/components/command-palette/actions/index.ts`

#### `ce/components/command-palette/actions/work-item-actions/`

- [ ] `ce/components/command-palette/actions/work-item-actions/change-state-list.tsx`
- [ ] `ce/components/command-palette/actions/work-item-actions/index.ts`

#### `ce/components/command-palette/modals/`

- [ ] `ce/components/command-palette/modals/project-level.tsx`
- [ ] `ce/components/command-palette/modals/work-item-level.tsx`
- [ ] `ce/components/command-palette/modals/workspace-level.tsx`

#### `ce/components/command-palette/power-k/`

- [ ] `ce/components/command-palette/power-k/constants.ts`
- [ ] `ce/components/command-palette/power-k/context-detector.ts`
- [ ] `ce/components/command-palette/power-k/types.ts`

#### `ce/components/command-palette/power-k/hooks/`

- [ ] `ce/components/command-palette/power-k/hooks/use-extended-context-indicator.ts`

#### `ce/components/command-palette/power-k/pages/context-based/`

- [ ] `ce/components/command-palette/power-k/pages/context-based/index.ts`
- [ ] `ce/components/command-palette/power-k/pages/context-based/root.tsx`

#### `ce/components/command-palette/power-k/pages/context-based/work-item/`

- [ ] `ce/components/command-palette/power-k/pages/context-based/work-item/state-menu-item.tsx`

#### `ce/components/command-palette/power-k/search/`

- [ ] `ce/components/command-palette/power-k/search/no-results-command.tsx`
- [ ] `ce/components/command-palette/power-k/search/search-results-map.tsx`

#### `ce/components/comments/`

- [ ] `ce/components/comments/comment-block.tsx`
- [ ] `ce/components/comments/index.ts`

#### `ce/components/common/`

- [ ] `ce/components/common/extended-app-header.tsx`
- [ ] `ce/components/common/quick-actions-factory.tsx`

#### `ce/components/common/modal/`

- [ ] `ce/components/common/modal/global.tsx`

#### `ce/components/common/subscription/`

- [ ] `ce/components/common/subscription/subscription-pill.tsx`

#### `ce/components/cycles/`

- [ ] `ce/components/cycles/additional-actions.tsx`
- [ ] `ce/components/cycles/index.ts`

#### `ce/components/cycles/active-cycle/`

- [ ] `ce/components/cycles/active-cycle/index.ts`
- [ ] `ce/components/cycles/active-cycle/root.tsx`

#### `ce/components/cycles/analytics-sidebar/`

- [ ] `ce/components/cycles/analytics-sidebar/base.tsx`
- [ ] `ce/components/cycles/analytics-sidebar/index.ts`
- [ ] `ce/components/cycles/analytics-sidebar/root.tsx`

#### `ce/components/cycles/end-cycle/`

- [ ] `ce/components/cycles/end-cycle/index.ts`
- [ ] `ce/components/cycles/end-cycle/modal.tsx`

#### `ce/components/de-dupe/`

- [ ] `ce/components/de-dupe/de-dupe-button.tsx`

#### `ce/components/de-dupe/duplicate-modal/`

- [ ] `ce/components/de-dupe/duplicate-modal/index.ts`
- [ ] `ce/components/de-dupe/duplicate-modal/root.tsx`

#### `ce/components/de-dupe/duplicate-popover/`

- [ ] `ce/components/de-dupe/duplicate-popover/index.ts`
- [ ] `ce/components/de-dupe/duplicate-popover/root.tsx`

#### `ce/components/de-dupe/issue-block/`

- [ ] `ce/components/de-dupe/issue-block/button-label.tsx`

#### `ce/components/desktop/`

- [ ] `ce/components/desktop/helper.ts`
- [ ] `ce/components/desktop/index.ts`
- [ ] `ce/components/desktop/sidebar-workspace-menu.tsx`

#### `ce/components/editor/embeds/mentions/`

- [ ] `ce/components/editor/embeds/mentions/index.ts`
- [ ] `ce/components/editor/embeds/mentions/root.tsx`

#### `ce/components/epics/epic-modal/`

- [ ] `ce/components/epics/epic-modal/index.ts`
- [ ] `ce/components/epics/epic-modal/modal.tsx`

#### `ce/components/estimates/`

- [ ] `ce/components/estimates/estimate-list-item-buttons.tsx`
- [ ] `ce/components/estimates/helper.tsx`
- [ ] `ce/components/estimates/index.ts`

#### `ce/components/estimates/inputs/`

- [ ] `ce/components/estimates/inputs/index.ts`
- [ ] `ce/components/estimates/inputs/time-input.tsx`

#### `ce/components/estimates/points/`

- [ ] `ce/components/estimates/points/delete.tsx`
- [ ] `ce/components/estimates/points/index.ts`

#### `ce/components/estimates/update/`

- [ ] `ce/components/estimates/update/index.ts`
- [ ] `ce/components/estimates/update/modal.tsx`

#### `ce/components/gantt-chart/`

- [ ] `ce/components/gantt-chart/index.ts`

#### `ce/components/gantt-chart/blocks/`

- [ ] `ce/components/gantt-chart/blocks/block-row-list.tsx`
- [ ] `ce/components/gantt-chart/blocks/blocks-list.tsx`

#### `ce/components/gantt-chart/dependency/`

- [ ] `ce/components/gantt-chart/dependency/dependency-paths.tsx`
- [ ] `ce/components/gantt-chart/dependency/draggable-dependency-path.tsx`
- [ ] `ce/components/gantt-chart/dependency/index.ts`

#### `ce/components/gantt-chart/dependency/blockDraggables/`

- [ ] `ce/components/gantt-chart/dependency/blockDraggables/index.ts`
- [ ] `ce/components/gantt-chart/dependency/blockDraggables/left-draggable.tsx`
- [ ] `ce/components/gantt-chart/dependency/blockDraggables/right-draggable.tsx`

#### `ce/components/gantt-chart/layers/`

- [ ] `ce/components/gantt-chart/layers/additional-layers.tsx`
- [ ] `ce/components/gantt-chart/layers/index.ts`

#### `ce/components/global/`

- [ ] `ce/components/global/index.ts`
- [ ] `ce/components/global/version-number.tsx`

#### `ce/components/global/product-updates/`

- [ ] `ce/components/global/product-updates/changelog.tsx`
- [ ] `ce/components/global/product-updates/header.tsx`

#### `ce/components/home/`

- [ ] `ce/components/home/header.tsx`
- [ ] `ce/components/home/index.ts`
- [ ] `ce/components/home/peek-overviews.tsx`

#### `ce/components/inbox/`

- [ ] `ce/components/inbox/source-pill.tsx`

#### `ce/components/instance/`

- [ ] `ce/components/instance/index.ts`
- [ ] `ce/components/instance/maintenance-message.tsx`

#### `ce/components/issues/`

- [ ] `ce/components/issues/header.tsx`

#### `ce/components/issues/bulk-operations/`

- [ ] `ce/components/issues/bulk-operations/index.ts`
- [ ] `ce/components/issues/bulk-operations/root.tsx`

#### `ce/components/issues/filters/`

- [ ] `ce/components/issues/filters/issue-types.tsx`
- [ ] `ce/components/issues/filters/team-project.tsx`

#### `ce/components/issues/filters/applied-filters/`

- [ ] `ce/components/issues/filters/applied-filters/issue-types.tsx`

#### `ce/components/issues/issue-detail-widgets/`

- [ ] `ce/components/issues/issue-detail-widgets/action-buttons.tsx`
- [ ] `ce/components/issues/issue-detail-widgets/collapsibles.tsx`
- [ ] `ce/components/issues/issue-detail-widgets/modals.tsx`

#### `ce/components/issues/issue-details/`

- [ ] `ce/components/issues/issue-details/additional-activity-root.tsx`
- [ ] `ce/components/issues/issue-details/additional-properties.tsx`
- [ ] `ce/components/issues/issue-details/index.ts`
- [ ] `ce/components/issues/issue-details/issue-creator.tsx`
- [ ] `ce/components/issues/issue-details/issue-identifier.tsx`
- [ ] `ce/components/issues/issue-details/issue-type-activity.tsx`
- [ ] `ce/components/issues/issue-details/issue-type-switcher.tsx`
- [ ] `ce/components/issues/issue-details/parent-select-root.tsx`

#### `ce/components/issues/issue-details/issue-properties-activity/`

- [ ] `ce/components/issues/issue-details/issue-properties-activity/index.ts`
- [ ] `ce/components/issues/issue-details/issue-properties-activity/root.tsx`

#### `ce/components/issues/issue-details/sidebar/`

- [ ] `ce/components/issues/issue-details/sidebar/date-alert.tsx`
- [ ] `ce/components/issues/issue-details/sidebar/transfer-hop-info.tsx`

#### `ce/components/issues/issue-layouts/`

- [ ] `ce/components/issues/issue-layouts/additional-properties.tsx`
- [ ] `ce/components/issues/issue-layouts/issue-stats.tsx`
- [ ] `ce/components/issues/issue-layouts/utils.tsx`

#### `ce/components/issues/issue-layouts/empty-states/`

- [ ] `ce/components/issues/issue-layouts/empty-states/index.ts`
- [ ] `ce/components/issues/issue-layouts/empty-states/team-issues.tsx`
- [ ] `ce/components/issues/issue-layouts/empty-states/team-project.tsx`
- [ ] `ce/components/issues/issue-layouts/empty-states/team-view-issues.tsx`

#### `ce/components/issues/issue-layouts/quick-action-dropdowns/`

- [ ] `ce/components/issues/issue-layouts/quick-action-dropdowns/copy-menu-helper.tsx`
- [ ] `ce/components/issues/issue-layouts/quick-action-dropdowns/duplicate-modal.tsx`
- [ ] `ce/components/issues/issue-layouts/quick-action-dropdowns/index.ts`

#### `ce/components/issues/issue-modal/`

- [ ] `ce/components/issues/issue-modal/index.ts`
- [ ] `ce/components/issues/issue-modal/issue-type-select.tsx`
- [ ] `ce/components/issues/issue-modal/modal-additional-properties.tsx`
- [ ] `ce/components/issues/issue-modal/provider.tsx`
- [ ] `ce/components/issues/issue-modal/template-select.tsx`

#### `ce/components/issues/quick-add/`

- [ ] `ce/components/issues/quick-add/index.ts`
- [ ] `ce/components/issues/quick-add/root.tsx`

#### `ce/components/issues/worklog/activity/`

- [ ] `ce/components/issues/worklog/activity/filter-root.tsx`
- [ ] `ce/components/issues/worklog/activity/index.ts`
- [ ] `ce/components/issues/worklog/activity/root.tsx`
- [ ] `ce/components/issues/worklog/activity/worklog-create-button.tsx`

#### `ce/components/issues/worklog/property/`

- [ ] `ce/components/issues/worklog/property/index.ts`
- [ ] `ce/components/issues/worklog/property/root.tsx`

#### `ce/components/license/`

- [ ] `ce/components/license/index.ts`

#### `ce/components/license/modal/`

- [ ] `ce/components/license/modal/index.ts`
- [ ] `ce/components/license/modal/upgrade-modal.tsx`

#### `ce/components/navigations/`

- [ ] `ce/components/navigations/index.ts`
- [ ] `ce/components/navigations/top-navigation-root.tsx`
- [ ] `ce/components/navigations/use-navigation-items.ts`

#### `ce/components/onboarding/tour/`

- [ ] `ce/components/onboarding/tour/root.tsx`
- [ ] `ce/components/onboarding/tour/sidebar.tsx`

#### `ce/components/pages/`

- [ ] `ce/components/pages/extra-actions.tsx`
- [ ] `ce/components/pages/index.ts`

#### `ce/components/pages/editor/`

- [ ] `ce/components/pages/editor/index.ts`

#### `ce/components/pages/editor/ai/`

- [ ] `ce/components/pages/editor/ai/ask-pi-menu.tsx`
- [ ] `ce/components/pages/editor/ai/index.ts`
- [ ] `ce/components/pages/editor/ai/menu.tsx`

#### `ce/components/pages/editor/embed/`

- [ ] `ce/components/pages/editor/embed/index.ts`
- [ ] `ce/components/pages/editor/embed/issue-embed-upgrade-card.tsx`

#### `ce/components/pages/header/`

- [ ] `ce/components/pages/header/collaborators-list.tsx`
- [ ] `ce/components/pages/header/lock-control.tsx`
- [ ] `ce/components/pages/header/move-control.tsx`
- [ ] `ce/components/pages/header/share-control.tsx`

#### `ce/components/pages/modals/`

- [ ] `ce/components/pages/modals/index.ts`
- [ ] `ce/components/pages/modals/modals.tsx`
- [ ] `ce/components/pages/modals/move-page-modal.tsx`

#### `ce/components/pages/navigation-pane/`

- [ ] `ce/components/pages/navigation-pane/index.ts`

#### `ce/components/pages/navigation-pane/tab-panels/`

- [ ] `ce/components/pages/navigation-pane/tab-panels/assets.tsx`
- [ ] `ce/components/pages/navigation-pane/tab-panels/root.tsx`

#### `ce/components/pages/navigation-pane/tab-panels/empty-states/`

- [ ] `ce/components/pages/navigation-pane/tab-panels/empty-states/assets.tsx`
- [ ] `ce/components/pages/navigation-pane/tab-panels/empty-states/outline.tsx`

#### `ce/components/preferences/`

- [ ] `ce/components/preferences/theme-switcher.tsx`

#### `ce/components/projects/`

- [ ] `ce/components/projects/header.tsx`
- [ ] `ce/components/projects/mobile-header.tsx`
- [ ] `ce/components/projects/page.tsx`

#### `ce/components/projects/create/`

- [ ] `ce/components/projects/create/attributes.tsx`
- [ ] `ce/components/projects/create/root.tsx`
- [ ] `ce/components/projects/create/template-select.tsx`
- [ ] `ce/components/projects/create/utils.ts`

#### `ce/components/projects/navigation/`

- [ ] `ce/components/projects/navigation/helper.tsx`

#### `ce/components/projects/settings/`

- [ ] `ce/components/projects/settings/features-list.tsx`
- [ ] `ce/components/projects/settings/useProjectColumns.tsx`

#### `ce/components/projects/settings/intake/`

- [ ] `ce/components/projects/settings/intake/header.tsx`

#### `ce/components/projects/teamspaces/`

- [ ] `ce/components/projects/teamspaces/teamspace-list.tsx`

#### `ce/components/relations/`

- [ ] `ce/components/relations/activity.ts`
- [ ] `ce/components/relations/index.tsx`

#### `ce/components/rich-filters/filter-value-input/`

- [ ] `ce/components/rich-filters/filter-value-input/root.tsx`

#### `ce/components/sidebar/`

- [ ] `ce/components/sidebar/app-switcher.tsx`
- [ ] `ce/components/sidebar/index.ts`
- [ ] `ce/components/sidebar/project-navigation-root.tsx`

#### `ce/components/views/`

- [ ] `ce/components/views/access-controller.tsx`
- [ ] `ce/components/views/helper.tsx`

#### `ce/components/views/filters/`

- [ ] `ce/components/views/filters/access-filter.tsx`

#### `ce/components/views/publish/`

- [ ] `ce/components/views/publish/index.ts`
- [ ] `ce/components/views/publish/modal.tsx`
- [ ] `ce/components/views/publish/use-view-publish.tsx`

#### `ce/components/workflow/`

- [ ] `ce/components/workflow/index.ts`
- [ ] `ce/components/workflow/state-option.tsx`
- [ ] `ce/components/workflow/use-workflow-drag-n-drop.ts`
- [ ] `ce/components/workflow/workflow-disabled-message.tsx`
- [ ] `ce/components/workflow/workflow-disabled-overlay.tsx`
- [ ] `ce/components/workflow/workflow-group-tree.tsx`

#### `ce/components/workspace/`

- [ ] `ce/components/workspace/app-switcher.tsx`
- [ ] `ce/components/workspace/content-wrapper.tsx`
- [ ] `ce/components/workspace/delete-workspace-modal.tsx`
- [ ] `ce/components/workspace/delete-workspace-section.tsx`
- [ ] `ce/components/workspace/edition-badge.tsx`
- [ ] `ce/components/workspace/upgrade-badge.tsx`

#### `ce/components/workspace/billing/`

- [ ] `ce/components/workspace/billing/billing-actions-button.tsx`
- [ ] `ce/components/workspace/billing/index.ts`
- [ ] `ce/components/workspace/billing/root.tsx`

#### `ce/components/workspace/billing/comparison/`

- [ ] `ce/components/workspace/billing/comparison/frequency-toggle.tsx`
- [ ] `ce/components/workspace/billing/comparison/plan-detail.tsx`
- [ ] `ce/components/workspace/billing/comparison/root.tsx`

#### `ce/components/workspace/members/`

- [ ] `ce/components/workspace/members/index.ts`
- [ ] `ce/components/workspace/members/invite-modal.tsx`
- [ ] `ce/components/workspace/members/members-activity-button.tsx`

#### `ce/components/workspace/settings/`

- [ ] `ce/components/workspace/settings/useMemberColumns.tsx`

#### `ce/components/workspace/sidebar/`

- [ ] `ce/components/workspace/sidebar/extended-sidebar-item.tsx`
- [ ] `ce/components/workspace/sidebar/helper.tsx`
- [ ] `ce/components/workspace/sidebar/sidebar-item.tsx`
- [ ] `ce/components/workspace/sidebar/teams-sidebar-list.tsx`

#### `ce/components/workspace-notifications/`

- [ ] `ce/components/workspace-notifications/index.ts`
- [ ] `ce/components/workspace-notifications/list-root.tsx`

#### `ce/components/workspace-notifications/notification-card/`

- [ ] `ce/components/workspace-notifications/notification-card/content.ts`
- [ ] `ce/components/workspace-notifications/notification-card/root.tsx`

#### `ce/hooks/`

- [ ] `ce/hooks/use-additional-editor-mention.tsx`
- [ ] `ce/hooks/use-additional-favorite-item-details.ts`
- [ ] `ce/hooks/use-bulk-operation-status.ts`
- [ ] `ce/hooks/use-debounced-duplicate-issues.tsx`
- [ ] `ce/hooks/use-editor-flagging.ts`
- [ ] `ce/hooks/use-file-size.ts`
- [ ] `ce/hooks/use-issue-embed.tsx`
- [ ] `ce/hooks/use-issue-properties.tsx`
- [ ] `ce/hooks/use-notification-preview.tsx`
- [ ] `ce/hooks/use-page-flag.ts`
- [ ] `ce/hooks/use-timeline-chart.ts`
- [ ] `ce/hooks/use-workspace-issue-properties-extended.tsx`

#### `ce/hooks/app-rail/`

- [ ] `ce/hooks/app-rail/index.ts`
- [ ] `ce/hooks/app-rail/provider.tsx`

#### `ce/hooks/editor/`

- [ ] `ce/hooks/editor/use-extended-editor-config.ts`

#### `ce/hooks/pages/`

- [ ] `ce/hooks/pages/index.ts`
- [ ] `ce/hooks/pages/use-extended-editor-extensions.ts`
- [ ] `ce/hooks/pages/use-pages-pane-extensions.ts`

#### `ce/hooks/rich-filters/`

- [ ] `ce/hooks/rich-filters/use-filters-operator-configs.ts`

#### `ce/hooks/store/`

- [ ] `ce/hooks/store/index.ts`
- [ ] `ce/hooks/store/use-page-store.ts`
- [ ] `ce/hooks/store/use-page.ts`

#### `ce/hooks/work-item-filters/`

- [ ] `ce/hooks/work-item-filters/use-work-item-filters-config.tsx`

#### `ce/store/`

- [ ] `ce/store/analytics.store.ts`
- [ ] `ce/store/command-palette.store.ts`
- [ ] `ce/store/global-view.store.ts`
- [ ] `ce/store/power-k.store.ts`
- [ ] `ce/store/project-inbox.store.ts`
- [ ] `ce/store/project-view.store.ts`
- [ ] `ce/store/root.store.ts`
- [ ] `ce/store/state.store.ts`

#### `ce/store/cycle/`

- [ ] `ce/store/cycle/index.ts`

#### `ce/store/estimates/`

- [ ] `ce/store/estimates/estimate.ts`

#### `ce/store/issue/epic/`

- [ ] `ce/store/issue/epic/filter.store.ts`
- [ ] `ce/store/issue/epic/index.ts`
- [ ] `ce/store/issue/epic/issue.store.ts`

#### `ce/store/issue/helpers/`

- [ ] `ce/store/issue/helpers/base-issue-store.ts`
- [ ] `ce/store/issue/helpers/base-issue.store.ts`
- [ ] `ce/store/issue/helpers/filter-utils.ts`

#### `ce/store/issue/issue-details/`

- [ ] `ce/store/issue/issue-details/activity.store.ts`
- [ ] `ce/store/issue/issue-details/root.store.ts`

#### `ce/store/issue/team/`

- [ ] `ce/store/issue/team/filter.store.ts`
- [ ] `ce/store/issue/team/index.ts`
- [ ] `ce/store/issue/team/issue.store.ts`

#### `ce/store/issue/team-project/`

- [ ] `ce/store/issue/team-project/filter.store.ts`
- [ ] `ce/store/issue/team-project/index.ts`
- [ ] `ce/store/issue/team-project/issue.store.ts`

#### `ce/store/issue/team-views/`

- [ ] `ce/store/issue/team-views/filter.store.ts`
- [ ] `ce/store/issue/team-views/index.ts`
- [ ] `ce/store/issue/team-views/issue.store.ts`

#### `ce/store/issue/workspace/`

- [ ] `ce/store/issue/workspace/issue.store.ts`

#### `ce/store/member/`

- [ ] `ce/store/member/project-member.store.ts`

#### `ce/store/pages/`

- [ ] `ce/store/pages/extended-base-page.ts`

#### `ce/store/timeline/`

- [ ] `ce/store/timeline/base-timeline.store.ts`
- [ ] `ce/store/timeline/index.ts`

#### `ce/store/user/`

- [ ] `ce/store/user/permission.store.ts`

#### `ce/store/workspace/`

- [ ] `ce/store/workspace/index.ts`

#### `ce/types/`

- [ ] `ce/types/gantt-chart.ts`
- [ ] `ce/types/index.ts`

#### `ce/types/issue-types/`

- [ ] `ce/types/issue-types/index.ts`

#### `ce/types/pages/`

- [ ] `ce/types/pages/pane-extensions.ts`

#### `ce/types/projects/`

- [ ] `ce/types/projects/index.ts`
- [ ] `ce/types/projects/project-activity.ts`
- [ ] `ce/types/projects/projects.ts`

## `core/` — 1589 archivos

#### `core/components/account/`

- [ ] `core/components/account/deactivate-account-modal.tsx`
- [ ] `core/components/account/terms-and-conditions.tsx`

#### `core/components/account/auth-forms/`

- [ ] `core/components/account/auth-forms/auth-banner.tsx`
- [ ] `core/components/account/auth-forms/auth-header.tsx`
- [ ] `core/components/account/auth-forms/auth-root.tsx`
- [ ] `core/components/account/auth-forms/email.tsx`
- [ ] `core/components/account/auth-forms/forgot-password-popover.tsx`
- [ ] `core/components/account/auth-forms/forgot-password.tsx`
- [ ] `core/components/account/auth-forms/form-root.tsx`
- [ ] `core/components/account/auth-forms/index.ts`
- [ ] `core/components/account/auth-forms/password.tsx`
- [ ] `core/components/account/auth-forms/reset-password.tsx`
- [ ] `core/components/account/auth-forms/set-password.tsx`
- [ ] `core/components/account/auth-forms/unique-code.tsx`

#### `core/components/account/auth-forms/common/`

- [ ] `core/components/account/auth-forms/common/container.tsx`
- [ ] `core/components/account/auth-forms/common/header.tsx`

#### `core/components/analytics/`

- [ ] `core/components/analytics/analytics-filter-actions.tsx`
- [ ] `core/components/analytics/analytics-section-wrapper.tsx`
- [ ] `core/components/analytics/analytics-wrapper.tsx`
- [ ] `core/components/analytics/empty-state.tsx`
- [ ] `core/components/analytics/export.ts`
- [ ] `core/components/analytics/insight-card.tsx`
- [ ] `core/components/analytics/loaders.tsx`
- [ ] `core/components/analytics/total-insights.tsx`
- [ ] `core/components/analytics/trend-piece.tsx`

#### `core/components/analytics/insight-table/`

- [ ] `core/components/analytics/insight-table/data-table.tsx`
- [ ] `core/components/analytics/insight-table/index.ts`
- [ ] `core/components/analytics/insight-table/loader.tsx`
- [ ] `core/components/analytics/insight-table/root.tsx`

#### `core/components/analytics/overview/`

- [ ] `core/components/analytics/overview/active-project-item.tsx`
- [ ] `core/components/analytics/overview/active-projects.tsx`
- [ ] `core/components/analytics/overview/index.ts`
- [ ] `core/components/analytics/overview/project-insights.tsx`
- [ ] `core/components/analytics/overview/root.tsx`

#### `core/components/analytics/select/`

- [ ] `core/components/analytics/select/analytics-params.tsx`
- [ ] `core/components/analytics/select/duration.tsx`
- [ ] `core/components/analytics/select/project.tsx`
- [ ] `core/components/analytics/select/select-x-axis.tsx`
- [ ] `core/components/analytics/select/select-y-axis.tsx`

#### `core/components/analytics/work-items/`

- [ ] `core/components/analytics/work-items/created-vs-resolved.tsx`
- [ ] `core/components/analytics/work-items/customized-insights.tsx`
- [ ] `core/components/analytics/work-items/index.ts`
- [ ] `core/components/analytics/work-items/priority-chart.tsx`
- [ ] `core/components/analytics/work-items/root.tsx`
- [ ] `core/components/analytics/work-items/utils.ts`
- [ ] `core/components/analytics/work-items/workitems-insight-table.tsx`

#### `core/components/analytics/work-items/modal/`

- [ ] `core/components/analytics/work-items/modal/content.tsx`
- [ ] `core/components/analytics/work-items/modal/header.tsx`
- [ ] `core/components/analytics/work-items/modal/index.tsx`

#### `core/components/api-token/`

- [ ] `core/components/api-token/delete-token-modal.tsx`
- [ ] `core/components/api-token/empty-state.tsx`
- [ ] `core/components/api-token/token-list-item.tsx`

#### `core/components/api-token/modal/`

- [ ] `core/components/api-token/modal/create-token-modal.tsx`
- [ ] `core/components/api-token/modal/form.tsx`
- [ ] `core/components/api-token/modal/generated-token-details.tsx`

#### `core/components/appearance/`

- [ ] `core/components/appearance/index.ts`
- [ ] `core/components/appearance/theme-switcher.tsx`

#### `core/components/archives/`

- [ ] `core/components/archives/archive-tabs-list.tsx`
- [ ] `core/components/archives/index.ts`

#### `core/components/auth-screens/`

- [ ] `core/components/auth-screens/auth-base.tsx`
- [ ] `core/components/auth-screens/footer.tsx`
- [ ] `core/components/auth-screens/header.tsx`
- [ ] `core/components/auth-screens/not-authorized-view.tsx`

#### `core/components/auth-screens/project/`

- [ ] `core/components/auth-screens/project/project-access-restriction.tsx`

#### `core/components/auth-screens/workspace/`

- [ ] `core/components/auth-screens/workspace/not-a-member.tsx`

#### `core/components/automation/`

- [ ] `core/components/automation/auto-archive-automation.tsx`
- [ ] `core/components/automation/auto-close-automation.tsx`
- [ ] `core/components/automation/index.ts`
- [ ] `core/components/automation/select-month-modal.tsx`

#### `core/components/base-layouts/`

- [ ] `core/components/base-layouts/constants.ts`
- [ ] `core/components/base-layouts/layout-switcher.tsx`

#### `core/components/base-layouts/gantt/`

- [ ] `core/components/base-layouts/gantt/index.ts`
- [ ] `core/components/base-layouts/gantt/layout.tsx`
- [ ] `core/components/base-layouts/gantt/sidebar.tsx`

#### `core/components/base-layouts/hooks/`

- [ ] `core/components/base-layouts/hooks/use-group-drop-target.ts`
- [ ] `core/components/base-layouts/hooks/use-layout-state.ts`

#### `core/components/base-layouts/kanban/`

- [ ] `core/components/base-layouts/kanban/group-header.tsx`
- [ ] `core/components/base-layouts/kanban/group.tsx`
- [ ] `core/components/base-layouts/kanban/item.tsx`
- [ ] `core/components/base-layouts/kanban/layout.tsx`

#### `core/components/base-layouts/list/`

- [ ] `core/components/base-layouts/list/group-header.tsx`
- [ ] `core/components/base-layouts/list/group.tsx`
- [ ] `core/components/base-layouts/list/item.tsx`
- [ ] `core/components/base-layouts/list/layout.tsx`

#### `core/components/base-layouts/loaders/`

- [ ] `core/components/base-layouts/loaders/layout-loader.tsx`

#### `core/components/chart/`

- [ ] `core/components/chart/utils.ts`

#### `core/components/comments/`

- [ ] `core/components/comments/comment-create.tsx`
- [ ] `core/components/comments/comment-reaction.tsx`
- [ ] `core/components/comments/comments.tsx`
- [ ] `core/components/comments/index.ts`
- [ ] `core/components/comments/quick-actions.tsx`

#### `core/components/comments/card/`

- [ ] `core/components/comments/card/display.tsx`
- [ ] `core/components/comments/card/edit-form.tsx`
- [ ] `core/components/comments/card/root.tsx`

#### `core/components/common/`

- [ ] `core/components/common/access-field.tsx`
- [ ] `core/components/common/breadcrumb-link.tsx`
- [ ] `core/components/common/count-chip.tsx`
- [ ] `core/components/common/cover-image.tsx`
- [ ] `core/components/common/empty-state.tsx`
- [ ] `core/components/common/latest-feature-block.tsx`
- [ ] `core/components/common/logo-spinner.tsx`
- [ ] `core/components/common/new-empty-state.tsx`
- [ ] `core/components/common/page-access-icon.tsx`
- [ ] `core/components/common/pro-icon.tsx`
- [ ] `core/components/common/quick-actions-factory.tsx`
- [ ] `core/components/common/quick-actions-helper.tsx`
- [ ] `core/components/common/switcher-label.tsx`

#### `core/components/common/activity/`

- [ ] `core/components/common/activity/activity-block.tsx`
- [ ] `core/components/common/activity/activity-item.tsx`
- [ ] `core/components/common/activity/helper.tsx`
- [ ] `core/components/common/activity/user.tsx`

#### `core/components/common/applied-filters/`

- [ ] `core/components/common/applied-filters/date.tsx`
- [ ] `core/components/common/applied-filters/members.tsx`

#### `core/components/common/filters/`

- [ ] `core/components/common/filters/created-at.tsx`
- [ ] `core/components/common/filters/created-by.tsx`

#### `core/components/common/layout/sidebar/`

- [ ] `core/components/common/layout/sidebar/property-list-item.tsx`

#### `core/components/core/`

- [ ] `core/components/core/activity.tsx`
- [ ] `core/components/core/app-header.tsx`
- [ ] `core/components/core/content-overflow-HOC.tsx`
- [ ] `core/components/core/content-wrapper.tsx`
- [ ] `core/components/core/image-picker-popover.tsx`
- [ ] `core/components/core/page-title.tsx`
- [ ] `core/components/core/render-if-visible-HOC.tsx`

#### `core/components/core/description-versions/`

- [ ] `core/components/core/description-versions/dropdown-item.tsx`
- [ ] `core/components/core/description-versions/dropdown.tsx`
- [ ] `core/components/core/description-versions/index.ts`
- [ ] `core/components/core/description-versions/modal.tsx`
- [ ] `core/components/core/description-versions/root.tsx`

#### `core/components/core/filters/`

- [ ] `core/components/core/filters/date-filter-modal.tsx`
- [ ] `core/components/core/filters/date-filter-select.tsx`

#### `core/components/core/list/`

- [ ] `core/components/core/list/index.ts`
- [ ] `core/components/core/list/list-item.tsx`
- [ ] `core/components/core/list/list-root.tsx`

#### `core/components/core/modals/`

- [ ] `core/components/core/modals/bulk-delete-issues-modal-item.tsx`
- [ ] `core/components/core/modals/bulk-delete-issues-modal.tsx`
- [ ] `core/components/core/modals/change-email-modal.tsx`
- [ ] `core/components/core/modals/existing-issues-list-modal.tsx`
- [ ] `core/components/core/modals/gpt-assistant-popover.tsx`
- [ ] `core/components/core/modals/issue-search-modal-empty-state.tsx`
- [ ] `core/components/core/modals/user-image-upload-modal.tsx`
- [ ] `core/components/core/modals/workspace-image-upload-modal.tsx`

#### `core/components/core/multiple-select/`

- [ ] `core/components/core/multiple-select/entity-select-action.tsx`
- [ ] `core/components/core/multiple-select/group-select-action.tsx`
- [ ] `core/components/core/multiple-select/index.ts`
- [ ] `core/components/core/multiple-select/select-group.tsx`

#### `core/components/core/sidebar/`

- [ ] `core/components/core/sidebar/progress-chart.tsx`
- [ ] `core/components/core/sidebar/sidebar-menu-hamburger-toggle.tsx`
- [ ] `core/components/core/sidebar/single-progress-stats.tsx`

#### `core/components/core/sidebar/progress-stats/`

- [ ] `core/components/core/sidebar/progress-stats/assignee.tsx`
- [ ] `core/components/core/sidebar/progress-stats/label.tsx`
- [ ] `core/components/core/sidebar/progress-stats/shared.ts`
- [ ] `core/components/core/sidebar/progress-stats/state_group.tsx`

#### `core/components/core/theme/`

- [ ] `core/components/core/theme/color-inputs.tsx`
- [ ] `core/components/core/theme/custom-theme-selector.tsx`
- [ ] `core/components/core/theme/download-config-button.tsx`
- [ ] `core/components/core/theme/import-config-button.tsx`
- [ ] `core/components/core/theme/theme-mode-selector.tsx`
- [ ] `core/components/core/theme/theme-switch.tsx`

#### `core/components/cycles/`

- [ ] `core/components/cycles/cycle-peek-overview.tsx`
- [ ] `core/components/cycles/cycles-view-header.tsx`
- [ ] `core/components/cycles/cycles-view.tsx`
- [ ] `core/components/cycles/delete-modal.tsx`
- [ ] `core/components/cycles/form.tsx`
- [ ] `core/components/cycles/modal.tsx`
- [ ] `core/components/cycles/quick-actions.tsx`
- [ ] `core/components/cycles/transfer-issues-modal.tsx`
- [ ] `core/components/cycles/transfer-issues.tsx`

#### `core/components/cycles/active-cycle/`

- [ ] `core/components/cycles/active-cycle/cycle-stats.tsx`
- [ ] `core/components/cycles/active-cycle/productivity.tsx`
- [ ] `core/components/cycles/active-cycle/progress.tsx`
- [ ] `core/components/cycles/active-cycle/use-cycles-details.ts`

#### `core/components/cycles/analytics-sidebar/`

- [ ] `core/components/cycles/analytics-sidebar/index.ts`
- [ ] `core/components/cycles/analytics-sidebar/issue-progress.tsx`
- [ ] `core/components/cycles/analytics-sidebar/progress-stats.tsx`
- [ ] `core/components/cycles/analytics-sidebar/root.tsx`
- [ ] `core/components/cycles/analytics-sidebar/sidebar-details.tsx`
- [ ] `core/components/cycles/analytics-sidebar/sidebar-header.tsx`

#### `core/components/cycles/applied-filters/`

- [ ] `core/components/cycles/applied-filters/date.tsx`
- [ ] `core/components/cycles/applied-filters/index.ts`
- [ ] `core/components/cycles/applied-filters/root.tsx`
- [ ] `core/components/cycles/applied-filters/status.tsx`

#### `core/components/cycles/archived-cycles/`

- [ ] `core/components/cycles/archived-cycles/header.tsx`
- [ ] `core/components/cycles/archived-cycles/index.ts`
- [ ] `core/components/cycles/archived-cycles/modal.tsx`
- [ ] `core/components/cycles/archived-cycles/root.tsx`
- [ ] `core/components/cycles/archived-cycles/view.tsx`

#### `core/components/cycles/dropdowns/`

- [ ] `core/components/cycles/dropdowns/estimate-type-dropdown.tsx`
- [ ] `core/components/cycles/dropdowns/index.ts`

#### `core/components/cycles/dropdowns/filters/`

- [ ] `core/components/cycles/dropdowns/filters/end-date.tsx`
- [ ] `core/components/cycles/dropdowns/filters/index.ts`
- [ ] `core/components/cycles/dropdowns/filters/root.tsx`
- [ ] `core/components/cycles/dropdowns/filters/start-date.tsx`
- [ ] `core/components/cycles/dropdowns/filters/status.tsx`

#### `core/components/cycles/list/`

- [ ] `core/components/cycles/list/cycle-list-group-header.tsx`
- [ ] `core/components/cycles/list/cycle-list-item-action.tsx`
- [ ] `core/components/cycles/list/cycle-list-project-group-header.tsx`
- [ ] `core/components/cycles/list/cycles-list-item.tsx`
- [ ] `core/components/cycles/list/cycles-list-map.tsx`
- [ ] `core/components/cycles/list/index.ts`
- [ ] `core/components/cycles/list/root.tsx`

#### `core/components/dropdowns/`

- [ ] `core/components/dropdowns/buttons.tsx`
- [ ] `core/components/dropdowns/constants.ts`
- [ ] `core/components/dropdowns/date-range.tsx`
- [ ] `core/components/dropdowns/date.tsx`
- [ ] `core/components/dropdowns/estimate.tsx`
- [ ] `core/components/dropdowns/layout.tsx`
- [ ] `core/components/dropdowns/merged-date.tsx`
- [ ] `core/components/dropdowns/priority.tsx`

#### `core/components/dropdowns/cycle/`

- [ ] `core/components/dropdowns/cycle/cycle-options.tsx`
- [ ] `core/components/dropdowns/cycle/index.tsx`

#### `core/components/dropdowns/intake-state/`

- [ ] `core/components/dropdowns/intake-state/base.tsx`
- [ ] `core/components/dropdowns/intake-state/dropdown.tsx`

#### `core/components/dropdowns/member/`

- [ ] `core/components/dropdowns/member/avatar.tsx`
- [ ] `core/components/dropdowns/member/base.tsx`
- [ ] `core/components/dropdowns/member/dropdown.tsx`
- [ ] `core/components/dropdowns/member/member-options.tsx`

#### `core/components/dropdowns/module/`

- [ ] `core/components/dropdowns/module/base.tsx`
- [ ] `core/components/dropdowns/module/button-content.tsx`
- [ ] `core/components/dropdowns/module/dropdown.tsx`
- [ ] `core/components/dropdowns/module/module-options.tsx`

#### `core/components/dropdowns/project/`

- [ ] `core/components/dropdowns/project/base.tsx`
- [ ] `core/components/dropdowns/project/dropdown.tsx`

#### `core/components/dropdowns/state/`

- [ ] `core/components/dropdowns/state/base.tsx`
- [ ] `core/components/dropdowns/state/dropdown.tsx`

#### `core/components/editor/document/`

- [ ] `core/components/editor/document/editor.tsx`

#### `core/components/editor/embeds/mentions/`

- [ ] `core/components/editor/embeds/mentions/index.ts`
- [ ] `core/components/editor/embeds/mentions/root.tsx`
- [ ] `core/components/editor/embeds/mentions/user.tsx`

#### `core/components/editor/lite-text/`

- [ ] `core/components/editor/lite-text/editor.tsx`
- [ ] `core/components/editor/lite-text/index.ts`
- [ ] `core/components/editor/lite-text/lite-toolbar.tsx`
- [ ] `core/components/editor/lite-text/toolbar.tsx`

#### `core/components/editor/pdf/`

- [ ] `core/components/editor/pdf/document.tsx`
- [ ] `core/components/editor/pdf/index.ts`

#### `core/components/editor/rich-text/`

- [ ] `core/components/editor/rich-text/editor.tsx`
- [ ] `core/components/editor/rich-text/index.ts`

#### `core/components/editor/rich-text/description-input/`

- [ ] `core/components/editor/rich-text/description-input/index.ts`
- [ ] `core/components/editor/rich-text/description-input/loader.tsx`
- [ ] `core/components/editor/rich-text/description-input/root.tsx`

#### `core/components/editor/sticky-editor/`

- [ ] `core/components/editor/sticky-editor/color-palette.tsx`
- [ ] `core/components/editor/sticky-editor/editor.tsx`
- [ ] `core/components/editor/sticky-editor/index.ts`
- [ ] `core/components/editor/sticky-editor/toolbar.tsx`

#### `core/components/empty-state/`

- [ ] `core/components/empty-state/comic-box-button.tsx`
- [ ] `core/components/empty-state/detailed-empty-state-root.tsx`
- [ ] `core/components/empty-state/helper.tsx`
- [ ] `core/components/empty-state/section-empty-state-root.tsx`
- [ ] `core/components/empty-state/simple-empty-state-root.tsx`

#### `core/components/estimates/`

- [ ] `core/components/estimates/empty-screen.tsx`
- [ ] `core/components/estimates/estimate-disable-switch.tsx`
- [ ] `core/components/estimates/estimate-list-item.tsx`
- [ ] `core/components/estimates/estimate-list.tsx`
- [ ] `core/components/estimates/estimate-search.tsx`
- [ ] `core/components/estimates/index.ts`
- [ ] `core/components/estimates/loader-screen.tsx`
- [ ] `core/components/estimates/radio-select.tsx`
- [ ] `core/components/estimates/root.tsx`

#### `core/components/estimates/create/`

- [ ] `core/components/estimates/create/modal.tsx`
- [ ] `core/components/estimates/create/stage-one.tsx`

#### `core/components/estimates/delete/`

- [ ] `core/components/estimates/delete/modal.tsx`

#### `core/components/estimates/inputs/`

- [ ] `core/components/estimates/inputs/index.ts`
- [ ] `core/components/estimates/inputs/number-input.tsx`
- [ ] `core/components/estimates/inputs/root.tsx`
- [ ] `core/components/estimates/inputs/text-input.tsx`

#### `core/components/estimates/points/`

- [ ] `core/components/estimates/points/create-root.tsx`
- [ ] `core/components/estimates/points/create.tsx`
- [ ] `core/components/estimates/points/index.ts`
- [ ] `core/components/estimates/points/preview.tsx`
- [ ] `core/components/estimates/points/update.tsx`

#### `core/components/exporter/`

- [ ] `core/components/exporter/column.tsx`
- [ ] `core/components/exporter/export-form.tsx`
- [ ] `core/components/exporter/export-modal.tsx`
- [ ] `core/components/exporter/guide.tsx`
- [ ] `core/components/exporter/prev-exports.tsx`
- [ ] `core/components/exporter/single-export.tsx`

#### `core/components/gantt-chart/`

- [ ] `core/components/gantt-chart/constants.ts`
- [ ] `core/components/gantt-chart/index.ts`
- [ ] `core/components/gantt-chart/root.tsx`

#### `core/components/gantt-chart/blocks/`

- [ ] `core/components/gantt-chart/blocks/block-row.tsx`
- [ ] `core/components/gantt-chart/blocks/block.tsx`

#### `core/components/gantt-chart/chart/`

- [ ] `core/components/gantt-chart/chart/header.tsx`
- [ ] `core/components/gantt-chart/chart/index.ts`
- [ ] `core/components/gantt-chart/chart/main-content.tsx`
- [ ] `core/components/gantt-chart/chart/root.tsx`
- [ ] `core/components/gantt-chart/chart/timeline-drag-helper.tsx`

#### `core/components/gantt-chart/chart/views/`

- [ ] `core/components/gantt-chart/chart/views/index.ts`
- [ ] `core/components/gantt-chart/chart/views/month.tsx`
- [ ] `core/components/gantt-chart/chart/views/quarter.tsx`
- [ ] `core/components/gantt-chart/chart/views/week.tsx`

#### `core/components/gantt-chart/contexts/`

- [ ] `core/components/gantt-chart/contexts/index.tsx`

#### `core/components/gantt-chart/data/`

- [ ] `core/components/gantt-chart/data/index.ts`

#### `core/components/gantt-chart/helpers/`

- [ ] `core/components/gantt-chart/helpers/add-block.tsx`
- [ ] `core/components/gantt-chart/helpers/draggable.tsx`
- [ ] `core/components/gantt-chart/helpers/index.ts`

#### `core/components/gantt-chart/helpers/blockResizables/`

- [ ] `core/components/gantt-chart/helpers/blockResizables/left-resizable.tsx`
- [ ] `core/components/gantt-chart/helpers/blockResizables/right-resizable.tsx`
- [ ] `core/components/gantt-chart/helpers/blockResizables/use-gantt-resizable.ts`

#### `core/components/gantt-chart/sidebar/`

- [ ] `core/components/gantt-chart/sidebar/gantt-dnd-HOC.tsx`
- [ ] `core/components/gantt-chart/sidebar/index.ts`
- [ ] `core/components/gantt-chart/sidebar/root.tsx`
- [ ] `core/components/gantt-chart/sidebar/utils.ts`

#### `core/components/gantt-chart/sidebar/issues/`

- [ ] `core/components/gantt-chart/sidebar/issues/block.tsx`
- [ ] `core/components/gantt-chart/sidebar/issues/index.ts`
- [ ] `core/components/gantt-chart/sidebar/issues/sidebar.tsx`

#### `core/components/gantt-chart/sidebar/modules/`

- [ ] `core/components/gantt-chart/sidebar/modules/block.tsx`
- [ ] `core/components/gantt-chart/sidebar/modules/index.ts`
- [ ] `core/components/gantt-chart/sidebar/modules/sidebar.tsx`

#### `core/components/gantt-chart/views/`

- [ ] `core/components/gantt-chart/views/helpers.ts`
- [ ] `core/components/gantt-chart/views/index.ts`
- [ ] `core/components/gantt-chart/views/month-view.ts`
- [ ] `core/components/gantt-chart/views/quarter-view.ts`
- [ ] `core/components/gantt-chart/views/week-view.ts`

#### `core/components/global/`

- [ ] `core/components/global/index.ts`
- [x] `core/components/global/timezone-select.tsx`

#### `core/components/global/product-updates/`

- [ ] `core/components/global/product-updates/fallback.tsx`
- [ ] `core/components/global/product-updates/footer.tsx`
- [ ] `core/components/global/product-updates/index.ts`
- [ ] `core/components/global/product-updates/modal.tsx`

#### `core/components/home/`

- [ ] `core/components/home/home-dashboard-widgets.tsx`
- [ ] `core/components/home/index.ts`
- [ ] `core/components/home/root.tsx`
- [ ] `core/components/home/user-greetings.tsx`

#### `core/components/home/widgets/`

- [ ] `core/components/home/widgets/index.ts`

#### `core/components/home/widgets/empty-states/`

- [ ] `core/components/home/widgets/empty-states/index.ts`
- [ ] `core/components/home/widgets/empty-states/links.tsx`
- [ ] `core/components/home/widgets/empty-states/no-projects.tsx`
- [ ] `core/components/home/widgets/empty-states/recents.tsx`
- [ ] `core/components/home/widgets/empty-states/stickies.tsx`

#### `core/components/home/widgets/links/`

- [ ] `core/components/home/widgets/links/action.tsx`
- [ ] `core/components/home/widgets/links/create-update-link-modal.tsx`
- [ ] `core/components/home/widgets/links/index.ts`
- [ ] `core/components/home/widgets/links/link-detail.tsx`
- [ ] `core/components/home/widgets/links/links.tsx`
- [ ] `core/components/home/widgets/links/root.tsx`
- [ ] `core/components/home/widgets/links/use-links.tsx`

#### `core/components/home/widgets/loaders/`

- [ ] `core/components/home/widgets/loaders/home-loader.tsx`
- [ ] `core/components/home/widgets/loaders/index.ts`
- [ ] `core/components/home/widgets/loaders/loader.tsx`
- [ ] `core/components/home/widgets/loaders/quick-links.tsx`
- [ ] `core/components/home/widgets/loaders/recent-activity.tsx`

#### `core/components/home/widgets/manage/`

- [ ] `core/components/home/widgets/manage/index.tsx`
- [ ] `core/components/home/widgets/manage/widget-item-drag-handle.tsx`
- [ ] `core/components/home/widgets/manage/widget-item.tsx`
- [ ] `core/components/home/widgets/manage/widget-list.tsx`
- [ ] `core/components/home/widgets/manage/widget.helpers.ts`

#### `core/components/home/widgets/recents/`

- [ ] `core/components/home/widgets/recents/filters.tsx`
- [x] `core/components/home/widgets/recents/index.tsx`
- [ ] `core/components/home/widgets/recents/issue.tsx`
- [ ] `core/components/home/widgets/recents/page.tsx`
- [ ] `core/components/home/widgets/recents/project.tsx`

#### `core/components/icons/`

- [ ] `core/components/icons/index.ts`
- [ ] `core/components/icons/locked-component.tsx`

#### `core/components/icons/attachment/`

- [ ] `core/components/icons/attachment/attachment-icon.tsx`
- [ ] `core/components/icons/attachment/audio-file-icon.tsx`
- [ ] `core/components/icons/attachment/css-file-icon.tsx`
- [ ] `core/components/icons/attachment/csv-file-icon.tsx`
- [ ] `core/components/icons/attachment/default-file-icon.tsx`
- [ ] `core/components/icons/attachment/doc-file-icon.tsx`
- [ ] `core/components/icons/attachment/document-icon.tsx`
- [ ] `core/components/icons/attachment/figma-file-icon.tsx`
- [ ] `core/components/icons/attachment/html-file-icon.tsx`
- [ ] `core/components/icons/attachment/img-file-icon.tsx`
- [ ] `core/components/icons/attachment/index.ts`
- [ ] `core/components/icons/attachment/jpg-file-icon.tsx`
- [ ] `core/components/icons/attachment/js-file-icon.tsx`
- [ ] `core/components/icons/attachment/pdf-file-icon.tsx`
- [ ] `core/components/icons/attachment/png-file-icon.tsx`
- [ ] `core/components/icons/attachment/rar-file-icon.tsx`
- [ ] `core/components/icons/attachment/setting-icon.tsx`
- [ ] `core/components/icons/attachment/sheet-file-icon.tsx`
- [ ] `core/components/icons/attachment/svg-file-icon.tsx`
- [ ] `core/components/icons/attachment/tune-icon.tsx`
- [ ] `core/components/icons/attachment/txt-file-icon.tsx`
- [ ] `core/components/icons/attachment/video-file-icon.tsx`
- [ ] `core/components/icons/attachment/zip-file-icon.tsx`

#### `core/components/inbox/`

- [ ] `core/components/inbox/inbox-issue-status.tsx`
- [ ] `core/components/inbox/inbox-status-icon.tsx`
- [ ] `core/components/inbox/index.ts`
- [ ] `core/components/inbox/root.tsx`

#### `core/components/inbox/content/`

- [ ] `core/components/inbox/content/inbox-issue-header.tsx`
- [ ] `core/components/inbox/content/inbox-issue-mobile-header.tsx`
- [ ] `core/components/inbox/content/index.ts`
- [ ] `core/components/inbox/content/issue-properties.tsx`
- [ ] `core/components/inbox/content/issue-root.tsx`
- [ ] `core/components/inbox/content/root.tsx`

#### `core/components/inbox/inbox-filter/`

- [ ] `core/components/inbox/inbox-filter/index.ts`
- [ ] `core/components/inbox/inbox-filter/root.tsx`

#### `core/components/inbox/inbox-filter/applied-filters/`

- [ ] `core/components/inbox/inbox-filter/applied-filters/date.tsx`
- [ ] `core/components/inbox/inbox-filter/applied-filters/label.tsx`
- [ ] `core/components/inbox/inbox-filter/applied-filters/member.tsx`
- [ ] `core/components/inbox/inbox-filter/applied-filters/priority.tsx`
- [ ] `core/components/inbox/inbox-filter/applied-filters/root.tsx`
- [ ] `core/components/inbox/inbox-filter/applied-filters/state.tsx`
- [ ] `core/components/inbox/inbox-filter/applied-filters/status.tsx`

#### `core/components/inbox/inbox-filter/filters/`

- [ ] `core/components/inbox/inbox-filter/filters/date.tsx`
- [ ] `core/components/inbox/inbox-filter/filters/filter-selection.tsx`
- [ ] `core/components/inbox/inbox-filter/filters/labels.tsx`
- [ ] `core/components/inbox/inbox-filter/filters/members.tsx`
- [ ] `core/components/inbox/inbox-filter/filters/priority.tsx`
- [ ] `core/components/inbox/inbox-filter/filters/state.tsx`
- [ ] `core/components/inbox/inbox-filter/filters/status.tsx`

#### `core/components/inbox/inbox-filter/sorting/`

- [ ] `core/components/inbox/inbox-filter/sorting/order-by.tsx`

#### `core/components/inbox/modals/`

- [ ] `core/components/inbox/modals/decline-issue-modal.tsx`
- [ ] `core/components/inbox/modals/delete-issue-modal.tsx`
- [ ] `core/components/inbox/modals/select-duplicate.tsx`
- [ ] `core/components/inbox/modals/snooze-issue-modal.tsx`

#### `core/components/inbox/modals/create-modal/`

- [ ] `core/components/inbox/modals/create-modal/create-root.tsx`
- [ ] `core/components/inbox/modals/create-modal/index.ts`
- [ ] `core/components/inbox/modals/create-modal/issue-description.tsx`
- [ ] `core/components/inbox/modals/create-modal/issue-properties.tsx`
- [ ] `core/components/inbox/modals/create-modal/issue-title.tsx`
- [ ] `core/components/inbox/modals/create-modal/modal.tsx`

#### `core/components/inbox/sidebar/`

- [ ] `core/components/inbox/sidebar/inbox-list-item.tsx`
- [ ] `core/components/inbox/sidebar/inbox-list.tsx`
- [ ] `core/components/inbox/sidebar/index.ts`
- [ ] `core/components/inbox/sidebar/root.tsx`

#### `core/components/instance/`

- [ ] `core/components/instance/index.ts`
- [ ] `core/components/instance/maintenance-view.tsx`
- [ ] `core/components/instance/not-ready-view.tsx`

#### `core/components/integration/`

- [ ] `core/components/integration/confirm-action-modal.tsx`
- [ ] `core/components/integration/connected-account-details.tsx`
- [ ] `core/components/integration/oauth-callback-page.tsx`
- [ ] `core/components/integration/single-integration-card.tsx`
- [ ] `core/components/integration/utils.ts`

#### `core/components/integration/github/`

- [ ] `core/components/integration/github/personal-connect-card.tsx`
- [ ] `core/components/integration/github/pr-state-mapping-modal.tsx`
- [ ] `core/components/integration/github/pr-state-mapping.tsx`
- [ ] `core/components/integration/github/project-issue-sync-modal.tsx`
- [ ] `core/components/integration/github/project-issue-sync.tsx`
- [ ] `core/components/integration/github/select-repository.tsx`

#### `core/components/integration/slack/`

- [ ] `core/components/integration/slack/select-channel.tsx`

#### `core/components/issues/`

- [ ] `core/components/issues/archive-issue-modal.tsx`
- [ ] `core/components/issues/archived-issues-header.tsx`
- [ ] `core/components/issues/confirm-issue-discard.tsx`
- [ ] `core/components/issues/create-issue-toast-action-items.tsx`
- [ ] `core/components/issues/delete-issue-modal.tsx`
- [ ] `core/components/issues/filters.tsx`
- [ ] `core/components/issues/issue-update-status.tsx`
- [ ] `core/components/issues/label.tsx`
- [ ] `core/components/issues/layout-quick-actions.tsx`
- [ ] `core/components/issues/parent-issues-list-modal.tsx`
- [ ] `core/components/issues/title-input.tsx`

#### `core/components/issues/attachment/`

- [ ] `core/components/issues/attachment/attachment-detail.tsx`
- [ ] `core/components/issues/attachment/attachment-item-list.tsx`
- [ ] `core/components/issues/attachment/attachment-list-item.tsx`
- [ ] `core/components/issues/attachment/attachment-list-upload-item.tsx`
- [ ] `core/components/issues/attachment/attachment-upload-details.tsx`
- [ ] `core/components/issues/attachment/attachment-upload.tsx`
- [ ] `core/components/issues/attachment/attachments-list.tsx`
- [ ] `core/components/issues/attachment/delete-attachment-modal.tsx`
- [ ] `core/components/issues/attachment/index.ts`
- [ ] `core/components/issues/attachment/root.tsx`

#### `core/components/issues/bulk-operations/`

- [ ] `core/components/issues/bulk-operations/upgrade-banner.tsx`

#### `core/components/issues/issue-detail/`

- [ ] `core/components/issues/issue-detail/cycle-select.tsx`
- [ ] `core/components/issues/issue-detail/identifier-text.tsx`
- [ ] `core/components/issues/issue-detail/index.ts`
- [ ] `core/components/issues/issue-detail/issue-detail-quick-actions.tsx`
- [ ] `core/components/issues/issue-detail/main-content.tsx`
- [ ] `core/components/issues/issue-detail/module-select.tsx`
- [ ] `core/components/issues/issue-detail/parent-select.tsx`
- [ ] `core/components/issues/issue-detail/relation-select.tsx`
- [ ] `core/components/issues/issue-detail/root.tsx`
- [ ] `core/components/issues/issue-detail/sidebar.tsx`
- [ ] `core/components/issues/issue-detail/subscription.tsx`

#### `core/components/issues/issue-detail/issue-activity/`

- [ ] `core/components/issues/issue-detail/issue-activity/activity-comment-root.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity-filter.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/helper.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/index.ts`
- [ ] `core/components/issues/issue-detail/issue-activity/loader.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/root.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/sort-root.tsx`

#### `core/components/issues/issue-detail/issue-activity/activity/`

- [ ] `core/components/issues/issue-detail/issue-activity/activity/activity-list.tsx`

#### `core/components/issues/issue-detail/issue-activity/activity/actions/`

- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/archived-at.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/assignee.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/attachment.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/cycle.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/default.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/description.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/estimate.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/inbox.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/index.ts`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/label-activity-chip.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/label.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/link.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/module.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/name.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/parent.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/priority.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/relation.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/start_date.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/state.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/target_date.tsx`

#### `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/`

- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/activity-block.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/issue-link.tsx`
- [ ] `core/components/issues/issue-detail/issue-activity/activity/actions/helpers/issue-user.tsx`

#### `core/components/issues/issue-detail/label/`

- [ ] `core/components/issues/issue-detail/label/create-label.tsx`
- [ ] `core/components/issues/issue-detail/label/index.ts`
- [ ] `core/components/issues/issue-detail/label/label-list-item.tsx`
- [ ] `core/components/issues/issue-detail/label/label-list.tsx`
- [ ] `core/components/issues/issue-detail/label/root.tsx`

#### `core/components/issues/issue-detail/label/select/`

- [ ] `core/components/issues/issue-detail/label/select/label-select.tsx`
- [ ] `core/components/issues/issue-detail/label/select/root.tsx`

#### `core/components/issues/issue-detail/links/`

- [ ] `core/components/issues/issue-detail/links/create-update-link-modal.tsx`
- [ ] `core/components/issues/issue-detail/links/index.ts`
- [ ] `core/components/issues/issue-detail/links/link-detail.tsx`
- [ ] `core/components/issues/issue-detail/links/link-item.tsx`
- [ ] `core/components/issues/issue-detail/links/link-list.tsx`
- [ ] `core/components/issues/issue-detail/links/links.tsx`
- [ ] `core/components/issues/issue-detail/links/root.tsx`

#### `core/components/issues/issue-detail/parent/`

- [ ] `core/components/issues/issue-detail/parent/index.ts`
- [ ] `core/components/issues/issue-detail/parent/root.tsx`
- [ ] `core/components/issues/issue-detail/parent/sibling-item.tsx`
- [ ] `core/components/issues/issue-detail/parent/siblings.tsx`

#### `core/components/issues/issue-detail/reactions/`

- [ ] `core/components/issues/issue-detail/reactions/index.ts`
- [ ] `core/components/issues/issue-detail/reactions/issue-comment.tsx`
- [ ] `core/components/issues/issue-detail/reactions/issue.tsx`

#### `core/components/issues/issue-detail-widgets/`

- [ ] `core/components/issues/issue-detail-widgets/action-buttons.tsx`
- [ ] `core/components/issues/issue-detail-widgets/index.ts`
- [ ] `core/components/issues/issue-detail-widgets/issue-detail-widget-collapsibles.tsx`
- [ ] `core/components/issues/issue-detail-widgets/issue-detail-widget-modals.tsx`
- [ ] `core/components/issues/issue-detail-widgets/root.tsx`
- [ ] `core/components/issues/issue-detail-widgets/widget-button.tsx`

#### `core/components/issues/issue-detail-widgets/attachments/`

- [ ] `core/components/issues/issue-detail-widgets/attachments/content.tsx`
- [ ] `core/components/issues/issue-detail-widgets/attachments/helper.tsx`
- [ ] `core/components/issues/issue-detail-widgets/attachments/index.ts`
- [ ] `core/components/issues/issue-detail-widgets/attachments/quick-action-button.tsx`
- [ ] `core/components/issues/issue-detail-widgets/attachments/root.tsx`
- [ ] `core/components/issues/issue-detail-widgets/attachments/title.tsx`

#### `core/components/issues/issue-detail-widgets/links/`

- [ ] `core/components/issues/issue-detail-widgets/links/content.tsx`
- [ ] `core/components/issues/issue-detail-widgets/links/helper.tsx`
- [ ] `core/components/issues/issue-detail-widgets/links/index.ts`
- [ ] `core/components/issues/issue-detail-widgets/links/quick-action-button.tsx`
- [ ] `core/components/issues/issue-detail-widgets/links/root.tsx`
- [ ] `core/components/issues/issue-detail-widgets/links/title.tsx`

#### `core/components/issues/issue-detail-widgets/relations/`

- [ ] `core/components/issues/issue-detail-widgets/relations/content.tsx`
- [ ] `core/components/issues/issue-detail-widgets/relations/helper.tsx`
- [ ] `core/components/issues/issue-detail-widgets/relations/index.ts`
- [ ] `core/components/issues/issue-detail-widgets/relations/quick-action-button.tsx`
- [ ] `core/components/issues/issue-detail-widgets/relations/root.tsx`
- [ ] `core/components/issues/issue-detail-widgets/relations/title.tsx`

#### `core/components/issues/issue-detail-widgets/sub-issues/`

- [ ] `core/components/issues/issue-detail-widgets/sub-issues/content.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/display-filters.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/filters.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/helper.ts`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/index.ts`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/quick-action-button.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/root.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/title-actions.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/title.tsx`

#### `core/components/issues/issue-detail-widgets/sub-issues/issues-list/`

- [ ] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/list-group.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/list-item.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/properties.tsx`
- [ ] `core/components/issues/issue-detail-widgets/sub-issues/issues-list/root.tsx`

#### `core/components/issues/issue-layouts/`

- [ ] `core/components/issues/issue-layouts/group-drag-overlay.tsx`
- [ ] `core/components/issues/issue-layouts/issue-layout-HOC.tsx`
- [ ] `core/components/issues/issue-layouts/layout-icon.tsx`
- [ ] `core/components/issues/issue-layouts/utils.tsx`

#### `core/components/issues/issue-layouts/calendar/`

- [ ] `core/components/issues/issue-layouts/calendar/base-calendar-root.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/calendar.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/day-tile.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/header.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/issue-block-root.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/issue-block.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/issue-blocks.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/quick-add-issue-actions.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/utils.ts`
- [ ] `core/components/issues/issue-layouts/calendar/week-days.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/week-header.tsx`

#### `core/components/issues/issue-layouts/calendar/dropdowns/`

- [ ] `core/components/issues/issue-layouts/calendar/dropdowns/index.ts`
- [ ] `core/components/issues/issue-layouts/calendar/dropdowns/months-dropdown.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/dropdowns/options-dropdown.tsx`

#### `core/components/issues/issue-layouts/calendar/roots/`

- [ ] `core/components/issues/issue-layouts/calendar/roots/cycle-root.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/roots/module-root.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/roots/project-root.tsx`
- [ ] `core/components/issues/issue-layouts/calendar/roots/project-view-root.tsx`

#### `core/components/issues/issue-layouts/empty-states/`

- [ ] `core/components/issues/issue-layouts/empty-states/archived-issues.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/cycle.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/global-view.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/index.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/module.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/profile-view.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/project-epic.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/project-issues.tsx`
- [ ] `core/components/issues/issue-layouts/empty-states/project-view.tsx`

#### `core/components/issues/issue-layouts/filters/`

- [ ] `core/components/issues/issue-layouts/filters/index.ts`

#### `core/components/issues/issue-layouts/filters/applied-filters/`

- [ ] `core/components/issues/issue-layouts/filters/applied-filters/cycle.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/date.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/index.ts`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/label.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/members.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/module.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/priority.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/project.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/state-group.tsx`
- [ ] `core/components/issues/issue-layouts/filters/applied-filters/state.tsx`

#### `core/components/issues/issue-layouts/filters/header/`

- [ ] `core/components/issues/issue-layouts/filters/header/index.ts`
- [ ] `core/components/issues/issue-layouts/filters/header/layout-selection.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/mobile-layout-selection.tsx`

#### `core/components/issues/issue-layouts/filters/header/display-filters/`

- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/display-filters-selection.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/display-properties.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/extra-options.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/group-by.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/index.ts`
- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/order-by.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/display-filters/sub-group-by.tsx`

#### `core/components/issues/issue-layouts/filters/header/filters/`

- [ ] `core/components/issues/issue-layouts/filters/header/filters/assignee.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/created-by.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/cycle.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/due-date.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/index.ts`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/labels.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/mentions.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/module.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/priority.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/project.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/start-date.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/state-group.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/filters/state.tsx`

#### `core/components/issues/issue-layouts/filters/header/helpers/`

- [ ] `core/components/issues/issue-layouts/filters/header/helpers/dropdown.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/helpers/filter-header.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/helpers/filter-option.tsx`
- [ ] `core/components/issues/issue-layouts/filters/header/helpers/index.ts`

#### `core/components/issues/issue-layouts/gantt/`

- [ ] `core/components/issues/issue-layouts/gantt/base-gantt-root.tsx`
- [ ] `core/components/issues/issue-layouts/gantt/blocks.tsx`
- [ ] `core/components/issues/issue-layouts/gantt/index.ts`

#### `core/components/issues/issue-layouts/kanban/`

- [ ] `core/components/issues/issue-layouts/kanban/base-kanban-root.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/block.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/blocks-list.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/default.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/kanban-group.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/swimlanes.tsx`

#### `core/components/issues/issue-layouts/kanban/headers/`

- [ ] `core/components/issues/issue-layouts/kanban/headers/group-by-card.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/headers/sub-group-by-card.tsx`

#### `core/components/issues/issue-layouts/kanban/roots/`

- [ ] `core/components/issues/issue-layouts/kanban/roots/cycle-root.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/roots/module-root.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/roots/profile-issues-root.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/roots/project-root.tsx`
- [ ] `core/components/issues/issue-layouts/kanban/roots/project-view-root.tsx`

#### `core/components/issues/issue-layouts/list/`

- [ ] `core/components/issues/issue-layouts/list/base-list-root.tsx`
- [ ] `core/components/issues/issue-layouts/list/block-root.tsx`
- [ ] `core/components/issues/issue-layouts/list/block.tsx`
- [ ] `core/components/issues/issue-layouts/list/blocks-list.tsx`
- [ ] `core/components/issues/issue-layouts/list/default.tsx`
- [ ] `core/components/issues/issue-layouts/list/list-group.tsx`

#### `core/components/issues/issue-layouts/list/headers/`

- [ ] `core/components/issues/issue-layouts/list/headers/group-by-card.tsx`

#### `core/components/issues/issue-layouts/list/roots/`

- [ ] `core/components/issues/issue-layouts/list/roots/archived-issue-root.tsx`
- [ ] `core/components/issues/issue-layouts/list/roots/cycle-root.tsx`
- [x] `core/components/issues/issue-layouts/list/roots/module-root.tsx`
- [ ] `core/components/issues/issue-layouts/list/roots/profile-issues-root.tsx`
- [ ] `core/components/issues/issue-layouts/list/roots/project-root.tsx`
- [ ] `core/components/issues/issue-layouts/list/roots/project-view-root.tsx`

#### `core/components/issues/issue-layouts/properties/`

- [ ] `core/components/issues/issue-layouts/properties/all-properties.tsx`
- [ ] `core/components/issues/issue-layouts/properties/index.ts`
- [ ] `core/components/issues/issue-layouts/properties/label-dropdown.tsx`
- [ ] `core/components/issues/issue-layouts/properties/labels.tsx`
- [ ] `core/components/issues/issue-layouts/properties/with-display-properties-HOC.tsx`

#### `core/components/issues/issue-layouts/quick-action-dropdowns/`

- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/all-issue.tsx`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/archived-issue.tsx`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/cycle-issue.tsx`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/helper.tsx`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/index.ts`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/issue-detail.tsx`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/module-issue.tsx`
- [ ] `core/components/issues/issue-layouts/quick-action-dropdowns/project-issue.tsx`

#### `core/components/issues/issue-layouts/quick-add/`

- [ ] `core/components/issues/issue-layouts/quick-add/index.ts`
- [ ] `core/components/issues/issue-layouts/quick-add/root.tsx`

#### `core/components/issues/issue-layouts/quick-add/button/`

- [ ] `core/components/issues/issue-layouts/quick-add/button/gantt.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/button/index.ts`
- [ ] `core/components/issues/issue-layouts/quick-add/button/kanban.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/button/list.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/button/spreadsheet.tsx`

#### `core/components/issues/issue-layouts/quick-add/form/`

- [ ] `core/components/issues/issue-layouts/quick-add/form/calendar.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/form/gantt.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/form/index.ts`
- [ ] `core/components/issues/issue-layouts/quick-add/form/kanban.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/form/list.tsx`
- [ ] `core/components/issues/issue-layouts/quick-add/form/spreadsheet.tsx`

#### `core/components/issues/issue-layouts/roots/`

- [ ] `core/components/issues/issue-layouts/roots/all-issue-layout-root.tsx`
- [ ] `core/components/issues/issue-layouts/roots/archived-issue-layout-root.tsx`
- [ ] `core/components/issues/issue-layouts/roots/cycle-layout-root.tsx`
- [ ] `core/components/issues/issue-layouts/roots/module-layout-root.tsx`
- [ ] `core/components/issues/issue-layouts/roots/project-layout-root.tsx`
- [ ] `core/components/issues/issue-layouts/roots/project-view-layout-root.tsx`

#### `core/components/issues/issue-layouts/spreadsheet/`

- [ ] `core/components/issues/issue-layouts/spreadsheet/base-spreadsheet-root.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/issue-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/issue-row.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-header-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-header.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-table.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/spreadsheet-view.tsx`

#### `core/components/issues/issue-layouts/spreadsheet/columns/`

- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/assignee-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/attachment-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/created-on-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/cycle-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/due-date-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/estimate-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/header-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/index.ts`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/label-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/link-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/module-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/priority-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/start-date-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/state-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/sub-issue-column.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/columns/updated-on-column.tsx`

#### `core/components/issues/issue-layouts/spreadsheet/roots/`

- [ ] `core/components/issues/issue-layouts/spreadsheet/roots/cycle-root.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/roots/module-root.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/roots/project-root.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/roots/project-view-root.tsx`
- [ ] `core/components/issues/issue-layouts/spreadsheet/roots/workspace-root.tsx`

#### `core/components/issues/issue-modal/`

- [ ] `core/components/issues/issue-modal/base.tsx`
- [ ] `core/components/issues/issue-modal/draft-issue-layout.tsx`
- [ ] `core/components/issues/issue-modal/form.tsx`
- [ ] `core/components/issues/issue-modal/modal.tsx`

#### `core/components/issues/issue-modal/components/`

- [ ] `core/components/issues/issue-modal/components/default-properties.tsx`
- [ ] `core/components/issues/issue-modal/components/description-editor.tsx`
- [ ] `core/components/issues/issue-modal/components/index.ts`
- [ ] `core/components/issues/issue-modal/components/parent-tag.tsx`
- [ ] `core/components/issues/issue-modal/components/project-select.tsx`
- [ ] `core/components/issues/issue-modal/components/title-input.tsx`

#### `core/components/issues/issue-modal/context/`

- [ ] `core/components/issues/issue-modal/context/index.ts`
- [ ] `core/components/issues/issue-modal/context/issue-modal-context.tsx`

#### `core/components/issues/peek-overview/`

- [ ] `core/components/issues/peek-overview/error.tsx`
- [ ] `core/components/issues/peek-overview/header.tsx`
- [ ] `core/components/issues/peek-overview/index.ts`
- [ ] `core/components/issues/peek-overview/issue-detail.tsx`
- [ ] `core/components/issues/peek-overview/loader.tsx`
- [ ] `core/components/issues/peek-overview/properties.tsx`
- [ ] `core/components/issues/peek-overview/root.tsx`
- [ ] `core/components/issues/peek-overview/view.tsx`

#### `core/components/issues/preview-card/`

- [ ] `core/components/issues/preview-card/date.tsx`
- [ ] `core/components/issues/preview-card/index.ts`
- [ ] `core/components/issues/preview-card/root.tsx`

#### `core/components/issues/relations/`

- [ ] `core/components/issues/relations/issue-list-item.tsx`
- [ ] `core/components/issues/relations/issue-list.tsx`
- [ ] `core/components/issues/relations/properties.tsx`

#### `core/components/issues/select/`

- [ ] `core/components/issues/select/base.tsx`
- [ ] `core/components/issues/select/dropdown.tsx`
- [ ] `core/components/issues/select/index.ts`

#### `core/components/issues/workspace-draft/`

- [ ] `core/components/issues/workspace-draft/delete-modal.tsx`
- [ ] `core/components/issues/workspace-draft/draft-issue-block.tsx`
- [ ] `core/components/issues/workspace-draft/draft-issue-properties.tsx`
- [ ] `core/components/issues/workspace-draft/empty-state.tsx`
- [ ] `core/components/issues/workspace-draft/index.ts`
- [ ] `core/components/issues/workspace-draft/loader.tsx`
- [ ] `core/components/issues/workspace-draft/quick-action.tsx`
- [ ] `core/components/issues/workspace-draft/root.tsx`

#### `core/components/labels/`

- [ ] `core/components/labels/create-update-label-inline.tsx`
- [ ] `core/components/labels/delete-label-modal.tsx`
- [ ] `core/components/labels/index.ts`
- [ ] `core/components/labels/label-drag-n-drop-HOC.tsx`
- [ ] `core/components/labels/label-utils.ts`
- [ ] `core/components/labels/project-setting-label-group.tsx`
- [ ] `core/components/labels/project-setting-label-item.tsx`
- [ ] `core/components/labels/project-setting-label-list.tsx`

#### `core/components/labels/label-block/`

- [ ] `core/components/labels/label-block/label-item-block.tsx`
- [ ] `core/components/labels/label-block/label-name.tsx`

#### `core/components/license/`

- [ ] `core/components/license/index.ts`

#### `core/components/license/modal/`

- [ ] `core/components/license/modal/index.ts`

#### `core/components/license/modal/card/`

- [ ] `core/components/license/modal/card/base-paid-plan-card.tsx`
- [ ] `core/components/license/modal/card/checkout-button.tsx`
- [ ] `core/components/license/modal/card/discount-info.tsx`
- [ ] `core/components/license/modal/card/free-plan.tsx`
- [ ] `core/components/license/modal/card/index.ts`
- [ ] `core/components/license/modal/card/plan-upgrade.tsx`
- [ ] `core/components/license/modal/card/talk-to-sales.tsx`

#### `core/components/modules/`

- [ ] `core/components/modules/delete-module-modal.tsx`
- [ ] `core/components/modules/form.tsx`
- [ ] `core/components/modules/index.ts`
- [ ] `core/components/modules/modal.tsx`
- [ ] `core/components/modules/module-card-item.tsx`
- [ ] `core/components/modules/module-layout-icon.tsx`
- [ ] `core/components/modules/module-list-item-action.tsx`
- [ ] `core/components/modules/module-list-item.tsx`
- [ ] `core/components/modules/module-peek-overview.tsx`
- [ ] `core/components/modules/module-status-dropdown.tsx`
- [ ] `core/components/modules/module-view-header.tsx`
- [ ] `core/components/modules/modules-list-view.tsx`
- [ ] `core/components/modules/quick-actions.tsx`

#### `core/components/modules/analytics-sidebar/`

- [ ] `core/components/modules/analytics-sidebar/index.ts`
- [ ] `core/components/modules/analytics-sidebar/issue-progress.tsx`
- [ ] `core/components/modules/analytics-sidebar/progress-stats.tsx`
- [ ] `core/components/modules/analytics-sidebar/root.tsx`

#### `core/components/modules/applied-filters/`

- [ ] `core/components/modules/applied-filters/date.tsx`
- [ ] `core/components/modules/applied-filters/index.ts`
- [ ] `core/components/modules/applied-filters/members.tsx`
- [ ] `core/components/modules/applied-filters/root.tsx`
- [ ] `core/components/modules/applied-filters/status.tsx`

#### `core/components/modules/archived-modules/`

- [ ] `core/components/modules/archived-modules/header.tsx`
- [ ] `core/components/modules/archived-modules/index.ts`
- [ ] `core/components/modules/archived-modules/modal.tsx`
- [ ] `core/components/modules/archived-modules/root.tsx`
- [ ] `core/components/modules/archived-modules/view.tsx`

#### `core/components/modules/dropdowns/`

- [ ] `core/components/modules/dropdowns/index.ts`
- [ ] `core/components/modules/dropdowns/order-by.tsx`

#### `core/components/modules/dropdowns/filters/`

- [ ] `core/components/modules/dropdowns/filters/index.ts`
- [ ] `core/components/modules/dropdowns/filters/lead.tsx`
- [ ] `core/components/modules/dropdowns/filters/members.tsx`
- [ ] `core/components/modules/dropdowns/filters/root.tsx`
- [ ] `core/components/modules/dropdowns/filters/start-date.tsx`
- [ ] `core/components/modules/dropdowns/filters/status.tsx`
- [ ] `core/components/modules/dropdowns/filters/target-date.tsx`

#### `core/components/modules/gantt-chart/`

- [ ] `core/components/modules/gantt-chart/blocks.tsx`
- [ ] `core/components/modules/gantt-chart/index.ts`
- [ ] `core/components/modules/gantt-chart/modules-list-layout.tsx`

#### `core/components/modules/links/`

- [ ] `core/components/modules/links/create-update-modal.tsx`
- [ ] `core/components/modules/links/index.ts`
- [ ] `core/components/modules/links/list-item.tsx`
- [ ] `core/components/modules/links/list.tsx`

#### `core/components/modules/select/`

- [ ] `core/components/modules/select/index.ts`
- [ ] `core/components/modules/select/status.tsx`

#### `core/components/modules/sidebar-select/`

- [ ] `core/components/modules/sidebar-select/index.ts`
- [ ] `core/components/modules/sidebar-select/select-status.tsx`

#### `core/components/navigation/`

- [ ] `core/components/navigation/app-rail-root.tsx`
- [ ] `core/components/navigation/customize-navigation-dialog.tsx`
- [ ] `core/components/navigation/index.ts`
- [ ] `core/components/navigation/items-root.tsx`
- [ ] `core/components/navigation/project-actions-menu.tsx`
- [ ] `core/components/navigation/project-header-button.tsx`
- [ ] `core/components/navigation/project-header.tsx`
- [ ] `core/components/navigation/tab-navigation-overflow-menu.tsx`
- [ ] `core/components/navigation/tab-navigation-root.tsx`
- [ ] `core/components/navigation/tab-navigation-utils.ts`
- [ ] `core/components/navigation/tab-navigation-visible-item.tsx`
- [ ] `core/components/navigation/top-nav-power-k.tsx`
- [ ] `core/components/navigation/use-active-tab.ts`
- [ ] `core/components/navigation/use-project-actions.ts`
- [ ] `core/components/navigation/use-responsive-tab-layout.ts`
- [ ] `core/components/navigation/use-tab-preferences.ts`

#### `core/components/onboarding/`

- [ ] `core/components/onboarding/create-or-join-workspaces.tsx`
- [ ] `core/components/onboarding/create-workspace.tsx`
- [ ] `core/components/onboarding/header.tsx`
- [ ] `core/components/onboarding/index.ts`
- [ ] `core/components/onboarding/invitations.tsx`
- [ ] `core/components/onboarding/invite-members.tsx`
- [ ] `core/components/onboarding/profile-setup.tsx`
- [ ] `core/components/onboarding/root.tsx`
- [ ] `core/components/onboarding/step-indicator.tsx`
- [ ] `core/components/onboarding/switch-account-dropdown.tsx`
- [ ] `core/components/onboarding/switch-account-modal.tsx`

#### `core/components/onboarding/steps/`

- [ ] `core/components/onboarding/steps/index.ts`
- [ ] `core/components/onboarding/steps/root.tsx`

#### `core/components/onboarding/steps/common/`

- [ ] `core/components/onboarding/steps/common/header.tsx`
- [ ] `core/components/onboarding/steps/common/index.ts`

#### `core/components/onboarding/steps/profile/`

- [ ] `core/components/onboarding/steps/profile/consent.tsx`
- [ ] `core/components/onboarding/steps/profile/index.ts`
- [ ] `core/components/onboarding/steps/profile/root.tsx`
- [ ] `core/components/onboarding/steps/profile/set-password.tsx`

#### `core/components/onboarding/steps/role/`

- [ ] `core/components/onboarding/steps/role/index.ts`
- [ ] `core/components/onboarding/steps/role/root.tsx`

#### `core/components/onboarding/steps/team/`

- [ ] `core/components/onboarding/steps/team/index.ts`
- [ ] `core/components/onboarding/steps/team/root.tsx`

#### `core/components/onboarding/steps/usecase/`

- [ ] `core/components/onboarding/steps/usecase/index.ts`
- [ ] `core/components/onboarding/steps/usecase/root.tsx`

#### `core/components/onboarding/steps/workspace/`

- [ ] `core/components/onboarding/steps/workspace/create.tsx`
- [ ] `core/components/onboarding/steps/workspace/index.ts`
- [ ] `core/components/onboarding/steps/workspace/join-invites.tsx`
- [ ] `core/components/onboarding/steps/workspace/root.tsx`

#### `core/components/pages/`

- [ ] `core/components/pages/pages-list-main-content.tsx`
- [ ] `core/components/pages/pages-list-view.tsx`

#### `core/components/pages/dropdowns/`

- [ ] `core/components/pages/dropdowns/actions.tsx`
- [ ] `core/components/pages/dropdowns/index.ts`

#### `core/components/pages/editor/`

- [ ] `core/components/pages/editor/content-limit-banner.tsx`
- [ ] `core/components/pages/editor/editor-body.tsx`
- [ ] `core/components/pages/editor/page-root.tsx`
- [ ] `core/components/pages/editor/title.tsx`

#### `core/components/pages/editor/header/`

- [ ] `core/components/pages/editor/header/index.ts`
- [ ] `core/components/pages/editor/header/logo-picker.tsx`
- [ ] `core/components/pages/editor/header/root.tsx`

#### `core/components/pages/editor/summary/`

- [ ] `core/components/pages/editor/summary/content-browser.tsx`
- [ ] `core/components/pages/editor/summary/heading-components.tsx`
- [ ] `core/components/pages/editor/summary/index.ts`

#### `core/components/pages/editor/toolbar/`

- [ ] `core/components/pages/editor/toolbar/color-dropdown.tsx`
- [ ] `core/components/pages/editor/toolbar/index.ts`
- [ ] `core/components/pages/editor/toolbar/options-dropdown.tsx`
- [ ] `core/components/pages/editor/toolbar/root.tsx`
- [ ] `core/components/pages/editor/toolbar/toolbar.tsx`

#### `core/components/pages/header/`

- [ ] `core/components/pages/header/actions.tsx`
- [ ] `core/components/pages/header/archived-badge.tsx`
- [ ] `core/components/pages/header/copy-link-control.tsx`
- [ ] `core/components/pages/header/favorite-control.tsx`
- [ ] `core/components/pages/header/index.ts`
- [ ] `core/components/pages/header/offline-badge.tsx`
- [ ] `core/components/pages/header/root.tsx`
- [ ] `core/components/pages/header/syncing-badge.tsx`

#### `core/components/pages/list/`

- [ ] `core/components/pages/list/block-item-action.tsx`
- [ ] `core/components/pages/list/block.tsx`
- [ ] `core/components/pages/list/index.ts`
- [ ] `core/components/pages/list/order-by.tsx`
- [ ] `core/components/pages/list/root.tsx`
- [ ] `core/components/pages/list/search-input.tsx`
- [ ] `core/components/pages/list/tab-navigation.tsx`

#### `core/components/pages/list/applied-filters/`

- [ ] `core/components/pages/list/applied-filters/index.ts`
- [ ] `core/components/pages/list/applied-filters/root.tsx`

#### `core/components/pages/list/filters/`

- [ ] `core/components/pages/list/filters/index.ts`
- [ ] `core/components/pages/list/filters/root.tsx`

#### `core/components/pages/loaders/`

- [ ] `core/components/pages/loaders/page-content-loader.tsx`
- [ ] `core/components/pages/loaders/page-loader.tsx`

#### `core/components/pages/modals/`

- [ ] `core/components/pages/modals/create-page-modal.tsx`
- [ ] `core/components/pages/modals/delete-page-modal.tsx`
- [ ] `core/components/pages/modals/export-page-modal.tsx`
- [ ] `core/components/pages/modals/page-form.tsx`

#### `core/components/pages/navigation-pane/`

- [ ] `core/components/pages/navigation-pane/index.ts`
- [ ] `core/components/pages/navigation-pane/root.tsx`
- [ ] `core/components/pages/navigation-pane/tabs-list.tsx`

#### `core/components/pages/navigation-pane/tab-panels/`

- [ ] `core/components/pages/navigation-pane/tab-panels/assets.tsx`
- [ ] `core/components/pages/navigation-pane/tab-panels/outline.tsx`
- [ ] `core/components/pages/navigation-pane/tab-panels/root.tsx`

#### `core/components/pages/navigation-pane/tab-panels/info/`

- [ ] `core/components/pages/navigation-pane/tab-panels/info/actors-info.tsx`
- [ ] `core/components/pages/navigation-pane/tab-panels/info/document-info.tsx`
- [ ] `core/components/pages/navigation-pane/tab-panels/info/root.tsx`
- [ ] `core/components/pages/navigation-pane/tab-panels/info/version-history.tsx`

#### `core/components/pages/navigation-pane/types/`

- [ ] `core/components/pages/navigation-pane/types/extensions.ts`
- [ ] `core/components/pages/navigation-pane/types/index.ts`

#### `core/components/pages/version/`

- [ ] `core/components/pages/version/editor.tsx`
- [ ] `core/components/pages/version/index.ts`
- [ ] `core/components/pages/version/main-content.tsx`
- [ ] `core/components/pages/version/root.tsx`

#### `core/components/power-k/`

- [ ] `core/components/power-k/global-shortcuts.tsx`
- [ ] `core/components/power-k/projects-app-provider.tsx`

#### `core/components/power-k/actions/`

- [ ] `core/components/power-k/actions/helper.ts`

#### `core/components/power-k/config/`

- [ ] `core/components/power-k/config/account-commands.ts`
- [ ] `core/components/power-k/config/commands.ts`
- [ ] `core/components/power-k/config/help-commands.ts`
- [ ] `core/components/power-k/config/miscellaneous-commands.ts`
- [ ] `core/components/power-k/config/preferences-commands.ts`

#### `core/components/power-k/config/creation/`

- [ ] `core/components/power-k/config/creation/command.ts`
- [ ] `core/components/power-k/config/creation/root.ts`

#### `core/components/power-k/config/navigation/`

- [ ] `core/components/power-k/config/navigation/commands.ts`
- [ ] `core/components/power-k/config/navigation/root.ts`

#### `core/components/power-k/core/`

- [ ] `core/components/power-k/core/context-detector.ts`
- [ ] `core/components/power-k/core/registry.ts`
- [ ] `core/components/power-k/core/shortcut-handler.ts`
- [ ] `core/components/power-k/core/types.ts`

#### `core/components/power-k/hooks/`

- [ ] `core/components/power-k/hooks/use-context-indicator.ts`

#### `core/components/power-k/menus/`

- [ ] `core/components/power-k/menus/builder.tsx`
- [ ] `core/components/power-k/menus/cycles.tsx`
- [ ] `core/components/power-k/menus/empty-state.tsx`
- [ ] `core/components/power-k/menus/labels.tsx`
- [ ] `core/components/power-k/menus/members.tsx`
- [ ] `core/components/power-k/menus/modules.tsx`
- [ ] `core/components/power-k/menus/projects.tsx`
- [ ] `core/components/power-k/menus/settings.tsx`
- [ ] `core/components/power-k/menus/views.tsx`
- [ ] `core/components/power-k/menus/workspaces.tsx`

#### `core/components/power-k/ui/modal/`

- [ ] `core/components/power-k/ui/modal/command-item-shortcut-badge.tsx`
- [ ] `core/components/power-k/ui/modal/command-item.tsx`
- [ ] `core/components/power-k/ui/modal/commands-list.tsx`
- [ ] `core/components/power-k/ui/modal/constants.ts`
- [ ] `core/components/power-k/ui/modal/context-indicator.tsx`
- [ ] `core/components/power-k/ui/modal/footer.tsx`
- [ ] `core/components/power-k/ui/modal/header.tsx`
- [ ] `core/components/power-k/ui/modal/search-menu.tsx`
- [ ] `core/components/power-k/ui/modal/search-results-map.tsx`
- [ ] `core/components/power-k/ui/modal/search-results.tsx`
- [ ] `core/components/power-k/ui/modal/shortcuts-root.tsx`
- [ ] `core/components/power-k/ui/modal/wrapper.tsx`

#### `core/components/power-k/ui/pages/`

- [ ] `core/components/power-k/ui/pages/default.tsx`
- [ ] `core/components/power-k/ui/pages/index.ts`
- [ ] `core/components/power-k/ui/pages/root.tsx`
- [ ] `core/components/power-k/ui/pages/work-item-selection-page.tsx`

#### `core/components/power-k/ui/pages/context-based/`

- [ ] `core/components/power-k/ui/pages/context-based/index.ts`
- [ ] `core/components/power-k/ui/pages/context-based/root.tsx`

#### `core/components/power-k/ui/pages/context-based/cycle/`

- [ ] `core/components/power-k/ui/pages/context-based/cycle/commands.ts`

#### `core/components/power-k/ui/pages/context-based/module/`

- [ ] `core/components/power-k/ui/pages/context-based/module/commands.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/module/index.ts`
- [ ] `core/components/power-k/ui/pages/context-based/module/root.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/module/status-menu.tsx`

#### `core/components/power-k/ui/pages/context-based/page/`

- [ ] `core/components/power-k/ui/pages/context-based/page/commands.ts`

#### `core/components/power-k/ui/pages/context-based/work-item/`

- [ ] `core/components/power-k/ui/pages/context-based/work-item/commands.ts`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/cycles-menu.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/estimates-menu.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/index.ts`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/labels-menu.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/modules-menu.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/priorities-menu.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/root.tsx`
- [ ] `core/components/power-k/ui/pages/context-based/work-item/states-menu.tsx`

#### `core/components/power-k/ui/pages/open-entity/`

- [ ] `core/components/power-k/ui/pages/open-entity/project-cycles-menu.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/project-modules-menu.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/project-settings-menu.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/project-views-menu.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/projects-menu.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/root.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/shared.ts`
- [ ] `core/components/power-k/ui/pages/open-entity/workspace-settings-menu.tsx`
- [ ] `core/components/power-k/ui/pages/open-entity/workspaces-menu.tsx`

#### `core/components/power-k/ui/pages/preferences/`

- [ ] `core/components/power-k/ui/pages/preferences/index.ts`
- [ ] `core/components/power-k/ui/pages/preferences/languages-menu.tsx`
- [ ] `core/components/power-k/ui/pages/preferences/root.tsx`
- [ ] `core/components/power-k/ui/pages/preferences/start-of-week-menu.tsx`
- [ ] `core/components/power-k/ui/pages/preferences/themes-menu.tsx`
- [ ] `core/components/power-k/ui/pages/preferences/timezone-menu.tsx`

#### `core/components/power-k/ui/renderer/`

- [ ] `core/components/power-k/ui/renderer/command.tsx`
- [ ] `core/components/power-k/ui/renderer/shared.ts`
- [ ] `core/components/power-k/ui/renderer/shortcut.tsx`

#### `core/components/power-k/utils/`

- [ ] `core/components/power-k/utils/navigation.ts`

#### `core/components/profile/`

- [ ] `core/components/profile/profile-issues-filter.tsx`
- [ ] `core/components/profile/profile-issues.tsx`
- [ ] `core/components/profile/profile-setting-content-wrapper.tsx`
- [ ] `core/components/profile/sidebar.tsx`
- [ ] `core/components/profile/start-of-week-preference.tsx`
- [ ] `core/components/profile/time.tsx`

#### `core/components/profile/activity/`

- [ ] `core/components/profile/activity/activity-list.tsx`
- [ ] `core/components/profile/activity/download-button.tsx`
- [ ] `core/components/profile/activity/profile-activity-list.tsx`
- [ ] `core/components/profile/activity/workspace-activity-list.tsx`

#### `core/components/profile/overview/`

- [ ] `core/components/profile/overview/activity.tsx`
- [ ] `core/components/profile/overview/priority-distribution.tsx`
- [ ] `core/components/profile/overview/state-distribution.tsx`
- [ ] `core/components/profile/overview/stats.tsx`
- [ ] `core/components/profile/overview/workload.tsx`

#### `core/components/project/`

- [ ] `core/components/project/archive-restore-modal.tsx`
- [ ] `core/components/project/card-list.tsx`
- [ ] `core/components/project/card.tsx`
- [ ] `core/components/project/confirm-project-member-remove.tsx`
- [ ] `core/components/project/create-project-modal.tsx`
- [ ] `core/components/project/delete-project-modal.tsx`
- [ ] `core/components/project/empty-state.tsx`
- [ ] `core/components/project/filters.tsx`
- [ ] `core/components/project/form-loader.tsx`
- [ ] `core/components/project/form.tsx`
- [ ] `core/components/project/header.tsx`
- [ ] `core/components/project/integration-card.tsx`
- [ ] `core/components/project/join-project-modal.tsx`
- [ ] `core/components/project/leave-project-modal.tsx`
- [ ] `core/components/project/member-header-column.tsx`
- [ ] `core/components/project/member-list-item.tsx`
- [ ] `core/components/project/member-list.tsx`
- [ ] `core/components/project/member-select.tsx`
- [ ] `core/components/project/multi-select-modal.tsx`
- [ ] `core/components/project/project-feature-update.tsx`
- [ ] `core/components/project/project-network-icon.tsx`
- [ ] `core/components/project/project-settings-member-defaults.tsx`
- [ ] `core/components/project/root.tsx`
- [ ] `core/components/project/search-projects.tsx`
- [ ] `core/components/project/send-project-invitation-modal.tsx`

#### `core/components/project/applied-filters/`

- [ ] `core/components/project/applied-filters/access.tsx`
- [ ] `core/components/project/applied-filters/date.tsx`
- [ ] `core/components/project/applied-filters/index.ts`
- [ ] `core/components/project/applied-filters/members.tsx`
- [ ] `core/components/project/applied-filters/project-display-filters.tsx`
- [ ] `core/components/project/applied-filters/root.tsx`

#### `core/components/project/create/`

- [ ] `core/components/project/create/common-attributes.tsx`
- [ ] `core/components/project/create/header.tsx`
- [ ] `core/components/project/create/project-create-buttons.tsx`

#### `core/components/project/dropdowns/`

- [ ] `core/components/project/dropdowns/order-by.tsx`

#### `core/components/project/dropdowns/filters/`

- [ ] `core/components/project/dropdowns/filters/access.tsx`
- [x] `core/components/project/dropdowns/filters/created-at.tsx`
- [ ] `core/components/project/dropdowns/filters/index.ts`
- [ ] `core/components/project/dropdowns/filters/lead.tsx`
- [ ] `core/components/project/dropdowns/filters/member-list.tsx`
- [ ] `core/components/project/dropdowns/filters/members.tsx`
- [ ] `core/components/project/dropdowns/filters/root.tsx`

#### `core/components/project/publish-project/`

- [ ] `core/components/project/publish-project/modal.tsx`

#### `core/components/project/settings/`

- [ ] `core/components/project/settings/control-section.tsx`
- [ ] `core/components/project/settings/features-list.tsx`
- [ ] `core/components/project/settings/helper.tsx`
- [ ] `core/components/project/settings/member-columns.tsx`

#### `core/components/project-states/`

- [ ] `core/components/project-states/group-item.tsx`
- [ ] `core/components/project-states/group-list.tsx`
- [ ] `core/components/project-states/index.ts`
- [ ] `core/components/project-states/loader.tsx`
- [ ] `core/components/project-states/root.tsx`
- [ ] `core/components/project-states/state-delete-modal.tsx`
- [ ] `core/components/project-states/state-item-title.tsx`
- [ ] `core/components/project-states/state-item.tsx`
- [ ] `core/components/project-states/state-list.tsx`

#### `core/components/project-states/create-update/`

- [ ] `core/components/project-states/create-update/create.tsx`
- [ ] `core/components/project-states/create-update/form.tsx`
- [ ] `core/components/project-states/create-update/index.ts`
- [ ] `core/components/project-states/create-update/update.tsx`

#### `core/components/project-states/options/`

- [ ] `core/components/project-states/options/delete.tsx`
- [ ] `core/components/project-states/options/index.ts`
- [ ] `core/components/project-states/options/mark-as-default.tsx`

#### `core/components/readonly/`

- [ ] `core/components/readonly/cycle.tsx`
- [ ] `core/components/readonly/date.tsx`
- [ ] `core/components/readonly/estimate.tsx`
- [ ] `core/components/readonly/index.tsx`
- [ ] `core/components/readonly/labels.tsx`
- [ ] `core/components/readonly/member.tsx`
- [ ] `core/components/readonly/module.tsx`
- [ ] `core/components/readonly/priority.tsx`
- [ ] `core/components/readonly/state.tsx`

#### `core/components/rich-filters/`

- [ ] `core/components/rich-filters/filters-row.tsx`
- [ ] `core/components/rich-filters/filters-toggle.tsx`
- [ ] `core/components/rich-filters/shared.ts`

#### `core/components/rich-filters/add-filters/`

- [ ] `core/components/rich-filters/add-filters/button.tsx`
- [ ] `core/components/rich-filters/add-filters/dropdown.tsx`

#### `core/components/rich-filters/filter-item/`

- [ ] `core/components/rich-filters/filter-item/close-button.tsx`
- [ ] `core/components/rich-filters/filter-item/container.tsx`
- [ ] `core/components/rich-filters/filter-item/invalid.tsx`
- [ ] `core/components/rich-filters/filter-item/loader.tsx`
- [ ] `core/components/rich-filters/filter-item/property.tsx`
- [ ] `core/components/rich-filters/filter-item/root.tsx`

#### `core/components/rich-filters/filter-value-input/`

- [ ] `core/components/rich-filters/filter-value-input/root.tsx`

#### `core/components/rich-filters/filter-value-input/date/`

- [ ] `core/components/rich-filters/filter-value-input/date/range.tsx`
- [ ] `core/components/rich-filters/filter-value-input/date/single.tsx`

#### `core/components/rich-filters/filter-value-input/select/`

- [ ] `core/components/rich-filters/filter-value-input/select/multi.tsx`
- [ ] `core/components/rich-filters/filter-value-input/select/selected-options-display.tsx`
- [ ] `core/components/rich-filters/filter-value-input/select/shared.tsx`
- [ ] `core/components/rich-filters/filter-value-input/select/single.tsx`

#### `core/components/settings/`

- [ ] `core/components/settings/boxed-control-item.tsx`
- [ ] `core/components/settings/content-wrapper.tsx`
- [ ] `core/components/settings/control-item.tsx`
- [ ] `core/components/settings/heading.tsx`
- [ ] `core/components/settings/helper.ts`
- [ ] `core/components/settings/layout.tsx`
- [ ] `core/components/settings/page-header.tsx`

#### `core/components/settings/mobile/`

- [ ] `core/components/settings/mobile/nav.tsx`

#### `core/components/settings/profile/`

- [ ] `core/components/settings/profile/heading.tsx`
- [ ] `core/components/settings/profile/modal.tsx`

#### `core/components/settings/profile/content/`

- [ ] `core/components/settings/profile/content/index.ts`
- [ ] `core/components/settings/profile/content/root.tsx`

#### `core/components/settings/profile/content/pages/`

- [ ] `core/components/settings/profile/content/pages/api-tokens.tsx`
- [ ] `core/components/settings/profile/content/pages/index.ts`
- [ ] `core/components/settings/profile/content/pages/security.tsx`

#### `core/components/settings/profile/content/pages/activity/`

- [ ] `core/components/settings/profile/content/pages/activity/activity-list.tsx`
- [ ] `core/components/settings/profile/content/pages/activity/index.ts`
- [ ] `core/components/settings/profile/content/pages/activity/root.tsx`

#### `core/components/settings/profile/content/pages/general/`

- [ ] `core/components/settings/profile/content/pages/general/form.tsx`
- [ ] `core/components/settings/profile/content/pages/general/index.ts`
- [ ] `core/components/settings/profile/content/pages/general/root.tsx`

#### `core/components/settings/profile/content/pages/notifications/`

- [ ] `core/components/settings/profile/content/pages/notifications/email-notification-form.tsx`
- [ ] `core/components/settings/profile/content/pages/notifications/index.ts`
- [ ] `core/components/settings/profile/content/pages/notifications/root.tsx`

#### `core/components/settings/profile/content/pages/preferences/`

- [ ] `core/components/settings/profile/content/pages/preferences/default-list.tsx`
- [ ] `core/components/settings/profile/content/pages/preferences/index.ts`
- [ ] `core/components/settings/profile/content/pages/preferences/language-and-timezone-list.tsx`
- [ ] `core/components/settings/profile/content/pages/preferences/root.tsx`

#### `core/components/settings/profile/sidebar/`

- [ ] `core/components/settings/profile/sidebar/header.tsx`
- [ ] `core/components/settings/profile/sidebar/index.ts`
- [ ] `core/components/settings/profile/sidebar/item-categories.tsx`
- [ ] `core/components/settings/profile/sidebar/root.tsx`
- [ ] `core/components/settings/profile/sidebar/workspace-options.tsx`

#### `core/components/settings/project/content/`

- [ ] `core/components/settings/project/content/feature-control-item.tsx`

#### `core/components/settings/project/sidebar/`

- [ ] `core/components/settings/project/sidebar/header.tsx`
- [ ] `core/components/settings/project/sidebar/index.ts`
- [ ] `core/components/settings/project/sidebar/item-categories.tsx`
- [ ] `core/components/settings/project/sidebar/item-icon.tsx`
- [ ] `core/components/settings/project/sidebar/root.tsx`

#### `core/components/settings/sidebar/`

- [ ] `core/components/settings/sidebar/item.tsx`

#### `core/components/settings/workspace/sidebar/`

- [ ] `core/components/settings/workspace/sidebar/header.tsx`
- [ ] `core/components/settings/workspace/sidebar/index.ts`
- [ ] `core/components/settings/workspace/sidebar/item-categories.tsx`
- [ ] `core/components/settings/workspace/sidebar/item-icon.tsx`
- [ ] `core/components/settings/workspace/sidebar/root.tsx`

#### `core/components/sidebar/`

- [ ] `core/components/sidebar/add-button.tsx`
- [ ] `core/components/sidebar/resizable-sidebar.tsx`
- [ ] `core/components/sidebar/search-button.tsx`
- [ ] `core/components/sidebar/sidebar-item.tsx`
- [ ] `core/components/sidebar/sidebar-navigation.tsx`
- [ ] `core/components/sidebar/sidebar-toggle-button.tsx`
- [ ] `core/components/sidebar/sidebar-wrapper.tsx`

#### `core/components/stickies/`

- [ ] `core/components/stickies/action-bar.tsx`
- [ ] `core/components/stickies/delete-modal.tsx`
- [ ] `core/components/stickies/widget.tsx`

#### `core/components/stickies/layout/`

- [ ] `core/components/stickies/layout/stickies-infinite.tsx`
- [ ] `core/components/stickies/layout/stickies-list.tsx`
- [ ] `core/components/stickies/layout/stickies-loader.tsx`
- [ ] `core/components/stickies/layout/stickies-truncated.tsx`
- [ ] `core/components/stickies/layout/sticky-dnd-wrapper.tsx`
- [ ] `core/components/stickies/layout/sticky.helpers.ts`

#### `core/components/stickies/modal/`

- [ ] `core/components/stickies/modal/index.tsx`
- [ ] `core/components/stickies/modal/search.tsx`
- [ ] `core/components/stickies/modal/stickies.tsx`

#### `core/components/stickies/sticky/`

- [ ] `core/components/stickies/sticky/index.ts`
- [ ] `core/components/stickies/sticky/inputs.tsx`
- [ ] `core/components/stickies/sticky/root.tsx`
- [ ] `core/components/stickies/sticky/sticky-item-drag-handle.tsx`
- [ ] `core/components/stickies/sticky/use-operations.tsx`

#### `core/components/ui/`

- [ ] `core/components/ui/empty-space.tsx`
- [ ] `core/components/ui/integration-and-import-export-banner.tsx`
- [ ] `core/components/ui/labels-list.tsx`
- [ ] `core/components/ui/markdown-to-component.tsx`
- [ ] `core/components/ui/profile-empty-state.tsx`

#### `core/components/ui/loader/`

- [ ] `core/components/ui/loader/cycle-module-board-loader.tsx`
- [ ] `core/components/ui/loader/cycle-module-list-loader.tsx`
- [ ] `core/components/ui/loader/notification-loader.tsx`
- [ ] `core/components/ui/loader/pages-loader.tsx`
- [ ] `core/components/ui/loader/projects-loader.tsx`
- [ ] `core/components/ui/loader/utils.tsx`
- [ ] `core/components/ui/loader/view-list-loader.tsx`

#### `core/components/ui/loader/layouts/`

- [ ] `core/components/ui/loader/layouts/calendar-layout-loader.tsx`
- [ ] `core/components/ui/loader/layouts/gantt-layout-loader.tsx`
- [ ] `core/components/ui/loader/layouts/kanban-layout-loader.tsx`
- [ ] `core/components/ui/loader/layouts/list-layout-loader.tsx`
- [ ] `core/components/ui/loader/layouts/members-layout-loader.tsx`
- [ ] `core/components/ui/loader/layouts/spreadsheet-layout-loader.tsx`

#### `core/components/ui/loader/layouts/project-inbox/`

- [ ] `core/components/ui/loader/layouts/project-inbox/inbox-layout-loader.tsx`
- [ ] `core/components/ui/loader/layouts/project-inbox/inbox-sidebar-loader.tsx`

#### `core/components/ui/loader/settings/`

- [ ] `core/components/ui/loader/settings/activity.tsx`
- [ ] `core/components/ui/loader/settings/api-token.tsx`
- [ ] `core/components/ui/loader/settings/email.tsx`
- [ ] `core/components/ui/loader/settings/import-and-export.tsx`
- [ ] `core/components/ui/loader/settings/integration.tsx`
- [ ] `core/components/ui/loader/settings/members.tsx`
- [ ] `core/components/ui/loader/settings/web-hook.tsx`

#### `core/components/user/`

- [ ] `core/components/user/index.ts`
- [ ] `core/components/user/user-greetings.tsx`

#### `core/components/views/`

- [ ] `core/components/views/delete-view-modal.tsx`
- [ ] `core/components/views/form.tsx`
- [ ] `core/components/views/helper.tsx`
- [ ] `core/components/views/modal.tsx`
- [ ] `core/components/views/quick-actions.tsx`
- [ ] `core/components/views/view-list-header.tsx`
- [ ] `core/components/views/view-list-item-action.tsx`
- [ ] `core/components/views/view-list-item.tsx`
- [ ] `core/components/views/views-list.tsx`

#### `core/components/views/applied-filters/`

- [ ] `core/components/views/applied-filters/access.tsx`
- [ ] `core/components/views/applied-filters/index.tsx`
- [ ] `core/components/views/applied-filters/root.tsx`

#### `core/components/views/filters/`

- [ ] `core/components/views/filters/filter-selection.tsx`
- [ ] `core/components/views/filters/order-by.tsx`

#### `core/components/web-hooks/`

- [ ] `core/components/web-hooks/create-webhook-modal.tsx`
- [ ] `core/components/web-hooks/delete-webhook-modal.tsx`
- [ ] `core/components/web-hooks/empty-state.tsx`
- [ ] `core/components/web-hooks/generated-hook-details.tsx`
- [ ] `core/components/web-hooks/index.ts`
- [ ] `core/components/web-hooks/utils.ts`
- [ ] `core/components/web-hooks/webhooks-list-item.tsx`
- [ ] `core/components/web-hooks/webhooks-list.tsx`

#### `core/components/web-hooks/form/`

- [ ] `core/components/web-hooks/form/delete-section.tsx`
- [ ] `core/components/web-hooks/form/event-types.tsx`
- [ ] `core/components/web-hooks/form/form.tsx`
- [ ] `core/components/web-hooks/form/index.ts`
- [ ] `core/components/web-hooks/form/individual-event-options.tsx`
- [ ] `core/components/web-hooks/form/input.tsx`
- [ ] `core/components/web-hooks/form/secret-key.tsx`
- [ ] `core/components/web-hooks/form/toggle.tsx`

#### `core/components/work-item-filters/`

- [ ] `core/components/work-item-filters/filters-row.tsx`
- [ ] `core/components/work-item-filters/filters-toggle.tsx`

#### `core/components/work-item-filters/filters-hoc/`

- [ ] `core/components/work-item-filters/filters-hoc/base.tsx`
- [ ] `core/components/work-item-filters/filters-hoc/project-level.tsx`
- [ ] `core/components/work-item-filters/filters-hoc/shared.ts`
- [ ] `core/components/work-item-filters/filters-hoc/workspace-level.tsx`

#### `core/components/workspace/`

- [ ] `core/components/workspace/ConfirmWorkspaceMemberRemove.tsx`
- [ ] `core/components/workspace/confirm-workspace-member-remove.tsx`
- [ ] `core/components/workspace/create-workspace-form.tsx`
- [ ] `core/components/workspace/delete-workspace-form.tsx`
- [ ] `core/components/workspace/logo.tsx`

#### `core/components/workspace/billing/comparison/`

- [ ] `core/components/workspace/billing/comparison/base.tsx`
- [ ] `core/components/workspace/billing/comparison/feature-detail.tsx`
- [ ] `core/components/workspace/billing/comparison/index.ts`

#### `core/components/workspace/invite-modal/`

- [ ] `core/components/workspace/invite-modal/actions.tsx`
- [ ] `core/components/workspace/invite-modal/fields.tsx`
- [ ] `core/components/workspace/invite-modal/form.tsx`

#### `core/components/workspace/settings/`

- [ ] `core/components/workspace/settings/invitations-list-item.tsx`
- [ ] `core/components/workspace/settings/member-columns.tsx`
- [ ] `core/components/workspace/settings/members-list-item.tsx`
- [ ] `core/components/workspace/settings/members-list.tsx`
- [ ] `core/components/workspace/settings/workspace-details.tsx`

#### `core/components/workspace/sidebar/`

- [ ] `core/components/workspace/sidebar/dropdown-item.tsx`
- [ ] `core/components/workspace/sidebar/project-navigation.tsx`
- [ ] `core/components/workspace/sidebar/projects-list-item.tsx`
- [ ] `core/components/workspace/sidebar/projects-list.tsx`
- [ ] `core/components/workspace/sidebar/quick-actions.tsx`
- [ ] `core/components/workspace/sidebar/sidebar-item.tsx`
- [ ] `core/components/workspace/sidebar/sidebar-menu-items.tsx`
- [ ] `core/components/workspace/sidebar/user-menu-item.tsx`
- [ ] `core/components/workspace/sidebar/user-menu-root.tsx`
- [ ] `core/components/workspace/sidebar/user-menu.tsx`
- [ ] `core/components/workspace/sidebar/workspace-menu-header.tsx`
- [ ] `core/components/workspace/sidebar/workspace-menu-item.tsx`
- [ ] `core/components/workspace/sidebar/workspace-menu-root.tsx`
- [ ] `core/components/workspace/sidebar/workspace-menu.tsx`

#### `core/components/workspace/sidebar/favorites/`

- [ ] `core/components/workspace/sidebar/favorites/favorite-folder.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorites-menu.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorites.helpers.ts`
- [ ] `core/components/workspace/sidebar/favorites/new-fav-folder.tsx`

#### `core/components/workspace/sidebar/favorites/favorite-items/`

- [ ] `core/components/workspace/sidebar/favorites/favorite-items/index.ts`
- [ ] `core/components/workspace/sidebar/favorites/favorite-items/root.tsx`

#### `core/components/workspace/sidebar/favorites/favorite-items/common/`

- [ ] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-drag-handle.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-quick-action.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-title.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorite-items/common/favorite-item-wrapper.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorite-items/common/helper.tsx`
- [ ] `core/components/workspace/sidebar/favorites/favorite-items/common/index.ts`

#### `core/components/workspace/sidebar/help-section/`

- [ ] `core/components/workspace/sidebar/help-section/index.ts`
- [ ] `core/components/workspace/sidebar/help-section/root.tsx`

#### `core/components/workspace/views/`

- [ ] `core/components/workspace/views/default-view-list-item.tsx`
- [ ] `core/components/workspace/views/default-view-quick-action.tsx`
- [ ] `core/components/workspace/views/delete-view-modal.tsx`
- [ ] `core/components/workspace/views/form.tsx`
- [ ] `core/components/workspace/views/header.tsx`
- [ ] `core/components/workspace/views/modal.tsx`
- [ ] `core/components/workspace/views/quick-action.tsx`
- [ ] `core/components/workspace/views/view-list-item.tsx`
- [ ] `core/components/workspace/views/views-list.tsx`

#### `core/components/workspace-notifications/`

- [ ] `core/components/workspace-notifications/index.ts`
- [ ] `core/components/workspace-notifications/notification-app-sidebar-option.tsx`
- [ ] `core/components/workspace-notifications/root.tsx`

#### `core/components/workspace-notifications/sidebar/`

- [ ] `core/components/workspace-notifications/sidebar/empty-state.tsx`
- [ ] `core/components/workspace-notifications/sidebar/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/loader.tsx`
- [ ] `core/components/workspace-notifications/sidebar/root.tsx`

#### `core/components/workspace-notifications/sidebar/filters/`

- [ ] `core/components/workspace-notifications/sidebar/filters/applied-filter.tsx`

#### `core/components/workspace-notifications/sidebar/filters/menu/`

- [ ] `core/components/workspace-notifications/sidebar/filters/menu/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/filters/menu/menu-option-item.tsx`
- [ ] `core/components/workspace-notifications/sidebar/filters/menu/root.tsx`

#### `core/components/workspace-notifications/sidebar/header/`

- [ ] `core/components/workspace-notifications/sidebar/header/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/header/root.tsx`

#### `core/components/workspace-notifications/sidebar/header/options/`

- [ ] `core/components/workspace-notifications/sidebar/header/options/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/header/options/root.tsx`

#### `core/components/workspace-notifications/sidebar/header/options/menu-option/`

- [ ] `core/components/workspace-notifications/sidebar/header/options/menu-option/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/header/options/menu-option/menu-item.tsx`
- [ ] `core/components/workspace-notifications/sidebar/header/options/menu-option/root.tsx`

#### `core/components/workspace-notifications/sidebar/notification-card/`

- [ ] `core/components/workspace-notifications/sidebar/notification-card/content.tsx`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/item.tsx`

#### `core/components/workspace-notifications/sidebar/notification-card/options/`

- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/archive.tsx`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/button.tsx`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/read.tsx`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/root.tsx`

#### `core/components/workspace-notifications/sidebar/notification-card/options/snooze/`

- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/snooze/index.ts`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/snooze/modal.tsx`
- [ ] `core/components/workspace-notifications/sidebar/notification-card/options/snooze/root.tsx`

#### `core/constants/`

- [ ] `core/constants/ai.ts`
- [ ] `core/constants/calendar.ts`
- [ ] `core/constants/editor.ts`
- [ ] `core/constants/fetch-keys.ts`
- [ ] `core/constants/gantt-chart.ts`
- [ ] `core/constants/plans.tsx`
- [ ] `core/constants/sidebar-favorites.ts`

#### `core/custom-events/`

- [ ] `core/custom-events/chat-support.ts`

#### `core/hooks/`

- [ ] `core/hooks/use-app-router.tsx`
- [ ] `core/hooks/use-auto-save.tsx`
- [ ] `core/hooks/use-auto-scroller.tsx`
- [ ] `core/hooks/use-chat-support.ts`
- [ ] `core/hooks/use-collaborative-page-actions.tsx`
- [ ] `core/hooks/use-current-time.tsx`
- [ ] `core/hooks/use-debounce.tsx`
- [ ] `core/hooks/use-dropdown-key-down.tsx`
- [ ] `core/hooks/use-dropdown.ts`
- [ ] `core/hooks/use-expandable-search.ts`
- [ ] `core/hooks/use-extended-sidebar-overview-outside-click.tsx`
- [ ] `core/hooks/use-favorite-item-details.tsx`
- [ ] `core/hooks/use-group-dragndrop.ts`
- [x] `core/hooks/use-integration-popup.tsx`
- [ ] `core/hooks/use-intersection-observer.ts`
- [ ] `core/hooks/use-issue-layout-store.ts`
- [ ] `core/hooks/use-issue-peek-overview-redirection.tsx`
- [ ] `core/hooks/use-issues-actions.tsx`
- [ ] `core/hooks/use-keypress.tsx`
- [ ] `core/hooks/use-local-storage.tsx`
- [ ] `core/hooks/use-multiple-select.ts`
- [ ] `core/hooks/use-navigation-preferences.ts`
- [ ] `core/hooks/use-online-status.ts`
- [ ] `core/hooks/use-page-fallback.ts`
- [ ] `core/hooks/use-page-filters.ts`
- [ ] `core/hooks/use-page-operations.ts`
- [ ] `core/hooks/use-parse-editor-content.ts`
- [ ] `core/hooks/use-peek-overview-outside-click.tsx`
- [ ] `core/hooks/use-platform-os.tsx`
- [ ] `core/hooks/use-project-issue-properties.ts`
- [ ] `core/hooks/use-query-params.ts`
- [ ] `core/hooks/use-realtime-page-events.tsx`
- [ ] `core/hooks/use-reload-confirmation.tsx`
- [ ] `core/hooks/use-stickies.tsx`
- [ ] `core/hooks/use-table-keyboard-navigation.tsx`
- [ ] `core/hooks/use-timeline-chart.ts`
- [ ] `core/hooks/use-timer.tsx`
- [ ] `core/hooks/use-timezone-converter.tsx`
- [ ] `core/hooks/use-timezone.tsx`
- [ ] `core/hooks/use-window-size.tsx`
- [ ] `core/hooks/use-workspace-invitation.tsx`
- [ ] `core/hooks/use-workspace-issue-properties.ts`
- [ ] `core/hooks/use-workspace-paths.ts`

#### `core/hooks/context/`

- [ ] `core/hooks/context/use-issue-modal.tsx`

#### `core/hooks/editor/`

- [ ] `core/hooks/editor/index.ts`
- [ ] `core/hooks/editor/use-editor-config.ts`
- [ ] `core/hooks/editor/use-editor-mention.tsx`

#### `core/hooks/oauth/`

- [ ] `core/hooks/oauth/core.tsx`
- [ ] `core/hooks/oauth/extended.tsx`
- [ ] `core/hooks/oauth/index.ts`

#### `core/hooks/store/`

- [ ] `core/hooks/store/use-analytics.ts`
- [ ] `core/hooks/store/use-app-theme.ts`
- [ ] `core/hooks/store/use-calendar-view.ts`
- [ ] `core/hooks/store/use-command-palette.ts`
- [ ] `core/hooks/store/use-cycle-filter.ts`
- [ ] `core/hooks/store/use-cycle.ts`
- [ ] `core/hooks/store/use-dashboard.ts`
- [ ] `core/hooks/store/use-editor-asset.ts`
- [ ] `core/hooks/store/use-favorite.ts`
- [ ] `core/hooks/store/use-global-view.ts`
- [ ] `core/hooks/store/use-home.ts`
- [ ] `core/hooks/store/use-inbox-issues.ts`
- [ ] `core/hooks/store/use-instance.ts`
- [ ] `core/hooks/store/use-issue-detail.ts`
- [ ] `core/hooks/store/use-issues.ts`
- [ ] `core/hooks/store/use-kanban-view.ts`
- [ ] `core/hooks/store/use-label.ts`
- [ ] `core/hooks/store/use-member.ts`
- [ ] `core/hooks/store/use-module-filter.ts`
- [ ] `core/hooks/store/use-module.ts`
- [ ] `core/hooks/store/use-multiple-select-store.ts`
- [ ] `core/hooks/store/use-power-k.ts`
- [ ] `core/hooks/store/use-project-filter.ts`
- [ ] `core/hooks/store/use-project-inbox.ts`
- [ ] `core/hooks/store/use-project-publish.ts`
- [ ] `core/hooks/store/use-project-state.ts`
- [ ] `core/hooks/store/use-project-view.ts`
- [ ] `core/hooks/store/use-project.ts`
- [ ] `core/hooks/store/use-router-params.ts`
- [ ] `core/hooks/store/use-webhook.ts`
- [ ] `core/hooks/store/use-workspace.ts`

#### `core/hooks/store/estimates/`

- [ ] `core/hooks/store/estimates/index.ts`
- [ ] `core/hooks/store/estimates/use-estimate-point.ts`
- [ ] `core/hooks/store/estimates/use-estimate.ts`
- [ ] `core/hooks/store/estimates/use-project-estimate.ts`

#### `core/hooks/store/notifications/`

- [ ] `core/hooks/store/notifications/index.ts`
- [ ] `core/hooks/store/notifications/use-notification.ts`
- [ ] `core/hooks/store/notifications/use-workspace-notifications.ts`

#### `core/hooks/store/user/`

- [ ] `core/hooks/store/user/index.ts`
- [ ] `core/hooks/store/user/user-permissions.ts`
- [ ] `core/hooks/store/user/user-user-profile.ts`
- [ ] `core/hooks/store/user/user-user-settings.ts`
- [ ] `core/hooks/store/user/user-user.ts`

#### `core/hooks/store/work-item-filters/`

- [ ] `core/hooks/store/work-item-filters/use-work-item-filter-instance.ts`
- [ ] `core/hooks/store/work-item-filters/use-work-item-filters.ts`

#### `core/hooks/store/workspace-draft/`

- [ ] `core/hooks/store/workspace-draft/index.ts`
- [ ] `core/hooks/store/workspace-draft/use-workspace-draft-issue-filters.ts`
- [ ] `core/hooks/store/workspace-draft/use-workspace-draft-issue.ts`

#### `core/layouts/auth-layout/`

- [ ] `core/layouts/auth-layout/project-wrapper.tsx`
- [ ] `core/layouts/auth-layout/workspace-wrapper.tsx`

#### `core/layouts/default-layout/`

- [ ] `core/layouts/default-layout/index.tsx`

#### `core/lib/`

- [ ] `core/lib/local-storage.ts`
- [ ] `core/lib/store-context.tsx`

#### `core/lib/app-rail/`

- [ ] `core/lib/app-rail/context.tsx`
- [ ] `core/lib/app-rail/index.ts`
- [ ] `core/lib/app-rail/provider.tsx`
- [ ] `core/lib/app-rail/types.ts`

#### `core/lib/b-progress/`

- [ ] `core/lib/b-progress/AppProgressBar.tsx`

#### `core/lib/polyfills/`

- [ ] `core/lib/polyfills/index.ts`

#### `core/lib/wrappers/`

- [ ] `core/lib/wrappers/authentication-wrapper.tsx`
- [ ] `core/lib/wrappers/instance-wrapper.tsx`
- [ ] `core/lib/wrappers/store-wrapper.tsx`

#### `core/services/`

- [!] `core/services/ai.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/analytics.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [x] `core/services/api.service.ts`
- [!] `core/services/app_config.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [x] `core/services/app_installation.service.ts`
- [!] `core/services/auth.service.ts` — Legacy error rethrow (5×) + `signOut` manipula DOM (`document.createElement`, `document.body.appendChild`) sin guard SSR y sin remover el form tras `form.submit()` (aceptable porque hay reload inmediato).
- [!] `core/services/cycle.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/cycle_archive.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/dashboard.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/estimate.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/file-upload.service.ts` — Legacy error rethrow + `console.log` dejado (debería usar `console.error` o eliminarse).
- [!] `core/services/file.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/instance.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue_filter.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/module.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/module_archive.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/sticky.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/timezone.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/user.service.ts` — Legacy error rethrow (26× catches) + múltiples `Promise<any>`.
- [!] `core/services/view.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/webhook.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/workspace-notification.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/workspace.service.ts` — Legacy error rethrow (42× catches) + múltiples `Promise<any>`.

#### `core/services/favorite/`

- [!] `core/services/favorite/favorite.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [x] `core/services/favorite/index.ts`

#### `core/services/inbox/`

- [!] `core/services/inbox/inbox-issue.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [x] `core/services/inbox/index.ts`
- [!] `core/services/inbox/intake-work_item_version.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.

#### `core/services/integrations/`

- [x] `core/services/integrations/github-user-connection.service.ts`
- [x] `core/services/integrations/github.service.ts`
- [x] `core/services/integrations/gitlab.service.ts`
- [x] `core/services/integrations/index.ts`
- [x] `core/services/integrations/integration.service.ts`
- [x] `core/services/integrations/jira.service.ts`

#### `core/services/issue/`

- [x] `core/services/issue/index.ts`
- [!] `core/services/issue/issue.service.ts` — Legacy error rethrow (32× catches) + múltiples `Promise<any>`.
- [!] `core/services/issue/issue_activity.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/issue_archive.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/issue_attachment.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/issue_comment.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/issue_label.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/issue_reaction.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/issue_relation.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/work_item_version.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/issue/workspace_draft.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.

#### `core/services/page/`

- [x] `core/services/page/index.ts`
- [!] `core/services/page/project-page-version.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/page/project-page.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.

#### `core/services/project/`

- [x] `core/services/project/index.ts`
- [!] `core/services/project/project-archive.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/project/project-export.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/project/project-member.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/project/project-publish.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/project/project-state.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.
- [!] `core/services/project/project.service.ts` — Legacy error pattern: raw axios error rethrow en `.catch` (bypassa contrato `ApiError` de `api.service.ts`). Refactor pendiente a `throw new ApiError(error?.response)` para consistencia.

#### `core/store/`

- [!] `core/store/analytics.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/base-command-palette.store.ts`
- [x] `core/store/base-power-k.store.ts`
- [!] `core/store/cycle.store.ts` — Raw error rethrow + `console.log` para errores (línea 607) + `eslint-disable` en línea 503 para `no-unused-vars`. Revisar si el parámetro es realmente innecesario.
- [x] `core/store/cycle_filter.store.ts`
- [!] `core/store/dashboard.store.ts` — Raw error rethrow + `as unknown as T` en línea 140 (type laundering en widgetStats) + `: any` inferido en callback `.then((res: any) =>` línea 186.
- [!] `core/store/favorite.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/global-view.store.ts` — Raw error rethrow + `: any` en anotaciones.
- [!] `core/store/instance.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/label.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [!] `core/store/module.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/module_filter.store.ts`
- [x] `core/store/multiple_select.store.ts`
- [!] `core/store/project-view.store.ts` — `: any` en anotaciones.
- [!] `core/store/root.store.ts` — 12× `as unknown as RootStore` (type laundering por arquitectura modular CE/EE) + 2× `localStorage.setItem` en `resetOnSignOut` sin guard SSR (aceptable por contexto de uso client-only)
- [x] `core/store/router.store.ts`
- [!] `core/store/state.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/theme.store.ts` — `localStorage.setItem` en 6 toggle methods sin guard SSR. `theme.store` se instancia en `CoreRootStore` constructor que SÍ corre en SSR; aunque los toggles se llaman por click, un path de hidratación podría invocarlos. Envolver con `if (typeof window !== 'undefined')`.

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
- [!] `core/store/issue/issue_gantt_view.store.ts` — `: any` (4×) — ya flagged en scan general inicial.
- [x] `core/store/issue/issue_kanban_view.store.ts`
- [x] `core/store/issue/root.store.ts`

#### `core/store/issue/archived/`

- [!] `core/store/issue/archived/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/archived/index.ts`
- [!] `core/store/issue/archived/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/cycle/`

- [!] `core/store/issue/cycle/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/cycle/index.ts`
- [!] `core/store/issue/cycle/issue.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).

#### `core/store/issue/helpers/`

- [!] `core/store/issue/helpers/base-issues-utils.ts` — `: any` + `console.log` (detectado en scan previo). 2× non-null assertions (`!.`).
- [!] `core/store/issue/helpers/base-issues.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/issue/helpers/issue-filter-helper.store.ts`

#### `core/store/issue/issue-details/`

- [!] `core/store/issue/issue-details/attachment.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/issue/issue-details/comment.store.ts` — Raw error rethrow + `: any`.
- [!] `core/store/issue/issue-details/comment_reaction.store.ts` — `console.log("error", error)` en líneas 126 y 166 — debería ser `console.error`. Raw rethrows. `: any`.
- [x] `core/store/issue/issue-details/issue.store.ts`
- [!] `core/store/issue/issue-details/link.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/issue/issue-details/reaction.store.ts` — `: any` en anotaciones.
- [!] `core/store/issue/issue-details/relation.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/issue/issue-details/root.store.ts`
- [x] `core/store/issue/issue-details/sub_issues.store.ts`
- [x] `core/store/issue/issue-details/sub_issues_filter.store.ts`
- [!] `core/store/issue/issue-details/subscription.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/module/`

- [!] `core/store/issue/module/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/module/index.ts`
- [!] `core/store/issue/module/issue.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).

#### `core/store/issue/profile/`

- [!] `core/store/issue/profile/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/profile/index.ts`
- [!] `core/store/issue/profile/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/project/`

- [!] `core/store/issue/project/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/project/index.ts`
- [!] `core/store/issue/project/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/project-views/`

- [!] `core/store/issue/project-views/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/project-views/index.ts`
- [!] `core/store/issue/project-views/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/workspace/`

- [!] `core/store/issue/workspace/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [x] `core/store/issue/workspace/index.ts`
- [!] `core/store/issue/workspace/issue.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/issue/workspace-draft/`

- [!] `core/store/issue/workspace-draft/filter.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
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
- [!] `core/store/project/project.store.ts` — 11× `console.log("Failed to ...", error)` deberían ser `console.error`. Raw error rethrows. `: any` en algún retorno.
- [x] `core/store/project/project_filter.store.ts`

#### `core/store/sticky/`

- [!] `core/store/sticky/sticky.store.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).

#### `core/store/timeline/`

- [x] `core/store/timeline/issues-timeline.store.ts`
- [x] `core/store/timeline/modules-timeline.store.ts`

#### `core/store/user/`

- [!] `core/store/user/account.store.ts` — `: any` en anotaciones.
- [!] `core/store/user/base-permissions.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/user/index.ts` — Legacy raw throw + `console.log` para errores (debería ser `console.error` o eliminarse).
- [!] `core/store/user/profile.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/user/settings.store.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.

#### `core/store/workspace/`

- [x] `core/store/workspace/api-token.store.ts`
- [!] `core/store/workspace/home.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [!] `core/store/workspace/index.ts` — Legacy error pattern: raw axios rethrow en `.catch` (no usa `ApiError`). Misma deuda técnica que los services.
- [x] `core/store/workspace/link.store.ts`
- [x] `core/store/workspace/webhook.store.ts`

#### `core/types/`

- [ ] `core/types/navigation-preferences.ts`

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

- [ ] `react-router.config.ts`
- [ ] `vite.config.ts`

