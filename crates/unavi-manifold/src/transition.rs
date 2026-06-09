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
    SeamSize,
    SeamState,
    TransitionCooldown,
};

#[derive(Debug, Clone, Copy)]
enum SeamEntrySide {
    Front,
    Back,
}

/// Check if the line segment from `prev_pos` to `curr_pos` intersects the
/// seam box. Uses ray-box intersection to handle fast movement that might
/// pass through entirely.
fn check_box_entry_with_side(
    prev_pos: Vec3,
    curr_pos: Vec3,
    seam_transform: &GlobalTransform,
    size: SeamSize,
) -> Option<SeamEntrySide> {
    let seam_affine = seam_transform.affine();
    let prev_local = seam_affine.inverse().transform_point3(prev_pos);
    let curr_local = seam_affine.inverse().transform_point3(curr_pos);

    let half_width = size.width / 2.0;
    let half_height = size.height / 2.0;
    let half_depth = SEAM_DEPTH / 2.0;

    let ray_dir = curr_local - prev_local;
    let ray_length = ray_dir.length();

    if ray_length < 1.0e-6 {
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

    if (entry_t - t5).abs() < f32::EPSILON {
        Some(SeamEntrySide::Front)
    } else if (entry_t - t6).abs() < f32::EPSILON {
        Some(SeamEntrySide::Back)
    } else if prev_local.z < 0.0 {
        Some(SeamEntrySide::Front)
    } else {
        Some(SeamEntrySide::Back)
    }
}

#[derive(EntityEvent)]
pub struct CrossedSeam {
    pub entity:              Entity,
    pub destination:         Entity,
    pub transition_rotation: Quat,
}

/// Carry a rigid body's momentum across a chart transition.
///
/// [`apply_seam_crossings`] maps the body's pose through the gluing
/// isometry `g = dest · Rπ · src⁻¹ ∈ SE(3)`. Velocities live in the tangent
/// space, so they transform by the rotational part of `g` alone: rotating
/// linear and angular velocity by `transition_rotation` preserves momentum
/// across the seam as a pure SE(3) action, applying uniformly to the player and
/// to any other dynamic body that traverses the seam.
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

const EXTRA_SPAWN_OFFSET: f32 = 0.005;

pub(crate) fn apply_seam_crossings(
    mut commands: Commands,
    time: Res<Time>,
    mut travelers: Query<
        (
            Entity,
            &mut TransitionCooldown,
            &mut Transform,
            &mut GlobalTransform,
            &mut PrevTranslation,
        ),
        (With<ManifoldBody>, Without<Seam>),
    >,
    seams: Query<(&GlobalTransform, &SeamSize, &GluedTo, &SeamState), With<Seam>>,
    destinations: Query<&GlobalTransform, Without<ManifoldBody>>,
    seam_destinations: Query<(), With<Seam>>,
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

        let mut teleported = false;

        for (source_transform, size, destination, state) in &seams {
            if *state != SeamState::Open {
                continue;
            }
            let Some(entry_side) = check_box_entry_with_side(
                prev_translation,
                curr_translation,
                source_transform,
                *size,
            ) else {
                continue;
            };

            let Ok(dest_transform) = destinations.get(destination.0) else {
                continue;
            };

            let dest_is_seam = seam_destinations.contains(destination.0);

            let (new_translation, new_rotation, transition_rotation) = if dest_is_seam {
                let out_dir = match entry_side {
                    SeamEntrySide::Front => dest_transform.forward(),
                    SeamEntrySide::Back => dest_transform.back(),
                };
                let min_spawn = SEAM_DEPTH / 2.0 + EXTRA_SPAWN_OFFSET;
                let offset = out_dir * min_spawn;

                let flip_rot = Quat::from_rotation_y(std::f32::consts::PI);
                let flip_matrix = Mat4::from_quat(flip_rot);
                let m = dest_transform.to_matrix()
                    * flip_matrix
                    * source_transform.to_matrix().inverse()
                    * traveler_transform.to_matrix();
                let (_, rotation, translation) = m.to_scale_rotation_translation();
                let transition_rotation = rotation * traveler_transform.rotation().inverse();
                (translation + offset, rotation, transition_rotation)
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
            cooldown.last_travel = Some(elapsed);
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
