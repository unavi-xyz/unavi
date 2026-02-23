use std::sync::Arc;

use loro::LoroDoc;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::api::wired::scene::WiredSceneRt;

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub rt: RuntimeData,
}

pub struct RuntimeData {
    wired_scene: WiredSceneRt,
}

pub struct RuntimeDataResult {
    pub rt: RuntimeData,
}

impl RuntimeData {
    pub fn spawn() -> RuntimeDataResult {
        let doc = Arc::new(LoroDoc::new()); // TODO

        RuntimeDataResult {
            rt: Self {
                wired_scene: WiredSceneRt {
                    table: ResourceTable::default(),
                    doc,
                },
            },
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
