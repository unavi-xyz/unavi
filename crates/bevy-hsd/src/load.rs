use std::collections::HashMap;

use bevy::prelude::*;
use loro::TreeID;

use crate::{Layer, Stage, StageCompiled, StageLayers, StageLoaded};

pub fn load_stages(
    stages: Query<(Entity, &mut StageLoaded, &mut StageCompiled, &Stage)>,
    // layers: Query<(&LayerStrength, &LayerOpinions)>,
    // opinions: Query<(&OpinionTarget, Option<&OpinionOp<Xform>>)>,
    mut commands: Commands,
) {
    for (stage_ent, mut loaded, mut compiled, stage) in stages {
        if loaded.0 {
            continue;
        }

        // Delete old layers.
        commands.entity(stage_ent).despawn_related::<StageLayers>();

        // Create new layers.
        let mut node_ents: HashMap<TreeID, Entity> = HashMap::new();

        for layer_data in &stage.0.layers {
            // Spawn layer entity.
            let _layer_ent = commands.spawn(Layer { stage: stage_ent }).id();

            // Spawn node entities.
            for node in &layer_data.nodes {
                if node_ents.contains_key(&node.id) {
                    // Node already created on an earlier layer.
                    continue;
                }

                let ent = commands.spawn(()).id();

                // Use actual parent from tree structure.
                if let Some(parent_id) = node.parent
                    && let Some(parent_ent) = node_ents.get(&parent_id)
                {
                    commands.entity(*parent_ent).add_child(ent);
                }

                node_ents.insert(node.id, ent);
            }

            // Apply opinions.
        }

        loaded.0 = true;
        compiled.0 = false;
    }
}
