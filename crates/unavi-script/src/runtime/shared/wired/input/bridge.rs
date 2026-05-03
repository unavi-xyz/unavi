use async_channel::Sender;
use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdRecordId, NodeId};
use blake3::Hash;
use loro::TreeID;
use unavi_input::{
    SqueezeDown, SqueezeUp,
    actions::{MenuDesktopAction, MenuLeftHandAction, MenuRightHandAction},
    raycast::PrimaryRaycastInput,
    schminput::BoolActionValue,
};

use crate::runtime::native::wired::input::bindings::wired::input::types::{
    InputAction, InputDevice, InputEvent,
};

#[derive(Component)]
pub struct GlobalInputListener {
    pub tx: Sender<InputEvent>,
}

#[derive(Component)]
pub struct InputListener {
    pub target_doc: Hash,
    pub target_node: TreeID,
    pub tx: Sender<InputEvent>,
}

pub fn bridge_squeeze_down(
    trigger: On<SqueezeDown>,
    raycasters: Query<(), With<PrimaryRaycastInput>>,
) -> Option<SendInput> {
    let device = match raycasters.get(trigger.pointer) {
        Ok(()) => InputDevice::Keyboard,
        Err(_) => {
            // TODO get VR handedness
            return None;
        }
    };

    let event = InputEvent {
        action: InputAction::GrabDown,
        device,
    };

    Some(SendInput {
        event,
        target_node: Some(trigger.entity),
    })
}

pub fn bridge_squeeze_up(
    trigger: On<SqueezeUp>,
    raycasters: Query<(), With<PrimaryRaycastInput>>,
) -> Option<SendInput> {
    let device = match raycasters.get(trigger.pointer) {
        Ok(()) => InputDevice::Keyboard,
        Err(_) => {
            // TODO get VR handedness
            return None;
        }
    };

    let event = InputEvent {
        action: InputAction::GrabUp,
        device,
    };

    Some(SendInput {
        event,
        target_node: Some(trigger.entity),
    })
}

pub struct MenuInput {
    device: InputDevice,
    value: bool,
    prev: bool,
}

pub fn bridge_menu_desktop(
    action: Query<&BoolActionValue, With<MenuDesktopAction>>,
    mut prev: Local<bool>,
) -> Option<SendInput> {
    let value = action.single().is_ok_and(|a| a.any);
    if value == *prev {
        return None;
    }
    let input = MenuInput {
        device: InputDevice::Keyboard,
        value,
        prev: *prev,
    };
    *prev = value;
    Some(bridge_menu(input))
}

pub fn bridge_menu_left(
    action: Query<&BoolActionValue, With<MenuLeftHandAction>>,
    mut prev: Local<bool>,
) -> Option<SendInput> {
    let value = action.single().is_ok_and(|a| a.any);
    if value == *prev {
        return None;
    }
    let input = MenuInput {
        device: InputDevice::LeftHand,
        value,
        prev: *prev,
    };
    *prev = value;
    Some(bridge_menu(input))
}

pub fn bridge_menu_right(
    action: Query<&BoolActionValue, With<MenuRightHandAction>>,
    mut prev: Local<bool>,
) -> Option<SendInput> {
    let value = action.single().is_ok_and(|a| a.any);
    if value == *prev {
        return None;
    }
    let input = MenuInput {
        device: InputDevice::RightHand,
        value,
        prev: *prev,
    };
    *prev = value;
    Some(bridge_menu(input))
}

const fn bridge_menu(input: MenuInput) -> SendInput {
    let action = if input.value && !input.prev {
        InputAction::MenuDown
    } else {
        InputAction::MenuUp
    };

    let event = InputEvent {
        action,
        device: input.device,
    };

    SendInput {
        event,
        target_node: None,
    }
}

pub struct SendInput {
    pub event: InputEvent,
    pub target_node: Option<Entity>,
}

pub fn send_to_listeners(
    trigger: In<Option<SendInput>>,
    global: Query<&GlobalInputListener>,
    listeners: Query<&InputListener>,
    nodes: Query<(&NodeId, &HsdChild)>,
    docs: Query<&HsdRecordId>,
) {
    let Some(trigger) = &*trigger else {
        return;
    };

    for g in global {
        let _ = g.tx.try_send(trigger.event);
    }

    let Some(target_node) = trigger.target_node else {
        return;
    };

    let Ok((id, node_doc)) = nodes.get(target_node) else {
        return;
    };

    let Ok(doc_id) = docs.get(node_doc.0) else {
        return;
    };

    for l in listeners {
        if l.target_node != id.0 && l.target_doc != doc_id.0 {
            continue;
        }

        let _ = l.tx.try_send(trigger.event);
    }
}
