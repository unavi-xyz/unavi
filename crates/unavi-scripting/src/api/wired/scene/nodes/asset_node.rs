use wasm_bridge::component::Resource;

use crate::{
    api::wired::{
        math::bindings::types::Transform,
        scene::bindings::composition::{Asset, HostAssetNode},
    },
    data::ScriptData,
};

use super::base::{AssetData, NodeRes};

impl HostAssetNode for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<wasm_bridge::component::Resource<NodeRes>> {
        NodeRes::new_res(self)
    }

    fn name(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<String> {
        let data = self.table.get(&self_)?.read();
        Ok(data.name.clone())
    }
    fn set_name(&mut self, self_: Resource<NodeRes>, value: String) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.name = value;
        Ok(())
    }

    fn asset(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Option<Asset>> {
        let asset = self.table.get(&self_)?.read().asset.clone();

        let res = match asset {
            Some(v) => Some(match v {
                AssetData::Composition(v) => Asset::Composition(self.table.push(v)?),
                AssetData::Document(v) => Asset::Document(self.table.push(v)?),
            }),
            None => None,
        };

        Ok(res)
    }
    fn set_asset(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Asset>,
    ) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.asset = match value {
            Some(v) => Some(match v {
                Asset::Composition(r) => AssetData::Composition(self.table.get(&r)?.clone()),
                Asset::Document(r) => AssetData::Document(self.table.get(&r)?.clone()),
            }),
            None => None,
        };
        Ok(())
    }

    fn global_transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        NodeRes::global_transform(self, &self_)
    }
    fn transform(&mut self, self_: Resource<NodeRes>) -> wasm_bridge::Result<Transform> {
        let data = self.table.get(&self_)?.read();
        Ok(data.transform.into())
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
        NodeRes::parent(self, self.table.get(&self_)?.clone())
    }
    fn children(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasm_bridge::Result<Vec<Resource<NodeRes>>> {
        NodeRes::children(self, self.table.get(&self_)?.clone())
    }
    fn add_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::add_child(
            self,
            self.table.get(&self_)?.clone(),
            self.table.get(&value)?.clone(),
        );
        Ok(())
    }
    fn remove_child(
        &mut self,
        self_: Resource<NodeRes>,
        value: Resource<NodeRes>,
    ) -> wasm_bridge::Result<()> {
        NodeRes::remove_child(
            self,
            self.table.get(&self_)?.clone(),
            self.table.get(&value)?.clone(),
        );
        Ok(())
    }

    fn drop(&mut self, rep: Resource<NodeRes>) -> wasm_bridge::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::tests::init_test_data;

    use super::*;

    #[test]
    fn test_cleanup_resource() {
        let (_, mut data) = init_test_data();

        let res = HostAssetNode::new(&mut data).unwrap();
        let res_weak = Resource::<NodeRes>::new_own(res.rep());

        HostAssetNode::drop(&mut data, res).unwrap();
        assert!(data.table.get(&res_weak).is_err());
    }
}
