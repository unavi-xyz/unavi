use std::{sync::Arc, time::Duration};

use anyhow::bail;
use iroh::endpoint::{Connection, ConnectionError, RecvStream, SendStream, VarInt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Notify,
};
use tracing::{Instrument, error, info, info_span};

mod agent;
mod object;
mod types;

pub async fn handle_connection(connection: Connection, cancel: &Arc<Notify>) -> anyhow::Result<()> {
    let peer = connection.remote_id();
    let span = info_span!("connect", %peer);
    inner(connection, cancel).instrument(span).await
}

async fn inner(connection: Connection, cancel: &Arc<Notify>) -> anyhow::Result<()> {
    info!("Connected");
    let connection = Arc::new(connection);

    let task_recv = {
        let span = info_span!("recv");
        let connection = Arc::clone(&connection);
        n0_future::task::spawn(async move { recv_streams(connection).await }.instrument(span))
    };

    let task_send = {
        let span = info_span!("send");
        let connection = Arc::clone(&connection);
        n0_future::task::spawn(async move { send_streams(connection).await }.instrument(span))
    };

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
        res = task_recv => {
            res??;
        },
        res = task_send => {
            res??;
        },
    };

    Ok(())
}

async fn recv_streams(connection: Arc<Connection>) -> anyhow::Result<()> {
    let mut i = 0;

    loop {
        let span = info_span!("stream", i);
        i += 1;

        let (tx, rx) = connection.accept_bi().await?;

        n0_future::task::spawn(
            async move {
                if let Err(err) = recv_stream(tx, rx).await {
                    error!(?err);
                }
            }
            .instrument(span),
        );
    }
}

async fn recv_stream(tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    let ident = StreamIdent::read(&mut rx).await?;

    match ident {
        StreamIdent::Agent => agent::recv_agent_stream(tx, rx).await?,
        StreamIdent::Object => object::recv_object_stream(tx, rx).await?,
    }

    Ok(())
}

async fn send_streams(connection: Arc<Connection>) -> anyhow::Result<()> {
    // Spawn agent sender
    {
        let connection = Arc::clone(&connection);
        n0_future::task::spawn(async move {
            loop {
                if let Err(err) = agent::send_agent_stream(&connection).await {
                    error!(?err, "agent sender");
                }
            }
        });
    }

    // Manage object senders
    loop {
        // TODO recv commands from ecs

        n0_future::time::sleep(Duration::from_mins(1)).await;
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum StreamIdent {
    Agent,
    Object,
}

impl StreamIdent {
    async fn read(rx: &mut RecvStream) -> anyhow::Result<Self> {
        let buf = [rx.read_u8().await?];
        let ident = postcard::from_bytes::<Self>(&buf)?;
        Ok(ident)
    }

    async fn write(&self, tx: &mut SendStream) -> anyhow::Result<()> {
        let mut buf = [0u8];
        postcard::to_slice(self, &mut buf)?;
        tx.write_u8(buf[0]).await?;
        Ok(())
    }
}
