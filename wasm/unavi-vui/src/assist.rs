use wired_math::types::Vec3;

use crate::{
    attention::Attention,
    tuning::Tuning,
};

/// How near `pointer` has come to `slot`: 1 on top of it, 0 at the edge of
/// reach.
///
/// What a mote notices an approach with, and the same falloff [`lean`] leans
/// by — so coming closer and being reached for share one sense of range
/// rather than two that can drift apart.
#[must_use]
pub fn proximity(slot: Vec3, pointer: Vec3, tuning: &Tuning) -> f32 {
    if tuning.lean_range <= f32::EPSILON {
        return 0.0;
    }
    1.0 - ((pointer - slot).length() / tuning.lean_range).clamp(0.0, 1.0)
}

/// The offset an attended mote takes toward the pointer.
#[must_use]
pub fn lean(slot: Vec3, pointer: Vec3, attention: Attention, tuning: &Tuning) -> Vec3 {
    if !attention.is_active() {
        return Vec3::ZERO;
    }
    let toward = pointer - slot;
    let distance = toward.length();
    if distance <= f32::EPSILON {
        return Vec3::ZERO;
    }
    toward / distance * (tuning.lean_dist * proximity(slot, pointer, tuning))
}

/// Eases `current` toward `target` at `speed` per second, framerate
/// independent.
#[must_use]
pub fn approach(current: Vec3, target: Vec3, speed: f32, delta: f32) -> Vec3 {
    current.lerp(target, (speed * delta).clamp(0.0, 1.0))
}

/// [`approach`] for a scalar, settling exactly rather than approaching
/// forever: an ease that only ever gets closer leaves every mote a hair off
/// its resting look, and redraws them all to say so.
#[must_use]
pub fn approach_scalar(current: f32, target: f32, speed: f32, delta: f32, settle: f32) -> f32 {
    if (target - current).abs() <= settle {
        return target;
    }
    (target - current).mul_add((speed * delta).clamp(0.0, 1.0), current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning::DEFAULT
    }

    #[test]
    fn only_an_attended_mote_leans() {
        let slot = Vec3::ZERO;
        let pointer = Vec3::new(0.1, 0.0, 0.0);
        assert_eq!(lean(slot, pointer, Attention::Idle, &tuning()), Vec3::ZERO);
        assert_eq!(lean(slot, pointer, Attention::Near, &tuning()), Vec3::ZERO);
        assert_ne!(
            lean(slot, pointer, Attention::Attended, &tuning()),
            Vec3::ZERO
        );
    }

    #[test]
    fn lean_points_at_the_pointer_and_is_capped() {
        let offset = lean(
            Vec3::ZERO,
            Vec3::new(0.05, 0.0, 0.0),
            Attention::Attended,
            &tuning(),
        );
        assert!(offset.x > 0.0);
        assert!(offset.length() <= tuning().lean_dist + 1.0e-5);
    }

    #[test]
    fn lean_falls_off_with_distance() {
        let near = lean(
            Vec3::ZERO,
            Vec3::new(0.05, 0.0, 0.0),
            Attention::Attended,
            &tuning(),
        );
        let far = lean(
            Vec3::ZERO,
            Vec3::new(0.3, 0.0, 0.0),
            Attention::Attended,
            &tuning(),
        );
        assert!(near.length() > far.length());
    }

    #[test]
    fn nothing_leans_beyond_range() {
        let beyond = Vec3::new(tuning().lean_range * 2.0, 0.0, 0.0);
        assert_eq!(
            lean(Vec3::ZERO, beyond, Attention::Attended, &tuning()),
            Vec3::ZERO
        );
    }

    #[test]
    fn proximity_is_one_on_the_mote_and_nothing_at_the_edge_of_reach() {
        let slot = Vec3::ZERO;
        assert!((proximity(slot, slot, &tuning()) - 1.0).abs() < 1.0e-6);

        let edge = Vec3::new(tuning().lean_range, 0.0, 0.0);
        assert!(proximity(slot, edge, &tuning()).abs() < 1.0e-6);
        assert!(
            proximity(slot, edge * 2.0, &tuning()).abs() < 1.0e-6,
            "and no further"
        );
    }

    #[test]
    fn proximity_rises_as_the_pointer_closes() {
        let slot = Vec3::ZERO;
        let far = proximity(slot, Vec3::new(0.3, 0.0, 0.0), &tuning());
        let near = proximity(slot, Vec3::new(0.1, 0.0, 0.0), &tuning());
        assert!(near > far);
        assert!(far >= 0.0);
    }

    #[test]
    fn a_pointer_inside_the_mote_does_not_produce_a_nan() {
        let offset = lean(Vec3::ZERO, Vec3::ZERO, Attention::Attended, &tuning());
        assert!(offset.is_finite());
    }

    #[test]
    fn approach_converges_without_overshooting() {
        let target = Vec3::new(1.0, 0.0, 0.0);
        let mut current = Vec3::ZERO;
        for _ in 0..200 {
            current = approach(current, target, 12.0, 0.016);
            assert!(current.x <= target.x + 1.0e-5, "never overshoots");
        }
        assert!((current - target).length() < 1.0e-3);
    }

    #[test]
    fn a_long_frame_does_not_overshoot() {
        let target = Vec3::new(1.0, 0.0, 0.0);
        let stepped = approach(Vec3::ZERO, target, 12.0, 1.0);
        assert_eq!(stepped, target);
    }

    #[test]
    fn a_scalar_takes_more_than_one_frame_to_arrive() {
        let part_way = approach_scalar(0.0, 1.0, 14.0, 1.0 / 60.0, 0.004);
        assert!(
            part_way > 0.0 && part_way < 1.0,
            "arriving in one frame is the snap this replaces; got {part_way}"
        );
    }

    /// Exactly, not nearly: a style is only rewritten when it differs, so a
    /// mote that merely approaches its resting look never stops redrawing.
    #[test]
    #[expect(clippy::float_cmp, reason = "landing exactly is the invariant")]
    fn a_scalar_settles_on_its_target_exactly() {
        let mut current = 0.0;
        for _ in 0..200 {
            current = approach_scalar(current, 1.0, 14.0, 1.0 / 60.0, 0.004);
        }
        assert_eq!(current, 1.0);

        let settled = approach_scalar(1.0, 1.0, 14.0, 1.0 / 60.0, 0.004);
        assert_eq!(settled, 1.0, "and stays there");
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "landing exactly is the invariant")]
    fn a_scalar_settles_going_down_as_well_as_up() {
        let mut current = 1.0;
        for _ in 0..200 {
            current = approach_scalar(current, 0.0, 14.0, 1.0 / 60.0, 0.004);
        }
        assert_eq!(current, 0.0);
    }
}
