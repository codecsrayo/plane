// src/jobs/mod.rs
//! Background jobs procesados por apalis sobre PostgreSQL.
//!
//! Módulos:
//!   github_sync          — Importación inicial de issues desde GitHub al crear un RepoSync
//!   notifications        — Creación de notificaciones in-app para actividad de issues
//!   export               — Exportación de issues a CSV/ZIP y subida a S3
//!   scheduled            — Tareas periódicas: archivado y cierre automático de issues
//!   cleanup              — Limpieza periódica de logs, versiones y soft-deletes
//!   email_notification   — Envío de emails agrupados por receptor cada 5 min
//!   instance_traces      — Telemetría de instancia cada 6 h
//!   cron                 — Scheduler tokio que reemplaza Celery beat
//!   webhook_delivery     — Entrega de webhooks salientes con firma HMAC-SHA256

pub mod cleanup;
pub mod cron;
pub mod email_notification;
pub mod export;
pub mod github_sync;
pub mod instance_traces;
pub mod notifications;
pub mod scheduled;
pub mod webhook_delivery;
pub mod workspace_seed;
