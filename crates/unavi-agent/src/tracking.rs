use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackingSource {
    #[default]
    Desktop,
    Network,
    Xr,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TrackedPose {
    pub rotation: Quat,
    pub translation: Vec3,
}

impl TrackedPose {
    #[must_use]
    pub const fn new(translation: Vec3, rotation: Quat) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    #[must_use]
    pub const fn from_transform(transform: &Transform) -> Self {
        Self {
            rotation: transform.rotation,
            translation: transform.translation,
        }
    }

    #[must_use]
    pub fn to_transform(&self) -> Transform {
        Transform {
            rotation: self.rotation,
            translation: self.translation,
            ..Default::default()
        }
    }
}

#[derive(Component, Default)]
#[require(TrackedPose, Transform)]
pub struct TrackedHead;

#[derive(Component, Default)]
#[require(TrackedPose, Transform)]
pub struct TrackedHand {
    pub is_left: bool,
}

/// Syncs [`TrackedPose`] to [`Transform`] for all tracked entities.
pub(crate) fn sync_tracked_pose_to_transform(
    mut tracked: Query<(&TrackedPose, &mut Transform), Changed<TrackedPose>>,
) {
    for (pose, mut transform) in &mut tracked {
        transform.rotation = pose.rotation;
        transform.translation = pose.translation;
    }
}
