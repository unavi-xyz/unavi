use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    Compiled, LayerOpinions, LayerStrength, OpinionOp, OpinionTarget, Stage, StageLayers,
    attributes::xform::Xform,
};

#[derive(Default)]
struct NodeAttrs {
    xform: Option<OpinionOp<Xform>>,
}

pub fn compile_stages(
    stages: Query<(&mut Compiled, &StageLayers), With<Stage>>,
    layers: Query<(&LayerStrength, &LayerOpinions)>,
    opinions: Query<(&OpinionTarget, Option<&OpinionOp<Xform>>)>,
    mut commands: Commands,
) {
    for (mut compiled, stage_layers) in stages {
        if compiled.0 {
            continue;
        }

        // Read layers.
        let mut resolved_layers = Vec::with_capacity(stage_layers.len());

        for layer_ent in stage_layers.iter() {
            let Ok((strength, layer_opinions)) = layers.get(layer_ent) else {
                continue;
            };

            resolved_layers.push((strength.0, layer_opinions.0.clone()));
        }

        resolved_layers.sort_by(|a, b| a.0.cmp(&b.0));
        info!(?resolved_layers);

        // Merge layer opinions.
        let mut node_attrs = HashMap::<Entity, NodeAttrs>::new();

        for (_, layer_opinions) in resolved_layers {
            for opinion_ent in layer_opinions {
                let Ok((target, xform)) = opinions.get(opinion_ent) else {
                    continue;
                };

                let node = node_attrs.entry(target.0).or_default();

                if let Some(next) = xform {
                    node.xform = Some(
                        node.xform
                            .take()
                            .map_or_else(|| next.clone(), |v| v.merge(next)),
                    );
                }
            }
        }

        // Apply node attributes.
        for (node_ent, attrs) in node_attrs {
            let mut node = commands.entity(node_ent);

            if let Some(OpinionOp::Set(xform)) = attrs.xform {
                node.insert(xform.into_transform());
            } else {
                node.remove::<Transform>();
            }
        }

        compiled.0 = true;
    }
}
