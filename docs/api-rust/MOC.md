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

_¿Qué, por qué y cómo luce el sistema completo?_

| Nota                    | Contenido                                      |
| ----------------------- | ---------------------------------------------- |
| [[vision-objetivo]]     | Metas, decisiones de diseño, qué queda intacto |
| [[vision-stack]]        | Tabla de librerías Rust vs equivalente Django  |
| [[vision-arquitectura]] | Diagramas actual vs futuro, lo que desaparece  |

---

### 🧱 Fundamentos

_Base técnica: schema, ORM, persistencia._

| Nota                               | Contenido                                        |
| ---------------------------------- | ------------------------------------------------ |
| [[fundamentos-orm]]                | SeaORM, sea-orm-migration, baseline desde Django |
| [[fundamentos-migraciones-estado]] | Estado de m001–m007, fixes aplicados, comandos   |
| [[fundamentos-siembra-datos]]      | Integrations, Instance Config y Workspace Seed    |
| [[fundamentos-soft-delete]]        | Trait SoftDeleteExt + macro `impl_soft_delete!`  |

---

### ⚙️ Implementación

_Código concreto: cómo se construye la API._

| Nota                         | Contenido                                                   |
| ---------------------------- | ----------------------------------------------------------- |
| [[impl-bootstrap]]           | `main.rs`, `config.rs`, `error.rs`, Swagger UI              |
| [[impl-autenticacion]]       | Session Cookie + API Key — extractores completos            |
| [[impl-appstate-repository]] | AppState, Repository Pattern — aislar SeaORM                |
| [[impl-extractores-auth]]    | CurrentUser, WorkspaceMemberGuard, ProjectMemberGuard, RBAC |
| [[impl-error-jobs-cron]]     | AppError, Job Pattern (apalis), Cron (tokio-cron-scheduler) |

---

### 🏗️ Dominio

_Lógica de negocio de las features principales._

| Nota                           | Contenido                                                                                       |
| ------------------------------ | ----------------------------------------------------------------------------------------------- |
| [[dominio-workspace-seed]]     | WorkspaceSeedJob apalis — datos iniciales JSON                                                  |
| [[dominio-integraciones]]      | GitHub App, GitLab OAuth, Slack OAuth — flujos y endpoints                                      |
| [[dominio-workspace-settings]] | Panel de configuración: General, Members, Exports, Webhooks                                     |
| [[dominio-issues]]             | Issues CRUD completo, comments, attachments, links, reactions, relations, sub-issues, actividad |
| [[dominio-proyectos]]          | Projects CRUD, members, invitations, states, labels, estimates, identificadores                 |
| [[dominio-ciclos]]             | Cycles CRUD, cycle-issues, transfers, progress, analytics por ciclo                             |
| [[dominio-modulos]]            | Modules CRUD, module-issues, links, sub-issues anidados                                         |
| [[dominio-paginas]]            | Pages CRUD, bloques, favoritos, permisos, public access                                         |
| [[dominio-notificaciones]]     | Notificaciones, reads, suscripciones, email/in-app delivery                                     |
| [[dominio-intake]]             | Intake (Triage): sources, filters, accept/decline, conversión a issue                           |
| [[dominio-analytics]]          | Analytics: demand, burn-down, custom charts, exports                                            |
| [[dominio-importadores]]       | Importadores: GitHub, Jira, CSV — jobs de migración                                             |
| [[dominio-busqueda]]           | Global search, workspace search, filtros avanzados                                              |
| [[dominio-vistas]]             | Vistas guardadas (Project/Workspace) y favoritos                                                |
| [[dominio-usuario]]            | Perfil, configuración, cuentas OAuth y dashboards                                               |
| [[dominio-servicios-core]]     | Assets (v1/v2), API Tokens y Timezones                                                          |
| [[dominio-ia]]                 | Asistente Pi, reformulación de texto, multi-proveedor LLM (OpenAI/Anthropic/Gemini)             |

---

### 📋 Planificación

_Estrategia de migración y puntos de riesgo._

| Nota             | Contenido                                               |
| ---------------- | ------------------------------------------------------- |
| [[plan-fases]]   | Estrategia en 6 fases (Fase 0 → Fase 5 shutdown Django) |
| [[plan-riesgos]] | 13 puntos críticos de riesgo operacional                |

---

### 📚 Referencia

_Documentación de apoyo: estructura, diagramas, testing._

