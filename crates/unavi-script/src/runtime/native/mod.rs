use wasmtime::component::{HasSelf, Linker};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxView, WasiView};

use crate::{
    permissions::{ApiName, ApiPermissions},
    runtime::Runtime,
};

pub mod wired;

pub struct NativeRuntime {
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

pub fn add_apis_to_linker(
    linker: &mut Linker<Runtime>,
    perms: &ApiPermissions,
) -> wasmtime::Result<()> {
    if perms.contains(&ApiName::Input) {
        wired::input::bindings::wired::input::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
        wired::input::bindings::wired::input::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    }

    if perms.contains(&ApiName::Scene) {
        wired::scene::bindings::wired::scene::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
        wired::scene::bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    }

    Ok(())
}
