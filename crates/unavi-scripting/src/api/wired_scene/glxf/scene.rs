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
    pub nodes: Vec<u32>,
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
        todo!()
    }
    fn set_name(
        &mut self,
        self_: Resource<GlxfSceneRes>,
        value: String,
    ) -> wasm_bridge::Result<()> {
        todo!()
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

            world.commands().entity(*node_ent).set_parent(*scene_ent);
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

            world
                .commands()
                .entity(*scene_ent)
                .remove_children(&[*node_ent]);
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
