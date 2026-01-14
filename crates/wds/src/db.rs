use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use rusqlite::Connection;
use time::OffsetDateTime;
use tokio::sync::Mutex;

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
        run_migrations(&conn)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        Ok(db)
    }

    /// Create a new in-memory database.
    ///
    /// # Errors
    ///
    /// Errors if the connection could not be opened or if migration fails.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open in-memory sqlite")?;
        run_migrations(&conn)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        Ok(db)
    }

    /// Async wrapper that runs the database operation in a blocking task.
    ///
    /// # Errors
    ///
    /// Errors if the task could not be joined.
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let handle = n0_future::task::spawn(async move {
            let conn = conn.lock().await;
            f(&conn)
        });
        handle.await?
    }

    /// Async wrapper for mutable connection access (transactions).
    ///
    /// # Errors
    ///
    /// Errors if the task could not be joined.
    pub async fn call_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let handle = n0_future::task::spawn(async move {
            let mut conn = conn.lock().await;
            f(&mut conn)
        });
        handle.await?
    }
}

fn run_migrations(conn: &Connection) -> Result<()> {
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

        let now = OffsetDateTime::now_utc().unix_timestamp();

        conn.execute(
            "INSERT INTO _migrations (id, applied) VALUES (?, ?)",
            rusqlite::params![version, now],
        )
        .context("record migration")?;
    }

    Ok(())
}
