use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::wired::scene::gltf::{Host, HostGltf},
    },
    state::StoreState,
};

use super::{material::MaterialRes, mesh::MeshRes, node::NodeRes, scene::SceneRes};

#[derive(Default)]
pub struct GltfDocument {
    pub active_scene: Option<u32>,
    pub default_scene: Option<u32>,
    pub materials: Vec<u32>,
    pub meshes: Vec<u32>,
    pub nodes: Vec<u32>,
    pub scenes: Vec<u32>,
    ref_count: RefCountCell,
}

impl RefCount for GltfDocument {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for GltfDocument {}

impl HostGltf for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<GltfDocument>> {
        let node = GltfDocument::default();
        let table_res = self.table.push(node)?;
        let res = GltfDocument::from_res(&table_res, &self.table)?;

        let documents = self.entities.documents.clone();
        let rep = res.rep();
        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(SpatialBundle::default()).id();
            let mut documents = documents.write().unwrap();
            documents.insert(rep, entity);
        });

        Ok(res)
    }

    fn active_scene(
        &mut self,
        self_: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .active_scene
            .and_then(|rep| SceneRes::from_rep(rep, &self.table).ok()))
    }
    fn set_active_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Option<Resource<SceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;

        if let Some(prev) = data.active_scene {
            let scenes = self.entities.scenes.clone();
            self.commands.push(move |world: &mut World| {
                let scenes = scenes.read().unwrap();
                let prev_ent = scenes.get(&prev).unwrap();
                world.entity_mut(*prev_ent).remove_parent();
            });
        }

        data.active_scene = value.map(|res| res.rep());

        if let Some(scene_rep) = data.active_scene {
            let documents = self.entities.documents.clone();
            let scenes = self.entities.scenes.clone();
            let root_rep = self_.rep();
            self.commands.push(move |world: &mut World| {
                let documents = documents.read().unwrap();
                let root_ent = documents.get(&root_rep).unwrap();

                let scenes = scenes.read().unwrap();
                let scene_ent = scenes.get(&scene_rep).unwrap();

                world.entity_mut(*scene_ent).set_parent(*root_ent);
            });
        }

        Ok(())
    }

    fn default_scene(
        &mut self,
        self_: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .default_scene
            .and_then(|rep| SceneRes::from_rep(rep, &self.table).ok()))
    }
    fn set_default_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.default_scene = Some(value.rep());
        Ok(())
    }

    fn list_materials(
        &mut self,
        self_: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<MaterialRes>>> {
        let data = self.table.get(&self_)?;
        let materials = data
            .materials
            .iter()
            .map(|rep| MaterialRes::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(materials)
    }
    fn add_material(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MaterialRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.materials.push(value.rep());
        Ok(())
    }
    fn remove_material(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MaterialRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.materials = data
            .materials
            .iter()
            .copied()
            .filter(|rep| *rep != value.rep())
            .collect();
        Ok(())
    }

    fn list_meshes(
        &mut self,
        self_: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<MeshRes>>> {
        let data = self.table.get(&self_)?;
        let meshes = data
            .meshes
            .iter()
            .map(|rep| MeshRes::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(meshes)
    }
    fn add_mesh(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MeshRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.meshes.push(value.rep());
        Ok(())
    }
    fn remove_mesh(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MeshRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.meshes = data
            .meshes
            .iter()
            .copied()
            .filter(|rep| *rep != value.rep())
            .collect();
        Ok(())
    }

    fn list_nodes(
        &mut self,
        self_: Resource<GltfDocument>,
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
        self_: Resource<GltfDocument>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(value.rep());
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GltfDocument>,
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

    fn list_scenes(
        &mut self,
        self_: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?;
        let scenes = data
            .scenes
            .iter()
            .map(|rep| SceneRes::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(scenes)
    }
    fn add_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.scenes.push(value.rep());
        Ok(())
    }
    fn remove_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<SceneRes>,
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

    fn drop(&mut self, rep: Resource<GltfDocument>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = GltfDocument::handle_drop(rep, &mut self.table)?;

        if dropped {
            let documents = self.entities.documents.clone();
            self.commands.push(move |world: &mut World| {
                let mut nodes = documents.write().unwrap();
                let entity = nodes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for StoreState {}
