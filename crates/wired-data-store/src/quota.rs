use anyhow::{Context, Result};
use sqlx::Sqlite;

use crate::DataStoreView;

/// Default quota: 5 GB.
pub const DEFAULT_QUOTA_BYTES: i64 = 5 * 1024 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
#[error("quota exceeded: {used} / {limit} bytes")]
pub struct QuotaExceeded {
    pub used: i64,
    pub limit: i64,
}

/// User storage quota information.
pub struct UserQuota {
    pub owner_did: String,
    pub bytes_used: i64,
    pub quota_bytes: i64,
}

impl UserQuota {
    /// Returns remaining bytes available.
    #[must_use]
    pub const fn bytes_remaining(&self) -> i64 {
        let remaining = self.quota_bytes - self.bytes_used;
        if remaining < 0 { 0 } else { remaining }
    }
}

/// Ensures a quota record exists for the user, creating one with defaults if not.
pub async fn ensure_quota_exists<'e, E>(executor: E, owner_did: &str) -> Result<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_secs()
        .cast_signed();

    sqlx::query!(
        "INSERT OR IGNORE INTO user_quotas (owner_did, quota_bytes, created, updated)
         VALUES (?, ?, ?, ?)",
        owner_did,
        DEFAULT_QUOTA_BYTES,
        now,
        now
    )
    .execute(executor)
    .await
    .context("ensure quota exists")?;

    Ok(())
}

/// Atomically reserves bytes against the user's quota.
///
/// Returns `QuotaExceeded` if adding `bytes` would exceed the quota.
///
/// Note: This function consumes the executor. For transactions, pass `&mut *tx`.
pub async fn reserve_bytes<'e, E>(
    executor: E,
    owner_did: &str,
    bytes: i64,
) -> Result<(), QuotaExceeded>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_secs()
        .cast_signed();

    // Atomic check-and-update.
    let result = sqlx::query!(
        "UPDATE user_quotas
         SET bytes_used = bytes_used + ?, updated = ?
         WHERE owner_did = ? AND bytes_used + ? <= quota_bytes",
        bytes,
        now,
        owner_did,
        bytes
    )
    .execute(executor)
    .await
    .map_err(|_| QuotaExceeded {
        used: 0,
        limit: DEFAULT_QUOTA_BYTES,
    })?;

    if result.rows_affected() == 0 {
        // Quota exceeded (or user doesn't exist).
        return Err(QuotaExceeded {
            used: 0,
            limit: DEFAULT_QUOTA_BYTES,
        });
    }

    Ok(())
}

/// Releases bytes from the user's quota.
///
/// Uses `MAX(0, bytes_used - amount)` to prevent negative values.
pub async fn release_bytes<'e, E>(executor: E, owner_did: &str, bytes: i64) -> Result<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_secs()
        .cast_signed();

    sqlx::query!(
        "UPDATE user_quotas
         SET bytes_used = MAX(0, bytes_used - ?), updated = ?
         WHERE owner_did = ?",
        bytes,
        now,
        owner_did
    )
    .execute(executor)
    .await
    .context("release quota bytes")?;

    Ok(())
}

impl DataStoreView {
    /// Returns the current storage quota information for the owner.
    ///
    /// # Errors
    ///
    /// Returns error if quota cannot be queried from database.
    pub async fn get_storage_quota(&self) -> Result<UserQuota> {
        let owner_did = self.owner_did.to_string();

        ensure_quota_exists(self.db.pool(), &owner_did).await?;

        let row = sqlx::query!(
            "SELECT bytes_used, quota_bytes FROM user_quotas WHERE owner_did = ?",
            owner_did
        )
        .fetch_one(self.db.pool())
        .await
        .context("query user quota")?;

        Ok(UserQuota {
            owner_did,
            bytes_used: row.bytes_used,
            quota_bytes: row.quota_bytes,
        })
    }

    /// Sets a custom storage quota for the owner.
    ///
    /// # Errors
    ///
    /// Returns error if quota cannot be updated in database.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn set_storage_quota(&self, quota_bytes: i64) -> Result<()> {
        let owner_did = self.owner_did.to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        ensure_quota_exists(self.db.pool(), &owner_did).await?;

        sqlx::query!(
            "UPDATE user_quotas SET quota_bytes = ?, updated = ? WHERE owner_did = ?",
            quota_bytes,
            now,
            owner_did
        )
        .execute(self.db.pool())
        .await
        .context("set quota")?;

        Ok(())
    }
}
