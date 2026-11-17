//! Where a thing stands: a few metres from spawn, on the side away from the
//! gate, facing spawn.

use wired_guest::math::Vec3;

/// The spawn point's radius in the space asset.
const SPAWN_RADIUS: f32 = 4.0;

/// Nearest distance from spawn, and how much farther a thing may stand.
const NEAREST: f32 = SPAWN_RADIUS + 2.0;
const SPREAD: f32 = 3.0;
/// Half the arc, in radians from the direction opposite the gate.
const SWEEP: f32 = 1.92;

#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub position: Vec3,
    /// Rotation about Y that turns the thing's +Z side toward spawn.
    pub yaw:      f32,
}

/// A word of the id as a fraction in `[0, 1)`.
fn unit(word: u64) -> f32 {
    (word >> 40) as f32 / (1_u64 << 24) as f32
}

pub fn placement(id: (u64, u64, u64, u64)) -> Pose {
    let distance = unit(id.1).mul_add(SPREAD, NEAREST);
    let angle = unit(id.2).mul_add(2.0, -1.0) * SWEEP;
    let position = Vec3::new(distance * angle.sin(), 0.0, distance * angle.cos());
    Pose {
        position,
        yaw: (-position.x).atan2(-position.z),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::GROUND_SIZE;

    /// The gate's prim in the space asset.
    const GATE_AT: Vec3 = Vec3::new(0.0, 0.0, -10.0);

    fn ids() -> impl Iterator<Item = (u64, u64, u64, u64)> {
        (1..500_u64).map(|i| {
            let word = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            (word, word.rotate_left(13), word.rotate_left(29), !word)
        })
    }

    #[test]
    fn placement_follows_the_id() {
        let id = (5, 6, 7, 8);
        assert_eq!(placement(id).position, placement(id).position);
        assert!(ids().any(|other| placement(other).position != placement(id).position));
    }

    #[test]
    fn a_thing_stands_on_the_slab_clear_of_spawn_and_the_gate() {
        let edge = GROUND_SIZE / 2.0 - 2.0;
        for id in ids() {
            let at = placement(id).position;
            assert!(
                at.x.abs() <= edge && at.z.abs() <= edge,
                "{id:?} stands at {at}"
            );
            assert!(at.length() > SPAWN_RADIUS + 1.5, "{id:?} stands at {at}");
            assert!(at.distance(GATE_AT) > 6.0, "{id:?} stands at {at}");
        }
    }

    #[test]
    fn a_thing_faces_spawn() {
        for id in ids() {
            let pose = placement(id);
            let toward = Vec3::new(pose.yaw.sin(), 0.0, pose.yaw.cos());
            let spawn = (-pose.position).normalize();
            assert!(toward.distance(spawn) < 1.0e-3, "{id:?} faces {toward}");
        }
    }
}
