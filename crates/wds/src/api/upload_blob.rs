use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use futures::{StreamExt, TryStreamExt};
use irpc::WithChannels;
use n0_error::Meta;
use time::OffsetDateTime;

use crate::{
    ConnectionState,
    api::{ApiService, UploadBlob, authenticate, tag::BlobTag},
    quota::ensure_quota_exists,
};

const DEFAULT_BLOB_TTL: Duration = Duration::from_hours(1);

pub async fn upload_blob(
    conn: Arc<ConnectionState>,
    WithChannels { inner, tx, rx, .. }: WithChannels<UploadBlob, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(conn, tx);
    let did_str = did.to_string();

    let db = conn.db.pool();
    let mut db_tx = db.begin().await?;
    ensure_quota_exists(&mut *db_tx, &did_str).await?;
    db_tx.commit().await?;

    let total_bytes = Arc::new(AtomicUsize::default());
    let target_len = inner.byte_len;

    let stream = {
        let total_bytes = Arc::clone(&total_bytes);
        rx.into_stream()
            .map(move |res| {
                if let Ok(b) = &res {
                    let len = total_bytes.fetch_add(b.len(), Ordering::Release);
                    if len > target_len {
                        return Err(irpc::channel::mpsc::RecvError::MaxMessageSizeExceeded {
                            meta: Meta::default(),
                        });
                    }
                }
                res
            })
            .map_err(|_| std::io::ErrorKind::Other.into())
    };

    let temp_tag = conn.blob_store.add_stream(stream).await.temp_tag().await?;
    if temp_tag.hash().as_bytes() != inner.hash.as_bytes() {
        tx.send(Err("stored hash does not equal specified hash".into()))
            .await?;
        return Ok(());
    }

    let blob_len = total_bytes.load(Ordering::Acquire);
    if blob_len != target_len {
        tx.send(Err("invalid byte length".into())).await?;
        return Ok(());
    }
    let blob_len = i64::try_from(blob_len)?;

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

    let hash_str = temp_tag.hash().to_string();

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
    let blob_tag = BlobTag::new(did.clone(), temp_tag.hash().into());
    let tag_name = blob_tag.to_string();

    conn.blob_store.tags().set(tag_name, temp_tag).await?;

    // TODO We could end up with DB tracking but no blob tag if we crash or error.

    Ok(())
}
