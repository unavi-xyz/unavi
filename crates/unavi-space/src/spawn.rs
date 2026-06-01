use bevy::prelude::*;
use bevy_hsd::attributes::spawn::SpawnPoint;
use rand::Rng;

use crate::Space;

/// Pick a spawn position for `space`, expressed in the space's local frame.
///
/// Each [`SpawnPoint`] inside the space defines a horizontal circle around its
/// prim origin; we pick one such circle uniformly at random and then a uniform
/// random point inside it. Returns `None` when no spawn point belongs to the
/// space — callers should fall back to the space origin.
#[must_use]
pub fn pick_spawn(
    space: Entity,
    spawn_points: &Query<(&SpawnPoint, &GlobalTransform, &ChildOf)>,
    parents: &Query<&ChildOf>,
    spaces: &Query<&GlobalTransform, With<Space>>,
) -> Option<Vec3> {
    let space_inv = spaces.get(space).ok()?.affine().inverse();
    let candidates: Vec<(Vec3, f32)> = spawn_points
        .iter()
        .filter(|(_, _, child_of)| belongs_to_space(child_of.parent(), space, parents))
        .map(|(s, gt, _)| (space_inv.transform_point3(gt.translation()), s.radius))
        .collect();
    if candidates.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    let (center, radius) = candidates[rng.random_range(0..candidates.len())];
    if radius <= 0.0 {
        return Some(center);
    }

    let theta = rng.random_range(0.0..std::f32::consts::TAU);
    let r = radius * rng.random_range(0.0_f32..1.0).sqrt();
    Some(Vec3::new(
        r.mul_add(theta.cos(), center.x),
        center.y,
        r.mul_add(theta.sin(), center.z),
    ))
}

fn belongs_to_space(mut current: Entity, space: Entity, parents: &Query<&ChildOf>) -> bool {
    loop {
        if current == space {
            return true;
        }
        match parents.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => return false,
        }
    }
}
