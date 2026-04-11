---
titulo: Estructura completa de archivos
tags:
  - estructura
  - rust
  - archivos
relacionado:
  - "[[05-testing]]"
  - "[[10-patrones]]"
---

## Estructura completa de archivos — estado objetivo

```
apps/api_rust/
├── Cargo.toml
├── Dockerfile
├── seeds/
│   └── data/
│       ├── projects.json    ← copiado de apps/api/plane/seeds/data/
│       ├── states.json
│       ├── labels.json
│       ├── cycles.json
│       ├── modules.json
│       ├── issues.json
│       ├── views.json
│       └── pages.json
├── src/
│   ├── main.rs              ← bootstrap: AppState + router + workers + scheduler
│   ├── config.rs            ← env vars tipadas (dotenvy)
│   ├── error.rs             ← AppError → HTTP responses (thiserror)
│   ├── lib.rs
│   ├── auth/
│   │   ├── middleware.rs    ← CurrentUser extractor
│   │   └── permissions.rs  ← WorkspaceMemberGuard, ProjectMemberGuard
│   ├── entities/            ← 122 entidades generadas por sea-orm-cli (NO editar)
│   ├── repositories/        ← acceso a DB aislado, un archivo por dominio
│   │   ├── mod.rs
│   │   ├── issues.rs
│   │   ├── workspaces.rs
│   │   ├── projects.rs
│   │   ├── states.rs
│   │   └── ...
│   ├── routes/              ← handlers Axum, llaman a repositories
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   ├── cycles.rs
│   │   ├── modules.rs
│   │   └── integrations.rs
│   ├── jobs/                ← apalis workers + cron
│   │   ├── mod.rs           ← build_monitor() — registra todos los workers
│   │   ├── workspace_seed/
│   │   │   ├── mod.rs       ← WorkspaceSeedJob, handle_workspace_seed
│   │   │   └── seed_data.rs ← structs de deserialización de JSON
│   │   ├── github_sync.rs
│   │   ├── notifications.rs
│   │   ├── export.rs
│   │   └── scheduled.rs     ← tokio-cron-scheduler (reemplaza beatworker)
│   └── utils/
│       ├── mod.rs
│       └── soft_delete.rs   ← SoftDeleteExt trait + impl_soft_delete! macro ✅
├── migration/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            ← Migrator con todas las migraciones en orden
│       ├── main.rs
│       └── migrations/
│           ├── mod.rs
│           ├── m20260410_000001_baseline.rs  ← schema completo desde Django ✅
│           └── m20240101_000007_seed_data.rs ← integrations + instance_configs ✅
└── tests/
    └── bruno/
        ├── bruno.json
        ├── environments/
        │   ├── local.bru
        │   └── staging.bru
        └── health/
            └── get_health.bru
```

---

