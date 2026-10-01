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

use crate::{
    auth::{
        ALPN,
        Message,
        PROOF_DEADLINE,
        bindings::Bindings,
        handshake::Handshake,
    },
    identity::Identity,
    resolver::Resolver,
};

/// How long a remote that proved nothing is dialed without retrying the proof,
/// so a peer that does not speak `wired/auth` costs one deadline, not one per
/// dial.
const FAILURE_MEMORY: Duration = Duration::from_mins(1);

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
        let mut waiting = HashMap::<EndpointId, Vec<oneshot::Sender<()>>>::new();
        let mut failed = HashMap::<EndpointId, Instant>::new();

        while let Some(message) = rx.recv().await {
            match message {
                Message::Prove(remote, reply) if self.bindings.is_bound(remote) => {
                    reply.send(()).ok();
                }
                Message::Prove(remote, reply)
                    if failed
                        .get(&remote)
                        .is_some_and(|at| at.elapsed() < FAILURE_MEMORY) =>
                {
                    reply.send(()).ok();
                }
                Message::Prove(remote, reply) => match waiting.entry(remote) {
                    Entry::Occupied(mut entry) => entry.get_mut().push(reply),
                    Entry::Vacant(entry) => {
                        entry.insert(vec![reply]);
                        self.spawn_proof(remote);
                    }
                },
                Message::Proved(remote, proved) => {
                    if proved {
                        failed.remove(&remote);
                    } else {
                        failed.retain(|_, at| at.elapsed() < FAILURE_MEMORY);
                        failed.insert(remote, Instant::now());
                    }
                    for reply in waiting.remove(&remote).into_iter().flatten() {
                        reply.send(()).ok();
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
            let proof = exchange(&endpoint, remote, &bindings, &identity, &resolver);

            let proved = match n0_future::time::timeout(PROOF_DEADLINE, proof).await {
                Ok(Ok(())) => true,
                Ok(Err(err)) => {
                    debug!(?err, %remote, "peer proved no identity");
                    false
                }
                Err(_) => {
                    debug!(%remote, "identity proof timed out");
                    false
                }
            };

            tx.send(Message::Proved(remote, proved)).await.ok();
        });
    }
}

async fn exchange(
    endpoint: &Endpoint,
    remote: EndpointId,
    bindings: &Bindings,
    identity: &Identity,
    resolver: &Resolver,
) -> anyhow::Result<()> {
    let connection = endpoint.connect(remote, ALPN).await?;
    let (mut tx, mut rx) = connection.open_bi().await?;

    let handshake = Handshake {
        identity,
        resolver,
        local: endpoint.id(),
        remote,
    };

    let did = handshake.dial(&mut tx, &mut rx).await?;
    bindings.bind(remote, did);

    connection.close(VarInt::from_u32(0), b"done");
    Ok(())
}
