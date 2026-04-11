---
titulo: Plane API Rust — Mapa de Contenido
aliases:
  - inicio
  - home
  - index
tags:
  - plane
  - rust
  - moc
cssClass: moc
estado: activo
---

# 🦀 Plane API Rust

> [!SUMMARY] Objetivo central
> Reemplazar Django + Celery (~750 MB RAM) por un binario Rust único (~20 MB).
> Rust toma **ownership total** del schema PostgreSQL. `plane-live` (Node.js) no se toca.

---

## 🗺️ Clusters de conocimiento

### 🎯 Visión
*¿Qué, por qué y cómo luce el sistema completo?*

| Nota | Contenido |
|------|-----------|
| [[vision-objetivo]] | Metas, decisiones de diseño, qué queda intacto |
| [[vision-stack]] | Tabla de librerías Rust vs equivalente Django |
| [[vision-arquitectura]] | Diagramas actual vs futuro, lo que desaparece |

---

### 🧱 Fundamentos
*Base técnica: schema, ORM, persistencia.*

| Nota | Contenido |
|------|-----------|
| [[fundamentos-orm]] | SeaORM, sea-orm-migration, baseline desde Django |
| [[fundamentos-migraciones-estado]] | Estado de m001–m006, fixes aplicados, comandos |
| [[fundamentos-soft-delete]] | Trait SoftDeleteExt + macro `impl_soft_delete!` |

---

### ⚙️ Implementación
*Código concreto: cómo se construye la API.*

| Nota | Contenido |
|------|-----------|
| [[impl-bootstrap]] | `main.rs`, `config.rs`, `error.rs`, Swagger UI |
| [[impl-autenticacion]] | Session Cookie + API Key — extractores completos |
| [[impl-appstate-repository]] | AppState, Repository Pattern — aislar SeaORM |
| [[impl-extractores-auth]] | CurrentUser, WorkspaceMemberGuard, ProjectMemberGuard, RBAC |
| [[impl-error-jobs-cron]] | AppError, Job Pattern (apalis), Cron (tokio-cron-scheduler) |

---

### 🏗️ Dominio
*Lógica de negocio de las features principales.*

| Nota | Contenido |
|------|-----------|
| [[dominio-workspace-seed]] | WorkspaceSeedJob apalis — datos iniciales JSON |
| [[dominio-integraciones]] | GitHub App, GitLab OAuth, Slack OAuth — flujos y endpoints |
| [[dominio-workspace-settings]] | Panel de configuración: General, Members, Exports, Webhooks |

---

### 📋 Planificación
*Estrategia de migración y puntos de riesgo.*

| Nota | Contenido |
|------|-----------|
| [[plan-fases]] | Estrategia en 6 fases (Fase 0 → Fase 5 shutdown Django) |
| [[plan-riesgos]] | 13 puntos críticos de riesgo operacional |

---

### 📚 Referencia
*Documentación de apoyo: estructura, diagramas, testing.*

| Nota | Contenido |
|------|-----------|
| [[ref-estructura-archivos]] | Árbol de directorios del proyecto Rust (estado objetivo) |
| [[ref-estructura-django]] | Árbol completo de `apps/api` — fuente de migración |
| [[ref-diagramas-secuencia]] | Sequence diagrams: workspace seed, auth, requests |
| [[ref-diagramas-flujo]] | Flowcharts: workspace, projects, issues, roles |
| [[ref-testing]] | Swagger UI, axum-test, Bruno — estrategia completa |

---

## 📊 Estado de implementación

| Área | Estado | Notas |
|------|--------|-------|
| Soft delete | ✅ Implementado | `src/utils/soft_delete.rs` |
| Migraciones m001–m005 | ✅ Aplicadas | Baseline completo |
| Migración m006 (integraciones) | 🔄 Pendiente prueba | |
| Bootstrap API (`main.rs`) | 📝 Diseñado | Ver [[impl-bootstrap]] |
| Auth Session Cookie | 📝 Diseñado | Ver [[impl-autenticacion]] |
| Auth API Key | 📝 Diseñado | Ver [[impl-autenticacion]] |
| Extractores WorkspaceMemberGuard | 📝 Diseñado | Ver [[impl-extractores-auth]] |
| WorkspaceSeedJob (apalis) | 📝 Diseñado | Ver [[dominio-workspace-seed]] |
| Endpoints Fase 2 | 📝 Planificado | Ver [[plan-fases]] |
| Background jobs Fase 3 | 📝 Planificado | |

---

## 🔗 Relaciones clave entre notas

```
vision-objetivo ──────────────────► vision-arquitectura
     │                                      │
     ▼                                      ▼
vision-stack ──────────────────────► plan-fases
     │                                      │
     ▼                                      ▼
fundamentos-orm ────────────────────► fundamentos-migraciones-estado
     │                                      │
     ▼                                      ▼
impl-bootstrap ─────────────────────► impl-autenticacion
     │                                      │
     ▼                                      ▼
impl-appstate-repository ───────────► impl-extractores-auth
     │                                      │
     ▼                                      ▼
dominio-workspace-seed ─────────────► dominio-integraciones
                                           │
                                           ▼
                                    dominio-workspace-settings
```

---

## 🏷️ Tags del vault

`#rust` `#seaorm` `#axum` `#apalis` `#plane` `#django-migration` `#sea-orm-migration`

---

*Vault: `docs/` rama `feature/integrations-panel-fix-17593507967815292912`*
