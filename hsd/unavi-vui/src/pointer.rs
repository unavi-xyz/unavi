use wired_math::types::{
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    view::Aim,
    wired::input::{
        context::pointers,
        types::{
            Pointer,
            PointerKind,
            Ray,
        },
    },
};

/// Where the viewer is, and where they are pointing. On desktop these agree;
/// in VR a hand aims somewhere the head is not looking, which is the whole
/// reason they are two things.
#[derive(Debug, Clone, Copy)]
pub struct Gaze {
    pub eye: Transform,
    pub ray: Ray,
}

impl Gaze {
    /// Reads this frame's pointers, falling back to the eye's own forward ray
    /// when the host offers none.
    #[must_use]
    pub fn read(eye: &Transform) -> Self {
        let ray = pointers()
            .ok()
            .and_then(|pointers| leading(&pointers))
            .unwrap_or_else(|| eye_ray(eye));
        Self { eye: *eye, ray }
    }
}

/// How busy a hand is, whichever way it is acting.
const fn effort(pointer: &Pointer) -> f32 {
    pointer.trigger.max(pointer.grip)
}

/// The pointer attention follows: whichever is acting hardest, and failing
/// that the one a platform actually has.
fn leading(pointers: &[Pointer]) -> Option<Ray> {
    let active = pointers.iter().filter(|pointer| pointer.active);

    let acting = active
        .clone()
        .filter(|pointer| effort(pointer) > 0.0)
        .max_by(|a, b| effort(a).total_cmp(&effort(b)));

    acting
        .or_else(|| {
            [
                PointerKind::Screen,
                PointerKind::RightHand,
                PointerKind::LeftHand,
            ]
            .into_iter()
            .find_map(|kind| active.clone().find(|pointer| pointer.kind == kind))
        })
        .map(|pointer| pointer.ray)
}

fn eye_ray(eye: &Transform) -> Ray {
    Ray {
        origin: eye.translation,
        dir:    eye.rotation * Vec3::new(0.0, 0.0, -1.0),
    }
}

/// Where a pointer's ray crosses an orbit's aim plane, in the plane's own
/// coordinates and in the world. The plane stands `lift` in front of the
/// orbit.
#[must_use]
pub fn aim(ray: &Ray, anchor: &Transform, lift: f32) -> Option<Aim> {
    let normal = anchor.rotation * Vec3::Z;
    let origin = anchor.translation + normal * lift;

    let denominator = ray.dir.dot(normal);
    if denominator.abs() < 1.0e-6 {
        return None;
    }

    let distance = (origin - ray.origin).dot(normal) / denominator;
    if distance < 0.0 {
        return None;
    }

    let world = ray.origin + ray.dir * distance;
    let relative = world - origin;
    Some(Aim {
        local: Vec2::new(
            relative.dot(anchor.rotation * Vec3::X),
            relative.dot(anchor.rotation * Vec3::Y),
        ),
        world,
    })
}

/// A free grab point `depth` along the pointer's ray. A held mote follows
/// this rather than the aim plane.
#[must_use]
pub fn hand(ray: &Ray, depth: f32) -> Vec3 {
    ray.origin + ray.dir * depth
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use wired_math::types::Quat;

    use super::*;

    /// A pointer at the origin aimed down -Z, and an orbit a metre away
    /// facing back at it.
    fn ray() -> Ray {
        Ray {
            origin: Vec3::ZERO,
            dir:    Vec3::new(0.0, 0.0, -1.0),
        }
    }

    fn anchor() -> Transform {
        Transform {
            translation: Vec3::new(0.0, 0.0, -1.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }
    }

    fn turned(radians: f32) -> Ray {
        let rotation = Quat::new(0.0, (radians * 0.5).sin(), 0.0, (radians * 0.5).cos());
        Ray {
            origin: Vec3::ZERO,
            dir:    rotation * Vec3::new(0.0, 0.0, -1.0),
        }
    }

    #[test]
    fn looking_straight_at_an_orbit_aims_at_its_centre() {
        let aim = aim(&ray(), &anchor(), 0.0).expect("aim");
        assert!(aim.local.length() < 1.0e-5);
        assert!((aim.world - anchor().translation).length() < 1.0e-5);
    }

    #[test]
    fn turning_away_moves_the_aim_across_the_plane() {
        let aim = aim(&turned(0.2), &anchor(), 0.0).expect("aim");
        assert!(aim.local.x < 0.0, "turning left aims left of centre");
        assert!(aim.local.y.abs() < 1.0e-5);
    }

    #[test]
    fn an_orbit_behind_the_pointer_is_not_aimed_at() {
        assert!(aim(&turned(PI), &anchor(), 0.0).is_none());
    }

    #[test]
    fn a_ray_along_the_plane_crosses_it_nowhere() {
        assert!(aim(&turned(PI * 0.5), &anchor(), 0.0).is_none());
    }

    #[test]
    fn lift_moves_the_aim_plane_toward_the_pointer() {
        let flat = aim(&ray(), &anchor(), 0.0).expect("aim");
        let lifted = aim(&ray(), &anchor(), 0.1).expect("aim");
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
        let aim = aim(&turned(0.2), &rolled, 0.0).expect("aim");
        assert!(
            aim.local.x.abs() < 1.0e-5 && aim.local.y > 0.0,
            "a quarter-turned orbit reads a look to its left as a look along \
             its own up"
        );
    }

    #[test]
    fn the_hand_holds_its_depth_along_the_ray() {
        let ray = turned(0.4);
        let hand = hand(&ray, 1.5);
        assert!(((hand - ray.origin).length() - 1.5).abs() < 1.0e-4);
        assert!(
            hand.x < 0.0 && hand.z < 0.0,
            "the grab point follows where the pointer aims"
        );
    }

    #[test]
    fn a_pointer_pulling_hardest_is_the_one_attention_follows() {
        let pointer = |kind, grip, x: f32| Pointer {
            kind,
            active: true,
            ray: Ray {
                origin: Vec3::new(x, 0.0, 0.0),
                dir:    Vec3::new(0.0, 0.0, -1.0),
            },
            trigger: 0.0,
            grip,
            axis: Vec2::ZERO,
            hit: None,
        };

        let leading = leading(&[
            pointer(PointerKind::LeftHand, 0.2, -1.0),
            pointer(PointerKind::RightHand, 0.9, 1.0),
        ])
        .expect("a pointer");
        assert!((leading.origin.x - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn an_untracked_pointer_is_never_the_one_followed() {
        let mut screen = Pointer {
            kind:    PointerKind::Screen,
            active:  false,
            ray:     ray(),
            trigger: 0.0,
            grip:    1.0,
            axis:    Vec2::ZERO,
            hit:     None,
        };
        assert!(leading(&[screen]).is_none());

        screen.active = true;
        assert!(leading(&[screen]).is_some());
    }
}
