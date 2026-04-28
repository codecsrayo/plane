// src/jobs/mod.rs
//! Background jobs processed by apalis on PostgreSQL.
//!
//! Modules:
//!   github_sync          — Initial import of issues from GitHub when creating a RepoSync
//!   notifications        — Creation of in-app notifications for issue activity
//!   export               — Issue export to CSV/ZIP and upload to S3
//!   scheduled            — Periodic tasks: auto-archiving and auto-closing of issues
//!   cleanup              — Periodic cleanup of logs, versions and soft-deletes
//!   email_notification   — Sending emails grouped by receiver every 5 min
//!   instance_traces      — Instance telemetry every 6 h
//!   cron                 — Tokio scheduler that replaces Celery beat
//!   webhook_delivery     — Outgoing webhook delivery with HMAC-SHA256 signature

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
