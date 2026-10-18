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
    MIN_LIMBO_STAY,
    SPACE_LOAD_TIMEOUT,
    arm_limbo_arrival,
    drive_limbo_exit,
    enter_space,
    fall_back_to_limbo,
};
use crate::{
    camera::veil::{
        COVER,
        Veil,
        reveal_veil,
    },
    scene::{
        SceneState,
        respawn::Respawn,
        travel::{
            Departing,
            drive_travel,
        },
    },
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
        arrival.mark_grounded(Vec3::ZERO);
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
        .init_resource::<Departing>()
        .insert_resource(Veil::revealed())
        .init_resource::<ActiveSpace>()
        .init_resource::<RespawnedInto>()
        .init_resource::<GroundPresent>()
        .init_state::<SceneState>()
        .add_observer(enter_space)
        .add_observer(record_respawn)
        .add_systems(OnEnter(SceneState::Limbo), (arm_limbo_arrival, reveal_veil))
        .add_systems(OnEnter(SceneState::Space), reveal_veil)
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

/// Lets the veil run its full cover, as the frames of a real run would.
fn cover_veil(app: &mut App) {
    app.world_mut().resource_mut::<Veil>().step(COVER);
}

/// Runs the frames in which a passed gate closes the veil, enters the space,
/// and lands in `SceneState::Space`.
fn pass_gate(app: &mut App) {
    advance(app, MIN_LIMBO_STAY);
    app.update();
    cover_veil(app);
    app.update();
    app.update();
}

/// Runs the frames in which a travel request closes the veil, drops the
/// space, and lands in limbo.
fn depart(app: &mut App) {
    app.update();
    cover_veil(app);
    app.update();
    app.update();
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
    pass_gate(app);
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
    depart(&mut app);
    assert_eq!(state(&app), SceneState::Limbo);
    assert!(app.world().get_entity(start).is_err());

    advance(&mut app, GROUND_GRACE);
    pass_gate(&mut app);
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
    depart(&mut app);
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
    pass_gate(&mut app);
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
    depart(&mut app);
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
    pass_gate(&mut app);
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
    depart(&mut app);
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
    pass_gate(&mut app);
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
    pass_gate(&mut app);
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

    pass_gate(&mut app);
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

    pass_gate(&mut app);
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
    pass_gate(&mut app);
    assert_eq!(state(&app), SceneState::Space);
    assert_eq!(respawned_into(&app), Some(space));
}

#[test]
fn keeps_waiting_with_its_target_when_the_space_never_loads() {
    let mut app = setup();
    enter(&mut app, namespace(1));

    let target_ns = namespace(2);
    travel_to(&mut app, target_ns);
    depart(&mut app);
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

#[test]
fn a_passed_gate_waits_for_the_veil_before_entering() {
    let mut app = setup();
    set_ground(&mut app, true);
    app.world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded));

    app.update();
    app.update();
    assert_eq!(
        state(&app),
        SceneState::Limbo,
        "the agent must not be let in while the world is still visible"
    );

    pass_gate(&mut app);
    assert_eq!(state(&app), SceneState::Space);
}

#[test]
fn a_space_dropped_mid_cover_lifts_the_veil_again() {
    let mut app = setup();
    set_ground(&mut app, true);
    let space = app
        .world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded))
        .id();

    app.update();
    app.update();
    app.world_mut().resource_mut::<Veil>().step(COVER * 0.5);

    app.world_mut().entity_mut(space).despawn();
    app.update();
    cover_veil(&mut app);
    app.update();
    app.update();
    assert_eq!(state(&app), SceneState::Limbo);
    assert!(
        !app.world().resource::<Veil>().is_covered(),
        "an abandoned arrival must not leave the veil closed"
    );
}

#[test]
fn travel_keeps_the_space_until_the_veil_is_covered() {
    let mut app = setup();
    let start = enter(&mut app, namespace(1));

    travel_to(&mut app, namespace(2));
    app.update();
    assert!(app.world().get_entity(start).is_ok());
    assert!(app.world().resource::<PendingTravel>().0.is_none());

    app.world_mut().resource_mut::<Veil>().step(COVER * 0.5);
    app.update();
    app.update();
    assert!(
        app.world().get_entity(start).is_ok(),
        "the space went before the veil hid it"
    );
    assert_eq!(state(&app), SceneState::Space);

    cover_veil(&mut app);
    app.update();
    assert!(app.world().get_entity(start).is_err());
}

