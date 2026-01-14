use std::{str::FromStr, time::Duration};

use blake3::Hash;
use futures::StreamExt;
use rusqlite::params;
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
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let expired = self
            .db
            .call(move |conn| {
                let mut stmt =
                    conn.prepare("SELECT record_id, owner FROM record_pins WHERE expires < ?")?;
                let rows = stmt.query_map(params![now], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                let result: Vec<(String, String)> = rows.filter_map(Result::ok).collect();
                Ok(result)
            })
            .await?;

        for (record_id, owner) in expired {
            if let Err(e) = self.gc_record_pin(&owner, &record_id).await {
                tracing::warn!(owner = %owner, record_id = %record_id,
                    "failed to gc record pin: {e}");
            }
        }

        Ok(())
    }

    /// Cleans up expired blob pins from the database.
    async fn gc_blob_pins(&self) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let expired = self
            .db
            .call(move |conn| {
                let mut stmt =
                    conn.prepare("SELECT hash, owner FROM blob_pins WHERE expires < ?")?;
                let rows = stmt.query_map(params![now], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                let result: Vec<(String, String)> = rows.filter_map(Result::ok).collect();
                Ok(result)
            })
            .await?;

        for (hash, owner) in expired {
            if let Err(e) = self.gc_blob_pin(&owner, &hash).await {
                tracing::warn!(owner = %owner, hash = %hash, "failed to gc blob pin: {e}");
            }
        }

        Ok(())
    }

    /// Cleans up orphaned blob tags that have no corresponding pin entry.
    async fn gc_blob_store(&self) -> anyhow::Result<()> {
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

            let exists = match self
                .db
                .call({
                    let owner = owner.clone();
                    let hash = hash.clone();
                    move |conn| {
                        let result: Option<i64> = conn
                            .query_row(
                                "SELECT size FROM blob_pins WHERE owner = ? AND hash = ?",
                                params![&owner, &hash],
                                |row| row.get(0),
                            )
                            .ok();
                        Ok(result.is_some())
                    }
                })
                .await
            {
                Ok(exists) => exists,
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
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let owner = owner.to_string();
        let record_id = record_id.to_string();

        self.db
            .call_mut(move |conn| {
                // Check if pin exists and is expired.
                let pin_expires: Option<i64> = conn
                    .query_row(
                        "SELECT expires FROM record_pins WHERE owner = ? AND record_id = ?",
                        params![&owner, &record_id],
                        |row| row.get(0),
                    )
                    .ok();

                let Some(expires) = pin_expires else {
                    // Pin already removed.
                    return Ok(());
                };

                if expires >= now {
                    // Pin was extended.
                    return Ok(());
                }

                // Pin is expired - delete it and release quota for envelopes.
                let tx = conn.transaction()?;

                let total_size: i64 = tx
                    .query_row(
                        "SELECT COALESCE(SUM(size), 0) FROM envelopes WHERE record_id = ?",
                        params![&record_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                tx.execute(
                    "DELETE FROM record_pins WHERE owner = ? AND record_id = ?",
                    params![&owner, &record_id],
                )?;

                if total_size > 0 {
                    release_bytes(&tx, &owner, total_size)?;
                }

                // Check if any pins remain for this record.
                let remaining_pins: i64 = tx
                    .query_row(
                        "SELECT COUNT(*) FROM record_pins WHERE record_id = ?",
                        params![&record_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if remaining_pins == 0 {
                    // No pins remain - delete record and all related data.
                    tx.execute(
                        "DELETE FROM envelopes WHERE record_id = ?",
                        params![&record_id],
                    )?;
                    tx.execute(
                        "DELETE FROM record_blob_deps WHERE record_id = ?",
                        params![&record_id],
                    )?;
                    tx.execute(
                        "DELETE FROM record_schemas WHERE record_id = ?",
                        params![&record_id],
                    )?;
                    tx.execute(
                        "DELETE FROM record_acl_read WHERE record_id = ?",
                        params![&record_id],
                    )?;
                    tx.execute("DELETE FROM records WHERE id = ?", params![&record_id])?;
                }

                tx.commit()?;
                Ok(())
            })
            .await
    }

    /// Garbage collect a single blob pin if expired.
    /// Silently succeeds if pin was extended or already removed.
    pub(crate) async fn gc_blob_pin(&self, owner: &str, hash: &str) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let owner_str = owner.to_string();
        let hash_str = hash.to_string();

        // Check if pin exists and is expired, get size if so.
        let pin_info = self
            .db
            .call({
                let owner = owner_str.clone();
                let hash = hash_str.clone();
                move |conn| {
                    let result: Option<(i64, i64)> = conn
                        .query_row(
                            "SELECT size, expires FROM blob_pins WHERE owner = ? AND hash = ?",
                            params![&owner, &hash],
                            |row| Ok((row.get(0)?, row.get(1)?)),
                        )
                        .ok();
                    Ok(result)
                }
            })
            .await?;

        let Some((size, expires)) = pin_info else {
            // Pin already removed.
            return Ok(());
        };

        if expires >= now {
            // Pin was extended.
            return Ok(());
        }

        // Pin is expired - delegate to gc_single_blob_pin.
        self.gc_single_blob_pin(&owner_str, &hash_str, size).await
    }

    async fn gc_single_blob_pin(&self, owner: &str, hash: &str, size: i64) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let owner_str = owner.to_string();
        let hash_str = hash.to_string();

        let deleted = self
            .db
            .call_mut({
                let owner = owner_str.clone();
                let hash = hash_str.clone();
                move |conn| {
                    let tx = conn.transaction()?;

                    // Check if any pinned record still depends on this blob.
                    // If so, extend the blob pin to match the longest-living dependent record.
                    let max_record_expires: Option<i64> = tx
                        .query_row(
                            "SELECT MAX(p.expires) FROM record_blob_deps d
                             JOIN record_pins p ON d.record_id = p.record_id
                             WHERE d.blob_hash = ? AND p.owner = ? AND p.expires > ?",
                            params![&hash, &owner, now],
                            |row| row.get(0),
                        )
                        .ok()
                        .flatten();

                    if let Some(new_expires) = max_record_expires {
                        // A pinned record still needs this blob - extend the pin.
                        tx.execute(
                            "UPDATE blob_pins SET expires = ? WHERE owner = ? AND hash = ?",
                            params![new_expires, &owner, &hash],
                        )?;

                        tx.commit()?;
                        return Ok(false); // Not deleted.
                    }

                    // No pinned record needs this blob - safe to delete.
                    tx.execute(
                        "DELETE FROM blob_pins WHERE owner = ? AND hash = ?",
                        params![&owner, &hash],
                    )?;

                    release_bytes(&tx, &owner, size)?;

                    tx.commit()?;
                    Ok(true) // Deleted.
                }
            })
            .await?;

        if deleted {
            // Delete blob tag after DB commit.
            // If this fails, `gc_blob_store` will clean it up later.
            let owner_did = Did::from_str(&owner_str)?;
            let hash_parsed = Hash::from_str(&hash_str)?;
            let tag = BlobTag::new(owner_did, hash_parsed);
            self.blobs
                .as_ref()
                .as_ref()
                .tags()
                .delete(tag.to_string())
                .await?;
        }

        Ok(())
    }
}
