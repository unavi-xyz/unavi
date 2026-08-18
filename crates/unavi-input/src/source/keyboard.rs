use bevy::prelude::*;

use crate::{
    action::ActionState,
    config::InputConfig,
};

pub fn read(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut state: ResMut<ActionState>,
) {
    for (action, binding) in config.bindings.axes() {
        for dpad in &binding.dpads {
            state.accumulate(action, dpad.value(&keys));
        }
    }

    for (action, binding) in config.bindings.buttons() {
        if binding.keys.iter().any(|key| keys.pressed(*key)) {
            state.press(action, 1.0);
        }
    }
}