#[test]
fn travel_from_limbo_retargets_the_arrival() {
    let mut app = setup();
    let first = namespace(1);
    let second = namespace(2);
    app.world_mut().resource_mut::<LimboArrival>().target = Some(first.into());
    app.world_mut().spawn(Space(first.into()));

    travel_to(&mut app, second);
    app.update();

    assert!(
        app.world().resource::<LimboArrival>().target == Some(second.into()),
        "travel from limbo must retarget the arrival"
    );
    assert!(app.world().resource::<PendingTravel>().0.is_none());
    assert_eq!(spaces_of(&mut app, second), 1);

    travel_to(&mut app, second);
    app.update();
    assert_eq!(
        spaces_of(&mut app, second),
        1,
        "an existing space is reused"
    );
    assert_eq!(state(&app), SceneState::Limbo);
}

fn spaces_of(app: &mut App, ns: NamespaceId) -> usize {
    app.world_mut()
        .query::<&Space>()
        .iter(app.world())
        .filter(|space| space.0 == ns.into())
        .count()
}

#[test]
fn a_space_ready_at_once_still_waits_out_the_minimum_stay() {
    let mut app = setup();
    set_ground(&mut app, true);
    app.world_mut()
        .spawn((Space(namespace(1).into()), HsdLoaded));

    app.update();
    advance(
        &mut app,
        MIN_LIMBO_STAY.saturating_sub(Duration::from_millis(1)),
    );
    app.update();
    cover_veil(&mut app);
    app.update();
    app.update();
    assert_eq!(
        state(&app),
        SceneState::Limbo,
        "a ready space must not cut limbo short"
    );

    pass_gate(&mut app);
    assert_eq!(state(&app), SceneState::Space);
}

#[test]
fn limbo_puts_the_body_down_once_and_then_leaves_it_alone() {
    use avian3d::prelude::{
        AngularVelocity,
        LinearVelocity,
        Position,
        RigidBodyDisabled,
    };
    use bevy::ecs::system::RunSystemOnce;
    use unavi_agent::{
        LocalAgent,
        LocalAgentEntities,
        config::AgentConfig,
    };
    use unavi_space::grid::Spaceless;

    let mut app = App::new();
    let config = AgentConfig::default();
    let floor = Vec3::new(5000.0, 0.0, 5000.0);
    let spawn = floor + super::LIMBO_SPAWN_OFFSET + Vec3::Y * config.float_height();

    app.world_mut()
        .spawn((super::Limbo, Transform::from_translation(floor)));
    let body = app
        .world_mut()
        .spawn((
            Transform::default(),
            Position(Vec3::ZERO),
            LinearVelocity(Vec3::ONE),
            AngularVelocity(Vec3::ONE),
        ))
        .id();
    let tracked_head = app.world_mut().spawn_empty().id();
    let agent = app
        .world_mut()
        .spawn((
            LocalAgent,
            config,
            LocalAgentEntities { body, tracked_head },
        ))
        .id();

    app.world_mut()
        .run_system_once(super::place_agent_in_limbo)
        .expect("place runs");

    assert_eq!(
        app.world().entity(body).get::<Position>().map(|pos| pos.0),
        Some(spawn),
        "the agent must start on the limbo floor"
    );
    assert_eq!(
        app.world()
            .entity(body)
            .get::<LinearVelocity>()
            .map(|vel| vel.0),
        Some(Vec3::ZERO)
    );
    assert!(
        app.world().entity(agent).contains::<super::PlacedInLimbo>(),
        "placement must happen once per limbo stay"
    );
    assert!(
        !app.world().entity(body).contains::<RigidBodyDisabled>(),
        "limbo must leave the body simulated so the player can walk"
    );

    // The player walks away from the spawn point. A later tick must not drag
    // them back.
    let walked = spawn + Vec3::new(3.0, 0.0, 0.0);
    app.world_mut()
        .entity_mut(body)
        .insert((Transform::from_translation(walked), Position(walked)));
    app.world_mut()
        .run_system_once(super::place_agent_in_limbo)
        .expect("place runs again");

    assert_eq!(
        app.world().entity(body).get::<Position>().map(|pos| pos.0),
        Some(walked),
        "limbo must not keep pulling the player back"
    );

    app.world_mut().entity_mut(agent).insert(Spaceless);
    app.world_mut()
        .run_system_once(super::leave_limbo)
        .expect("leave runs");

    assert!(!app.world().entity(agent).contains::<Spaceless>());
    assert!(
        !app.world().entity(agent).contains::<super::PlacedInLimbo>(),
        "the next limbo stay must place the agent again"
    );
}
