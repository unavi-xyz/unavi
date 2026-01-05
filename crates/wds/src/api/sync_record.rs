use std::sync::Arc;

use irpc::WithChannels;
use smol_str::ToSmolStr;

use crate::{StoreContext, sync::client::sync_to_remote};

use super::{ApiService, SyncRecord, authenticate};

pub async fn sync_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<SyncRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);

    let result = sync_to_remote(did, &ctx, inner.remote, inner.record_id).await;

    tx.send(result.map_err(|e| e.to_smolstr())).await?;

    Ok(())
}
