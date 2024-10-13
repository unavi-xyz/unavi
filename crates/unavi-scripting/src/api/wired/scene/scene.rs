use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        id::UniqueId,
        wired::scene::bindings::scene::{Host, HostScene},
    },
    data::ScriptData,
};

use super::nodes::base::NodeRes;

#[derive(Component, Clone, Copy, Debug)]
pub struct SceneId(pub u32);

#[derive(Bundle)]
pub struct GltfSceneBundle {
    pub id: SceneId,
    pub scene: SceneBundle,
}

impl GltfSceneBundle {
    pub fn new(id: u32) -> Self {
        Self {
            id: SceneId(id),
            scene: SceneBundle::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneRes(Arc<RwLock<SceneData>>);

#[derive(Default, Debug)]
pub struct SceneData {
    pub id: UniqueId,
    pub entity: OnceLock<Entity>,
    pub name: String,
    pub nodes: Vec<NodeRes>,
}

impl SceneRes {
    pub fn read(&self) -> RwLockReadGuard<SceneData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<SceneData> {
        self.0.write().unwrap()
    }
}

impl HostScene for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<SceneRes>> {
        let data = SceneRes(Arc::new(RwLock::new(SceneData::default())));
        let res = self.table.push(data.clone())?;

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let data = data.write();
                let entity = world.spawn(GltfSceneBundle::new(data.id.into())).id();
                data.entity.set(entity).unwrap();
            }))
            .unwrap();

        Ok(res)
    }

    fn id(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<u32> {
        Ok(self.table.get(&self_)?.read().id.into())
    }

    fn name(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.read();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<SceneRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.name = value;
        Ok(())
    }

    fn nodes(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        let nodes = self.table.get(&self_)?.read().nodes.clone();
        let nodes = nodes
            .into_iter()
            .map(|r| self.table.push(r))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<SceneRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?.clone();
        let node_data = self.table.get(&value)?.clone();

        data.write().nodes.push(node_data.clone());

        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let scene_ent = *data.read().entity.get().unwrap();
                let node_ent = *node_data.read().entity.get().unwrap();
                world.entity_mut(node_ent).set_parent(scene_ent);
            }))
            .unwrap();

        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<SceneRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?;
        let mut data_write = data.write();

        data_write
            .nodes
            .iter()
            .position(|r| r.read().id == data_write.id)
            .map(|index| data_write.nodes.remove(index));

        let node_data = self.table.get(&value)?.clone();
        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let node_ent = *node_data.read().entity.get().unwrap();
                world.entity_mut(node_ent).remove_parent();
            }))
            .unwrap();

        Ok(())
    }

    fn drop(&mut self, rep: Resource<SceneRes>) -> wasm_bridge::Result<()> {
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

        let res = HostScene::new(&mut data).unwrap();
        let res_weak = Resource::<SceneRes>::new_own(res.rep());

        HostScene::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }
}
