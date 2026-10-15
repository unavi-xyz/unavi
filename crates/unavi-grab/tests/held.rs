//! What a grab does to the body it carries: releases on a vanished pointer,
//! restores whatever gravity preceded it, refuses a second grab, clamps a
//! teleported pointer's pull, and answers a reach change.

use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_grab::events::{
    Grabbed,
    Released,
};
use unavi_input::{
    action::{
        Action,
        ActionState,
    },
    pointer::{
        GripPressed,
        GripReleased,
        PointerAim,
        PointerHit,
        PointerKind,
    },
};

/// Matches `held`'s own clamp.
const MAX_GRAB_SPEED: f32 = 20.0;

const STEP: std::time::Duration = std::time::Duration::from_millis(50);

#[derive(Resource, Default)]
struct GrabLog(Vec<Entity>);

#[derive(Resource, Default)]
struct ReleaseLog(Vec<Entity>);

fn record_grab(trigger: On<Grabbed>, mut log: ResMut<GrabLog>) {
    log.0.push(trigger.pointer);
}

fn record_release(trigger: On<Released>, mut log: ResMut<ReleaseLog>) {
    log.0.push(trigger.pointer);
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        bevy::scene::ScenePlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        bevy::input::InputPlugin,
        unavi_physics::PhysicsPlugin,
        unavi_grab::GrabPlugin,
    ))
    .init_asset::<Mesh>()
    .add_message::<GripPressed>()
    .add_message::<GripReleased>()
    .init_resource::<ActionState>()
    .init_resource::<GrabLog>()
    .init_resource::<ReleaseLog>()
    .add_observer(record_grab)
    .add_observer(record_release)
    // A grab is recognized by the `GravityScale` the grab itself adds, which
    // would fight real gravity on a falling body.
    .insert_resource(Gravity(Vec3::ZERO))
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(STEP));
    app.finish();
    app.cleanup();
    app
}

fn spawn_target(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.1),
            Transform::from_xyz(0.0, 0.0, -1.0),
        ))
        .id()
}

fn spawn_pointer(app: &mut App) -> Entity {
    app.world_mut().spawn(Transform::default()).id()
}

const fn aim() -> Ray3d {
    Ray3d::new(Vec3::ZERO, Dir3::NEG_Z)
}
const REACH: f32 = 10.0;

fn hit_on(app: &App, entity: Entity) -> PointerAim {
    let position = app
        .world()
        .entity(entity)
        .get::<Transform>()
        .map_or(Vec3::ZERO, |transform| transform.translation);
    PointerAim {
        kind:    PointerKind::Screen,
        pointer: Entity::PLACEHOLDER,
        ray:     aim(),
        reach:   REACH,
        hit:     Some(PointerHit {
            entity,
            position,
            normal: Vec3::Z,
            distance: position.length(),
        }),
    }
}

fn squeeze_down(app: &mut App, entity: Entity, pointer: Entity) {
    let mut aim = hit_on(app, entity);
    aim.pointer = pointer;
    app.world_mut().write_message(GripPressed(aim));
}

fn squeeze_up(app: &mut App, entity: Entity, pointer: Entity) {
    let mut aim = hit_on(app, entity);
    aim.pointer = pointer;
    app.world_mut().write_message(GripReleased(aim));
}

fn squeeze_down_on_nothing(app: &mut App, pointer: Entity) {
    app.world_mut().write_message(GripPressed(PointerAim {
        kind: PointerKind::Screen,
        pointer,
        ray: aim(),
        reach: REACH,
        hit: None,
    }));
}

fn squeeze_up_on_nothing(app: &mut App, pointer: Entity) {
    app.world_mut().write_message(GripReleased(PointerAim {
        kind: PointerKind::Screen,
        pointer,
        ray: aim(),
        reach: REACH,
        hit: None,
    }));
}

fn step(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

fn is_grabbed(app: &App, entity: Entity) -> bool {
    app.world().entity(entity).contains::<GravityScale>()
}

/// Lets transforms sync once before anything grabs, matching how a grab
/// itself needs a settled `GlobalTransform` to compute its hold offset.
fn settle(app: &mut App) {
    step(app, 1);
}

#[test]
fn a_vanished_pointer_releases_and_restores_no_gravity() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    settle(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);
    assert!(is_grabbed(&app, target), "setup: grab did not take");

    app.world_mut().despawn(pointer);
    step(&mut app, 1);

    assert!(
        !is_grabbed(&app, target),
        "a grab outlived the pointer that was supposed to be carrying it"
    );
    assert!(
        !app.world().entity(target).contains::<GravityScale>(),
        "gravity was not restored to its pre-grab absence"
    );
    assert_eq!(
        app.world().resource::<ReleaseLog>().0,
        vec![pointer],
        "the vanished pointer's release was never told apart"
    );
}

#[test]
fn a_prior_gravity_scale_is_restored_on_release() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    app.world_mut().entity_mut(target).insert(GravityScale(0.5));
    settle(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);
    assert_eq!(
        app.world().entity(target).get::<GravityScale>().copied(),
        Some(GravityScale(0.0)),
        "setup: the grab did not zero gravity"
    );

    squeeze_up(&mut app, target, pointer);
    step(&mut app, 1);

    assert_eq!(
        app.world().entity(target).get::<GravityScale>().copied(),
        Some(GravityScale(0.5)),
        "release replaced the object's own gravity scale instead of giving it back"
    );
}

