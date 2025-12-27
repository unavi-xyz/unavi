use blake3::Hash;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use iroh::EndpointId;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{SessionToken, StoreContext, sync::combined_stream::CombinedStream};

use super::{ALPN, SyncMsg};

#[derive(Debug)]
pub struct SyncResult {
    pub received: usize,
    pub sent: usize,
}

pub async fn sync_to_remote(
    ctx: &StoreContext,
    remote: EndpointId,
    session: SessionToken,
    record_id: Hash,
) -> anyhow::Result<SyncResult> {
    let db = ctx.db.pool();
    let id_str = record_id.to_string();

    let conn = ctx.endpoint.connect(remote, ALPN).await?;
    let (tx, rx) = conn.open_bi().await?;
    let mut framed = Framed::new(CombinedStream(tx, rx), LengthDelimitedCodec::new());

    let local_vv = super::shared::get_record_vv(db, &id_str).await?;

    let begin = SyncMsg::Begin {
        session,
        record_id,
        vv: local_vv.encode(),
    };
    framed
        .send(BytesMut::from(postcard::to_stdvec(&begin)?.as_slice()).freeze())
        .await?;

    // Receive envelopes from server.
    let Some(bytes) = framed.next().await else {
        anyhow::bail!("connection closed");
    };
    let incoming: SyncMsg = postcard::from_bytes(&bytes?)?;

    let received = if let SyncMsg::Envelopes(envelopes) = incoming {
        let count = envelopes.len();
        for env_bytes in &envelopes {
            super::shared::store_envelope(db, &ctx.blobs, &id_str, env_bytes).await?;
        }
        count
    } else {
        0
    };

    // Send our envelopes.
    let to_send = super::shared::fetch_all_envelopes(db, &id_str).await?;
    let sent = to_send.len();
    framed
        .send(
            BytesMut::from(postcard::to_stdvec(&SyncMsg::Envelopes(to_send))?.as_slice()).freeze(),
        )
        .await?;

    let Some(bytes) = framed.next().await else {
        anyhow::bail!("connection closed");
    };
    let done: SyncMsg = postcard::from_bytes(&bytes?)?;

    if !matches!(done, SyncMsg::Done) {
        anyhow::bail!("expected Done message");
    }

    Ok(SyncResult { received, sent })
}
