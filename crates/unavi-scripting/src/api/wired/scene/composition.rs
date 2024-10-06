use std::sync::{Arc, OnceLock, RwLock};

use bevy::prelude::*;
use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::{
    bindings::composition::{Host, HostComposition},
    nodes::base::NodeRes,
};

#[derive(Debug, Clone)]
pub struct CompositionRes(Arc<RwLock<CompositionData>>);

#[derive(Default, Debug)]
pub struct CompositionData {
    pub entity: OnceLock<Entity>,
    pub nodes: Vec<NodeRes>,
}

impl CompositionRes {
    pub fn new(data: &mut ScriptData) -> Self {
        let composition = CompositionRes(Arc::new(RwLock::new(CompositionData::default())));

        {
            let composition = composition.clone();
            data.commands.push(move |world: &mut World| {
                let entity = world.spawn(SpatialBundle::default()).id();
                composition.0.write().unwrap().entity.set(entity).unwrap();
            });
        }

        composition
    }
}

impl HostComposition for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<CompositionRes>> {
        let data = CompositionRes::new(self);
        let res = self.table.push(data)?;
        Ok(res)
    }

    fn nodes(
        &mut self,
        self_: Resource<CompositionRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        let nodes = self.table.get(&self_)?.0.read().unwrap().nodes.clone();
        Ok(nodes
            .into_iter()
            .map(|d| self.table.push(d))
            .collect::<Result<_, _>>()?)
    }
    fn add_node(
        &mut self,
        self_: Resource<CompositionRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let value = self.table.get(&value)?.clone();
        let data = self.table.get(&self_)?;
        data.0.write().unwrap().nodes.push(value);
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<CompositionRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let id = self.table.get(&value)?.0.read().unwrap().id;
        let mut data = self.table.get(&self_)?.0.write().unwrap();
        data.nodes
            .iter()
            .position(|r| r.0.read().unwrap().id == id)
            .map(|index| data.nodes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<CompositionRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for ScriptData {}
