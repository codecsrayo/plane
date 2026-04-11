use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    // Load .env from project root or current directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = std::path::Path::new(manifest_dir)
        .ancestors()
        .find(|p| p.join(".env").exists())
        .unwrap_or_else(|| std::path::Path::new("."));
    dotenvy::from_path(root.join(".env")).ok();

    // Build DATABASE_URL from POSTGRES_* vars if not set
    if std::env::var("DATABASE_URL").is_err() {
        let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "plane".into());
        let pass = std::env::var("POSTGRES_PASSWORD").unwrap_or_default();
        let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "plane".into());
        let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
        let port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".into());
        std::env::set_var(
            "DATABASE_URL",
            format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, db),
        );
    }

    cli::run_cli(migration::Migrator).await;
}
