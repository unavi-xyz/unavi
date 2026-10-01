//! One connection's streams: tagged by [`StreamKind`], capped per kind, and
//! fanned out to the avatar, object and state handlers.

use std::{
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
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
use n0_future::task::{
    AbortOnDropHandle,
    JoinSet,
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
    debug,
    error,
    info,
    info_span,
};
use xdid::core::did::Did;

use crate::{
    avatar_sync,
    link::{
        PeerLink,
        SlotGuard,
    },
    object_sync,
    replication,
};

/// How long a connection waits for its peer's `wired/auth` handshake before
/// serving it as anonymous.
const AUTH_DEADLINE: Duration = Duration::from_secs(5);

/// Most streams a peer may hold open at once, of each kind.
const MAX_AVATAR_STREAMS: usize = 1;
const MAX_STATE_STREAMS: usize = 1;
const MAX_OBJECT_STREAMS: usize = 64;

/// Most streams a peer may hold open at once, kind not yet read.
const MAX_STREAMS: usize = MAX_AVATAR_STREAMS + MAX_STATE_STREAMS + MAX_OBJECT_STREAMS;

/// What a stream carries, sent as its first byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StreamKind {
    Avatar = 0,
    Object = 1,
    State  = 2,
}

impl TryFrom<u8> for StreamKind {
    type Error = u8;

    fn try_from(byte: u8) -> Result<Self, u8> {
        match byte {
            0 => Ok(Self::Avatar),
            1 => Ok(Self::Object),
            2 => Ok(Self::State),
            other => Err(other),
        }
    }
}

impl StreamKind {
    async fn read(rx: &mut RecvStream) -> anyhow::Result<Self> {
        let byte = rx.read_u8().await?;
        Self::try_from(byte).map_err(|byte| anyhow::anyhow!("unknown stream kind {byte}"))
    }

    pub async fn write(self, tx: &mut SendStream) -> anyhow::Result<()> {
        tx.write_u8(self as u8).await?;
        Ok(())
    }

    const fn limit(self) -> usize {
        match self {
            Self::Avatar => MAX_AVATAR_STREAMS,
            Self::Object => MAX_OBJECT_STREAMS,
            Self::State => MAX_STATE_STREAMS,
        }
    }
}

/// Serves `connection` until it closes, `cancel` fires, or a stream fails.
///
/// Waits for the peer's DID first, so a blocked identity never gets a guest
/// window; a peer that proves none in time is served as anonymous.
pub(super) async fn handle_connection(
    link: &PeerLink,
    slot: &SlotGuard,
    connection: Connection,
    cancel: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let peer = connection.remote_id();
    let did = link
        .view()
        .identity()
        .bindings
        .bound(peer, AUTH_DEADLINE)
        .await;

    if link.is_blocked(peer) {
        info!(%peer, "Refusing a blocked peer");
        connection.close(VarInt::from_u32(2), b"blocked");
        return Ok(());
    }

    let span = info_span!("connect", %peer);
    serve(link, slot, connection, cancel, did)
        .instrument(span)
        .await
}

async fn serve(
    link: &PeerLink,
    slot: &SlotGuard,
    connection: Connection,
    cancel: oneshot::Receiver<()>,
    did: Option<Did>,
) -> anyhow::Result<()> {
    info!(did = ?did.as_ref().map(ToString::to_string), "Connected");
    let connection = Arc::new(connection);
    link.attach(slot, Arc::clone(&connection));

    let task_recv = AbortOnDropHandle::new(n0_future::task::spawn({
        let link = link.clone();
        let connection = Arc::clone(&connection);
        async move { recv_streams(&link, connection, did).await }.instrument(info_span!("recv"))
    }));

    let task_send = AbortOnDropHandle::new(n0_future::task::spawn({
        let link = link.clone();
        let connection = Arc::clone(&connection);
        async move { send_streams(&link, connection).await }.instrument(info_span!("send"))
    }));

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
        res = task_recv => res??,
        res = task_send => res??,
    };

    Ok(())
}

