//! A non-finite or degenerate parameter reaches the solver intact, where it
//! spreads to every body sharing an island. Anything derived from a scene, a
//! script, or avatar geometry passes through here first.

use bevy::prelude::Vec3;

#[must_use]
pub fn positive(v: f32) -> bool {
    v.is_finite() && v > 0.0
}

#[must_use]
pub fn nonneg(v: f32) -> bool {
    v.is_finite() && v >= 0.0
}

/// A velocity, force, or point, accepted only if every component is finite.
///
/// One non-finite component is enough: the solver multiplies it through the
/// body's contacts, so the NaN reaches every other body in the same island.
#[must_use]
pub fn vec3(v: [f32; 3]) -> Option<Vec3> {
    let v = Vec3::from_array(v);
    v.is_finite().then_some(v)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;

    use super::{
        nonneg,
        positive,
        vec3,
    };

    #[test]
    fn a_finite_vector_passes_through_unchanged() {
        assert_eq!(vec3([1.0, -2.0, 3.5]), Some(Vec3::new(1.0, -2.0, 3.5)));
        assert_eq!(vec3([0.0; 3]), Some(Vec3::ZERO));
    }

    /// Any one component poisons the whole vector, so the check is per-axis.
    #[test]
    fn a_single_non_finite_component_rejects_the_vector() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(vec3([bad, 0.0, 0.0]), None, "x = {bad} was accepted");
            assert_eq!(vec3([0.0, bad, 0.0]), None, "y = {bad} was accepted");
            assert_eq!(vec3([0.0, 0.0, bad]), None, "z = {bad} was accepted");
        }
    }

    #[test]
    fn zero_is_not_positive_but_is_nonneg() {
        assert!(!positive(0.0));
        assert!(nonneg(0.0));
    }

    #[test]
    fn non_finite_values_are_rejected() {
        for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(!positive(v), "{v} passed positive");
            assert!(!nonneg(v), "{v} passed nonneg");
        }
    }

    /// A scene stores lengths as `f64`; the cast to `f32` is what physics
    /// actually sees, so the check has to happen after it.
    #[test]
    fn an_f64_that_flushes_to_zero_or_infinity_in_f32_is_rejected() {
        assert!(!positive(1.0e-300_f64 as f32));
        assert!(!positive(1.0e300_f64 as f32));
    }
}
