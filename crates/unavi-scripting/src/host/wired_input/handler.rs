use wasm_bridge::component::Resource;

use crate::state::StoreState;

use super::wired::input::{
    handler::{HostSpatialHandler, Node},
    types::InputEvent,
};

pub struct SpatialHandler;

impl HostSpatialHandler for StoreState {
    fn new(&mut self, node: Resource<Node>) -> wasm_bridge::Result<Resource<SpatialHandler>> {
        let handler = SpatialHandler;
        let res = self.table.push_child(handler, &node)?;
        let rep = res.rep();

        let node = self.table.get_mut(&node)?;
        node.handlers.push(res);

        Ok(Resource::new_own(rep))
    }

    fn handle_input(
        &mut self,
        _self_: Resource<SpatialHandler>,
    ) -> wasm_bridge::Result<Option<InputEvent>> {
        Ok(None)
    }

    fn drop(&mut self, _rep: Resource<SpatialHandler>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
