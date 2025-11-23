use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::{
    api::wired::ecs::{ComponentWrite, WiredEcsData},
    commands::WasmCommand,
};

const MAX_COMMANDS: usize = 256;
const MAX_WRITE: usize = 1024;

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub rt: RuntimeData,
}

pub struct RuntimeData {
    pub wired_ecs: WiredEcsData,
}

pub struct RuntimeDataResult {
    pub rt: RuntimeData,
    pub commands_recv: tokio::sync::mpsc::Receiver<WasmCommand>,
    pub write_recv: tokio::sync::broadcast::Receiver<ComponentWrite>,
}

impl RuntimeData {
    pub fn spawn() -> RuntimeDataResult {
        let (commands_send, commands_recv) = tokio::sync::mpsc::channel(MAX_COMMANDS);
        let (write_send, write_recv) = tokio::sync::broadcast::channel(MAX_WRITE);

        RuntimeDataResult {
            rt: Self {
                wired_ecs: WiredEcsData {
                    commands: commands_send,
                    write: write_send,
                    components: Vec::new(),
                    entity_id: 0,
                    systems: Vec::new(),
                },
            },
            commands_recv,
            write_recv,
        }
    }
}

impl WasiView for StoreState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.resource_table,
        }
    }
}

impl StoreState {
    pub fn new(wasi: WasiCtx, rt: RuntimeData) -> Self {
        Self {
            wasi,
            resource_table: ResourceTable::default(),
            rt,
        }
    }
}
