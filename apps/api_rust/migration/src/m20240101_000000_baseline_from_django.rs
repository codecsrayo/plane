// Baseline migration: schema inicial importado desde Django.
//
// STUB compilable. Para completarlo con acceso a la DB:
//
//   pg_dump --schema-only --no-owner --no-acl \
//       -d postgres://plane:plane@localhost:5432/plane \
//       > apps/api_rust/migration/sql/baseline.sql
//
// Luego reemplazar up() con:
//   db.execute_unprepared(include_str!("../sql/baseline.sql")).await?;
//
// down() en dev: DROP SCHEMA public CASCADE; CREATE SCHEMA public;
//
// Una vez aplicado, Rust es el unico dueno del schema.
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
        // TODO: reemplazar con execute_unprepared(include_str!("../sql/baseline.sql"))
        // cuando haya acceso a la DB de Django.
        //
        // Tabla placeholder para que sea-orm-cli migrate status funcione en dev.
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

#[derive(DeriveIden)]
enum Placeholder {
    Table,
    Id,
    Note,
}
