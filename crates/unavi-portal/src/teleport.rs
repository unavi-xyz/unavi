use std::f32::consts::PI;

use bevy::{math::Affine3A, prelude::*};

use crate::{
    Portal, PortalBounds, PortalBoxState, PortalDestination, PortalTraveler, PrevTranslation,
    TravelCooldown,
};

const MIN_SPAWN_DISTANCE: f32 = 0.03;

#[derive(Debug, Clone, Copy)]
enum PortalEntrySide {
    Front,
    Back,
}

/// Check if a point is inside the portal's 3D box bounds.
fn is_inside_portal_box(
    point: Vec3,
    portal_transform: &GlobalTransform,
    bounds: &PortalBounds,
) -> bool {
    let portal_affine = portal_transform.affine();
    let local_point = portal_affine.inverse().transform_point3(point);

    let half_width = bounds.width / 2.0;
    let half_height = bounds.height / 2.0;
    let half_depth = bounds.depth / 2.0;

    local_point.x.abs() <= half_width
        && local_point.y.abs() <= half_height
        && local_point.z.abs() <= half_depth
}

const EPSILON: f32 = 1e-4;

/// Check if the line segment from `prev_pos` to `curr_pos` intersects the portal box.
/// Uses ray-box intersection to handle fast movement that might pass through entirely.
fn check_box_entry_with_side(
    prev_pos: Vec3,
    curr_pos: Vec3,
    portal_transform: &GlobalTransform,
    bounds: &PortalBounds,
) -> Option<PortalEntrySide> {
    let portal_affine = portal_transform.affine();
    let prev_local = portal_affine.inverse().transform_point3(prev_pos);
    let curr_local = portal_affine.inverse().transform_point3(curr_pos);

    let half_width = bounds.width / 2.0;
    let half_height = bounds.height / 2.0;
    let half_depth = bounds.depth / 2.0;

    let ray_dir = curr_local - prev_local;
    let ray_length = ray_dir.length();

    if ray_length < 1e-6 {
        return None;
    }

    let ray_dir_norm = ray_dir / ray_length;

    let inv_dir = Vec3::new(
        1.0 / ray_dir_norm.x,
        1.0 / ray_dir_norm.y,
        1.0 / ray_dir_norm.z,
    );

    let t1 = (-half_width - prev_local.x) * inv_dir.x;
    let t2 = (half_width - prev_local.x) * inv_dir.x;
    let t3 = (-half_height - prev_local.y) * inv_dir.y;
    let t4 = (half_height - prev_local.y) * inv_dir.y;
    let t5 = (-half_depth - prev_local.z) * inv_dir.z;
    let t6 = (half_depth - prev_local.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax || tmin > ray_length {
        return None;
    }

    let entry_t = tmin.max(0.0);

    if (entry_t - t5).abs() < EPSILON {
        Some(PortalEntrySide::Front)
    } else if (entry_t - t6).abs() < EPSILON {
        Some(PortalEntrySide::Back)
    } else if prev_local.z < 0.0 {
        Some(PortalEntrySide::Front)
    } else {
        Some(PortalEntrySide::Back)
    }
}

pub fn handle_traveler_teleport(
    time: Res<Time>,
    mut travelers: Query<
        (
            &mut TravelCooldown,
            &mut Transform,
            &mut GlobalTransform,
            &mut PrevTranslation,
            &mut PortalBoxState,
        ),
        (With<PortalTraveler>, Without<Portal>),
    >,
    portals: Query<(&GlobalTransform, &PortalBounds, &PortalDestination), With<Portal>>,
    destination_portals: Query<&GlobalTransform, With<Portal>>,
) {
    let elapsed = time.elapsed();

    for (mut cooldown, mut transform, mut global_transform, mut prev, mut box_state) in
        &mut travelers
    {
        let prev_translation = prev.0;
        let curr_translation = global_transform.translation();

        if let Some(last_travel) = &cooldown.last_travel {
            if elapsed
                .checked_sub(*last_travel)
                .expect("elapsed time greater than last travel time")
                < cooldown.duration
            {
                prev.0 = curr_translation;
                continue;
            }

            cooldown.last_travel = None;
        }

        let mut currently_inside_any = false;
        for (portal_transform, bounds, _) in &portals {
            if is_inside_portal_box(curr_translation, portal_transform, bounds) {
                currently_inside_any = true;
                break;
            }
        }

        if !currently_inside_any {
            box_state.current_box = None;
        }

        if box_state.current_box.is_some() {
            prev.0 = curr_translation;
            continue;
        }

        for (portal_transform, bounds, destination) in &portals {
            let Some(entry_side) = check_box_entry_with_side(
                prev_translation,
                curr_translation,
                portal_transform,
                bounds,
            ) else {
                continue;
            };

            let Ok(destination_transform) = destination_portals.get(destination.0) else {
                continue;
            };

            let portal_local = portal_transform.affine().inverse() * global_transform.affine();
            let rotated =
                Affine3A::from_rotation_translation(Quat::from_rotation_y(PI), Vec3::ZERO)
                    * portal_local;

            let max_spawn = bounds.depth / 2.0;
            let min_spawn = MIN_SPAWN_DISTANCE.min(max_spawn);

            let spawn_z = match entry_side {
                PortalEntrySide::Front => -rotated.translation.z.abs().clamp(min_spawn, max_spawn),
                PortalEntrySide::Back => rotated.translation.z.abs().clamp(min_spawn, max_spawn),
            };

            let translation = Vec3::new(rotated.translation.x, rotated.translation.y, spawn_z);
            let rotation = Quat::from_mat3(&rotated.matrix3.into());

            let corrected = Affine3A::from_rotation_translation(rotation, translation);

            let new_transform = GlobalTransform::from(destination_transform.affine() * corrected);

            *global_transform = new_transform;
            *transform = new_transform.into();
            prev.0 = new_transform.translation();

            box_state.current_box = Some(destination.0);
            cooldown.last_travel = Some(elapsed);
            break;
        }

        prev.0 = global_transform.translation();
    }
}
