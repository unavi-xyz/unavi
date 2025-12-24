use anyhow::Context;
use sqlx::Sqlite;

const DEFAULT_QUOTA_BYTES: i64 = 5 * 1024 * 1024 * 1024;

/// Ensures a quota record exists for the user, creating one with defaults if not.
pub async fn ensure_quota_exists<'e, E>(executor: E, owner: &str) -> anyhow::Result<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "INSERT OR IGNORE INTO user_quotas (owner, quota_bytes) VALUES (?, ?)",
        owner,
        DEFAULT_QUOTA_BYTES,
    )
    .execute(executor)
    .await
    .context("ensure quota exists")?;

    Ok(())
}

pub struct QuotaExceeded;

pub async fn reserve_bytes<'e, E>(
    executor: E,
    owner: &str,
    n_bytes: i64,
) -> Result<(), QuotaExceeded>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let res = sqlx::query!(
        "UPDATE user_quotas
         SET bytes_used = bytes_used + ?
         WHERE owner = ? AND bytes_used + ? <= quota_bytes",
        n_bytes,
        owner,
        n_bytes
    )
    .execute(executor)
    .await
    .map_err(|_| QuotaExceeded)?;

    if res.rows_affected() == 0 {
        return Err(QuotaExceeded);
    }

    Ok(())
}

pub async fn release_bytes<'e, E>(executor: E, owner: &str, n_bytes: i64) -> anyhow::Result<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "UPDATE user_quotas
         SET bytes_used = MAX(0, bytes_used - ?)
         WHERE owner = ?",
        n_bytes,
        owner,
    )
    .execute(executor)
    .await?;

    Ok(())
}
