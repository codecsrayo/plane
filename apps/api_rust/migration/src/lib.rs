pub use sea_orm_migration::prelude::*;

mod m20240101_000000_baseline_from_django;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000000_baseline_from_django::Migration),
            // Futuras migraciones se agregan aquí en orden:
            // Box::new(m20240201_000001_add_..::Migration),
        ]
    }
}
