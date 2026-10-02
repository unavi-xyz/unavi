//! Validated collider construction. Nothing here builds a shape from a
//! dimension that failed [`finite`](crate::finite).

use avian3d::prelude::{
    Collider,
    Position,
    Rotation,
};
use bevy::prelude::*;

use crate::finite::{
    nonnegative_length,
    positive_length,
};

#[must_use]
pub fn sphere(radius: f32) -> Option<Collider> {
    if !positive_length(radius) {
        warn!("collider sphere: radius must be positive (got {radius})");
        return None;
    }
    Some(Collider::sphere(radius))
}

#[must_use]
pub fn capsule(radius: f32, height: f32) -> Option<Collider> {
    if !positive_length(radius) {
        warn!("collider capsule: radius must be positive (got {radius})");
        return None;
    }
    if !nonnegative_length(height) {
        warn!("collider capsule: height must be non-negative (got {height})");
        return None;
    }
    Some(Collider::capsule(radius, height))
}

#[must_use]
pub fn cuboid(x: f32, y: f32, z: f32) -> Option<Collider> {
    if !positive_length(x) || !positive_length(y) || !positive_length(z) {
        warn!("collider cuboid: all dimensions must be positive (got {x}, {y}, {z})");
        return None;
    }
    Some(Collider::cuboid(x, y, z))
}

#[must_use]
pub fn cylinder(radius: f32, height: f32) -> Option<Collider> {
    if !positive_length(radius) {
        warn!("collider cylinder: radius must be positive (got {radius})");
        return None;
    }
    if !nonnegative_length(height) {
        warn!("collider cylinder: height must be non-negative (got {height})");
        return None;
    }
    Some(Collider::cylinder(radius, height))
}

/// Margin trimmed from the body capsule's radius for the sensor shape a
/// character controller casts against, so the sensor never catches on the
/// body's own hull.
pub const SENSOR_MARGIN: f32 = 0.01;

/// A rig's body capsule, plus the slightly narrower sensor cylinder a
/// character controller (Tnua) casts down with, built from the one
/// radius/height pair so [`SENSOR_MARGIN`] is applied once.
///
/// `total_height` is passed straight through to [`capsule`]'s height
/// parameter, matching how `unavi-agent` sizes the rig today; it is not
/// corrected for the capsule's hemispheres.
#[must_use]
pub fn rig_capsule(radius: f32, total_height: f32) -> Option<(Collider, Collider)> {
    let body = capsule(radius, total_height)?;
    let sensor = cylinder(radius - SENSOR_MARGIN, 0.0)?;
    Some((body, sensor))
}

/// Adds `collider` with its physics pose seeded from `seed`, or parks it as
/// [`Parked`](crate::degenerate::Parked) when `seed` is degenerate.
///
/// Avian's insert hook scales the shape by the entity's `GlobalTransform` and
/// reads `Position`/`Rotation`, whose defaults are `PLACEHOLDER` (`MAX`). A
/// collider added before transform propagation must carry its own pose; `seed`
/// is the caller's composition of the transform chain.
pub fn insert_collider(
    commands: &mut Commands,
    entity: Entity,
    collider: Collider,
    seed: &Transform,
) {
    if crate::degenerate::transform_is_valid(seed) {
        commands.entity(entity).insert((
            collider,
            Position(seed.translation),
            Rotation(seed.rotation),
        ));
    } else {
        commands.entity(entity).insert(crate::degenerate::Parked {
            collider: Some(collider),
            body:     None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        capsule,
        cuboid,
        cylinder,
        rig_capsule,
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

    #[test]
    fn rig_capsule_builds_both_shapes_for_a_sane_rig() {
        let (body, sensor) = rig_capsule(0.3, 1.7).expect("sane rig dimensions");
        let _ = (body, sensor);
    }

    /// A radius at or under the sensor margin leaves nothing for the sensor
    /// cylinder, so the whole pair is refused rather than built lopsided.
    #[test]
    fn rig_capsule_refuses_a_radius_too_small_for_the_sensor_margin() {
        assert!(rig_capsule(0.01, 1.7).is_none());
        assert!(rig_capsule(0.0, 1.7).is_none());
    }
}
