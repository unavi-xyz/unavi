use std::sync::mpsc::SyncSender;

use wasm_bridge::component::ResourceTable;
use wasm_bridge_wasi::{WasiCtx, WasiView};

use super::commands::ScriptCommand;

pub struct ScriptState {
    pub sender: SyncSender<ScriptCommand>,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
}

impl WasiView for ScriptState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}
