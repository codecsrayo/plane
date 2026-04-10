/// Baseline migration — Schema inicial importado desde Django
///
/// Este archivo es un STUB que compila correctamente.
/// Para completarlo:
///
///   1. Con acceso a la DB de Django en ejecución:
///        pg_dump --schema-only --no-owner --no-acl \
///          -d postgres://plane:plane@localhost:5432/plane \
///          > /tmp/django_schema.sql
///
///   2. Pegar el contenido del dump en `up()` dentro de `manager.get_connection().execute_unprepared(SQL)`.
///
///   3. El `down()` debe hacer DROP de todas las tablas en orden inverso
///      respetando las FK (o simplemente `DROP SCHEMA public CASCADE; CREATE SCHEMA public;`
///      en entornos de dev).
///
/// IMPORTANTE: Una vez aplicado, Django ya no gestiona el schema.
/// Rust es el único dueño desde ese punto.
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000000_baseline_from_django"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: reemplazar con el pg_dump --schema-only de la DB Django.
        // Ejemplo de cómo ejecutar SQL raw:
        //
        //   let db = manager.get_connection();
        //   db.execute_unprepared(include_str!("../sql/baseline.sql")).await?;
        //
        // Por ahora se crea solo la tabla de control de migraciones de SeaORM
        // para que `sea-orm-cli migrate status` funcione en dev.
        manager
            .create_table(
                Table::create()
                    .table(Placeholder::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Placeholder::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Placeholder::Note).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Placeholder::Table).to_owned())
            .await
    }
}

/// Tabla placeholder — se elimina cuando se reemplaza por el baseline real.
#[derive(DeriveIden)]
enum Placeholder {
    Table,
    Id,
    Note,
}
