---
titulo: Estado de migraciones SeaORM
aliases:
  - estado-migraciones
  - migraciones-baseline
tags:
  - migraciones
  - seaorm
  - seguimiento
  - fundamentos
  - implementado
relacionado:
  - "[[MOC]]"
  - "[[fundamentos-orm]]"
  - "[[plan-fases]]"
  - "[[plan-riesgos]]"
estado: activo
---

# Estado de migraciones SeaORM

> [!SUCCESS] Actualizado: 12 abril 2026
> m001–m006 consolidadas en Baseline SQL. m007 (seeds) aplicada.

---

## Seguimiento de archivos de migración

| Archivo                      | Contenido                                                                   | Estado |
| ---------------------------- | --------------------------------------------------------------------------- | ------ |
| `m20260410_000001_baseline`  | Schema completo (Django legacy) cargado vía `sql/baseline.sql`              | ✅     |
| `m20240101_000007_seed_data` | Filas estáticas para `integrations` e `instance_configurations`. Ver [[fundamentos-siembra-datos]]. | ✅     |

> [!NOTE] Consolidación de migraciones
> Para simplificar el arranque del proyecto Rust, las migraciones incrementales `m001` a `m006` que existían en la fase de diseño se han consolidado en un único script SQL de **baseline**. Esto garantiza que el schema sea idéntico al de Django sin el overhead de 120+ archivos de migración.

---

## Fixes aplicados (10 abr 2026)

### Baseline SQL

- **fix** `indexes`: agregados indexes y unique constraints parciales (`WHERE deleted_at IS NULL`) para todas las entidades core.
- **fix** `project_deploy_boards`: tabla integrada correctamente con FKs a `intakes`.

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

> Las últimas 5 son las que materializa la migración Rust `m20260410_000001_baseline` (Baseline SQL). Ver [[dominio-integraciones]] para la documentación completa de esas entidades.

---

## Índices — tablas de integración

> Todos presentes en `sql/baseline.sql`. Ninguna tabla de integración tiene `WHERE deleted_at IS NULL` porque estos modelos **no usan soft-delete** (sus filas se eliminan físicamente al desinstalar).

### `workspace_integrations`

| Índice | Tipo | Columnas |
|--------|------|----------|
| `workspace_integrations_workspace_id_integration_fa041c22_uniq` | UNIQUE | `(workspace_id, integration_id)` — garantiza 1 fila por provider por workspace |
| `workspace_integrations_workspace_id_27ebeb6b` | btree | `workspace_id` |
| `workspace_integrations_integration_id_6cb0aace` | btree | `integration_id` |
| `workspace_integrations_actor_id_21619aa1` | btree | `actor_id` |
| `workspace_integrations_api_token_id_bdb1759b` | btree | `api_token_id` |
| `workspace_integrations_created_by_id_37639c73` | btree | `created_by_id` |
| `workspace_integrations_updated_by_id_fce01dcb` | btree | `updated_by_id` |

### `db_githubprstatemapping`

| Índice | Tipo | Columnas |
|--------|------|----------|
| `db_githubprstatemapping_workspace_integration_id_8a615667_uniq` | UNIQUE | `(workspace_integration_id, project_id, github_pr_state)` — 1 mapping por estado de PR por proyecto |
| `db_githubprstatemapping_workspace_integration_id_2eab555c` | btree | `workspace_integration_id` |
| `db_githubprstatemapping_project_id_361c0ac2` | btree | `project_id` |
| `db_githubprstatemapping_state_id_d2959076` | btree | `state_id` |
| `db_githubprstatemapping_created_by_id_381aa92b` | btree | `created_by_id` |
| `db_githubprstatemapping_updated_by_id_3330c9ff` | btree | `updated_by_id` |

### `github_repositories` / `github_repository_syncs`

