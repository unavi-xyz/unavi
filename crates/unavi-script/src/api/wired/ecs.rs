use std::collections::HashMap;

use bevy::prelude::info;
use tokio::sync::mpsc::Sender;
use wasmtime::component::HasData;
use wired::ecs::types::{Component, ComponentId, Schedule, System, SystemId};

use crate::commands::WasmCommand;

wasmtime::component::bindgen!({
    world: "host",
    path: "../../protocol/wit/wired-ecs",
    additional_derives: [Hash, PartialEq],
    async: true,
});

pub struct HasWiredEcsData;

impl HasData for HasWiredEcsData {
    type Data<'a> = &'a mut WiredEcsData;
}

pub struct WiredEcsData {
    pub initialized: bool,

    pub components: Vec<Component>,
    pub systems: Vec<System>,
    pub schedules: HashMap<Schedule, Vec<SystemId>>,

    pub commands: Sender<WasmCommand>,
}

const MAX_COMPONENTS: usize = 1_000;
const MAX_SYSTEMS: usize = 1_000;

impl wired::ecs::host_api::Host for WiredEcsData {
    async fn register_component(&mut self, component: Component) -> Result<ComponentId, String> {
        // Each component is a unique (key, type) pair. This ensures that:
        // 1. Multiple components with the same type can be created.
        // 2. Multiple components with the same key don't cause type conflicts.
        //    (This is important to avoid key sabotage, where script A registers script B's
        //    component with the wrong type.)
        if let Some(id) = self.components.iter().position(|c| *c == component) {
            Ok(id as u64)
        } else {
            let id = self.components.len() as u64;
            info!("Registering component {id}: {}", component.key);

            if self.components.len() >= MAX_COMPONENTS {
                Err("Max component count reached".to_string())
            } else {
                self.commands
                    .send(WasmCommand::RegisterComponent { id })
                    .await
                    .map_err(|e| format!("Error sending command: {e}"))?;
                self.components.push(component);
                Ok(id)
            }
        }
    }

    async fn register_system(&mut self, system: System) -> Result<SystemId, String> {
        let id = self.systems.len() as u64;
        info!("Registering system {id}");

        if self.systems.len() >= MAX_SYSTEMS {
            Err("Max system count reached".to_string())
        } else {
            self.commands
                .send(WasmCommand::RegisterSystem { id })
                .await
                .map_err(|e| format!("Error sending command: {e}"))?;
            self.schedules.entry(system.schedule).or_default().push(id);
            self.systems.push(system);
            Ok(id)
        }
    }
}
