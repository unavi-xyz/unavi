use std::{str::FromStr, time::Duration};

use blake3::Hash;
use futures::StreamExt;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{StoreContext, quota::release_bytes, tag::BlobTag};

/// Pins with TTL shorter than this threshold get fast GC via spawned tasks.
pub const FAST_GC_THRESHOLD: Duration = Duration::from_mins(5);

impl StoreContext {
    /// Runs garbage collection on the data store.
    ///
    /// # Errors
    ///
    /// Errors if the initial database query fails or if listing blob tags fails.
    /// Individual pin and tag cleanup failures are logged and do not stop GC.
    pub async fn run_gc(&self) -> anyhow::Result<()> {
        self.gc_record_pins().await?;
        self.gc_blob_pins().await?;
        self.gc_blob_store().await?;
        Ok(())
    }

    /// Cleans up expired record pins from the database.
    async fn gc_record_pins(&self) -> anyhow::Result<()> {
        let db = self.db.pool();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let expired = sqlx::query!(
            "SELECT record_id, owner FROM record_pins
             WHERE expires IS NOT NULL AND expires < ?",
            now
        )
        .fetch_all(db)
        .await?;

        for pin in expired {
            if let Err(e) = self.gc_record_pin(&pin.owner, &pin.record_id).await {
                tracing::warn!(owner = %pin.owner, record_id = %pin.record_id,
                    "failed to gc record pin: {e}");
            }
        }

        Ok(())
    }

    /// Cleans up expired blob pins from the database.
    async fn gc_blob_pins(&self) -> anyhow::Result<()> {
        let db = self.db.pool();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let expired = sqlx::query!(
            "SELECT hash, owner FROM blob_pins
             WHERE expires IS NOT NULL AND expires < ?",
            now
        )
        .fetch_all(db)
        .await?;

        for pin in expired {
            if let Err(e) = self.gc_blob_pin(&pin.owner, &pin.hash).await {
                tracing::warn!(owner = %pin.owner, hash = %pin.hash, "failed to gc blob pin: {e}");
            }
        }

        Ok(())
    }

    /// Cleans up orphaned blob tags that have no corresponding pin entry.
    async fn gc_blob_store(&self) -> anyhow::Result<()> {
        let db = self.db.pool();
        let mut tags = self.blobs.as_ref().as_ref().tags().list().await?;

        while let Some(tag_result) = tags.next().await {
            let tag_info = match tag_result {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("failed to read tag: {e}");
                    continue;
                }
            };

            let tag_name = tag_info.name.to_string();

            let Ok(blob_tag) = BlobTag::from_str(&tag_name) else {
                // Skip malformed tags.
                continue;
            };

            let owner = blob_tag.owner().to_string();
            let hash = blob_tag.hash().to_string();

            let exists = match sqlx::query!(
                "SELECT size FROM blob_pins WHERE owner = ? AND hash = ?",
                owner,
                hash
            )
            .fetch_optional(db)
            .await
            {
                Ok(row) => row.is_some(),
                Err(e) => {
                    tracing::warn!(tag = %tag_name, "failed to check pin existence: {e}");
                    continue;
                }
            };

            if !exists
                && let Err(e) = self
                    .blobs
                    .as_ref()
                    .as_ref()
                    .tags()
                    .delete(tag_info.name)
                    .await
            {
                tracing::warn!(tag = %tag_name, "failed to delete orphaned tag: {e}");
            }
        }

        Ok(())
    }

    /// Garbage collect a single record pin if expired.
    /// Silently succeeds if pin was extended or already removed.
    pub(crate) async fn gc_record_pin(&self, owner: &str, record_id: &str) -> anyhow::Result<()> {
        let db = self.db.pool();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        // Check if pin exists and is expired.
        let pin = sqlx::query!(
            "SELECT expires FROM record_pins WHERE owner = ? AND record_id = ?",
            owner,
            record_id
        )
        .fetch_optional(db)
        .await?;

        let Some(pin) = pin else {
            // Pin already removed.
            return Ok(());
        };

        if pin.expires.is_none_or(|e| e >= now) {
            // Pin was extended or has no expiration.
            return Ok(());
        }

        // Pin is expired - delete it and release quota for envelopes.
        let mut tx = db.begin().await?;

        let total_size: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(size), 0) as total FROM envelopes WHERE record_id = ?",
            record_id
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "DELETE FROM record_pins WHERE owner = ? AND record_id = ?",
            owner,
            record_id
        )
        .execute(&mut *tx)
        .await?;

        if total_size > 0 {
            release_bytes(&mut *tx, owner, total_size).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Garbage collect a single blob pin if expired.
    /// Silently succeeds if pin was extended or already removed.
    pub(crate) async fn gc_blob_pin(&self, owner: &str, hash: &str) -> anyhow::Result<()> {
        let db = self.db.pool();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        // Check if pin exists and is expired.
        let pin = sqlx::query!(
            "SELECT size, expires FROM blob_pins WHERE owner = ? AND hash = ?",
            owner,
            hash
        )
        .fetch_optional(db)
        .await?;

        let Some(pin) = pin else {
            // Pin already removed.
            return Ok(());
        };

        if pin.expires.is_none_or(|e| e >= now) {
            // Pin was extended or has no expiration.
            return Ok(());
        }

        // Pin is expired - delegate to gc_single_blob_pin.
        self.gc_single_blob_pin(db, owner, hash, pin.size).await
    }

    async fn gc_single_blob_pin(
        &self,
        db: &sqlx::SqlitePool,
        owner: &str,
        hash: &str,
        size: i64,
    ) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        // Use transaction to ensure pin deletion and quota decrement are atomic.
        let mut tx = db.begin().await?;

        // Check if any pinned record still depends on this blob.
        // If so, extend the blob pin to match the longest-living dependent record.
        let max_record_expires: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(p.expires) FROM record_blob_deps d
             JOIN record_pins p ON d.record_id = p.record_id
             WHERE d.blob_hash = ? AND p.owner = ?
               AND (p.expires IS NULL OR p.expires > ?)",
        )
        .bind(hash)
        .bind(owner)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        if let Some(new_expires) = max_record_expires {
            // A pinned record still needs this blob - extend the pin.
            sqlx::query!(
                "UPDATE blob_pins SET expires = ? WHERE owner = ? AND hash = ?",
                new_expires,
                owner,
                hash
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            return Ok(());
        }

        // No pinned record needs this blob - safe to delete.
        sqlx::query!(
            "DELETE FROM blob_pins WHERE owner = ? AND hash = ?",
            owner,
            hash
        )
        .execute(&mut *tx)
        .await?;

        release_bytes(&mut *tx, owner, size).await?;

        tx.commit().await?;

        // Delete blob tag after DB commit.
        // If this fails, `gc_blob_store` will clean it up later.
        let owner_did = Did::from_str(owner)?;
        let hash_parsed = Hash::from_str(hash)?;
        let tag = BlobTag::new(owner_did, hash_parsed);
        self.blobs
            .as_ref()
            .as_ref()
            .tags()
            .delete(tag.to_string())
            .await?;

        Ok(())
    }
}
