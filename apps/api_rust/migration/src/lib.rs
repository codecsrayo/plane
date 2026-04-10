pub use sea_orm_migration::prelude::*;

mod migrations::m20240101_000001_auth;
mod migrations::m20240101_000002_users_and_sessions;
mod migrations::m20240101_000003_workspaces_and_tokens;
mod migrations::m20240101_000004_projects_and_states;
mod migrations::m20240101_000005_issues_and_modules;
mod migrations::m20240101_000006_integrations_and_misc;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_auth::Migration),
            Box::new(m20240101_000002_users_and_sessions::Migration),
            Box::new(m20240101_000003_workspaces_and_tokens::Migration),
            Box::new(m20240101_000004_projects_and_states::Migration),
            Box::new(m20240101_000005_issues_and_modules::Migration),
            Box::new(m20240101_000006_integrations_and_misc::Migration),
        ]
    }
}
