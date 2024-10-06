use std::sync::{Arc, OnceLock, RwLock};

use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::{
    bindings::document::{Host, HostDocument},
    material::MaterialRes,
    mesh::MeshRes,
    nodes::base::NodeRes,
    scene::SceneRes,
};

#[derive(Debug, Clone)]
pub struct DocumentRes(pub Arc<RwLock<DocumentData>>);

#[derive(Default, Debug)]
pub struct DocumentData {
    pub active_scene: Option<SceneRes>,
    pub default_scene: Option<SceneRes>,
    pub materials: Vec<MaterialRes>,
    pub meshes: Vec<MeshRes>,
    pub nodes: Vec<NodeRes>,
    pub scenes: Vec<SceneRes>,
    pub entity: OnceLock<Entity>,
}

impl HostDocument for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<DocumentRes>> {
        let data = DocumentRes(Arc::new(RwLock::new(DocumentData::default())));
        let res = self.table.push(data.clone())?;

        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(SpatialBundle::default()).id();
            data.0.write().unwrap().entity.set(entity).unwrap();
        });

        Ok(res)
    }

    fn active_scene(
        &mut self,
        self_: Resource<DocumentRes>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self
            .table
            .get(&self_)?
            .0
            .read()
            .unwrap()
            .active_scene
            .clone();
        let res = match data {
            Some(d) => Some(self.table.push(d)?),
            None => None,
        };
        Ok(res)
    }
    fn set_active_scene(
        &mut self,
        self_: Resource<DocumentRes>,
        value: Option<Resource<SceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();
        let mut data_write = data.0.write().unwrap();

        if let Some(prev) = data_write.active_scene.clone() {
            self.commands.push(move |world: &mut World| {
                let prev_ent = *prev.0.read().unwrap().entity.get().unwrap();
                world.entity_mut(prev_ent).remove_parent();
            });
        }

        let scene_data = match value {
            Some(r) => Some(self.table.get(&r)?.clone()),
            None => None,
        };

        data_write.active_scene = scene_data.clone();
        drop(data_write);

        if let Some(scene_data) = scene_data {
            self.commands.push(move |world: &mut World| {
                let document_ent = *data.0.read().unwrap().entity.get().unwrap();
                let scene_ent = *scene_data.0.read().unwrap().entity.get().unwrap();

                world.entity_mut(scene_ent).set_parent(document_ent);
            });
        }

        Ok(())
    }

    fn default_scene(
        &mut self,
        self_: Resource<DocumentRes>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self
            .table
            .get(&self_)?
            .0
            .read()
            .unwrap()
            .default_scene
            .clone();
        let res = match data {
            Some(d) => Some(self.table.push(d)?),
            None => None,
        };
        Ok(res)
    }
    fn set_default_scene(
        &mut self,
        self_: Resource<DocumentRes>,
        value: Option<Resource<SceneRes>>,
    ) -> wasm_bridge::Result<()> {
        let mut data_write = self.table.get(&self_)?.0.write().unwrap();

        if let Some(value) = &value {
            let scene_data = self.table.get(value)?.clone();
            data_write.default_scene = Some(scene_data);
        } else {
            data_write.default_scene = None;
        }
        Ok(())
    }

    fn scenes(
        &mut self,
        self_: Resource<DocumentRes>,
    ) -> wasm_bridge::Result<Vec<Resource<SceneRes>>> {
        let scenes = self.table.get(&self_)?.0.read().unwrap().scenes.clone();
        let res = scenes
            .into_iter()
            .map(|r| self.table.push(r))
            .collect::<Result<_, _>>()?;
        Ok(res)
    }
    fn add_scene(
        &mut self,
        self_: Resource<DocumentRes>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let mut data_write = self.table.get(&self_)?.0.write().unwrap();
        let scene_data = self.table.get(&value)?.clone();
        data_write.scenes.push(scene_data);
        Ok(())
    }
    fn remove_scene(
        &mut self,
        self_: Resource<DocumentRes>,
        value: Resource<SceneRes>,
    ) -> wasm_bridge::Result<()> {
        let mut data_write = self.table.get(&self_)?.0.write().unwrap();
        let scene_data = self.table.get(&value)?.0.read().unwrap();

        data_write
            .scenes
            .iter()
            .position(|r| r.0.read().unwrap().id == scene_data.id)
            .map(|index| data_write.scenes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<DocumentRes>) -> wasm_bridge::Result<()> {
        // if dropped {
        //     let documents = self
        //         .api
        //         .wired_scene
        //         .as_ref()
        //         .unwrap()
        //         .entities
        //         .documents
        //         .clone();
        //
        //     self.commands.push(move |world: &mut World| {
        //         let mut nodes = documents.write().unwrap();
        //         let entity = nodes.remove(&id).unwrap();
        //         world.despawn(entity);
        //     });
        // }

        Ok(())
    }
}

impl Host for ScriptData {}
