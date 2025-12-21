//! [`irpc`] WDS API, for use both locally or as an `iroh` protocol.

use irpc::{Client, WithChannels, channel::oneshot, rpc::RemoteService, rpc_requests};
use irpc_iroh::IrohProtocol;
use serde::{Deserialize, Serialize};
use tracing::error;

pub const ALPN: &[u8] = b"wds/api";

pub fn new_protocol() -> IrohProtocol<ApiService> {
    let (tx, mut rx) = irpc::channel::mpsc::channel(16);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    let handler = ApiService::remote_handler(local_sender);

    IrohProtocol::new(handler)
}

#[rpc_requests(message = ApiMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ApiService {
    #[rpc(tx=oneshot::Sender<i64>)]
    #[wrap(Multiply)]
    Multiply(i64, i64),
}

async fn handle_requests(rx: &mut irpc::channel::mpsc::Receiver<ApiMessage>) -> anyhow::Result<()> {
    while let Some(req) = rx.recv().await? {
        match req {
            ApiMessage::Multiply(WithChannels { inner, tx, .. }) => {
                tx.send(inner.0 * inner.1).await?;
            }
        }
    }

    Ok(())
}
