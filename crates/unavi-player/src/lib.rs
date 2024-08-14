use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use unavi_avatar::AvatarPlugin;

mod body;
mod controls;
mod input;
mod look;
mod menu;

pub use body::{Player, PlayerCamera};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AvatarPlugin,
            TnuaAvian3dPlugin::default(),
            TnuaControllerPlugin::default(),
        ))
        .insert_resource(input::InputMap::default())
        .add_event::<look::CameraLookEvent>()
        .add_systems(Startup, body::spawn_player)
        .add_systems(
            Update,
            (
                body::set_avatar_head,
                body::setup_first_person,
                input::handle_raycast_input,
                look::grab_mouse,
                menu::play_menu_animation,
                (
                    (
                        input::read_keyboard_input,
                        (look::read_mouse_input, look::apply_camera_look).chain(),
                    ),
                    (
                        (controls::void_teleport, controls::move_player).chain(),
                        body::rotate_avatar_head,
                    ),
                )
                    .chain(),
            ),
        );
    }
}
