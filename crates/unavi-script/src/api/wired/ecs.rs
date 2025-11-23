use std::sync::Arc;

use wasmtime::component::HasData;
use wired::ecs::{
    host_api::SystemOrder,
    types::{Component, ComponentId, EntityId, System, SystemId},
};

use crate::commands::WasmCommand;

wasmtime::component::bindgen!({
    world: "host",
    path: "../../protocol/wit/wired-ecs",
    additional_derives: [Hash, PartialEq],
    imports: {
        default: async,
    },
    exports: {
        default: async,
    }
});

pub struct HasWiredEcsData;

impl HasData for HasWiredEcsData {
    type Data<'a> = &'a mut WiredEcsData;
}

#[derive(Clone)]
pub struct ComponentWrite {
    pub entity: EntityId,
    pub component: ComponentId,
    pub data: Arc<Vec<u8>>,
}

pub struct WiredEcsData {
    pub commands: tokio::sync::mpsc::Sender<WasmCommand>,
    pub write: tokio::sync::broadcast::Sender<ComponentWrite>,
    pub components: Vec<Component>,
    pub entity_id: EntityId,
    pub systems: Vec<System>,
}

const KB: usize = 1024;

const MAX_COMPONENTS: usize = 10_000;
const MAX_COMPONENT_BYTES: usize = 50 * KB;
const MAX_ENTITIES: u64 = 1_000_000;
const MAX_KEY_LEN: usize = 512;
const MAX_SYSTEMS: usize = 10_000;
const MAX_SYSTEM_PARAMS: usize = 16;

// TODO: VObject memory tracking (counter on component add / remove)

impl wired::ecs::host_api::Host for WiredEcsData {
    async fn register_component(&mut self, component: Component) -> Result<ComponentId, String> {
        if let Some(id) = self.components.iter().position(|c| *c == component) {
            Ok(id as u32)
        } else {
            if component.key.len() > MAX_KEY_LEN {
                return Err("Component key too long".to_string());
            }
            if self.components.len() >= MAX_COMPONENTS {
                return Err("Max component count reached".to_string());
            }

            let id = self.components.len() as u32;

            self.commands
                .send(WasmCommand::RegisterComponent { id })
                .await
                .map_err(|e| format!("Error sending command: {e}"))?;

            self.components.push(component);

            Ok(id)
        }
    }

    async fn register_system(&mut self, system: System) -> Result<SystemId, String> {
        let id = self.systems.len() as u32;

        if self.systems.len() >= MAX_SYSTEMS {
            return Err("Max system count reached".to_string());
        }
        if system.params.len() > MAX_SYSTEM_PARAMS {
            return Err("Too many system params".to_string());
        }

        self.commands
            .send(WasmCommand::RegisterSystem {
                id,
                system: system.clone(),
            })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        self.systems.push(system);

        Ok(id)
    }

    async fn order_systems(
        &mut self,
        a: SystemId,
        order: SystemOrder,
        b: SystemId,
    ) -> Result<(), String> {
        if a == b {
            return Err("Cannot order a system against itself".to_string());
        }
        self.commands
            .send(WasmCommand::OrderSystems { a, order, b })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        Ok(())
    }

    async fn spawn(&mut self) -> Result<EntityId, String> {
        let id = self.entity_id + 1;

        if id >= MAX_ENTITIES {
            return Err("Max entity count reached".to_string());
        }

        self.commands
            .send(WasmCommand::Spawn { id })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        self.entity_id = id;

        Ok(id)
    }
    async fn despawn(&mut self, entity: EntityId) -> Result<(), String> {
        self.commands
            .send(WasmCommand::Despawn { id: entity })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        Ok(())
    }

    async fn insert_component(
        &mut self,
        entity: EntityId,
        component: ComponentId,
        data: Vec<u8>,
    ) -> Result<(), String> {
        if data.len() > MAX_COMPONENT_BYTES {
            return Err("Max component size reached".to_string());
        }
        self.commands
            .send(WasmCommand::WriteComponent {
                entity_id: entity,
                component_id: component,
                data,
                insert: true,
            })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        Ok(())
    }
    async fn write_component(
        &mut self,
        entity: EntityId,
        component: ComponentId,
        data: Vec<u8>,
    ) -> Result<(), String> {
        if data.len() > MAX_COMPONENT_BYTES {
            return Err("Max component size reached".to_string());
        }

        // Immediate write to any pending system calls.
        if self.write.receiver_count() > 1 {
            self.write
                .send(ComponentWrite {
                    entity,
                    component,
                    data: Arc::new(data.clone()),
                })
                .map_err(|e| format!("Error sending write: {e}"))?;
        }

        // Deferred write to Bevy.
        self.commands
            .send(WasmCommand::WriteComponent {
                entity_id: entity,
                component_id: component,
                data,
                insert: false,
            })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;

        Ok(())
    }
    async fn remove_component(
        &mut self,
        entity: EntityId,
        component: ComponentId,
    ) -> Result<(), String> {
        self.commands
            .send(WasmCommand::RemoveComponent {
                entity_id: entity,
                component_id: component,
            })
            .await
            .map_err(|e| format!("Error sending command: {e}"))?;
        Ok(())
    }
}
