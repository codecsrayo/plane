---
titulo: ORM y Migraciones
tags:
  - seaorm
  - migraciones
  - rust
relacionado:
  - "[[00-README]]"
  - "[[08-estado-migraciones]]"
---

## ORM: SeaORM

**SeaORM es el único ORM del proyecto. No se usa Diesel.**

Razones:

| Característica                      | SeaORM                                          |
| ----------------------------------- | ----------------------------------------------- |
| Async nativo Tokio                  | ✅                                              |
| Migrations en Rust                  | ✅ sea-orm-migration                            |
| Generar entities desde DB existente | ✅ `sea-orm-cli generate entity`                |
| Relaciones FK / M2M                 | ✅ has_many, belongs_to, many_to_many           |
| Soft delete integrado               | ✅ con ActiveModel hooks o `sea-orm-softdelete` |
| Con Axum                            | ✅ natural                                      |

El factor decisivo es `sea-orm-cli generate entity --database-url $DATABASE_URL`:
apunta al Postgres existente (con el schema de Django) y genera automáticamente
todos los entities Rust. Con 127 migraciones y ~50 modelos Django, esto ahorra
semanas de trabajo de transcripción manual.

---

## Migraciones: sea-orm-migration (Rust toma ownership del schema)

Django desaparece — Rust es el nuevo dueño del schema. El flujo es:

### Paso 1 — Baseline (una sola vez, al inicio de Fase 0)

Volcar el schema actual de Django como una migración baseline en SeaORM:

```bash
# 1. Generar el SQL del schema actual de Django
docker compose run --rm api python manage.py sqlmigrate ... # o pg_dump --schema-only

# 2. Crear la migración baseline en SeaORM
sea migrate generate "baseline_from_django"
# → editar el archivo generado para incluir el SQL del dump

# 3. Generar todos los entities desde la DB existente
sea generate entity \
  --database-url postgres://plane:k1rh814uw1gnCDjTinZK_df7OkAjXJ8QFcjNcFjwsME@localhost/plane \
  --output-dir src/entities \
  --with-serde both \
  --date-time-crate time
```

### Paso 2 — Todas las migraciones futuras en Rust

```bash
# Crear nueva migración
sea migrate generate "add_column_x_to_issues"

# Aplicar en dev
sea migrate up

# Revertir
sea migrate down
```

Cada migración es un archivo Rust con `up()` y `down()`:

```rust
// m20240101_000001_add_column_x_to_issues.rs
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Issues::Table)
                    .add_column(ColumnDef::new(Issues::PriorityWeight).integer().not_null().default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Issues::Table)
                    .drop_column(Issues::PriorityWeight)
                    .to_owned(),
            )
            .await
    }
}
```

### En producción (CI/CD)

```bash
# El binario Rust aplica sus propias migraciones al arrancar
# (o vía comando separado antes del deploy)
./plane-api migrate up
```

`plane-migrator` (Django) desaparece del docker-compose.

---

## Próximo paso — Fase 0

```bash
cd apps/api_rust

# Inicializar workspace Cargo
cargo init --name plane-api

# Inicializar crate de migraciones
cargo new migration --lib

# Instalar CLI de SeaORM
cargo install sea-orm-cli

# Generar entities desde la DB existente de Django
sea-orm-cli generate entity \
  --database-url "postgres://plane:plane@localhost:5432/plane" \
  --output-dir src/entities \
  --with-serde both
```

---

