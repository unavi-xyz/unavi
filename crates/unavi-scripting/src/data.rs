use bevy::{ecs::world::CommandQueue, prelude::*};
use wasm_bridge::component::{Resource, ResourceTable, ResourceTableError};
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::api::{utils::RefResource, ApiData};

pub struct ScriptData {
    pub api: ApiData,
    pub commands: CommandQueue,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
    pub wasi_table: wasm_bridge_wasi::ResourceTable,
}

impl WasiView for ScriptData {
    fn table(&mut self) -> &mut wasm_bridge_wasi::ResourceTable {
        &mut self.wasi_table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl Default for ScriptData {
    fn default() -> Self {
        Self {
            api: ApiData::default(),
            commands: CommandQueue::default(),
            table: ResourceTable::default(),
            wasi: WasiCtxBuilder::new().build(),
            wasi_table: wasm_bridge_wasi::ResourceTable::default(),
        }
    }
}

impl ScriptData {
    pub fn clone_res<T: RefResource>(
        &self,
        res: &Resource<T>,
    ) -> Result<Resource<T>, ResourceTableError> {
        T::from_res(res, &self.table)
    }

    /// Inserts a component into the given node if the value is `Some`.
    /// If the value is `None`, removes the component from the entity.
    pub fn node_insert_option<T: Bundle>(&mut self, node: u32, value: Option<T>) {
        let nodes = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .nodes
            .clone();

        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let entity = nodes.get(&node).unwrap();
            let mut entity = world.entity_mut(*entity);

            if let Some(value) = value {
                entity.insert(value);
            } else {
                entity.remove::<T>();
            }
        });
    }

    /// Inserts a component into the given node.
    pub fn node_insert<T: Bundle>(&mut self, node: u32, value: T) {
        self.node_insert_option(node, Some(value))
    }

    /// Removes a component from the given node.
    pub fn node_remove<T: Bundle>(&mut self, node: u32) {
        self.node_insert_option::<T>(node, None)
    }
}
