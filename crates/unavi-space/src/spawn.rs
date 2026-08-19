use bevy::prelude::*;
use bevy_hsd::attributes::spawn::SpawnPoint;
use rand::Rng;
use unavi_util::hierarchy::descends_from;

/// A world position to spawn at inside `space`, which only sits at the origin
/// while it is the active one.
#[must_use]
pub fn pick_spawn(
    space: Entity,
    spawn_points: &Query<(&SpawnPoint, &GlobalTransform, &ChildOf)>,
    parents: &Query<&ChildOf>,
) -> Option<Vec3> {
    let candidates: Vec<(Vec3, f32)> = spawn_points
        .iter()
        .filter(|(_, _, child_of)| descends_from(child_of.parent(), space, parents))
        .map(|(s, gt, _)| (gt.translation(), s.radius))
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
