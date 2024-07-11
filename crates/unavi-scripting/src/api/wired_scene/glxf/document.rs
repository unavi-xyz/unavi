use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::wired::scene::glxf::{Asset, AssetBorrow, Host, HostGlxf},
    },
    state::StoreState,
};

use super::{
    asset_gltf::GltfAssetRes, asset_glxf::GlxfAssetRes, node::GlxfNodeRes, scene::GlxfSceneRes,
};

#[derive(Default)]
pub struct GlxfDocument {
    pub active_scene: Option<u32>,
    pub assets_gltf: Vec<u32>,
    pub assets_glxf: Vec<u32>,
    pub default_scene: Option<u32>,
    pub nodes: Vec<u32>,
    pub scenes: Vec<u32>,
    ref_count: RefCountCell,
}

impl RefCount for GlxfDocument {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for GlxfDocument {}

impl HostGlxf for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<GlxfDocument>> {
        let node = GlxfDocument::default();
        let table_res = self.table.push(node)?;
        let res = GlxfDocument::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn list_assets(&mut self, self_: Resource<GlxfDocument>) -> wasm_bridge::Result<Vec<Asset>> {
        let data = self.table.get(&self_)?;
        let assets_glxf = data
            .assets_glxf
            .iter()
            .map(|rep| -> wasm_bridge::Result<_> {
                Ok(Asset::Glxf(GlxfAssetRes::from_rep(*rep, &self.table)?))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let assets_gltf = data
            .assets_gltf
            .iter()
            .map(|rep| -> wasm_bridge::Result<_> {
                Ok(Asset::Gltf(GltfAssetRes::from_rep(*rep, &self.table)?))
            })
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
        let data = self.table.get_mut(&self_)?;

        match value {
            AssetBorrow::Gltf(res) => {
                data.assets_gltf.push(res.rep());
            }
            AssetBorrow::Glxf(res) => {
                data.assets_glxf.push(res.rep());
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
                data.assets_gltf = data
                    .assets_gltf
                    .iter()
                    .copied()
                    .filter(|rep| *rep != res.rep())
                    .collect();
            }
            AssetBorrow::Glxf(res) => {
                data.assets_glxf = data
                    .assets_glxf
                    .iter()
                    .copied()
                    .filter(|rep| *rep != res.rep())
                    .collect();
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
            .and_then(|rep| GlxfSceneRes::from_rep(rep, &self.table).ok()))
    }
    fn set_active_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Option<Resource<GlxfSceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.active_scene = value.map(|res| res.rep());
        Ok(())
    }

    fn default_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
    ) -> wasm_bridge::Result<Option<Resource<GlxfSceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .default_scene
            .and_then(|rep| GlxfSceneRes::from_rep(rep, &self.table).ok()))
    }
    fn set_default_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfSceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.default_scene = Some(value.rep());
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
            .map(|rep| GlxfNodeRes::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfNodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(value.rep());
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfNodeRes>,
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

    fn list_scenes(
        &mut self,
        self_: Resource<GlxfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<GlxfSceneRes>>> {
        let data = self.table.get(&self_)?;
        let scenes = data
            .scenes
            .iter()
            .map(|rep| GlxfSceneRes::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(scenes)
    }
    fn add_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfSceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.scenes.push(value.rep());
        Ok(())
    }
    fn remove_scene(
        &mut self,
        self_: Resource<GlxfDocument>,
        value: Resource<GlxfSceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.scenes = data
            .scenes
            .iter()
            .copied()
            .filter(|rep| *rep != value.rep())
            .collect();
        Ok(())
    }

    fn drop(&mut self, rep: Resource<GlxfDocument>) -> wasm_bridge::Result<()> {
        GlxfDocument::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}

impl Host for StoreState {
    fn get_root(&mut self) -> wasm_bridge::Result<Resource<GlxfDocument>> {
        todo!()
    }
}
