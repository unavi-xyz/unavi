use bevy::{gltf::GltfNode, prelude::*, utils::HashMap};
use bevy_vrm::animations::vrm::VRM_ANIMATION_TARGETS;

use super::{
    mixamo::{MIXAMO_ANIMATION_TARGETS, MIXAMO_BONE_NAMES},
    AnimationName, AvatarAnimationClips,
};

#[derive(Component, Clone)]
pub struct AvatarAnimationNodes(pub HashMap<AnimationName, AnimationNodeIndex>);

#[derive(Component)]
pub struct CreatedAnimationGraph;

pub fn load_animation_nodes(
    avatars: Query<(Entity, &AvatarAnimationClips), Without<Handle<AnimationGraph>>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut commands: Commands,
    mut gltfs: ResMut<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    nodes: Res<Assets<GltfNode>>,
) {
    for (entity, animations) in avatars.iter() {
        let mut graph = AnimationGraph::default();
        let mut animation_nodes = HashMap::default();

        let mut failed = false;

        for (name, animation) in animations.0.iter() {
            let clip = match clips.get_mut(&animation.clip) {
                Some(c) => c,
                None => {
                    failed = true;
                    break;
                }
            };

            let gltf = match gltfs.get_mut(&animation.gltf) {
                Some(c) => c,
                None => {
                    failed = true;
                    break;
                }
            };

            let clip_curves = clip.curves_mut();

            for (name, target) in MIXAMO_ANIMATION_TARGETS.iter() {
                if let Some(mut curves) = clip_curves.remove(target) {
                    let mut to_remove = Vec::default();

                    for (i, curve) in curves.iter_mut().enumerate() {
                        if let Keyframes::Translation(translations) = &mut curve.keyframes {
                            // TODO: Fix translation animations.
                            // For some reason, all bones get rotated strangely when a translation
                            // is applied.
                            to_remove.push(i);

                            for item in translations {
                                item.x /= 100.0;
                                item.y /= 100.0;
                                item.z /= 100.0;

                                item.x = -item.x;
                            }
                        }
                        if let Keyframes::Rotation(rotations) = &mut curve.keyframes {
                            // Find target node.
                            let mixamo_name = MIXAMO_BONE_NAMES[name];
                            let node = match gltf.named_nodes.get(mixamo_name) {
                                Some(n) => n,
                                None => {
                                    error!("No animation gltf node for {}", mixamo_name);
                                    continue;
                                }
                            };

                            // Get all parents, up to root.
                            let node = nodes.get(node).unwrap();
                            let mut parents = Vec::default();
                            create_parent_chain(gltf, &nodes, &mut parents, node);

                            // Retarget rotation to be in VRM rig space.
                            let parent_rot = parents
                                .iter()
                                .rev()
                                .fold(Quat::default(), |rot, n| rot * n.transform.rotation);

                            let inverse_rot = (parent_rot * node.transform.rotation).inverse();

                            for item in rotations {
                                *item = parent_rot * *item * inverse_rot;

                                item.y = -item.y;
                                item.w = -item.w;
                            }
                        }
                    }

                    for index in to_remove.into_iter().rev() {
                        curves.remove(index);
                    }

                    let vrm_target = VRM_ANIMATION_TARGETS[name];
                    clip_curves.insert(vrm_target, curves);
                } else {
                    warn!("No animation curves for {}", name);
                }
            }

            let node = graph.add_clip(animation.clip.clone(), 1.0, graph.root);
            animation_nodes.insert(*name, node);
        }

        if failed {
            continue;
        }

        let graph = graphs.add(graph);

        commands
            .entity(entity)
            .insert((graph, AvatarAnimationNodes(animation_nodes)));
    }
}

fn create_parent_chain(
    gltf: &Gltf,
    nodes: &Res<Assets<GltfNode>>,
    parents: &mut Vec<GltfNode>,
    target: &GltfNode,
) {
    for handle in gltf.nodes.iter() {
        let node = nodes.get(handle).unwrap();
        if node.children.iter().any(|c| c.name == target.name) {
            parents.push(node.clone());
            create_parent_chain(gltf, nodes, parents, node);
            break;
        };
    }
}
