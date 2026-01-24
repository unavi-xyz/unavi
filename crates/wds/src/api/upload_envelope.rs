use std::sync::Arc;

use futures::TryStreamExt;
use irpc::WithChannels;
use tracing::warn;

use crate::{
    StoreContext,
    api::{ApiError, ApiService, UploadEnvelope, authenticate},
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

    let result = store_envelope(&ctx.db, ctx.blobs.as_ref().as_ref(), &record_id, &env_bytes).await;

    if let Err(ref e) = result {
        warn!(record_id = %record_id, ?e, "failed to store envelope");
    }

    tx.send(result.map_err(ApiError::from)).await?;
    Ok(())
}
