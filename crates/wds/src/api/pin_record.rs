use std::{sync::Arc, time::Duration};

use irpc::WithChannels;
use rusqlite::params;
use time::OffsetDateTime;

use crate::{
    StoreContext,
    api::{ApiError, ApiService, MAX_PIN_DURATION, PinRecord, authenticate},
    gc::FAST_GC_THRESHOLD,
    quota::{ensure_quota_exists, reserve_bytes},
};

pub async fn pin_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let record_id = inner.id.to_string();

    let expires = inner
        .expires
        .min((OffsetDateTime::now_utc() + MAX_PIN_DURATION).unix_timestamp());

    let result = ctx
        .db
        .call_mut({
            let did_str = did_str.clone();
            let record_id = record_id.clone();
            move |conn| {
                let tx = conn.transaction()?;

                // Ensure user has a quota entry.
                ensure_quota_exists(&tx, &did_str)?;

                // Check if user already has this record pinned.
                let existing_pin: Option<String> = tx
                    .query_row(
                        "SELECT record_id FROM record_pins WHERE owner = ? AND record_id = ?",
                        params![&did_str, &record_id],
                        |row| row.get(0),
                    )
                    .ok();

                if existing_pin.is_some() {
                    // Already pinned - just update expiration.
                    tx.execute(
                        "UPDATE record_pins SET expires = ? WHERE owner = ? AND record_id = ?",
                        params![expires, &did_str, &record_id],
                    )?;
                } else {
                    // New pin - reserve quota for existing envelopes.
                    let total_size: i64 = tx
                        .query_row(
                            "SELECT COALESCE(SUM(size), 0) FROM envelopes WHERE record_id = ?",
                            params![&record_id],
                            |row| row.get(0),
                        )
                        .unwrap_or(0);

                    if total_size > 0 && reserve_bytes(&tx, &did_str, total_size).is_err() {
                        return Ok(Err(ApiError::QuotaExceeded));
                    }

                    tx.execute(
                        "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
                        params![&record_id, &did_str, expires],
                    )?;
                }

                // Auto-pin blob dependencies with the same expiration.
                let blob_deps: Vec<String> = {
                    let mut stmt =
                        tx.prepare("SELECT blob_hash FROM record_blob_deps WHERE record_id = ?")?;
                    stmt.query_map(params![&record_id], |row| row.get(0))?
                        .filter_map(Result::ok)
                        .collect()
                };

                for blob_hash in blob_deps {
                    if let Err(e) = pin_or_extend_blob(&tx, &did_str, &blob_hash, expires) {
                        // Log but don't fail - blob might have been deleted.
                        tracing::warn!(
                            blob = %blob_hash,
                            error = %e,
                            "failed to auto-pin blob dependency"
                        );
                    }
                }

                tx.commit()?;
                Ok(Ok(()))
            }
        })
        .await?;

    match result {
        Ok(()) => {}
        Err(e) => {
            tx.send(Err(e)).await?;
            return Ok(());
        }
    }

    // Schedule fast GC for short-lived pins.
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ttl_secs = expires.saturating_sub(now);
    if ttl_secs >= 0 {
        let ttl = Duration::from_secs(ttl_secs.cast_unsigned());
        if ttl < FAST_GC_THRESHOLD {
            let ctx = Arc::clone(&ctx);
            let owner = did_str.clone();
            let record_id = record_id.clone();
            n0_future::task::spawn(async move {
                n0_future::time::sleep(ttl).await;
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
fn pin_or_extend_blob(
    conn: &rusqlite::Connection,
    owner: &str,
    hash: &str,
    expires: i64,
) -> anyhow::Result<()> {
    // Try to extend existing pin using MAX to never shorten.
    let updated = conn.execute(
        "UPDATE blob_pins SET expires = MAX(expires, ?)
         WHERE owner = ? AND hash = ?",
        params![expires, owner, hash],
    )?;

    if updated == 0 {
        // User doesn't have this blob pinned. Check if it exists in any pin.
        let blob_size: Option<i64> = conn
            .query_row(
                "SELECT size FROM blob_pins WHERE hash = ? LIMIT 1",
                params![hash],
                |row| row.get(0),
            )
            .ok();

        if let Some(size) = blob_size {
            // Blob exists, create pin for this user and charge quota.
            reserve_bytes(conn, owner, size)
                .map_err(|_| anyhow::anyhow!("quota exceeded for blob dependency"))?;

            conn.execute(
                "INSERT INTO blob_pins (hash, owner, expires, size) VALUES (?, ?, ?, ?)",
                params![hash, owner, expires, size],
            )?;
        }
        // If blob doesn't exist at all, silently skip (it may have been GC'd).
    }

    Ok(())
}
