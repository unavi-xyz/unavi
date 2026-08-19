//! A non-finite or degenerate parameter reaches the solver intact, where it
//! spreads to every body sharing an island. Anything derived from a scene, a
//! script, or avatar geometry passes through here first.

use bevy::prelude::{
    Quat,
    Vec3,
};

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

/// A rotation, returned normalized, and only if it names one.
///
/// The zero quaternion is the case worth naming: it is the default a guest
/// gets from zeroed memory, it is not a rotation, and normalizing it is what
/// turns it into the NaN the solver spreads.
#[must_use]
pub fn quat(v: [f32; 4]) -> Option<Quat> {
    let q = Quat::from_array(v).normalize();
    q.is_finite().then_some(q)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{
        Quat,
        Vec3,
    };

    use super::{
        nonneg,
        positive,
        quat,
        vec3,
    };

    #[test]
    fn a_finite_vector_passes_through_unchanged() {
        assert_eq!(vec3([1.0, -2.0, 3.5]), Some(Vec3::new(1.0, -2.0, 3.5)));
        assert_eq!(vec3([0.0; 3]), Some(Vec3::ZERO));
    }

    #[test]
    fn a_single_non_finite_component_rejects_the_vector() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(vec3([bad, 0.0, 0.0]), None, "x = {bad} was accepted");
            assert_eq!(vec3([0.0, bad, 0.0]), None, "y = {bad} was accepted");
            assert_eq!(vec3([0.0, 0.0, bad]), None, "z = {bad} was accepted");
        }
    }

    #[test]
    fn a_rotation_comes_back_normalized() {
        let q = quat([0.0, 0.0, 0.0, 2.0]).expect("finite");
        assert!((q.length() - 1.0).abs() < 1.0e-6);
        assert!(q.abs_diff_eq(Quat::IDENTITY, 1.0e-6));
    }

    #[test]
    fn a_rotation_without_a_direction_is_rejected() {
        assert_eq!(quat([0.0; 4]), None, "the zero quaternion was accepted");
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(quat([bad, 0.0, 0.0, 1.0]), None, "{bad} was accepted");
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
