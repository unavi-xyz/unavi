//! The `wired/space/1` link: one live QUIC connection per peer, carrying the
//! avatar, object and state streams.

use std::sync::{
    Arc,
    atomic::{
        AtomicU64,
        Ordering,
    },
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router,
};
use hsd::id::{
    DocId,
    PrimId,
};
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        PathId,
    },
};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use unavi_policy::trust::{
    Trust,
    TrustTable,
};
use web_time::Instant;

use crate::{
    authority::SpaceView,
    avatar_sync::{
        RemoteAgent,
        ResolvedPose,
    },
    discovery::{
        Peer,
        PeerPresence,
    },
    identity::LocalIdentity,
    index::Index,
    link::senders::PeerSender,
    object_sync::ResolvedObject,
    replication::Replicas,
};

mod accept;
pub mod codec;
pub mod dial;
pub mod inbox;
pub mod senders;
pub(crate) mod streams;

use inbox::Inbox;

pub const ALPN: &[u8] = b"wired/space/1";

/// A point-in-time bandwidth and latency reading for one peer, pulled from the
/// underlying QUIC connection. Byte counters are cumulative.
#[derive(Clone, Copy)]
pub struct PeerNetStats {
    pub peer:     EndpointId,
    pub bytes_tx: u64,
    pub bytes_rx: u64,
    pub rtt_ms:   f32,
}

/// One peer's connection slot. Dropping `cancel` ends the connection task.
struct Slot {
    token:   u64,
    _cancel: oneshot::Sender<()>,
    conn:    Option<Arc<Connection>>,
}

struct LinkInner {
    view:        SpaceView,
    presence:    PeerPresence,
    slots:       Mutex<HashMap<EndpointId, Slot>>,
    next_token:  AtomicU64,
    next_stream: AtomicU64,
    poses:       Inbox<EndpointId, (Instant, ResolvedPose)>,
    objects:     Inbox<(EndpointId, DocId, PrimId), (Instant, ResolvedObject)>,
}

/// The `wired/space/1` link to other peers: one live connection each, and the
/// handles a connection task needs that it cannot reach through the world.
///
/// Constructed when the iroh endpoint appears, cloned into every spawned task.
#[derive(Resource, Clone)]
pub struct PeerLink(Arc<LinkInner>);

/// A claimed connection slot. Dropping it frees the slot and tears down what
/// the connection spawned in the world, unless a newer connection took the
/// slot over.
pub(crate) struct SlotGuard {
    link:  PeerLink,
    peer:  EndpointId,
    token: u64,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        let mut slots = self.link.0.slots.lock();
        let ours = match slots.get(&self.peer) {
            Some(slot) if slot.token == self.token => {
                slots.remove(&self.peer);
                true
            }
            Some(_) => false,
            None => true,
        };
        drop(slots);
        if ours {
            self.link.end_connection(self.peer);
        }
    }
}

impl PeerLink {
    fn new(view: SpaceView, presence: PeerPresence) -> Self {
        Self(Arc::new(LinkInner {
            view,
            presence,
            slots: Mutex::new(HashMap::new()),
            next_token: AtomicU64::new(0),
            next_stream: AtomicU64::new(0),
            poses: Inbox::default(),
            objects: Inbox::default(),
        }))
    }

    #[must_use]
    pub fn view(&self) -> &SpaceView {
        &self.0.view
    }

    #[must_use]
    pub fn presence(&self) -> &PeerPresence {
        &self.0.presence
    }

    #[must_use]
    pub fn poses(&self) -> &Inbox<EndpointId, (Instant, ResolvedPose)> {
        &self.0.poses
    }

    #[must_use]
    pub fn objects(&self) -> &Inbox<(EndpointId, DocId, PrimId), (Instant, ResolvedObject)> {
        &self.0.objects
    }

    pub(crate) fn next_stream_gen(&self) -> u64 {
        self.0.next_stream.fetch_add(1, Ordering::Relaxed)
    }

    /// Claims the connection slot for `peer`, or `None` if rejected. The
    /// canonical connection (dialed by the greater endpoint id) always wins; a
    /// non-canonical one is kept only when no connection exists, so
    /// one-directional discovery still connects.
    fn claim_connection(
        &self,
        peer: EndpointId,
        canonical: bool,
    ) -> Option<(SlotGuard, oneshot::Receiver<()>)> {
        let mut slots = self.0.slots.lock();
        if !canonical && slots.contains_key(&peer) {
            return None;
        }
        let token = self.0.next_token.fetch_add(1, Ordering::Relaxed);
        let (cancel_tx, cancel_rx) = oneshot::channel();
        slots.insert(
            peer,
            Slot {
                token,
                _cancel: cancel_tx,
                conn: None,
            },
        );
        drop(slots);
        let guard = SlotGuard {
            link: self.clone(),
            peer,
            token,
        };
        Some((guard, cancel_rx))
    }

