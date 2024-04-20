use std::sync::{mpsc::SyncSender, Arc};

use tokio::sync::Mutex;
use wasm_bridge_wasi::{WasiCtx, WasiView};

use super::{commands::ScriptCommand, resource_table::ResourceTable};

pub struct ScriptState {
    pub sender: Arc<Mutex<SyncSender<ScriptCommand>>>,
    pub table: Arc<Mutex<ResourceTable>>,
    pub wasi_table: wasm_bridge_wasi::ResourceTable,
    pub wasi_ctx: WasiCtx,
}

impl WasiView for ScriptState {
    fn table(&mut self) -> &mut wasm_bridge_wasi::ResourceTable {
        &mut self.wasi_table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}
