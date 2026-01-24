use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    LayerOpinions, LayerStrength, OpinionOp, OpinionTarget, Stage, StageCompiled, StageLayers,
    attributes::{Attribute, xform::Xform},
    stage::Attrs,
};

pub fn compile_stages(
    stages: Query<(&mut StageCompiled, &StageLayers), With<Stage>>,
    layers: Query<(&LayerStrength, &LayerOpinions)>,
    opinions: Query<(&OpinionTarget, Option<&OpinionOp<Xform>>)>,
    asset_server: ResMut<AssetServer>,
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
        debug_assert!(
            resolved_layers.first().map(|(i, _)| *i).unwrap_or_default()
                <= resolved_layers.last().map(|(i, _)| *i).unwrap_or_default(),
            "layers not sorted in correct order"
        );

        // Merge layer opinions.
        let mut node_attrs = HashMap::<Entity, Attrs>::new();

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
                            .map_or_else(|| next.0.clone(), |v| v.merge(&next.0)),
                    );
                }
            }
        }

        // Apply node attributes.
        #[expect(clippy::cast_possible_truncation)]
        for (node_ent, attrs) in node_attrs {
            let mut node = commands.entity(node_ent);

            if let Some(mat) = attrs.material {
                // TODO node as asset

                let mut out_mat = StandardMaterial::default();

                if let Some(color) = mat.color {
                    out_mat.base_color = Color::Srgba(Srgba {
                        red: color[0] as f32,
                        green: color[1] as f32,
                        blue: color[2] as f32,
                        alpha: color[3] as f32,
                    });
                }

                out_mat.metallic = mat.metallic as f32;
                out_mat.perceptual_roughness = mat.roughness as f32;

                let handle = asset_server.add(out_mat);
                node.insert(MeshMaterial3d(handle));
            } else {
                node.remove::<MeshMaterial3d<StandardMaterial>>();
            }

            if let Some(_mesh) = attrs.mesh {
                // TODO lazy load node asset
            } else {
                node.remove::<Mesh3d>();
            }

            if let Some(xform) = attrs.xform {
                node.insert(xform.into_transform());
            } else {
                node.remove::<Transform>();
            }
        }

        compiled.0 = true;
    }
}
