//! Object transform interpolation for smooth networked physics.

use avian3d::dynamics::rigid_body::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;

use crate::networking::{object_publish::LocallyOwned, thread::space::MAX_OBJECT_TICKRATE};

/// Target transform for object interpolation.
#[derive(Component)]
pub struct ObjectTransformTarget {
    pub translation: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
}

impl ObjectTransformTarget {
    fn speed(&self) -> f32 {
        f32::from(MAX_OBJECT_TICKRATE) / 1.5
    }

    /// Create from current transform and velocities.
    pub fn from_current(
        transform: &Transform,
        lin_vel: Option<&LinearVelocity>,
        ang_vel: Option<&AngularVelocity>,
    ) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            linear_velocity: lin_vel.map_or(Vec3::ZERO, |v| v.0),
            angular_velocity: ang_vel.map_or(Vec3::ZERO, |v| v.0),
        }
    }
}

impl Default for ObjectTransformTarget {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }
}

/// Interpolates objects towards their network targets.
pub fn lerp_objects_to_target(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &ObjectTransformTarget,
            Option<&mut LinearVelocity>,
            Option<&mut AngularVelocity>,
        ),
        Without<LocallyOwned>,
    >,
) {
    for (mut transform, target, lin_vel, ang_vel) in &mut query {
        let t = (target.speed() * time.delta_secs()).min(1.0);
        transform.translation = transform.translation.lerp(target.translation, t);
        transform.rotation = transform.rotation.slerp(target.rotation, t);

        if let Some(mut lin_vel) = lin_vel {
            // lin_vel.0 = lin_vel.0.lerp(target.linear_velocity, t);
            lin_vel.0 = target.linear_velocity;
        }
        if let Some(mut ang_vel) = ang_vel {
            // ang_vel.0 = ang_vel.0.lerp(target.angular_velocity, t);
            ang_vel.0 = target.angular_velocity;
        }
    }
}
