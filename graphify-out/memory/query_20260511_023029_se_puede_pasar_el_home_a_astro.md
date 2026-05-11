---
type: "query"
date: "2026-05-11T02:30:29.627211+00:00"
question: "se puede pasar el home a astro?"
contributor: "graphify"
source_nodes: ["WorkspaceHomeView", "HomeStore", "home.ts"]
---

# Q: se puede pasar el home a astro?

## Answer

apps/home es HTML estático puro (1161 líneas, servido por nginx). Astro es ideal para esto: SSG, zero-JS por defecto, pnpm monorepo compatible. Migración directa: crear apps/home como Astro project, extraer secciones en .astro components, mover assets, output estático reemplaza el index.html actual.

## Source Nodes

- WorkspaceHomeView
- HomeStore
- home.ts
