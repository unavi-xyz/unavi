use std::{sync::Arc, time::Duration};

use anyhow::bail;
use iroh::endpoint::{Connection, ConnectionError, Side, VarInt};
use tokio::sync::Notify;
use tracing::{Instrument, info, info_span};

pub async fn handle_connection(
    connection: &Connection,
    cancel: &Arc<Notify>,
) -> anyhow::Result<()> {
    let peer = connection.remote_id();
    let span = info_span!("connect", %peer);
    inner(connection, cancel).instrument(span).await
}

async fn inner(connection: &Connection, cancel: &Arc<Notify>) -> anyhow::Result<()> {
    tokio::select! {
        () = cancel.notified() => {
            connection.close(VarInt::from_u32(0), b"done");
            n0_future::time::sleep(Duration::from_secs(5)).await;
        },
        err = connection.closed() => {
            match err {
                ConnectionError::ConnectionClosed(reason) => {
                    info!("Peer closed connection: {reason}");
                }
                ConnectionError::LocallyClosed => {
                    info!("Closed connection");
                }
                err => {
                    bail!("connection error: {err:?}")
                }
            }
        },
        res = manage_connection(connection) => {
            res?;
        },
    };

    Ok(())
}

async fn manage_connection(connection: &Connection) -> anyhow::Result<()> {
    info!("Connected");

    let (_tx, _rx) = match connection.side() {
        Side::Client => connection.open_bi().await?,
        Side::Server => connection.accept_bi().await?,
    };

    Ok(())
}
