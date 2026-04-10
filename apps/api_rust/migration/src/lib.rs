pub use sea_orm_migration::prelude::*;

mod m20240101_000000_baseline;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000000_baseline::Migration),
            // Futuras migraciones se agregan aquí en orden:
            // Box::new(m20240201_000001_add_..::Migration),
        ]
    }
}
