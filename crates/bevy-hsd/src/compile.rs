use std::{borrow::Cow, collections::HashMap};

use bevy::prelude::*;
use bevy_wds::LocalActor;
use loro::LoroValue;
use loro_surgeon::Hydrate;

use crate::{
    LayerEnabled, LayerOpinions, LayerStrength, OpinionAttrs, OpinionTarget, Stage, StageCompiled,
    StageLayers, merge::merge_values, stage::Attrs,
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

        // Merge layer opinions into raw LoroValues.
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

        // Apply node attributes by hydrating merged values to typed Attrs.
        #[expect(clippy::cast_possible_truncation)]
        for (node_ent, raw_attrs) in node_attrs {
            let mut node = commands.entity(node_ent);

            // Convert merged HashMap to LoroMapValue for hydration.
            let merged_map: loro::LoroMapValue = raw_attrs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into_owned()))
                .collect::<std::collections::HashMap<_, _>>()
                .into();

            // Hydrate to typed Attrs.
            let attrs = match Attrs::hydrate(&LoroValue::Map(merged_map)) {
                Ok(a) => a,
                Err(err) => {
                    warn!(?err, "failed to hydrate attrs");
                    continue;
                }
            };

            // Apply material attributes.
            let has_material = attrs.mesh_colors.is_some();
            if has_material {
                let out_mat = StandardMaterial::default();
                // TODO: Load mesh colors from blob.
                let handle = asset_server.add(out_mat);
                node.insert(MeshMaterial3d(handle));
            } else {
                node.remove::<MeshMaterial3d<StandardMaterial>>();
            }

            // Apply transform attributes.
            if attrs.xform_pos.is_some() || attrs.xform_rot.is_some() || attrs.xform_scale.is_some()
            {
                let mut transform = Transform::default();

                if let Some(pos) = attrs.xform_pos {
                    transform.translation = Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32);
                }
                if let Some(rot) = attrs.xform_rot {
                    transform.rotation =
                        Quat::from_xyzw(rot[0] as f32, rot[1] as f32, rot[2] as f32, rot[3] as f32);
                }
                if let Some(scale) = attrs.xform_scale {
                    transform.scale = Vec3::new(scale[0] as f32, scale[1] as f32, scale[2] as f32);
                }

                node.insert(transform);
            } else {
                node.remove::<Transform>();
            }

            // TODO: Handle mesh attributes, xform_parent, etc.
        }

        compiled.0 = true;
    }
}
