//! `Transform`/`Quat` helpers shared across guests.

use crate::math::{
    Quat,
    Transform,
    Vec3,
};

/// A transform that draws nothing: zero scale at the origin.
#[must_use]
pub const fn hidden() -> Transform {
    Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ZERO)
}

/// A rotation of `radians` about the world Y axis.
#[must_use]
pub fn yaw(radians: f32) -> Quat {
    Quat::from_rotation_y(radians)
}

/// The viewer-frame position a held tool or artifact rides at.
///
/// Forward and slightly down-right, clear of the body. Every held body and
/// the muzzle it fires from share this offset, so they agree on where "in
/// hand" is.
pub const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
