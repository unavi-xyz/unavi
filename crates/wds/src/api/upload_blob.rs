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
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    StoreContext,
    api::{ApiService, UploadBlob, authenticate, tag::BlobTag},
    quota::ensure_quota_exists,
};

const DEFAULT_BLOB_TTL: Duration = Duration::from_hours(1);

pub async fn upload_blob(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, rx, .. }: WithChannels<UploadBlob, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let db = ctx.db.pool();

    ensure_quota_exists(db, &did_str).await?;

    let quota = sqlx::query!(
        "SELECT bytes_used, quota_bytes FROM user_quotas WHERE owner = ?",
        did_str
    )
    .fetch_one(db)
    .await?;

    // May become inaccurate by the time the upload completes.
    // Limit is properly enforced in the update transaction later.
    // This serves to prevent outrageous upload sizes.
    let estimated_max_len = quota.quota_bytes - quota.bytes_used;

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

    let temp_tag = ctx.blobs.add_stream(stream).await.temp_tag().await?;
    let blob_len = total_bytes.load(Ordering::Acquire);

    debug!(?blob_len, "wrote blob to store");

    let mut db_tx = db.begin().await?;

    let update_res = sqlx::query!(
        "UPDATE user_quotas
         SET bytes_used = bytes_used + ?
         WHERE owner = ? AND bytes_used + ? <= quota_bytes",
        blob_len,
        did_str,
        blob_len
    )
    .execute(&mut *db_tx)
    .await?;

    if update_res.rows_affected() == 0 {
        // Quota exceeded (or user doesn't exist).
        tx.send(Err("quota exceeded".into())).await?;
        return Ok(());
    }

    let hash: Hash = temp_tag.hash().into();
    let hash_str = hash.to_string();

    let expires = (OffsetDateTime::now_utc() + DEFAULT_BLOB_TTL).unix_timestamp();

    sqlx::query!(
        "INSERT INTO blob_pins (hash, owner, expires, size) VALUES (?, ?, ?, ?)",
        hash_str,
        did_str,
        expires,
        blob_len
    )
    .execute(&mut *db_tx)
    .await?;

    db_tx.commit().await?;

    // Only persist blob tag after tracking blob in DB.
    let blob_tag = BlobTag::new(did.clone(), hash);
    let tag_name = blob_tag.to_string();

    ctx.blobs.tags().set(tag_name, temp_tag).await?;

    // TODO We could end up with DB tracking but no blob tag if we crash or error.

    tx.send(Ok(hash)).await?;

    Ok(())
}
