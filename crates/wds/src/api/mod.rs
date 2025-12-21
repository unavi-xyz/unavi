//! [`irpc`] WDS API, for use both locally or as an `iroh` protocol.

use std::{sync::Arc, time::Duration};

use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};
use irpc_iroh::IrohProtocol;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::error;

use crate::ConnectionState;

pub const ALPN: &[u8] = b"wds/api";

pub fn protocol(connection: Arc<ConnectionState>) -> IrohProtocol<ApiService> {
    let (tx, mut rx) = irpc::channel::mpsc::channel(8);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&connection, &mut rx).await {
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
    #[rpc(tx=oneshot::Sender<Result<String, SmolStr>>)]
    #[wrap(CreateRecord)]
    CreateRecord,
}

async fn handle_requests(
    connection: &Arc<ConnectionState>,
    rx: &mut irpc::channel::mpsc::Receiver<ApiMessage>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await? {
        let connection = Arc::clone(connection);

        tokio::spawn(async move {
            if let Err(e) = handle_message(connection, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

async fn handle_message(connection: Arc<ConnectionState>, msg: ApiMessage) -> anyhow::Result<()> {
    match msg {
        ApiMessage::CreateRecord(WithChannels { tx, .. }) => {
            if connection.authentication.get().is_none() {
                tx.send(Err("unauthenticated".into())).await?;
                return Ok(());
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}
