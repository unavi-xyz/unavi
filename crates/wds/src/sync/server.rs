use blake3::Hash;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use loro::VersionVector;
use rusqlite::params;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;
use xdid::core::did::Did;

use wired_schemas::surg::Acl;

use crate::{SessionToken, StoreContext, error::ApiError, sync::SyncMsg};

/// Reads the next message from the stream and decodes it.
async fn recv_msg<S>(framed: &mut Framed<S, LengthDelimitedCodec>) -> Result<SyncMsg, ApiError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let Some(bytes) = framed.next().await else {
        return Err(ApiError::SyncFailed);
    };
    let bytes = bytes.map_err(|_| ApiError::SyncFailed)?;
    postcard::from_bytes(&bytes).map_err(|_| ApiError::SyncFailed)
}

/// Sends a message to the stream.
async fn send_msg<S>(
    framed: &mut Framed<S, LengthDelimitedCodec>,
    msg: &SyncMsg,
) -> Result<(), ApiError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let msg_bytes = postcard::to_stdvec(msg).map_err(|_| ApiError::Internal)?;
    framed
        .send(BytesMut::from(msg_bytes.as_slice()).freeze())
        .await
        .map_err(|_| ApiError::SyncFailed)
}

/// Authenticates the sync request and checks read permission.
async fn authenticate_and_authorize(
    ctx: &StoreContext,
    session: &SessionToken,
    record_id: &Hash,
) -> Result<(Did, VersionVector), ApiError> {
    let Some(conn_state) = ctx.connections.get_async(session).await else {
        return Err(ApiError::Unauthenticated);
    };
    let requester_did = conn_state.get().did.clone();
    drop(conn_state);

    let id_str = record_id.to_string();

    // Check if record is pinned and get version vector.
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
        .await
        .map_err(|_| ApiError::Internal)?;

    if !is_pinned {
        info!("sync denied: record not pinned");
        return Err(ApiError::RecordNotFound);
    }

    let local_vv = match found_vv {
        Some(vv_bytes) => VersionVector::decode(&vv_bytes).map_err(|_| ApiError::Internal)?,
        None => VersionVector::new(),
    };

    // Check read permission.
    let doc = super::shared::reconstruct_current_doc(&ctx.db, &id_str)
        .await
        .map_err(|_| ApiError::Internal)?;
    let acl = Acl::load(&doc).map_err(|_| ApiError::Internal)?;
    if !acl.can_read(&requester_did) {
        info!(?acl, %requester_did, "sync denied: user not permitted");
        return Err(ApiError::RecordNotFound);
    }

    Ok((requester_did, local_vv))
}

pub async fn handle_sync<S>(
    ctx: &StoreContext,
    mut framed: Framed<S, LengthDelimitedCodec>,
) -> Result<(), ApiError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let request = recv_msg(&mut framed).await?;

    let SyncMsg::Begin {
        session,
        record_id,
        vv: remote_vv_bytes,
    } = request
    else {
        return Err(ApiError::SyncFailed);
    };

    info!(?record_id, "handling sync");

    let (_requester_did, local_vv) = authenticate_and_authorize(ctx, &session, &record_id).await?;

    let id_str = record_id.to_string();

    // Send blob dependency hashes so client can fetch missing blobs.
    let blob_hashes = ctx
        .db
        .call({
            let id_str = id_str.clone();
            move |conn| super::shared::get_blob_dep_hashes_sync(conn, &id_str)
        })
        .await
        .map_err(|_| ApiError::Internal)?;

    send_msg(&mut framed, &SyncMsg::BlobHashes(blob_hashes)).await?;

    // Wait for client to signal it has fetched all needed blobs.
    let ready_msg = recv_msg(&mut framed).await?;
    if !matches!(ready_msg, SyncMsg::Ready) {
        return Err(ApiError::SyncFailed);
    }

    let remote_vv = VersionVector::decode(&remote_vv_bytes).map_err(|_| ApiError::SyncFailed)?;

    // Send envelopes remote is missing.
    let to_send = ctx
        .db
        .call({
            let id_str = id_str.clone();
            move |conn| {
                super::shared::fetch_envelopes_for_diff_sync(conn, &id_str, &local_vv, &remote_vv)
            }
        })
        .await
        .map_err(|_| ApiError::Internal)?;

    send_msg(&mut framed, &SyncMsg::Envelopes(to_send)).await?;

    // Receive envelopes we're missing. Write permission is checked in store_envelope.
    let Ok(incoming) = recv_msg(&mut framed).await else {
        return Ok(());
    };

    if let SyncMsg::Envelopes(envelopes) = incoming {
        let blobs = ctx.blobs.as_ref().as_ref();
        for env_bytes in envelopes {
            super::shared::store_envelope(&ctx.db, blobs, &id_str, &env_bytes)
                .await
                .map_err(ApiError::from)?;
        }
    }

    Ok(())
}
