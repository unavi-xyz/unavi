use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::wired::scene::scene::{Host, HostScene},
    },
    state::StoreState,
};

use super::node::NodeRes;

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

#[derive(Default)]
pub struct SceneRes {
    pub nodes: Vec<u32>,
    name: String,
    ref_count: RefCountCell,
}

impl RefCount for SceneRes {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for SceneRes {}

impl HostScene for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<SceneRes>> {
        let node = SceneRes::default();
        let table_res = self.table.push(node)?;
        let res = SceneRes::from_res(&table_res, &self.table)?;

        let rep = res.rep();
        let scenes = self.entities.scenes.clone();
        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(GltfSceneBundle::new(rep)).id();
            let mut scenes = scenes.write().unwrap();
            scenes.insert(rep, entity);
        });

        Ok(res)
    }

    fn id(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn name(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?;
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<SceneRes>, value: String) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.name = value;
        Ok(())
    }

    fn nodes(&mut self, self_: Resource<SceneRes>) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
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
        self_: Resource<SceneRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let scene_rep = self_.rep();
        let node_rep = value.rep();

        let data = self.table.get_mut(&self_)?;
        data.nodes.push(node_rep);

        let nodes = self.entities.nodes.clone();
        let scenes = self.entities.scenes.clone();
        self.commands.push(move |world: &mut World| {
            let scenes = scenes.read().unwrap();
            let scene_ent = scenes.get(&scene_rep).unwrap();

            let nodes = nodes.read().unwrap();
            let node_ent = nodes.get(&node_rep).unwrap();

            world.commands().entity(*node_ent).set_parent(*scene_ent);
        });

        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<SceneRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let node_rep = value.rep();

        let data = self.table.get_mut(&self_)?;
        data.nodes = data
            .nodes
            .iter()
            .copied()
            .filter(|rep| *rep != value.rep())
            .collect();

        let nodes = self.entities.nodes.clone();
        self.commands.push(move |world: &mut World| {
            let nodes = nodes.read().unwrap();
            let node_ent = nodes.get(&node_rep).unwrap();
            world.commands().entity(*node_ent).remove_parent();
        });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<SceneRes>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = SceneRes::handle_drop(rep, &mut self.table)?;

        if dropped {
            let scenes = self.entities.scenes.clone();
            self.commands.push(move |world: &mut World| {
                let mut scenes = scenes.write().unwrap();
                let entity = scenes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

impl Host for StoreState {}

#[cfg(test)]
mod tests {
    use crate::api::wired_scene::{gltf::node::NodeId, wired::scene::node::HostNode};

    use super::*;

    #[test]
    fn test_new() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let _ = HostScene::new(&mut state).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let found_scene_id = world.query::<&SceneId>().single(&world);
        assert_eq!(found_scene_id.0, 1);
    }

    #[test]
    fn test_add_node() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let scene = HostScene::new(&mut state).unwrap();
        let node = HostNode::new(&mut state).unwrap();
        HostScene::add_node(&mut state, scene, node).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let (node_ent, _) = world.query::<(Entity, &NodeId)>().single(&world);
        let (scene_children, _) = world.query::<(&Children, &SceneId)>().single(&world);
        assert!(scene_children.contains(&node_ent));
    }

    #[test]
    fn test_remove_node() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let scene = HostScene::new(&mut state).unwrap();
        let node = HostNode::new(&mut state).unwrap();
        HostScene::add_node(
            &mut state,
            Resource::new_own(scene.rep()),
            Resource::new_own(node.rep()),
        )
        .unwrap();
        HostScene::remove_node(&mut state, scene, node).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let children_query = world.query::<(&Children, &SceneId)>().get_single(&world);
        assert!(children_query.is_err());
    }

    #[test]
    fn test_nodes() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let scene = HostScene::new(&mut state).unwrap();
        let node = HostNode::new(&mut state).unwrap();
        let node_rep = node.rep();
        HostScene::add_node(
            &mut state,
            Resource::new_own(scene.rep()),
            Resource::new_own(node.rep()),
        )
        .unwrap();

        let nodes = HostScene::nodes(&mut state, Resource::new_own(scene.rep())).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].rep(), node_rep);

        HostScene::remove_node(&mut state, Resource::new_own(scene.rep()), node).unwrap();

        let nodes = HostScene::nodes(&mut state, Resource::new_own(scene.rep())).unwrap();
        assert!(nodes.is_empty());
    }
}
