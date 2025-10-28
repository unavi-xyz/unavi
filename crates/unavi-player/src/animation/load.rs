use bevy::{animation::animated_field, platform::collections::HashMap, prelude::*};
use bevy_gltf_kun::import::gltf::{
    GltfKun,
    animation::{RawChannelData, RawGltfAnimation},
    node::GltfNode,
};
use bevy_vrm::{BoneName, animations::vrm::VRM_ANIMATION_TARGETS};

use crate::animation::mixamo::MIXAMO_BONE_NAMES;

use super::AnimationName;

#[derive(Component, Clone)]
pub struct AvatarAnimationClips(pub HashMap<AnimationName, AvatarAnimation>);

#[derive(Clone)]
pub struct AvatarAnimation {
    pub gltf: Handle<GltfKun>,
    pub animation: Handle<RawGltfAnimation>,
}

#[derive(Component, Clone)]
pub struct AvatarAnimationNodes(pub HashMap<AnimationName, AnimationNodeIndex>);

pub(crate) fn load_animation_nodes(
    gltfs: Res<Assets<GltfKun>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    nodes: Res<Assets<GltfNode>>,
    raw_animations: Res<Assets<RawGltfAnimation>>,
    to_load: Query<(Entity, &AvatarAnimationClips), Without<AnimationGraphHandle>>,
) {
    for (entity, animations) in to_load.iter() {
        let mut graph = AnimationGraph::default();
        let mut animation_nodes = HashMap::default();

        let mut failed = false;

        for (name, animation) in animations.0.iter() {
            let Some(raw_animation) = raw_animations.get(animation.animation.id()) else {
                failed = true;
                break;
            };

            let gltf = match gltfs.get(&animation.gltf) {
                Some(c) => c,
                None => {
                    failed = true;
                    break;
                }
            };

            info!("Loading avatar animation: {name:?}");

            let mut clip = AnimationClip::default();

            for channel in &raw_animation.channels {
                let Some(mixamo_name) = channel.target_path.last().map(|x| x.as_str()) else {
                    continue;
                };

                let Some((bone_name, _)) =
                    MIXAMO_BONE_NAMES.iter().find(|(_, v)| **v == mixamo_name)
                else {
                    // warn!("Unknown animation leaf name: {leaf_name}");
                    continue;
                };

                // Head transform is set by user's camera.
                if *bone_name == BoneName::Head {
                    continue;
                }

                // Retarget the animation.
                let vrm_target = VRM_ANIMATION_TARGETS[bone_name];

                match &channel.data {
                    RawChannelData::Translation { .. } => {
                        continue;
                        // TODO: fix translation animations
                        //       - scale out of mixamo size to avatar size?
                        //
                        // let samples = timestamps
                        //     .clone()
                        //     .into_iter()
                        //     .zip(values.clone().into_iter())
                        //     .map(|(t, mut item)| {
                        //         item.x /= 100.0;
                        //         item.y /= 100.0;
                        //         item.z /= 100.0;
                        //
                        //         // TODO: Only if VRM 0
                        //         item.x = -item.x;
                        //
                        //         (t, item)
                        //     });
                        //
                        // let curve = match UnevenSampleAutoCurve::new(samples) {
                        //     Ok(c) => c,
                        //     Err(e) => {
                        //         warn!("Failed to retarget {:?}: {e:?}", channel.target_path);
                        //         continue;
                        //     }
                        // };
                        //
                        // let property = animated_field!(Transform::translation);
                        // let curve = AnimatableCurve::new(property, curve);
                        //
                        // clip.add_curve_to_target(vrm_target, curve);
                    }
                    RawChannelData::Rotation { timestamps, values } => {
                        // Get Mixamo rest pose.
                        let mixamo_node_handle = match gltf.named_nodes.get(mixamo_name) {
                            Some(n) => n,
                            None => {
                                warn!("No animation gltf node for {mixamo_name}");
                                continue;
                            }
                        };
                        let mixamo_node = nodes.get(mixamo_node_handle).unwrap();

                        let mut mixamo_parents = Vec::with_capacity(channel.target_path.len());
                        create_parent_chain(
                            gltf,
                            &nodes,
                            &mut mixamo_parents,
                            &mixamo_node_handle.id(),
                        );

                        let mixamo_rest = mixamo_parents
                            .iter()
                            .rev()
                            .fold(Quat::IDENTITY, |rot, n| rot * n.transform.rotation);

                        // Retarget rotations from Mixamo-space to Bevy-space.
                        let samples = timestamps
                            .clone()
                            .into_iter()
                            .zip(values.clone().into_iter())
                            .map(|(t, item)| {
                                let mut item = mixamo_rest.mul_quat(item).mul_quat(
                                    (mixamo_rest * mixamo_node.transform.rotation).inverse(),
                                );

                                // TODO: Only if VRM 0
                                item.y = -item.y;
                                item.w = -item.w;
                                let item = item.normalize();

                                (t, item)
                            });

                        let curve = match UnevenSampleAutoCurve::new(samples) {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Failed to retarget {:?}: {e:?}", channel.target_path);
                                continue;
                            }
                        };

                        let property = animated_field!(Transform::rotation);
                        let curve = AnimatableCurve::new(property, curve);

                        clip.add_curve_to_target(vrm_target, curve);
                    }
                    _ => {}
                }
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

fn create_parent_chain(
    gltf: &GltfKun,
    nodes: &Res<Assets<GltfNode>>,
    parents: &mut Vec<GltfNode>,
    target: &AssetId<GltfNode>,
) {
    for handle in gltf.nodes.iter() {
        let Some(node) = nodes.get(handle) else {
            continue;
        };

        if node.children.iter().any(|c| c.id() == *target) {
            parents.push(node.clone());
            create_parent_chain(gltf, nodes, parents, &handle.id());
            break;
        };
    }
}
