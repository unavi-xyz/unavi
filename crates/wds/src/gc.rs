use std::{
    str::FromStr,
    time::Duration,
};

use blake3::Hash;
use rusqlite::params;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{
    StoreContext,
    quota::release_bytes,
    tag::BlobTag,
};

/// Pins with TTL shorter than this threshold get fast GC via spawned tasks.
pub const FAST_GC_THRESHOLD: Duration = Duration::from_mins(5);

impl StoreContext {
    /// Runs garbage collection on the data store.
    ///
    /// Content referenced by hosted docs is protected by iroh-docs' own store
    /// tags; this pass only reclaims explicit blob pins that have expired.
    pub async fn run_gc(&self) -> anyhow::Result<()> {
        self.gc_sessions().await;
        self.gc_blob_pins().await
    }

    /// Drops sessions past their expiry, so the table tracks live sessions
    /// rather than every session the process has ever issued.
    async fn gc_sessions(&self) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.connections.retain_async(|_, c| c.expires > now).await;
    }

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
                Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
            })
            .await?;

        for (hash, owner) in expired {
            if let Err(e) = self.gc_blob_pin(&owner, &hash).await {
                tracing::warn!(owner = %owner, hash = %hash, "failed to gc blob pin: {e}");
            }
        }

        Ok(())
    }

    /// Garbage collect a single blob pin if expired.
    /// Silently succeeds if the pin was extended or already removed.
    pub(crate) async fn gc_blob_pin(&self, owner: &str, hash: &str) -> anyhow::Result<()> {
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

                    let info: Option<(i64, i64)> = tx
                        .query_row(
                            "SELECT size, expires FROM blob_pins WHERE owner = ? AND hash = ?",
                            params![&owner, &hash],
                            |row| Ok((row.get(0)?, row.get(1)?)),
                        )
                        .ok();

                    let Some((size, expires)) = info else {
                        return Ok(false);
                    };
                    if expires >= now {
                        return Ok(false);
                    }

                    tx.execute(
                        "DELETE FROM blob_pins WHERE owner = ? AND hash = ?",
                        params![&owner, &hash],
                    )?;
                    release_bytes(&tx, &owner, size)?;
                    tx.commit()?;
                    Ok(true)
                }
            })
            .await?;

        if deleted {
            let tag = BlobTag::new(Did::from_str(&owner_str)?, Hash::from_str(&hash_str)?);
            self.blob_store().tags().delete(tag.to_string()).await?;
        }

        Ok(())
    }
}
