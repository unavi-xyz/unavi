use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::wired::scene::glxf::HostGlxfScene,
    },
    state::StoreState,
};

use super::node::GlxfNodeRes;

#[derive(Component, Clone, Copy, Debug)]
pub struct GlxfSceneId(pub u32);

#[derive(Bundle)]
pub struct GlxfSceneBundle {
    pub id: GlxfSceneId,
    pub scene: SceneBundle,
}

impl GlxfSceneBundle {
    pub fn new(id: u32) -> Self {
        Self {
            id: GlxfSceneId(id),
            scene: SceneBundle::default(),
        }
    }
}

#[derive(Default)]
pub struct GlxfSceneRes {
    name: String,
    nodes: Vec<u32>,
    ref_count: RefCountCell,
}

impl RefCount for GlxfSceneRes {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for GlxfSceneRes {}

impl HostGlxfScene for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<GlxfSceneRes>> {
        let node = GlxfSceneRes::default();
        let table_res = self.table.push(node)?;
        let res = GlxfSceneRes::from_res(&table_res, &self.table)?;

        let rep = res.rep();
        let glxf_scenes = self.entities.glxf_scenes.clone();
        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(GlxfSceneBundle::new(rep)).id();
            let mut scenes = glxf_scenes.write().unwrap();
            scenes.insert(rep, entity);
        });

        Ok(res)
    }

    fn id(&mut self, self_: Resource<GlxfSceneRes>) -> wasm_bridge::Result<u32> {
        Ok(self_.rep())
    }

    fn name(&mut self, self_: Resource<GlxfSceneRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?;
        Ok(data.name.clone())
    }
    fn set_name(
        &mut self,
        self_: Resource<GlxfSceneRes>,
        value: String,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.name = value;
        Ok(())
    }

    fn nodes(
        &mut self,
        self_: Resource<GlxfSceneRes>,
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
        self_: Resource<GlxfSceneRes>,
        value: Resource<GlxfNodeRes>,
    ) -> wasm_bridge::Result<()> {
        let scene_rep = self_.rep();
        let node_rep = value.rep();

        let data = self.table.get_mut(&self_)?;
        data.nodes.push(node_rep);

        let glxf_nodes = self.entities.glxf_nodes.clone();
        let glxf_scenes = self.entities.glxf_scenes.clone();
        self.commands.push(move |world: &mut World| {
            let scenes = glxf_scenes.read().unwrap();
            let scene_ent = scenes.get(&scene_rep).unwrap();

            let nodes = glxf_nodes.read().unwrap();
            let node_ent = nodes.get(&node_rep).unwrap();

            world.entity_mut(*node_ent).set_parent(*scene_ent);
        });

        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<GlxfSceneRes>,
        value: Resource<GlxfNodeRes>,
    ) -> wasm_bridge::Result<()> {
        let scene_rep = self_.rep();
        let node_rep = value.rep();

        let data = self.table.get_mut(&self_)?;
        data.nodes = data
            .nodes
            .iter()
            .copied()
            .filter(|rep| *rep != value.rep())
            .collect();

        let glxf_nodes = self.entities.glxf_nodes.clone();
        let glxf_scenes = self.entities.glxf_scenes.clone();
        self.commands.push(move |world: &mut World| {
            let scenes = glxf_scenes.read().unwrap();
            let scene_ent = scenes.get(&scene_rep).unwrap();

            let nodes = glxf_nodes.read().unwrap();
            let node_ent = nodes.get(&node_rep).unwrap();

            world.entity_mut(*scene_ent).remove_children(&[*node_ent]);
        });

        Ok(())
    }

    fn drop(&mut self, rep: Resource<GlxfSceneRes>) -> wasm_bridge::Result<()> {
        let id = rep.rep();
        let dropped = GlxfSceneRes::handle_drop(rep, &mut self.table)?;

        if dropped {
            let glxf_scenes = self.entities.glxf_scenes.clone();
            self.commands.push(move |world: &mut World| {
                let mut scenes = glxf_scenes.write().unwrap();
                let entity = scenes.remove(&id).unwrap();
                world.despawn(entity);
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::wired_scene::{glxf::node::GlxfNodeId, wired::scene::glxf::HostGlxfNode};

    use super::*;

    #[test]
    fn test_new() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let _ = HostGlxfScene::new(&mut state).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let (found_id, _) = world
            .query::<(&GlxfSceneId, &Handle<Scene>)>()
            .single(&world);
        assert_eq!(found_id.0, 1);
    }

    #[test]
    fn test_add_node() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let scene = HostGlxfScene::new(&mut state).unwrap();
        let node = HostGlxfNode::new(&mut state).unwrap();
        HostGlxfScene::add_node(&mut state, scene, node).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let (node_ent, _) = world.query::<(Entity, &GlxfNodeId)>().single(&world);
        let (scene_children, _) = world.query::<(&Children, &GlxfSceneId)>().single(&world);
        assert!(scene_children.contains(&node_ent));
    }

    #[test]
    fn test_remove_node() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let scene = HostGlxfScene::new(&mut state).unwrap();
        let node = HostGlxfNode::new(&mut state).unwrap();
        HostGlxfScene::add_node(
            &mut state,
            Resource::new_own(scene.rep()),
            Resource::new_own(node.rep()),
        )
        .unwrap();
        HostGlxfScene::remove_node(&mut state, scene, node).unwrap();

        world.commands().append(&mut state.commands);
        world.flush_commands();

        let children_query = world
            .query::<(&Children, &GlxfSceneId)>()
            .get_single(&world);
        assert!(children_query.is_err());
    }

    #[test]
    fn test_nodes() {
        let mut world = World::new();
        let root_ent = world.spawn_empty().id();
        let mut state = StoreState::new("test".to_string(), root_ent);

        let scene = HostGlxfScene::new(&mut state).unwrap();
        let node = HostGlxfNode::new(&mut state).unwrap();
        let node_rep = node.rep();
        HostGlxfScene::add_node(
            &mut state,
            Resource::new_own(scene.rep()),
            Resource::new_own(node.rep()),
        )
        .unwrap();

        let nodes = HostGlxfScene::nodes(&mut state, Resource::new_own(scene.rep())).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].rep(), node_rep);

        HostGlxfScene::remove_node(&mut state, Resource::new_own(scene.rep()), node).unwrap();

        let nodes = HostGlxfScene::nodes(&mut state, Resource::new_own(scene.rep())).unwrap();
        assert!(nodes.is_empty());
    }
}
