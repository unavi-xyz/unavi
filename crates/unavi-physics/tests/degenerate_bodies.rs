//! A scene, a script, or an avatar rig may drive a transform to zero scale or
//! to NaN. Physics has to survive it and recover when the transform does.

use std::time::Duration;

use avian3d::prelude::{
    Collider,
    Position,
    RigidBody,
    Rotation,
};
use bevy::prelude::*;
use unavi_physics::{
    PhysicsPlugin,
    body::{
        DisabledCollider,
        DisabledRigidBody,
        insert_collider,
    },
};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        bevy::scene::ScenePlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        PhysicsPlugin,
    ))
    .init_asset::<Mesh>()
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(1.0 / 60.0),
    ));
    app.finish();
    app.cleanup();
    app
}

fn step(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

fn set_transform(app: &mut App, entity: Entity, transform: Transform) {
    app.world_mut().entity_mut(entity).insert(transform);
}

fn has<C: Component>(app: &App, entity: Entity) -> bool {
    app.world().entity(entity).get::<C>().is_some()
}

#[test]
fn a_zero_scale_collider_is_parked_and_restored() {
    let mut app = app();
    let entity = app
        .world_mut()
        .spawn((Collider::cuboid(1.0, 1.0, 1.0), Transform::default()))
        .id();
    step(&mut app, 2);
    assert!(
        has::<Collider>(&app, entity),
        "collider never became active"
    );

    set_transform(&mut app, entity, Transform::from_scale(Vec3::ZERO));
    step(&mut app, 2);
    assert!(
        !has::<Collider>(&app, entity),
        "collider stayed active at zero scale"
    );
    assert!(
        has::<DisabledCollider>(&app, entity),
        "parked collider was dropped rather than stashed"
    );

    set_transform(&mut app, entity, Transform::from_scale(Vec3::ONE));
    step(&mut app, 2);
    assert!(
        has::<Collider>(&app, entity),
        "collider was not restored once the scale recovered"
    );
    assert!(!has::<DisabledCollider>(&app, entity));
}

/// The NaN a VRM rig can produce from degenerate bone geometry.
#[test]
fn a_nan_transform_parks_the_rigid_body_and_restores_it() {
    let mut app = app();
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::default(),
        ))
        .id();
    step(&mut app, 2);

    set_transform(&mut app, entity, Transform::from_xyz(f32::NAN, 0.0, 0.0));
    step(&mut app, 2);
    assert!(
        !has::<RigidBody>(&app, entity),
        "rigid body stayed active with a NaN transform"
    );
    assert!(has::<DisabledRigidBody>(&app, entity));
    assert!(
        !has::<Position>(&app, entity),
        "a NaN Position was left for the solver to read"
    );

    set_transform(&mut app, entity, Transform::IDENTITY);
    step(&mut app, 2);
    assert!(
        has::<RigidBody>(&app, entity),
        "rigid body was not restored once the transform recovered"
    );
    assert!(!has::<DisabledRigidBody>(&app, entity));
}

/// A child inherits its parent's degenerate scale through propagation, so the
/// guard has to read the global transform, not the local one.
#[test]
fn a_child_of_a_zero_scale_parent_is_parked() {
    let mut app = app();
    let parent = app
        .world_mut()
        .spawn(Transform::from_scale(Vec3::ZERO))
        .id();
    let child = app
        .world_mut()
        .spawn((
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 1.0, 0.0),
            ChildOf(parent),
        ))
        .id();
    step(&mut app, 2);

    assert!(
        has::<DisabledCollider>(&app, child),
        "a child under a collapsed parent kept an active collider"
    );

    set_transform(&mut app, parent, Transform::IDENTITY);
    step(&mut app, 2);
    assert!(
        has::<Collider>(&app, child),
        "the child's collider was not restored with its parent"
    );
}

#[test]
fn insert_collider_seeds_the_physics_pose_from_the_seed_transform() {
    let mut app = app();
    let entity = app.world_mut().spawn(Transform::default()).id();
    let seed = Transform::from_xyz(5.0, 2.0, -3.0);

    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        insert_collider(&mut commands, entity, Collider::sphere(0.5), &seed);
    });
    set_transform(&mut app, entity, seed);
    step(&mut app, 2);

    let position = app
        .world()
        .entity(entity)
        .get::<Position>()
        .expect("collider has no Position");
    assert!(
        (position.0 - seed.translation).length() < 1.0e-4,
        "Position did not come from the seed transform, got {:?}",
        position.0
    );
    assert!(
        app.world().entity(entity).get::<Rotation>().is_some(),
        "collider has no Rotation"
    );
}

/// Avian's placeholder pose is `MAX`; a degenerate seed must never be written
/// into physics at all, not written and then repaired.
#[test]
fn insert_collider_parks_a_degenerate_seed_without_touching_physics() {
    let mut app = app();
    let entity = app
        .world_mut()
        .spawn(Transform::from_scale(Vec3::ZERO))
        .id();
    let seed = Transform::from_scale(Vec3::ZERO);

    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        insert_collider(&mut commands, entity, Collider::sphere(0.5), &seed);
    });
    step(&mut app, 2);

    assert!(
        has::<DisabledCollider>(&app, entity),
        "a degenerate seed produced a live collider"
    );
    assert!(!has::<Collider>(&app, entity));
}

#[test]
fn ordinary_bodies_are_untouched() {
    let mut app = app();
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::default(),
    ));
    let falling = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 6.0, 0.0),
        ))
        .id();

    step(&mut app, 240);

    assert!(!has::<DisabledCollider>(&app, falling));
    assert!(!has::<DisabledRigidBody>(&app, falling));
    let y = app
        .world()
        .entity(falling)
        .get::<Transform>()
        .expect("transform")
        .translation
        .y;
    assert!(
        (0.5..1.5).contains(&y),
        "a body that should have landed on the ground is at y = {y}"
    );
}
