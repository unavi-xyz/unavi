use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

use crate::DataStore;

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
pub async fn ensure_quota_exists(pool: &Pool<Sqlite>, owner_did: &str) -> Result<()> {
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
    .execute(pool)
    .await
    .context("ensure quota exists")?;

    Ok(())
}

/// Atomically reserves bytes against the user's quota.
///
/// Returns `QuotaExceeded` if adding `bytes` would exceed the quota.
pub async fn reserve_bytes(
    pool: &Pool<Sqlite>,
    owner_did: &str,
    bytes: i64,
) -> Result<(), QuotaExceeded> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_secs()
        .cast_signed();

    // Ensure quota record exists.
    ensure_quota_exists(pool, owner_did)
        .await
        .map_err(|_| QuotaExceeded {
            used: 0,
            limit: DEFAULT_QUOTA_BYTES,
        })?;

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
    .execute(pool)
    .await
    .map_err(|_| QuotaExceeded {
        used: 0,
        limit: DEFAULT_QUOTA_BYTES,
    })?;

    if result.rows_affected() == 0 {
        // Query current state to return accurate error.
        let quota: (i64, i64) = sqlx::query_as(
            "SELECT bytes_used, quota_bytes FROM user_quotas WHERE owner_did = ?",
        )
        .bind(owner_did)
        .fetch_one(pool)
        .await
        .map_err(|_| QuotaExceeded {
            used: 0,
            limit: DEFAULT_QUOTA_BYTES,
        })?;

        return Err(QuotaExceeded {
            used: quota.0,
            limit: quota.1,
        });
    }

    Ok(())
}

/// Releases bytes from the user's quota.
///
/// Uses `MAX(0, bytes_used - amount)` to prevent negative values.
pub async fn release_bytes(pool: &Pool<Sqlite>, owner_did: &str, bytes: i64) -> Result<()> {
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
    .execute(pool)
    .await
    .context("release quota bytes")?;

    Ok(())
}

impl DataStore {
    /// Returns the current storage quota information for the owner.
    ///
    /// # Errors
    ///
    /// Returns error if quota cannot be queried from database.
    pub async fn get_storage_quota(&self) -> Result<UserQuota> {
        let owner_did = self.owner_did.to_string();

        ensure_quota_exists(self.db.pool(), &owner_did).await?;

        let row: (i64, i64) = sqlx::query_as(
            "SELECT bytes_used, quota_bytes FROM user_quotas WHERE owner_did = ?",
        )
        .bind(&owner_did)
        .fetch_one(self.db.pool())
        .await
        .context("query user quota")?;

        Ok(UserQuota {
            owner_did,
            bytes_used: row.0,
            quota_bytes: row.1,
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
