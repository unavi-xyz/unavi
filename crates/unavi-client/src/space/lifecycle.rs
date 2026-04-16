use std::collections::HashMap;

use bevy::prelude::*;
use blake3::Hash;

use crate::{
    networking::thread::{NetworkCommand, NetworkingThread},
    space::Space,
};

/// Marker placed on a space entity to request joining that space.
/// Adding it → networking thread joins gossip.
/// Removing it (or despawning the entity) → networking thread leaves.
#[derive(Component, Debug)]
pub struct JoinedSpace;

/// Watch for newly joined spaces and send the Join command.
pub fn on_space_joined(
    added: Query<(Entity, &Space), Added<JoinedSpace>>,
    nt: Res<NetworkingThread>,
    mut cache: Local<HashMap<Entity, Hash>>,
) {
    for (entity, space) in &added {
        cache.insert(entity, space.0);
        if nt
            .command_tx
            .try_send(NetworkCommand::Join(space.0))
            .is_err()
        {
            warn!("failed to send Join for space {}", space.0);
        }
    }
}

/// Watch for removed `JoinedSpace` (explicit removal or entity despawn) and
/// send the Leave command. Uses a local cache because the component value is
/// gone by the time `RemovedComponents` fires.
pub fn on_space_left(
    mut removed: RemovedComponents<JoinedSpace>,
    nt: Res<NetworkingThread>,
    mut cache: Local<HashMap<Entity, Hash>>,
) {
    for entity in removed.read() {
        if let Some(hash) = cache.remove(&entity)
            && nt.command_tx.try_send(NetworkCommand::Leave(hash)).is_err()
        {
            warn!("failed to send Leave for space {hash}");
        }
    }
}
