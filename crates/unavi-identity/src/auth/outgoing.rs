//! The dialing side of `wired/auth`, run once per remote however many dials
//! wait on it.

use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    sync::Arc,
    time::Duration,
};

use iroh::{
    Endpoint,
    EndpointId,
    endpoint::VarInt,
};
use n0_future::time::Instant;
use tokio::sync::{
    mpsc,
    oneshot,
};
use tracing::debug;
use xdid::core::did::Did;

use crate::{
    auth::{
        ALPN,
        CLOSE_REFUSED,
        DIAL_DEADLINE,
        Message,
        PROOF_DEADLINE,
        Proof,
        bindings::Bindings,
        handshake::Handshake,
    },
    identity::Identity,
    resolver::Resolver,
};

/// How long a remote that refused a proof is left alone, so a peer that does
/// not speak `wired/auth` costs one deadline, not one per dial. Only a refusal
/// is remembered: an unreachable peer is retried on the next dial.
const FAILURE_MEMORY: Duration = Duration::from_mins(1);

/// Peers that answered and refused a proof, for [`FAILURE_MEMORY`]. A peer
/// that could not be reached is not one of them: nothing was learned about it.
#[derive(Default)]
struct Refusals(HashMap<EndpointId, Instant>);

impl Refusals {
    fn remembered(&self, peer: EndpointId) -> bool {
        self.0
            .get(&peer)
            .is_some_and(|at| at.elapsed() < FAILURE_MEMORY)
    }

    fn record(&mut self, peer: EndpointId, proof: Proof) {
        if proof == Proof::Refused {
            self.0.retain(|_, at| at.elapsed() < FAILURE_MEMORY);
            self.0.insert(peer, Instant::now());
        } else {
            self.0.remove(&peer);
        }
    }
}

/// Hooks are installed on the endpoint builder, before the endpoint exists, so
/// they cannot hold it themselves; this task does the dialing for them.
pub struct Outgoing {
    pub endpoint: Endpoint,
    pub tx:       mpsc::Sender<Message>,
    pub bindings: Arc<Bindings>,
    pub identity: Arc<Identity>,
    pub resolver: Arc<Resolver>,
}

impl Outgoing {
    pub async fn run(self, mut rx: mpsc::Receiver<Message>) {
        let mut waiting = HashMap::<EndpointId, Vec<oneshot::Sender<Option<Did>>>>::new();
        let mut refused = Refusals::default();

        while let Some(message) = rx.recv().await {
            match message {
                Message::Prove(remote, reply) if self.bindings.is_bound(remote) => {
                    reply.send(self.bindings.did_of(remote)).ok();
                }
                Message::Prove(remote, reply) if refused.remembered(remote) => {
                    reply.send(None).ok();
                }
                Message::Prove(remote, reply) => match waiting.entry(remote) {
                    Entry::Occupied(mut entry) => entry.get_mut().push(reply),
                    Entry::Vacant(entry) => {
                        entry.insert(vec![reply]);
                        self.spawn_proof(remote);
                    }
                },
                Message::Proved(remote, proof) => {
                    refused.record(remote, proof);
                    let did = self.bindings.did_of(remote);
                    for reply in waiting.remove(&remote).into_iter().flatten() {
                        reply.send(did.clone()).ok();
                    }
                }
            }
        }
    }

    fn spawn_proof(&self, remote: EndpointId) {
        let endpoint = self.endpoint.clone();
        let tx = self.tx.clone();
        let bindings = Arc::clone(&self.bindings);
        let identity = Arc::clone(&self.identity);
        let resolver = Arc::clone(&self.resolver);

        n0_future::task::spawn(async move {
            let proof = prove(&endpoint, remote, &bindings, &identity, &resolver).await;
            tx.send(Message::Proved(remote, proof)).await.ok();
        });
    }
}

/// Proves `remote` to itself and to us, bounding the dial and the handshake
/// separately: reaching a peer never dialed before is discovery, not proof.
async fn prove(
    endpoint: &Endpoint,
    remote: EndpointId,
    bindings: &Bindings,
    identity: &Identity,
    resolver: &Resolver,
) -> Proof {
    let connection =
        match n0_future::time::timeout(DIAL_DEADLINE, endpoint.connect(remote, ALPN)).await {
            Ok(Ok(connection)) => connection,
            Ok(Err(err)) => {
                debug!(?err, %remote, "peer not reachable");
                return Proof::Unreachable;
            }
            Err(_) => {
                debug!(%remote, "dial timed out");
                return Proof::Unreachable;
            }
        };

    let exchange = async {
        let (mut tx, mut rx) = connection.open_bi().await?;
        let handshake = Handshake {
            identity,
            resolver,
            local: endpoint.id(),
            remote,
        };

        handshake.dial(&mut tx, &mut rx).await
    };

    match n0_future::time::timeout(PROOF_DEADLINE, exchange).await {
        Ok(Ok(did)) => {
            bindings.bind(remote, did.clone());
            debug!(%did, %remote, "peer identified");
            connection.close(VarInt::from_u32(0), b"done");
            Proof::Proved
        }
        Ok(Err(err)) => {
            debug!(?err, %remote, "peer proved no identity");
            connection.close(VarInt::from_u32(CLOSE_REFUSED), b"proof refused");
            Proof::Refused
        }
        Err(_) => {
            debug!(%remote, "identity proof timed out");
            connection.close(VarInt::from_u32(CLOSE_REFUSED), b"proof timed out");
            Proof::Refused
        }
    }
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    /// A peer that never answered says nothing about its DID, so a later dial
    /// must still prove it. Only a peer that answered and refused is left
    /// alone, and only for a while.
    #[test]
    fn only_a_refusal_is_remembered() {
        let mut refusals = Refusals::default();
        let unreachable = SecretKey::generate().public();
        let refusing = SecretKey::generate().public();

        refusals.record(unreachable, Proof::Unreachable);
        refusals.record(refusing, Proof::Refused);

        assert!(!refusals.remembered(unreachable));
        assert!(refusals.remembered(refusing));

        refusals.record(refusing, Proof::Proved);
        assert!(!refusals.remembered(refusing));
    }
}
