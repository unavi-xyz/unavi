use std::sync::mpsc::SyncSender;

use wasm_bridge::component::ResourceTable;
use wasm_bridge_wasi::{WasiCtx, WasiView};

pub struct ScriptState {
    pub sender: SyncSender<ScriptCommand>,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
}

pub enum ScriptCommand {
    Query(Vec<ComponentId>),
    RegisterComponent(Component),
    SpawnEntity(SpawnEntity),
}

pub type ComponentId = u32;
pub type EntityId = u32;
pub type InstanceId = u32;

pub struct Component {
    pub id: ComponentId,
    pub instance: InstanceId,
}

pub struct SpawnEntity {
    pub components: Vec<Component>,
    pub id: EntityId,
}

impl WasiView for ScriptState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
