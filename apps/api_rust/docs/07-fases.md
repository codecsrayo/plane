---
titulo: Estrategia de migración por fases
tags:
  - migracion
  - fases
  - planificacion
relacionado: [[04-arquitectura]], [[08-estado-migraciones]]
---

## Estrategia de migración — por fases

### Fase 0 — Scaffolding + Baseline (2–3 días)

- [ ] `cargo new plane-api && cargo new migration`
- [ ] `Cargo.toml` con SeaORM, Axum, Tokio, apalis, utoipa
- [ ] Generar baseline SQL desde la DB actual de Django
- [ ] Crear migración `m_baseline_from_django` con ese SQL
- [ ] Generar entities con `sea-orm-cli generate entity`
- [ ] `GET /api/health/` funcionando contra la DB
- [ ] Dockerfile multi-stage
- [ ] Traefik: routing condicional por path
- [ ] Montar Swagger UI en `/docs`
- [ ] Colección Bruno inicial en `tests/bruno/`

### Fase 1 — Auth middleware

- [ ] Leer tabla `authtoken_token` → `CurrentUser` extractor Axum
- [ ] Role check (workspace_member, project_member) via SeaORM
- [ ] Tests de integración con `axum-test` contra DB real

### Fase 2 — Endpoints de alta frecuencia

- [ ] `GET/POST /api/workspaces/{slug}/projects/{id}/issues/`
- [ ] `GET/PATCH/DELETE /api/workspaces/{slug}/projects/{id}/issues/{id}/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/states/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/members/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/cycles/`
- [ ] `GET /api/workspaces/{slug}/projects/{id}/modules/`
- [ ] `GET /api/workspaces/{slug}/projects/`

### Fase 3 — Background jobs (eliminar Celery + RabbitMQ)

- [ ] Setup apalis con backend PostgreSQL
- [ ] Migrar todos los Celery tasks a apalis workers
- [ ] Setup tokio-cron-scheduler para tareas periódicas
- [ ] Eliminar `bgworker`, `beatworker`, `plane-mq` del docker-compose

### Fase 4 — Endpoints restantes (~275 paths)

Cubrir el resto priorizando por frecuencia de uso en logs.

### Fase 5 — Shutdown Django completo

- [ ] 0 tráfico hacia el contenedor `api`
- [ ] Eliminar `api`, `plane-migrator` del docker-compose
- [ ] Eliminar el directorio `apps/api/` del repo (o archivar en rama)
- [ ] Rust aplica sus propias migraciones en el deploy

---

> [!IMPORTANT] Fase 5 — punto de no retorno
> Una vez eliminado `apps/api/`, no hay rollback sencillo.
> Asegurarse de 0% tráfico hacia el contenedor Django antes de ejecutar.

