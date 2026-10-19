//! The app's scene flow: limbo while no space is loaded, travel between
//! spaces, and the startup join into home or `--join`.

use bevy::{
    light::{
        CascadeShadowConfigBuilder,
        light_consts::lux,
    },
    prelude::*,
};
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{
        BuildRouter,
        IrohRouter,
    },
};
use bevy_vrm::mtoon::MtoonSun;
use unavi_agent::LocalAgent;
use unavi_space::grid::Spaceless;

use crate::camera::veil::reveal_veil;

pub mod home;
mod limbo;
mod respawn;
mod startup;
mod system_scripts;
mod travel;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .init_resource::<limbo::arrival::LimboArrival>()
            .init_resource::<travel::Departing>()
            .insert_resource(startup::StartupHold)
            .add_observer(limbo::arrival::enter_space)
            .add_observer(respawn::respawn)
            .add_observer(home::enter_home)
            .add_systems(
                OnEnter(SceneState::Limbo),
                (
                    limbo::arrival::arm_limbo_arrival,
                    limbo::spawn_limbo,
                    spawn_local_agent,
                    limbo::enter_limbo,
                ),
            )
            // The first limbo entry is the launch itself, whose veil the
            // startup hold lifts instead.
            .add_systems(
                OnEnter(SceneState::Limbo),
                reveal_veil.run_if(not(resource_exists::<startup::StartupHold>)),
            )
            .add_systems(OnEnter(SceneState::Space), reveal_veil)
            .add_systems(
                OnExit(SceneState::Limbo),
                (limbo::despawn_limbo, build_iroh_router, limbo::leave_limbo),
            )
            .add_systems(
                Startup,
                (
                    spawn_sun,
                    home::join_startup_space,
                    system_scripts::spawn_system_scripts,
                ),
            )
            .add_systems(
                Update,
                (
                    home::resolve_home,
                    limbo::arrival::fall_back_to_limbo,
                    limbo::ground::probe_ground.run_if(in_state(SceneState::Limbo)),
                    limbo::arrival::drive_limbo_exit,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                startup::release_startup.run_if(resource_exists::<startup::StartupHold>),
            )
            .add_systems(
                FixedUpdate,
                (
                    travel::drive_travel,
                    limbo::place_agent_in_limbo.run_if(in_state(SceneState::Limbo)),
                    respawn::teleport_from_void,
                ),
            );
    }
}

#[derive(Default, Debug, States, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneState {
    /// Empty "limbo" scene, if not in any spaces.
    /// Acts as a loading screen or fallback on error.
    #[default]
    Limbo,
    /// Actively within a space.
    Space,
}

/// Builds the iroh router on first space entry, once gossip, space, and data
/// store protocols have registered their handlers. The router can only be
/// spawned once, so [`IrohRouter`] gates against rebuilding on later re-entry.
fn build_iroh_router(
    endpoints: Query<Entity, (With<IrohEndpoint>, Without<IrohRouter>)>,
    mut commands: Commands,
) {
    for entity in &endpoints {
        commands.entity(entity).trigger(BuildRouter);
    }
}

fn spawn_local_agent(local_agent: Query<(), With<LocalAgent>>, mut commands: Commands) {
    if !local_agent.is_empty() {
        return;
    }
    commands.spawn((LocalAgent, Spaceless));
}

fn spawn_sun(mut commands: Commands) {
    // WebGL2 caps a directional light at one cascade, so range is the only
    // lever on texel size there.
    let (num_cascades, maximum_distance): (usize, f32) =
        cfg_select! {
            all(target_family = "wasm", not(feature = "webgpu")) => (1, 45.0),
            _ => (4, 100.0),
        };

    commands.spawn((
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 8.0,
            maximum_distance,
            minimum_distance: 0.1,
            num_cascades,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::DIRECT_SUNLIGHT,
            shadow_maps_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-0.9, 10.0, 3.8).looking_at(Vec3::ZERO, Vec3::Y),
        MtoonSun,
    ));
}
