use wasmtime::component::{HasSelf, Linker, Resource};

use crate::runtime::{
    Runtime,
    shared::wired::scene::{doc::DocRes, node::NodeRes},
};

pub mod document;
pub mod material;
pub mod mesh;
pub mod node;

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::{doc::DocRes, node::NodeRes};

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.document": DocRes,
            "wired:scene/types.node":     NodeRes,
            "wired:scene/types.mesh":     super::mesh::MeshRes,
            "wired:scene/types.material": super::material::MaterialRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

pub fn add_to_linker(linker: &mut Linker<Runtime>) -> wasmtime::Result<()> {
    bindings::wired::scene::api::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |r| r)?;
    Ok(())
}

impl bindings::wired::scene::api::Host for Runtime {
    async fn self_node(&mut self) -> wasmtime::Result<Resource<NodeRes>> {
        let rep = self.backend.wired_scene.lock().await.self_node();
        Ok(Resource::new_own(rep))
    }

    async fn self_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        let rep = self.backend.wired_scene.lock().await.self_document();
        Ok(Resource::new_own(rep))
    }

    async fn get_document(&mut self, id: Vec<u8>) -> wasmtime::Result<Option<Resource<DocRes>>> {
        let rep = self.backend.wired_scene.lock().await.get_document(id);
        Ok(rep.map(Resource::new_own))
    }

    async fn create_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .create_document()
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(rep))
    }

    async fn remove_document(&mut self, id: Vec<u8>) -> wasmtime::Result<()> {
        self.backend.wired_scene.lock().await.remove_document(id);
        Ok(())
    }

    async fn load_hsd(
        &mut self,
        _blob_id: Vec<u8>,
    ) -> wasmtime::Result<Result<Resource<DocRes>, String>> {
        todo!()
    }
}

impl bindings::wired::scene::types::Host for Runtime {}
