use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use wasmtime::component::HasData;
use wired::ecs::types::{
    Component, ComponentId, ComponentType, Primitive, Schedule, System, SystemId,
};

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
const MAX_COMPONENT_SIZE: usize = 10_000;
const MAX_COMPONENT_TYPES: usize = 100;
const MAX_KEY_LEN: usize = 200;

const MAX_SYSTEMS: usize = 1_000;
const MAX_SYSTEM_PARAMS: usize = 16;

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
            if component.key.len() > MAX_KEY_LEN {
                return Err("Key too long".to_string());
            }

            let id = self.components.len() as u64;

            if self.components.len() >= MAX_COMPONENTS {
                return Err("Max component count reached".to_string());
            }
            if component.types.len() > MAX_COMPONENT_TYPES {
                return Err("Too many component types".to_string());
            }

            let size = component.types.iter().map(|t| t.size()).sum::<usize>();
            if size > MAX_COMPONENT_SIZE {
                return Err("Component too large".to_string());
            }

            self.commands
                .send(WasmCommand::RegisterComponent {
                    id,
                    key: component.key.clone(),
                    size,
                })
                .await
                .map_err(|e| format!("Error sending command: {e}"))?;
            self.components.push(component);
            Ok(id)
        }
    }

    async fn register_system(&mut self, system: System) -> Result<SystemId, String> {
        let id = self.systems.len() as u64;

        if self.systems.len() >= MAX_SYSTEMS {
            return Err("Max system count reached".to_string());
        }
        if system.params.len() > MAX_SYSTEM_PARAMS {
            return Err("Too many system params".to_string());
        }

        self.commands
            .send(WasmCommand::RegisterSystem {
                id,
                schedule: system.schedule,
            })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        self.schedules.entry(system.schedule).or_default().push(id);
        self.systems.push(system);
        Ok(id)
    }
}

impl ComponentType {
    fn size(&self) -> usize {
        match self {
            ComponentType::Primitive(p) => p.size(),
            ComponentType::Map(_) => todo!(),
            ComponentType::Vec(_) => todo!(),
        }
    }
}

impl Primitive {
    fn size(&self) -> usize {
        match self {
            Primitive::Bool => std::mem::size_of::<bool>(),
            Primitive::F32 => std::mem::size_of::<f32>(),
            Primitive::F64 => std::mem::size_of::<f64>(),
            Primitive::I8 => std::mem::size_of::<i8>(),
            Primitive::I16 => std::mem::size_of::<i16>(),
            Primitive::I32 => std::mem::size_of::<i32>(),
            Primitive::I64 => std::mem::size_of::<i64>(),
            Primitive::U8 => std::mem::size_of::<u8>(),
            Primitive::U16 => std::mem::size_of::<u16>(),
            Primitive::U32 => std::mem::size_of::<u32>(),
            Primitive::U64 => std::mem::size_of::<u64>(),
            Primitive::String => todo!(),
        }
    }
}
