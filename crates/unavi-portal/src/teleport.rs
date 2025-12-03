use bevy::prelude::*;

use crate::{
    Portal, PortalBounds, PortalDestination, PortalTraveler, PrevTranslation, TravelCooldown,
};

#[derive(Debug, Clone, Copy)]
enum PortalEntrySide {
    Front,
    Back,
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

#[derive(EntityEvent)]
pub struct PortalTeleport {
    pub entity: Entity,
    pub delta_rotation: Quat,
}

const EXTRA_SPAWN_OFFSET: f32 = 0.005;

pub(crate) fn handle_traveler_teleport(
    mut commands: Commands,
    time: Res<Time>,
    mut travelers: Query<
        (
            Entity,
            &mut TravelCooldown,
            &mut Transform,
            &mut GlobalTransform,
            &mut PrevTranslation,
        ),
        (With<PortalTraveler>, Without<Portal>),
    >,
    portals: Query<(&GlobalTransform, &PortalBounds, &PortalDestination), With<Portal>>,
    destination_portals: Query<&GlobalTransform, With<Portal>>,
) {
    let elapsed = time.elapsed();

    for (entity, mut cooldown, mut transform, traveler_transform, mut prev) in &mut travelers {
        let curr_translation = traveler_transform.translation();

        // Initialize prev on first frame to avoid false teleport from (0,0,0).
        if prev.0 == Vec3::ZERO {
            prev.0 = curr_translation;
            continue;
        }

        let prev_translation = prev.0;

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

        for (source_transform, bounds, destination) in &portals {
            let Some(entry_side) = check_box_entry_with_side(
                prev_translation,
                curr_translation,
                source_transform,
                bounds,
            ) else {
                continue;
            };

            let Ok(dest_transform) = destination_portals.get(destination.0) else {
                continue;
            };

            // Apply offset away from portal to avoid spam teleports.
            let out_dir = match entry_side {
                PortalEntrySide::Front => dest_transform.forward(),
                PortalEntrySide::Back => dest_transform.back(),
            };

            let bounds_d = bounds.depth / 2.0;
            let min_spawn = bounds_d + EXTRA_SPAWN_OFFSET;
            let offset = out_dir * min_spawn;

            let flip_rot = Quat::from_rotation_y(std::f32::consts::PI);
            let flip_matrix = Mat4::from_quat(flip_rot);
            let new_traveler_transform = dest_transform.to_matrix()
                * flip_matrix
                * source_transform.to_matrix().inverse()
                * traveler_transform.to_matrix();

            let (_, rotation, translation) = new_traveler_transform.to_scale_rotation_translation();

            transform.translation = translation + offset;
            transform.rotation = rotation;

            prev.0 = transform.translation;
            cooldown.last_travel = Some(elapsed);

            let portal_delta = dest_transform.rotation() * source_transform.rotation().inverse();
            let delta_rotation = portal_delta * flip_rot;

            commands.entity(entity).trigger(|entity| PortalTeleport {
                entity,
                delta_rotation,
            });

            break;
        }

        prev.0 = traveler_transform.translation();
    }
}
