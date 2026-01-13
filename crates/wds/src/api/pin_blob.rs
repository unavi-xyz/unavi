use std::{sync::Arc, time::Duration};

use irpc::WithChannels;
use rusqlite::params;
use time::OffsetDateTime;

use crate::{
    StoreContext,
    api::{ApiError, ApiService, MAX_PIN_DURATION, PinBlob, authenticate},
    gc::FAST_GC_THRESHOLD,
    quota::ensure_quota_exists,
};

pub async fn pin_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinBlob, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let hash_str = inner.hash.to_string();

    let expires = inner
        .expires
        .min((OffsetDateTime::now_utc() + MAX_PIN_DURATION).unix_timestamp());

    let rows_affected = ctx
        .db
        .async_call_mut({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                let tx = conn.transaction()?;

                ensure_quota_exists(&tx, &did_str)?;

                // Update existing blob pin, if it exists.
                let rows = tx.execute(
                    "UPDATE blob_pins SET expires = ? WHERE owner = ? AND hash = ?",
                    params![expires, &did_str, &hash_str],
                )?;

                tx.commit()?;
                Ok(rows)
            }
        })
        .await?;

    if rows_affected == 0 {
        // Pin didn't already exist, user must upload the blob first.
        tx.send(Err(ApiError::BlobNotFound)).await?;
        return Ok(());
    }

    // Schedule fast GC for short-lived pins.
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ttl_secs = expires.saturating_sub(now);
    if ttl_secs >= 0 {
        let ttl = Duration::from_secs(ttl_secs.cast_unsigned());
        if ttl < FAST_GC_THRESHOLD {
            let ctx = Arc::clone(&ctx);
            let owner = did_str.clone();
            let hash = hash_str.clone();
            unavi_wasm_compat::spawn(async move {
                unavi_wasm_compat::sleep(ttl).await;
                if let Err(e) = ctx.gc_blob_pin(&owner, &hash).await {
                    tracing::warn!(%hash, "fast gc blob pin failed: {e}");
                }
            });
        }
    }

    tx.send(Ok(())).await?;
    Ok(())
}