#[test]
fn a_second_grab_on_an_already_held_body_is_refused() {
    let mut app = app();
    let (first, second) = (spawn_pointer(&mut app), spawn_pointer(&mut app));
    let target = spawn_target(&mut app);
    settle(&mut app);

    squeeze_down(&mut app, target, first);
    step(&mut app, 1);
    squeeze_down(&mut app, target, second);
    step(&mut app, 1);

    assert_eq!(
        app.world().resource::<GrabLog>().0,
        vec![first],
        "a body already held answered a second squeeze from another pointer"
    );
}

#[test]
fn a_huge_pointer_jump_is_clamped_rather_than_flung() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    settle(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);
    assert!(is_grabbed(&app, target), "setup: grab did not take");

    // A teleport or portal crossing: the pointer jumps instead of moving
    // continuously.
    app.world_mut()
        .entity_mut(pointer)
        .get_mut::<Transform>()
        .expect("transform")
        .translation = Vec3::new(0.0, 0.0, 100_000.0);
    step(&mut app, 2);

    let speed = app
        .world()
        .entity(target)
        .get::<LinearVelocity>()
        .expect("velocity")
        .0
        .length();
    assert!(
        speed <= MAX_GRAB_SPEED + 1.0e-3,
        "a teleported pointer flung its object at {speed} m/s"
    );
}

#[test]
fn a_despawned_held_entity_fires_exactly_one_release() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    settle(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);
    assert!(is_grabbed(&app, target), "setup: grab did not take");

    // A scene unload or a script despawning the object outright, not a
    // release through the grip.
    app.world_mut().despawn(target);
    step(&mut app, 1);

    assert_eq!(
        app.world().resource::<ReleaseLog>().0,
        vec![pointer],
        "a despawned held entity did not fire exactly one release"
    );
}

#[test]
fn a_demotion_off_dynamic_releases_the_grab() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    settle(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);
    assert!(is_grabbed(&app, target), "setup: grab did not take");

    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Kinematic);
    step(&mut app, 1);

    assert!(
        !is_grabbed(&app, target),
        "a body demoted off `Dynamic` kept answering to its pointer"
    );
    assert_eq!(
        app.world().resource::<ReleaseLog>().0,
        vec![pointer],
        "demotion did not release the grab through the same path as a release"
    );
}

#[test]
fn removing_the_rigid_body_clears_grabbable() {
    let mut app = app();
    let target = spawn_target(&mut app);
    settle(&mut app);
    assert!(
        app.world()
            .entity(target)
            .contains::<unavi_input::crosshair::Grabbable>(),
        "setup: a dynamic body was never marked grabbable"
    );

    app.world_mut().entity_mut(target).remove::<RigidBody>();
    step(&mut app, 1);

    assert!(
        !app.world()
            .entity(target)
            .contains::<unavi_input::crosshair::Grabbable>(),
        "a body that stopped being a rigid body at all stayed grabbable"
    );
}

#[test]
fn nothing_fires_a_release_without_a_prior_grab() {
    let mut app = app();
    let (first, second) = (spawn_pointer(&mut app), spawn_pointer(&mut app));
    let target = spawn_target(&mut app);
    settle(&mut app);

    // The second pointer's squeeze is refused outright (the body already
    // answers to `first`); letting go of it must not look like releasing a
    // grab it never held.
    squeeze_down(&mut app, target, first);
    step(&mut app, 1);
    squeeze_down(&mut app, target, second);
    step(&mut app, 1);
    squeeze_up(&mut app, target, second);
    step(&mut app, 1);

    assert_eq!(
        app.world().resource::<ReleaseLog>().0,
        Vec::<Entity>::new(),
        "a squeeze that never took a grab still fired a release"
    );

    // A pending grab that the user let go of before anything became
    // grabbable must not fire a release either: it was never a `Grabbed`.
    let pointer = spawn_pointer(&mut app);
    let bystander = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, -1.0))
        .id();
    squeeze_down_on_nothing(&mut app, pointer);
    step(&mut app, 1);
    squeeze_up_on_nothing(&mut app, pointer);
    app.world_mut()
        .entity_mut(bystander)
        .insert((RigidBody::Dynamic, Collider::sphere(0.1)));
    step(&mut app, 2);

    assert_eq!(
        app.world().resource::<ReleaseLog>().0,
        Vec::<Entity>::new(),
        "a cancelled pending grab fired a release it never earned"
    );
    assert_eq!(
        app.world().resource::<GrabLog>().0,
        vec![first],
        "a cancelled pending grab was granted anyway"
    );
}

#[test]
fn reach_pushes_a_held_object_farther_from_its_pointer() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    settle(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);

    let settled = app
        .world()
        .entity(target)
        .get::<LinearVelocity>()
        .expect("velocity")
        .0;
    assert!(
        settled.length() < 1.0e-6,
        "the object was not at rest at its hold point before reaching"
    );

    app.world_mut()
        .resource_mut::<ActionState>()
        .accumulate_delta(Action::Reach, Vec2::new(0.0, 3.0));
    step(&mut app, 1);

    let pushed = app
        .world()
        .entity(target)
        .get::<LinearVelocity>()
        .expect("velocity")
        .0;
    assert!(
        pushed.z < -0.1,
        "reaching out did not push the held object away from its pointer \
         (velocity.z = {})",
        pushed.z
    );
}
