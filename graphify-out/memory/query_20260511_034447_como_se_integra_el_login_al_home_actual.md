---
type: "query"
date: "2026-05-11T03:44:47.043351+00:00"
question: "como se integra el login al home actual?"
contributor: "graphify"
source_nodes: ["HomePage()", "AuthenticationWrapper", "AuthBase()", "EPageTypes"]
---

# Q: como se integra el login al home actual?

## Answer

El home (web/app/(home)/page.tsx) usa pageType=NON_AUTHENTICATED. AuthenticationWrapper lo intercepta: si currentUser existe y está onboarded, redirige al workspace; si no está onboarded, redirige a /onboarding. Si no hay usuario, renderiza AuthBase con SIGN_IN. El flujo es: AuthenticationWrapper fetch currentUser via SWR -> evalua EPageTypes.NON_AUTHENTICATED -> redirige o muestra login form.

## Source Nodes

- HomePage()
- AuthenticationWrapper
- AuthBase()
- EPageTypes