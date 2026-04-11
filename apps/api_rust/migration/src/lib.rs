pub use sea_orm_migration::prelude::*;

mod migrations;

use migrations::m20240101_000001_auth_django;
use migrations::m20240101_000002_users_and_sessions;
use migrations::m20240101_000003_workspaces_and_tokens;
use migrations::m20240101_000004_projects_and_states;
use migrations::m20240101_000005_issues_and_modules;
use migrations::m20240101_000006_integrations_and_misc;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_auth_django::Migration),
            Box::new(m20240101_000002_users_and_sessions::Migration),
            Box::new(m20240101_000003_workspaces_and_tokens::Migration),
            Box::new(m20240101_000004_projects_and_states::Migration),
            Box::new(m20240101_000005_issues_and_modules::Migration),
            Box::new(m20240101_000006_integrations_and_misc::Migration),
        ]
    }
}
