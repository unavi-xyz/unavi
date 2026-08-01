use wasmtime::component::{
    HasSelf,
    Linker,
};
use wasmtime_wasi::{
    ResourceTable,
    WasiCtx,
    WasiCtxView,
    WasiView,
};

use crate::{
    quota::limiter::QuotaLimiter,
    runtime::Runtime,
};

pub mod wired;

pub struct NativeRuntime {
    pub table:    ResourceTable,
    pub wasi_ctx: WasiCtx,
    pub limiter:  QuotaLimiter,
}

impl WasiView for Runtime {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx:   &mut self.native.wasi_ctx,
            table: &mut self.native.table,
        }
    }
}

pub fn add_apis_to_linker(linker: &mut Linker<Runtime>) -> wasmtime::Result<()> {
    wired::agent::bindings::wired::agent::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::agent::bindings::wired::agent::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::event::bindings::wired::event::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::event::bindings::wired::event::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::input::bindings::wired::input::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::input::bindings::wired::input::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::input::bindings::wired::input::context::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::scene::bindings::wired::scene::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::scene::bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::wds::bindings::wired::wds::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::wds::bindings::wired::wds::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::kv::bindings::wired::kv::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::kv::bindings::wired::kv::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::peer::bindings::wired::peer::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::peer::bindings::wired::peer::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::portal::bindings::wired::portal::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    wired::physics::bindings::wired::physics::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    wired::physics::bindings::wired::physics::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;

    Ok(())
}
