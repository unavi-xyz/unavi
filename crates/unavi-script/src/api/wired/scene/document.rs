use loro::{LoroMap, LoroValue, TreeParentId};
use wasmtime::component::Resource;

use super::bindings::wired::scene::types::{Document, Material, Mesh};
use crate::api::wired::scene::{
    WiredSceneRt, material::HostMaterial, mesh::HostMesh, node::HostNode,
};

pub struct HostDocument;

impl super::bindings::wired::scene::types::HostDocument for WiredSceneRt {
    async fn create_material(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Material>> {
        let list = self.material_list()?;
        let index = list.len();
        let map: LoroMap = list
            .push_container(LoroMap::new())
            .map_err(|e| anyhow::anyhow!("push material: {e}"))?;
        map.insert("base_color", LoroValue::Null)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(self.table.push(HostMaterial { index })?)
    }

    async fn create_mesh(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<Mesh>> {
        let list = self.mesh_list()?;
        let index = list.len();
        let map: LoroMap = list
            .push_container(LoroMap::new())
            .map_err(|e| anyhow::anyhow!("push mesh: {e}"))?;
        map.insert("topology", 3i64)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(self.table.push(HostMesh { index })?)
    }

    async fn create_node(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Resource<HostNode>> {
        let tree_id = self
            .node_tree()?
            .create(TreeParentId::Root)
            .map_err(|e| anyhow::anyhow!("create node: {e}"))?;
        Ok(self.table.push(HostNode { tree_id })?)
    }

    async fn roots(
        &mut self,
        _self_: Resource<Document>,
    ) -> wasmtime::Result<Vec<Resource<HostNode>>> {
        let tree = self.node_tree()?;
        let root_ids = tree.children(TreeParentId::Root).unwrap_or_default();
        let mut out = Vec::with_capacity(root_ids.len());
        for tree_id in root_ids {
            out.push(self.table.push(HostNode { tree_id })?);
        }
        Ok(out)
    }

    async fn drop(&mut self, rep: Resource<Document>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
