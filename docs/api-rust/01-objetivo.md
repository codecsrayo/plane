---
titulo: Objetivo y decisiones iniciales
tags:
  - plane
  - rust
  - objetivo
relacionado: [[00-README]]
---

## Objetivo

Reemplazar la API Django + Celery por un stack Rust completo que toma ownership
total del schema PostgreSQL. Django desaparece incluyendo sus migraciones.

**Metas:**

- ~750 MB (Django + Celery + RabbitMQ) → ~20 MB
- Eliminar bgworker, beatworker, RabbitMQ
- Rust es el dueño del schema — no más migraciones Django
- Mismo PostgreSQL, mismo Redis/Valkey, mismo plane-live

---

## plane-live — NO se toca

`plane-live` es un servidor **Hocuspocus (Y.js CRDT)** para edición colaborativa
de Pages e issue descriptions. Implementa sincronización CRDT por WebSocket,
persiste estado binario Y.js + HTML via HTTP a la API, y usa Redis para sync
entre múltiples instancias. Es Node.js (~60 MB), no Python. No es el problema.

**Se mantiene intacto.** Solo se migra Django → Rust.

---

