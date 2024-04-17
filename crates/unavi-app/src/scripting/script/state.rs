use std::sync::mpsc::SyncSender;

use wasm_bridge::component::ResourceTable;
use wasm_bridge_wasi::{WasiCtx, WasiView};

pub struct ScriptState {
    pub send_command: SyncSender<ScriptCommand>,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
}

pub enum ScriptCommand {
    CreateMesh,
}

impl WasiView for ScriptState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
