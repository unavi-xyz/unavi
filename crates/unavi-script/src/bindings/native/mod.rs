//! Lowers the host world onto wasmtime: one generated binding, and impls that
//! only convert values and call into [`crate::host`].

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
    error::ScriptError,
    host::ScriptHost,
    quota::limiter::QuotaLimiter,
};

mod agent;
mod convert;
mod event;
mod input;
mod peer;
mod physics;
mod portal;
mod scene;
mod shading;
mod storage;

pub mod generated {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "unavi:host/shell",
        // Only the calls that wait on the world are async; the rest answer
        // from the script's own state at once.
        imports: {
            "wired:scene/document.open-document": async | trappable,
            "wired:scene/document.create-document": async | trappable,
            "wired:scene/document.copy-document": async | trappable,
            "wired:scene/document.delete-document": async | trappable,
            "wired:scene/document.[method]document.create-prim": async | trappable,
            "wired:scene/document.[method]document.apply": async | trappable,
            "wired:scene/document.[method]document.commit": async | trappable,
            "wired:scene/document.[method]document.place": async | trappable,
            "wired:shading/graph.set-graph": async | trappable,
            "wired:shading/graph.set-overrides": async | trappable,
            "wired:physics/simulation.raycast": async | trappable,
            "wired:physics/simulation.velocity": async | trappable,
            "wired:physics/simulation.set-velocity": async | trappable,
            "wired:physics/simulation.set-force": async | trappable,
            "wired:agent/local.attach": async | trappable,
            "wired:portal/portals.open": async | trappable,
            "wired:portal/portals.pair": async | trappable,
            "wired:portal/portals.travel": async | trappable,
            "unavi:host/node-storage.get": async | trappable,
            "unavi:host/node-storage.list-entries": async | trappable,
            default: trappable,
        },
        exports: { default: async },
        trappable_error_type: {
            "wired:core/error.error" => crate::error::ScriptError,
        },
        with: {
            "wired:scene/document.document": crate::host::scene::DocumentRes,
            "wired:event/messaging.message-subscription": crate::host::event::MessageSubscription,
            "wired:input/types.input-subscription": crate::host::input::InputSubscription,
            "wired:portal/portals.intent-subscription": crate::host::portal::IntentSubscription,
            "unavi:host/node-storage.pending-value": crate::host::storage::PendingValue,
            "unavi:host/node-storage.pending-entries": crate::host::storage::PendingEntries,
        },
    });
}

pub use generated::{
    Shell,
    ShellPre,
};

/// A store's data: the script's host context and what WASI needs.
pub struct HostCtx {
    pub host:    ScriptHost,
    pub limiter: QuotaLimiter,
    /// Epochs the current call has run for, reset each tick.
    pub epochs:  u32,
    table:       ResourceTable,
    wasi:        WasiCtx,
}

impl HostCtx {
    #[must_use]
    pub fn new(host: ScriptHost, limiter: QuotaLimiter, wasi: WasiCtx) -> Self {
        Self {
            host,
            limiter,
            epochs: 0,
            table: ResourceTable::default(),
            wasi,
        }
    }
}

impl WasiView for HostCtx {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx:   &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// A linker holding WASI and every host interface.
pub fn linker(engine: &wasmtime::Engine) -> wasmtime::Result<Linker<HostCtx>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    Shell::add_to_linker::<_, HasSelf<_>>(&mut linker, |ctx| ctx)?;
    Ok(linker)
}

impl generated::wired::core::error::Host for HostCtx {
    fn convert_error(
        &mut self,
        err: ScriptError,
    ) -> wasmtime::Result<generated::wired::core::error::Error> {
        convert::error(err)
    }
}

impl generated::wired::core::math::Host for HostCtx {}
impl generated::wired::core::ids::Host for HostCtx {}

impl generated::wired::script::host::Host for HostCtx {
    fn granted(
        &mut self,
        permission: generated::wired::core::error::Permission,
    ) -> wasmtime::Result<bool> {
        Ok(self.host.granted(convert::permission(permission)))
    }
}
