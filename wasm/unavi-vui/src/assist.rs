use wired_math::types::Vec3;

use crate::{
    attention::Attention,
    tuning::Tuning,
};

/// The offset an attended mote takes toward the pointer.
#[must_use]
pub fn lean(slot: Vec3, pointer: Vec3, attention: Attention, tuning: &Tuning) -> Vec3 {
    if !attention.is_active() || tuning.lean_range <= f32::EPSILON {
        return Vec3::ZERO;
    }
    let toward = pointer - slot;
    let distance = toward.length();
    if distance <= f32::EPSILON {
        return Vec3::ZERO;
    }
    let falloff = 1.0 - (distance / tuning.lean_range).clamp(0.0, 1.0);
    toward / distance * (tuning.lean_dist * falloff)
}

/// Eases `current` toward `target` at `speed` per second, framerate
/// independent.
#[must_use]
pub fn approach(current: Vec3, target: Vec3, speed: f32, delta: f32) -> Vec3 {
    current.lerp(target, (speed * delta).clamp(0.0, 1.0))
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
}
