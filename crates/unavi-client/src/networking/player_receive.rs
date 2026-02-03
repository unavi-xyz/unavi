use std::{collections::HashMap, sync::atomic::Ordering};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use iroh::EndpointId;
use unavi_player::{
    AvatarBones,
    animation::{AvatarAnimationNodes, bone_mask_group},
};

use crate::networking::{event::PlayerInboundState, thread::space::DEFAULT_TICKRATE};

/// Tracks which bones are masked from animation and their target rotations.
#[derive(Component, Default)]
pub struct TrackedBoneState {
    pub masked: u64,
    pub target: HashMap<BoneName, Quat>,
    pub speed: f32,
}

#[derive(Component, Deref)]
pub struct RemotePlayer(pub EndpointId);

/// Target transform for interpolation.
#[derive(Component)]
pub struct TransformTarget {
    pub translation: Vec3,
    pub rotation: Quat,
    pub speed: f32,
}

impl Default for TransformTarget {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            speed: f32::from(DEFAULT_TICKRATE),
        }
    }
}

fn speed_from_hz(hz: u8) -> f32 {
    f32::from(hz) / 1.5
}

/// Receives network data and updates [`TransformTarget`].
pub fn receive_player_transforms(mut players: Query<(&PlayerInboundState, &mut TransformTarget)>) {
    for (state, mut target) in &mut players {
        // Update speed from negotiated tickrate.
        let hz = state.tickrate.load(Ordering::Relaxed);
        target.speed = speed_from_hz(hz);

        let Some(iframe) = state.pose.iframe.try_lock() else {
            continue;
        };
        let Some(pframe) = state.pose.pframe.try_lock() else {
            continue;
        };
        let Some(iframe) = iframe.as_ref() else {
            continue;
        };
        let Some(pframe) = pframe.as_ref() else {
            continue;
        };

        let iframe_root: Vec3 = iframe.pose.root.pos.into();
        target.translation = iframe_root;

        if pframe.iframe_id == iframe.id {
            // Apply p-frame delta position relative to i-frame baseline.
            target.translation = pframe.pose.root.pos.apply_to(iframe_root);
            target.rotation = pframe.pose.root.rot.into();
        } else {
            // Only apply i-frame rotation.
            target.rotation = iframe.pose.root.rot.into();
        }
    }
}

/// Interpolates [`Transform`] towards [`TransformTarget`] each frame.
pub fn lerp_to_target(time: Res<Time>, mut query: Query<(&mut Transform, &TransformTarget)>) {
    for (mut transform, target) in &mut query {
        let t = (target.speed * time.delta_secs()).min(1.0);
        transform.translation = transform.translation.lerp(target.translation, t);
        transform.rotation = transform.rotation.slerp(target.rotation, t);
    }
}

/// Receives bone poses and updates animation masks.
///
/// When a bone appears in the I-frame, it gets masked from animation.
/// When a bone disappears, animation is re-enabled for it.
pub fn receive_remote_bones(
    players: Query<(&PlayerInboundState, &Children), With<RemotePlayer>>,
    mut avatars: Query<(
        &AvatarAnimationNodes,
        &AnimationGraphHandle,
        &mut TrackedBoneState,
    )>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (state, children) in &players {
        let Some(avatar_entity) = children.iter().find(|c| avatars.contains(*c)) else {
            continue;
        };
        let Ok((nodes, graph_handle, mut bone_state)) = avatars.get_mut(avatar_entity) else {
            continue;
        };
        let Some(graph) = graphs.get_mut(graph_handle.id()) else {
            continue;
        };

        // Update speed from tickrate.
        let hz = state.tickrate.load(Ordering::Relaxed);
        bone_state.speed = speed_from_hz(hz);

        let Some(iframe) = state.pose.iframe.try_lock() else {
            continue;
        };
        let Some(iframe) = iframe.as_ref() else {
            continue;
        };

        let pframe = state.pose.pframe.try_lock();
        if let Some(ref pframe) = pframe
            && let Some(pframe) = pframe.as_ref()
            && pframe.iframe_id == iframe.id
        {
            for bone in &pframe.pose.bones {
                bone_state.target.insert(bone.id, bone.transform.rot.into());
            }
        } else {
            // Build new mask from I-frame bones.
            let mut new_mask: u64 = 0;
            bone_state.target.clear();

            for bone in &iframe.pose.bones {
                let group = bone_mask_group(bone.id);
                new_mask |= 1 << group;
                bone_state.target.insert(bone.id, bone.transform.rot.into());
            }

            // Update animation node masks if changed.
            if new_mask != bone_state.masked {
                let added = new_mask & !bone_state.masked;
                let removed = bone_state.masked & !new_mask;

                for &node_idx in nodes.0.values() {
                    let node = &mut graph[node_idx];

                    // Add mask for newly tracked bones (disable animation).
                    if added != 0 {
                        node.mask |= added;
                    }

                    // Remove mask for no-longer-tracked bones (enable animation).
                    if removed != 0 {
                        node.mask &= !removed;
                    }
                }

                bone_state.masked = new_mask;
            }
        }
    }
}

/// Interpolates tracked bone rotations toward their targets.
pub fn slerp_to_target(
    time: Res<Time>,
    players: Query<&Children, With<RemotePlayer>>,
    avatars: Query<(&AvatarBones, &TrackedBoneState)>,
    mut bone_transforms: Query<&mut Transform, With<BoneName>>,
) {
    for children in &players {
        let Some((bones, state)) = children.iter().find_map(|c| avatars.get(c).ok()) else {
            continue;
        };

        let t = (state.speed * time.delta_secs()).min(1.0);

        for (&bone_name, &target_rot) in &state.target {
            let Some(&bone_entity) = bones.get(&bone_name) else {
                continue;
            };
            let Ok(mut transform) = bone_transforms.get_mut(bone_entity) else {
                continue;
            };

            transform.rotation = transform.rotation.slerp(target_rot, t);
        }
    }
}
