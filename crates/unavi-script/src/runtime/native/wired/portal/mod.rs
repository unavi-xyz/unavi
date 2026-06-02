use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::scene::prim::PrimRes,
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::prim::PrimRes;

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-portal",
        with: {
            "wired:scene/types.prim": PrimRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::portal::api::Host for Runtime {
    async fn open(
        &mut self,
        prim: Resource<PrimRes>,
        target_space: Vec<u8>,
    ) -> wasmtime::Result<()> {
        shared::wired::portal::open(&self.api, prim.rep(), target_space)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
