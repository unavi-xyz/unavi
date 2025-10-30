use bevy::prelude::*;

/// Source of tracking data for the player.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackingSource {
    /// Desktop mode: keyboard/mouse input drives animations and head tracking.
    #[default]
    Desktop,
    /// VR mode: headset and controller tracking drives bone transforms directly.
    Vr,
    /// Network mode: bone transforms received from network.
    Network,
}

/// A tracked pose (position + rotation) for a body part.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TrackedPose {
    pub rotation: Quat,
    pub translation: Vec3,
}

impl TrackedPose {
    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            rotation: transform.rotation,
            translation: transform.translation,
        }
    }

    pub fn to_transform(&self) -> Transform {
        Transform {
            rotation: self.rotation,
            translation: self.translation,
            ..Default::default()
        }
    }
}

/// Marker component for the tracked head entity.
#[derive(Component, Default)]
#[require(TrackedPose, Transform, GlobalTransform, Visibility)]
pub struct TrackedHead;

/// Marker component for tracked hands (future use).
#[derive(Component, Default)]
#[require(TrackedPose, Transform, GlobalTransform, Visibility)]
pub struct TrackedHand {
    pub is_left: bool,
}

/// Syncs [TrackedPose] to [Transform] for all tracked entities.
pub(crate) fn sync_tracked_pose_to_transform(
    mut tracked: Query<(&TrackedPose, &mut Transform), Changed<TrackedPose>>,
) {
    for (pose, mut transform) in tracked.iter_mut() {
        transform.rotation = pose.rotation;
        transform.translation = pose.translation;
    }
}
