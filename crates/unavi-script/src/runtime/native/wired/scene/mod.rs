use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::scene::{document::DocRes, prim::PrimRes},
    },
};

pub mod document;
pub mod prim;
mod types;

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::{document::DocRes, prim::PrimRes};

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.document": DocRes,
            "wired:scene/types.prim":     PrimRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::scene::api::Host for Runtime {
    async fn self_prim(&mut self) -> wasmtime::Result<Resource<PrimRes>> {
        shared::wired::scene::self_prim(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn self_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::self_document(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get_document(&mut self, id: Vec<u8>) -> wasmtime::Result<Option<Resource<DocRes>>> {
        shared::wired::scene::get_document(&self.api, id)
            .await
            .map(|r| r.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::create_document(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_document(&mut self, id: Vec<u8>) -> wasmtime::Result<()> {
        shared::wired::scene::remove_document(&self.api, id)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn load_hsd(
        &mut self,
        blob: Vec<u8>,
    ) -> wasmtime::Result<Result<Resource<DocRes>, String>> {
        Ok(shared::wired::scene::load_hsd(&self.api, blob)
            .await
            .map(Resource::new_own)
            .map_err(|e| e.to_string()))
    }
}

impl bindings::wired::scene::types::Host for Runtime {}
