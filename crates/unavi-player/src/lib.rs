use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_xpbd3d::TnuaXpbd3dPlugin;
use unavi_world::WorldState;

mod controls;
mod input;
mod look;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TnuaXpbd3dPlugin::default(), TnuaControllerPlugin::default()))
            .insert_resource(input::InputMap::default())
            .add_event::<look::YawEvent>()
            .add_event::<look::PitchEvent>()
            .init_resource::<look::MouseSettings>()
            .add_systems(OnEnter(WorldState::InWorld), controls::spawn_player)
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
