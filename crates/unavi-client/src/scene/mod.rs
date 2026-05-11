use bevy::{
    light::{CascadeShadowConfigBuilder, light_consts::lux},
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
            .add_observer(limbo::exit_limbo_on_space_join)
            .add_observer(respawn::respawn)
            .add_systems(
                OnEnter(SceneState::Limbo),
                (limbo::spawn_limbo, spawn_local_agent),
            )
            .add_systems(OnExit(SceneState::Limbo), limbo::despawn_limbo)
            .add_systems(Startup, (spawn_sun, home::join_home))
            .add_systems(FixedUpdate, respawn::teleport_from_void);
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
    commands.spawn((
        CascadeShadowConfigBuilder {
            #[cfg(not(all(target_family = "wasm", not(feature = "webgpu"))))]
            first_cascade_far_bound: 8.0,
            // WebGL onl gets 1 cascade, so push it further back.
            #[cfg(all(target_family = "wasm", not(feature = "webgpu")))]
            first_cascade_far_bound: 20.0,
            maximum_distance: 50.0,
            minimum_distance: 0.1,
            num_cascades: 3,
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
