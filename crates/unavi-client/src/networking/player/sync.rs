use bevy::prelude::*;

use crate::networking::{
    player::{LocalPlayerState, OwnedObjectEntry, PlayerStateDirty},
    thread::{
        NetworkCommand, NetworkingThread,
        space::types::state::{ObjectStateEntry, PlayerStateMsg},
    },
};

/// Mark `LocalPlayerState` dirty when owned objects are added or removed.
pub fn detect_local_changes(
    mut commands: Commands,
    local_player: Query<Entity, With<LocalPlayerState>>,
    added: Query<Entity, Added<OwnedObjectEntry>>,
    mut removed: RemovedComponents<OwnedObjectEntry>,
) {
    let Ok(entity) = local_player.single() else {
        return;
    };

    let has_change = !added.is_empty() || removed.read().count() > 0;
    if has_change {
        commands.entity(entity).insert(PlayerStateDirty);
    }
}

/// Serialize current state and send `UpdateLocalState` to the networking thread.
pub fn broadcast_state_delta(
    mut commands: Commands,
    nt: Res<NetworkingThread>,
    local_player: Query<(Entity, &Children), (With<LocalPlayerState>, With<PlayerStateDirty>)>,
    entries: Query<&OwnedObjectEntry>,
) {
    let Ok((entity, children)) = local_player.single() else {
        return;
    };

    let objects: Vec<ObjectStateEntry> = children
        .iter()
        .filter_map(|child| entries.get(child).ok())
        .map(|e| ObjectStateEntry {
            record_id: *e.record_id.as_bytes(),
            node_id: e.node_id.clone(),
        })
        .collect();

    let msg = PlayerStateMsg {
        objects,
        portals: Vec::new(),
    };

    let _ = nt
        .command_tx
        .try_send(NetworkCommand::UpdateLocalState(msg));
    commands.entity(entity).remove::<PlayerStateDirty>();
}
