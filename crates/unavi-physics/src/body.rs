use avian3d::{
    physics_transform::PhysicsTransformSystems,
    prelude::{
        Collider,
        PhysicsSchedule,
        PhysicsSystems,
        Position,
        RigidBody,
        Rotation,
    },
};
use bevy::prelude::*;

/// Holds a collider taken off an entity whose global transform is degenerate,
/// so it can go back on unchanged once the transform recovers.
#[derive(Component)]
pub struct DisabledCollider(pub Collider);

#[derive(Component)]
pub struct DisabledRigidBody(pub RigidBody);

/// Adds `collider` with its physics pose seeded from `seed`, or parks it as a
/// [`DisabledCollider`] when `seed` is degenerate.
///
/// Avian's insert hook scales the shape by the entity's `GlobalTransform` and
/// reads `Position`/`Rotation`, whose defaults are `PLACEHOLDER` (`MAX`). A
/// collider added before transform propagation must carry its own pose; `seed`
/// is the caller's composition of the transform chain.
pub fn insert_collider(
    commands: &mut Commands,
    entity: Entity,
    collider: Collider,
    seed: &Transform,
) {
    if transform_is_valid(seed) {
        commands.entity(entity).insert((
            collider,
            Position(seed.translation),
            Rotation(seed.rotation),
        ));
    } else {
        commands.entity(entity).insert(DisabledCollider(collider));
    }
}

pub(crate) struct DegenerateBodyPlugin;

impl Plugin for DegenerateBodyPlugin {
    fn build(&self, app: &mut App) {
        // Avian propagates transforms itself at the head of `Prepare` and then
        // reads the result in `TransformToPosition`; between the two is the
        // only point where the global transform is current and unread.
        app.add_systems(
            PhysicsSchedule,
            park_degenerate_bodies
                .in_set(PhysicsSystems::Prepare)
                .after(PhysicsTransformSystems::Propagate)
                .before(PhysicsTransformSystems::TransformToPosition)
                // Matching avian's own transform systems, which opt out rather
                // than order against every exclusive system in the schedule.
                .ambiguous_with_all(),
        );
    }
}

fn is_valid_seed(scale: Vec3, rotation: Quat, translation: Vec3) -> bool {
    translation.is_finite()
        && rotation.is_finite()
        && scale.is_finite()
        && scale.x != 0.0
        && scale.y != 0.0
        && scale.z != 0.0
}

fn transform_is_valid(t: &Transform) -> bool {
    is_valid_seed(t.scale, t.rotation, t.translation)
}

fn global_transform_is_valid(t: &GlobalTransform) -> bool {
    let (s, r, tr) = t.to_scale_rotation_translation();
    is_valid_seed(s, r, tr)
}

/// Removes the collider and rigid body from any entity whose global transform
/// went degenerate, and restores them when it recovers.
///
/// A zero scale collapses the scaled shape, and a non-finite transform reaches
/// the solver as a non-finite `Position` that spreads across the body's whole
/// island.
fn park_degenerate_bodies(
    mut commands: Commands,
    active_col: Query<(Entity, &Collider, &GlobalTransform), Without<DisabledCollider>>,
    parked_col: Query<(Entity, &DisabledCollider, &GlobalTransform)>,
    active_rb: Query<(Entity, &RigidBody, &GlobalTransform), Without<DisabledRigidBody>>,
    parked_rb: Query<(Entity, &DisabledRigidBody, &GlobalTransform)>,
) {
    for (entity, collider, transform) in &active_col {
        if !global_transform_is_valid(transform) {
            let saved = collider.clone();
            commands
                .entity(entity)
                .remove::<Collider>()
                .insert(DisabledCollider(saved));
        }
    }
    for (entity, disabled, transform) in &parked_col {
        if global_transform_is_valid(transform) {
            let restored = disabled.0.clone();
            let seed = transform.compute_transform();
            commands.entity(entity).remove::<DisabledCollider>();
            insert_collider(&mut commands, entity, restored, &seed);
        }
    }
    for (entity, rb, transform) in &active_rb {
        if !global_transform_is_valid(transform) {
            let saved = *rb;
            commands
                .entity(entity)
                .remove::<(RigidBody, Position, Rotation)>()
                .insert(DisabledRigidBody(saved));
        }
    }
    for (entity, disabled, transform) in &parked_rb {
        if global_transform_is_valid(transform) {
            let restored = disabled.0;
            let global = transform.compute_transform();
            commands
                .entity(entity)
                .remove::<DisabledRigidBody>()
                .insert((
                    restored,
                    Position(global.translation),
                    Rotation(global.rotation),
                ));
        }
    }
}
