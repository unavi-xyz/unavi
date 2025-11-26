use std::f32::consts::PI;

use bevy::{math::Affine3A, prelude::*};

use crate::{
    Portal, PortalBounds, PortalBoxState, PortalDestination, PortalTraveler, PrevTranslation,
    TravelCooldown,
};

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

/// Check if movement represents entering the portal box from outside.
fn check_box_entry(
    prev_pos: Vec3,
    curr_pos: Vec3,
    portal_transform: &GlobalTransform,
    bounds: &PortalBounds,
) -> bool {
    let was_outside = !is_inside_portal_box(prev_pos, portal_transform, bounds);
    let is_inside = is_inside_portal_box(curr_pos, portal_transform, bounds);
    was_outside && is_inside
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
            if !check_box_entry(prev_translation, curr_translation, portal_transform, bounds) {
                continue;
            }

            let Ok(destination_transform) = destination_portals.get(destination.0) else {
                continue;
            };

            let new_transform = GlobalTransform::from(
                destination_transform.affine()
                    * Affine3A::from_rotation_translation(Quat::from_rotation_y(PI), Vec3::ZERO)
                    * portal_transform.affine().inverse()
                    * global_transform.affine(),
            );

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
