use wasm_bridge::component::Resource;

use crate::{
    actions::ScriptAction,
    api::{
        utils::{RefCount, RefCountCell, RefResource},
        wired_scene::wired::scene::scene::{Host, HostScene},
    },
    state::StoreState,
};

use super::node::Node;

#[derive(Default)]
pub struct Scene {
    pub nodes: Vec<u32>,
    ref_count: RefCountCell,
}

impl RefCount for Scene {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Scene {}

impl HostScene for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<Scene>> {
        let node = Scene::default();
        let table_res = self.table.push(node)?;
        let res = Scene::from_res(&table_res, &self.table)?;

        // self.sender
        //     .send(ScriptAction::CreateScene { id: res.rep() })?;

        Ok(res)
    }

    fn nodes(&mut self, self_: Resource<Scene>) -> wasm_bridge::Result<Vec<Resource<Node>>> {
        let data = self.table.get(&self_)?;
        let nodes = data
            .nodes
            .iter()
            .map(|rep| Node::from_rep(*rep, &self.table))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<Scene>,
        value: Resource<Node>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(value.rep());
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<Scene>,
        value: Resource<Node>,
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

    fn drop(&mut self, rep: Resource<Scene>) -> wasm_bridge::Result<()> {
        Scene::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}

impl Host for StoreState {}
