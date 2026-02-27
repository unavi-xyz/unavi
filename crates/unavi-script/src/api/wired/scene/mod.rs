//! # `wired:scene` API
//!
//! Each script is attached to an HSD document.
//! These APIs interact directly with that underlying Loro document.
//!
//! This allows full Read / Write support outside of the Bevy ECS.

use std::sync::Arc;

use loro::{LoroDoc, LoroList, LoroMap, LoroTree, TreeID};
use wasmtime_wasi::ResourceTable;

mod document;
mod material;
mod mesh;
mod node;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.node": super::node::HostNode,
            "wired:scene/types.material": super::material::HostMaterial,
            "wired:scene/types.mesh": super::mesh::HostMesh,
            "wired:scene/types.document": super::document::HostDocument,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

pub struct WiredSceneRt {
    pub table: ResourceTable,
    pub doc: Arc<LoroDoc>,
}

impl WiredSceneRt {
    fn hsd_map(&self) -> LoroMap {
        self.doc.get_map("hsd")
    }

    fn node_tree(&self) -> wasmtime::Result<LoroTree> {
        self.hsd_map()
            .get_or_create_container("nodes", LoroTree::new())
            .map_err(|e| anyhow::anyhow!("node tree: {e}"))
    }

    fn mesh_list(&self) -> wasmtime::Result<LoroList> {
        self.hsd_map()
            .get_or_create_container("meshes", LoroList::new())
            .map_err(|e| anyhow::anyhow!("mesh list: {e}"))
    }

    fn material_list(&self) -> wasmtime::Result<LoroList> {
        self.hsd_map()
            .get_or_create_container("materials", LoroList::new())
            .map_err(|e| anyhow::anyhow!("material list: {e}"))
    }

    fn mesh_map(&self, index: usize) -> wasmtime::Result<LoroMap> {
        let list = self.mesh_list()?;
        match list.get(index) {
            Some(loro::ValueOrContainer::Container(loro::Container::Map(m))) => Ok(m),
            _ => Err(anyhow::anyhow!("mesh {index} not found")),
        }
    }

    fn material_map(&self, index: usize) -> wasmtime::Result<LoroMap> {
        let list = self.material_list()?;
        match list.get(index) {
            Some(loro::ValueOrContainer::Container(loro::Container::Map(m))) => Ok(m),
            _ => Err(anyhow::anyhow!("material {index} not found")),
        }
    }

    fn node_meta(&self, id: TreeID) -> wasmtime::Result<LoroMap> {
        self.node_tree()?
            .get_meta(id)
            .map_err(|e| anyhow::anyhow!("node meta: {e}"))
    }
}

impl bindings::wired::scene::types::Host for WiredSceneRt {}
