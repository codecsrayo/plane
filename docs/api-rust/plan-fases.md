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

| Fase | Nombre | Duración estimada | Estado |
|------|--------|-------------------|--------|
| 0 | Scaffolding + Baseline | 2–3 días | 🔄 En progreso |
| 1 | Auth middleware | 3–5 días | 📝 Planificado |
| 2 | Endpoints alta frecuencia | 2–4 semanas | 📝 Planificado |
| 3 | Background jobs | 1–2 semanas | 📝 Planificado |
| 4 | Endpoints restantes | Continuo | 📝 Planificado |
| 5 | Shutdown Django completo | 1 día | 📝 Planificado |

---

## Fase 0 — Scaffolding + Baseline (2–3 días)

- [x] Baseline de migraciones SeaORM (m001–m005 ✅, m006 🔄)
- [x] Entities generadas (`sea-orm-cli generate entity`) — 122 entities ✅
- [x] Soft delete implementado ✅
- [ ] `main.rs` completo (AppState + router + Swagger UI) — ver [[impl-bootstrap]]
- [ ] `GET /api/health/` funcionando contra la DB
- [ ] Dockerfile multi-stage
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
- [ ] `GET /api/integrations/` — ver [[dominio-integraciones]]
- [ ] `GET/POST/DELETE /api/workspaces/{slug}/workspace-integrations/` — ver [[dominio-integraciones]]

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

## Fase 4 — Endpoints restantes (~275 paths)

Cubrir el resto priorizando por frecuencia de uso en logs de Traefik.

Categorías pendientes:
- Issues: comments, attachments, links, reactions, relations, sub-issues
- Pages + versions
- Cycles + cycle issues
- Modules + module issues
- Analytics (puede mantenerse en Django más tiempo)
- Import/Export avanzado
- Intake/Triage
- Deploy boards (space API pública)
- Autenticación OAuth (GitHub, GitLab, Google, Gitea)

---

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
