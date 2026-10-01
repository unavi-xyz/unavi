//! Quantized wire encodings for poses: full-precision keyframes and compact
//! deltas against them, plus the one check every received value passes.

use bevy::math::{
    Quat,
    Vec3,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::grid::SPACE_CELL_SIZE;

pub mod f16_vec3;
pub mod f32_vec3;
pub mod i8_vec3;
pub mod pose;
pub mod quat;
pub mod rigid_transform;

/// Precision marker for a full-precision frame that deltas resolve against.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Keyframe;

/// Precision marker for a compact frame relative to the last keyframe.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Delta;

/// Fastest linear speed accepted from a peer, in m/s.
pub const MAX_LINEAR_SPEED: f32 = 200.0;

/// Fastest angular speed accepted from a peer, in rad/s.
pub const MAX_ANGULAR_SPEED: f32 = 100.0;

/// A space-relative position from a peer, if it is finite and inside the
/// space's grid cell.
#[must_use]
pub fn position(v: Vec3) -> Option<Vec3> {
    (v.is_finite() && v.abs().max_element() <= SPACE_CELL_SIZE).then_some(v)
}

/// A velocity from a peer, clamped to `max`; `None` if not finite.
#[must_use]
pub fn velocity(v: Vec3, max: f32) -> Option<Vec3> {
    v.is_finite().then(|| v.clamp_length_max(max))
}

/// A rotation from a peer, normalized; `None` if not finite.
#[must_use]
pub fn rotation(q: Quat) -> Option<Quat> {
    (q.is_finite() && q.length_squared() > f32::EPSILON).then(|| q.normalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_and_far_values_are_refused() {
        assert_eq!(position(Vec3::NAN), None);
        assert_eq!(position(Vec3::splat(SPACE_CELL_SIZE * 2.0)), None);
        assert_eq!(position(Vec3::ONE), Some(Vec3::ONE));
        assert_eq!(velocity(Vec3::INFINITY, 1.0), None);
        assert!(velocity(Vec3::splat(1.0e30), 10.0).is_some_and(|v| v.length() <= 10.001));
        assert_eq!(rotation(Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0)), None);
    }
}
