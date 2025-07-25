use tokio::sync::mpsc::Receiver;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiView};

use crate::{api::wired::ecs::WiredEcsData, commands::WasmCommand};

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub rt: RuntimeData,
}

pub struct RuntimeData {
    pub wired_ecs: WiredEcsData,
}

impl RuntimeData {
    pub fn spawn() -> RuntimeDataResult {
        let (commands_send, commands_recv) = tokio::sync::mpsc::channel(MAX_COMMANDS);

        RuntimeDataResult {
            rt: RuntimeData {
                wired_ecs: WiredEcsData {
                    initialized: false,
                    components: Default::default(),
                    systems: Default::default(),
                    schedules: Default::default(),
                    commands: commands_send,
                },
            },
            commands_recv,
        }
    }
}

impl IoView for StoreState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

impl WasiView for StoreState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

const MAX_COMMANDS: usize = 200;

impl StoreState {
    pub fn new(wasi: WasiCtx, rt: RuntimeData) -> StoreState {
        Self {
            wasi,
            resource_table: ResourceTable::default(),
            rt,
        }
    }
}

pub struct RuntimeDataResult {
    pub rt: RuntimeData,
    pub commands_recv: Receiver<WasmCommand>,
}
