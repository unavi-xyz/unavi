use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;
use bevy_wds::LocalActor;
use loro::LoroValue;
use loro_surgeon::Hydrate;

use crate::{
    LayerEnabled, LayerOpinions, LayerStrength, OpinionAttrs, OpinionTarget, Stage, StageCompiled,
    StageLayers,
    attributes::{material::MaterialAttr, mesh::MeshAttr},
    merge::merge_values,
};

pub fn compile_stages(
    stages: Query<(&mut StageCompiled, &StageLayers), With<Stage>>,
    layers: Query<(&LayerEnabled, &LayerStrength, &LayerOpinions)>,
    opinions: Query<(&OpinionTarget, &OpinionAttrs)>,
    actor: Query<&LocalActor>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    let Ok(_actor) = actor.single() else {
        return;
    };

    for (mut compiled, stage_layers) in stages {
        if compiled.0 {
            continue;
        }

        // Read layers.
        let mut resolved_layers = Vec::with_capacity(stage_layers.len());

        for layer_ent in stage_layers.iter() {
            let Ok((enabled, strength, layer_opinions)) = layers.get(layer_ent) else {
                continue;
            };

            if !enabled.0 {
                continue;
            }

            resolved_layers.push((strength.0, layer_opinions.0.clone()));
        }

        resolved_layers.sort_by(|a, b| a.0.cmp(&b.0));
        debug_assert!(
            resolved_layers.first().map(|(i, _)| *i).unwrap_or_default()
                <= resolved_layers.last().map(|(i, _)| *i).unwrap_or_default(),
            "layers not sorted in correct order"
        );

        // Merge layer opinions.
        let mut node_attrs = HashMap::<Entity, HashMap<&str, Cow<LoroValue>>>::new();

        for (_, layer_opinions) in resolved_layers {
            for opinion_ent in layer_opinions {
                let Ok((target, attrs)) = opinions.get(opinion_ent) else {
                    continue;
                };

                let target_attrs = node_attrs.entry(target.0).or_default();

                for (key, next) in attrs.0.iter() {
                    let Some(prev) = target_attrs.remove(key.as_str()) else {
                        target_attrs.insert(key, Cow::Borrowed(next));
                        continue;
                    };

                    let out = merge_values(prev, next);
                    target_attrs.insert(key, out);
                }
            }
        }

        // Apply node attributes.
        #[expect(clippy::cast_possible_truncation)]
        for (node_ent, attrs) in node_attrs {
            let mut node = commands.entity(node_ent);

            if let Ok(Some(material)) = attrs
                .get("material")
                .map(|v| MaterialAttr::hydrate(v))
                .transpose()
                .inspect_err(|err| {
                    warn!(?err, "failed to hydrate material");
                })
            {
                let mut out_mat = StandardMaterial::default();

                if let Some(color) = material.base_color {
                    out_mat.base_color = Color::Srgba(Srgba {
                        red: color[0] as f32,
                        green: color[1] as f32,
                        blue: color[2] as f32,
                        alpha: color[3] as f32,
                    });
                }

                if let Some(metallic) = material.metallic {
                    out_mat.metallic = metallic as f32;
                }
                if let Some(roughness) = material.roughness {
                    out_mat.perceptual_roughness = roughness as f32;
                }

                // TODO async load texture blobs

                let handle = asset_server.add(out_mat);
                node.insert(MeshMaterial3d(handle));
            } else {
                node.remove::<MeshMaterial3d<StandardMaterial>>();
            }

            if let Ok(Some(_mesh)) = attrs
                .get("mesh")
                .map(|v| MeshAttr::hydrate(v))
                .transpose()
                .inspect_err(|err| {
                    warn!(?err, "failed to hydrate mesh");
                })
            {
            } else {
                node.remove::<Mesh3d>();
            }
        }

        compiled.0 = true;
    }
}

#[derive(Component)]
pub struct MeshBlobs {}
