use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::controls::Player;

#[derive(Resource)]
pub struct InputMap {
    pub key_forward: KeyCode,
    pub key_backward: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_jump: KeyCode,
    pub key_sprint: KeyCode,
    pub key_crouch: KeyCode,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            key_forward: KeyCode::W,
            key_backward: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_jump: KeyCode::Space,
            key_sprint: KeyCode::ShiftLeft,
            key_crouch: KeyCode::ControlLeft,
        }
    }
}

pub fn read_keyboard_input(
    keys: Res<Input<KeyCode>>,
    input_map: Res<InputMap>,
    mut players: Query<&mut Player>,
) {
    for mut player in players.iter_mut() {
        player.input.forward = keys.pressed(input_map.key_forward);
        player.input.backward = keys.pressed(input_map.key_backward);
        player.input.left = keys.pressed(input_map.key_left);
        player.input.right = keys.pressed(input_map.key_right);
        player.input.jump = keys.pressed(input_map.key_jump);
        player.input.sprint = keys.pressed(input_map.key_sprint);
    }
}