    /// Records the live connection behind a claimed slot, for stats.
    fn attach(&self, guard: &SlotGuard, conn: Arc<Connection>) {
        if let Some(slot) = self.0.slots.lock().get_mut(&guard.peer)
            && slot.token == guard.token
        {
            slot.conn = Some(conn);
        }
    }

    /// Whether a connection to `peer` is open or being opened.
    #[must_use]
    pub fn is_connected(&self, peer: EndpointId) -> bool {
        self.0.slots.lock().contains_key(&peer)
    }

    /// Drops the live connection to `peer`, if there is one, and forgets the
    /// DID it proved so a reconnect has to prove it again.
    pub fn disconnect(&self, peer: EndpointId) {
        self.0.slots.lock().remove(&peer);
        self.0.view.identity().bindings.unbind(peer);
    }

    /// Whether `peer` is blocked, by the DID it proved over `wired/auth` or by
    /// endpoint for this session.
    #[must_use]
    pub fn is_blocked(&self, peer: EndpointId) -> bool {
        self.0.view.trust_of(peer) == Trust::Blocked
    }

    /// Bandwidth and latency of every live connection.
    #[must_use]
    pub fn net_stats(&self) -> Vec<PeerNetStats> {
        self.0
            .slots
            .lock()
            .iter()
            .filter_map(|(peer, slot)| {
                let conn = slot.conn.as_ref()?;
                let s = conn.stats();
                let rtt_ms = conn
                    .rtt(PathId::ZERO)
                    .map_or(0.0, |d| d.as_secs_f32() * 1000.0);
                Some(PeerNetStats {
                    peer: *peer,
                    bytes_tx: s.udp_tx.bytes,
                    bytes_rx: s.udp_rx.bytes,
                    rtt_ms,
                })
            })
            .collect()
    }

    /// Despawns what a connection to `peer` put in the world: its avatar and
    /// its sender entities.
    fn end_connection(&self, peer: EndpointId) {
        let teardown = move |world: &mut World| {
            if let Some(agent) = world
                .get_resource::<Index<RemoteAgent>>()
                .and_then(|index| index.get(peer))
            {
                world.despawn(agent);
            }
            let senders = world
                .query::<(Entity, &PeerSender)>()
                .iter(world)
                .filter(|(_, s)| s.0 == peer)
                .map(|(e, _)| e)
                .collect::<Vec<_>>();
            for sender in senders {
                world.despawn(sender);
            }
        };
        if self.view().commands().push(teardown).try_send().is_err() {
            let commands = self.view().commands();
            task::spawn(async move {
                let _ = commands.push(teardown).send().await;
            });
        }
    }
}

/// Installs the link once the endpoint and the local identity both exist.
pub fn register_protocol(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    identity: Option<Res<LocalIdentity>>,
    policy: Res<unavi_policy::Policy>,
    replicas: Res<Replicas>,
    trust: Res<TrustTable>,
    presence: Res<PeerPresence>,
    async_world: Res<AsyncWorld>,
    mut commands: Commands,
) {
    let Ok(endpoint) = endpoints.get(trigger.entity) else {
        return;
    };
    let Some(identity) = identity else {
        warn!("Iroh endpoint appeared before the local identity; not installing the space link");
        return;
    };

    let view = SpaceView::new(
        policy.clone(),
        replicas.clone(),
        identity.clone(),
        endpoint.0.id(),
        trust.clone(),
        async_world.clone(),
    );
    let link = PeerLink::new(view.clone(), presence.clone());
    commands.insert_resource(view);
    commands.insert_resource(link.clone());

    commands
        .entity(trigger.entity)
        .queue(router::accept(move |builder| {
            builder.accept(ALPN, accept::SpaceProtocol::new(link))
        }));
}

/// Ends the connection to a peer no longer tracked.
pub fn disconnect_peer(
    trigger: On<Remove, Peer>,
    peers: Query<&Peer>,
    link: Option<Res<PeerLink>>,
) {
    if let (Ok(peer), Some(link)) = (peers.get(trigger.entity), link) {
        link.disconnect(peer.0.id);
    }
}
