use anyhow::bail;
use blake3::Hash;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use iroh::{EndpointAddr, endpoint::VarInt};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{StoreContext, sync::combined_stream::CombinedStream};

use super::{ALPN, SyncMsg};

pub async fn sync_to_remote(
    ctx: &StoreContext,
    remote: EndpointAddr,
    record_id: Hash,
) -> anyhow::Result<()> {
    let db = ctx.db.pool();
    let id_str = record_id.to_string();

    let connection = ctx.endpoint.connect(remote, ALPN).await?;
    let (tx, rx) = connection.open_bi().await?;
    let mut framed = Framed::new(CombinedStream(tx, rx), LengthDelimitedCodec::new());

    // TODO auth with wds/auth

    let local_vv = super::shared::get_record_vv(db, &id_str).await?;

    let begin = SyncMsg::Begin {
        session: [0; 32],
        record_id,
        vv: local_vv.encode(),
    };
    framed
        .send(BytesMut::from(postcard::to_stdvec(&begin)?.as_slice()).freeze())
        .await?;

    // Receive envelopes from remote.
    let Some(bytes) = framed.next().await else {
        anyhow::bail!("no next frame");
    };
    let incoming: SyncMsg = postcard::from_bytes(&bytes?)?;

    if let SyncMsg::Envelopes(envelopes) = incoming {
        for env_bytes in &envelopes {
            super::shared::store_envelope(db, &ctx.blobs, &id_str, env_bytes).await?;
        }
    } else {
        bail!("unexpected message")
    }

    // Send our envelopes.
    let to_send = super::shared::fetch_all_envelopes(db, &id_str).await?;
    let to_send = postcard::to_stdvec(&SyncMsg::Envelopes(to_send))?;
    framed.send(to_send.into()).await?;

    connection.close(VarInt::default(), b"done");

    Ok(())
}
