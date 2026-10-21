//! A mutual DID proof, run ahead of every other protocol.
//!
//! The endpoint's `before_connect` hook dials `wired/auth` ahead of the
//! connection the caller asked for. Protocols behind it need no support of
//! their own, since they read the peer's DID out of [`Bindings`]. One exchange
//! proves both ends.

use std::{
    sync::Arc,
    time::Duration,
};

use iroh::{
    Endpoint,
    EndpointId,
    endpoint::Builder,
};
use n0_future::task::AbortOnDropHandle;
use parking_lot::Mutex;
use tokio::sync::{
    mpsc,
    oneshot,
};
use xdid::core::did::Did;

pub use crate::auth::{
    bindings::Bindings,
    protocol::AuthProtocol,
};
use crate::{
    auth::{
        hooks::Hooks,
        outgoing::Outgoing,
    },
    identity::Identity,
    resolver::Resolver,
};

pub mod bindings;
mod handshake;
mod hooks;
mod outgoing;
mod protocol;

pub const ALPN: &[u8] = b"wired/auth";

/// Bounds the handshake itself, in either direction.
const PROOF_DEADLINE: Duration = Duration::from_secs(20);

/// Bounds reaching a peer whose addresses are not known here yet. The first
/// dial to a peer waits on relay and holepunch discovery, which is not the
/// handshake running late and must not be charged to it.
const DIAL_DEADLINE: Duration = Duration::from_secs(120);

/// How long a dial waits for its peer's proof before going ahead unbound.
/// Every dial triggers one, and a caller that needs the DID asks for it
/// directly, so a cold peer must not hold the dial.
const BIND_WAIT: Duration = Duration::from_secs(3);

const CLOSE_REFUSED: u32 = 403;

/// How long the accepting end holds a proven connection open for the peer to
/// read its verdict. The peer should close as soon as it has read it; this only
/// bounds a peer that never does.
const CLOSE_GRACE: Duration = Duration::from_secs(5);

pub(crate) enum Message {
    /// Prove `peer`, answering with the DID it proved once the attempt settles.
    /// A peer already bound answers from the binding.
    Prove(EndpointId, oneshot::Sender<Option<Did>>),
    /// A handshake finished.
    Proved(EndpointId, Proof),
}

/// How an outgoing proof ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Proof {
    /// The peer answered and proved a DID.
    Proved,
    /// The peer answered and its proof did not hold. Retrying it at once is
    /// pointless, so the attempt is remembered for a while.
    Refused,
    /// The peer could not be reached. Nothing is known about it, so the next
    /// dial tries again.
    Unreachable,
}

/// The `wired/auth` wiring for one endpoint.
///
/// A proof names the endpoint it was made to, so the handshake has to run over
/// the same endpoint that is about to dial.
pub struct EndpointAuth {
    tx:       mpsc::Sender<Message>,
    rx:       Mutex<Option<mpsc::Receiver<Message>>>,
    bindings: Arc<Bindings>,
    identity: Arc<Identity>,
    resolver: Arc<Resolver>,
}

impl EndpointAuth {
    #[must_use]
    pub fn new(identity: Arc<Identity>, resolver: Arc<Resolver>) -> Self {
        let (tx, rx) = mpsc::channel(16);

        Self {
            tx,
            rx: Mutex::new(Some(rx)),
            bindings: Arc::default(),
            identity,
            resolver,
        }
    }

    /// The DIDs peers have proven to this endpoint. Everything above the
    /// transport reads a peer's identity from here, never from a claim made
    /// elsewhere.
    #[must_use]
    pub const fn bindings(&self) -> &Arc<Bindings> {
        &self.bindings
    }

    /// Proves `peer` and answers the DID it proved, reusing a live binding.
    ///
    /// `None` means it proved nothing before `deadline`: unreachable, or its
    /// proof did not hold. Reads are open to anyone, so that is anonymity
    /// rather than an error, and the next attempt starts afresh. Callers that
    /// judge a peer by its DID ask here rather than read [`Bindings`], whose
    /// entries end with the peer's last connection.
    pub async fn prove(&self, peer: EndpointId, deadline: Duration) -> Option<Did> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Message::Prove(peer, tx)).await.ok()?;
        n0_future::time::timeout(deadline, rx).await.ok()?.ok()?
    }

    /// Installs the hooks that dial `wired/auth` ahead of an outgoing
    /// connection and track how long each peer stays connected, which is how
    /// long its binding lasts.
    #[must_use]
    pub fn install(&self, builder: Builder) -> Builder {
        builder.hooks(Hooks {
            tx:       self.tx.clone(),
            bindings: Arc::clone(&self.bindings),
        })
    }

    /// Answers `wired/auth` on `endpoint` and pre-authenticates the outgoing
    /// connections the hooks intercept. Both stop when the guard drops.
    ///
    /// Returns `None` if this endpoint is already served.
    #[must_use]
    pub fn serve(&self, endpoint: Endpoint) -> Option<(AuthProtocol, AbortOnDropHandle<()>)> {
        let rx = self.rx.lock().take()?;

        let protocol = AuthProtocol::new(
            endpoint.id(),
            Arc::clone(&self.bindings),
            Arc::clone(&self.identity),
            Arc::clone(&self.resolver),
        );

        let outgoing = Outgoing {
            endpoint,
            tx: self.tx.clone(),
            bindings: Arc::clone(&self.bindings),
            identity: Arc::clone(&self.identity),
            resolver: Arc::clone(&self.resolver),
        };

        let handle = n0_future::task::spawn(outgoing.run(rx));
        Some((protocol, AbortOnDropHandle::new(handle)))
    }
}
