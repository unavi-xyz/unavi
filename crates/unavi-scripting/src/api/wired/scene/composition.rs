use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::bindings::{
    composition::{Asset, Host, HostAssetNode, HostComposition},
    node::Transform,
};

pub struct Composition {}

impl HostComposition for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<Composition>> {
        todo!();
    }

    fn nodes(
        &mut self,
        self_: Resource<Composition>,
    ) -> wasm_bridge::Result<Vec<Resource<AssetNode>>> {
        todo!();
    }
    fn add_node(
        &mut self,
        self_: Resource<Composition>,
        value: Resource<AssetNode>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }
    fn remove_node(
        &mut self,
        self_: Resource<Composition>,
        value: Resource<AssetNode>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn drop(&mut self, rep: Resource<Composition>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

pub struct AssetNode {}

impl HostAssetNode for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<AssetNode>> {
        todo!();
    }

    fn name(&mut self, self_: Resource<AssetNode>) -> wasm_bridge::Result<String> {
        todo!();
    }
    fn set_name(&mut self, self_: Resource<AssetNode>, value: String) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn asset(&mut self, self_: Resource<AssetNode>) -> wasm_bridge::Result<Option<Asset>> {
        todo!();
    }
    fn set_asset(
        &mut self,
        self_: Resource<AssetNode>,
        value: Option<Asset>,
    ) -> wasm_bridge::Result<()> {
        todo!()
    }

    fn parent(
        &mut self,
        self_: Resource<AssetNode>,
    ) -> wasm_bridge::Result<Option<Resource<AssetNode>>> {
        todo!()
    }
    fn children(
        &mut self,
        self_: Resource<AssetNode>,
    ) -> wasm_bridge::Result<Vec<Resource<AssetNode>>> {
        todo!()
    }
    fn add_child(
        &mut self,
        self_: Resource<AssetNode>,
        value: Resource<AssetNode>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }
    fn remove_child(
        &mut self,
        self_: Resource<AssetNode>,
        value: Resource<AssetNode>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn global_transform(&mut self, self_: Resource<AssetNode>) -> wasm_bridge::Result<Transform> {
        todo!();
    }
    fn transform(&mut self, self_: Resource<AssetNode>) -> wasm_bridge::Result<Transform> {
        todo!();
    }
    fn set_transform(
        &mut self,
        self_: Resource<AssetNode>,
        value: Transform,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn drop(&mut self, rep: Resource<AssetNode>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

impl Host for ScriptData {}
