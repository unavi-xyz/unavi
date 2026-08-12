use bevy::{
    animation::animated_field,
    platform::collections::HashMap,
    prelude::*,
};
use bevy_vrm::{
    BoneName,
    animations::vrm::VRM_ANIMATION_TARGETS,
};

use super::{
    AnimationName,
    bone_mask_group,
    mixamo::MIXAMO_BONE_NAMES,
    raw::RawAnimations,
};

#[derive(Component, Clone)]
pub struct AvatarAnimationClips {
    pub handle:  Handle<RawAnimations>,
    pub indices: HashMap<AnimationName, usize>,
}

#[derive(Component, Clone)]
pub struct AvatarAnimationNodes(pub HashMap<AnimationName, AnimationNodeIndex>);

pub(crate) fn load_animation_nodes(
    raw_animations: Res<Assets<RawAnimations>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    to_load: Query<(Entity, &AvatarAnimationClips), Without<AnimationGraphHandle>>,
) {
    for (entity, animations) in to_load.iter() {
        let Some(raw) = raw_animations.get(&animations.handle) else {
            continue;
        };

        let mut graph = AnimationGraph::default();
        let mut animation_nodes = HashMap::default();

        // Mask groups let tracked bones opt out of animation.
        for (bone_name, &target_id) in VRM_ANIMATION_TARGETS.iter() {
            let mask_group = bone_mask_group(*bone_name);
            graph.add_target_to_mask_group(target_id, mask_group);
        }

        for (name, &index) in &animations.indices {
            let Some(animation) = raw.animations.get(index) else {
                continue;
            };

            info!("Loading avatar animation: {name:?}");

            let mut clip = AnimationClip::default();

            for channel in &animation.channels {
                let Some((bone_name, _)) = MIXAMO_BONE_NAMES
                    .iter()
                    .find(|(_, v)| **v == channel.target)
                else {
                    continue;
                };

                // Head transform is set by user's camera.
                if *bone_name == BoneName::Head {
                    continue;
                }

                let vrm_target = VRM_ANIMATION_TARGETS[bone_name];

                let Some(mixamo_node) = raw.nodes.get(&channel.target) else {
                    warn!("No animation gltf node for {}", channel.target);
                    continue;
                };

                let mixamo_rest = raw.parent_rest(&channel.target);

                // Retarget rotations from Mixamo-space to Bevy-space.
                let samples = channel
                    .timestamps
                    .iter()
                    .copied()
                    .zip(channel.values.iter().copied())
                    .map(|(t, item)| {
                        let mut item = mixamo_rest
                            .mul_quat(item)
                            .mul_quat((mixamo_rest * mixamo_node.rotation).inverse());

                        // TODO: Only if VRM 0
                        item.y = -item.y;
                        item.w = -item.w;
                        let item = item.normalize();

                        (t, item)
                    });

                let curve = match UnevenSampleAutoCurve::new(samples) {
                    Ok(c) => c,
                    Err(err) => {
                        warn!("Failed to retarget {}: {err:?}", channel.target);
                        continue;
                    }
                };

                let property = animated_field!(Transform::rotation);
                let curve = AnimatableCurve::new(property, curve);

                clip.add_curve_to_target(vrm_target, curve);
            }

            let clip_handle = clips.add(clip);

            let node_idx = graph.add_clip(clip_handle, 1.0, graph.root);
            animation_nodes.insert(name.clone(), node_idx);
        }

        let graph = graphs.add(graph);

        commands.entity(entity).insert((
            AnimationGraphHandle(graph),
            AvatarAnimationNodes(animation_nodes),
        ));
    }
}
