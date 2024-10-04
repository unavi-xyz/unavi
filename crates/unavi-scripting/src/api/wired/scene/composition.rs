use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    data::ScriptData,
};

use super::{
    bindings::composition::{Host, HostComposition},
    nodes::base::NodeRes,
};

#[derive(Default)]
pub struct Composition {
    pub nodes: Vec<Resource<NodeRes>>,
    ref_count: RefCountCell,
}

impl RefCount for Composition {
    fn ref_count(&self) -> &std::cell::Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Composition {}

impl HostComposition for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<Composition>> {
        let node = Composition::default();
        let table_res = self.table.push(node)?;
        let res = self.clone_res(&table_res)?;
        let rep = res.rep();

        let compositions = self
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .compositions
            .clone();

        self.commands.push(move |world: &mut World| {
            let entity = world.spawn(SpatialBundle::default()).id();
            let mut compositions = compositions.write().unwrap();
            compositions.insert(rep, entity);
        });

        Ok(res)
    }

    fn nodes(
        &mut self,
        self_: Resource<Composition>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        let data = self.table.get(&self_)?;
        let nodes = data
            .nodes
            .iter()
            .map(|res| self.clone_res(res))
            .collect::<Result<_, _>>()?;
        Ok(nodes)
    }
    fn add_node(
        &mut self,
        self_: Resource<Composition>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let res = self.clone_res(&value)?;
        let data = self.table.get_mut(&self_)?;
        data.nodes.push(res);
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<Composition>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.nodes
            .iter()
            .position(|r| r.rep() == value.rep())
            .map(|index| data.nodes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<Composition>) -> wasm_bridge::Result<()> {
        Composition::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}

impl Host for ScriptData {}
