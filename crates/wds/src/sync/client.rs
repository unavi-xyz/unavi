use anyhow::{Context, bail};
use blake3::Hash;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use iroh::EndpointAddr;
use iroh_blobs::{BlobFormat, HashAndFormat};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, warn};
use xdid::{core::did::Did, methods::key::Signer};

use crate::{StoreContext, auth::AuthService, sync::combined_stream::CombinedStream};

use super::SyncMsg;

pub async fn sync_to_remote<S>(
    did: Did,
    signer: &S,
    ctx: &StoreContext,
    remote: EndpointAddr,
    record_id: Hash,
) -> anyhow::Result<()>
where
    S: Signer + Sync,
{
    let db = ctx.db.pool();
    let id_str = record_id.to_string();

    // Authenticate.
    let auth_client =
        irpc_iroh::client::<AuthService>(ctx.endpoint.clone(), remote.clone(), crate::auth::ALPN);

    let session = crate::auth::client::authenticate(did, signer, remote.id, &auth_client)
        .await
        .context("auth")?;

    // Sync.
    let connection = ctx
        .endpoint
        .connect(remote.clone(), crate::sync::ALPN)
        .await?;
    let (tx, rx) = connection.open_bi().await?;
    let mut framed = Framed::new(CombinedStream(tx, rx), LengthDelimitedCodec::new());

    let local_vv = super::shared::get_record_vv(db, &id_str).await?;

    debug!(%record_id, "beginning sync");

    let begin = SyncMsg::Begin {
        session,
        record_id,
        vv: local_vv.encode(),
    };
    framed
        .send(BytesMut::from(postcard::to_stdvec(&begin)?.as_slice()).freeze())
        .await?;

    // Receive blob dependency hashes from remote.
    let Some(bytes) = framed.next().await else {
        anyhow::bail!("no next frame");
    };
    let incoming: SyncMsg = postcard::from_bytes(&bytes?)?;

    let SyncMsg::BlobHashes(blob_hashes) = incoming else {
        bail!("expected BlobHashes message");
    };

    // Fetch missing blobs via iroh_blobs protocol.
    let blobs = ctx.blobs.as_ref().as_ref();
    let missing_hashes: Vec<_> = {
        let mut missing = Vec::new();
        for hash in blob_hashes {
            let iroh_hash: iroh_blobs::Hash = hash.into();
            if !blobs.has(iroh_hash).await? {
                missing.push(hash);
            }
        }
        missing
    };

    if !missing_hashes.is_empty() {
        // Connect via iroh_blobs ALPN to fetch blobs.
        let blob_conn = ctx
            .endpoint
            .connect(remote.clone(), iroh_blobs::ALPN)
            .await
            .context("connect for blobs")?;

        for hash in missing_hashes {
            let iroh_hash: iroh_blobs::Hash = hash.into();
            debug!(%hash, "fetching missing blob");
            if let Err(e) = blobs
                .remote()
                .fetch(
                    blob_conn.clone(),
                    HashAndFormat {
                        hash: iroh_hash,
                        format: BlobFormat::Raw,
                    },
                )
                .await
            {
                warn!(%hash, error = %e, "failed to fetch blob");
            }
        }
    }

    // Signal we're ready for envelopes.
    let ready = postcard::to_stdvec(&SyncMsg::Ready)?;
    framed.send(ready.into()).await?;

    // Receive envelopes from remote.
    let Some(bytes) = framed.next().await else {
        anyhow::bail!("no envelopes frame");
    };
    let incoming: SyncMsg = postcard::from_bytes(&bytes?)?;

    let SyncMsg::Envelopes(envelopes) = incoming else {
        bail!("expected Envelopes message");
    };

    for env_bytes in &envelopes {
        super::shared::store_envelope(db, blobs, &id_str, env_bytes)
            .await
            .context("store envelope")?;
    }

    // Send our envelopes.
    let to_send = super::shared::fetch_all_envelopes(db, &id_str).await?;
    let to_send = postcard::to_stdvec(&SyncMsg::Envelopes(to_send))?;
    framed.send(to_send.into()).await?;

    let reason = connection.closed().await;
    debug!(%reason,"sync closed");

    Ok(())
}
