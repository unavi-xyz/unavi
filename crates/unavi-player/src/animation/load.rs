use bevy::{animation::animated_field, platform::collections::HashMap, prelude::*};
use bevy_gltf_kun::import::gltf::animation::{RawChannelData, RawGltfAnimation};
use bevy_vrm::{BoneName, animations::vrm::VRM_ANIMATION_TARGETS};

use crate::animation::mixamo::MIXAMO_BONE_NAMES;

use super::AnimationName;

#[derive(Component, Clone)]
pub struct AvatarAnimationClips(pub HashMap<AnimationName, AvatarAnimation>);

#[derive(Clone)]
pub struct AvatarAnimation(pub Handle<RawGltfAnimation>);

#[derive(Component, Clone)]
pub struct AvatarAnimationNodes(pub HashMap<AnimationName, AnimationNodeIndex>);

pub(crate) fn load_animation_nodes(
    avatars: Query<(Entity, &AvatarAnimationClips), Without<AnimationGraphHandle>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    raw_animations: Res<Assets<RawGltfAnimation>>,
) {
    for (entity, animations) in avatars.iter() {
        let mut graph = AnimationGraph::default();
        let mut animation_nodes = HashMap::default();

        let mut failed = false;

        for (name, animation) in animations.0.iter() {
            let Some(raw_animation) = raw_animations.get(animation.0.id()) else {
                failed = true;
                break;
            };

            info!("Loading avatar animation: {name:?}");

            let mut clip = AnimationClip::default();

            for channel in &raw_animation.channels {
                // info!("Channel path: {:?}", channel.target_path);

                let Some(leaf_name) = channel.target_path.last().map(|x| x.as_str()) else {
                    continue;
                };

                let Some((bone_name, _)) = MIXAMO_BONE_NAMES.iter().find(|(_, v)| **v == leaf_name)
                else {
                    // warn!("Unknown animation leaf name: {leaf_name}");
                    continue;
                };

                // Head transform is set by user's camera.
                if *bone_name == BoneName::Head {
                    continue;
                }

                // Retarget the animation.
                let curve = match &channel.data {
                    RawChannelData::Translation { timestamps, values } => {
                        let samples = timestamps
                            .clone()
                            .into_iter()
                            .zip(values.clone().into_iter())
                            .map(|(t, mut item)| {
                                item.x /= 100.0;
                                item.y /= 100.0;
                                item.z /= 100.0;

                                item.x = -item.x;

                                (t, item)
                            });

                        let curve = match UnevenSampleAutoCurve::new(samples) {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Failed to retarget {:?}: {e:?}", channel.target_path);
                                continue;
                            }
                        };

                        let property = animated_field!(Transform::translation);

                        AnimatableCurve::new(property, curve)
                    }
                    RawChannelData::Rotation { values, .. } => {
                        for _item in values {
                            // TODO
                        }
                        continue;
                    }
                    _ => continue,
                };

                let vrm_target = VRM_ANIMATION_TARGETS[bone_name];
                clip.add_curve_to_target(vrm_target, curve);
            }

            // Save the clip as a new asset.
            let clip_handle = clips.add(clip);

            let node_idx = graph.add_clip(clip_handle, 1.0, graph.root);
            animation_nodes.insert(name.clone(), node_idx);
        }

        if failed {
            continue;
        }

        let graph = graphs.add(graph);

        commands.entity(entity).insert((
            AnimationGraphHandle(graph),
            AvatarAnimationNodes(animation_nodes),
        ));
    }
}

// fn create_parent_chain(
//     gltf: &Gltf,
//     nodes: &Res<Assets<GltfNode>>,
//     parents: &mut Vec<GltfNode>,
//     target: &GltfNode,
// ) {
//     for handle in gltf.nodes.iter() {
//         let node = nodes.get(handle).unwrap();
//         if node.children.iter().any(|c| c.name == target.name) {
//             parents.push(node.clone());
//             create_parent_chain(gltf, nodes, parents, node);
//             break;
//         };
//     }
// }
