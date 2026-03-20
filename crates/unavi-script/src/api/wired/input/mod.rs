use std::sync::Arc;

use wasmtime::component::{Resource, ResourceTable};

use crate::{api::wired::scene::node::HostNode, load::state::RuntimeData};

pub mod bridge;
pub mod registry;

pub use registry::{InputAction, InputDevice, InputRegistry, QueuedEvent};

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-input",
        with: {
            "wired:scene/types.node":
                crate::api::wired::scene::node::HostNode,
            "wired:input/types.input-listener": super::HostInputListener,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::input::types::{
    InputAction as WitAction, InputDevice as WitDevice, InputEvent,
};

#[derive(Default)]
pub struct WiredInputRt {
    pub registry: InputRegistry,
    pub table: ResourceTable,
}

pub struct HostInputListener {
    pub queue: registry::ListenerQueue,
    registry: InputRegistry,
    entity: Option<bevy::prelude::Entity>,
}

const fn to_wit_action(a: InputAction) -> WitAction {
    match a {
        InputAction::GrabDown => WitAction::GrabDown,
        InputAction::GrabUp => WitAction::GrabUp,
        InputAction::MenuDown => WitAction::MenuDown,
        InputAction::MenuUp => WitAction::MenuUp,
    }
}

const fn to_wit_device(d: InputDevice) -> WitDevice {
    match d {
        InputDevice::Keyboard => WitDevice::Keyboard,
        InputDevice::LeftHand => WitDevice::LeftHand,
        InputDevice::RightHand => WitDevice::RightHand,
    }
}

impl bindings::wired::input::api::Host for RuntimeData {
    async fn register_input_listener(
        &mut self,
        target: Resource<HostNode>,
    ) -> wasmtime::Result<Resource<HostInputListener>> {
        let inner = Arc::clone(&self.wired_scene.table.get(&target)?.inner);
        let entity = inner
            .entity
            .lock()
            .expect("entity lock")
            .unwrap_or(bevy::prelude::Entity::PLACEHOLDER);
        let queue = self
            .wired_input
            .registry
            .0
            .lock()
            .expect("registry lock")
            .register_node(entity);
        Ok(self.wired_input.table.push(HostInputListener {
            queue,
            registry: self.wired_input.registry.clone(),
            entity: Some(entity),
        })?)
    }
}

impl bindings::wired::input::types::Host for RuntimeData {}

impl bindings::wired::input::types::HostInputListener for RuntimeData {
    async fn poll(
        &mut self,
        self_: Resource<HostInputListener>,
    ) -> wasmtime::Result<Option<InputEvent>> {
        let listener = self.wired_input.table.get(&self_)?;
        let event = listener.queue.lock().expect("queue lock").pop_front();
        Ok(event.map(|e| InputEvent {
            action: to_wit_action(e.action),
            device: to_wit_device(e.device),
        }))
    }

    async fn drop(&mut self, rep: Resource<HostInputListener>) -> wasmtime::Result<()> {
        let listener = self.wired_input.table.delete(rep)?;
        let mut inner = listener.registry.0.lock().expect("registry lock");
        match listener.entity {
            Some(entity) => inner.remove_node(entity, &listener.queue),
            None => inner.remove_system(&listener.queue),
        }
        drop(inner);
        Ok(())
    }
}

impl bindings::wired::input::system_api::Host for RuntimeData {
    async fn system_input_listener(&mut self) -> wasmtime::Result<Resource<HostInputListener>> {
        let queue = self
            .wired_input
            .registry
            .0
            .lock()
            .expect("registry lock")
            .register_system();
        Ok(self.wired_input.table.push(HostInputListener {
            queue,
            registry: self.wired_input.registry.clone(),
            entity: None,
        })?)
    }
}
