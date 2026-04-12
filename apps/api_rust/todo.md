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

## Documentación

La documentación vive en el vault Obsidian en `docs/` (raíz del repo).
Abre la carpeta `docs/` como vault en Obsidian.

Notas relevantes para este módulo:

- `api-rust/00-README` — Mapa de contenido
- `api-rust/07-fases` — Fases de migración con checklist
- `api-rust/08-estado-migraciones` — Estado de migraciones m001–m006
- `api-rust/14-integraciones` — GitHub / GitLab / Slack
- `api-rust/15-workspace-settings` — Panel de configuración del workspace

## Estado rápido

| Ítem | Estado |
|------|--------|
| Soft delete (`SoftDeleteExt`) | ✅ |
| Migraciones m001–m006 (Baseline SQL) | ✅ |
| Migración m007 (Seed data) | ✅ |
| Fase 0 — scaffolding | 🔄 en progreso |
| Fase 1 — Auth middleware | ⬜ |
| Fase 2 — Endpoints alta frecuencia | ⬜ |
| Fase 3 — Background jobs (apalis) | ⬜ |
| Fase 4 — Endpoints restantes | ⬜ |
| Fase 5 — Shutdown Django | ⬜ |
