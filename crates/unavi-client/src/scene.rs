use std::time::Duration;

use bevy::{
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    prelude::*,
};
use bevy_hsd::HsdDoc;
use bevy_vrm::mtoon::MtoonSun;
use unavi_agent::LocalAgent;
use unavi_space::Space;

const SPAWN_DELAY: Duration = Duration::from_secs(1);

pub fn spawn_agent(
    spaces: Query<(), (With<Space>, With<HsdDoc>)>,
    mut commands: Commands,
    mut spawned: Local<bool>,
    time: Res<Time>,
    mut started: Local<Option<Duration>>,
) {
    if *spawned || spaces.is_empty() {
        return;
    }

    let now = time.elapsed();

    if let Some(s) = &*started {
        if *s + SPAWN_DELAY > now {
            return;
        }
    } else {
        *started = Some(now);
        return;
    }

    commands.spawn(LocalAgent);
    *spawned = true;
}

pub fn spawn_scene(mut commands: Commands) {
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
