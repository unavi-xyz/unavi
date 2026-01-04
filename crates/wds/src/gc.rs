use std::str::FromStr;

use blake3::Hash;
use futures::StreamExt;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{DataStore, quota::release_bytes, tag::BlobTag};

impl DataStore {
    /// Runs garbage collection on the data store.
    ///
    /// # Errors
    ///
    /// Errors if the initial database query fails or if listing blob tags fails.
    /// Individual pin and tag cleanup failures are logged and do not stop GC.
    pub async fn run_gc(&self) -> anyhow::Result<()> {
        self.gc_blob_pins().await?;
        self.gc_blob_store().await?;
        Ok(())
    }

    /// Cleans up expired pin records from the database.
    async fn gc_blob_pins(&self) -> anyhow::Result<()> {
        let db = self.ctx.db.pool();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let expired = sqlx::query!(
            "SELECT hash, owner, size FROM blob_pins
             WHERE expires IS NOT NULL AND expires < ?",
            now
        )
        .fetch_all(db)
        .await?;

        for pin in expired {
            if let Err(e) = self
                .gc_single_pin(db, &pin.owner, &pin.hash, pin.size)
                .await
            {
                tracing::warn!(owner = %pin.owner, hash = %pin.hash, "failed to gc pin: {e}");
            }
        }

        Ok(())
    }

    async fn gc_single_pin(
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
        self.ctx.blobs.tags().delete(tag.to_string()).await?;

        Ok(())
    }

    /// Cleans up orphaned blob tags that have no corresponding pin entry.
    async fn gc_blob_store(&self) -> anyhow::Result<()> {
        let db = self.ctx.db.pool();
        let mut tags = self.ctx.blobs.tags().list().await?;

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

            if !exists && let Err(e) = self.ctx.blobs.tags().delete(tag_info.name).await {
                tracing::warn!(tag = %tag_name, "failed to delete orphaned tag: {e}");
            }
        }

        Ok(())
    }
}
