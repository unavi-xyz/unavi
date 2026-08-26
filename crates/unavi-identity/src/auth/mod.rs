//! One mutual DID proof, run ahead of every other protocol.
//!
//! `wired/auth` is dialed by the endpoint's `before_connect` hook before the
//! connection a caller actually asked for, so protocols behind it need no
//! support of their own: they read the peer's DID out of [`binding`]. The proof
//! is mutual, so one handshake identifies both ends, and the accepting side
//! knows who called it without any per-message token.

use std::{
    collections::{
        HashMap,
        HashSet,
        hash_map::Entry,
    },
    sync::Arc,
    time::Duration,
};

use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
    endpoint::{
        AfterHandshakeOutcome,
        BeforeConnectOutcome,
        Builder,
        Connection,
        EndpointHooks,
        VarInt,
    },
    protocol::{
        AcceptError,
        ProtocolHandler,
    },
};
use n0_future::task::AbortOnDropHandle;
use parking_lot::Mutex;
use tokio::sync::{
    mpsc,
    oneshot,
};
use tracing::debug;

pub mod binding;
pub mod handshake;

pub const ALPN: &[u8] = b"wired/auth";

/// Retried because a peer whose own identity is still loading cannot answer
/// yet.
const ATTEMPTS: u32 = 5;
const RETRY_DELAY: Duration = Duration::from_secs(1);
/// Generous, since answering means resolving a DID that may be `did:web`.
const TIMEOUT: Duration = Duration::from_secs(15);

const CLOSE_REFUSED: u32 = 403;

/// ALPNs that refuse a peer which has proven no DID.
///
/// Deliberately empty: doc and blob sync, and a registry's read side, must stay
/// reachable with no account at all, and a blanket rule at the connection layer
/// would take that away. Reach and trust are enforced above the transport.
const GATED_ALPNS: &[&[u8]] = &[];

/// The `wired/auth` wiring for one endpoint.
///
/// One per endpoint, not one per process: a proof names the endpoint it was
/// made to, so the handshake runs over the same endpoint that is about to dial.
/// The binding table it fills is process-wide.
pub struct EndpointAuth {
    tx: mpsc::Sender<Message>,
    rx: Mutex<Option<mpsc::Receiver<Message>>>,
}

impl EndpointAuth {
    #[must_use]
    pub fn new() -> Arc<Self> {
        let (tx, rx) = mpsc::channel(16);
        Arc::new(Self {
            tx,
            rx: Mutex::new(Some(rx)),
        })
    }

    /// Installs the hooks on an endpoint builder. Safe to call once per bind
    /// attempt; only the endpoint that binds ever uses them.
    #[must_use]
    pub fn install(&self, builder: Builder) -> Builder {
        builder.hooks(Hooks {
            tx: self.tx.clone(),
        })
    }

    /// Answers `wired/auth` on `endpoint` and pre-authenticates the outgoing
    /// connections the hooks intercept. Both stop when the guard drops.
    ///
    /// Returns `None` if this endpoint is already served.
    #[must_use]
    pub fn serve(&self, endpoint: Endpoint) -> Option<(Protocol, AbortOnDropHandle<()>)> {
        let rx = self.rx.lock().take()?;
        let protocol = Protocol {
            local: endpoint.id(),
        };
        let handle = n0_future::task::spawn(run_outgoing(endpoint, self.tx.clone(), rx));
        Some((protocol, AbortOnDropHandle::new(handle)))
    }
}

/// Intercepts connection establishment so a peer is identified before the
/// protocol that wanted it runs.
#[derive(Debug)]
struct Hooks {
    tx: mpsc::Sender<Message>,
}