| Índice | Tipo | Columnas |
|--------|------|----------|
| `github_repository_syncs_project_id_repository_id_0f3705e6_uniq` | UNIQUE | `(project_id, repository_id)` — 1 sync por repo por proyecto |
| `github_repository_syncs_repository_id_key` | UNIQUE | `repository_id` — OneToOne con `github_repositories` |
| `github_issue_syncs_repository_sync_id_issue_id_4b34427e_uniq` | UNIQUE | `(repository_sync_id, issue_id)` |
| `github_comment_syncs_issue_sync_id_comment_id_38c82e7b_uniq` | UNIQUE | `(issue_sync_id, comment_id)` |
| `github_repository_syncs_workspace_integration_id_62858398` | btree | `workspace_integration_id` |
| `github_repository_syncs_project_id_e7e8291e` | btree | `project_id` |
| `github_repository_syncs_actor_id_1fa689fe` | btree | `actor_id` |
| `github_repository_syncs_label_id_eb1e9bd7` | btree | `label_id` |
| `github_repositories_project_id_65c546bb` | btree | `project_id` |
| `github_issue_syncs_issue_id_450cb083` | btree | `issue_id` |
| `github_issue_syncs_repository_sync_id_ba0d4de4` | btree | `repository_sync_id` |
| `github_comment_syncs_comment_id_6feec6d1` | btree | `comment_id` |
| `github_comment_syncs_issue_sync_id_5e738eb5` | btree | `issue_sync_id` |

> ⚠️ **Gap crítico:** `github_repositories` no tiene un índice en `(workspace_id, repo_id_github)`. El fix de soft-delete resurrection (commit `e030f50`) necesita buscar filas por `github_id` externo — sin ese índice la búsqueda es full scan. Considerar agregar en migración futura de Rust.

### `gitlab_repositories` / `gitlab_repository_syncs`

| Índice | Tipo | Columnas |
|--------|------|----------|
| `gitlab_repository_syncs_project_id_repository_id_13a57d00_uniq` | UNIQUE | `(project_id, repository_id)` |
| `gitlab_repository_syncs_repository_id_key` | UNIQUE | `repository_id` — OneToOne con `gitlab_repositories` |
| `gitlab_issue_syncs_repository_sync_id_issue_id_14bdcc2c_uniq` | UNIQUE | `(repository_sync_id, issue_id)` |
| `gitlab_comment_syncs_issue_sync_id_comment_id_61435f60_uniq` | UNIQUE | `(issue_sync_id, comment_id)` |
| `gitlab_repository_syncs_workspace_integration_id_4b878644` | btree | `workspace_integration_id` |
| `gitlab_repository_syncs_project_id_9d61576c` | btree | `project_id` |

### `slack_project_syncs`

| Índice | Tipo | Columnas |
|--------|------|----------|
| `slack_project_syncs_team_id_project_id_50a144a7_uniq` | UNIQUE | `(team_id, project_id)` — 1 sync por workspace de Slack por proyecto |
| `slack_project_syncs_workspace_integration_id_d89c9b40` | btree | `workspace_integration_id` |
| `slack_project_syncs_project_id_016dc792` | btree | `project_id` |

### `user_github_connections`

| Índice | Tipo | Columnas |
|--------|------|----------|
| `user_github_connections_user_id_key` | UNIQUE | `user_id` — 1 conexión GitHub personal por usuario |
| `user_github_connections_created_by_id_99678dc5` | btree | `created_by_id` |
| `user_github_connections_updated_by_id_de42cb21` | btree | `updated_by_id` |

### `integrations` (tabla de referencia estática)

| Índice | Tipo | Columnas |
|--------|------|----------|
| `integrations_provider_6537a106_like` | btree varchar_pattern_ops | `provider` — búsquedas LIKE por nombre de provider |
| `integrations_created_by_id_0b6edd52` | btree | `created_by_id` |
| `integrations_updated_by_id_d6d00d15` | btree | `updated_by_id` |

---

## Próximos pasos

- [x] ~~Verificar constraint unique de `github_pr_state` en el baseline SQL.~~ — confirmado: `(workspace_integration_id, project_id, github_pr_state)` UNIQUE
- [ ] Agregar índice `(workspace_id, github_id)` en `github_repositories` para el lookup de soft-delete resurrection (ver gap documentado en sección de índices arriba).
- [ ] Iniciar implementación de Repositories en Rust.

---

## 🔗 Navegar

← [[fundamentos-orm]] | [[MOC]] | → [[plan-fases]] | Riesgos: [[plan-riesgos]]

---

_`docs/api-rust/fundamentos-migraciones-estado.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
