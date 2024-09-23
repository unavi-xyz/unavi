use bevy::prelude::*;
use bevy_vr_controller::movement::PlayerInputState;

use crate::menu::MenuState;

#[derive(Resource)]
pub struct UnaviInputMap {
    pub key_menu: KeyCode,
}

impl Default for UnaviInputMap {
    fn default() -> Self {
        Self {
            key_menu: KeyCode::Tab,
        }
    }
}

pub fn read_keyboard_input(
    input_map: Res<UnaviInputMap>,
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<State<MenuState>>,
    mut next_menu: ResMut<NextState<MenuState>>,
    players: Query<&PlayerInputState>,
) {
    for input in players.iter() {
        let did_move = input.forward || input.backward || input.left || input.right || input.jump;

        if did_move {
            next_menu.set(MenuState::Closed);
        } else if keys.just_pressed(input_map.key_menu) {
            match menu.get() {
                MenuState::Closed => next_menu.set(MenuState::Open),
                MenuState::Open => next_menu.set(MenuState::Closed),
            }
        }
    }
}
