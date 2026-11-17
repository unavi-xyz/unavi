//! Where a surface stands relative to the local agent's eye.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

/// Yaw-only rotation facing `forward`, so a surface stands upright regardless
/// of where the viewer was looking when it was placed.
#[must_use]
pub fn yaw_only(forward: Vec3) -> Quat {
    let theta = (-forward.x).atan2(-forward.z);
    Quat::new(0.0, (theta * 0.5).sin(), 0.0, (theta * 0.5).cos())
}

/// The direction the viewer faces, flattened onto the ground plane.
#[must_use]
pub fn facing(eye: &Transform) -> Vec3 {
    let forward = eye.rotation * Vec3::new(0.0, 0.0, -1.0);
    Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero()
}
