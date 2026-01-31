use std::{collections::HashMap, sync::atomic::Ordering};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use iroh::EndpointId;
use unavi_player::AvatarBones;

use crate::networking::{event::PlayerInboundState, thread::space::DEFAULT_TICKRATE};

/// Target bone rotations for interpolation.
#[derive(Component, Default)]
pub struct BoneRotationTargets {
    pub targets: HashMap<BoneName, Quat>,
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

/// Receives network data and updates [`TransformTarget`].
pub fn receive_player_transforms(mut players: Query<(&PlayerInboundState, &mut TransformTarget)>) {
    for (state, mut target) in &mut players {
        // Update speed from negotiated tickrate.
        let hz = state.tickrate.load(Ordering::Relaxed);
        target.speed = f32::from(hz);

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

/// Receives bone poses and updates [`BoneRotationTargets`].
pub fn receive_remote_bones(
    players: Query<(&PlayerInboundState, &Children), With<RemotePlayer>>,
    mut avatar_targets: Query<&mut BoneRotationTargets>,
) {
    for (state, children) in &players {
        let Some(avatar_entity) = children
            .iter()
            .find(|child| avatar_targets.contains(*child))
        else {
            continue;
        };

        let Ok(mut targets) = avatar_targets.get_mut(avatar_entity) else {
            continue;
        };

        // Update speed from tickrate.
        let hz = state.tickrate.load(Ordering::Relaxed);
        targets.speed = f32::from(hz);

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
                targets.targets.insert(bone.id, bone.transform.rot.into());
            }
        } else {
            for bone in &iframe.pose.bones {
                targets.targets.insert(bone.id, bone.transform.rot.into());
            }
        }
    }
}

/// Interpolates bone rotations towards [`BoneRotationTargets`].
pub fn slerp_bone_rotations(
    time: Res<Time>,
    players: Query<&Children, With<RemotePlayer>>,
    avatars: Query<(&AvatarBones, &BoneRotationTargets)>,
    mut bone_transforms: Query<&mut Transform, With<BoneName>>,
) {
    for children in &players {
        let Some((bones, targets)) = children.iter().find_map(|child| avatars.get(child).ok())
        else {
            continue;
        };

        let t = (targets.speed * time.delta_secs()).min(1.0);

        for (bone_name, &target_rot) in &targets.targets {
            let Some(&bone_entity) = bones.get(bone_name) else {
                continue;
            };
            let Ok(mut bone_transform) = bone_transforms.get_mut(bone_entity) else {
                continue;
            };
            bone_transform.rotation = bone_transform.rotation.slerp(target_rot, t);
        }
    }
}
