use bevy::{
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
};
use bevy_hsd::StageNodesLoaded;
use bevy_vrm::mtoon::MtoonSun;
use unavi_locomotion::LocalAgentSpawner;

pub fn spawn_agent(
    asset_server: Res<AssetServer>,
    new_stages: Query<(), Added<StageNodesLoaded>>,
    mut commands: Commands,
    mut spawned: Local<bool>,
) {
    if *spawned || new_stages.is_empty() {
        return;
    }

    // TODO pass in xr mode to spawner config
    LocalAgentSpawner::default().spawn(&mut commands, &asset_server);

    *spawned = true;
}

pub fn spawn_scene(mut commands: Commands, mut ambient: ResMut<AmbientLight>) {
    ambient.brightness = lux::OVERCAST_DAY;

    commands.spawn((
        CascadeShadowConfigBuilder {
            #[cfg(not(target_family = "wasm"))]
            first_cascade_far_bound: 8.0,
            #[cfg(not(target_family = "wasm"))]
            maximum_distance: 50.0,
            #[cfg(target_family = "wasm")]
            maximum_distance: 20.0,
            minimum_distance: 0.1,
            num_cascades: 3,
            ..Default::default()
        }
        .build(),
        DirectionalLight {
            illuminance: lux::DIRECT_SUNLIGHT / 2.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-0.9, 10.0, 3.8).looking_at(Vec3::ZERO, Vec3::Y),
        MtoonSun,
    ));
}
