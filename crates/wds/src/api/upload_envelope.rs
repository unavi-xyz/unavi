use std::sync::Arc;

use futures::TryStreamExt;
use irpc::WithChannels;

use crate::{
    StoreContext,
    api::{ApiService, UploadEnvelope, authenticate},
    sync::shared::store_envelope,
};

pub async fn upload_envelope(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, rx, .. }: WithChannels<UploadEnvelope, ApiService>,
) -> anyhow::Result<()> {
    let _did = authenticate!(ctx, inner, tx);

    // Collect envelope bytes from stream.
    let env_bytes: Vec<u8> = rx
        .into_stream()
        .try_fold(Vec::new(), |mut acc, chunk| async move {
            acc.extend_from_slice(&chunk);
            Ok(acc)
        })
        .await
        .map_err(|e| anyhow::anyhow!("failed to receive envelope: {e}"))?;

    let record_id = inner.record_id.to_string();

    if let Err(e) = store_envelope(ctx.db.pool(), &ctx.blobs, &record_id, &env_bytes).await {
        tx.send(Err(e.to_string().into())).await?;
        return Ok(());
    }

    tx.send(Ok(())).await?;
    Ok(())
}