/// Releases a stream kind's slot when its handler ends.
struct KindSlot(Arc<AtomicUsize>);

impl KindSlot {
    fn claim(open: &Arc<AtomicUsize>, limit: usize) -> Option<Self> {
        open.try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
            (n < limit).then_some(n + 1)
        })
        .ok()
        .map(|_| Self(Arc::clone(open)))
    }
}

impl Drop for KindSlot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

async fn recv_streams(
    link: &PeerLink,
    connection: Arc<Connection>,
    did: Option<Did>,
) -> anyhow::Result<()> {
    let peer = connection.remote_id();
    let open: [Arc<AtomicUsize>; 3] = Default::default();
    let mut streams = JoinSet::new();

    loop {
        while let Some(Some(_)) = n0_future::future::now_or_never(streams.join_next()) {}

        let (tx, mut rx) = match connection.accept_bi().await {
            Ok(pair) => pair,
            Err(err) if is_graceful_close(&err) => return Ok(()),
            Err(err) => return Err(err).context("accept_bi"),
        };

        if streams.len() >= MAX_STREAMS {
            debug!(%peer, "stream cap reached; refusing a stream");
            let _ = rx.stop(VarInt::from_u32(1));
            continue;
        }

        let link = link.clone();
        let open = open.clone();
        let did = did.clone();
        streams.spawn(
            async move {
                let kind = match StreamKind::read(&mut rx).await {
                    Ok(kind) => kind,
                    Err(err) => {
                        debug!(?err, "unreadable stream kind");
                        return;
                    }
                };
                let Some(_slot) = KindSlot::claim(&open[kind as usize], kind.limit()) else {
                    debug!(?kind, "stream kind cap reached; refusing a stream");
                    let _ = rx.stop(VarInt::from_u32(1));
                    return;
                };
                if let Err(err) = recv_stream(&link, peer, did, kind, tx, rx).await {
                    error!(?err);
                }
            }
            .instrument(info_span!("stream")),
        );
    }
}

async fn recv_stream(
    link: &PeerLink,
    peer: EndpointId,
    did: Option<Did>,
    kind: StreamKind,
    _tx: SendStream,
    rx: RecvStream,
) -> anyhow::Result<()> {
    match kind {
        StreamKind::Avatar => avatar_sync::wire::recv_agent_stream(link, peer, rx).await,
        StreamKind::Object => object_sync::wire::recv_object_stream(link, peer, rx).await,
        StreamKind::State => replication::wire::recv_state_stream(link, peer, did, rx).await,
    }
}

const STREAM_LOOP_DELAY: Duration = Duration::from_secs(1);

/// Spawns a task running a stream sender again a second after it ends, for as
/// long as the connection lives.
macro_rules! keep_open {
    ($link:expr, $connection:expr, $what:literal, $send:path) => {{
        let link = $link.clone();
        let connection = Arc::clone(&$connection);
        AbortOnDropHandle::new(n0_future::task::spawn(async move {
            loop {
                if let Err(err) = $send(&link, &connection).await {
                    error!(?err, concat!($what, " stream error"));
                }
                n0_future::time::sleep(STREAM_LOOP_DELAY).await;
            }
        }))
    }};
}

async fn send_streams(link: &PeerLink, connection: Arc<Connection>) -> anyhow::Result<()> {
    let tasks = [
        keep_open!(
            link,
            connection,
            "Avatar",
            avatar_sync::wire::send_agent_stream
        ),
        keep_open!(
            link,
            connection,
            "State",
            replication::wire::send_state_stream
        ),
        keep_open!(
            link,
            connection,
            "Object",
            object_sync::wire::send_object_stream
        ),
    ];
    n0_future::join_all(tasks).await;
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

pub fn read_disconnected(err: &std::io::Error) -> bool {
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
