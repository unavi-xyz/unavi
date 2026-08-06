use std::{
    future::ready,
    io,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
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

/// Ceiling on a single upload, independent of quota. Nothing this store serves
/// is legitimately larger, and without it one request can outrun the quota
/// check by however much disk is left.
const MAX_UPLOAD_BYTES: i64 = 256 * 1024 * 1024;

pub async fn upload_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, rx, .. }: WithChannels<UploadBlob, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let headroom = ctx
        .db
        .call({
            let did_str = did_str.clone();
            move |conn| {
                ensure_quota_exists(conn, &did_str)?;
                let remaining: i64 = conn.query_row(
                    "SELECT quota_bytes - bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )?;
                Ok(remaining.clamp(0, MAX_UPLOAD_BYTES))
            }
        })
        .await?;

    let total_bytes = Arc::new(AtomicI64::default());
    let aborted = Arc::new(AtomicBool::new(false));

    // Enforced as the stream is consumed, not after: `add_stream` will happily
    // write an unbounded body to disk before any post-hoc check could run.
    //
    // Ending the stream rather than yielding an error is load-bearing —
    // `add_stream` only sends its terminating `Done` when the input runs to
    // completion, so an error item leaves the writer waiting forever.
    let stream = {
        let total_bytes = Arc::clone(&total_bytes);
        let aborted = Arc::clone(&aborted);
        rx.into_stream()
            .take_while(move |incoming| {
                let within = incoming.as_ref().is_ok_and(|bytes| {
                    let len = i64::try_from(bytes.len()).unwrap_or(i64::MAX);
                    total_bytes
                        .fetch_add(len, Ordering::Release)
                        .saturating_add(len)
                        <= headroom
                });
                if !within {
                    aborted.store(true, Ordering::Release);
                }
                ready(within)
            })
            .map(|incoming| incoming.map_err(io::Error::other))
    };

    let upload = ctx
        .blob_store()
        .blobs()
        .add_stream(stream)
        .await
        .temp_tag()
        .await;
    let blob_len = total_bytes.load(Ordering::Acquire);

    if aborted.load(Ordering::Acquire) {
        let err = if blob_len > headroom {
            ApiError::QuotaExceeded
        } else {
            ApiError::Internal
        };
        tx.send(Err(err)).await?;
        return Ok(());
    }

    let Ok(temp_tag) = upload else {
        tx.send(Err(ApiError::Internal)).await?;
        return Ok(());
    };

    debug!(?blob_len, "wrote blob to store");

    let hash: Hash = temp_tag.hash().into();
    let hash_str = hash.to_string();
    let expires = (OffsetDateTime::now_utc() + DEFAULT_BLOB_TTL).unix_timestamp();

    let quota_ok = record_pin(&ctx, &did_str, &hash_str, blob_len, expires).await?;

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

/// Records the owner's pin of a freshly uploaded blob, charging its size.
///
/// Returns `false` when the charge would exceed the owner's quota.
async fn record_pin(
    ctx: &StoreContext,
    did: &str,
    hash: &str,
    blob_len: i64,
    expires: i64,
) -> anyhow::Result<bool> {
    let did = did.to_string();
    let hash = hash.to_string();
    ctx.db
        .call_mut(move |conn| {
            let tx = conn.transaction()?;
            // Re-uploading content this owner already pins must not charge
            // twice: the pin is one row, so it can only be released once.
            let pinned = tx
                .query_row(
                    "SELECT 1 FROM blob_pins WHERE owner = ? AND hash = ?",
                    params![&did, &hash],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if pinned {
                tx.execute(
                    "UPDATE blob_pins SET expires = MAX(expires, ?) WHERE owner = ? AND hash = ?",
                    params![expires, &did, &hash],
                )?;
                tx.commit()?;
                return Ok(true);
            }
            if reserve_bytes(&tx, &did, blob_len).is_err() {
                return Ok(false);
            }
            tx.execute(
                "INSERT INTO blob_pins (hash, owner, expires, size) VALUES (?, ?, ?, ?)",
                params![&hash, &did, expires, blob_len],
            )?;
            tx.commit()?;
            Ok(true)
        })
        .await
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

/// Whether the caller's *own* pin of this blob is present.
///
/// Scoped to the caller rather than asking the blob store directly: content
/// addressing makes an unscoped answer a membership oracle over everyone
/// else's data.
pub async fn blob_exists(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<BlobExists, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();
    let hash_str = inner.hash.to_string();

    let pinned = ctx
        .db
        .call(move |conn| {
            Ok(conn
                .query_row(
                    "SELECT 1 FROM blob_pins WHERE owner = ? AND hash = ?",
                    params![&did_str, &hash_str],
                    |_| Ok(true),
                )
                .unwrap_or(false))
        })
        .await?;

    let exists = pinned
        && ctx
            .blob_store()
            .blobs()
            .has(inner.hash)
            .await
            .unwrap_or(false);

    tx.send(Ok(exists)).await?;
    Ok(())
}
