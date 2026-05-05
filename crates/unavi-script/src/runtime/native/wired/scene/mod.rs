use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::scene::{document::DocRes, node::NodeRes},
    },
};

pub mod document;
pub mod material;
pub mod mesh;
pub mod node;
mod types;

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::{
        document::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.document": DocRes,
            "wired:scene/types.material": MaterialRes,
            "wired:scene/types.mesh":     MeshRes,
            "wired:scene/types.node":     NodeRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::scene::api::Host for Runtime {
    async fn self_node(&mut self) -> wasmtime::Result<Resource<NodeRes>> {
        shared::wired::scene::self_node(&self.api)
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn self_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::self_document(&self.api)
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
        shared::wired::scene::remove_document(&self.api, id).map_err(wasmtime::Error::from_anyhow)
    }

    async fn load_hsd(
        &mut self,
        blob_id: Vec<u8>,
    ) -> wasmtime::Result<Result<Resource<DocRes>, String>> {
        Ok(shared::wired::scene::load_hsd(&self.api, blob_id)
            .await
            .map(Resource::new_own)
            .map_err(|e| e.to_string()))
    }
}

impl bindings::wired::scene::types::Host for Runtime {}
