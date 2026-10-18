//! Telling a teleport from a walk, and carrying world-locked things across one.
//!
//! The local agent respawns by jumping the camera kilometres in a single tick,
//! which no amount of walking does. Anything a script left standing in the
//! world relative to the viewer is carried by the same move the viewer made.

use crate::math::{
    Transform,
    Vec3,
};

/// Metres between consecutive fixed-tick eyes past which the move is a
/// teleport rather than motion.
pub const TELEPORT_JUMP: f32 = 2.0;

/// Whether the eye going from `from` to `to` in one tick is a teleport.
#[must_use]
pub fn is_teleport(from: &Transform, to: &Transform) -> bool {
    (to.translation - from.translation).length_squared() > TELEPORT_JUMP * TELEPORT_JUMP
}

/// The move that takes `from` to `to`, so that `relative(from, to) * x` keeps
/// `x` where it was relative to the eye. The move is rigid, since the eye's
/// scale is not part of where it stands.
#[must_use]
pub fn relative(from: &Transform, to: &Transform) -> Transform {
    let back = from.rotation.inverse();
    let undo = Transform::new(-(back * from.translation), back, Vec3::ONE);
    let there = Transform::new(to.translation, to.rotation, Vec3::ONE);
    there * undo
}

/// Remembers the eye from the previous fixed tick.
#[derive(Debug, Default)]
pub struct Watch {
    last: Option<Transform>,
}

impl Watch {
    /// Records `eye`. Returns the move to carry world-locked things by when
    /// it is a teleport from the previous tick's eye.
    pub fn step(&mut self, eye: &Transform) -> Option<Transform> {
        let last = self.last.replace(*eye)?;
        is_teleport(&last, eye).then(|| relative(&last, eye))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Quat;

    fn eye_at(x: f32) -> Transform {
        Transform::from_translation(Vec3::new(x, 1.6, 0.0))
    }

    #[test]
    fn a_jump_of_exactly_the_threshold_is_still_a_walk() {
        assert!(!is_teleport(&eye_at(0.0), &eye_at(TELEPORT_JUMP)));
    }

    #[test]
    fn a_jump_past_the_threshold_is_a_teleport() {
        assert!(is_teleport(&eye_at(0.0), &eye_at(TELEPORT_JUMP + 0.01)));
    }

    #[test]
    fn the_first_eye_seen_is_never_a_teleport() {
        let mut watch = Watch::default();
        assert!(watch.step(&eye_at(500.0)).is_none());
    }

    #[test]
    fn the_watch_reports_a_jump_once_and_then_follows_the_new_eye() {
        let mut watch = Watch::default();
        assert!(watch.step(&eye_at(0.0)).is_none());
        assert!(watch.step(&eye_at(300.0)).is_some());
        assert!(watch.step(&eye_at(300.5)).is_none());
    }

    #[test]
    fn a_carried_anchor_keeps_its_pose_relative_to_the_eye() {
        let from = Transform::new(
            Vec3::new(1.0, 1.6, -2.0),
            Quat::from_rotation_y(0.6),
            Vec3::ONE,
        );
        let to = Transform::new(
            Vec3::new(-300.0, 4.0, 80.0),
            Quat::from_rotation_y(-1.9),
            Vec3::ONE,
        );
        let anchor = Transform::new(
            Vec3::new(1.4, 1.5, -2.6),
            Quat::from_rotation_y(0.2),
            Vec3::ONE,
        );

        let carried = relative(&from, &to) * anchor;

        let offset_before = from.rotation.inverse() * (anchor.translation - from.translation);
        let offset_after = to.rotation.inverse() * (carried.translation - to.translation);
        assert!((offset_before - offset_after).length() < 1.0e-4);

        let facing_before = from.rotation.inverse() * (anchor.rotation * Vec3::NEG_Z);
        let facing_after = to.rotation.inverse() * (carried.rotation * Vec3::NEG_Z);
        assert!((facing_before - facing_after).length() < 1.0e-4);
    }

    #[test]
    fn the_relative_move_takes_the_old_eye_to_the_new_one() {
        let from = eye_at(3.0);
        let to = Transform::new(
            Vec3::new(9.0, 2.0, 7.0),
            Quat::from_rotation_y(1.1),
            Vec3::ONE,
        );
        let moved = relative(&from, &to) * from;
        assert!((moved.translation - to.translation).length() < 1.0e-4);
    }
}
