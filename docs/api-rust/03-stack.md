---
titulo: Stack completo Rust
tags:
  - rust
  - stack
  - axum
  - seaorm
  - apalis
relacionado: [[00-README]], [[04-arquitectura]]
---

## Stack completo

| Rol             | Librería                         | Reemplaza                        |
| --------------- | -------------------------------- | -------------------------------- |
| HTTP framework  | **Axum**                         | Django REST Framework + uvicorn  |
| Async runtime   | **Tokio**                        | —                                |
| ORM             | **SeaORM**                       | Django ORM                       |
| Migrations      | **sea-orm-migration**            | Django migrations (127 archivos) |
| Serialización   | **serde + serde_json**           | DRF serializers                  |
| Auth JWT        | **jsonwebtoken**                 | DRF TokenAuthentication          |
| Background jobs | **apalis** (backend: Postgres)   | Celery bgworker + RabbitMQ       |
| Cron jobs       | **tokio-cron-scheduler**         | Celery beatworker                |
| Redis           | **fred**                         | django-redis                     |
| S3 / MinIO      | **aws-sdk-s3**                   | boto3 + django-storages          |
| Email           | **lettre**                       | Django email backend             |
| HTTP client     | **reqwest**                      | requests                         |
| Logging         | **tracing + tracing-subscriber** | python-json-logger               |
| Config          | **dotenvy**                      | django settings                  |
| Métricas        | **axum-prometheus**              | scout-apm                        |
| API Docs        | **utoipa + utoipa-swagger-ui**   | drf-spectacular                  |

---

> [!TIP] Referencia rápida
> Para ver cómo cada librería se usa en la práctica, ver [[10-patrones]].

