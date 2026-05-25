use std::sync::{
    LazyLock,
    Mutex,
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
    peer::Peer,
};

pub mod ecs;
mod inbound;
mod outbound;
mod shared;
mod types;

static CONNECTIONS: LazyLock<Mutex<HashMap<EndpointId, oneshot::Sender<()>>>> =
    LazyLock::new(Mutex::default);

pub const ALPN: &[u8] = b"wired/space/0";

pub fn register_protocol(trigger: On<Add, IrohEndpoint>, mut commands: Commands) {
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

    // Dropping the sender signals the connection task to exit.
    let mut conns = CONNECTIONS.lock().expect("connections lock");
    conns.remove(&peer.0.id);
    drop(conns);

    for (entity, p) in streams {
        if p.0 != peer.0.id {
            continue;
        }
        commands.entity(entity).despawn();
    }
}
