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
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{
        RouterBuilderFn,
        RouterBuilderFnTarget,
    },
};
use hsd::id::{
    DocId,
    PrimId,
};
use iroh::EndpointId;
#[cfg(feature = "devtools")]
use iroh::endpoint::{
    Connection,
    PathId,
};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use unavi_policy::trust::Trust;
use unavi_util::async_task::spawn_async_task;
use web_time::Instant;

use crate::{
    connection::ecs::{
        PeerStream,
        agent::inbound::ResolvedPose,
        object::ResolvedObject,
    },
    identity::LocalIdentity,
    inbox::Inbox,
    peer::Peer,
    state::replicas::Replicas,
    view::SpaceView,
};

pub mod ecs;
mod inbound;
mod outbound;
mod shared;
mod types;

/// A point-in-time bandwidth and latency reading for one peer, pulled from the
/// underlying QUIC connection. Byte counters are cumulative; the panel diffs
/// them.
#[cfg(feature = "devtools")]
#[derive(Clone, Copy)]
pub struct PeerNetStats {
    pub peer:     EndpointId,
    pub bytes_tx: u64,
    pub bytes_rx: u64,
    pub rtt_ms:   f32,
}

/// Untracks a connection from the dev tools network panel when it drops with
/// the connection task.
#[cfg(feature = "devtools")]
pub struct ConnGuard {
    link: PeerLink,
    peer: EndpointId,
    conn: Arc<Connection>,
}

#[cfg(feature = "devtools")]
impl Drop for ConnGuard {
    fn drop(&mut self) {
        let mut live = self.link.0.live.lock();
        // Clear only the tracked connection: a superseded duplicate must not
        // evict the live connection that replaced it under the same peer id.
        if live
            .get(&self.peer)
            .is_some_and(|c| Arc::ptr_eq(c, &self.conn))
        {
            live.remove(&self.peer);
        }
    }
}

struct LinkInner {
    view:        SpaceView,
    connections: Mutex<HashMap<EndpointId, (u64, oneshot::Sender<()>)>>,
    next_token:  AtomicU64,
    next_stream: AtomicU64,
    poses:       Inbox<EndpointId, (Instant, ResolvedPose)>,
    objects:     Inbox<(EndpointId, DocId, PrimId), (Instant, ResolvedObject)>,
    #[cfg(feature = "devtools")]
    live:        Mutex<HashMap<EndpointId, Arc<Connection>>>,
}

/// The `wired/space/1` link to other peers: one live connection each, and the
/// handles a connection task needs that it cannot reach through the world.
///
/// Constructed when the iroh endpoint appears, cloned into every spawned task.
#[derive(Resource, Clone)]
pub struct PeerLink(Arc<LinkInner>);

impl PeerLink {
    fn new(view: SpaceView) -> Self {
        Self(Arc::new(LinkInner {
            view,
            connections: Mutex::new(HashMap::new()),
            next_token: AtomicU64::new(0),
            next_stream: AtomicU64::new(0),
            poses: Inbox::new(),
            objects: Inbox::new(),
            #[cfg(feature = "devtools")]
            live: Mutex::new(HashMap::new()),
        }))
    }

    #[must_use]
    pub fn view(&self) -> &SpaceView {
        &self.0.view
    }

    #[must_use]
    pub fn poses(&self) -> &Inbox<EndpointId, (Instant, ResolvedPose)> {
        &self.0.poses
    }

    #[must_use]
    pub fn objects(&self) -> &Inbox<(EndpointId, DocId, PrimId), (Instant, ResolvedObject)> {
        &self.0.objects
    }

    #[must_use]
    pub fn next_stream_gen(&self) -> u64 {
        self.0.next_stream.fetch_add(1, Ordering::Relaxed)
    }

    /// Claims the connection slot for `peer`, or `None` if rejected. The
    /// canonical connection (dialed by the greater endpoint id) always wins; a
    /// non-canonical one is kept only when no connection exists, so
    /// one-directional discovery (peeking through a portal) still connects.
    fn claim_connection(
        &self,
        peer: EndpointId,
        canonical: bool,
    ) -> Option<(u64, oneshot::Receiver<()>)> {
        let mut conns = self.0.connections.lock();
        if !canonical && conns.contains_key(&peer) {
            return None;
        }
        let token = self.0.next_token.fetch_add(1, Ordering::Relaxed);
        let (cancel_tx, cancel_rx) = oneshot::channel();
        conns.insert(peer, (token, cancel_tx));
        drop(conns);
        Some((token, cancel_rx))
    }

