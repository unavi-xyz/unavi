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
