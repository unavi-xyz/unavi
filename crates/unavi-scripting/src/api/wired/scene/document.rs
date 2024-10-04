use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    data::ScriptData,
};

use super::{
    bindings::document::{Host, HostDocument},
    material::MaterialRes,
    mesh::MeshRes,
    nodes::base::NodeRes,
    scene::SceneRes,
};

#[derive(Default, Debug)]
pub struct Document {
    pub active_scene: Option<Resource<SceneRes>>,
    pub default_scene: Option<Resource<SceneRes>>,
    pub materials: Vec<Resource<MaterialRes>>,
    pub meshes: Vec<Resource<MeshRes>>,
    pub nodes: Vec<Resource<NodeRes>>,
    pub scenes: Vec<Resource<SceneRes>>,
    ref_count: RefCountCell,
}

impl RefCount for Document {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Document {}

impl HostDocument for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<Document>> {
        let node = Document::default();
        let table_res = self.table.push(node)?;
        let res = self.clone_res(&table_res)?;
        let rep = res.rep();

        let documents = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .documents
            .clone();

        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(SpatialBundle::default()).id();
            let mut documents = documents.write().unwrap();
            documents.insert(rep, entity);
        });

        Ok(res)
    }

    fn active_scene(
        &mut self,
        self_: Resource<Document>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .active_scene
            .as_ref()
            .and_then(|res| self.clone_res(res).ok()))
    }
    fn set_active_scene(
        &mut self,
        self_: Resource<Document>,
        value: Option<Resource<SceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let res = value.and_then(|v| self.clone_res(&v).ok());

        let data = self.table.get_mut(&self_)?;

        if let Some(prev) = &data.active_scene {
            let prev_rep = prev.rep();
            let scenes = self
                .api
                .wired_scene
                .as_ref()
                .unwrap()
                .entities
                .scenes
                .clone();
            self.commands.push(move |world: &mut World| {
                let scenes = scenes.read().unwrap();
                let prev_ent = scenes.get(&prev_rep).unwrap();
                world.entity_mut(*prev_ent).remove_parent();
            });
        }

        data.active_scene = res;

        if let Some(active_scene) = &data.active_scene {
            let documents = self
                .api
                .wired_scene
                .as_ref()
                .unwrap()
                .entities
                .documents
                .clone();
            let scenes = self
                .api
                .wired_scene
                .as_ref()
                .unwrap()
                .entities
                .scenes
                .clone();
            let root_rep = self_.rep();
            let scene_rep = active_scene.rep();

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
        self_: Resource<Document>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?;
        Ok(data
            .default_scene
            .as_ref()
            .and_then(|res| self.clone_res(res).ok()))
    }
    fn set_default_scene(
        &mut self,
        self_: Resource<Document>,
        value: Option<Resource<SceneRes>>,
    ) -> wasm_bridge::Result<()> {
        if let Some(value) = value {
            let res = self.clone_res(&value)?;
            let data = self.table.get_mut(&self_)?;
            data.default_scene = Some(res);
        } else {
            let data = self.table.get_mut(&self_)?;
            data.default_scene = None;
        }
        Ok(())
    }

    fn scenes(
        &mut self,
        self_: Resource<Document>,
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
        self_: Resource<Document>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.scenes.push(res);
        Ok(())
    }
    fn remove_scene(
        &mut self,
        self_: Resource<Document>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.scenes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.scenes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<Document>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = Document::handle_drop(rep, &mut self.table)?;

        if dropped {
            let documents = self
                .api
                .wired_scene
                .as_ref()
                .unwrap()
                .entities
                .documents
                .clone();

            self.commands.push(move |world: &mut World| {
                let mut nodes = documents.write().unwrap();
                let entity = nodes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for ScriptData {}
