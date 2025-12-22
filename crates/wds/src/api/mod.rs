//! [`irpc`] WDS API, for use both locally or as an `iroh` protocol.

use std::sync::Arc;

use bytes::Bytes;
use irpc::{
    Client, WithChannels,
    channel::{mpsc, oneshot},
    rpc_requests,
};
use irpc_iroh::IrohProtocol;
use loro::LoroDoc;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::error;

use crate::ConnectionState;

mod pin_blob;
mod tag;
mod upload_blob;

pub const ALPN: &[u8] = b"wds/api";

pub fn protocol(conn: Arc<ConnectionState>) -> IrohProtocol<ApiService> {
    let (tx, mut rx) = irpc::channel::mpsc::channel(8);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&conn, &mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    IrohProtocol::with_sender(local_sender)
}

#[rpc_requests(message = ApiMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiService {
    #[rpc(tx=oneshot::Sender<Result<blake3::Hash, SmolStr>>)]
    #[wrap(CreateRecord)]
    CreateRecord,
    #[rpc(rx=mpsc::Receiver<Bytes>,tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(UploadBlob)]
    UploadBlob { hash: blake3::Hash, byte_len: usize },
    #[rpc(tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(PinBlob)]
    PinBlob { hash: blake3::Hash, expires: i64 },
    #[rpc(tx=oneshot::Sender<Result<(), SmolStr>>)]
    #[wrap(PinRecord)]
    PinRecord { id: blake3::Hash, expires: i64 },
}

async fn handle_requests(
    conn: &Arc<ConnectionState>,
    rx: &mut irpc::channel::mpsc::Receiver<ApiMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await? {
        let conn = Arc::clone(conn);

        tokio::spawn(async move {
            if let Err(e) = handle_message(conn, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

macro_rules! authenticate {
    ($conn:tt,$tx:tt) => {
        match $conn.authentication.get() {
            Some(did) => did,
            None => {
                $tx.send(Err("unauthenticated".into())).await?;
                return Ok(());
            }
        }
    };
}

pub(crate) use authenticate;

async fn handle_message(conn: Arc<ConnectionState>, msg: ApiMessage) -> anyhow::Result<()> {
    match msg {
        ApiMessage::CreateRecord(WithChannels { tx, .. }) => {
            let _did = authenticate!(conn, tx);

            let _doc = LoroDoc::new();

            todo!()
        }
        ApiMessage::UploadBlob(channels) => {
            upload_blob::upload_blob(conn, channels).await?;
        }
        ApiMessage::PinBlob(channels) => {
            pin_blob::pin_blob(conn, channels).await?;
        }
        ApiMessage::PinRecord(WithChannels { inner: _, tx, .. }) => {
            let _did = authenticate!(conn, tx);
            todo!()
        }
    }

    Ok(())
}
