use std::{
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
    time::Duration,
};

use blake3::Hash;
use futures::{StreamExt, TryStreamExt};
use irpc::WithChannels;
use n0_error::Meta;
use rusqlite::params;
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    StoreContext,
    api::{ApiError, ApiService, UploadBlob, authenticate},
    quota::{ensure_quota_exists, reserve_bytes},
    tag::BlobTag,
};

const DEFAULT_BLOB_TTL: Duration = Duration::from_hours(1);

pub async fn upload_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, rx, .. }: WithChannels<UploadBlob, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    // Ensure quota exists and get current quota info.
    let (bytes_used, quota_bytes) = ctx
        .db
        .call({
            let did_str = did_str.clone();
            move |conn| {
                ensure_quota_exists(conn, &did_str)?;

                let (bytes_used, quota_bytes): (i64, i64) = conn.query_row(
                    "SELECT bytes_used, quota_bytes FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;

                Ok((bytes_used, quota_bytes))
            }
        })
        .await?;

    // May become inaccurate by the time the upload completes.
    // Limit is properly enforced in the update transaction later.
    // This serves to prevent outrageous upload sizes.
    let estimated_max_len = quota_bytes - bytes_used;

    let total_bytes = Arc::new(AtomicI64::default());

    let stream = {
        let total_bytes = Arc::clone(&total_bytes);
        rx.into_stream()
            .map(move |incoming| {
                if let Ok(b) = &incoming {
                    let b_len = i64::try_from(b.len()).unwrap_or(i64::MAX);
                    let len = total_bytes.fetch_add(b_len, Ordering::Release);
                    if len > estimated_max_len {
                        return Err(irpc::channel::mpsc::RecvError::MaxMessageSizeExceeded {
                            meta: Meta::default(),
                        });
                    }
                }
                incoming
            })
            .map_err(|_| std::io::ErrorKind::Other.into())
    };

    let temp_tag = ctx
        .blobs
        .as_ref()
        .as_ref()
        .add_stream(stream)
        .await
        .temp_tag()
        .await?;
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

    // Only persist blob tag after tracking blob in DB.
    let blob_tag = BlobTag::new(did.clone(), hash);
    let tag_name = blob_tag.to_string();

    ctx.blobs
        .as_ref()
        .as_ref()
        .tags()
        .set(tag_name, temp_tag)
        .await?;

    tx.send(Ok(hash)).await?;

    Ok(())
}
