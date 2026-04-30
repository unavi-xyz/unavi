use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::runtime::Runtime;

pub mod api;

pub struct NativeStoreState {
    pub table: ResourceTable,
    pub wasi_ctx: WasiCtx,
}

impl WasiView for Runtime {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.native.wasi_ctx,
            table: &mut self.native.table,
        }
    }
}
