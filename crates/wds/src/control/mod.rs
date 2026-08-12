//! [`irpc`] control-plane API for hosting, pinning, uploads and quota; data
//! moves over iroh-docs reconciliation and iroh-blobs.

use std::{
    sync::Arc,
    time::Duration,
};

use blake3::Hash;
use bytes::Bytes;
use iroh_docs::NamespaceId;
use irpc::{
    Client,
    channel::{
        mpsc,
        oneshot,
    },
    rpc_requests,
};
use irpc_iroh::IrohProtocol;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::{
    error,
    warn,
};

use crate::{
    SessionToken,
    StoreContext,
    error::ApiError,
};

mod blob;
mod doc;
mod quota;

const MAX_PIN_DURATION: Duration = Duration::from_hours(24 * 90);

pub const ALPN: &[u8] = b"wds/control/0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub bytes_used:  i64,
    pub quota_bytes: i64,
}

pub(crate) fn protocol(
    ctx: Arc<StoreContext>,
) -> (Client<ControlService>, IrohProtocol<ControlService>) {
    let (tx, mut rx) = irpc::channel::mpsc::channel(32);

    n0_future::task::spawn(async move {
        while let Err(err) = handle_requests(&ctx, &mut rx).await {
            error!("Error handling request: {err:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    (client, IrohProtocol::with_sender(local_sender))
}

#[rpc_requests(message = ControlMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ControlService {
    #[rpc(rx=mpsc::Receiver<Bytes>,tx=oneshot::Sender<Result<Hash, ApiError>>)]
    #[wrap(UploadBlob)]
    UploadBlob { s: SessionToken },
    #[rpc(tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(PinBlob)]
    PinBlob {
        s:       SessionToken,
        hash:    Hash,
        expires: i64,
    },
    #[rpc(tx=oneshot::Sender<Result<bool, ApiError>>)]
    #[wrap(BlobExists)]
    BlobExists { s: SessionToken, hash: Hash },
    #[rpc(tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(HostDoc)]
    HostDoc { s: SessionToken, ns: NamespaceId },
    #[rpc(tx=oneshot::Sender<Result<(), ApiError>>)]
    #[wrap(UnhostDoc)]
    UnhostDoc { s: SessionToken, ns: NamespaceId },
    #[rpc(tx=oneshot::Sender<Result<QuotaInfo, ApiError>>)]
    #[wrap(GetQuota)]
    GetQuota { s: SessionToken },
}

async fn handle_requests(
    ctx: &Arc<StoreContext>,
    rx: &mut irpc::channel::mpsc::Receiver<ControlMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await? {
        let ctx = Arc::clone(ctx);

        n0_future::task::spawn(async move {
            if let Err(err) = handle_message(ctx, msg).await {
                warn!("Error handling message: {err:?}");
            }
        });
    }

    Ok(())
}

macro_rules! authenticate {
    ($ctx:tt, $inner:tt, $tx:tt) => {
        match $ctx.connections.get_async(&$inner.s).await {
            Some(c) if c.expires > ::time::OffsetDateTime::now_utc().unix_timestamp() => {
                c.did.clone()
            }
            _ => {
                $tx.send(Err($crate::error::ApiError::Unauthenticated))
                    .await?;
                return Ok(());
            }
        }
    };
}

pub(crate) use authenticate;

// n0_future futures are intentionally !Send on wasm (single-threaded, no
// Send needed there); Send-bounded elsewhere.
#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
async fn handle_message(ctx: Arc<StoreContext>, msg: ControlMessage) -> anyhow::Result<()> {
    match msg {
        ControlMessage::UploadBlob(channels) => blob::upload_blob(ctx, channels).await,
        ControlMessage::PinBlob(channels) => blob::pin_blob(ctx, channels).await,
        ControlMessage::BlobExists(channels) => blob::blob_exists(ctx, channels).await,
        ControlMessage::HostDoc(channels) => doc::host_doc(ctx, channels).await,
        ControlMessage::UnhostDoc(channels) => doc::unhost_doc(ctx, channels).await,
        ControlMessage::GetQuota(channels) => quota::get_quota(ctx, channels).await,
    }
}
