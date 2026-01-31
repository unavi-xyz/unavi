use bevy::prelude::*;
use bevy_wds::{BlobDeps, BlobDepsLoaded};

use crate::{StageNode, StageNodes, StageNodesLoaded};

pub fn stage_nodes_loaded(
    mut commands: Commands,
    loading: Query<(Entity, &StageNodes), Without<StageNodesLoaded>>,
    unloaded_nodes: Query<(), (With<StageNode>, With<BlobDeps>, Without<BlobDepsLoaded>)>,
) {
    for (ent, deps) in loading {
        let has_unloaded = deps
            .0
            .iter()
            .any(|dep_ent| unloaded_nodes.contains(*dep_ent));

        if !has_unloaded {
            commands.entity(ent).insert(StageNodesLoaded);
        }
    }
}
