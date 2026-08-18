use bevy::prelude::*;

use crate::{
    action::ActionState,
    config::InputConfig,
    source::deadzone,
};

pub fn read(gamepads: Query<&Gamepad>, config: Res<InputConfig>, mut state: ResMut<ActionState>) {
    let stick_deadzone = config.tuning.stick_deadzone;

    for gamepad in gamepads {
        for (action, binding) in config.bindings.axes() {
            for stick in &binding.sticks {
                let (x, y) = stick.axes();
                let raw = Vec2::new(
                    gamepad.get(x).unwrap_or_default(),
                    gamepad.get(y).unwrap_or_default(),
                );
                state.accumulate(action, deadzone(raw, stick_deadzone));
            }
        }

        for (action, binding) in config.bindings.buttons() {
            let held = binding
                .pad
                .iter()
                .map(|button| gamepad.get(*button).unwrap_or_default())
                .fold(0.0_f32, f32::max);
            if held > 0.0 {
                state.press(action, held);
            }
        }
    }
}
