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

/// Where a held tool or artifact rides, in the viewer's own frame.
///
/// Forward and slightly down-right, clear of the body. Shared by halo,
/// physgun and spawner so the physgun's muzzle and every tool's visible body
/// agree on where "in hand" is.
pub const ARTIFACT_OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);
