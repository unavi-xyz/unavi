use std::collections::HashMap;

use bevy::prelude::info;
use wasmtime::component::HasData;
use wired::ecs::types::{Component, ComponentId, Query, QueryId, Schedule, System, SystemId};

wasmtime::component::bindgen!({
    world: "host",
    path: "../../protocol/wit/wired-ecs",
    additional_derives: [Hash, PartialEq],
});

pub struct HasWiredEcsData;

impl HasData for HasWiredEcsData {
    type Data<'a> = &'a mut WiredEcsData;
}

#[derive(Default)]
pub struct WiredEcsData {
    pub initialized: bool,

    pub components: Vec<Component>,
    pub systems: Vec<System>,
    pub schedules: HashMap<Schedule, Vec<SystemId>>,
}

impl wired::ecs::host_api::Host for WiredEcsData {
    fn register_component(&mut self, component: Component) -> ComponentId {
        // Each component is a unique (key, type) pair. This ensures that:
        // 1. Multiple components with the same type can be created.
        // 2. Multiple components with the same key don't cause type conflicts.
        //    (This is important to avoid key sabotage, where script A registers script B's
        //    component with the wrong type.)
        if let Some(id) = self.components.iter().position(|c| *c == component) {
            id as u64
        } else {
            let id = self.components.len() as u64;
            info!("Registering component {id}: {}", component.key);
            self.components.push(component);
            id
        }
    }
    fn register_system(&mut self, system: System) -> SystemId {
        let id = self.systems.len() as u64;
        info!("Registering system {id}");
        self.schedules.entry(system.schedule).or_default().push(id);
        self.systems.push(system);
        id
    }
}
