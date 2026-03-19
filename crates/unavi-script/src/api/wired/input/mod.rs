use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, Weak},
};

use bevy::prelude::*;
use wasmtime::component::{Resource, ResourceTable};

use crate::{api::wired::scene::node::HostNode, load::state::RuntimeData};

#[derive(Clone, Copy)]
pub enum InputAction {
    GrabDown,
    GrabUp,
    MenuDown,
    MenuUp,
}

#[derive(Clone, Copy)]
pub enum InputDevice {
    Keyboard,
    LeftHand,
    RightHand,
}

#[derive(Clone, Copy)]
pub struct QueuedEvent {
    pub action: InputAction,
    pub device: InputDevice,
}

type HandlerQueue = Arc<Mutex<VecDeque<QueuedEvent>>>;
type WeakQueue = Weak<Mutex<VecDeque<QueuedEvent>>>;

#[derive(Default)]
struct InnerRegistry {
    node_handlers: HashMap<Entity, Vec<WeakQueue>>,
    system_handlers: Vec<WeakQueue>,
}

impl InnerRegistry {
    fn push_node(&mut self, entity: Entity, event: QueuedEvent) {
        if let Some(queues) = self.node_handlers.get_mut(&entity) {
            queues.retain(|w| {
                w.upgrade().is_some_and(|q| {
                    q.lock().expect("queue lock").push_back(event);
                    true
                })
            });
        }
        self.push_system(event);
    }

    fn push_system(&mut self, event: QueuedEvent) {
        self.system_handlers.retain(|w| {
            w.upgrade().is_some_and(|q| {
                q.lock().expect("queue lock").push_back(event);
                true
            })
        });
    }

    fn register_node(&mut self, entity: Entity) -> HandlerQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        self.node_handlers
            .entry(entity)
            .or_default()
            .push(Arc::downgrade(&queue));
        queue
    }

    fn register_system(&mut self) -> HandlerQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        self.system_handlers.push(Arc::downgrade(&queue));
        queue
    }
}

#[derive(Resource, Clone, Default)]
pub struct InputRegistry(Arc<Mutex<InnerRegistry>>);

impl InputRegistry {
    pub fn push_node(&self, entity: Entity, event: QueuedEvent) {
        self.0
            .lock()
            .expect("registry lock")
            .push_node(entity, event);
    }

    pub fn push_system(&self, event: QueuedEvent) {
        self.0.lock().expect("registry lock").push_system(event);
    }
}

#[derive(Default)]
pub struct WiredInputRt {
    pub registry: InputRegistry,
    pub table: ResourceTable,
}

pub struct HostInputListener {
    pub queue: HandlerQueue,
}

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
            .unwrap_or(Entity::PLACEHOLDER);
        let queue = self
            .wired_input
            .registry
            .0
            .lock()
            .expect("registry lock")
            .register_node(entity);
        Ok(self.wired_input.table.push(HostInputListener { queue })?)
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
        self.wired_input.table.delete(rep)?;
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
        Ok(self.wired_input.table.push(HostInputListener { queue })?)
    }
}
