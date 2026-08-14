use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    image::{
        ImageAddressMode,
        ImageLoaderSettings,
        ImageSampler,
        ImageSamplerDescriptor,
    },
    math::Affine2,
    prelude::*,
};
use bevy_hsd::loaded::HsdLoaded;
use iroh_docs::NamespaceId;
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};
use unavi_space::{
    Space,
    anchor::{
        ActiveSpace,
        SPACE_CELL_SIZE,
    },
};

use crate::scene::{
    SceneState,
    respawn::Respawn,
};

/// Delay after a space loads, to allow scripts to execute and spawn the scene.
const SPACE_LOAD_DELAY: Duration = Duration::from_secs(1);
/// Exit limbo anyway if a space never reports loaded, so a missing or broken
/// asset can't strand the local agent on the limbo floor indefinitely.
const SPACE_LOAD_TIMEOUT: Duration = Duration::from_secs(30);

const LIMBO_OFFSET: Vec3 = Vec3::new(SPACE_CELL_SIZE, 0.0, SPACE_CELL_SIZE);
const PLANE_SIZE: f32 = 2048.0;
const TEXTURE_SIZE: f32 = 16.0;

#[derive(Component)]
pub struct Limbo;

pub fn spawn_limbo(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let dev_white = asset_server
        .load_builder()
        .with_settings(|s: &mut ImageLoaderSettings| {
            let mut desc = ImageSamplerDescriptor::nearest();
            desc.address_mode_u = ImageAddressMode::Repeat;
            desc.address_mode_v = ImageAddressMode::Repeat;
            desc.address_mode_w = ImageAddressMode::Repeat;
            s.sampler = ImageSampler::Descriptor(desc);
        })
        .load("image/dev-white.png");

    let material = StandardMaterial {
        base_color: tailwind::SKY_100.into(),
        base_color_texture: Some(dev_white),
        clearcoat: 0.4,
        clearcoat_perceptual_roughness: 0.4,
        emissive: tailwind::SKY_500.into(),
        emissive_exposure_weight: 0.4,
        metallic: 0.3,
        perceptual_roughness: 0.7,
        uv_transform: Affine2::from_scale(Vec2::splat(PLANE_SIZE / TEXTURE_SIZE)),
        ..Default::default()
    };

    let mesh = Plane3d::new(Vec3::Y, Vec2::splat(PLANE_SIZE));

    commands.spawn((
        Limbo,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Transform::from_translation(LIMBO_OFFSET),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
    ));
}

