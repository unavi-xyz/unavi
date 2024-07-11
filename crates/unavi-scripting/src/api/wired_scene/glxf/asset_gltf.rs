use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::{
            gltf::{document::GltfDocument, node::NodeRes},
            wired::scene::glxf::HostAssetGltf,
        },
    },
    state::StoreState,
};

pub struct GltfAssetRes {
    document: u32,
    nodes: Vec<u32>,
    ref_count: RefCountCell,
}

impl RefCount for GltfAssetRes {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for GltfAssetRes {}

impl HostAssetGltf for StoreState {
    fn new(
        &mut self,
        document: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Resource<GltfAssetRes>> {
        let data = GltfAssetRes {
            document: document.rep(),
            nodes: Vec::default(),
            ref_count: RefCountCell::default(),
        };
        let table_res = self.table.push(data)?;
        let res = GltfAssetRes::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn document(
        &mut self,
        self_: Resource<GltfAssetRes>,
    ) -> wasm_bridge::Result<Resource<GltfDocument>> {
        let data = self.table.get(&self_)?;
        let res = GltfDocument::from_rep(data.document, &self.table)?;
        Ok(res)
    }

    fn list_nodes(
        &mut self,
        self_: Resource<GltfAssetRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        let data = self.table.get(&self_)?;
        let nodes = data
            .nodes
            .iter()
            .map(|rep| NodeRes::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<GltfAssetRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(value.rep());
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GltfAssetRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes = data
            .nodes
            .iter()
            .copied()
            .filter(|rep| *rep != value.rep())
            .collect();
        Ok(())
    }

    fn drop(&mut self, rep: Resource<GltfAssetRes>) -> wasm_bridge::Result<()> {
        GltfAssetRes::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}
