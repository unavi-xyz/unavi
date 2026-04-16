use std::collections::HashSet;

use bevy::prelude::*;
use blake3::Hash;

use crate::{
    networking::{
        peer::{Peer, PeerKnownSpaces},
        thread::{NetworkCommand, NetworkingThread},
    },
    space::{JoinedSpace, Space},
};

/// After any space is left, disconnect peers that no longer share any space
/// with us. Called in `PostUpdate` so it runs after `on_space_left` has sent
/// the Leave command and after the `JoinedSpace` cache is updated.
pub fn check_orphan_peers(
    nt: Res<NetworkingThread>,
    mut commands: Commands,
    peers: Query<(Entity, &Peer, &PeerKnownSpaces)>,
    joined: Query<&Space, With<JoinedSpace>>,
    mut removed_spaces: RemovedComponents<JoinedSpace>,
) {
    // Only run if a space was left this frame.
    if removed_spaces.read().count() == 0 {
        return;
    }

    let our_spaces: HashSet<Hash> = joined.iter().map(|s| s.0).collect();

    for (entity, peer, known) in &peers {
        if known.0.is_disjoint(&our_spaces) {
            info!(id = %peer.0, "disconnecting orphan peer (no shared spaces)");
            let _ = nt
                .command_tx
                .try_send(NetworkCommand::DisconnectPeer(peer.0));
            commands.entity(entity).despawn();
        }
    }
}
