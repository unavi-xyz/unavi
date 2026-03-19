use bevy::prelude::*;
use unavi_input::{
    SqueezeDown, SqueezeUp, actions::CoreActions, schminput::prelude::BoolActionValue,
};
use unavi_script::{InputAction, InputDevice, InputRegistry, QueuedEvent};

pub fn bridge_squeeze_down(trigger: On<SqueezeDown>, registry: Res<InputRegistry>) {
    registry.push_node(
        trigger.entity,
        QueuedEvent {
            action: InputAction::GrabDown,
            device: InputDevice::RightHand,
        },
    );
}

pub fn bridge_squeeze_up(trigger: On<SqueezeUp>, registry: Res<InputRegistry>) {
    registry.push_node(
        trigger.entity,
        QueuedEvent {
            action: InputAction::GrabUp,
            device: InputDevice::RightHand,
        },
    );
}

pub fn update_action_buffer(
    core_actions: Res<CoreActions>,
    action_values: Query<&BoolActionValue>,
    mut prev_menu: Local<bool>,
    registry: Res<InputRegistry>,
) {
    let current = action_values
        .get(core_actions.menu)
        .is_ok_and(|v: &BoolActionValue| v.any);
    let pressed = current && !*prev_menu;
    let released = !current && *prev_menu;
    *prev_menu = current;

    if pressed {
        registry.push_system(QueuedEvent {
            action: InputAction::MenuDown,
            device: InputDevice::Keyboard,
        });
    } else if released {
        registry.push_system(QueuedEvent {
            action: InputAction::MenuUp,
            device: InputDevice::Keyboard,
        });
    }
}
