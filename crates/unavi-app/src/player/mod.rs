use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_xpbd3d::TnuaXpbd3dPlugin;

use crate::state::AppState;

mod controls;
mod events;
mod input;
mod look;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TnuaXpbd3dPlugin, TnuaControllerPlugin))
            .insert_resource(input::InputMap::default())
            .add_event::<events::YawEvent>()
            .add_event::<events::PitchEvent>()
            .init_resource::<look::MouseSettings>()
            .add_systems(OnEnter(AppState::InWorld), controls::spawn_player)
            .add_systems(
                Update,
                (
                    look::grab_mouse,
                    (controls::void_teleport, input::read_keyboard_input)
                        .before(controls::move_player),
                    (
                        look::read_mouse_input,
                        (
                            look::set_look_direction,
                            controls::apply_yaw,
                            controls::apply_pitch,
                        ),
                        controls::move_player,
                    )
                        .chain(),
                ),
            );
    }
}
