use bevy::prelude::*;

mod controls;
mod events;
mod input;
mod look;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(input::InputMap::default())
            .add_event::<events::YawEvent>()
            .add_event::<events::PitchEvent>()
            .init_resource::<look::MouseSettings>()
            .add_systems(Startup, controls::spawn_player)
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
