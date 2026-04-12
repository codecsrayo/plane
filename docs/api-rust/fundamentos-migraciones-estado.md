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

> Las últimas 5 son las que materializa la migración Rust `m006`. Ver [[dominio-integraciones]] para la documentación completa de esas entidades.

---

## Próximos pasos

- [ ] Verificar constraint unique de `github_pr_state` en el baseline SQL.
- [ ] Iniciar implementación de Repositories en Rust.

---

## 🔗 Navegar

← [[fundamentos-orm]] | [[MOC]] | → [[plan-fases]] | Riesgos: [[plan-riesgos]]
