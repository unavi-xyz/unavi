use std::sync::Arc;

use irpc::WithChannels;
use tracing::warn;

use crate::{Identity, StoreContext, signed_bytes::IrohSigner, sync::client::sync_to_remote};

use super::{ApiError, ApiService, SyncRecord, authenticate};

pub async fn sync_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<SyncRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);

    // Clone user identity out of lock to avoid holding it across await.
    let user_identity: Option<Arc<Identity>> = ctx.user_identity.read().clone();

    // Use user identity's signing key if available (embedded WDS),
    // otherwise fall back to endpoint key (requires DID doc service entry).
    let result = if let Some(identity) = &user_identity {
        sync_to_remote(
            did,
            identity.signing_key(),
            &ctx,
            inner.remote,
            inner.record_id,
        )
        .await
    } else {
        sync_to_remote(
            did,
            &IrohSigner(ctx.endpoint.secret_key()),
            &ctx,
            inner.remote,
            inner.record_id,
        )
        .await
    };

    tx.send(result.map_err(|err| {
        warn!(?err, "SyncRecord failed");
        ApiError::SyncFailed
    }))
    .await?;

    Ok(())
}
