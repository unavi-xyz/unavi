use wasm_bridge::component::Resource;

use crate::data::StoreData;

use super::bindings::dwn::{
    Dwn, Host, HostDwn, HostQuery, HostQueryBuilder, Query, QueryBuilder, QueryReply,
};

impl Host for StoreData {}

impl HostQueryBuilder for StoreData {
    fn record_id(&mut self, self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_record_id(
        &mut self,
        self_: Resource<QueryBuilder>,
        value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn schema(&mut self, self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_schema(
        &mut self,
        self_: Resource<QueryBuilder>,
        value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn protocol(&mut self, self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_protocol(
        &mut self,
        self_: Resource<QueryBuilder>,
        value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn run(&mut self, self_: Resource<QueryBuilder>) -> wasm_bridge::Result<Resource<Query>> {
        todo!();
    }

    fn drop(&mut self, rep: Resource<QueryBuilder>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

impl HostQuery for StoreData {
    fn poll(&mut self, self_: Resource<Query>) -> wasm_bridge::Result<Option<QueryReply>> {
        todo!();
    }

    fn finished(&mut self, self_: Resource<Query>) -> wasm_bridge::Result<bool> {
        todo!();
    }

    fn drop(&mut self, rep: Resource<Query>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

impl HostDwn for StoreData {
    fn query(&mut self, self_: Resource<Dwn>) -> wasm_bridge::Result<Resource<QueryBuilder>> {
        todo!();
    }

    fn drop(&mut self, rep: wasm_bridge::component::Resource<Dwn>) -> wasm_bridge::Result<()> {
        todo!();
    }
}