| Nota                        | Contenido                                                |
| --------------------------- | -------------------------------------------------------- |
| [[ref-estructura-archivos]] | Árbol de directorios del proyecto Rust (estado objetivo) |
| [[ref-estructura-django]]   | Árbol completo de `apps/api` — fuente de migración       |
| [[ref-diagramas-secuencia]] | Sequence diagrams: workspace seed, auth, requests        |
| [[ref-diagramas-flujo]]     | Flowcharts: workspace, projects, issues, roles           |
| [[ref-testing]]             | Swagger UI, axum-test, Bruno — estrategia completa       |

---

## 📊 Estado de implementación

| Área                               | Estado               | Notas                                                 |
| ---------------------------------- | -------------------- | ----------------------------------------------------- |
| Soft delete                        | ✅ Implementado      | `src/utils/soft_delete.rs`                            |
| Migraciones m001–m006 (Baseline)   | ✅ Aplicadas         | Schema completo vía `sql/baseline.sql`                |
| Migración m007 (Seeds)             | ✅ Aplicadas         | Integraciones e Instance config inicial               |
| Bootstrap API (`main.rs`)          | 📝 Diseñado          | Ver [[impl-bootstrap]]                                |
| Auth Session Cookie                | 📝 Diseñado          | Ver [[impl-autenticacion]]                            |
| Auth API Key                       | 📝 Diseñado          | Ver [[impl-autenticacion]]                            |
| Extractores WorkspaceMemberGuard   | 📝 Diseñado          | Ver [[impl-extractores-auth]]                         |
| WorkspaceSeedJob (apalis)          | 📝 Diseñado          | Ver [[dominio-workspace-seed]]                        |
| Endpoints Fase 2                   | 📝 Planificado       | Ver [[plan-fases]]                                    |
| Background jobs Fase 3             | 📝 Planificado       |                                                       |
| Dominio Issues (Fase 4)            | 📝 Documentado       | Ver [[dominio-issues]] — ~20 endpoints                |
| Dominio Proyectos (Fase 4)         | 📝 Documentado       | Ver [[dominio-proyectos]] — ~15 endpoints             |
| Dominio Ciclos (Fase 4)            | 📝 Documentado       | Ver [[dominio-ciclos]] — ~8 endpoints                 |
| Dominio Módulos (Fase 4)           | 📝 Documentado       | Ver [[dominio-modulos]] — ~8 endpoints                |
| Dominio Páginas (Fase 4)           | 📝 Documentado       | Ver [[dominio-paginas]] — ~8 endpoints                |
| Dominio Notificaciones (Fase 4)    | 📝 Documentado       | Ver [[dominio-notificaciones]] — ~6 endpoints         |
| Dominio Intake (Fase 4)            | 📝 Documentado       | Ver [[dominio-intake]] — ~6 endpoints                 |
| Dominio Analytics (Fase 4)         | 📝 Documentado       | Ver [[dominio-analytics]] — ~5 endpoints              |
| Dominio Importadores (Fase 4)      | 📝 Documentado       | Ver [[dominio-importadores]] — ~4 endpoints           |
| Dominio Búsqueda (Fase 4)          | 📝 Documentado       | Ver [[dominio-busqueda]] — ~3 endpoints               |
| Dominio IA / Asistente Pi (Fase 4) | 🔴 Pendiente urgente | Ver [[dominio-ia]] — 3 endpoints, 1 ausente en Django |

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
fundamentos-siembra-datos ──────────► dominio-workspace-seed
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
                              ┌────────────┼────────────┐
                              ▼            ▼            ▼
                   dominio-proyectos  dominio-issues  dominio-workspace-settings
                              │            │
                    ┌─────────┤            ├──────────────┐
                    ▼         ▼            ▼              ▼
             dominio-ciclos  dominio-  dominio-       dominio-
                            modulos   notificaciones  paginas
                                           │
                              ┌────────────┼─────────────┐
                              ▼            ▼             ▼
                       dominio-intake  dominio-analytics dominio-
                                                        importadores
                                                             │
                                                             ▼
                                                      dominio-busqueda
                                                             │
                                                             ▼
                                                        dominio-ia
```

---

## 🏷️ Tags del vault

`#rust` `#seaorm` `#axum` `#apalis` `#plane` `#django-migration` `#sea-orm-migration`

---

_Vault: `docs/` rama `feature/integrations-panel-fix-17593507967815292912`_
