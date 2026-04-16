use std::collections::HashSet;

use bevy::prelude::*;
use blake3::Hash;

use crate::{
    networking::{
        peer::{Peer, PeerKnownSpaces, PeerStateStatus},
        thread::{NetworkCommand, NetworkingThread},
    },
    space::{JoinedSpace, Space},
};

/// Send `RequestPeerState` for peers we share a space with but haven't synced yet.
pub fn request_peer_state(
    nt: Res<NetworkingThread>,
    joined: Query<&Space, With<JoinedSpace>>,
    mut peers: Query<(&Peer, &PeerKnownSpaces, &mut PeerStateStatus)>,
) {
    let our_spaces: HashSet<Hash> = joined.iter().map(|s| s.0).collect();
    if our_spaces.is_empty() {
        return;
    }

    for (peer, known, mut status) in &mut peers {
        if *status != PeerStateStatus::NeverSynced {
            continue;
        }
        if !known.0.is_disjoint(&our_spaces) {
            let _ = nt
                .command_tx
                .try_send(NetworkCommand::RequestPeerState(peer.0));
            *status = PeerStateStatus::Requested;
        }
    }
}
