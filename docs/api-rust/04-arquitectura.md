---
titulo: Arquitectura actual vs futura
tags:
  - arquitectura
  - mermaid
  - django
  - rust
relacionado:
  - "[[03-stack]]"
  - "[[07-fases]]"
---

## Arquitectura actual vs futura — diagrama de flujo

### Stack actual (Django + Celery + RabbitMQ)

```mermaid
flowchart TD
    Browser["🌐 Browser / Frontend
(Next.js + React)"]
    Traefik["⚖️ Traefik
(proxy / routing)"]

    subgraph Actual["Stack actual ~750 MB"]
        direction TB
        uvicorn["uvicorn
(servidor ASGI)"]
        django["Django + DRF
(views, serializers, permissions)"]
        orm_django["Django ORM
(127 migraciones)"]
        mq["🐇 RabbitMQ plane-mq
~120 MB"]
        bg["Celery bgworker
~200 MB
(sync GitHub, emails,
exportes, webhooks)"]
        beat["Celery beatworker
~150 MB
(cron: limpiar tokens,
digest notificaciones)"]
        migrator["plane-migrator
(manage.py migrate)"]
    end

    subgraph Infra["Infraestructura compartida"]
        pg[("PostgreSQL
plane-db")]
        redis[("Redis / Valkey
plane-redis")]
        minio[("MinIO
plane-minio")]
        live["plane-live
(Hocuspocus / Y.js CRDT)
Node.js ~60 MB"]
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

### Stack futuro (Rust — ~20 MB total)

```mermaid
flowchart TD
    Browser["🌐 Browser / Frontend
(Next.js + React)"]
    Traefik["⚖️ Traefik
(proxy / routing)"]

    subgraph Futuro["Stack Rust ~20 MB"]
        direction TB
        axum["Axum
(HTTP framework)"]
        seaorm["SeaORM
(ORM + migraciones)"]
        apalis["apalis
(background jobs
vía tabla PostgreSQL)"]
        cron["tokio-cron-scheduler
(cron jobs — mismo proceso)"]
    end

    subgraph Infra["Infraestructura compartida"]
        pg[("PostgreSQL
plane-db")]
        redis[("Redis / Valkey
plane-redis")]
        minio[("MinIO
plane-minio")]
        live["plane-live
(Hocuspocus / Y.js CRDT)
Node.js ~60 MB — NO se toca"]
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

### Lo que desaparece

```mermaid
flowchart LR
    A["🐇 RabbitMQ
120 MB"] -- eliminado --> X["❌"]
    B["Celery bgworker
200 MB"] -- eliminado --> X
    C["Celery beatworker
150 MB"] -- eliminado --> X
    D["Django + uvicorn
280 MB"] -- reemplazado --> R["✅ Axum + Rust
~20 MB"]
    E["plane-migrator
Django migrations"] -- reemplazado --> S["✅ sea-orm-migration
(integrado en el binario)"]
```

### Impacto en frontend

> La migración a Rust **no reduce tiempos de carga percibidos** en el browser.
> El cuello de botella es el bundle JS de Next.js y la hydration de React — no la API.
> La ganancia es en el **servidor**: -730 MB RAM, mejor throughput bajo carga concurrente,
> y eliminación de 4 contenedores de infraestructura.

---

### Lo que desaparece

| Contenedor               | RAM         | Resultado     |
| ------------------------ | ----------- | ------------- |
| api (Django + uvicorn)   | ~280 MB     | → Rust ~20 MB |
| bgworker (Celery)        | ~200 MB     | → eliminado   |
| beatworker (Celery beat) | ~150 MB     | → eliminado   |
| plane-mq (RabbitMQ)      | ~120 MB     | → eliminado   |
| plane-migrator (Django)  | —           | → eliminado   |
| **Total**                | **~750 MB** | **~20 MB**    |

### Lo que se mantiene

| Servicio                        | Por qué                                        |
| ------------------------------- | ---------------------------------------------- |
| plane-live (Hocuspocus/Node.js) | protocolo Y.js CRDT — no reemplazable          |
| plane-db (PostgreSQL)           | misma DB, Rust toma ownership del schema       |
| plane-redis (Valkey)            | sigue necesario para plane-live y caché        |
| plane-minio (MinIO)             | sin cambio                                     |
| proxy (Traefik)                 | el mismo, se agrega routing al contenedor Rust |

---

