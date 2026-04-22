use bevy::prelude::*;
use bevy_iroh::endpoint::IrohEndpoint;
use unavi_util::async_task::spawn_async_task;

use crate::Peer;

pub fn connect_to_peers(
    trigger: On<Add, Peer>,
    peers: Query<&Peer>,
    endpoint: Query<&IrohEndpoint>,
) {
    let peer = peers.get(trigger.entity).expect("peer");

    let Ok(endpoint) = endpoint.single().map(|e| e.0.clone()) else {
        warn!("Unable to connect to peer: no endpoint");
        return;
    };

    spawn_async_task(async move {
        // TODO
    });
}
