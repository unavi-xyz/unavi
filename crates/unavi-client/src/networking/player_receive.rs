use std::sync::atomic::Ordering;

use bevy::prelude::*;
use bevy_vrm::BoneName;
use iroh::EndpointId;
use unavi_player::AvatarBones;

use crate::networking::{event::PlayerInboundState, thread::space::DEFAULT_TICKRATE};

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

        target.translation = iframe.pose.root.pos.into();

        if pframe.iframe_id == iframe.id {
            // Apply p-frame delta.
            target.translation = pframe.pose.root.pos.apply_to(target.translation);
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

/// Applies received bone poses after animations, overwriting animated values.
pub fn apply_remote_bones(
    players: Query<(&PlayerInboundState, &Children), With<RemotePlayer>>,
    avatar_bones: Query<&AvatarBones>,
    mut bone_transforms: Query<&mut Transform, With<BoneName>>,
) {
    for (state, children) in &players {
        // Find the avatar child with AvatarBones.
        let Some(bones) = children
            .iter()
            .find_map(|child| avatar_bones.get(child).ok())
        else {
            continue;
        };

        let Some(iframe) = state.pose.iframe.try_lock() else {
            continue;
        };
        let Some(iframe) = iframe.as_ref() else {
            continue;
        };

        // Apply I-frame bone rotations.
        for bone_pose in &iframe.pose.bones {
            let Some(&bone_entity) = bones.get(&bone_pose.id) else {
                continue;
            };
            let Ok(mut bone_transform) = bone_transforms.get_mut(bone_entity) else {
                continue;
            };
            bone_transform.rotation = bone_pose.transform.rot.into();
        }

        // Apply P-frame bone rotations if valid.
        let pframe = state.pose.pframe.try_lock();
        if let Some(ref pframe) = pframe
            && let Some(pframe) = pframe.as_ref()
            && pframe.iframe_id == iframe.id
        {
            for bone_pose in &pframe.pose.bones {
                let Some(&bone_entity) = bones.get(&bone_pose.id) else {
                    continue;
                };
                let Ok(mut bone_transform) = bone_transforms.get_mut(bone_entity) else {
                    continue;
                };
                bone_transform.rotation = bone_pose.transform.rot.into();
            }
        }
    }
}
