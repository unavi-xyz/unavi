use std::{sync::Arc, time::Duration};

use irpc::WithChannels;
use sqlx::Sqlite;
use time::OffsetDateTime;

use crate::{
    StoreContext,
    api::{ApiService, MAX_PIN_DURATION, PinRecord, authenticate},
    gc::FAST_GC_THRESHOLD,
    quota::{QuotaExceeded, ensure_quota_exists, reserve_bytes},
};

pub async fn pin_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let db = ctx.db.pool();
    let record_id = inner.id.to_string();

    let expires = inner
        .expires
        .min((OffsetDateTime::now_utc() + MAX_PIN_DURATION).unix_timestamp());

    let mut db_tx = db.begin().await?;

    // Ensure user has a quota entry.
    ensure_quota_exists(&mut *db_tx, &did_str).await?;

    // Check if user already has this record pinned.
    let existing_pin = sqlx::query!(
        "SELECT record_id FROM record_pins WHERE owner = ? AND record_id = ?",
        did_str,
        record_id
    )
    .fetch_optional(&mut *db_tx)
    .await?;

    if existing_pin.is_some() {
        // Already pinned - just update expiration.
        sqlx::query!(
            "UPDATE record_pins SET expires = ? WHERE owner = ? AND record_id = ?",
            expires,
            did_str,
            record_id
        )
        .execute(&mut *db_tx)
        .await?;
    } else {
        // New pin - reserve quota for existing envelopes.
        let total_size = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(size), 0) as total FROM envelopes WHERE record_id = ?",
            record_id
        )
        .fetch_one(&mut *db_tx)
        .await?;

        if total_size > 0
            && matches!(
                reserve_bytes(&mut *db_tx, &did_str, total_size).await,
                Err(QuotaExceeded)
            )
        {
            tx.send(Err("quota exceeded".into())).await?;
            return Ok(());
        }

        sqlx::query!(
            "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
            record_id,
            did_str,
            expires
        )
        .execute(&mut *db_tx)
        .await?;
    }

    // Auto-pin blob dependencies with the same expiration.
    let blob_deps: Vec<String> =
        sqlx::query_scalar("SELECT blob_hash FROM record_blob_deps WHERE record_id = ?")
            .bind(&record_id)
            .fetch_all(&mut *db_tx)
            .await?;

    for blob_hash in blob_deps {
        if let Err(e) = pin_or_extend_blob(&mut db_tx, &did_str, &blob_hash, expires).await {
            // Log but don't fail - blob might have been deleted.
            tracing::warn!(
                blob = %blob_hash,
                error = %e,
                "failed to auto-pin blob dependency"
            );
        }
    }

    db_tx.commit().await?;

    // Schedule fast GC for short-lived pins.
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ttl_secs = expires.saturating_sub(now);
    if ttl_secs >= 0 {
        let ttl = Duration::from_secs(ttl_secs.cast_unsigned());
        if ttl < FAST_GC_THRESHOLD {
            let ctx = Arc::clone(&ctx);
            let owner = did_str.clone();
            let record_id = record_id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(ttl).await;
                if let Err(e) = ctx.gc_record_pin(&owner, &record_id).await {
                    tracing::warn!(%record_id, "fast gc record pin failed: {e}");
                }
            });
        }
    }

    tx.send(Ok(())).await?;
    Ok(())
}

/// Pins or extends a blob pin for the given owner.
/// Creates a new pin if the user doesn't have one, extends if they do.
async fn pin_or_extend_blob(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    owner: &str,
    hash: &str,
    expires: i64,
) -> anyhow::Result<()> {
    // Try to extend existing pin using MAX to never shorten.
    let updated = sqlx::query!(
        "UPDATE blob_pins SET expires = MAX(expires, ?)
         WHERE owner = ? AND hash = ?",
        expires,
        owner,
        hash
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();

    if updated == 0 {
        // User doesn't have this blob pinned. Check if it exists in any pin.
        let blob_info = sqlx::query!("SELECT size FROM blob_pins WHERE hash = ? LIMIT 1", hash)
            .fetch_optional(&mut **tx)
            .await?;

        if let Some(info) = blob_info {
            // Blob exists, create pin for this user and charge quota.
            reserve_bytes(&mut **tx, owner, info.size)
                .await
                .map_err(|_| anyhow::anyhow!("quota exceeded for blob dependency"))?;

            sqlx::query!(
                "INSERT INTO blob_pins (hash, owner, expires, size) VALUES (?, ?, ?, ?)",
                hash,
                owner,
                expires,
                info.size
            )
            .execute(&mut **tx)
            .await?;
        }
        // If blob doesn't exist at all, silently skip (it may have been GC'd).
    }

    Ok(())
}