/// Pins the agent body to the limbo floor for as long as limbo lasts.
///
/// Held every tick rather than placed once on entry: the floor is a body like
/// any other, so a space becoming active shifts it out from under an agent
/// parked at a fixed point, and a load long enough to notice is long enough to
/// fall out of. Tracking the floor's own translation keeps the two together
/// through any shift.
pub fn hold_agent_in_limbo(
    agents: Query<&LocalAgentEntities, With<LocalAgent>>,
    floor: Query<&Transform, With<Limbo>>,
    mut bodies: Query<
        (
            &mut Transform,
            &mut Position,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        Without<Limbo>,
    >,
) {
    let (Ok(ents), Ok(floor)) = (agents.single(), floor.single()) else {
        return;
    };
    let Ok((mut tr, mut pos, mut vel, mut ang_vel)) = bodies.get_mut(ents.body) else {
        return;
    };

    tr.translation = floor.translation;
    pos.0 = floor.translation;
    *vel = LinearVelocity::default();
    *ang_vel = AngularVelocity::default();
}

pub fn despawn_limbo(limbo: Query<Entity, With<Limbo>>, mut commands: Commands) {
    for entity in limbo {
        commands.entity(entity).despawn();
    }
}

/// The space limbo is holding the local agent for.
#[derive(Resource, Default)]
pub struct LimboArrival {
    /// Namespace being travelled to. `None` takes whichever space loads first,
    /// which is how the startup entry into home arrives.
    pub target: Option<NamespaceId>,
    timeout:    Duration,
    ready:      Option<Arrival>,
}

struct Arrival {
    space: Entity,
    at:    Duration,
}

pub fn arm_limbo_arrival(time: Res<Time>, mut arrival: ResMut<LimboArrival>) {
    arrival.timeout = time.elapsed() + SPACE_LOAD_TIMEOUT;
    arrival.ready = None;
}

/// Drops back to limbo whenever the space underfoot is gone or still loading,
/// so the agent never stands in a space that has no scene to stand on.
pub fn fall_back_to_limbo(
    state: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    loaded: Query<(), (With<Space>, With<HsdLoaded>)>,
    mut next: ResMut<NextState<SceneState>>,
) {
    if !matches!(state.get(), SceneState::Space) {
        return;
    }
    if active.0.is_some_and(|space| loaded.contains(space)) {
        return;
    }
    info!("No loaded space, returning to limbo");
    next.set(SceneState::Limbo);
}

/// Leaves limbo once the arrival space reports loaded.
///
/// Reading [`HsdLoaded`] every tick rather than observing its insertion: a
/// space opened earlier through a portal is already loaded when travel to it
/// begins, so no insertion is coming.
pub fn drive_limbo_exit(
    state: Res<State<SceneState>>,
    time: Res<Time>,
    spaces: Query<(Entity, &Space, Has<HsdLoaded>)>,
    mut arrival: ResMut<LimboArrival>,
    mut commands: Commands,
) {
    if !matches!(state.get(), SceneState::Limbo) {
        return;
    }

    if let Some(ready) = &arrival.ready {
        // The settle delay is long enough for the space to be dropped again.
        if !spaces.get(ready.space).is_ok_and(|(_, _, loaded)| loaded) {
            arrival.ready = None;
            return;
        }
        if time.elapsed() >= ready.at {
            info!("Space loaded, exiting limbo");
            commands.trigger(EnterSpace {
                space: Some(ready.space),
            });
        }
        return;
    }

    let target = arrival.target;
    let loaded = spaces
        .iter()
        .filter(|(_, space, _)| target.is_none_or(|ns| ns == space.0))
        .find_map(|(entity, _, loaded)| loaded.then_some(entity));

    match loaded {
        Some(space) => {
            arrival.ready = Some(Arrival {
                space,
                at: time.elapsed() + SPACE_LOAD_DELAY,
            });
        }
        None if time.elapsed() >= arrival.timeout => {
            warn!("Space load timed out, exiting limbo anyway");
            let space = spaces
                .iter()
                .find(|(_, space, _)| target.is_some_and(|ns| ns == space.0))
                .map(|(entity, ..)| entity);
            commands.trigger(EnterSpace { space });
        }
        None => {}
    }
}

#[derive(Event)]
pub struct EnterSpace {
    /// Space to spawn into, overriding the active one so an arrival lands in
    /// what it travelled to rather than whichever space is nearest limbo.
    space: Option<Entity>,
}

pub fn enter_space(
    trigger: On<EnterSpace>,
    mut arrival: ResMut<LimboArrival>,
    mut next: ResMut<NextState<SceneState>>,
    mut commands: Commands,
) {
    arrival.target = None;
    arrival.ready = None;
    next.set(SceneState::Space);
    commands.trigger(Respawn {
        space: trigger.event().space,
    });
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use unavi_space::{
        anchor::ActiveSpace,
        travel::PendingTravel,
    };

    use super::*;
    use crate::scene::travel::drive_travel;

    #[derive(Resource, Default)]
    struct RespawnedInto(Option<Entity>);

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
            .init_state::<SceneState>()
            .add_observer(enter_space)
            .add_observer(record_respawn)
            .add_systems(OnEnter(SceneState::Limbo), arm_limbo_arrival)
            .add_systems(
                Update,
                (drive_travel, fall_back_to_limbo, drive_limbo_exit).chain(),
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
        app.world_mut().resource_mut::<PendingTravel>().0 = Some(target);
    }

    /// Enters a space so travel has somewhere to leave from.
    fn enter(app: &mut App, ns: NamespaceId) -> Entity {
        let space = app.world_mut().spawn((Space(ns), HsdLoaded)).id();
        app.world_mut().resource_mut::<ActiveSpace>().0 = Some(space);
        app.update();
        advance(app, SPACE_LOAD_DELAY);
        app.update();
        app.update();
        assert_eq!(state(app), SceneState::Space, "failed to enter start space");
        space
    }

    #[test]
    fn travels_to_an_already_loaded_space() {
        let mut app = setup();
        let start = enter(&mut app, namespace(1));

        // Opened earlier through a portal, so it is loaded before travel begins
        // and no `HsdLoaded` insertion is coming to notice.
        let target_ns = namespace(2);
        let target = app.world_mut().spawn((Space(target_ns), HsdLoaded)).id();

        travel_to(&mut app, target_ns);
        app.update();
        assert_eq!(state(&app), SceneState::Space);
        app.update();
        assert_eq!(state(&app), SceneState::Limbo);
        assert!(app.world().get_entity(start).is_err());

        advance(&mut app, SPACE_LOAD_DELAY);
        app.update();
        app.update();
        assert_eq!(state(&app), SceneState::Space);
        assert_eq!(respawned_into(&app), Some(target));
    }

    #[test]
    fn waits_for_the_travelled_to_space_not_another_loaded_one() {
        let mut app = setup();
        enter(&mut app, namespace(1));

        let bystander = app.world_mut().spawn((Space(namespace(3)), HsdLoaded)).id();
        let target_ns = namespace(2);
        let target = app.world_mut().spawn(Space(target_ns)).id();

        travel_to(&mut app, target_ns);
        app.update();
        app.update();
        assert_eq!(state(&app), SceneState::Limbo);

        advance(&mut app, SPACE_LOAD_DELAY * 2);
        app.update();
        assert_eq!(
            state(&app),
            SceneState::Limbo,
            "a loaded bystander space must not end the wait"
        );

        app.world_mut().entity_mut(target).insert(HsdLoaded);
        app.update();
        advance(&mut app, SPACE_LOAD_DELAY);
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
            .find_map(|(entity, space)| (space.0 == ns).then_some(entity))
            .expect("space read back fresh");

        app.world_mut().entity_mut(reloaded).insert(HsdLoaded);
        app.update();
        advance(&mut app, SPACE_LOAD_DELAY);
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
        let target = app.world_mut().spawn((Space(target_ns), HsdLoaded)).id();

        travel_to(&mut app, target_ns);
        app.update();
        app.update();
        assert_eq!(state(&app), SceneState::Limbo);

        app.world_mut().entity_mut(target).despawn();
        advance(&mut app, SPACE_LOAD_DELAY);
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
        advance(&mut app, SPACE_LOAD_DELAY);
        app.update();
        app.update();
        assert_eq!(state(&app), SceneState::Space);
    }

    #[test]
    fn leaves_limbo_when_the_space_never_loads() {
        let mut app = setup();
        enter(&mut app, namespace(1));

        travel_to(&mut app, namespace(2));
        app.update();
        app.update();
        assert_eq!(state(&app), SceneState::Limbo);

        advance(&mut app, SPACE_LOAD_TIMEOUT);
        app.update();
        app.update();
        assert_eq!(state(&app), SceneState::Space);
    }
}
