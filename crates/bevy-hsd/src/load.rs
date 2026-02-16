use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    Layer, LayerEnabled, NodeId, Opinion, OpinionAttrs, OpinionTarget, Stage, StageCompiled,
    StageLayers, StageLoaded, StageNode, StageNodes,
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
        let mut node_ents: HashMap<String, Entity> = HashMap::new();

        for layer_data in &stage.0.layers {
            // Spawn layer entity.
            let layer_ent = commands
                .spawn((Layer { stage: stage_ent }, LayerEnabled(layer_data.enabled)))
                .id();

            // Spawn opinions.
            for (node_id, attrs) in layer_data.opinions.iter() {
                let node_ent = node_ents.entry(node_id.clone()).or_insert_with(|| {
                    commands
                        .spawn((StageNode { stage: stage_ent }, NodeId(node_id.into())))
                        .id()
                });

                let Some(attrs) = attrs.as_map() else {
                    // Invalid type.
                    continue;
                };

                commands.spawn((
                    Opinion { layer: layer_ent },
                    OpinionTarget(*node_ent),
                    OpinionAttrs(attrs.clone()),
                ));
            }
        }

        loaded.0 = true;
        compiled.0 = false;
    }
}
