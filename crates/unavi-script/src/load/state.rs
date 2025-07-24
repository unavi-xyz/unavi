use wasmtime::component::ResourceTable;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiView};

use crate::api::wired::ecs::WiredEcsData;

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub data: RuntimeData,
}

#[derive(Default)]
pub struct RuntimeData {
    pub wired_ecs: WiredEcsData,
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

impl StoreState {
    pub fn new(wasi: WasiCtx) -> Self {
        Self {
            wasi,
            resource_table: ResourceTable::default(),
            data: RuntimeData::default(),
        }
    }
}
