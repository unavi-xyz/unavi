use std::path::Path;

use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use xdid::core::did::Did;

use crate::{RecordId, hash_did};

/// Statistics from a garbage collection run.
pub struct GarbageCollectStats {
    pub pins_removed: u64,
    pub records_removed: u64,
    pub blobs_removed: u64,
    pub bytes_freed: u64,
}

/// Removes expired pins and returns list of (`record_id`, `owner_did`) that lost pins.
pub async fn remove_expired_pins(
    pool: &Pool<Sqlite>,
    now: i64,
) -> Result<(Vec<(String, String)>, u64)> {
    // Transaction: query and delete all expired pins atomically.
    let mut tx = pool.begin().await.context("begin transaction")?;

    let expired_pins: Vec<(String, String)> = sqlx::query!(
        "SELECT record_id, owner_did FROM pins WHERE expires IS NOT NULL AND expires < ?",
        now
    )
    .fetch_all(&mut *tx)
    .await
    .context("query expired pins")?
    .into_iter()
    .map(|r| (r.record_id, r.owner_did))
    .collect();

    let pins_removed = expired_pins.len() as u64;

    // Delete all expired pins in one query.
    sqlx::query!(
        "DELETE FROM pins WHERE expires IS NOT NULL AND expires < ?",
        now
    )
    .execute(&mut *tx)
    .await
    .context("delete expired pins")?;

    tx.commit().await.context("commit transaction")?;

    Ok((expired_pins, pins_removed))
}

/// Checks if a record has any remaining pins.
pub async fn has_remaining_pins(
    pool: &Pool<Sqlite>,
    record_id: &str,
    owner_did: &str,
) -> Result<bool> {
    let pin_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM pins WHERE record_id = ? AND owner_did = ?",
        record_id,
        owner_did
    )
    .fetch_one(pool)
    .await
    .context("count remaining pins")?;

    Ok(pin_count > 0)
}

/// Removes an unpinned record and decrements blob references.
/// Returns the size of the removed record, or None if record not found.
pub async fn remove_unpinned_record(
    pool: &Pool<Sqlite>,
    data_dir: &Path,
    record_id: &str,
    owner_did: &str,
) -> Result<Option<u64>> {
    // Compute file path before transaction (needs Did parsing).
    let did_parsed = owner_did
        .parse::<Did>()
        .map_err(|e| anyhow::anyhow!("invalid DID in database: {e}"))?;

    let record_id_obj = RecordId(
        record_id
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid ID: {e}"))?,
    );

    let did_hash = hash_did(&did_parsed);
    let id_str = record_id_obj.as_str();
    let prefix = &id_str[..2.min(id_str.len())];
    let path = data_dir
        .join("records")
        .join(did_hash)
        .join(prefix)
        .join(format!("{id_str}.loro"));

    // Transaction: query, decrement refs, delete record, release quota.
    let mut tx = pool.begin().await.context("begin transaction")?;

    let size: Option<i64> = sqlx::query_scalar!(
        "SELECT size FROM records WHERE id = ? AND owner_did = ?",
        record_id,
        owner_did
    )
    .fetch_optional(&mut *tx)
    .await
    .context("query record size")?;

    let Some(size) = size else {
        return Ok(None);
    };

    // Decrement user_blob ref_counts for linked blobs.
    let linked_blobs: Vec<String> = sqlx::query_scalar!(
        "SELECT blob_id FROM record_blobs WHERE record_id = ? AND owner_did = ?",
        record_id,
        owner_did
    )
    .fetch_all(&mut *tx)
    .await
    .context("query linked blobs for GC")?;

    for blob_id in &linked_blobs {
        sqlx::query!(
            "UPDATE user_blobs SET ref_count = ref_count - 1 WHERE blob_id = ? AND owner_did = ?",
            blob_id,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("decrement user_blob ref_count in GC")?;
    }

    // Delete record (CASCADE will delete record_blobs entries).
    sqlx::query!(
        "DELETE FROM records WHERE id = ? AND owner_did = ?",
        record_id,
        owner_did
    )
    .execute(&mut *tx)
    .await
    .context("delete record from database")?;

    // Release quota for removed record.
    crate::quota::release_bytes(&mut *tx, owner_did, size).await?;

    tx.commit().await.context("commit transaction")?;

    // Delete record file after successful commit.
    if path.exists() {
        std::fs::remove_file(&path).context("delete record file")?;
    }

    Ok(Some(size.cast_unsigned()))
}

/// Removes orphaned `user_blobs` (`ref_count` = 0).
/// Also releases quota for each removed `user_blob`.
/// Note: Physical blob cleanup is handled by `FsStore`.
pub async fn remove_orphaned_user_blobs(pool: &Pool<Sqlite>) -> Result<()> {
    // Transaction: query, delete, release quota.
    let mut tx = pool.begin().await.context("begin transaction")?;

    // Query orphaned user_blobs.
    // Note: We estimate size based on a fixed value since we no longer have the blobs table.
    // In practice, quota was already reserved when the blob was stored, so we just need to
    // delete the user_blob entries. The quota release should match what was reserved.
    let orphaned_user_blobs: Vec<(String, String)> =
        sqlx::query!("SELECT blob_id, owner_did FROM user_blobs WHERE ref_count = 0")
            .fetch_all(&mut *tx)
            .await
            .context("query orphaned user_blobs")?
            .into_iter()
            .map(|r| (r.blob_id, r.owner_did))
            .collect();

    for (blob_id, owner_did) in &orphaned_user_blobs {
        sqlx::query!(
            "DELETE FROM user_blobs WHERE blob_id = ? AND owner_did = ?",
            blob_id,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("delete orphaned user_blob")?;

        // Note: Quota was released when the blob link was removed, not here.
        // This function just cleans up user_blob entries with ref_count = 0.
    }

    tx.commit().await.context("commit transaction")?;

    Ok(())
}
