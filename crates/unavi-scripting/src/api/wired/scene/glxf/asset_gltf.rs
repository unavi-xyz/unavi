use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired::scene::{
            bindings::wired::scene::glxf::HostAssetGltf,
            gltf::{document::GltfDocument, node::NodeRes},
        },
    },
    data::ScriptData,
};

#[derive(Debug)]
pub struct GltfAssetRes {
    document: Resource<GltfDocument>,
    nodes: Vec<Resource<NodeRes>>,
    ref_count: RefCountCell,
}

impl RefCount for GltfAssetRes {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for GltfAssetRes {}

impl HostAssetGltf for ScriptData {
    fn new(
        &mut self,
        document: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Resource<GltfAssetRes>> {
        let data = GltfAssetRes {
            document: self.clone_res(&document)?,
            nodes: Vec::default(),
            ref_count: RefCountCell::default(),
        };
        let table_res = self.table.push(data)?;
        let res = self.clone_res(&table_res)?;

        let assets = self.api.wired_scene.as_ref().unwrap().assets.clone();
        let documents = self.api.wired_scene.as_ref().unwrap().documents.clone();
        let rep = res.rep();
        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(SpatialBundle::default()).id();
            let mut assets = assets.write().unwrap();
            assets.insert(rep, entity);

            let documents = documents.read().unwrap();
            let doc_ent = documents.get(&document.rep()).unwrap();
            world.entity_mut(*doc_ent).set_parent(entity);
        });

        Ok(res)
    }

    fn document(
        &mut self,
        self_: Resource<GltfAssetRes>,
    ) -> wasm_bridge::Result<Resource<GltfDocument>> {
        let data = self.table.get(&self_)?;
        let res = self.clone_res(&data.document)?;
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
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<GltfAssetRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(res);
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GltfAssetRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.nodes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<GltfAssetRes>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = GltfAssetRes::handle_drop(rep, &mut self.table)?;

        if dropped {
            let assets = self.api.wired_scene.as_ref().unwrap().assets.clone();
            self.commands.push(move |world: &mut World| {
                let mut assets = assets.write().unwrap();
                let entity = assets.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}
