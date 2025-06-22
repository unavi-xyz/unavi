//! VRM player controller.

use bevy::prelude::*;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_vrm::VrmPlugins;

mod head;
mod spawner;

pub use spawner::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            VrmPlugins,
        ));
    }
}

/// | Group            | Height        |
/// | ---------------- | ------------- |
/// | Adult Male       | 1.70 – 1.78 m |
/// | Adult Female     | 1.60 – 1.67 m |
const DEFAULT_HEIGHT: f32 = 1.7;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Player {}

#[derive(Component)]
struct PlayerHeight(f32);

impl Default for PlayerHeight {
    fn default() -> Self {
        Self(DEFAULT_HEIGHT)
    }
}

#[derive(Component)]
struct AvatarHeight(f32);

impl Default for AvatarHeight {
    fn default() -> Self {
        Self(DEFAULT_HEIGHT)
    }
}

#[derive(Component, Default)]
#[require(Transform, PlayerHeight)]
struct PlayerBody;

#[derive(Component, Default)]
#[require(Transform)]
struct PlayerHead;

#[derive(Component, Default)]
#[require(Camera3d, Transform)]
struct PlayerCamera;
