use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use controls::InputState;

mod controls;
mod input;
mod look;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaAvian3dPlugin::default(),
            TnuaControllerPlugin::default(),
        ))
        .insert_resource(input::InputMap::default())
        .add_event::<look::YawEvent>()
        .add_event::<look::PitchEvent>()
        .init_resource::<look::LookDirection>()
        .add_systems(Startup, controls::spawn_player)
        .add_systems(
            Update,
            (
                input::handle_raycast_input,
                look::grab_mouse,
                (controls::void_teleport, input::read_keyboard_input).before(controls::move_player),
                (
                    look::read_mouse_input,
                    (controls::apply_yaw, controls::apply_pitch),
                    controls::move_player,
                )
                    .chain(),
            ),
        );
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub sprint_speed: f32,
    pub jump_height: f32,
    pub velocity: Vec3,
    pub input: InputState,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 7.0,
            sprint_speed: 10.0,
            jump_height: 2.0,
            velocity: Vec3::ZERO,
            input: InputState::default(),
        }
    }
}

#[derive(Component)]
pub struct PlayerCamera;
