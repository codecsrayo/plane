---
titulo: Objetivo y decisiones iniciales
aliases:
  - objetivo
  - metas
tags:
  - plane
  - rust
  - vision
relacionado:
  - "[[MOC]]"
  - "[[vision-stack]]"
  - "[[vision-arquitectura]]"
estado: activo
---

# Objetivo del proyecto

> [!SUMMARY] Una frase
> Reemplazar Django + Celery (~750 MB RAM) por un binario Rust único (~20 MB) que toma ownership total del schema PostgreSQL.

---

## Metas concretas

| Métrica | Antes (Django) | Después (Rust) |
|---------|---------------|----------------|
| RAM total API | ~750 MB | ~20 MB |
| Contenedores | 5 (`api`, `bgworker`, `beatworker`, `plane-mq`, `plane-migrator`) | 1 |
| Message broker | RabbitMQ (120 MB) | Eliminado |
| Background jobs | Celery via RabbitMQ | apalis via PostgreSQL |
| Cron jobs | Celery beatworker | tokio-cron-scheduler (mismo proceso) |

**Quién es el dueño del schema PostgreSQL:** Rust, a través de `sea-orm-migration`. Django desaparece incluyendo sus 126 migraciones históricas.

---

## plane-live — NO se toca

`plane-live` es un servidor **Hocuspocus (Y.js CRDT)** para edición colaborativa de Pages e issue descriptions.

**Lo que hace:**
- Sincronización CRDT por WebSocket
- Persiste estado binario Y.js + HTML via HTTP a la API
- Usa Redis para sync entre múltiples instancias

**Por qué no se migra:**
- Es Node.js (~60 MB), no Python — no es el problema de RAM
- Protocolo Y.js CRDT no es reemplazable trivialmente
- No tiene conexión al código Django que se elimina

**Se mantiene intacto.** Solo se migra Django → Rust.

---

## Decisiones de diseño relevantes

| Decisión | Opción elegida | Alternativa rechazada | Razón |
|----------|---------------|-----------------------|-------|
| ORM | SeaORM | Diesel | Async Tokio nativo + generación de entities desde DB |
| Background jobs | apalis (PostgreSQL-backed) | Celery + RabbitMQ | Elimina un broker entero; jobs en la misma DB |
| Soft delete | Trait custom | `seaorm-soft-delete` crate | Versión 0.1.0 incompatible con sea-orm 1.1.x |
| Auth session | Cookie `session-id` + tabla `sessions` | JWT | Compatibilidad con el frontend Next.js existente |
| Documentación API | utoipa + Swagger UI | OpenAPI manual | Generación automática desde macros Rust |
| Migrations | `sea-orm-migration` consolidado en baseline | Replicar 126 migraciones Django | Impracticable; un baseline hace lo mismo |

---

## Lo que se mantiene sin cambios

| Servicio | Por qué |
|----------|---------|
| `plane-live` (Hocuspocus/Node.js) | Protocolo Y.js CRDT — no reemplazable |
| `plane-db` (PostgreSQL) | Misma DB, Rust toma ownership del schema |
| `plane-redis` (Valkey) | Necesario para plane-live y caché |
| `plane-minio` (MinIO) | Sin cambio |
| Proxy (Traefik) | Se mantiene, agrega routing al contenedor Rust |

---

## 🔗 Navegar

← [[MOC]] | → [[vision-stack]] | → [[vision-arquitectura]] | → [[plan-fases]]
