use std::sync::{
    LazyLock,
    Mutex,
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
use iroh::EndpointId;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;

use crate::{
    connection::ecs::PeerStream,
    identity::bindings,
    peer::Peer,
};

pub mod ecs;
mod inbound;
mod outbound;
mod shared;
mod types;

/// One live connection per peer, keyed by endpoint, each tagged with a unique
/// token so a connection only ever clears its own slot.
static CONNECTIONS: LazyLock<Mutex<HashMap<EndpointId, (u64, oneshot::Sender<()>)>>> =
    LazyLock::new(Mutex::default);

static CONN_TOKEN: AtomicU64 = AtomicU64::new(0);

/// Claims the connection slot for `peer`, or `None` if rejected. The canonical
/// connection (dialed by the greater endpoint id) always wins; a non-canonical
/// one is kept only when no connection exists, so one-directional discovery
/// (peeking through a portal) still connects.
fn claim_connection(peer: EndpointId, canonical: bool) -> Option<(u64, oneshot::Receiver<()>)> {
    let mut conns = CONNECTIONS.lock().expect("connections lock");
    if !canonical && conns.contains_key(&peer) {
        return None;
    }
    let token = CONN_TOKEN.fetch_add(1, Ordering::Relaxed);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    conns.insert(peer, (token, cancel_tx));
    drop(conns);
    Some((token, cancel_rx))
}

/// Clears the connection slot for `peer`, returning whether this token still
/// held it; a superseded connection must not run per-peer teardown.
fn release_connection(peer: EndpointId, token: u64) -> bool {
    let mut conns = CONNECTIONS.lock().expect("connections lock");
    if conns.get(&peer).is_some_and(|(t, _)| *t == token) {
        conns.remove(&peer);
        drop(conns);
        if let Some(bindings) = bindings() {
            bindings.unbind(peer);
        }
        true
    } else {
        false
    }
}

/// Drops the live connection to `peer`, if there is one.
pub fn disconnect(peer: EndpointId) {
    // Dropping the cancel sender is what the connection task waits on; the
    // recv stream ending despawns the peer's `RemotePeer` entity.
    CONNECTIONS.lock().expect("connections lock").remove(&peer);
    if let Some(bindings) = bindings() {
        bindings.unbind(peer);
    }
}

/// Whether the DID `peer` proved over `wired/auth` is blocked. An endpoint
/// that proved none is a guest, so this refuses nobody by default.
pub fn is_blocked(peer: EndpointId) -> bool {
    bindings().is_some_and(|bindings| {
        unavi_policy::trust::of_peer(*peer.as_bytes(), &bindings)
            == unavi_policy::trust::Trust::Blocked
    })
}

pub const ALPN: &[u8] = b"wired/space/1";

pub fn register_protocol(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    if let Ok(endpoint) = endpoints.get(trigger.entity) {
        crate::peer::set_self_peer_id(*endpoint.0.id().as_bytes());
    }
    commands.spawn((
        RouterBuilderFn(Some(Box::new(|builder| {
            builder.accept(ALPN, inbound::SpaceProtocol)
        }))),
        RouterBuilderFnTarget(trigger.entity),
    ));
}

pub fn connect_to_peer(
    trigger: On<Add, Peer>,
    peers: Query<&Peer>,
    endpoint: Query<&IrohEndpoint>,
) {
    let Ok(endpoint) = endpoint.single().map(|e| e.0.clone()) else {
        warn!("Unable to connect to peer: no endpoint");
        return;
    };

    let peer = peers
        .get(trigger.entity)
        .map(|p| p.0.clone())
        .expect("peer");

    spawn_async_task(async move {
        outbound::try_open_connection(endpoint, peer).await;
    });
}

pub fn disconnect_peer(
    trigger: On<Remove, Peer>,
    peers: Query<&Peer>,
    streams: Query<(Entity, &PeerStream)>,
    mut commands: Commands,
) {
    let peer = peers.get(trigger.entity).expect("peer");

    // Dropping the sender exits the connection task; the ending recv stream
    // despawns the `RemotePeer`, releasing the peer's state.
    let mut conns = CONNECTIONS.lock().expect("connections lock");
    conns.remove(&peer.0.id);
    drop(conns);
    if let Some(bindings) = bindings() {
        bindings.unbind(peer.0.id);
    }

    for (entity, p) in streams {
        if p.0 != peer.0.id {
            continue;
        }
        commands.entity(entity).despawn();
    }
}
