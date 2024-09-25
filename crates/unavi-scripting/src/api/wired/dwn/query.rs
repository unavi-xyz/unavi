use wasm_bridge::component::Resource;

use crate::data::StoreData;

use super::bindings::dwn::{HostQuery, HostQueryBuilder, Query, QueryBuilder, QueryReply};

impl HostQueryBuilder for StoreData {
    fn record_id(&mut self, _self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_record_id(
        &mut self,
        _self_: Resource<QueryBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn schema(&mut self, _self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_schema(
        &mut self,
        _self_: Resource<QueryBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn protocol(&mut self, _self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_protocol(
        &mut self,
        _self_: Resource<QueryBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn run(&mut self, _self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Resource<Query>> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<QueryBuilder>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

impl HostQuery for StoreData {
    fn poll(&mut self, _self_: Resource<Query>) -> wasm_bridge::Result<Option<QueryReply>> {
        todo!();
    }

    fn finished(&mut self, _self_: Resource<Query>) -> wasm_bridge::Result<bool> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<Query>) -> wasm_bridge::Result<()> {
        todo!();
    }
}
