---
titulo: Estrategia de migración por fases
aliases:
  - fases
  - roadmap
  - plan-migracion
tags:
  - migracion
  - fases
  - planificacion
relacionado:
  - "[[MOC]]"
  - "[[vision-arquitectura]]"
  - "[[fundamentos-migraciones-estado]]"
  - "[[plan-riesgos]]"
  - "[[impl-bootstrap]]"
estado: activo
---

# Estrategia de migración — por fases

---

## Resumen de fases

| Fase | Nombre                    | Duración estimada | Estado         |
| ---- | ------------------------- | ----------------- | -------------- |
| 0    | Scaffolding + Baseline    | 2–3 días          | ✅ Completada  |
| 1    | Auth middleware           | 3–5 días          | 🔄 En progreso |
| 2    | Endpoints alta frecuencia | 2–4 semanas       | 📝 Planificado |
| 3    | Background jobs           | 1–2 semanas       | 📝 Planificado |
| 4    | Endpoints restantes       | Continuo          | 📝 Planificado |
| 5    | Shutdown Django completo  | 1 día             | 📝 Planificado |

---

## Fase 0 — Scaffolding + Baseline (2–3 días)

### ✅ Implementado en código real

- [x] Baseline SQL consolidado en `m20260410_000001_baseline.rs` — schema completo (tablas + índices + unique constraints parciales `WHERE deleted_at IS NULL`) cargado vía `include_str!("../sql/baseline.sql")`
- [x] Seed data en `m20240101_000007_seed_data.rs` — 3 filas en `integrations` (github/gitlab/slack) + todas las keys en `instance_configurations`; idempotente con `ON CONFLICT DO NOTHING`
- [x] Entities generadas — 122 entidades en `src/entities/` vía `sea-orm-cli generate entity`; incluye entidades de integraciones (`workspace_integrations`, `github_repositories`, `github_repository_syncs`, `db_githubprstatemapping`, `slack_project_syncs`, `user_github_connections`, etc.)
- [x] Soft delete implementado — `src/utils/soft_delete.rs`: trait `SoftDeleteExt` + macro `impl_soft_delete!`
- [x] `Cargo.toml` con dependencias completas — axum 0.8, sea-orm 1.1, apalis 0.7, reqwest 0.13, jsonwebtoken, ammonia 4, utoipa 5, lettre 0.11, fred 10 (Redis), aws-sdk-s3
- [x] `main.rs` completo — AppState + router + Scalar UI + tracing — ver [[impl-bootstrap]]
- [x] `config.rs` — env vars tipadas, ensambla `database_url` y `redis_url` desde vars crudas
- [x] `error.rs` — `AppError` con thiserror → HTTP responses
- [x] `auth/rate_limit.rs` — `RateLimitState` in-memory (Fase 3 → Redis)
- [x] `GET /api/health/` funcionando contra la DB — 200 OK / 503 si DB caída
- [x] Scalar UI en `/api/docs` (solo `DEBUG=1`) — spec OpenAPI disponible

### ❌ Pendiente

