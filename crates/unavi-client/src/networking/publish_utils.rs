//! Shared constants and helpers for agent and object publishing.

use std::time::Duration;

use bevy::math::{Quat, Vec3};

use crate::networking::thread::space::MAX_TICKRATE;

/// How many ticks between I-frames.
pub const IFRAME_FREQ: u64 = MAX_TICKRATE as u64 * 3;

/// Position change threshold in meters.
pub const POS_EPSILON: f32 = 0.001;
/// Rotation change threshold in radians.
pub const ROT_EPSILON: f32 = 0.005;
/// Velocity change threshold in m/s.
pub const VEL_EPSILON: f32 = 0.01;

/// How often we publish state to the network thread.
/// From there it will be broadcasted to peers at variable rates.
pub const PUBLISH_INTERVAL: Duration = Duration::from_millis(1000 / MAX_TICKRATE as u64);

/// Returns true if position or rotation changed beyond thresholds.
pub fn transform_changed(
    current_pos: Vec3,
    current_rot: Quat,
    last_pos: Vec3,
    last_rot: Quat,
) -> bool {
    current_pos.distance_squared(last_pos) > POS_EPSILON * POS_EPSILON
        || current_rot.angle_between(last_rot) > ROT_EPSILON
}

/// Returns true if velocity changed beyond threshold.
pub fn velocity_changed(current: Vec3, last: Vec3) -> bool {
    current.distance_squared(last) > VEL_EPSILON * VEL_EPSILON
}
