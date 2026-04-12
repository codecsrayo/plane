---
titulo: Estado de migraciones SeaORM
aliases:
  - estado-migraciones
  - migraciones-m001-m006
tags:
  - migraciones
  - seaorm
  - seguimiento
  - fundamentos
relacionado:
  - "[[MOC]]"
  - "[[fundamentos-orm]]"
  - "[[plan-fases]]"
  - "[[plan-riesgos]]"
estado: activo
---

# Estado de migraciones SeaORM

> [!SUCCESS] Actualizado: 10 abril 2026
> m001–m005 aplicadas. m006 pendiente de prueba en entorno completo.

---

## Seguimiento de archivos de migración

| Archivo                                  | Tablas principales creadas                                                                                                                                              | Estado              |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `m20260410_000001_baseline`              | auth*group, auth_permission, django*\*, changelogs, instances, integrations                                                                                             | ✅                  |
| `m20240101_000002_users_and_sessions`    | users, accounts, sessions, devices, file_assets, social_login_connections, user_github_connections, profiles                                                            | ✅                  |
| `m20240101_000003_workspaces_and_tokens` | workspaces, workspace_members, workspace_member_invites, workspace_themes, workspace_integrations, api_tokens, api_activity_logs, webhooks, webhook_logs, notifications | ✅                  |
| `m20240101_000004_projects_and_states`   | projects, states, labels, estimates, estimate*points, issue_types, project*\*, **project_deploy_boards**                                                                | ✅                  |
| `m20240101_000005_issues_and_modules`    | issues, issue*\*, cycles, cycle*_, modules, module\__, pages, page*\*, draft_issues, draft_issue*\*                                                                     | ✅                  |
| `m20240101_000006_integrations_and_misc` | descriptions, description*versions, intakes, intake_issues, deploy_boards, exporters, importers, github*_, gitlab\__, slack_project_syncs                               | 🔄 pendiente prueba |

> [!NOTE] Tablas de integraciones en m006
> Las entidades SeaORM para GitHub, GitLab y Slack ya están generadas en `src/entities/`. Ver [[dominio-integraciones]] para el detalle completo.

---

## Fixes aplicados (10 abr 2026)

### m005 — `issues_and_modules`

- **fix** `needless_borrows_for_generic_args`: removido `&` en 5 llamadas `.name(&format!(...))`
- **fix** `draft_issue_assignees/cycles/labels/modules`: agregados indexes y unique constraints parciales (`WHERE deleted_at IS NULL`)

### m004 — `projects_and_states`

- **fix** `project_deploy_boards`: tabla faltante agregada (FK a `intakes` diferida a m006)

### `migration/src/main.rs`

- **fix**: auto-carga `.env` desde raíz del proyecto
- **fix**: construye `DATABASE_URL` desde variables `POSTGRES_*`

### `migration/Cargo.toml`

- **fix**: agregado `dotenvy` como dependencia

---

## Comandos de migración

```bash
# Aplicar todas las pendientes
task rust:migrations:up

# Revertir la última
task rust:migrations:down

# Estado completo
task rust:migrations:status

# Drop + re-aplicar todo (dev only — destruye datos)
task rust:migrations:fresh

# Equivalente directo con cargo
cargo run -p migration -- up
cargo run -p migration -- down
cargo run -p migration -- status
```

---

## Migraciones históricas Django más relevantes

Las migraciones clave que influyeron en el schema baseline de Rust:

| Migración Django                                 | Qué introduce                     |
| ------------------------------------------------ | --------------------------------- |
| `0001_initial`                                   | Schema base 2022                  |
| `0047_webhook_*`                                 | Webhooks y API tokens             |
| `0063_state_is_triage`                           | Estado triage                     |
| `0085_intake_*`                                  | Módulo Intake (reemplaza Inbox)   |
| `0101_description_descriptionversion`            | Versiones de descripción          |
| `0122_add_github_gitlab_integrations`            | Modelos integración GitHub/GitLab |
| `0123_add_slack_integration`                     | Modelo Slack                      |
| `0124_githubprstatemapping_usergithubconnection` | PR mapping + user OAuth           |
| `0126_gitlab_sync_models`                        | GitLab sync completo              |

> Las últimas 5 son las que materializa la migración Rust `m006`. Ver [[dominio-integraciones]] para la documentación completa de esas entidades.

---

## Próximos pasos

- [ ] Probar m006 en entorno completo con las FK de `intakes` → `project_deploy_boards`
- [ ] Crear m007 para seed estático: filas de `integrations` + `instance_configurations`
- [ ] Verificar constraint unique de `github_pr_state` en m006

---

## 🔗 Navegar

← [[fundamentos-orm]] | [[MOC]] | → [[plan-fases]] | Riesgos: [[plan-riesgos]]
