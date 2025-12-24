//! [`irpc`] WDS user API, for use both locally or as an `iroh` protocol.

use std::sync::Arc;

use blake3::Hash;
use bytes::Bytes;
use irpc::{
    Client, WithChannels,
    channel::{mpsc, oneshot},
    rpc_requests,
};
use irpc_iroh::IrohProtocol;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::error;

use crate::{SessionToken, StoreContext};

mod pin_blob;
mod upload_blob;

pub const ALPN: &[u8] = b"wds/api";

pub(crate) fn protocol(ctx: Arc<StoreContext>) -> (Client<ApiService>, IrohProtocol<ApiService>) {
    let (tx, mut rx) = irpc::channel::mpsc::channel(32);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&ctx, &mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    (client, IrohProtocol::with_sender(local_sender))
}

#[rpc_requests(message = ApiMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiService {
    #[rpc(rx=mpsc::Receiver<Bytes>,tx=oneshot::Sender<Result<Hash, SmolStr>>)]
    #[wrap(UploadBlob)]
    UploadBlob { s: SessionToken },
    #[rpc(tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(PinBlob)]
    PinBlob {
        s: SessionToken,
        hash: Hash,
        expires: i64,
    },
    #[rpc(tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(PinRecord)]
    PinRecord {
        s: SessionToken,
        id: Hash,
        expires: i64,
    },
}

async fn handle_requests(
    ctx: &Arc<StoreContext>,
    rx: &mut irpc::channel::mpsc::Receiver<ApiMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await? {
        let ctx = Arc::clone(ctx);

        tokio::spawn(async move {
            if let Err(e) = handle_message(ctx, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

macro_rules! authenticate {
    ($ctx:tt,$inner:tt,$tx:tt) => {
        match $ctx.connections.get_async(&$inner.s).await {
            Some(c) => c.did.clone(),
            None => {
                $tx.send(Err("unauthenticated".into())).await?;
                return Ok(());
            }
        }
    };
}

pub(crate) use authenticate;

async fn handle_message(ctx: Arc<StoreContext>, msg: ApiMessage) -> anyhow::Result<()> {
    match msg {
        ApiMessage::UploadBlob(channels) => {
            upload_blob::upload_blob(ctx, channels).await?;
        }
        ApiMessage::PinBlob(channels) => {
            pin_blob::pin_blob(ctx, channels).await?;
        }
        ApiMessage::PinRecord(WithChannels { inner, tx, .. }) => {
            let _did = authenticate!(ctx, inner, tx);
            todo!()
        }
    }

    Ok(())
}
