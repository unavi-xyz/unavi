use blake3::Hash;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use iroh::EndpointId;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{SessionToken, StoreContext, sync::combined_stream::CombinedStream};

use super::{ALPN, SyncMsg};

/// Result of a sync operation.
#[derive(Debug)]
pub struct SyncResult {
    pub received: usize,
    pub sent: usize,
}

/// Syncs a record with a remote node.
pub async fn sync_to_remote(
    ctx: &StoreContext,
    remote: EndpointId,
    session: SessionToken,
    record_id: Hash,
) -> anyhow::Result<SyncResult> {
    let db = ctx.db.pool();
    let id_str = record_id.to_string();

    // 1. Connect to remote.
    let conn = ctx.endpoint.connect(remote, ALPN).await?;
    let (tx, rx) = conn.open_bi().await?;
    let combined = CombinedStream(tx, rx);
    let mut framed = Framed::new(combined, LengthDelimitedCodec::new());

    // 2. Begin sync by sending version vector.
    let local_vv = super::shared::get_record_vv(db, &id_str).await?;
    let vv_bytes = local_vv.encode();

    let begin = SyncMsg::Begin {
        session,
        record_id,
        vv: vv_bytes,
    };
    let begin_bytes = postcard::to_stdvec(&begin)?;
    framed
        .send(BytesMut::from(begin_bytes.as_slice()).freeze())
        .await?;

    // 3. Receive envelopes from server.
    let Some(bytes) = framed.next().await else {
        anyhow::bail!("connection closed before receiving envelopes");
    };
    let bytes = bytes?;
    let incoming: SyncMsg = postcard::from_bytes(&bytes)?;

    let received = if let SyncMsg::Envelopes(envelopes) = incoming {
        let count = envelopes.len();
        for env_bytes in &envelopes {
            super::shared::store_envelope(db, &id_str, env_bytes).await?;
        }
        count
    } else {
        0
    };

    // 4. Send envelopes the server is missing.
    let to_send = super::shared::fetch_all_envelopes(db, &id_str).await?;
    let sent = to_send.len();
    let msg = SyncMsg::Envelopes(to_send);
    let msg_bytes = postcard::to_stdvec(&msg)?;
    framed
        .send(BytesMut::from(msg_bytes.as_slice()).freeze())
        .await?;

    // 5. Wait for done.
    let Some(bytes) = framed.next().await else {
        anyhow::bail!("connection closed before Done");
    };
    let bytes = bytes?;
    let done: SyncMsg = postcard::from_bytes(&bytes)?;

    if !matches!(done, SyncMsg::Done) {
        anyhow::bail!("expected Done message");
    }

    Ok(SyncResult { received, sent })
}
