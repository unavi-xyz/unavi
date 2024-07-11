use std::sync::{Arc, RwLock};

use bevy::{ecs::world::CommandQueue, prelude::*, utils::HashMap};
use wasm_bridge::component::{Resource, ResourceTable};
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::api::wired_scene::glxf::document::GlxfDocument;

pub struct StoreState {
    pub commands: CommandQueue,
    pub entities: EntityMaps,
    pub root_glxf: Resource<GlxfDocument>,
    pub name: String,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
    pub wasi_table: wasm_bridge_wasi::ResourceTable,
}

impl WasiView for StoreState {
    fn table(&mut self) -> &mut wasm_bridge_wasi::ResourceTable {
        &mut self.wasi_table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl StoreState {
    pub fn new(name: String) -> Self {
        let mut table = ResourceTable::default();
        let root_glxf = table.push(GlxfDocument::default()).unwrap();

        Self {
            commands: CommandQueue::default(),
            entities: EntityMaps::default(),
            name,
            root_glxf,
            table,
            wasi: WasiCtxBuilder::new().build(),
            wasi_table: wasm_bridge_wasi::ResourceTable::default(),
        }
    }

    /// Inserts a component into the given node.
    pub fn node_insert<T: Component>(&mut self, node: u32, value: T) {
        let nodes = self.entities.nodes.clone();

        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let entity = nodes.get(&node).unwrap();
            let mut entity = world.entity_mut(*entity);
            entity.insert(value);
        });
    }

    /// Inserts a component into the given node if the value is `Some`.
    /// If the value is `None`, removes the component from the entity.
    pub fn node_insert_option<T: Component>(&mut self, node: u32, value: Option<T>) {
        let nodes = self.entities.nodes.clone();

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
}

#[derive(Default)]
pub struct EntityMaps {
    pub glxf_nodes: Arc<RwLock<HashMap<u32, Entity>>>,
    pub glxf_scenes: Arc<RwLock<HashMap<u32, Entity>>>,
    pub materials: Arc<RwLock<HashMap<u32, MaterialState>>>,
    pub nodes: Arc<RwLock<HashMap<u32, Entity>>>,
    pub primitives: Arc<RwLock<HashMap<u32, PrimitiveState>>>,
    pub scenes: Arc<RwLock<HashMap<u32, Entity>>>,
}

pub struct MaterialState {
    pub entity: Entity,
    pub handle: Handle<StandardMaterial>,
}

pub struct PrimitiveState {
    pub entity: Entity,
    pub handle: Handle<Mesh>,
}
