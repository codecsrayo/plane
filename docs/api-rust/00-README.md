---
titulo: Plane API Rust — Mapa de contenido
tags:
  - plane
  - rust
  - moc
estado: activo---

# Plane — API Rust

> [!SUMMARY] Objetivo
> Reemplazar Django + Celery (~750 MB) por un stack Rust completo (~20 MB).
> Rust toma ownership total del schema PostgreSQL. `plane-live` (Node.js) no se toca.

## Navegación rápida

| # | Nota | Descripción |
|---|------|-------------|
| 01 | [[01-objetivo]] | Metas, plane-live, decisiones de diseño |
| 02 | [[02-orm-y-migraciones]] | SeaORM, sea-orm-migration, baseline desde Django |
| 03 | [[03-stack]] | Tabla de librerías Rust vs equivalente Django |
| 04 | [[04-arquitectura]] | Diagramas de flujo actual vs futuro |
| 05 | [[05-testing]] | Swagger UI, axum-test, Bruno |
| 06 | [[06-soft-delete]] | Trait SoftDeleteExt + macro |
| 07 | [[07-fases]] | Estrategia de migración — Fases 0–5 |
| 08 | [[08-estado-migraciones]] | Seguimiento de migraciones m001–m006 |
| 09 | [[09-workspace-seed]] | WorkspaceSeedJob apalis + datos JSON |
| 10 | [[10-patrones]] | Repository, AppState, Extractors, AppError, Jobs, Cron |
| 11 | [[11-diagramas-secuencia]] | Sequence diagrams end-to-end |
| 12 | [[12-cosas-criticas]] | 13 puntos de riesgo operacional |
| 13 | [[13-estructura-archivos]] | Árbol de directorios del proyecto Rust |
| 14 | [[14-integraciones]] | GitHub App, GitLab OAuth, Slack OAuth |
| 15 | [[15-workspace-settings]] | Flujo completo del panel de configuración |
| 16 | [[16-diagramas-flujo]] | Diagramas de flujo adicionales |
| 17 | [[17-estructura-django]] | Estructura de carpetas Django (fuente de migración) |
| 18 | [[18-implementacion-api-inicial]] | Bootstrap API inicial — main.rs, config, error, Swagger UI |

## Estado de implementación — resumen

| Sección | Estado |
|---------|--------|
| Soft delete | ✅ Implementado |
| Migraciones baseline (m001–m005) | ✅ Aplicadas |
| Migración m006 (integraciones) | 🔄 Pendiente prueba |
| WorkspaceSeedJob (apalis) | 📝 Diseñado — no implementado |
| Endpoints Fase 2 | 📝 Planificado |
| Background jobs Fase 3 | 📝 Planificado |

## Tags del vault

`#rust` `#seaorm` `#axum` `#apalis` `#plane` `#django-migration`

