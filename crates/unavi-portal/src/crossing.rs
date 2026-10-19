use avian3d::prelude::{
    AngularVelocity,
    LinearVelocity,
};
use bevy::prelude::*;

use crate::{
    arrival::Arrival,
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

/// Where a segment passes through a portal's opening.
struct Hit {
    /// Plane-local, in metres from the portal's centre.
    point:      Vec2,
    /// Whether the segment starts on the side the portal's normal points to.
    from_front: bool,
}

/// Where the prev→curr segment crosses the portal plane within the opening.
/// Tests a sign change in plane-local `z` and the crossing point against the
/// rectangle, catching fast single-frame passes exactly at the plane.
fn segment_crosses_portal(prev_pos: Vec3, curr_pos: Vec3, frame: &PortalFrame) -> Option<Hit> {
    let prev_local = frame.local_from_world.transform_point3(prev_pos);
    let curr_local = frame.local_from_world.transform_point3(curr_pos);

    let from_front = prev_local.z >= 0.0;
    if from_front == (curr_local.z >= 0.0) {
        return None;
    }

    let s = prev_local.z / (prev_local.z - curr_local.z);
    let hit = prev_local.lerp(curr_local, s);

    (hit.x.abs() <= frame.half_size.x && hit.y.abs() <= frame.half_size.y).then_some(Hit {
        point: hit.truncate(),
        from_front,
    })
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
    /// The portal the body passed through.
    pub portal:              Entity,
    pub destination:         Entity,
    /// Where on `portal` the body passed, plane-local in metres.
    pub point:               Vec2,
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
    portals: Query<
        (
            Entity,
            &GlobalTransform,
            &PortalFrame,
            &Destination,
            &PortalState,
            Option<&Arrival>,
        ),
        With<Portal>,
    >,
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
            let inside_any_slab = portals.iter().any(|(_, t, frame, _, state, _)| {
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

        for (portal, source_transform, frame, destination, state, arrival) in &portals {
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
            let Some(hit) = segment_crosses_portal(prev_translation, curr_translation, frame)
            else {
                continue;
            };

            let (new_translation, new_rotation, transition_rotation) = if portal_destinations
                .contains(destination.0)
            {
                let Ok(dest_transform) = destinations.get(destination.0) else {
                    continue;
                };
                let m =
                    portal_transfer(source_transform, dest_transform) * transform.compute_affine();
                let (_, rotation, translation) = m.to_scale_rotation_translation();
                let transition_rotation = rotation * transform.rotation.inverse();
                (translation, rotation, transition_rotation)
            } else {
                let Some(arrival) = arrival else {
                    continue;
                };
                let transition_rotation = arrival.rotation_from(source_transform, hit.from_front);
                (
                    arrival.translation,
                    transition_rotation * transform.rotation,
                    transition_rotation,
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
                portal,
                destination: dest_entity,
                point: hit.point,
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
