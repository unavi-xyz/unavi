//! Pins the avian behaviour the collision-layer design rests on: whether a
//! collider with no rigid body blocks a dynamic body, and whether it is still
//! found by raycasts.

use avian3d::prelude::*;
use bevy::prelude::*;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        bevy::scene::ScenePlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        unavi_physics::PhysicsPlugin,
    ))
    .init_asset::<Mesh>()
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f32(1.0 / 60.0),
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

/// A collider with no rigid body is **not** static geometry — it takes part
/// in no collision response at all. UI bodies therefore need no sensor flag
/// to avoid shoving the player; a bare collider is already query-only.
#[test]
fn a_collider_without_a_rigid_body_blocks_nothing() {
    let mut app = app();

    app.world_mut().spawn((
        Collider::cuboid(10.0, 1.0, 10.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    let falling = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 4.0, 0.0),
        ))
        .id();

    step(&mut app, 240);

    let y = app
        .world()
        .entity(falling)
        .get::<Transform>()
        .expect("transform")
        .translation
        .y;

    assert!(
        y < 0.0,
        "a bodiless collider stopped a falling body, so it is static after \
         all and UI colliders would need sensors (y = {y})"
    );
}

#[test]
fn a_collider_without_a_rigid_body_is_still_raycast_hittable() {
    let mut app = app();

    let target = app
        .world_mut()
        .spawn((Collider::sphere(0.5), Transform::from_xyz(0.0, 0.0, -3.0)))
        .id();

    let caster = app
        .world_mut()
        .spawn((
            RayCaster::new(Vec3::ZERO, Dir3::NEG_Z).with_max_distance(10.0),
            Transform::default(),
        ))
        .id();

    step(&mut app, 8);

    let hits = app
        .world()
        .entity(caster)
        .get::<RayHits>()
        .expect("ray hits");

    assert_eq!(
        hits.iter().next().map(|hit| hit.entity),
        Some(target),
        "a bodiless collider was not found by a raycast"
    );
}
