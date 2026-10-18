use std::time::Duration;

use bevy::{
    prelude::*,
    state::app::StatesPlugin,
};
use bevy_hsd::loaded::HsdLoaded;
use iroh_docs::NamespaceId;
use unavi_script::ScriptStatus;
use unavi_space::{
    grid::ActiveSpace,
    membership::Space,
    travel::PendingTravel,
};

use super::arrival::{
    GROUND_GRACE,
    LimboArrival,
    SPACE_LOAD_TIMEOUT,
    arm_limbo_arrival,
    drive_limbo_exit,
    enter_space,
    fall_back_to_limbo,
};
use crate::scene::{
    SceneState,
    respawn::Respawn,
    travel::drive_travel,
};

#[derive(Resource, Default)]
struct RespawnedInto(Option<Entity>);

/// Whether the stand-in probe finds ground.
#[derive(Resource, Default)]
struct GroundPresent(bool);

/// Stands in for `recenter_active_space`, which promotes whichever space
/// the agent was just put down in.
fn record_respawn(
    trigger: On<Respawn>,
    mut into: ResMut<RespawnedInto>,
    mut active: ResMut<ActiveSpace>,
) {
    into.0 = trigger.event().space;
    if let Some(space) = trigger.event().space {
        active.0 = Some(space);
    }
}

/// Stands in for `probe_ground`, whose ray cast needs the physics world.
fn stand_in_probe(ground: Res<GroundPresent>, mut arrival: ResMut<LimboArrival>) {
    if ground.0 {
        arrival.mark_grounded();
    }
}

fn namespace(seed: u8) -> NamespaceId {
    NamespaceId::from(&[seed; 32])
}

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_resource::<Time>()
        .init_resource::<LimboArrival>()
        .init_resource::<PendingTravel>()
        .init_resource::<ActiveSpace>()
        .init_resource::<RespawnedInto>()
        .init_resource::<GroundPresent>()
        .init_state::<SceneState>()
        .add_observer(enter_space)
        .add_observer(record_respawn)
        .add_systems(OnEnter(SceneState::Limbo), arm_limbo_arrival)
        .add_systems(
            Update,
            (
                drive_travel,
                fall_back_to_limbo,
                stand_in_probe,
                drive_limbo_exit,
            )
                .chain(),
        );
    app
}

fn advance(app: &mut App, by: Duration) {
    app.world_mut().resource_mut::<Time>().advance_by(by);
}

fn state(app: &App) -> SceneState {
    *app.world().resource::<State<SceneState>>().get()
}

fn respawned_into(app: &App) -> Option<Entity> {
    app.world().resource::<RespawnedInto>().0
}

fn travel_to(app: &mut App, target: NamespaceId) {
    app.world_mut().resource_mut::<PendingTravel>().0 = Some(target.into());
}

fn set_ground(app: &mut App, present: bool) {
    app.world_mut().resource_mut::<GroundPresent>().0 = present;
}

fn spawn_script(app: &mut App, space: Entity) -> ScriptStatus {
    let status = ScriptStatus::default();
    app.world_mut().spawn((status.clone(), ChildOf(space)));
    status
}

fn enter(app: &mut App, ns: NamespaceId) -> Entity {
    let space = app.world_mut().spawn((Space(ns.into()), HsdLoaded)).id();
    app.world_mut().resource_mut::<ActiveSpace>().0 = Some(space);
    app.update();
    advance(app, GROUND_GRACE);
    app.update();
    app.update();
    assert_eq!(state(app), SceneState::Space, "failed to enter start space");
    space
}

/// Enters `SceneState::Space` with `space` just arrived at but not yet
/// promoted to active, the window `recenter_active_space` covers in
/// `PostUpdate`.
fn begin_entry(app: &mut App, space: Entity) {
    app.update();
    app.world_mut().resource_mut::<LimboArrival>().entering = Some(space);
    app.world_mut()
        .resource_mut::<NextState<SceneState>>()
        .set(SceneState::Space);
    app.update();
}

#[test]
fn entering_waits_for_recenter_before_falling_back() {
    let mut app = setup();
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();

    begin_entry(&mut app, space);

    app.update();
    assert_eq!(
        state(&app),
        SceneState::Space,
        "a pending promotion must not bounce back to limbo"
    );

    app.world_mut().resource_mut::<ActiveSpace>().0 = Some(space);
    app.update();
    assert_eq!(app.world().resource::<LimboArrival>().entering, None);
}

#[test]
fn a_dropped_entering_space_still_falls_back() {
    let mut app = setup();
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();

    begin_entry(&mut app, space);
    app.world_mut().entity_mut(space).despawn();
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);
}

#[test]
fn travels_to_an_already_loaded_space() {
    let mut app = setup();
    let start = enter(&mut app, namespace(1));

    let target_ns = namespace(2);
    let target = app
        .world_mut()
        .spawn((Space(target_ns.into()), HsdLoaded))
        .id();

    travel_to(&mut app, target_ns);
    app.update();
    assert_eq!(state(&app), SceneState::Space);
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);
    assert!(app.world().get_entity(start).is_err());

    advance(&mut app, GROUND_GRACE);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Space);
    assert_eq!(respawned_into(&app), Some(target));
}

