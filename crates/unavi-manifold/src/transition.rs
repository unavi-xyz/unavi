use avian3d::prelude::{
    AngularVelocity,
    LinearVelocity,
};
use bevy::prelude::*;

use crate::{
    GluedTo,
    ManifoldBody,
    PrevTranslation,
    SEAM_DEPTH,
    Seam,
    SeamLatch,
    SeamSize,
    SeamState,
    seam_transfer,
};

/// Whether the prev→curr segment crosses the seam plane within the opening.
///
/// Tests for a sign change in the plane-local `z` and checks the crossing point
/// against the rectangle, so the teleport fires exactly at the plane and fast
/// single-frame passes are still caught.
fn segment_crosses_seam(
    prev_pos: Vec3,
    curr_pos: Vec3,
    seam_transform: &GlobalTransform,
    size: SeamSize,
) -> bool {
    let inv = seam_transform.affine().inverse();
    let prev_local = inv.transform_point3(prev_pos);
    let curr_local = inv.transform_point3(curr_pos);

    if (prev_local.z >= 0.0) == (curr_local.z >= 0.0) {
        return false;
    }

    let s = prev_local.z / (prev_local.z - curr_local.z);
    let hit = prev_local.lerp(curr_local, s);

    hit.x.abs() <= size.width / 2.0 && hit.y.abs() <= size.height / 2.0
}

/// Whether `pos` lies inside the seam's overlap slab.
fn point_in_slab(pos: Vec3, seam_transform: &GlobalTransform, size: SeamSize) -> bool {
    let local = seam_transform.affine().inverse().transform_point3(pos);
    local.x.abs() <= size.width / 2.0
        && local.y.abs() <= size.height / 2.0
        && local.z.abs() <= SEAM_DEPTH / 2.0
}

#[derive(EntityEvent)]
pub struct CrossedSeam {
    pub entity:              Entity,
    pub destination:         Entity,
    pub transition_rotation: Quat,
}

/// Rotate a crossing body's velocity by the gluing rotation, so momentum
/// carries through the seam.
pub(crate) fn carry_momentum(
    event: On<CrossedSeam>,
    mut bodies: Query<(&mut LinearVelocity, &mut AngularVelocity)>,
) {
    let Ok((mut linear, mut angular)) = bodies.get_mut(event.entity) else {
        return;
    };
    let rotation = event.transition_rotation;
    linear.0 = rotation * linear.0;
    angular.0 = rotation * angular.0;
}

pub(crate) fn apply_seam_crossings(
    mut commands: Commands,
    mut travelers: Query<
        (Entity, &mut SeamLatch, &mut Transform, &mut PrevTranslation),
        (With<ManifoldBody>, Without<Seam>),
    >,
    seams: Query<(&GlobalTransform, &SeamSize, &GluedTo, &SeamState), With<Seam>>,
    destinations: Query<&GlobalTransform, Without<ManifoldBody>>,
    seam_destinations: Query<(), With<Seam>>,
) {
    // Runs before transform propagation so a teleport reaches the body's
    // descendants (the eye camera) the same frame, avoiding a one-frame view of
    // the space behind the seam. The body's parent chain is identity, so its
    // `Transform` is the current world pose.
    for (entity, mut latch, mut transform, mut prev) in &mut travelers {
        let curr_translation = transform.translation;

        // Seed prev; an unset (0,0,0) reads as a false crossing.
        if prev.0 == Vec3::ZERO {
            prev.0 = curr_translation;
            continue;
        }

        let prev_translation = prev.0;

        // Stay latched until clear of all slabs, else the landing slab re-fires.
        let inside_any_slab = seams.iter().any(|(t, size, _, state)| {
            *state == SeamState::Open && point_in_slab(curr_translation, t, *size)
        });
        if latch.0 {
            if !inside_any_slab {
                latch.0 = false;
            }
            prev.0 = curr_translation;
            continue;
        }

        let mut teleported = false;

        for (source_transform, size, destination, state) in &seams {
            if *state != SeamState::Open {
                continue;
            }
            if !segment_crosses_seam(prev_translation, curr_translation, source_transform, *size) {
                continue;
            }

            let Ok(dest_transform) = destinations.get(destination.0) else {
                continue;
            };

            let dest_is_seam = seam_destinations.contains(destination.0);

            let (new_translation, new_rotation, transition_rotation) = if dest_is_seam {
                let m =
                    seam_transfer(source_transform, dest_transform) * transform.compute_affine();
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

            prev.0 = transform.translation;
            latch.0 = true;
            teleported = true;

            let dest_entity = destination.0;
            commands.entity(entity).trigger(move |entity| CrossedSeam {
                entity,
                destination: dest_entity,
                transition_rotation,
            });

            break;
        }

        if !teleported {
            prev.0 = curr_translation;
        }
    }
}
