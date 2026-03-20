use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

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

pub(super) type ListenerQueue = Arc<Mutex<VecDeque<QueuedEvent>>>;

#[derive(Default)]
pub(super) struct InnerRegistry {
    pub(super) node_listeners: HashMap<Entity, Vec<ListenerQueue>>,
    pub(super) system_listeners: Vec<ListenerQueue>,
}

impl InnerRegistry {
    pub(super) fn push_node(&self, entity: Entity, event: QueuedEvent) {
        if let Some(queues) = self.node_listeners.get(&entity) {
            for queue in queues {
                queue.lock().expect("queue lock").push_back(event);
            }
        }
        self.push_system(event);
    }

    pub(super) fn push_system(&self, event: QueuedEvent) {
        for queue in &self.system_listeners {
            queue.lock().expect("queue lock").push_back(event);
        }
    }

    pub(super) fn register_node(&mut self, entity: Entity) -> ListenerQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        self.node_listeners
            .entry(entity)
            .or_default()
            .push(Arc::clone(&queue));
        queue
    }

    pub(super) fn register_system(&mut self) -> ListenerQueue {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        self.system_listeners.push(Arc::clone(&queue));
        queue
    }

    pub(super) fn remove_node(&mut self, entity: Entity, queue: &ListenerQueue) {
        if let Some(queues) = self.node_listeners.get_mut(&entity) {
            queues.retain(|q| !Arc::ptr_eq(q, queue));
        }
    }

    pub(super) fn remove_system(&mut self, queue: &ListenerQueue) {
        self.system_listeners.retain(|q| !Arc::ptr_eq(q, queue));
    }
}

#[derive(Resource, Clone, Default)]
pub struct InputRegistry(pub(super) Arc<Mutex<InnerRegistry>>);

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
