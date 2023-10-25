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
            .add_event::<events::LookEvent>()
            .add_event::<events::LookDeltaEvent>()
            .init_resource::<look::MouseSettings>()
            .add_systems(Startup, controls::spawn_player)
            .add_systems(PreUpdate, (look::input_to_look, look::forward_up).chain())
            .add_systems(
                Update,
                (
                    look::grab_mouse,
                    (
                        (
                            input::read_input,
                            controls::apply_yaw,
                            controls::apply_pitch,
                            controls::void_teleport,
                        ),
                        controls::move_player,
                    )
                        .chain(),
                ),
            );
    }
}
