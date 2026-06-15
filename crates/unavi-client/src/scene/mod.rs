use bevy::{
    light::{
        CascadeShadowConfigBuilder,
        light_consts::lux,
    },
    prelude::*,
};
use bevy_vrm::mtoon::MtoonSun;
use unavi_agent::LocalAgent;

mod home;
mod limbo;
mod respawn;
mod system_scripts;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_observer(limbo::track_space_load)
            .add_observer(limbo::exit_limbo_on_space_loaded)
            .add_observer(respawn::respawn)
            .add_systems(
                OnEnter(SceneState::Limbo),
                (limbo::spawn_limbo, spawn_local_agent),
            )
            .add_systems(OnExit(SceneState::Limbo), limbo::despawn_limbo)
            .add_systems(
                Startup,
                (
                    spawn_sun,
                    home::join_home,
                    system_scripts::spawn_system_scripts,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    limbo::exit_limbo_on_load_timeout,
                    respawn::teleport_from_void,
                    system_scripts::populate_firewall_entities,
                ),
            );
    }
}

#[derive(Default, Debug, States, Clone, Copy, PartialEq, Eq, Hash)]
enum SceneState {
    /// Empty "limbo" scene, if not in any spaces.
    /// Acts as a loading screen or fallback on error.
    #[default]
    Limbo,
    /// Main scene state.
    /// Actively within a space.
    Space,
}

fn spawn_local_agent(local_agent: Query<(), With<LocalAgent>>, mut commands: Commands) {
    if !local_agent.is_empty() {
        return;
    }
    commands.spawn(LocalAgent);
}

fn spawn_sun(mut commands: Commands) {
    let first_cascade_far_bound = cfg_select! {
        all(target_family = "wasm", not(feature = "webgpu")) => 32.0,
        _ => 8.0,
    };

    let num_cascades = cfg_select! {
        all(target_family = "wasm", not(feature = "webgpu")) => 1,
        _ => 3,
    };

    commands.spawn((
        CascadeShadowConfigBuilder {
            first_cascade_far_bound,
            maximum_distance: 64.0,
            minimum_distance: 0.1,
            num_cascades,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::DIRECT_SUNLIGHT,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-0.9, 10.0, 3.8).looking_at(Vec3::ZERO, Vec3::Y),
        MtoonSun,
    ));
}
