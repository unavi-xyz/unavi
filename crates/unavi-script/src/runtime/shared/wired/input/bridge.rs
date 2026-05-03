use async_channel::Sender;
use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdRecordId, NodeId};
use blake3::Hash;
use loro::TreeID;
use unavi_input::{SqueezeDown, SqueezeUp, raycast::PrimaryRaycastInput};

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
) -> SendInput {
    let device = match raycasters.get(trigger.pointer) {
        Ok(()) => InputDevice::Keyboard,
        Err(_) => {
            todo!("get VR handedness")
        }
    };

    let event = InputEvent {
        action: InputAction::GrabDown,
        device,
    };

    SendInput {
        event,
        target_node: trigger.entity,
    }
}

pub fn bridge_squeeze_up(
    trigger: On<SqueezeUp>,
    raycasters: Query<(), With<PrimaryRaycastInput>>,
) -> SendInput {
    let device = match raycasters.get(trigger.pointer) {
        Ok(()) => InputDevice::Keyboard,
        Err(_) => {
            todo!("get VR handedness")
        }
    };

    let event = InputEvent {
        action: InputAction::GrabUp,
        device,
    };

    SendInput {
        event,
        target_node: trigger.entity,
    }
}

// TODO menu input

pub struct SendInput {
    pub event: InputEvent,
    pub target_node: Entity,
}

pub fn send_to_listeners(
    trigger: In<SendInput>,
    global: Query<&GlobalInputListener>,
    listeners: Query<&InputListener>,
    nodes: Query<(&NodeId, &HsdChild)>,
    docs: Query<&HsdRecordId>,
) {
    for g in global {
        let _ = g.tx.try_send(trigger.event);
    }

    let Ok((id, node_doc)) = nodes.get(trigger.target_node) else {
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
