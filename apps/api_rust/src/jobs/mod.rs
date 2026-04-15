// src/jobs/mod.rs
//! Background jobs procesados por apalis sobre PostgreSQL.
//!
//! Módulos:
//!   github_sync   — Importación inicial de issues desde GitHub al crear un RepoSync
//!   notifications — Creación de notificaciones in-app para actividad de issues
//!   export        — Exportación de issues a CSV/ZIP y subida a S3
//!   scheduled     — Tareas periódicas: archivado y cierre automático de issues

pub mod export;
pub mod github_sync;
pub mod notifications;
pub mod scheduled;
pub mod workspace_seed;
