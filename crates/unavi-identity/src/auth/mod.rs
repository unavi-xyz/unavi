//! One mutual DID proof, run ahead of every other protocol.
//!
//! `wired/auth` is dialed by the endpoint's `before_connect` hook ahead of the
//! connection the caller asked for, so protocols behind it need no support of
//! their own: they read the peer's DID out of [`Bindings`]. The proof is
//! mutual, so one handshake identifies both ends.

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

use crate::{
    auth::{
        bindings::Bindings,
        hooks::Hooks,
        outgoing::Outgoing,
        protocol::Protocol,
    },
    identity::Identity,
    resolve::Resolver,
};

pub mod bindings;
pub mod handshake;
pub mod hooks;
pub mod outgoing;
pub mod protocol;

pub const ALPN: &[u8] = b"wired/auth";

/// Bounds the proof, because a dial waits on it.
const PROOF_DEADLINE: Duration = Duration::from_secs(20);
const CLOSE_REFUSED: u32 = 403;

pub enum Message {
    Prove(EndpointId, oneshot::Sender<()>),
    Proved(EndpointId),
}

/// The `wired/auth` wiring for one endpoint.
///
/// One per endpoint, not one per process: a proof names the endpoint it was
/// made to, so the handshake runs over the same endpoint that is about to dial.
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

    /// Safe to call once per bind attempt; only the endpoint that binds ever
    /// uses the hooks.
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
    pub fn serve(&self, endpoint: Endpoint) -> Option<(Protocol, AbortOnDropHandle<()>)> {
        let rx = self.rx.lock().take()?;

        let protocol = Protocol {
            local:    endpoint.id(),
            bindings: Arc::clone(&self.bindings),
            identity: Arc::clone(&self.identity),
            resolver: Arc::clone(&self.resolver),
        };

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
