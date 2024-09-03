use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use unavi_avatar::AvatarPlugin;

mod body;
mod controls;
mod input;
mod layers;
mod look;
mod menu;

pub use body::{LocalPlayer, PlayerCamera};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AvatarPlugin,
            TnuaAvian3dPlugin::default(),
            TnuaControllerPlugin::default(),
        ))
        .add_event::<look::CameraLookEvent>()
        .init_state::<menu::MenuState>()
        .insert_resource(input::InputMap::default())
        .add_systems(Startup, body::spawn_player)
        .add_systems(OnEnter(menu::MenuState::Open), menu::open_menu)
        .add_systems(OnExit(menu::MenuState::Open), menu::close_menu)
        .add_systems(
            Update,
            (
                body::calc_eye_offset,
                body::id_player_bones,
                body::set_avatar_head,
                body::setup_first_person,
                input::read_keyboard_input,
                look::grab_mouse,
                (
                    (look::read_mouse_input, look::apply_camera_look).chain(),
                    (
                        (
                            controls::void_teleport,
                            controls::move_player.before(input::read_keyboard_input),
                        )
                            .chain(),
                        body::rotate_avatar_head,
                        input::handle_raycast_input,
                    ),
                )
                    .chain(),
            ),
        );
    }
}
