use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired::scene::bindings::glxf::{Asset, AssetBorrow, Host, HostGlxf},
    },
    data::ScriptData,
};

use super::{
    asset_gltf::GltfAssetRes, asset_glxf::GlxfAssetRes, node::GlxfNodeRes, scene::GlxfSceneRes,
};

#[derive(Default, Debug)]
pub struct GlxfDocument {
    pub active_scene: Option<Resource<GlxfSceneRes>>,
    pub assets_gltf: Vec<Resource<GltfAssetRes>>,
    pub assets_glxf: Vec<Resource<GlxfAssetRes>>,
    pub default_scene: Option<Resource<GlxfSceneRes>>,
    pub nodes: Vec<Resource<GlxfNodeRes>>,
    pub scenes: Vec<Resource<GlxfSceneRes>>,
    ref_count: RefCountCell,
}

impl RefCount for GlxfDocument {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for GlxfDocument {}

impl HostGlxf for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<GlxfDocument>> {
        let node = GlxfDocument::default();
        let table_res = self.table.push(node)?;
        let res = self.clone_res(&table_res)?;

        let documents = self.api.wired_scene.as_ref().unwrap().documents.clone();
        let rep = res.rep();
        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(SpatialBundle::default()).id();
            let mut documents = documents.write().unwrap();
            documents.insert(rep, entity);
        });

        Ok(res)
    }

    fn list_assets(&mut self, self_: Resource<GlxfDocument>) -> wasm_bridge::Result<Vec<Asset>> {
        let data = self.table.get(&self_)?;
        let assets_glxf = data
            .assets_glxf
            .iter()
            .map(|res| -> wasm_bridge::Result<_> { Ok(Asset::Glxf(self.clone_res(res)?)) })
            .collect::<Result<Vec<_>, _>>()?;
        let assets_gltf = data
            .assets_gltf
            .iter()
            .map(|res| -> wasm_bridge::Result<_> { Ok(Asset::Gltf(self.clone_res(res)?)) })
            .collect::<Result<Vec<_>, _>>()?;

        let mut assets = assets_glxf;
        assets.extend(assets_gltf);
        assets.sort_by(|a, b| {
            let rep_a = match a {
                Asset::Gltf(res) => res.rep(),
                Asset::Glxf(res) => res.rep(),
            };
            let rep_b = match b {
                Asset::Gltf(res) => res.rep(),
                Asset::Glxf(res) => res.rep(),
            };
            rep_a.cmp(&rep_b)
        });

        Ok(assets)
    }
    fn add_asset(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: AssetBorrow,
    ) -> wasm_bridge::Result<()> {
        match value {
            AssetBorrow::Gltf(res) => {
                let res = self.clone_res(&res)?;
                let data = self.table.get_mut(&self_)?;
                data.assets_gltf.push(res);
            }
            AssetBorrow::Glxf(res) => {
                let res = self.clone_res(&res)?;
                let data = self.table.get_mut(&self_)?;
                data.assets_glxf.push(res);
            }
        }

        Ok(())
    }
    fn remove_asset(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: AssetBorrow,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;

        match value {
            AssetBorrow::Gltf(res) => {
                data.assets_gltf
                    .iter()
                    .position(|r| r.rep() == res.rep())
                    .map(|index| data.assets_gltf.remove(index));
            }
            AssetBorrow::Glxf(res) => {
                data.assets_glxf
                    .iter()
                    .position(|r| r.rep() == res.rep())
                    .map(|index| data.assets_glxf.remove(index));
            }
        }

        Ok(())
    }

    fn active_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
    ) -> wasm_bridge::Result<Option<Resource<GlxfSceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .active_scene
            .as_ref()
            .and_then(|res| self.clone_res(res).ok()))
    }
    fn set_active_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Option<Resource<GlxfSceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let value = value.as_ref().and_then(|res| self.clone_res(res).ok());

        let data = self.table.get_mut(&self_)?;

        if let Some(prev) = &data.active_scene {
            let prev_rep = prev.rep();
            let glxf_scenes = self.api.wired_scene.as_ref().unwrap().glxf_scenes.clone();
            self.commands.push(move |world: &mut World| {
                let glxf_scenes = glxf_scenes.read().unwrap();
                let prev_ent = glxf_scenes.get(&prev_rep).unwrap();
                world.entity_mut(*prev_ent).remove_parent();
            });
        }

        data.active_scene = value;

        if let Some(active_scene) = &data.active_scene {
            let scene_rep = active_scene.rep();
            let documents = self.api.wired_scene.as_ref().unwrap().documents.clone();
            let glxf_scenes = self.api.wired_scene.as_ref().unwrap().glxf_scenes.clone();
            let root_rep = self_.rep();
            self.commands.push(move |world: &mut World| {
                let documents = documents.read().unwrap();
                let root_ent = documents.get(&root_rep).unwrap();

                let glxf_scenes = glxf_scenes.read().unwrap();
                let scene_ent = glxf_scenes.get(&scene_rep).unwrap();

                world.entity_mut(*scene_ent).set_parent(*root_ent);
            });
        }

        Ok(())
    }

    fn default_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
    ) -> wasm_bridge::Result<Option<Resource<GlxfSceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .default_scene
            .as_ref()
            .and_then(|res| self.clone_res(res).ok()))
    }
    fn set_default_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfSceneRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.default_scene = Some(res);
        Ok(())
    }

    fn list_nodes(
        &mut self,
        self_: Resource<GlxfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<GlxfNodeRes>>> {
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
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfNodeRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(res);
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfNodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.nodes.remove(index));
        Ok(())
    }

    fn list_scenes(
        &mut self,
        self_: Resource<GlxfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<GlxfSceneRes>>> {
        let data = self.table.get(&self_)?;
        let scenes = data
            .scenes
            .iter()
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(scenes)
    }
    fn add_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfSceneRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.scenes.push(res);
        Ok(())
    }
    fn remove_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfSceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.scenes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.scenes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<GlxfDocument>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = GlxfDocument::handle_drop(rep, &mut self.table)?;

        if dropped {
            let documents = self.api.wired_scene.as_ref().unwrap().documents.clone();
            self.commands.push(move |world: &mut World| {
                let mut nodes = documents.write().unwrap();
                let entity = nodes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for ScriptData {
    fn get_root(&mut self) -> wasm_bridge::Result<Resource<GlxfDocument>> {
        let res = self.clone_res(&self.root_glxf)?;
        Ok(res)
    }
}
