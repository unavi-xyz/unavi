//! VRM player controller.

use bevy::{core_pipeline::auto_exposure::AutoExposurePlugin, prelude::*};
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use bevy_vrm::VrmPlugins;

// mod head;
mod input;
mod spawner;

pub use spawner::*;
use unavi_input::CursorGrabState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AutoExposurePlugin,
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            VrmPlugins,
        ))
        .init_state::<CursorGrabState>()
        .add_systems(
            Update,
            (
                input::apply_head_input.run_if(in_state(CursorGrabState::Locked)),
                input::apply_body_input,
            )
                .chain(),
        );
    }
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct Player {}

#[derive(Component)]
struct RealHeight(f32);

#[derive(Component)]
struct WalkSpeed(f32);

#[derive(Component)]
struct JumpStrength(f32);

#[derive(Component, Default)]
#[require(Transform)]
struct PlayerBody;

#[derive(Component, Default)]
#[require(Transform)]
struct PlayerHead;

#[derive(Component, Default)]
#[require(Camera3d, Transform)]
struct PlayerCamera;
