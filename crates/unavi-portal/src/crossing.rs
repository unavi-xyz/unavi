use avian3d::prelude::{
    AngularVelocity,
    LinearVelocity,
};
use bevy::prelude::*;

use crate::{
    body::{
        PortalBody,
        PortalLatch,
        PrevTranslation,
    },
    destination::Destination,
    portal::{
        PORTAL_DEPTH,
        Portal,
        PortalFrame,
        PortalState,
        portal_transfer,
    },
};

/// Closest possible distance from `center` to the segment `a`-`b`, computed
/// as a cheap lower bound (midpoint distance minus half the segment length)
/// rather than an exact point-segment distance. Good enough to cull a
/// portal that cannot possibly be crossed this frame before paying for the
/// exact local-space test.
fn min_distance_to_segment(center: Vec3, a: Vec3, b: Vec3) -> f32 {
    let midpoint = (a + b) * 0.5;
    let half_len = a.distance(b) / 2.0;
    (center.distance(midpoint) - half_len).max(0.0)
}

/// Whether the prev→curr segment crosses the portal plane within the
/// opening. Tests a sign change in plane-local `z` and the crossing point
/// against the rectangle, catching fast single-frame passes exactly at the
/// plane.
fn segment_crosses_portal(prev_pos: Vec3, curr_pos: Vec3, frame: &PortalFrame) -> bool {
    let prev_local = frame.local_from_world.transform_point3(prev_pos);
    let curr_local = frame.local_from_world.transform_point3(curr_pos);

    if (prev_local.z >= 0.0) == (curr_local.z >= 0.0) {
        return false;
    }

    let s = prev_local.z / (prev_local.z - curr_local.z);
    let hit = prev_local.lerp(curr_local, s);

    hit.x.abs() <= frame.half_size.x && hit.y.abs() <= frame.half_size.y
}

/// Whether `pos` lies inside the portal's overlap slab.
fn point_in_slab(pos: Vec3, frame: &PortalFrame) -> bool {
    let local = frame.local_from_world.transform_point3(pos);
    local.x.abs() <= frame.half_size.x
        && local.y.abs() <= frame.half_size.y
        && local.z.abs() <= PORTAL_DEPTH / 2.0
}

#[derive(EntityEvent)]
pub struct Crossed {
    pub entity:              Entity,
    pub destination:         Entity,
    pub transition_rotation: Quat,
}

/// Rotate a crossing body's velocity by the gluing rotation, so momentum
/// carries through the portal.
pub(crate) fn carry_momentum(
    event: On<Crossed>,
    mut bodies: Query<(&mut LinearVelocity, &mut AngularVelocity)>,
) {
    let Ok((mut linear, mut angular)) = bodies.get_mut(event.entity) else {
        return;
    };
    let rotation = event.transition_rotation;
    linear.0 = rotation * linear.0;
    angular.0 = rotation * angular.0;
}

pub fn apply_crossings(
    mut commands: Commands,
    mut travelers: Query<
        (
            Entity,
            &mut PortalLatch,
            &mut Transform,
            &mut PrevTranslation,
        ),
        (With<PortalBody>, Without<Portal>),
    >,
    portals: Query<(&GlobalTransform, &PortalFrame, &Destination, &PortalState), With<Portal>>,
    destinations: Query<&GlobalTransform, Without<PortalBody>>,
    portal_destinations: Query<(), With<Portal>>,
) {
    // Runs before transform propagation so a teleport reaches the body's eye
    // camera the same frame. The body's parent chain is identity (world
    // pose).
    for (entity, mut latch, mut transform, mut prev) in &mut travelers {
        let curr_translation = transform.translation;

        let Some(prev_translation) = prev.0 else {
            prev.0 = Some(curr_translation);
            continue;
        };

        // Stay latched until clear of all slabs, else the landing slab
        // re-fires.
        if latch.0 {
            let inside_any_slab = portals.iter().any(|(t, frame, _, state)| {
                *state == PortalState::Open
                    && curr_translation.distance_squared(t.translation())
                        <= (frame.bounding_radius + PORTAL_DEPTH).powi(2)
                    && point_in_slab(curr_translation, frame)
            });
            if !inside_any_slab {
                latch.0 = false;
            }
            prev.0 = Some(curr_translation);
            continue;
        }

        let mut teleported = false;

        for (source_transform, frame, destination, state) in &portals {
            if *state != PortalState::Open {
                continue;
            }
            let reach = frame.bounding_radius + PORTAL_DEPTH;
            if min_distance_to_segment(
                source_transform.translation(),
                prev_translation,
                curr_translation,
            ) > reach
            {
                continue;
            }
            if !segment_crosses_portal(prev_translation, curr_translation, frame) {
                continue;
            }

            let Ok(dest_transform) = destinations.get(destination.0) else {
                continue;
            };

            let dest_is_portal = portal_destinations.contains(destination.0);

            let (new_translation, new_rotation, transition_rotation) = if dest_is_portal {
                let m =
                    portal_transfer(source_transform, dest_transform) * transform.compute_affine();
                let (_, rotation, translation) = m.to_scale_rotation_translation();
                let transition_rotation = rotation * transform.rotation.inverse();
                (translation, rotation, transition_rotation)
            } else {
                (
                    dest_transform.translation(),
                    transform.rotation,
                    Quat::IDENTITY,
                )
            };

            transform.translation = new_translation;
            transform.rotation = new_rotation;

            prev.0 = Some(transform.translation);
            latch.0 = true;
            teleported = true;

            let dest_entity = destination.0;
            commands.entity(entity).trigger(move |entity| Crossed {
                entity,
                destination: dest_entity,
                transition_rotation,
            });

            break;
        }

        if !teleported {
            prev.0 = Some(curr_translation);
        }
    }
}

#[cfg(test)] mod tests;