- [ ] `Dockerfile` multi-stage — archivo existe pero está vacío
- [ ] Traefik: routing condicional por path — ver [[vision-arquitectura#Routing dual durante la migración Fases 1 4]]
- [ ] Colección Bruno inicial en `tests/bruno/` — ver [[ref-testing]]

---

## Fase 1 — Auth middleware

- [ ] Session Cookie extractor (`SessionUser`) — ver [[impl-autenticacion]]
- [ ] API Key extractor (`ApiKeyUser`) — ver [[impl-autenticacion]]
- [ ] `CurrentUser` extractor (Bearer Token para APIs internas) — ver [[impl-extractores-auth]]
- [ ] `WorkspaceMemberGuard` — ver [[impl-extractores-auth]]
- [ ] `ProjectMemberGuard` — ver [[impl-extractores-auth]]
- [ ] Rate limit middleware (API keys) — ver [[impl-autenticacion]]
- [ ] Tests de integración con `axum-test` contra DB real — ver [[ref-testing]]

---

## Fase 2 — Endpoints de alta frecuencia

### Workspaces

- [ ] `GET /api/workspaces/`
- [ ] `POST /api/workspaces/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/`
- [ ] `GET /api/workspaces/{slug}/members/`
- [ ] `PATCH/DELETE /api/workspaces/{slug}/members/{pk}/`
- [ ] `GET/POST /api/workspaces/{slug}/invitations/`
- [ ] `DELETE /api/workspaces/{slug}/invitations/{pk}/`

### Projects

- [ ] `GET /api/workspaces/{slug}/projects/`
- [ ] `POST /api/workspaces/{slug}/projects/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/`

### Issues

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/`

### Otros alta frecuencia

- [ ] `GET /api/workspaces/{slug}/projects/{id}/states/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/members/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/modules/`

### Workspace Settings (Fase 2 extendida)

- [ ] `GET/POST /api/workspaces/{slug}/webhooks/` — ver [[dominio-workspace-settings]]

### Integraciones (Fase 2 extendida) — ver [[dominio-integraciones]]

#### Globales
- [ ] `GET /api/integrations/` — listado de providers disponibles (3 filas estáticas: github/gitlab/slack)
- [ ] `GET /api/github/callback/` — sin auth; recibe `installation_id` + `state={workspace_slug}` de GitHub App; devuelve HTML con `postMessage`
- [ ] `POST /api/auth/github/user-callback/` — auth requerida; intercambia `code` por access_token OAuth personal; guarda en `user_github_connections`

#### Por workspace (requieren WorkspaceAdmin)
- [ ] `GET /api/workspaces/{slug}/workspace-integrations/`
- [ ] `POST /api/workspaces/{slug}/workspace-integrations/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/workspace-integrations/{pk}/`
- [ ] `DELETE /api/workspaces/{slug}/workspace-integrations/{provider}/provider/` — desinstala por nombre de provider
- [ ] `POST /api/workspaces/{slug}/workspace-integrations/{provider}/install/` — github: `{installation_id}`, gitlab/slack: `{code}`

#### Repos y sincronización (requieren WorkspaceAdmin)
- [ ] `GET /api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/` — lista repos accesibles vía installation token
- [ ] `GET/POST /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/` — listar/crear `GithubRepositorySync`
- [ ] `DELETE /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/{id}/` — desconectar repo; debe soft-delete `GithubRepository` + `GithubRepositorySync` juntos para evitar huérfanos

#### PR State Mapping
- [ ] `GET/POST /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/`
- [ ] `DELETE /api/workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{id}/`

#### Webhooks entrantes (sin auth de usuario — validación HMAC obligatoria)
- [ ] `POST /api/github-webhook/` — verificar `X-Hub-Signature-256`; handlers para `issues`, `pull_request`, `issue_comment`
- [ ] `POST /api/gitlab-webhook/` — verificar `X-Gitlab-Token`; handlers para `merge_request`, `note`

---

## Fase 3 — Background jobs (eliminar Celery + RabbitMQ)

- [ ] Setup apalis con backend PostgreSQL — ver [[impl-error-jobs-cron]]
- [ ] `WorkspaceSeedJob` — ver [[dominio-workspace-seed]]
- [ ] `GithubInitialIssueSyncJob` — ver [[dominio-integraciones]]
- [ ] `NotificationJob` + `EmailJob`
- [ ] `ExportJob` — ver [[dominio-workspace-settings#WS-4 — Exports]]
- [ ] `WebhookDeliveryJob` — ver [[dominio-workspace-settings#WS-6 — Webhooks]]
- [ ] tokio-cron-scheduler para tareas periódicas — ver [[impl-error-jobs-cron]]
- [ ] **Eliminar** `bgworker`, `beatworker`, `plane-mq` del docker-compose

---

## Fase 4 — Endpoints restantes (~83 endpoints)

Cubrir el resto priorizando por frecuencia de uso en logs de Traefik.
Ver docs de dominio en [[MOC#🏗️ Dominio]] para contexto detallado de cada área.

### Issues — ver [[dominio-issues]]

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/{id}/comments/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/comments/{comment_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/{id}/attachments/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/attachments/{attachment_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/{id}/links/`
- [ ] `PATCH/DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/links/{link_id}/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/issues/{id}/reactions/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/reactions/{reaction_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/{id}/relations/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/relations/{relation_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/{id}/sub-issues/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/issues/{id}/activity/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/` _(bulk create)_
- [ ] `POST /api/workspaces/{slug}/projects/{id}/issues/bulk-update/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/issues/bulk-delete/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/issues/{id}/` _(expandido con relaciones)_
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issue-views/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/issue-views/{view_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/issue-views/` _(workspace-level views)_
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/spreadsheet-states/`

### Proyectos — ver [[dominio-proyectos]]

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/states/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/states/{state_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/labels/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/labels/{label_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/estimates/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/estimates/{estimate_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/identifiers/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/members/`
- [ ] `PATCH/DELETE /api/workspaces/{slug}/projects/{id}/members/{member_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/invitations/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/invitations/{invite_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/` _(listado paginado completo con filtros)_

### Ciclos — ver [[dominio-ciclos]]

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/cycles/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/cycle-issues/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/cycle-issues/{issue_id}/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/transfer-issues/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/{cycle_id}/progress/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/current/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/upcoming/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/completed/`

### Módulos — ver [[dominio-modulos]]

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/modules/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/modules/{module_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/modules/{module_id}/module-issues/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/modules/{module_id}/module-issues/{issue_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/modules/{module_id}/links/`
- [ ] `PATCH/DELETE /api/workspaces/{slug}/projects/{id}/modules/{module_id}/links/{link_id}/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/modules/{module_id}/sub-issues/`

### Páginas — ver [[dominio-paginas]]

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/pages/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/pages/{page_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/pages/{page_id}/blocks/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/pages/{page_id}/blocks/{block_id}/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/pages/{page_id}/favorite/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/pages/{page_id}/favorite/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/pages/{page_id}/archive/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/pages/{page_id}/unarchive/`

### Notificaciones — ver [[dominio-notificaciones]]

- [ ] `GET /api/users/me/notifications/`
- [ ] `PATCH /api/users/me/notifications/{notification_id}/read/`
- [ ] `POST /api/users/me/notifications/mark-all-read/`
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/{id}/subscriptions/`
- [ ] `DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/subscriptions/`
- [ ] `GET /api/users/me/notification-preferences/`
- [ ] `PATCH /api/users/me/notification-preferences/`

### Intake / Triage — ver [[dominio-intake]]

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/intake/` _(sources)_
- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/intake-issues/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/intake-issues/{intake_id}/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/intake-issues/{intake_id}/accept/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/intake-issues/{intake_id}/decline/`
- [ ] `POST /api/workspaces/{slug}/projects/{id}/intake-issues/{intake_id}/snoozed/`

### Analytics — ver [[dominio-analytics]]

- [ ] `GET /api/workspaces/{slug}/analytics/` _(demand analytics)_
- [ ] `GET /api/workspaces/{slug}/analytics/export/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/analytics/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/analytics/burn-down/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/analytics/custom/`

### Importadores — ver [[dominio-importadores]]

- [ ] `GET/POST /api/workspaces/{slug}/importers/github/`
- [ ] `GET/DELETE /api/workspaces/{slug}/importers/github/{importer_id}/`
- [ ] `GET/POST /api/workspaces/{slug}/importers/jira/`
- [ ] `GET/POST /api/workspaces/{slug}/importers/csv/`

### Búsqueda — ver [[dominio-busqueda]]

- [ ] `GET /api/workspaces/{slug}/search/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/search/`
- [ ] `GET /api/search/` _(global)_

## Fase 5 — Shutdown Django completo

> [!DANGER] Punto de no retorno
> Una vez eliminado `apps/api/`, no hay rollback sencillo.
> Asegurarse de **0% tráfico** hacia el contenedor Django antes de ejecutar.

- [ ] Verificar métricas Traefik: 0 requests llegando al contenedor `api`
- [ ] Eliminar `api`, `bgworker`, `beatworker`, `plane-mq`, `plane-migrator` del docker-compose
- [ ] Eliminar configuración de Django de Traefik
- [ ] Rust aplica sus propias migraciones en el deploy: `./plane-api migrate up`
- [ ] Archivar (no borrar) el directorio `apps/api/` en una rama separada

---

## Routing Traefik durante las Fases 1–4

```yaml
# Rust — alta prioridad, toma los endpoints ya migrados
- "traefik.http.routers.api-rust.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-rust.priority=10"
# Django — baja prioridad, solo recibe lo que Rust no maneja aún
- "traefik.http.routers.api-django.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-django.priority=5"
```

Esto permite mover endpoints uno a uno sin downtime. Ver [[vision-arquitectura]].

---

## 🔗 Navegar

← [[MOC]] | → [[plan-riesgos]]

**Relacionado:** Arquitectura: [[vision-arquitectura]] | Bootstrap: [[impl-bootstrap]] | Estado actual: [[fundamentos-migraciones-estado]]

---

_`docs/api-rust/plan-fases.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
