use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::{
    Context,
    bail,
};
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        ConnectionError,
        RecvStream,
        SendStream,
        VarInt,
    },
};
use n0_future::task::AbortOnDropHandle;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    sync::oneshot,
};
use tracing::{
    Instrument,
    error,
    info,
    info_span,
};

mod agent;
mod identity;
mod object;
mod state;

pub async fn handle_connection(
    connection: Connection,
    cancel: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let peer = connection.remote_id();
    let span = info_span!("connect", %peer);
    inner(connection, cancel).instrument(span).await
}

async fn inner(connection: Connection, cancel: oneshot::Receiver<()>) -> anyhow::Result<()> {
    info!("Connected");
    let connection = Arc::new(connection);

    #[cfg(feature = "devtools")]
    let _conn_guard = crate::devtools::conn::track(connection.remote_id(), Arc::clone(&connection));

    let task_recv = {
        let span = info_span!("recv");
        let connection = Arc::clone(&connection);
        let handle =
            n0_future::task::spawn(async move { recv_streams(connection).await }.instrument(span));
        AbortOnDropHandle::new(handle)
    };

    let task_send = {
        let span = info_span!("send");
        let connection = Arc::clone(&connection);
        let handle =
            n0_future::task::spawn(async move { send_streams(connection).await }.instrument(span));
        AbortOnDropHandle::new(handle)
    };

    tokio::select! {
        _ = cancel => {
            connection.close(VarInt::from_u32(0), b"done");
            n0_future::time::sleep(Duration::from_secs(5)).await;
        },
        err = connection.closed() => {
            if is_graceful_close(&err) {
                info!("Connection closed: {err}");
            } else {
                bail!("connection error: {err:?}")
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
    let peer = connection.remote_id();
    let mut i = 0;
    let mut streams = Vec::new();

    loop {
        let span = info_span!("stream", i);
        i += 1;

        let (tx, rx) = match connection.accept_bi().await {
            Ok(pair) => pair,
            Err(err) if is_graceful_close(&err) => return Ok(()),
            Err(err) => return Err(err).context("accept_bi"),
        };

        let handle = n0_future::task::spawn(
            async move {
                if let Err(err) = recv_stream(peer, tx, rx).await {
                    error!(?err);
                }
            }
            .instrument(span),
        );
        streams.push(AbortOnDropHandle::new(handle));
    }
}

async fn recv_stream(peer: EndpointId, tx: SendStream, mut rx: RecvStream) -> anyhow::Result<()> {
    let ident = StreamIdent::read(&mut rx).await.context("read ident")?;
    info!("Stream ident: {ident:?}");

    match ident {
        StreamIdent::Identity => identity::prove_self_identity(peer, tx, rx).await?,
        StreamIdent::Agent => agent::recv_agent_stream(peer, tx, rx).await?,
        StreamIdent::Object => object::recv_object_stream(peer, tx, rx).await?,
        StreamIdent::State => state::recv_state_stream(peer, tx, rx).await?,
        StreamIdent::Unknown(_) => {}
    }

    Ok(())
}

const STREAM_LOOP_DELAY: Duration = Duration::from_secs(1);

async fn send_streams(connection: Arc<Connection>) -> anyhow::Result<()> {
    let task_identity = {
        let connection = Arc::clone(&connection);
        let handle =
            n0_future::task::spawn(async move { identity::verify_peer_identity(&connection).await });
        AbortOnDropHandle::new(handle)
    };

    let task_agent = {
        let connection = Arc::clone(&connection);
        let handle = n0_future::task::spawn(async move {
            loop {
                if let Err(err) = agent::send_agent_stream(&connection).await {
                    error!(?err, "Agent stream error");
                }
                n0_future::time::sleep(STREAM_LOOP_DELAY).await;
            }
        });
        AbortOnDropHandle::new(handle)
    };

    let task_state = {
        let connection = Arc::clone(&connection);
        let handle = n0_future::task::spawn(async move {
            loop {
                if let Err(err) = state::send_state_stream(&connection).await {
                    error!(?err, "State stream error");
                }
                n0_future::time::sleep(STREAM_LOOP_DELAY).await;
            }
        });
        AbortOnDropHandle::new(handle)
    };

    let task_objects = {
        let connection = Arc::clone(&connection);
        let handle = n0_future::task::spawn(async move {
            loop {
                if let Err(err) = object::send_object_stream(&connection).await {
                    error!(?err, "Object stream error");
                }
                n0_future::time::sleep(STREAM_LOOP_DELAY).await;
            }
        });
        AbortOnDropHandle::new(handle)
    };

    n0_future::join_all([task_identity, task_agent, task_state, task_objects]).await;

    Ok(())
}

const fn is_graceful_close(err: &ConnectionError) -> bool {
    matches!(
        err,
        ConnectionError::ConnectionClosed(_)
            | ConnectionError::LocallyClosed
            | ConnectionError::ApplicationClosed(_)
    )
}

fn read_disconnected(err: &std::io::Error) -> bool {
    use std::io::ErrorKind::{
        BrokenPipe,
        ConnectionAborted,
        ConnectionReset,
        NotConnected,
        UnexpectedEof,
    };
    matches!(
        err.kind(),
        UnexpectedEof | ConnectionReset | ConnectionAborted | NotConnected | BrokenPipe
    )
}

#[derive(Serialize, Deserialize, Debug)]
#[non_exhaustive]
enum StreamIdent {
    Identity,
    Agent,
    Object,
    State,
    Unknown(usize),
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
