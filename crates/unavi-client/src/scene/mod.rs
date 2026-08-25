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

pub mod home;
mod limbo;
mod respawn;
mod system_scripts;
mod travel;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .init_resource::<limbo::LimboArrival>()
            .add_observer(limbo::enter_space)
            .add_observer(respawn::respawn)
            .add_systems(
                OnEnter(SceneState::Limbo),
                (
                    limbo::arm_limbo_arrival,
                    limbo::spawn_limbo,
                    spawn_local_agent,
                ),
            )
            .add_systems(
                OnExit(SceneState::Limbo),
                (limbo::despawn_limbo, build_iroh_router),
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
                (limbo::fall_back_to_limbo, limbo::drive_limbo_exit).chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    travel::drive_travel,
                    limbo::hold_agent_in_limbo.run_if(in_state(SceneState::Limbo)),
                    respawn::teleport_from_void,
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
            shadow_maps_enabled: true,
            // soft_shadow_size: Some(0.2),
            ..Default::default()
        },
        Transform::from_xyz(-0.9, 10.0, 3.8).looking_at(Vec3::ZERO, Vec3::Y),
        MtoonSun,
    ));
}
