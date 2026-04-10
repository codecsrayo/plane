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
        manager
            .get_connection()
            .execute_unprepared(include_str!("../sql/baseline.sql"))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Conserva seaql_migrations para que el rollback pueda registrar el estado.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DO $$
DECLARE
    table_name text;
BEGIN
    FOR table_name IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename <> 'seaql_migrations'
    LOOP
        EXECUTE format('DROP TABLE IF EXISTS public.%I CASCADE', table_name);
    END LOOP;
END
$$;
"#,
            )
            .await?;

        Ok(())
    }
}
