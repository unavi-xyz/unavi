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

#[derive(Default)]
pub struct SceneRes {
    pub nodes: Vec<u32>,
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

        // self.sender
        //     .send(ScriptAction::CreateScene { id: res.rep() })?;

        Ok(res)
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
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(value.rep());
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<SceneRes>,
        value: Resource<NodeRes>,
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

    fn drop(&mut self, rep: Resource<SceneRes>) -> wasm_bridge::Result<()> {
        SceneRes::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}

impl Host for StoreState {}
