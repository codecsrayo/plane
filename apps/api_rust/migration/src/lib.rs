// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

mod migrations;

use migrations::m20240101_000007_seed_data;
use migrations::m20260410_000001_baseline;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260410_000001_baseline::Migration),
            Box::new(m20240101_000007_seed_data::Migration),
        ]
    }
}
