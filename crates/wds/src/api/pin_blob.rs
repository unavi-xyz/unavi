use std::{sync::Arc, time::Duration};

use irpc::WithChannels;
use time::OffsetDateTime;

use crate::{
    StoreContext,
    api::{ApiService, MAX_PIN_DURATION, PinBlob, authenticate},
    gc::FAST_GC_THRESHOLD,
    quota::ensure_quota_exists,
};

pub async fn pin_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinBlob, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let db = ctx.db.pool();
    let mut db_tx = db.begin().await?;
    ensure_quota_exists(&mut *db_tx, &did_str).await?;
    db_tx.commit().await?;

    let mut db_tx = db.begin().await?;

    let hash_str = inner.hash.to_string();

    let expires = inner
        .expires
        .min((OffsetDateTime::now_utc() + MAX_PIN_DURATION).unix_timestamp());

    // Update existing blob pin, if it exists.
    let res = sqlx::query!(
        "UPDATE blob_pins SET expires = ? WHERE owner = ? AND hash = ?",
        expires,
        did_str,
        hash_str,
    )
    .execute(&mut *db_tx)
    .await?;

    if res.rows_affected() == 0 {
        // Pin didn't already exist, user must upload the blob first.
        tx.send(Err("blob not found".into())).await?;
        return Ok(());
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
            let hash = hash_str.clone();
            tokio::spawn(async move {
                tokio::time::sleep(ttl).await;
                if let Err(e) = ctx.gc_blob_pin(&owner, &hash).await {
                    tracing::warn!(%hash, "fast gc blob pin failed: {e}");
                }
            });
        }
    }

    tx.send(Ok(())).await?;
    Ok(())
}
