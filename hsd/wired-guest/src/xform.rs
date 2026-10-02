//! Small `Transform`/`Quat` helpers duplicated across guests.
//!
//! `wired:scene` has no visibility flag (see the WIT gaps in the phase A
//! handoff), so [`hidden`] is the zero-scale workaround every guest used.

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
