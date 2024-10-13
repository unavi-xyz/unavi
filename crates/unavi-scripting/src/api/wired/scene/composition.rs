use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
    pub fn read(&self) -> RwLockReadGuard<CompositionData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<CompositionData> {
        self.0.write().unwrap()
    }

    pub fn new(data: &mut ScriptData) -> Self {
        let composition = CompositionRes(Arc::new(RwLock::new(CompositionData::default())));

        {
            let composition = composition.clone();
            data.command_send
                .try_send(Box::new(move |world: &mut World| {
                    let entity = world.spawn(SpatialBundle::default()).id();
                    composition.write().entity.set(entity).unwrap();
                }))
                .unwrap();
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
        let nodes = self.table.get(&self_)?.read().nodes.clone();
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
        data.write().nodes.push(value);
        Ok(())
    }
    fn remove_node(
        &mut self,
        self_: Resource<CompositionRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        let id = self.table.get(&value)?.read().id;
        let mut data = self.table.get(&self_)?.write();
        data.nodes
            .iter()
            .position(|r| r.read().id == id)
            .map(|index| data.nodes.remove(index));
        Ok(())
    }

    fn drop(&mut self, rep: Resource<CompositionRes>) -> wasm_bridge::Result<()> {
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

        let res = HostComposition::new(&mut data).unwrap();
        let res_weak = Resource::<CompositionRes>::new_own(res.rep());

        HostComposition::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }
}
