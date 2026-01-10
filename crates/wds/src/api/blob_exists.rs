use std::sync::Arc;

use irpc::WithChannels;

use crate::StoreContext;

use super::{ApiError, ApiService, BlobExists, authenticate};

pub async fn blob_exists(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<BlobExists, ApiService>,
) -> anyhow::Result<()> {
    let _did = authenticate!(ctx, inner, tx);

    let iroh_hash: iroh_blobs::Hash = inner.hash.into();
    let exists = ctx.blobs.as_ref().as_ref().blobs().has(iroh_hash).await?;

    tx.send(Ok(exists)).await?;

    Ok(())
}
