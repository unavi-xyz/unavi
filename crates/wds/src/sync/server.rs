use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use loro::VersionVector;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{StoreContext, sync::SyncMsg};

pub async fn handle_sync<S>(
    ctx: &StoreContext,
    mut framed: Framed<S, LengthDelimitedCodec>,
) -> anyhow::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let Some(bytes) = framed.next().await else {
        return Ok(());
    };
    let request: SyncMsg = postcard::from_bytes(&bytes?)?;

    let SyncMsg::Begin {
        session,
        record_id,
        vv: remote_vv_bytes,
    } = request
    else {
        anyhow::bail!("expected Begin message");
    };

    // Authenticate.
    if ctx.connections.get_async(&session).await.is_none() {
        return Ok(());
    }

    let id_str = record_id.to_string();
    let db = ctx.db.pool();

    let found = sqlx::query!("SELECT vv FROM records WHERE id = ?", id_str)
        .fetch_optional(db)
        .await?;

    let local_vv = if let Some(row) = found {
        VersionVector::decode(&row.vv)?
    } else {
        // Allow sync if record is pinned (for initialization).
        let pinned = sqlx::query!("SELECT owner FROM record_pins WHERE record_id = ?", id_str)
            .fetch_optional(db)
            .await?;

        if pinned.is_none() {
            return Ok(());
        }

        VersionVector::new()
    };

    let remote_vv = VersionVector::decode(&remote_vv_bytes)?;

    // Send envelopes remote is missing.
    let to_send =
        super::shared::fetch_envelopes_for_diff(db, &id_str, &local_vv, &remote_vv).await?;
    let msg_bytes = postcard::to_stdvec(&SyncMsg::Envelopes(to_send))?;
    framed
        .send(BytesMut::from(msg_bytes.as_slice()).freeze())
        .await?;

    // Receive envelopes we're missing.
    let Some(bytes) = framed.next().await else {
        return Ok(());
    };
    let incoming: SyncMsg = postcard::from_bytes(&bytes?)?;

    if let SyncMsg::Envelopes(envelopes) = incoming {
        for env_bytes in envelopes {
            super::shared::store_envelope(db, &ctx.blobs, &id_str, &env_bytes).await?;
        }
    }

    let done_bytes = postcard::to_stdvec(&SyncMsg::Done)?;
    framed
        .send(BytesMut::from(done_bytes.as_slice()).freeze())
        .await?;

    Ok(())
}
