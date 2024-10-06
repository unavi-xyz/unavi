use std::sync::{Arc, OnceLock, RwLock};

use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        id::ResourceId,
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
pub struct SceneRes(pub Arc<RwLock<SceneData>>);

#[derive(Default, Debug)]
pub struct SceneData {
    pub id: ResourceId,
    pub entity: OnceLock<Entity>,
    pub name: String,
    pub nodes: Vec<NodeRes>,
}

impl HostScene for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<SceneRes>> {
        let data = SceneRes(Arc::new(RwLock::new(SceneData::default())));
        let res = self.table.push(data.clone())?;

        self.commands.push(move |world: &mut World| {
            let data = data.0.write().unwrap();
            let entity = world.spawn(GltfSceneBundle::new(data.id.into())).id();
            data.entity.set(entity).unwrap();
        });

        Ok(res)
    }

    fn id(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<u32> {
        Ok(self.table.get(&self_)?.0.read().unwrap().id.into())
    }

    fn name(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.0.read().unwrap();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<SceneRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.0.write().unwrap();
        data.name = value;
        Ok(())
    }

    fn nodes(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        let nodes = self.table.get(&self_)?.0.read().unwrap().nodes.clone();
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

        data.0.write().unwrap().nodes.push(node_data.clone());

        self.commands.push(move |world: &mut World| {
            let scene_ent = *data.0.read().unwrap().entity.get().unwrap();
            let node_ent = *node_data.0.read().unwrap().entity.get().unwrap();
            world.entity_mut(node_ent).set_parent(scene_ent);
        });

        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<SceneRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get(&self_)?;
        let mut data_write = data.0.write().unwrap();

        data_write
            .nodes
            .iter()
            .position(|r| r.0.read().unwrap().id == data_write.id)
            .map(|index| data_write.nodes.remove(index));

        let node_data = self.table.get(&value)?.clone();
        self.commands.push(move |world: &mut World| {
            let node_ent = *node_data.0.read().unwrap().entity.get().unwrap();
            world.entity_mut(node_ent).remove_parent();
        });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<SceneRes>) -> wasm_bridge::Result<()> {
        // if dropped {
        //     let scenes = self
        //         .api
        //         .wired_scene
        //         .as_ref()
        //         .unwrap()
        //         .entities
        //         .scenes
        //         .clone();
        //
        //     self.commands.push(move |world: &mut World| {
        //         let mut scenes = scenes.write().unwrap();
        //         let entity = scenes.remove(&id).unwrap();
        //         world.despawn(entity);
        //     });
        // }

        Ok(())
    }
}

impl Host for ScriptData {}
