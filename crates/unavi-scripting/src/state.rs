use std::sync::{Arc, RwLock};

use bevy::{ecs::world::CommandQueue, prelude::*, utils::HashMap};
use wasm_bridge::component::{Resource, ResourceTable, ResourceTableError};
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::api::{utils::RefResource, wired_scene::glxf::document::GlxfDocument};

pub struct StoreState {
    pub commands: CommandQueue,
    pub entities: EntityMaps,
    pub name: String,
    pub root_glxf: Resource<GlxfDocument>,
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
    pub fn new(name: String, root_ent: Entity) -> Self {
        let mut table = ResourceTable::default();
        let table_res = table.push(GlxfDocument::default()).unwrap();
        let root_glxf = GlxfDocument::from_res(&table_res, &table).unwrap();

        let entities = EntityMaps::default();
        let mut commands = CommandQueue::default();

        let documents = entities.documents.clone();
        let rep = root_glxf.rep();
        commands.push(move |world: &mut World| {
            world.commands().entity(root_ent).with_children(|parent| {
                let entity = parent.spawn(SpatialBundle::default()).id();
                let mut documents = documents.write().unwrap();
                documents.insert(rep, entity);
            });
        });

        Self {
            commands,
            entities,
            name,
            root_glxf,
            table,
            wasi: WasiCtxBuilder::new().build(),
            wasi_table: wasm_bridge_wasi::ResourceTable::default(),
        }
    }

    pub fn clone_res<T: RefResource>(
        &self,
        res: &Resource<T>,
    ) -> Result<Resource<T>, ResourceTableError> {
        T::from_res(res, &self.table)
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
    pub assets: Arc<RwLock<HashMap<u32, Entity>>>,
    pub documents: Arc<RwLock<HashMap<u32, Entity>>>,
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
    pub handle: Handle<Mesh>,
}
