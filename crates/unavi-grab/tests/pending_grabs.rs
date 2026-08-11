//! A squeeze onto something not yet grabbable waits briefly, so a script can
//! answer a grab by making the thing grabbable.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_input::{
    SqueezeDown,
    SqueezeUp,
};

/// Matches `unavi_grab`'s own window.
const WINDOW: Duration = Duration::from_millis(500);
const STEP: Duration = Duration::from_millis(50);

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
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(STEP));
    app.finish();
    app.cleanup();
    app
}

/// A collider with no body, as a mote resting in an orbit.
fn spawn_target(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((Collider::sphere(0.1), Transform::from_xyz(0.0, 0.0, -1.0)))
        .id()
}

fn spawn_pointer(app: &mut App) -> Entity {
    app.world_mut().spawn(Transform::default()).id()
}

fn squeeze_down(app: &mut App, entity: Entity, pointer: Entity) {
    app.world_mut().trigger(SqueezeDown {
        entity: Some(entity),
        pointer,
    });
}

fn step(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

fn is_grabbed(app: &App, entity: Entity) -> bool {
    app.world().entity(entity).contains::<GravityScale>()
}

#[test]
fn a_body_added_after_the_squeeze_still_gets_grabbed() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);
    assert!(!is_grabbed(&app, target), "nothing grabbable yet");

    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Dynamic);
    step(&mut app, 2);

    assert!(
        is_grabbed(&app, target),
        "the grab did not latch on once the body appeared"
    );
}

#[test]
fn a_body_added_after_the_window_is_too_late() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(
        &mut app,
        (WINDOW.as_millis() / STEP.as_millis()) as usize + 3,
    );

    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Dynamic);
    step(&mut app, 2);

    assert!(
        !is_grabbed(&app, target),
        "a pending grab outlived its window"
    );
}

#[test]
fn releasing_cancels_a_pending_grab() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);

    app.world_mut().trigger(SqueezeUp {
        entity: Some(target),
        pointer,
    });
    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Dynamic);
    step(&mut app, 2);

    assert!(
        !is_grabbed(&app, target),
        "a grab started after the user had let go"
    );
}

#[test]
fn an_already_dynamic_body_is_grabbed_at_once() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);
    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Dynamic);
    step(&mut app, 1);

    squeeze_down(&mut app, target, pointer);
    step(&mut app, 1);

    assert!(is_grabbed(&app, target));
}
