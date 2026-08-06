use std::{
    io,
    sync::{
        Arc,
        atomic::{
            AtomicI64,
            Ordering,
        },
    },
    time::Duration,
};

use blake3::Hash;
use futures::StreamExt;
use irpc::WithChannels;
use rusqlite::params;
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    StoreContext,
    control::{
        BlobExists,
        ControlService,
        MAX_PIN_DURATION,
        PinBlob,
        UploadBlob,
        authenticate,
    },
    error::ApiError,
    gc::FAST_GC_THRESHOLD,
    quota::{
        ensure_quota_exists,
        reserve_bytes,
    },
    tag::BlobTag,
};

const DEFAULT_BLOB_TTL: Duration = Duration::from_hours(1);

pub async fn upload_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, rx, .. }: WithChannels<UploadBlob, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    ctx.db
        .call({
            let did_str = did_str.clone();
            move |conn| ensure_quota_exists(conn, &did_str)
        })
        .await?;

    let total_bytes = Arc::new(AtomicI64::default());

    let stream = {
        let total_bytes = Arc::clone(&total_bytes);
        rx.into_stream().map(move |incoming| {
            let bytes = incoming.map_err(io::Error::other)?;
            total_bytes.fetch_add(
                i64::try_from(bytes.len()).unwrap_or(i64::MAX),
                Ordering::Release,
            );
            Ok(bytes)
        })
    };

    let Ok(temp_tag) = ctx
        .blob_store()
        .blobs()
        .add_stream(stream)
        .await
        .temp_tag()
        .await
    else {
        tx.send(Err(ApiError::Internal)).await?;
        return Ok(());
    };
    let blob_len = total_bytes.load(Ordering::Acquire);

    debug!(?blob_len, "wrote blob to store");

    let hash: Hash = temp_tag.hash().into();
    let hash_str = hash.to_string();
    let expires = (OffsetDateTime::now_utc() + DEFAULT_BLOB_TTL).unix_timestamp();

    let quota_ok = ctx
        .db
        .call_mut({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                let tx = conn.transaction()?;
                if reserve_bytes(&tx, &did_str, blob_len).is_err() {
                    return Ok(false);
                }
                tx.execute(
                    "INSERT OR IGNORE INTO blob_pins (hash, owner, expires, size) VALUES (?, ?, ?, ?)",
                    params![&hash_str, &did_str, expires, blob_len],
                )?;
                tx.commit()?;
                Ok(true)
            }
        })
        .await?;

    if !quota_ok {
        tx.send(Err(ApiError::QuotaExceeded)).await?;
        return Ok(());
    }

    // Only persist the blob tag after tracking the blob in the DB.
    let tag_name = BlobTag::new(did.clone(), hash).to_string();
    ctx.blob_store().tags().set(tag_name, temp_tag).await?;

    tx.send(Ok(hash)).await?;
    Ok(())
}

pub async fn pin_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinBlob, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();
    let hash_str = inner.hash.to_string();

    let expires = inner
        .expires
        .min((OffsetDateTime::now_utc() + MAX_PIN_DURATION).unix_timestamp());

    let rows_affected = ctx
        .db
        .call_mut({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                let tx = conn.transaction()?;
                ensure_quota_exists(&tx, &did_str)?;
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
        tx.send(Err(ApiError::BlobNotFound)).await?;
        return Ok(());
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ttl_secs = expires.saturating_sub(now);
    if ttl_secs >= 0 {
        let ttl = Duration::from_secs(ttl_secs.cast_unsigned());
        if ttl < FAST_GC_THRESHOLD {
            let ctx = Arc::clone(&ctx);
            n0_future::task::spawn(async move {
                n0_future::time::sleep(ttl).await;
                if let Err(e) = ctx.gc_blob_pin(&did_str, &hash_str).await {
                    tracing::warn!(%hash_str, "fast gc blob pin failed: {e}");
                }
            });
        }
    }

    tx.send(Ok(())).await?;
    Ok(())
}

pub async fn blob_exists(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<BlobExists, ControlService>,
) -> anyhow::Result<()> {
    let _did = authenticate!(ctx, inner, tx);

    let exists = ctx
        .blob_store()
        .blobs()
        .has(inner.hash)
        .await
        .unwrap_or(false);

    tx.send(Ok(exists)).await?;
    Ok(())
}
