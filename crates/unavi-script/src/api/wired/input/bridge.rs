use bevy::prelude::*;
use unavi_input::{
    SqueezeDown, SqueezeUp,
    actions::{MenuDesktopAction, MenuLeftHandAction, MenuRightHandAction},
    schminput::prelude::BoolActionValue,
};

use super::{InputAction, InputDevice, InputRegistry, QueuedEvent};

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

pub fn update_menu_buffer(
    menu_desktop: Query<&BoolActionValue, With<MenuDesktopAction>>,
    menu_left: Query<&BoolActionValue, With<MenuLeftHandAction>>,
    menu_right: Query<&BoolActionValue, With<MenuRightHandAction>>,
    mut prev_desktop: Local<bool>,
    mut prev_left: Local<bool>,
    mut prev_right: Local<bool>,
    registry: Res<InputRegistry>,
) {
    push_menu_events(
        menu_desktop.single().ok(),
        &mut prev_desktop,
        InputDevice::Keyboard,
        &registry,
    );
    push_menu_events(
        menu_left.single().ok(),
        &mut prev_left,
        InputDevice::LeftHand,
        &registry,
    );
    push_menu_events(
        menu_right.single().ok(),
        &mut prev_right,
        InputDevice::RightHand,
        &registry,
    );
}

fn push_menu_events(
    value: Option<&BoolActionValue>,
    prev: &mut bool,
    device: InputDevice,
    registry: &InputRegistry,
) {
    let current = value.is_some_and(|v| v.any);
    let pressed = current && !*prev;
    let released = !current && *prev;
    *prev = current;

    if pressed {
        debug!("menu pressed");
        registry.push_system(QueuedEvent {
            action: InputAction::MenuDown,
            device,
        });
    } else if released {
        debug!("menu released");
        registry.push_system(QueuedEvent {
            action: InputAction::MenuUp,
            device,
        });
    }
}
