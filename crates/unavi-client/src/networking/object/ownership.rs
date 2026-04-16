use bevy::prelude::*;

use crate::networking::{
    object::publish::{DynObjectId, LocallyOwned},
    player::{LocalPlayerState, OwnedObjectEntry},
};

// TODO replace with observers

/// When `LocallyOwned` is added to a dynobject, create an `OwnedObjectEntry`
/// child on the `LocalPlayerState` entity.
pub fn on_locally_claimed(
    mut commands: Commands,
    local_player: Query<Entity, With<LocalPlayerState>>,
    added: Query<&DynObjectId, Added<LocallyOwned>>,
) {
    let Ok(player_entity) = local_player.single() else {
        return;
    };

    for dyn_id in &added {
        let entry = commands
            .spawn(OwnedObjectEntry {
                record_id: dyn_id.0.record,
                node_id: dyn_id.0.node.clone(),
            })
            .id();
        commands.entity(player_entity).add_child(entry);
    }
}

/// When `LocallyOwned` is removed, despawn the matching `OwnedObjectEntry` child.
pub fn on_locally_released(
    mut commands: Commands,
    local_player: Query<(Entity, &Children), With<LocalPlayerState>>,
    mut removed: RemovedComponents<LocallyOwned>,
    dyn_objects: Query<&DynObjectId>,
    entries: Query<(Entity, &OwnedObjectEntry)>,
) {
    for removed_entity in removed.read() {
        let Ok(dyn_id) = dyn_objects.get(removed_entity) else {
            continue;
        };

        let Ok((_, children)) = local_player.single() else {
            continue;
        };

        for &child in children {
            if let Ok((entry_entity, entry)) = entries.get(child)
                && entry.record_id == dyn_id.0.record
                && entry.node_id == dyn_id.0.node
            {
                commands.entity(entry_entity).despawn();
                break;
            }
        }
    }
}
