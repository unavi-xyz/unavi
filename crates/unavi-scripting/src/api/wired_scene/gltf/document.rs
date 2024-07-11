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
    pub active_scene: Option<Resource<SceneRes>>,
    pub default_scene: Option<Resource<SceneRes>>,
    pub materials: Vec<Resource<MaterialRes>>,
    pub meshes: Vec<Resource<MeshRes>>,
    pub nodes: Vec<Resource<NodeRes>>,
    pub scenes: Vec<Resource<SceneRes>>,
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
        let res = self.clone_res(&table_res)?;

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
            .as_ref()
            .and_then(|res| self.clone_res(res).ok()))
    }
    fn set_active_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Option<Resource<SceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let res = value.and_then(|v| self.clone_res(&v).ok());

        let data = self.table.get_mut(&self_)?;

        if let Some(prev) = &data.active_scene {
            let prev_rep = prev.rep();
            let scenes = self.entities.scenes.clone();
            self.commands.push(move |world: &mut World| {
                let scenes = scenes.read().unwrap();
                let prev_ent = scenes.get(&prev_rep).unwrap();
                world.entity_mut(*prev_ent).remove_parent();
            });
        }

        data.active_scene = res;

        if let Some(active_scene) = &data.active_scene {
            let documents = self.entities.documents.clone();
            let root_rep = self_.rep();
            let scene_rep = active_scene.rep();
            let scenes = self.entities.scenes.clone();
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
            .as_ref()
            .and_then(|res| self.clone_res(res).ok()))
    }
    fn set_default_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.default_scene = Some(res);
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
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(materials)
    }
    fn add_material(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MaterialRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.materials.push(res);
        Ok(())
    }
    fn remove_material(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MaterialRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.materials
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.materials.remove(index));
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
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(meshes)
    }
    fn add_mesh(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MeshRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.meshes.push(res);
        Ok(())
    }
    fn remove_mesh(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<MeshRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.meshes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.meshes.remove(index));
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
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(res);
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<NodeRes>,
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
        self_: Resource<GltfDocument>,
    ) -> wasm_bridge::Result<Vec<Resource<SceneRes>>> {
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
        self_: Resource<GltfDocument>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.scenes.push(res);
        Ok(())
    }
    fn remove_scene(
        &mut self,
        self_: Resource<GltfDocument>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.scenes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.scenes.remove(index));
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
