use std::f32::consts::PI;

use bevy::{math::Affine3A, prelude::*};

use crate::{
    Portal, PortalBounds, PortalDestination, PortalTraveler, PrevTranslation, TravelCooldown,
};

const EPSILON: f32 = 1e-6;

/// Check if a line segment crosses through a portal's bounded plane.
fn check_portal_crossing(
    prev_pos: Vec3,
    curr_pos: Vec3,
    portal_transform: &GlobalTransform,
    bounds: &PortalBounds,
) -> Option<Vec3> {
    let plane_point = portal_transform.translation();
    let plane_normal = portal_transform.rotation() * *bounds.normal;
    let line_dir = curr_pos - prev_pos;

    let denom = line_dir.dot(plane_normal);
    if denom.abs() < EPSILON {
        return None;
    }

    let t = (plane_point - prev_pos).dot(plane_normal) / denom;
    if !(0.0..=1.0).contains(&t) {
        return None;
    }

    let prev_dist = (prev_pos - plane_point).dot(plane_normal);
    let curr_dist = (curr_pos - plane_point).dot(plane_normal);
    if prev_dist.signum() == curr_dist.signum() {
        return None;
    }

    let intersection = prev_pos + t * line_dir;
    let portal_affine = portal_transform.affine();
    let local_point = portal_affine.inverse().transform_point3(intersection);

    let half_width = bounds.width / 2.0;
    let half_height = bounds.height / 2.0;

    if local_point.x.abs() <= half_width && local_point.y.abs() <= half_height {
        Some(intersection)
    } else {
        None
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
        ),
        (With<PortalTraveler>, Without<Portal>),
    >,
    portals: Query<(&GlobalTransform, &PortalBounds, &PortalDestination), With<Portal>>,
    destination_portals: Query<&GlobalTransform, With<Portal>>,
) {
    let elapsed = time.elapsed();

    for (mut cooldown, mut transform, mut global_transform, mut prev) in &mut travelers {
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

        for (portal_transform, bounds, destination) in &portals {
            let Some(_intersection) =
                check_portal_crossing(prev_translation, curr_translation, portal_transform, bounds)
            else {
                continue;
            };

            let Ok(destination_transform) = destination_portals.get(destination.0) else {
                continue;
            };

            let new_transform = GlobalTransform::from(
                destination_transform.affine()
                    * Affine3A::from_rotation_translation(Quat::from_rotation_y(PI), Vec3::ZERO)
                    * portal_transform.affine().inverse()
                    * global_transform.affine(),
            );

            info!(
                "Detected portal crossing, teleporting: {:?} -> {:?}",
                global_transform.translation(),
                new_transform.translation()
            );

            *global_transform = new_transform;
            *transform = new_transform.into();
            prev.0 = new_transform.translation();

            cooldown.last_travel = Some(elapsed);
            break;
        }

        prev.0 = global_transform.translation();
    }
}
