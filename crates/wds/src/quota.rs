use anyhow::Context;
use rusqlite::{Connection, params};

const DEFAULT_QUOTA_BYTES: i64 = 512 * 1024 * 1024;

/// Ensures a quota record exists for the user, creating one with defaults if not.
pub fn ensure_quota_exists(conn: &Connection, owner: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO user_quotas (owner, quota_bytes) VALUES (?, ?)",
        params![owner, DEFAULT_QUOTA_BYTES],
    )
    .context("ensure quota exists")?;
    Ok(())
}

pub struct QuotaExceeded;

pub fn reserve_bytes(conn: &Connection, owner: &str, n_bytes: i64) -> Result<(), QuotaExceeded> {
    let rows_affected = conn
        .execute(
            "UPDATE user_quotas
             SET bytes_used = bytes_used + ?
             WHERE owner = ? AND bytes_used + ? <= quota_bytes",
            params![n_bytes, owner, n_bytes],
        )
        .map_err(|_| QuotaExceeded)?;

    if rows_affected == 0 {
        return Err(QuotaExceeded);
    }

    Ok(())
}

pub fn release_bytes(conn: &Connection, owner: &str, n_bytes: i64) -> anyhow::Result<()> {
    conn.execute(
        "UPDATE user_quotas
         SET bytes_used = MAX(0, bytes_used - ?)
         WHERE owner = ?",
        params![n_bytes, owner],
    )?;
    Ok(())
}
