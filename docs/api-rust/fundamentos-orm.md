---
titulo: ORM y Migraciones — SeaORM
aliases:
  - orm
  - seaorm
  - migraciones-seaorm
tags:
  - seaorm
  - migraciones
  - rust
  - fundamentos
relacionado:
  - "[[MOC]]"
  - "[[fundamentos-migraciones-estado]]"
  - "[[fundamentos-soft-delete]]"
  - "[[plan-fases]]"
  - "[[ref-estructura-archivos]]"
estado: activo
---

# ORM y Migraciones — SeaORM

---

## ¿Por qué SeaORM y no Diesel?

**SeaORM es el único ORM del proyecto. No se usa Diesel.**

| Característica | SeaORM | Diesel |
|----------------|--------|--------|
| Async nativo Tokio | ✅ | ❌ (síncrono) |
| Migrations en Rust | ✅ `sea-orm-migration` | ✅ diesel_migrations |
| Generar entities desde DB existente | ✅ `sea-orm-cli generate entity` | ❌ requiere esquema manual |
| Relaciones FK / M2M | ✅ has_many, belongs_to, many_to_many | ✅ |
| Soft delete integrado | ✅ con hooks o trait custom | ⚠️ manual |
| Con Axum | ✅ natural | ✅ con adaptadores |

**El factor decisivo:** `sea-orm-cli generate entity --database-url $DATABASE_URL` apunta al Postgres existente (con el schema de Django) y genera automáticamente todos los entities Rust. Con 126 migraciones y ~50 modelos Django, esto ahorra semanas de trabajo de transcripción manual.

---

## Estrategia de migración — Rust toma ownership del schema

Django desaparece — Rust es el nuevo dueño del schema. El proceso es de una sola vez:

### Paso 1 — Baseline (Fase 0)

Volcar el schema actual de Django como una única migración baseline en SeaORM:

```bash
# 1. Generar el SQL del schema actual de Django
pg_dump --schema-only -d plane > baseline.sql

# 2. Crear la migración baseline en SeaORM
sea migrate generate "baseline_from_django"
# → editar el archivo generado para incluir el SQL del dump

# 3. Generar todos los entities desde la DB existente
sea-orm-cli generate entity \
  --database-url "postgres://plane:plane@localhost:5432/plane" \
  --output-dir src/entities \
  --with-serde both \
  --date-time-crate time
```

> [!NOTE] 122 entidades generadas
> La DB actual de Django produce 122 entities Rust. Están en `src/entities/` y **NO se editan a mano** — se regeneran con el CLI si cambia el schema.

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
// migration/src/migrations/m20240101_000001_add_column.rs
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Issues::Table)
                    .add_column(
                        ColumnDef::new(Issues::PriorityWeight)
                            .integer()
                            .not_null()
                            .default(0)
                    )
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
./plane-api migrate up
```

`plane-migrator` (Django) desaparece del docker-compose al llegar a [[plan-fases#Fase 5]].

---

## Estructura del crate de migraciones

```
migration/
├── Cargo.toml
└── src/
    ├── lib.rs            ← Migrator con todas las migraciones en orden
    ├── main.rs           ← auto-carga .env desde raíz del proyecto
    └── migrations/
        ├── mod.rs
        ├── m20260410_000001_baseline.rs        ← schema completo desde Django ✅
        ├── m20240101_000006_integrations.rs    ← integraciones 🔄
        └── m20240101_000007_seed_data.rs       ← integrations + instance_configs ✅
```

> [!IMPORTANT] `migration/src/main.rs` auto-carga `.env`
> Construye `DATABASE_URL` desde variables `POSTGRES_*`. Requiere `dotenvy` como dependencia del crate de migración.

---

## Comandos rápidos

```bash
# Desde apps/api_rust/

# Aplicar todas las pendientes
task rust:migrations:up

# Revertir la última
task rust:migrations:down

# Estado completo
task rust:migrations:status

# Drop + re-aplicar todo (dev only)
task rust:migrations:fresh

# Regenerar entities desde la DB
sea-orm-cli generate entity \
  --database-url "postgres://plane:plane@localhost:5432/plane" \
  --output-dir src/entities \
  --with-serde both
```

---

## 🔗 Navegar

← [[MOC]] | Estado actual de migraciones: [[fundamentos-migraciones-estado]] | Soft delete: [[fundamentos-soft-delete]] | Estructura: [[ref-estructura-archivos]]
