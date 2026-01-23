use std::collections::HashMap;

use bevy::prelude::*;
use smol_str::SmolStr;

use crate::{
    Layer, Opinion, OpinionOp, OpinionTarget, Stage, StageCompiled, StageLayers, StageLoaded,
};

pub fn load_stages(
    stages: Query<(Entity, &mut StageLoaded, &mut StageCompiled, &Stage)>,
    mut commands: Commands,
) {
    for (stage_ent, mut loaded, mut compiled, stage) in stages {
        if loaded.0 {
            continue;
        }

        // Delete old layers.
        commands.entity(stage_ent).despawn_related::<StageLayers>();

        // Create new layers.
        let mut node_ents: HashMap<SmolStr, Entity> = HashMap::new();

        for layer_data in &stage.0.layers {
            // Spawn layer entity.
            let layer_ent = commands.spawn(Layer { stage: stage_ent }).id();

            // Spawn node entities.
            for node in &layer_data.nodes {
                if node_ents.contains_key(&node.meta.id) {
                    // Node already created on an earlier layer.
                    continue;
                }

                let ent = commands.spawn(()).id();

                // Use actual parent from tree structure.
                if let Some(parent_id) = node.parent
                    && let Some(parent_node) = layer_data.nodes.iter().find(|n| n.id == parent_id)
                    && let Some(parent_ent) = node_ents.get(&parent_node.meta.id)
                {
                    commands.entity(*parent_ent).add_child(ent);
                }

                node_ents.insert(node.meta.id.clone(), ent);
            }

            // Spawn opinions.
            for opinion in &layer_data.opinions {
                let Some(node_ent) = node_ents.get(&opinion.id) else {
                    continue;
                };

                let mut opinion_ent =
                    commands.spawn((Opinion { layer: layer_ent }, OpinionTarget(*node_ent)));

                if let Some(material) = &opinion.attrs.material {
                    opinion_ent.insert(OpinionOp(material.clone()));
                }
                if let Some(mesh) = &opinion.attrs.mesh {
                    opinion_ent.insert(OpinionOp(mesh.clone()));
                }
                if let Some(xform) = &opinion.attrs.xform {
                    opinion_ent.insert(OpinionOp(xform.clone()));
                }
            }
        }

        loaded.0 = true;
        compiled.0 = false;
    }
}
