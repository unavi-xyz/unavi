use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use parking_lot::Mutex;
use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_initial.sql")];

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database at the given path.
    ///
    /// # Errors
    ///
    /// Errors if the connection could not be opened or if migration fails.
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path).context("open sqlite database")?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Create a new in-memory database.
    ///
    /// # Errors
    ///
    /// Errors if the connection could not be opened or if migration fails.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open in-memory sqlite")?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Async wrapper that runs the database operation in a blocking task.
    ///
    /// # Errors
    ///
    /// Errors if the task could not be joined.
    pub async fn async_call<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock();
            f(&conn)
        })
        .await
        .context("spawn_blocking join")?
    }

    /// Async wrapper for mutable connection access (transactions).
    ///
    /// # Errors
    ///
    /// Errors if the task could not be joined.
    pub async fn async_call_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let mut conn = conn.lock();
            f(&mut conn)
        })
        .await
        .context("spawn_blocking join")?
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock();

        // Create migrations table if needed.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY,
                applied INTEGER NOT NULL
            )",
            [],
        )
        .context("create migrations table")?;

        // Get current migration version.
        let current: i64 = conn
            .query_row("SELECT COALESCE(MAX(id), 0) FROM _migrations", [], |row| {
                row.get(0)
            })
            .context("get migration version")?;

        // Apply pending migrations.
        for (i, migration) in MIGRATIONS.iter().enumerate() {
            let version = i64::try_from(i + 1).expect("migration count exceeds i64::MAX");
            if version <= current {
                continue;
            }

            conn.execute_batch(migration)
                .with_context(|| format!("apply migration {version}"))?;

            let now = i64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system time")
                    .as_secs(),
            )
            .expect("timestamp overflow");

            conn.execute(
                "INSERT INTO _migrations (id, applied) VALUES (?, ?)",
                rusqlite::params![version, now],
            )
            .context("record migration")?;
        }

        drop(conn);

        Ok(())
    }
}