    /// Clears the connection slot for `peer`, returning whether this token
    /// still held it; a superseded connection must not run per-peer teardown.
    fn release_connection(&self, peer: EndpointId, token: u64) -> bool {
        let mut conns = self.0.connections.lock();
        if conns.get(&peer).is_some_and(|(t, _)| *t == token) {
            conns.remove(&peer);
            drop(conns);
            self.0.view.identity().bindings.unbind(peer);
            true
        } else {
            false
        }
    }

    /// Drops the live connection to `peer`, if there is one.
    pub fn disconnect(&self, peer: EndpointId) {
        // Dropping the cancel sender is what the connection task waits on; the
        // recv stream ending despawns the peer's `RemotePeer` entity.
        self.0.connections.lock().remove(&peer);
        self.0.view.identity().bindings.unbind(peer);
    }

    /// Whether the DID `peer` proved over `wired/auth` is blocked. An endpoint
    /// that proved none is a guest, so this refuses nobody by default.
    #[must_use]
    pub fn is_blocked(&self, peer: EndpointId) -> bool {
        self.0.view.trust_of(Some(peer)) == Trust::Blocked
    }

    /// Tracks a connection for the dev tools network panel, untracking it when
    /// the returned guard drops with the connection task.
    #[cfg(feature = "devtools")]
    #[must_use]
    pub fn track(&self, peer: EndpointId, conn: Arc<Connection>) -> ConnGuard {
        self.0.live.lock().insert(peer, Arc::clone(&conn));
        ConnGuard {
            link: self.clone(),
            peer,
            conn,
        }
    }

    #[cfg(feature = "devtools")]
    #[must_use]
    pub fn net_stats(&self) -> Vec<PeerNetStats> {
        self.0
            .live
            .lock()
            .iter()
            .map(|(peer, conn)| {
                let s = conn.stats();
                let rtt_ms = conn
                    .rtt(PathId::ZERO)
                    .map_or(0.0, |d| d.as_secs_f32() * 1000.0);
                PeerNetStats {
                    peer: *peer,
                    bytes_tx: s.udp_tx.bytes,
                    bytes_rx: s.udp_rx.bytes,
                    rtt_ms,
                }
            })
            .collect()
    }
}

pub const ALPN: &[u8] = b"wired/space/1";

pub fn register_protocol(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    identity: Option<Res<LocalIdentity>>,
    policy: Res<unavi_policy::registry::Policy>,
    replicas: Res<Replicas>,
    mut commands: Commands,
) {
    let Ok(endpoint) = endpoints.get(trigger.entity) else {
        return;
    };
    let Some(identity) = identity else {
        warn!("Iroh endpoint appeared before the local identity; not installing the space link");
        return;
    };

    let me = endpoint.0.id();
    let view = SpaceView::new(policy.clone(), replicas.clone(), identity.clone(), me);
    let link = PeerLink::new(view.clone());
    commands.insert_resource(view);
    commands.insert_resource(link.clone());

    commands.spawn((
        RouterBuilderFn(Some(Box::new(move |builder| {
            builder.accept(ALPN, inbound::SpaceProtocol::new(link))
        }))),
        RouterBuilderFnTarget(trigger.entity),
    ));
}

pub fn connect_to_peer(
    trigger: On<Add, Peer>,
    peers: Query<&Peer>,
    endpoint: Query<&IrohEndpoint>,
    link: Option<Res<PeerLink>>,
) {
    let Ok(endpoint) = endpoint.single().map(|e| e.0.clone()) else {
        warn!("Unable to connect to peer: no endpoint");
        return;
    };
    let Some(link) = link else {
        warn!("Unable to connect to peer: space link not installed yet");
        return;
    };

    let peer = peers
        .get(trigger.entity)
        .map(|p| p.0.clone())
        .expect("peer");

    let link = link.clone();
    spawn_async_task(async move {
        outbound::try_open_connection(link, endpoint, peer).await;
    });
}

pub fn disconnect_peer(
    trigger: On<Remove, Peer>,
    peers: Query<&Peer>,
    streams: Query<(Entity, &PeerStream)>,
    link: Option<Res<PeerLink>>,
    mut commands: Commands,
) {
    let peer = peers.get(trigger.entity).expect("peer");

    // Dropping the sender exits the connection task; the ending recv stream
    // despawns the `RemotePeer`, releasing the peer's state.
    if let Some(link) = link {
        link.disconnect(peer.0.id);
    }

    for (entity, p) in streams {
        if p.0 != peer.0.id {
            continue;
        }
        commands.entity(entity).despawn();
    }
}
