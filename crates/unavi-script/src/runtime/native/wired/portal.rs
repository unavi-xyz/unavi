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
    async fn open_portal(
        &mut self,
        prim: Resource<PrimRes>,
        space: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let space: [u8; 32] = space
            .as_slice()
            .try_into()
            .map_err(|_| wasmtime::Error::msg("space id must be 32 bytes"))?;
        shared::wired::portal::open_portal(&self.api, prim.rep(), space)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
