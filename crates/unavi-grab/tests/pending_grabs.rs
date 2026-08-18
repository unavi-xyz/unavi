//! A squeeze onto something not yet grabbable waits, so a script can answer a
//! grab by making the thing grabbable — either the thing the ray hit, or one
//! that arrives under the pointer afterwards.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_input::pointer::{
    PointerHit,
    PointerKind,
    PointerPressed,
    PointerReleased,
};

/// Matches `unavi_grab`'s own safety timeout.
const TIMEOUT: Duration = Duration::from_secs(5);
/// Matches `unavi_grab`'s own promotion window.
const PROMOTION: Duration = Duration::from_millis(500);
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
    // The pointer layer is what the test drives by hand, so what it would
    // have registered is registered here instead.
    .add_message::<PointerPressed>()
    .add_message::<PointerReleased>()
    // Bodies here are placed to be aimed at, not dropped, and a grab is
    // recognized by the `GravityScale` the grab itself adds — which a falling
    // body would need one of its own.
    .insert_resource(Gravity(Vec3::ZERO))
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

/// Aimed down -Z from the origin, as the desktop player's head is.
const REACH: f32 = 10.0;

const fn aim() -> Ray3d {
    Ray3d::new(Vec3::ZERO, Dir3::NEG_Z)
}

/// Stands in for the hit-test, which the pointer layer has already done by the
/// time a press is reported.
fn hit_on(app: &App, entity: Option<Entity>) -> Option<PointerHit> {
    let entity = entity?;
    let position = app
        .world()
        .entity(entity)
        .get::<Transform>()
        .map_or(Vec3::ZERO, |transform| transform.translation);
    Some(PointerHit {
        entity,
        position,
        normal: Vec3::Z,
        distance: position.length(),
    })
}

fn squeeze_down(app: &mut App, entity: Option<Entity>, pointer: Entity) {
    let hit = hit_on(app, entity);
    app.world_mut().write_message(PointerPressed {
        kind: PointerKind::Screen,
        pointer,
        ray: aim(),
        reach: REACH,
        hit,
    });
}

fn squeeze_up(app: &mut App, entity: Option<Entity>, pointer: Entity) {
    let hit = hit_on(app, entity);
    app.world_mut().write_message(PointerReleased {
        kind: PointerKind::Screen,
        pointer,
        ray: aim(),
        reach: REACH,
        hit,
    });
}

const fn steps_over(duration: Duration) -> usize {
    (duration.as_millis() / STEP.as_millis()) as usize + 3
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

    squeeze_down(&mut app, Some(target), pointer);
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
fn a_hold_that_pauses_before_the_drag_is_still_a_grab() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);

    squeeze_down(&mut app, Some(target), pointer);
    step(&mut app, steps_over(PROMOTION) * 2);

    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Dynamic);
    step(&mut app, 2);

    assert!(
        is_grabbed(&app, target),
        "a squeeze held still for a while and then dragged was refused, which \
         puts the whole tap-versus-take decision on a timer"
    );
}

#[test]
fn a_body_added_after_the_timeout_is_too_late() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);

    squeeze_down(&mut app, Some(target), pointer);
    step(&mut app, steps_over(TIMEOUT));

    app.world_mut()
        .entity_mut(target)
        .insert(RigidBody::Dynamic);
    step(&mut app, 2);

    assert!(
        !is_grabbed(&app, target),
        "a pending grab outlived its safety timeout"
    );
}

#[test]
fn releasing_cancels_a_pending_grab() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);
    let target = spawn_target(&mut app);

    squeeze_down(&mut app, Some(target), pointer);
    step(&mut app, 1);

    squeeze_up(&mut app, Some(target), pointer);
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

    squeeze_down(&mut app, Some(target), pointer);
    step(&mut app, 1);

    assert!(is_grabbed(&app, target));
}

/// The near miss: the ray slipped past whatever the user was plainly aiming
/// at, so there is no entity to wait on. Holding the squeeze still catches the
/// body that arrives under the pointer.
#[test]
fn a_squeeze_that_hit_nothing_catches_a_body_that_arrives_under_the_pointer() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);

    squeeze_down(&mut app, None, pointer);
    step(&mut app, 1);

    let target = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.1),
            Transform::from_xyz(0.0, 0.0, -1.0),
        ))
        .id();
    step(&mut app, 4);

    assert!(
        app.world().entity(target).contains::<LinearVelocity>(),
        "the body was never simulated, so the test proves nothing"
    );
    assert!(
        is_grabbed(&app, target),
        "a squeeze that missed refused the body that then appeared under it"
    );
}

/// The body a script promotes sits where the pointer *was* when it answered,
/// and the pointer has kept moving — during a drag, which is exactly when
/// this path runs, it moves fast. Demanding the answer be dead under the
/// crosshair some frames later asks the script to have predicted the aim.
#[test]
fn a_body_that_appears_beside_the_pointer_is_still_caught() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);

    squeeze_down(&mut app, None, pointer);
    step(&mut app, 1);

    let target = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.05),
            Transform::from_xyz(0.15, 0.0, -1.0),
        ))
        .id();
    step(&mut app, 4);

    assert!(
        is_grabbed(&app, target),
        "the answer to the squeeze drifted off the crosshair and was dropped"
    );
}

#[test]
fn a_body_promoted_well_off_the_pointer_is_left_alone() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);

    squeeze_down(&mut app, None, pointer);
    step(&mut app, 1);

    let elsewhere = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.05),
            Transform::from_xyz(1.5, 0.0, -1.0),
        ))
        .id();
    step(&mut app, 4);

    assert!(
        !is_grabbed(&app, elsewhere),
        "a squeeze caught a body nowhere near what it was pointing at"
    );
}

#[test]
fn a_body_promoted_behind_the_pointer_is_left_alone() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);

    squeeze_down(&mut app, None, pointer);
    step(&mut app, 1);

    let behind = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.05),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .id();
    step(&mut app, 4);

    assert!(!is_grabbed(&app, behind), "a squeeze reached backwards");
}

/// The other half of that rule: a squeeze must not quietly steal whatever
/// grabbable thing the pointer happens to sweep across while it is held.
#[test]
fn a_pending_grab_does_not_steal_a_body_it_merely_swept_over() {
    let mut app = app();
    let pointer = spawn_pointer(&mut app);

    let bystander = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.1),
            Transform::from_xyz(4.0, 0.0, -1.0),
        ))
        .id();
    step(&mut app, steps_over(PROMOTION));

    squeeze_down(&mut app, None, pointer);
    *app.world_mut()
        .entity_mut(bystander)
        .get_mut::<Transform>()
        .expect("transform") = Transform::from_xyz(0.0, 0.0, -1.0);
    step(&mut app, 4);

    assert!(
        !is_grabbed(&app, bystander),
        "a held squeeze grabbed a body that was already grabbable and simply \
         passed under the pointer"
    );
}
