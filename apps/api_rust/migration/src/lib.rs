pub use sea_orm_migration::prelude::*;

mod baseline;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(baseline::Migration),
            // Futuras migraciones se agregan aquí en orden:
            // Box::new(m20240201_000001_add_..::Migration),
        ]
    }
}