#[test]
fn waits_for_the_travelled_to_space_not_another_loaded_one() {
    let mut app = setup();
    enter(&mut app, namespace(1));

    let bystander = app
        .world_mut()
        .spawn((Space(namespace(3).into()), HsdLoaded))
        .id();
    let target_ns = namespace(2);
    let target = app.world_mut().spawn(Space(target_ns.into())).id();

    travel_to(&mut app, target_ns);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);

    advance(&mut app, GROUND_GRACE * 2);
    app.update();
    assert_eq!(
        state(&app),
        SceneState::Limbo,
        "a loaded bystander space must not end the wait"
    );

    app.world_mut().entity_mut(target).insert(HsdLoaded);
    app.update();
    advance(&mut app, GROUND_GRACE);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Space);

    assert_eq!(respawned_into(&app), Some(target));
    assert_ne!(respawned_into(&app), Some(bystander));
}

#[test]
fn travelling_to_the_current_space_reloads_it() {
    let mut app = setup();
    let ns = namespace(1);
    let start = enter(&mut app, ns);

    travel_to(&mut app, ns);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);
    assert!(
        app.world().get_entity(start).is_err(),
        "the space must be dropped, not left in place"
    );

    let reloaded = app
        .world_mut()
        .query::<(Entity, &Space)>()
        .iter(app.world())
        .find_map(|(entity, space)| (space.0 == ns.into()).then_some(entity))
        .expect("space read back fresh");

    app.world_mut().entity_mut(reloaded).insert(HsdLoaded);
    app.update();
    advance(&mut app, GROUND_GRACE);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Space);
    assert_eq!(respawned_into(&app), Some(reloaded));
}

#[test]
fn a_space_dropped_mid_settle_does_not_end_the_wait() {
    let mut app = setup();
    enter(&mut app, namespace(1));

    let target_ns = namespace(2);
    let target = app
        .world_mut()
        .spawn((Space(target_ns.into()), HsdLoaded))
        .id();

    travel_to(&mut app, target_ns);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);

    app.world_mut().entity_mut(target).despawn();
    advance(&mut app, GROUND_GRACE);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);
}

#[test]
fn returns_to_limbo_when_the_space_underfoot_unloads() {
    let mut app = setup();
    let space = enter(&mut app, namespace(1));

    app.world_mut().entity_mut(space).despawn();
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);
}

#[test]
fn a_still_loading_space_is_not_stood_in() {
    let mut app = setup();
    let space = enter(&mut app, namespace(1));

    app.world_mut().entity_mut(space).remove::<HsdLoaded>();
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);

    app.world_mut().entity_mut(space).insert(HsdLoaded);
    app.update();
    advance(&mut app, GROUND_GRACE);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Space);
}

#[test]
fn an_unsettled_script_holds_the_agent_until_it_settles() {
    let mut app = setup();
    set_ground(&mut app, true);
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();
    let script = spawn_script(&mut app, space);

    for _ in 0..3 {
        advance(&mut app, GROUND_GRACE);
        app.update();
    }
    assert_eq!(
        state(&app),
        SceneState::Limbo,
        "ground and time must not stand in for a script that has not settled"
    );

    script.set_initialized();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(
        state(&app),
        SceneState::Space,
        "found ground ends the wait without the grace period"
    );
}

#[test]
fn only_scripts_under_the_arrival_space_are_waited_on() {
    let mut app = setup();
    set_ground(&mut app, true);
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();
    let other = app
        .world_mut()
        .spawn((Space(namespace(2).into()), HsdLoaded))
        .id();
    spawn_script(&mut app, other);
    app.world_mut().resource_mut::<LimboArrival>().target = Some(namespace(1).into());

    for _ in 0..3 {
        app.update();
    }
    assert_eq!(state(&app), SceneState::Space);
    assert_eq!(respawned_into(&app), Some(space));
}

#[test]
fn a_trapped_script_counts_as_settled() {
    let mut app = setup();
    set_ground(&mut app, true);
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();
    let script = spawn_script(&mut app, space);
    assert!(script.trap());

    for _ in 0..3 {
        app.update();
    }
    assert_eq!(state(&app), SceneState::Space);
}

#[test]
fn a_space_without_ground_is_entered_after_the_grace_period() {
    let mut app = setup();
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();
    spawn_script(&mut app, space).set_initialized();

    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);

    advance(
        &mut app,
        GROUND_GRACE.saturating_sub(Duration::from_millis(1)),
    );
    app.update();
    app.update();
    assert_eq!(
        state(&app),
        SceneState::Limbo,
        "the grace period must run its full length"
    );

    advance(&mut app, Duration::from_millis(1));
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Space);
    assert_eq!(respawned_into(&app), Some(space));
}

#[test]
fn keeps_waiting_with_its_target_when_the_space_never_loads() {
    let mut app = setup();
    enter(&mut app, namespace(1));

    let target_ns = namespace(2);
    travel_to(&mut app, target_ns);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);

    advance(&mut app, SPACE_LOAD_TIMEOUT);
    app.update();
    app.update();
    assert_eq!(
        state(&app),
        SceneState::Limbo,
        "an unloaded space must never be entered"
    );
    assert!(
        app.world().resource::<LimboArrival>().target == Some(target_ns.into()),
        "the target must survive the timeout"
    );
}
