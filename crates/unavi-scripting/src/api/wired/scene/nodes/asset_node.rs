use wasm_bridge::component::Resource;

use crate::{
    api::{
        utils::RefResource,
        wired::{
            math::bindings::types::Transform,
            scene::bindings::composition::{Asset, HostAssetNode},
        },
    },
    data::ScriptData,
};

use super::base::NodeRes;

impl HostAssetNode for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<wasm_bridge::component::Resource<NodeRes>> {
        NodeRes::new_res(self)
    }

    fn name(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?;
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<NodeRes>, value: String) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.name = value;
        Ok(())
    }

    fn asset(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Option<Asset>> {
        let data = self.table.get(&self_)?;

        let asset = match &data.asset {
            Some(v) => Some(match v {
                Asset::Composition(v) => Asset::Composition(self.clone_res(v)?),
                Asset::Document(v) => Asset::Document(self.clone_res(v)?),
            }),
            None => None,
        };

        Ok(asset)
    }
    fn set_asset(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Asset>,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.asset = value;
        Ok(())
    }

    fn global_transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        NodeRes::global_transform(self, &self_)
    }
    fn transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        let node = self.table.get(&self_)?;
        Ok(node.transform.into())
    }
    fn set_transform(
        &mut self,
        self_: Resource<NodeRes>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        NodeRes::set_transform(self, &self_, value)
    }

    fn parent(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Option<Resource<NodeRes>>> {
        NodeRes::parent(self, &self_)
    }
    fn children(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        NodeRes::children(self, &self_)
    }
    fn add_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::add_child(self, &self_, &value)
    }
    fn remove_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::remove_child(self, &self_, &value)
    }

    fn drop(&mut self, rep: Resource<NodeRes>) -> wasm_bridge::Result<()> {
        NodeRes::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}
