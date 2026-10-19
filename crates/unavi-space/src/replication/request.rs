//! Asking peers for what a space's load dropped.
//!
//! Instancing a space replaces whatever its entity tree held, including the
//! pins, holds and session layers peers had stated there. The peers do not
//! know that, so the node that lost them asks for them again.

use bevy::prelude::*;

use crate::{
    discovery::{
        LastSeenIn,
        Peer,
    },
    membership::Space,
    replication::Replicas,
};

/// Asks every peer heard in a space just instanced here for its state in it.
///
/// A peer heard in the space only afterwards needs no request: the snapshot
/// its state stream opened with covered every space, and its deltas have
/// arrived since. One instancing sends one request per peer, and instancing the
/// space again sends another.
pub fn request_on_space_added(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    peers: Query<(&Peer, &LastSeenIn)>,
    replicas: Res<Replicas>,
) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    for (peer, seen) in &peers {
        if seen.contains(space.id()) && !replicas.request(peer.0.id, space.id()) {
            debug!(peer = %peer.0.id, space = %space.id(), "state request not sent");
        }
    }
}
