use wired_math::types::{
    Transform,
    Vec2,
    Vec3,
};

use crate::view::Aim;

#[must_use]
pub fn forward(eye: &Transform) -> Vec3 {
    eye.rotation * Vec3::new(0.0, 0.0, -1.0)
}

/// Where a pointer's ray crosses an orbit's aim plane, in the plane's own
/// coordinates and in the world.
///
/// The plane stands `lift` in front of the orbit rather than through it. That
/// is also where the orbit's hit surface goes, so the host's reticle — which
/// rides on whatever the ray struck — glides across the face of the dial
/// instead of sinking into whichever mote it is on.
#[must_use]
pub fn aim(eye: &Transform, anchor: &Transform, lift: f32) -> Option<Aim> {
    let normal = anchor.rotation * Vec3::Z;
    let origin = anchor.translation + normal * lift;

    let direction = forward(eye);
    let denominator = direction.dot(normal);
    if denominator.abs() < 1.0e-6 {
        return None;
    }

    let distance = (origin - eye.translation).dot(normal) / denominator;
    if distance < 0.0 {
        return None;
    }

    let world = eye.translation + direction * distance;
    let relative = world - origin;
    Some(Aim {
        local: Vec2::new(
            relative.dot(anchor.rotation * Vec3::X),
            relative.dot(anchor.rotation * Vec3::Y),
        ),
        world,
    })
}

/// A free grab point `depth` along the pointer's ray.
///
/// A held mote follows this rather than the aim plane, so a pickup keeps the
/// depth it was grabbed at and travels in three dimensions instead of sliding
/// around a pane of glass.
#[must_use]
pub fn hand(eye: &Transform, depth: f32) -> Vec3 {
    eye.translation + forward(eye) * depth
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use wired_math::types::Quat;

    use super::*;

    /// An eye at the origin looking down -Z, and an orbit a metre away facing
    /// back at it.
    fn eye() -> Transform {
        Transform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }
    }

    fn anchor() -> Transform {
        Transform {
            translation: Vec3::new(0.0, 0.0, -1.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }
    }

    fn turned(eye: &Transform, radians: f32) -> Transform {
        Transform {
            rotation: Quat::new(0.0, (radians * 0.5).sin(), 0.0, (radians * 0.5).cos()),
            ..*eye
        }
    }

    #[test]
    fn looking_straight_at_an_orbit_aims_at_its_centre() {
        let aim = aim(&eye(), &anchor(), 0.0).expect("aim");
        assert!(aim.local.length() < 1.0e-5);
        assert!((aim.world - anchor().translation).length() < 1.0e-5);
    }

    #[test]
    fn turning_away_moves_the_aim_across_the_plane() {
        let aim = aim(&turned(&eye(), 0.2), &anchor(), 0.0).expect("aim");
        assert!(aim.local.x < 0.0, "turning left aims left of centre");
        assert!(aim.local.y.abs() < 1.0e-5);
    }

    #[test]
    fn an_orbit_behind_the_pointer_is_not_aimed_at() {
        assert!(aim(&turned(&eye(), PI), &anchor(), 0.0).is_none());
    }

    #[test]
    fn a_ray_along_the_plane_crosses_it_nowhere() {
        assert!(aim(&turned(&eye(), PI * 0.5), &anchor(), 0.0).is_none());
    }

    #[test]
    fn lift_moves_the_aim_plane_toward_the_pointer() {
        let flat = aim(&eye(), &anchor(), 0.0).expect("aim");
        let lifted = aim(&eye(), &anchor(), 0.1).expect("aim");
        assert!(
            lifted.world.z > flat.world.z,
            "the lifted plane must stand in front of the orbit, or the \
             reticle sinks into the mote it is on"
        );
        assert!(
            lifted.local.length() < 1.0e-5,
            "lift is measured from the plane's own origin, so the centre is \
             still the centre and the hit surface can match the reach exactly"
        );
    }

    #[test]
    fn the_aim_is_measured_in_the_orbits_own_frame() {
        let rolled = Transform {
            rotation: Quat::new(0.0, 0.0, (PI * 0.25).sin(), (PI * 0.25).cos()),
            ..anchor()
        };
        let aim = aim(&turned(&eye(), 0.2), &rolled, 0.0).expect("aim");
        assert!(
            aim.local.x.abs() < 1.0e-5 && aim.local.y > 0.0,
            "a quarter-turned orbit reads a look to its left as a look along \
             its own up"
        );
    }

    #[test]
    fn the_hand_holds_its_depth_along_the_ray() {
        let eye = turned(&eye(), 0.4);
        let hand = hand(&eye, 1.5);
        assert!(((hand - eye.translation).length() - 1.5).abs() < 1.0e-4);
        assert!(
            hand.x < 0.0 && hand.z < 0.0,
            "the grab point follows where the pointer looks"
        );
    }
}
