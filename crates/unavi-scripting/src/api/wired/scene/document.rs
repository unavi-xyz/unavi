use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
pub struct DocumentRes(Arc<RwLock<DocumentData>>);

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

impl DocumentRes {
    pub fn read(&self) -> RwLockReadGuard<DocumentData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<DocumentData> {
        self.0.write().unwrap()
    }
}

impl HostDocument for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<DocumentRes>> {
        let data = DocumentRes(Arc::new(RwLock::new(DocumentData::default())));
        let res = self.table.push(data.clone())?;

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let entity = world.spawn(SpatialBundle::default()).id();
                data.write().entity.set(entity).unwrap();
            }))
            .unwrap();

        Ok(res)
    }

    fn active_scene(
        &mut self,
        self_: Resource<DocumentRes>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?.read().active_scene.clone();
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
        let mut data_write = data.write();

        if let Some(prev) = data_write.active_scene.clone() {
            self.command_send
                .try_send(Box::new(move |world: &mut World| {
                    let prev_ent = *prev.read().entity.get().unwrap();
                    world.entity_mut(prev_ent).remove_parent();
                }))
                .unwrap();
        }

        let scene_data = match value {
            Some(r) => Some(self.table.get(&r)?.clone()),
            None => None,
        };

        data_write.active_scene = scene_data.clone();
        drop(data_write);

        if let Some(scene_data) = scene_data {
            self.command_send
                .try_send(Box::new(move |world: &mut World| {
                    let document_ent = *data.read().entity.get().unwrap();
                    let scene_ent = *scene_data.read().entity.get().unwrap();

                    world.entity_mut(scene_ent).set_parent(document_ent);
                }))
                .unwrap();
        }

        Ok(())
    }

    fn default_scene(
        &mut self,
        self_: Resource<DocumentRes>,
    ) -> wasm_bridge::Result<Option<Resource<SceneRes>>> {
        let data = self.table.get(&self_)?.read().default_scene.clone();
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
        let mut data_write = self.table.get(&self_)?.write();
        let scene_data = self.table.get(&value)?.read();

        data_write
            .scenes
            .iter()
            .position(|r| r.read().id == scene_data.id)
            .map(|index| data_write.scenes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<DocumentRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl Host for ScriptData {}

#[cfg(test)]
mod tests {
    use crate::api::tests::init_test_data;

    use super::*;

    #[test]
    fn test_cleanup_resource() {
        let (_, mut data) = init_test_data();

        let res = HostDocument::new(&mut data).unwrap();
        let res_weak = Resource::<DocumentRes>::new_own(res.rep());

        HostDocument::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }
}