impl EndpointHooks for Hooks {
    async fn before_connect<'a>(
        &'a self,
        remote_addr: &'a EndpointAddr,
        alpn: &'a [u8],
    ) -> BeforeConnectOutcome {
        if alpn == ALPN || binding::is_bound(remote_addr.id) {
            return BeforeConnectOutcome::Accept;
        }

        // Always accepted, proven or not: reads are open to anyone, so a peer
        // that answers nothing is anonymous rather than refused. What a proven
        // DID unlocks is decided above the transport, and on the accept side by
        // `GATED_ALPNS`.
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(Message::Prove(remote_addr.id, tx))
            .await
            .is_ok()
        {
            rx.await.ok();
        }

        BeforeConnectOutcome::Accept
    }

    fn after_handshake<'a>(
        &'a self,
        conn: &'a Connection,
    ) -> impl Future<Output = AfterHandshakeOutcome> + Send + 'a {
        let alpn = conn.alpn();
        let outcome = if alpn == ALPN
            || !GATED_ALPNS.contains(&alpn)
            || binding::is_bound(conn.remote_id())
        {
            AfterHandshakeOutcome::accept()
        } else {
            AfterHandshakeOutcome::Reject {
                error_code: VarInt::from_u32(CLOSE_REFUSED),
                reason:     b"identity not proven".to_vec(),
            }
        };
        std::future::ready(outcome)
    }
}

/// Answers `wired/auth`. Registered on the router under [`ALPN`].
#[derive(Debug, Clone)]
pub struct Protocol {
    local: EndpointId,
}

impl ProtocolHandler for Protocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote = connection.remote_id();
        let (mut tx, mut rx) = connection.accept_bi().await?;

        match handshake::accept(&mut tx, &mut rx, self.local, remote).await {
            Ok(did) => {
                debug!(%did, "peer identified");
                binding::bind(*remote.as_bytes(), did);
            }
            Err(err) => {
                debug!(?err, "peer proved no identity");
                connection.close(VarInt::from_u32(CLOSE_REFUSED), b"proof refused");
            }
        }

        Ok(())
    }
}

enum Message {
    Prove(EndpointId, oneshot::Sender<bool>),
    Proved(EndpointId, bool),
}

/// Owns the endpoint the hooks may not hold, running one handshake per remote
/// and fanning its result out to every dial waiting on it.
async fn run_outgoing(
    endpoint: Endpoint,
    tx: mpsc::Sender<Message>,
    mut rx: mpsc::Receiver<Message>,
) {
    let mut proven = HashSet::new();
    let mut waiting: HashMap<EndpointId, Vec<oneshot::Sender<bool>>> = HashMap::new();

    while let Some(message) = rx.recv().await {
        match message {
            Message::Prove(remote, reply) if proven.contains(&remote) => {
                reply.send(true).ok();
            }
            Message::Prove(remote, reply) => match waiting.entry(remote) {
                Entry::Occupied(mut entry) => entry.get_mut().push(reply),
                Entry::Vacant(entry) => {
                    entry.insert(vec![reply]);
                    let endpoint = endpoint.clone();
                    let tx = tx.clone();
                    n0_future::task::spawn(async move {
                        let ok = prove(&endpoint, remote).await;
                        tx.send(Message::Proved(remote, ok)).await.ok();
                    });
                }
            },
            Message::Proved(remote, ok) => {
                if ok {
                    proven.insert(remote);
                }
                for reply in waiting.remove(&remote).into_iter().flatten() {
                    reply.send(ok).ok();
                }
            }
        }
    }
}

/// Whether `remote` proved a DID over a fresh `wired/auth` connection.
async fn prove(endpoint: &Endpoint, remote: EndpointId) -> bool {
    for attempt in 1..=ATTEMPTS {
        let run = n0_future::time::timeout(TIMEOUT, exchange(endpoint, remote))
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("identity proof timed out")));

        match run {
            Ok(()) => return true,
            Err(err) if attempt == ATTEMPTS => debug!(?err, %remote, "peer proved no identity"),
            Err(err) => {
                debug!(?err, "identity proof failed, retrying");
                n0_future::time::sleep(RETRY_DELAY).await;
            }
        }
    }

    false
}

async fn exchange(endpoint: &Endpoint, remote: EndpointId) -> anyhow::Result<()> {
    let connection = endpoint.connect(remote, ALPN).await?;
    let (mut tx, mut rx) = connection.open_bi().await?;

    let did = handshake::dial(&mut tx, &mut rx, endpoint.id(), remote).await?;
    binding::bind(*remote.as_bytes(), did);

    connection.close(VarInt::from_u32(0), b"done");
    Ok(())
}
