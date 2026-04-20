use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    networking::{
        peer::{Peer, PeerKnownSpaces},
        thread::{NetworkCommand, NetworkingThread},
    },
    space::Space,
};

pub fn on_space_add(trigger: On<Add, Space>, nt: Res<NetworkingThread>, spaces: Query<&Space>) {
    let space = spaces.get(trigger.entity).expect("has component");

    if nt
        .command_tx
        .try_send(NetworkCommand::JoinSpace(space.0))
        .is_err()
    {
        warn!(id = %space.0, "failed to send Join for space");
    }
}

pub fn on_space_remove(
    trigger: On<Remove, Space>,
    nt: Res<NetworkingThread>,
    spaces: Query<&Space>,
    peers: Query<(Entity, &Peer, &PeerKnownSpaces)>,
    mut commands: Commands,
) {
    let space = spaces.get(trigger.entity).expect("has component");

    if nt
        .command_tx
        .try_send(NetworkCommand::LeaveSpace(space.0))
        .is_err()
    {
        warn!(id = %space.0, "failed to send Leave for space");
    }

    let our_spaces = spaces.iter().map(|s| s.0).collect::<HashSet<_>>();

    for (ent, peer, peer_spaces) in peers {
        if !peer_spaces.0.is_disjoint(&our_spaces) {
            continue;
        }
        info!(id = %peer.0, "disconnecting orphan peer (no shared spaces)");
        let _ = nt
            .command_tx
            .try_send(NetworkCommand::DisconnectPeer(peer.0));
        commands.entity(ent).despawn();
    }
}
