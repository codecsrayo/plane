---
titulo: Estado de migraciones SeaORM
tags:
  - migraciones
  - seaorm
  - seguimiento
relacionado:
  - "[[02-orm-y-migraciones]]"
  - "[[07-fases]]"
---

## Estado de migraciones — seguimiento

### Archivos de migración

| Archivo                                  | Tablas creadas                                                                                                                                                                                       | Estado              |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `m20240101_000001_auth_django`           | auth*group, auth_group_permissions, auth_permission, django*\*, changelogs, instances, integrations                                                                                                  | ✅                  |
| `m20240101_000002_users_and_sessions`    | users, accounts, sessions, devices, device_sessions, file_assets, social_login_connections, user_github_connections, profiles                                                                        | ✅                  |
| `m20240101_000003_workspaces_and_tokens` | workspaces, workspace*members, workspace_member_invites, workspace_themes, workspace_integrations, workspace_user*\*, api_tokens, api_activity_logs, webhooks, webhook_logs, notifications, profiles | ✅                  |
| `m20240101_000004_projects_and_states`   | projects, states, labels, estimates, estimate*points, issue_types, project*\*, **project_deploy_boards** ✅ fix                                                                                      | ✅                  |
| `m20240101_000005_issues_and_modules`    | issues, issue*\*, cycles, cycle*_, modules, module\__, pages, page*\*, draft_issues, \*\*draft_issue*\*\*\* ✅ indexes+UQ fix                                                                        | ✅                  |
| `m20240101_000006_integrations_and_misc` | descriptions, description*versions, intakes, intake_issues, deploy_boards, exporters, importers, github*_, gitlab\__, slack_project_syncs                                                            | 🔄 pendiente prueba |

### Fixes aplicados (10 abr 2026)

- `fix`: `needless_borrows_for_generic_args` — removido `&` en 5 llamadas `.name(&format!(...))` en m005
- `fix`: `migration/src/main.rs` — auto-carga `.env` desde raíz del proyecto y construye `DATABASE_URL` desde `POSTGRES_*`
- `fix`: `migration/Cargo.toml` — agregado `dotenvy` como dependencia
- `fix`: `project_deploy_boards` — tabla faltante agregada a m004 (FK a `intakes` sigue diferida a m006)
- `fix`: `draft_issue_assignees/cycles/labels/modules` — agregados indexes y unique constraints parciales (`WHERE deleted_at IS NULL`) en m005

### Comandos de migración

```bash
# Aplicar todas las pendientes
task rust:migrations:up

# Revertir la última
task rust:migrations:down

# Estado completo
task rust:migrations:status

# Drop + re-aplicar todo (dev only)
task rust:migrations:fresh
```

---

