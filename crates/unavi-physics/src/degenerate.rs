//! Parks a body or collider whose global transform has gone degenerate (NaN,
//! or scaled to collapse or overflow), and restores it unchanged once the
//! transform recovers.

use avian3d::{
    physics_transform::PhysicsTransformSystems,
    prelude::{
        Collider,
        PhysicsSystems,
        Position,
        RigidBody,
        Rotation,
    },
};
use bevy::{
    app::FixedPostUpdate,
    math::Affine3A,
    prelude::*,
};

use crate::{
    finite::{
        MAX_DETERMINANT,
        MIN_DETERMINANT,
    },
    shape::insert_collider,
};

/// A collider and/or rigid body pulled off an entity whose global transform
/// went degenerate, held here unchanged so it can go back on once the
/// transform recovers.
#[derive(Component, Default)]
pub struct Parked {
    pub collider: Option<Collider>,
    pub body:     Option<RigidBody>,
}

pub(crate) struct DegenerateBodyPlugin;

impl Plugin for DegenerateBodyPlugin {
    fn build(&self, app: &mut App) {
        // Avian propagates transforms in `Prepare` and reads them in
        // `TransformToPosition`; between the two is the only point where the
        // global transform is current and unread. Both sets live in the outer
        // schedule (`FixedPostUpdate` for `PhysicsPlugins::default()`), not
        // `PhysicsSchedule`, where the same labels are empty and order
        // nothing.
        app.add_systems(
            FixedPostUpdate,
            park_degenerate_bodies
                .in_set(PhysicsSystems::Prepare)
                .after(PhysicsTransformSystems::Propagate)
                .before(PhysicsTransformSystems::TransformToPosition),
        );
    }
}

/// An affine's linear part is valid if it is finite and its determinant
/// stays within [`MIN_DETERMINANT`]/[`MAX_DETERMINANT`]: zero scale on any
/// axis collapses the determinant to zero, and a collapsed or blown-up shape
/// overflows the solver's area/volume math the same way a NaN does.
fn affine_is_valid(affine: Affine3A) -> bool {
    if !affine.matrix3.is_finite() || !affine.translation.is_finite() {
        return false;
    }
    let det = affine.matrix3.determinant();
    det.is_finite() && det.abs() > MIN_DETERMINANT && det.abs() < MAX_DETERMINANT
}

pub(crate) fn transform_is_valid(t: &Transform) -> bool {
    affine_is_valid(t.compute_affine())
}

fn global_transform_is_valid(t: &GlobalTransform) -> bool {
    affine_is_valid(t.affine())
}

/// Removes the collider and/or rigid body from any entity whose global
/// transform went degenerate, and restores them when it recovers.
///
/// A zero or extreme scale collapses or overflows the scaled shape, and a
/// non-finite transform reaches the solver as a non-finite `Position` that
/// spreads across the body's whole island.
///
/// Gated on `Changed<GlobalTransform>`, since validity can only change when
/// the transform does: a parked body only recovers when its transform moves,
/// and an active one only goes bad the same way. The `Added<Collider>` /
/// `Added<RigidBody>` arms catch the one way that filter can be fooled: a
/// collider or rigid body inserted this frame onto a transform that was
/// already degenerate (and so did not change) before it is ever read by this
/// system. [`insert_collider`] already seeds a fresh collider straight into
/// [`Parked`] in that case; a bare `RigidBody` insert (as `bevy-hsd`'s
/// `rigid_body` attribute does) has no such self-check, so it relies on this
/// filter catching it on the next physics step.
///
/// Reads `Option<&Parked>` rather than excluding parked entities outright:
/// a collider and rigid body on one entity can park on independent frames
/// (a rigid body added on top of an already-parked collider, both under one
/// still-degenerate transform, must still be caught and folded into the same
/// [`Parked`] rather than skipped because the entity already carries one).
fn park_degenerate_bodies(
    mut commands: Commands,
    bodies: Query<
        (
            Entity,
            Option<&Collider>,
            Option<&RigidBody>,
            Option<&Parked>,
            &GlobalTransform,
        ),
        (
            Or<(With<Collider>, With<RigidBody>, With<Parked>)>,
            Or<(Changed<GlobalTransform>, Added<Collider>, Added<RigidBody>)>,
        ),
    >,
) {
    for (entity, collider, rb, parked, transform) in &bodies {
        if global_transform_is_valid(transform) {
            let Some(parked) = parked else { continue };
            let seed = transform.compute_transform();
            commands.entity(entity).remove::<Parked>();
            if let Some(collider) = parked.collider.clone() {
                insert_collider(&mut commands, entity, collider, &seed);
            }
            if let Some(rb) = parked.body {
                commands.entity(entity).insert((
                    rb,
                    Position(seed.translation),
                    Rotation(seed.rotation),
                ));
            }
        } else if collider.is_some() || rb.is_some() {
            let mut parked = parked.map_or_else(Parked::default, |p| Parked {
                collider: p.collider.clone(),
                body:     p.body,
            });
            let mut entity_commands = commands.entity(entity);
            if let Some(collider) = collider {
                parked.collider = Some(collider.clone());
                entity_commands.remove::<Collider>();
            }
            if let Some(rb) = rb {
                parked.body = Some(*rb);
                entity_commands.remove::<(RigidBody, Position, Rotation)>();
            }
            entity_commands.insert(parked);
        }
    }
}
