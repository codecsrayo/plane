---
titulo: Arquitectura actual vs futura
aliases:
  - arquitectura
  - diagramas-arquitectura
tags:
  - arquitectura
  - mermaid
  - django
  - rust
relacionado:
  - "[[MOC]]"
  - "[[vision-stack]]"
  - "[[vision-objetivo]]"
  - "[[plan-fases]]"
  - "[[ref-diagramas-flujo]]"
estado: activo
---

# Arquitectura actual vs futura

---

## Stack actual (Django + Celery + RabbitMQ) — ~750 MB

```mermaid
flowchart TD
    Browser["🌐 Browser / Frontend\n(Next.js + React)"]
    Traefik["⚖️ Traefik\n(proxy / routing)"]

    subgraph Actual["Stack actual ~750 MB"]
        direction TB
        uvicorn["uvicorn\n(servidor ASGI)"]
        django["Django + DRF\n(views, serializers, permissions)"]
        orm_django["Django ORM\n(126 migraciones)"]
        mq["🐇 RabbitMQ plane-mq\n~120 MB"]
        bg["Celery bgworker\n~200 MB\n(sync GitHub, emails,\nexportes, webhooks)"]
        beat["Celery beatworker\n~150 MB\n(cron: limpiar tokens,\ndigest notificaciones)"]
        migrator["plane-migrator\n(manage.py migrate)"]
    end

    subgraph Infra["Infraestructura compartida"]
        pg[("PostgreSQL\nplane-db")]
        redis[("Redis / Valkey\nplane-redis")]
        minio[("MinIO\nplane-minio")]
        live["plane-live\n(Hocuspocus / Y.js CRDT)\nNode.js ~60 MB"]
    end

    Browser --> Traefik
    Traefik --> uvicorn
    uvicorn --> django
    django --> orm_django
    orm_django --> pg
    django -- "task.delay()" --> mq
    mq --> bg
    mq --> beat
    bg --> pg
    bg --> redis
    bg --> minio
    beat --> pg
    migrator -.->|"al arrancar"| pg
    live --> pg
    live --> redis
```

---

## Stack futuro (Rust — ~20 MB total)

```mermaid
flowchart TD
    Browser["🌐 Browser / Frontend\n(Next.js + React)"]
    Traefik["⚖️ Traefik\n(proxy / routing)"]

    subgraph Futuro["Stack Rust ~20 MB"]
        direction TB
        axum["Axum\n(HTTP framework)"]
        seaorm["SeaORM\n(ORM + migraciones)"]
        apalis["apalis\n(background jobs\nvía tabla PostgreSQL)"]
        cron["tokio-cron-scheduler\n(cron jobs — mismo proceso)"]
    end

    subgraph Infra["Infraestructura compartida"]
        pg[("PostgreSQL\nplane-db")]
        redis[("Redis / Valkey\nplane-redis")]
        minio[("MinIO\nplane-minio")]
        live["plane-live\n(Hocuspocus / Y.js CRDT)\nNode.js ~60 MB — NO se toca"]
    end

    Browser --> Traefik
    Traefik --> axum
    axum --> seaorm
    seaorm --> pg
    axum --> apalis
    apalis --> pg
    cron --> apalis
    axum --> redis
    axum --> minio
    live --> pg
    live --> redis
```

---

## Lo que desaparece

```mermaid
flowchart LR
    A["🐇 RabbitMQ\n120 MB"] -- eliminado --> X["❌"]
    B["Celery bgworker\n200 MB"] -- eliminado --> X
    C["Celery beatworker\n150 MB"] -- eliminado --> X
    D["Django + uvicorn\n280 MB"] -- reemplazado --> R["✅ Axum + Rust\n~20 MB"]
    E["plane-migrator\nDjango migrations"] -- reemplazado --> S["✅ sea-orm-migration\n(integrado en el binario)"]
```

### Tabla de impacto por contenedor

| Contenedor               | RAM         | Resultado     |
| ------------------------ | ----------- | ------------- |
| api (Django + uvicorn)   | ~280 MB     | → Rust ~20 MB |
| bgworker (Celery)        | ~200 MB     | → eliminado   |
| beatworker (Celery beat) | ~150 MB     | → eliminado   |
| plane-mq (RabbitMQ)      | ~120 MB     | → eliminado   |
| plane-migrator (Django)  | —           | → eliminado   |
| **Total**                | **~750 MB** | **~20 MB**    |

---

## Lo que se mantiene

| Servicio                        | Por qué                                        |
| ------------------------------- | ---------------------------------------------- |
| plane-live (Hocuspocus/Node.js) | Protocolo Y.js CRDT — no reemplazable          |
| plane-db (PostgreSQL)           | Misma DB, Rust toma ownership del schema       |
| plane-redis (Valkey)            | Sigue necesario para plane-live y caché        |
| plane-minio (MinIO)             | Sin cambio                                     |
| Proxy (Traefik)                 | El mismo, se agrega routing al contenedor Rust |

---

## Impacto en el frontend

> La migración a Rust **no reduce tiempos de carga percibidos** en el browser.
> El cuello de botella es el bundle JS de Next.js y la hydration de React — no la API.
>
> La ganancia es en el **servidor**: -730 MB RAM, mejor throughput bajo carga concurrente,
> y eliminación de 4 contenedores de infraestructura.

---

## Routing dual durante la migración (Fases 1–4)

Durante la migración, Django y Rust corren en paralelo. Traefik los distingue por prioridad:

```yaml
# Rust — alta prioridad, toma los endpoints ya migrados
- "traefik.http.routers.api-rust.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-rust.priority=10"
# Django — baja prioridad, solo recibe lo que Rust no maneja aún
- "traefik.http.routers.api-django.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-django.priority=5"
```

Esto permite mover endpoints uno a uno sin downtime. Ver [[plan-fases]] para la estrategia completa.

---

## 🔗 Navegar

← [[vision-stack]] | [[MOC]] | → [[plan-fases]] | Diagramas detallados: [[ref-diagramas-flujo]]

---

_`docs/api-rust/vision-arquitectura.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
