//! VRM player controller.

use bevy::prelude::*;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_vrm::VrmPlugins;

mod head;
mod input;
mod spawner;

pub use spawner::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            VrmPlugins,
        ))
        .add_systems(
            Update,
            (input::apply_head_input, input::apply_body_input).chain(),
        );
    }
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct Player {}

#[derive(Component)]
struct PlayerHeight(f32);

#[derive(Component)]
struct PlayerSpeed(f32);

#[derive(Component, Default)]
#[require(Transform)]
struct PlayerBody;

#[derive(Component, Default)]
#[require(Transform)]
struct PlayerHead;

#[derive(Component, Default)]
#[require(Camera3d, Transform)]
struct PlayerCamera;
