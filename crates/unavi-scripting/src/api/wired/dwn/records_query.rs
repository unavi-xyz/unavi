use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::bindings::records_query::{
    Host, HostRecordsQuery, HostRecordsQueryBuilder, RecordsQueryReply,
};

pub struct RecordsQuery;
pub struct RecordsQueryBuilder;

impl Host for ScriptData {}

impl HostRecordsQueryBuilder for ScriptData {
    fn record_id(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
    ) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_record_id(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn schema(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
    ) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_schema(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn protocol(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
    ) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_protocol(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn run(
        &mut self,
        _self_: Resource<RecordsQueryBuilder>,
    ) -> wasm_bridge::Result<Resource<RecordsQuery>> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<RecordsQueryBuilder>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

impl HostRecordsQuery for ScriptData {
    fn poll(
        &mut self,
        _self_: Resource<RecordsQuery>,
    ) -> wasm_bridge::Result<Option<RecordsQueryReply>> {
        todo!();
    }

    fn finished(&mut self, _self_: Resource<RecordsQuery>) -> wasm_bridge::Result<bool> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<RecordsQuery>) -> wasm_bridge::Result<()> {
        todo!();
    }
}
