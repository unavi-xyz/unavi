use std::sync::Arc;

use irpc::WithChannels;
use loro::ExportMode;
use tracing::warn;

use wired_schemas::surg::Acl;

use crate::{StoreContext, sync::shared::reconstruct_current_doc};

use super::{ApiError, ApiService, ReadRecord, authenticate};

pub async fn read_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<ReadRecord, ApiService>,
) -> anyhow::Result<()> {
    let requester = authenticate!(ctx, inner, tx);
    let id_str = inner.record_id.to_string();

    // Reconstruct the document from stored envelopes.
    let doc = match reconstruct_current_doc(&ctx.db, &id_str).await {
        Ok(doc) => doc,
        Err(err) => {
            warn!(record_id = %id_str, ?err, "failed to reconstruct doc");
            tx.send(Err(ApiError::Internal)).await?;
            return Ok(());
        }
    };

    // Check if record exists.
    if doc.state_frontiers().is_empty() {
        tx.send(Err(ApiError::RecordNotFound)).await?;
        return Ok(());
    }

    // Check ACL read permission.
    let acl = match Acl::load(&doc) {
        Ok(acl) => acl,
        Err(err) => {
            warn!(record_id = %id_str, ?err, "failed to load acl");
            tx.send(Err(ApiError::Internal)).await?;
            return Ok(());
        }
    };

    if !acl.can_read(&requester) {
        tx.send(Err(ApiError::AccessDenied)).await?;
        return Ok(());
    }

    // Export document bytes.
    let bytes = match doc.export(ExportMode::Snapshot) {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!(record_id = %id_str, ?err, "failed to export doc");
            tx.send(Err(ApiError::Internal)).await?;
            return Ok(());
        }
    };

    tx.send(Ok(bytes)).await?;
    Ok(())
}
