use std::sync::atomic::Ordering;

use bevy::prelude::*;
use iroh::EndpointId;

use crate::networking::{event::PlayerInboundState, thread::space::DEFAULT_TICKRATE};

#[derive(Component, Deref)]
pub struct OtherPlayer(pub EndpointId);

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

        for _pose in &iframe.pose.bones {
            // TODO: Apply bones.
        }

        if pframe.iframe_id == iframe.id {
            // Apply p-frame delta.
            target.translation = pframe.pose.root.pos.apply_to(target.translation);
            target.rotation = pframe.pose.root.rot.into();

            for _pose in &pframe.pose.bones {
                // TODO: Apply bones.
            }
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
