use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    Layer, Opinion, OpinionAttrs, OpinionTarget, Stage, StageCompiled, StageLayers, StageLoaded,
    StageNode, StageNodes,
};

pub fn load_stages(
    stages: Query<(Entity, &mut StageLoaded, &mut StageCompiled, &Stage)>,
    mut commands: Commands,
) {
    for (stage_ent, mut loaded, mut compiled, stage) in stages {
        if loaded.0 {
            continue;
        }

        // Clear ECS.
        commands.entity(stage_ent).despawn_related::<StageLayers>();
        commands.entity(stage_ent).despawn_related::<StageNodes>();

        // Create new layers.
        let mut node_ents: HashMap<i64, Entity> = HashMap::new();

        for layer_data in &stage.0.layers {
            // Spawn layer entity.
            let layer_ent = commands.spawn(Layer { stage: stage_ent }).id();

            // Spawn opinions.
            for opinion in &layer_data.opinions {
                let node_ent = node_ents
                    .entry(opinion.node)
                    .or_insert_with(|| commands.spawn(StageNode { stage: stage_ent }).id());

                commands.spawn((
                    Opinion { layer: layer_ent },
                    OpinionTarget(*node_ent),
                    OpinionAttrs(opinion.attrs.clone()),
                ));
            }
        }

        loaded.0 = true;
        compiled.0 = false;
    }
}
