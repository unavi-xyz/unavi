use sqlx::Sqlite;

/// Ensures a quota record exists for the user, creating one with defaults if not.
pub async fn ensure_quota_exists<'e, E>(executor: E, owner_did: &str) -> anyhow::Result<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "INSERT OR IGNORE INTO user_quotas (owner_did, quota_bytes)
         VALUES (?, ?, ?, ?)",
        owner_did,
        DEFAULT_QUOTA_BYTES,
    )
    .execute(executor)
    .await
    .context("ensure quota exists")?;

    Ok(())
}
