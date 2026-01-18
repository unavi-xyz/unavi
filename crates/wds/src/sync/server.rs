use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use loro::VersionVector;
use rusqlite::params;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::debug;

use wired_schemas::Acl;

use crate::{StoreContext, sync::SyncMsg};

pub async fn handle_sync<S>(
    ctx: &StoreContext,
    mut framed: Framed<S, LengthDelimitedCodec>,
) -> anyhow::Result<&'static str>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let Some(bytes) = framed.next().await else {
        return Ok("no bytes");
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

    debug!(?record_id, "handling sync");

    // Authenticate and get requester DID.
    let Some(conn_state) = ctx.connections.get_async(&session).await else {
        return Ok("unauthenticated");
    };
    let requester_did = conn_state.get().did.clone();
    drop(conn_state);

    let id_str = record_id.to_string();

    let (found_vv, is_pinned) = ctx
        .db
        .call({
            let id_str = id_str.clone();
            move |conn| {
                let found_vv: Option<Vec<u8>> = conn
                    .query_row(
                        "SELECT vv FROM records WHERE id = ?",
                        params![&id_str],
                        |row| row.get(0),
                    )
                    .ok();

                let is_pinned: bool = conn
                    .query_row(
                        "SELECT 1 FROM record_pins WHERE record_id = ? LIMIT 1",
                        params![&id_str],
                        |_| Ok(true),
                    )
                    .unwrap_or(false);

                Ok((found_vv, is_pinned))
            }
        })
        .await?;

    // Only sync if record is pinned.
    if !is_pinned {
        return Ok("not found");
    }

    let local_vv = if let Some(vv_bytes) = found_vv {
        VersionVector::decode(&vv_bytes)?
    } else {
        VersionVector::new()
    };

    // Check read permission before sending data.
    let doc = super::shared::reconstruct_current_doc(&ctx.db, &id_str).await?;
    let acl = Acl::load(&doc)?;
    if !acl.can_read(&requester_did) {
        // Silently deny - record "doesn't exist" for this user.
        return Ok("not found");
    }

    // Send blob dependency hashes so client can fetch missing blobs.
    let blob_hashes = ctx
        .db
        .call({
            let id_str = id_str.clone();
            move |conn| super::shared::get_blob_dep_hashes_sync(conn, &id_str)
        })
        .await?;

    let msg_bytes = postcard::to_stdvec(&SyncMsg::BlobHashes(blob_hashes))?;
    framed
        .send(BytesMut::from(msg_bytes.as_slice()).freeze())
        .await?;

    // Wait for client to signal it has fetched all needed blobs.
    let Some(bytes) = framed.next().await else {
        return Ok("no ready");
    };
    let ready_msg: SyncMsg = postcard::from_bytes(&bytes?)?;
    if !matches!(ready_msg, SyncMsg::Ready) {
        anyhow::bail!("expected Ready message");
    }

    let remote_vv = VersionVector::decode(&remote_vv_bytes)?;

    // Send envelopes remote is missing.
    let to_send = ctx
        .db
        .call({
            let id_str = id_str.clone();
            move |conn| {
                super::shared::fetch_envelopes_for_diff_sync(conn, &id_str, &local_vv, &remote_vv)
            }
        })
        .await?;

    let msg_bytes = postcard::to_stdvec(&SyncMsg::Envelopes(to_send))?;
    framed
        .send(BytesMut::from(msg_bytes.as_slice()).freeze())
        .await?;

    // Receive envelopes we're missing.
    // Write permission is checked in store_envelope.
    let Some(bytes) = framed.next().await else {
        return Ok("done");
    };
    let incoming: SyncMsg = postcard::from_bytes(&bytes?)?;

    if let SyncMsg::Envelopes(envelopes) = incoming {
        let blobs = ctx.blobs.as_ref().as_ref();
        for env_bytes in envelopes {
            super::shared::store_envelope(&ctx.db, blobs, &id_str, &env_bytes).await?;
        }
    }

    Ok("done")
}
