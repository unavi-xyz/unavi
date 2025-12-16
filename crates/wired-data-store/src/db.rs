use std::path::Path;

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_initial.sql"),
    include_str!("../migrations/002_sync_peers.sql"),
    include_str!("../migrations/003_sync_protocol.sql"),
    include_str!("../migrations/004_envelopes_snapshots.sql"),
];

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(db_path: &Path) -> Result<Self> {
        let url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .context("connect to sqlite database")?;

        run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    #[allow(dead_code)]
    pub async fn new_in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .context("connect to in-memory sqlite")?;

        run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    pub const fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    // Create migrations table if needed.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            applied INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .context("create migrations table")?;

    // Get current migration version.
    let current: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM _migrations")
        .fetch_one(pool)
        .await
        .context("get migration version")?;

    // Apply pending migrations.
    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = i64::try_from(i + 1).expect("migration count exceeds i64::MAX");

        if version > current {
            sqlx::query(migration)
                .execute(pool)
                .await
                .with_context(|| format!("apply migration {version}"))?;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_secs()
                .cast_signed();

            sqlx::query("INSERT INTO _migrations (id, applied) VALUES (?, ?)")
                .bind(version)
                .bind(now)
                .execute(pool)
                .await
                .context("record migration")?;
        }
    }

    Ok(())
}
