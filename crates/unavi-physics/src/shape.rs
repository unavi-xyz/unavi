use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::finite::{
    nonneg,
    positive,
};

#[must_use]
pub fn sphere(radius: f32) -> Option<Collider> {
    if !positive(radius) {
        warn!("collider sphere: radius must be positive (got {radius})");
        return None;
    }
    Some(Collider::sphere(radius))
}

#[must_use]
pub fn capsule(radius: f32, height: f32) -> Option<Collider> {
    if !positive(radius) {
        warn!("collider capsule: radius must be positive (got {radius})");
        return None;
    }
    if !nonneg(height) {
        warn!("collider capsule: height must be non-negative (got {height})");
        return None;
    }
    Some(Collider::capsule(radius, height))
}

#[must_use]
pub fn cuboid(x: f32, y: f32, z: f32) -> Option<Collider> {
    if !positive(x) || !positive(y) || !positive(z) {
        warn!("collider cuboid: all dimensions must be positive (got {x}, {y}, {z})");
        return None;
    }
    Some(Collider::cuboid(x, y, z))
}

#[must_use]
pub fn cylinder(radius: f32, height: f32) -> Option<Collider> {
    if !positive(radius) {
        warn!("collider cylinder: radius must be positive (got {radius})");
        return None;
    }
    if !nonneg(height) {
        warn!("collider cylinder: height must be non-negative (got {height})");
        return None;
    }
    Some(Collider::cylinder(radius, height))
}

#[cfg(test)]
mod tests {
    use super::{
        capsule,
        cuboid,
        cylinder,
        sphere,
    };

    #[test]
    fn valid_dimensions_build_a_collider() {
        assert!(sphere(0.5).is_some());
        assert!(capsule(0.3, 1.0).is_some());
        assert!(cuboid(1.0, 2.0, 3.0).is_some());
        assert!(cylinder(0.4, 0.0).is_some());
    }

    #[test]
    fn a_nan_dimension_builds_nothing() {
        let nan = f32::NAN;
        assert!(sphere(nan).is_none());
        assert!(capsule(nan, 1.0).is_none());
        assert!(capsule(0.3, nan).is_none());
        assert!(cuboid(1.0, nan, 1.0).is_none());
        assert!(cylinder(nan, 1.0).is_none());
        assert!(cylinder(0.4, nan).is_none());
    }

    /// A VRM whose eye and shoulder bones coincide yields a zero radius, and a
    /// zero-radius capsule has no surface for the solver to resolve against.
    #[test]
    fn a_zero_radius_builds_nothing() {
        assert!(sphere(0.0).is_none());
        assert!(capsule(0.0, 1.0).is_none());
        assert!(cylinder(0.0, 1.0).is_none());
        assert!(cuboid(1.0, 0.0, 1.0).is_none());
    }

    /// A capsule or cylinder of zero height is a sphere or a disc, both of
    /// which the solver handles.
    #[test]
    fn a_zero_height_is_accepted() {
        assert!(capsule(0.5, 0.0).is_some());
        assert!(cylinder(0.5, 0.0).is_some());
    }

    #[test]
    fn a_negative_dimension_builds_nothing() {
        assert!(sphere(-1.0).is_none());
        assert!(capsule(0.3, -1.0).is_none());
        assert!(cuboid(-1.0, 1.0, 1.0).is_none());
        assert!(cylinder(0.3, -1.0).is_none());
    }
}
