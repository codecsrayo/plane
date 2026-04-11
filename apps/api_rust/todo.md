---
titulo: Plane API Rust — Índice
tags:
  - plane
  - rust
  - index
estado: activo
---

# Plane — API Rust

> [!SUMMARY] Objetivo
> Reemplazar Django + Celery (\~750 MB) por Rust (\~20 MB).
> SeaORM toma ownership del schema. `plane-live` (Node.js) permanece intacto.

## 📂 Vault de documentación

La documentación está organizada como un vault Obsidian en `docs/`.
Abre la carpeta `apps/api_rust/docs/` como vault en Obsidian.

→ **Punto de entrada:** [[docs/00-README]]

## Notas disponibles

| Nota | Tema |
|------|------|
| [[docs/00-README]] | Mapa de contenido (MOC) |
| [[docs/01-objetivo]] | Objetivo, plane-live, decisiones de diseño |
| [[docs/02-orm-y-migraciones]] | SeaORM, sea-orm-migration, baseline desde Django |
| [[docs/03-stack]] | Tabla de librerías Rust vs Django |
| [[docs/04-arquitectura]] | Diagramas de flujo actual vs futuro |
| [[docs/05-testing]] | Swagger UI, axum-test, Bruno |
| [[docs/06-soft-delete]] | Trait SoftDeleteExt + macro |
| [[docs/07-fases]] | Estrategia de migración — Fases 0–5 |
| [[docs/08-estado-migraciones]] | Seguimiento de migraciones m001–m006 |
| [[docs/09-workspace-seed]] | WorkspaceSeedJob apalis + datos JSON |
| [[docs/10-patrones]] | Repository, AppState, Extractors, AppError, Jobs |
| [[docs/11-diagramas-secuencia]] | Sequence diagrams end-to-end |
| [[docs/12-cosas-criticas]] | 13 puntos de riesgo operacional |
| [[docs/13-estructura-archivos]] | Árbol de directorios del proyecto Rust |
| [[docs/14-integraciones]] | GitHub App, GitLab OAuth, Slack OAuth |
| [[docs/15-workspace-settings]] | Flujo completo del panel de configuración |

## Estado rápido

| Ítem | Estado |
|------|--------|
| Soft delete (`SoftDeleteExt`) | ✅ |
| Migraciones m001–m005 | ✅ |
| Migración m006 (integraciones) | 🔄 pendiente prueba |
| Fase 0 — scaffolding | 🔄 en progreso |
| Fase 1 — Auth middleware | ⬜ |
| Fase 2 — Endpoints alta frecuencia | ⬜ |
| Fase 3 — Background jobs (apalis) | ⬜ |
| Fase 4 — Endpoints restantes | ⬜ |
| Fase 5 — Shutdown Django | ⬜ |
