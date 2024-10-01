use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::bindings::records_write::{
    Host, HostRecordsWrite, HostRecordsWriteBuilder, RecordsWriteReply,
};

pub struct RecordsWrite;
pub struct RecordsWriteBuilder;

impl Host for ScriptData {}

impl HostRecordsWriteBuilder for ScriptData {
    fn record_id(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
    ) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_record_id(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn schema(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
    ) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_schema(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn protocol(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
    ) -> wasm_bridge::Result<Option<String>> {
        todo!();
    }
    fn set_protocol(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
        _value: Option<String>,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn run(
        &mut self,
        _self_: Resource<RecordsWriteBuilder>,
    ) -> wasm_bridge::Result<Resource<RecordsWrite>> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<RecordsWriteBuilder>) -> wasm_bridge::Result<()> {
        todo!();
    }
}

impl HostRecordsWrite for ScriptData {
    fn poll(
        &mut self,
        _self_: Resource<RecordsWrite>,
    ) -> wasm_bridge::Result<Option<RecordsWriteReply>> {
        todo!();
    }

    fn finished(&mut self, _self_: Resource<RecordsWrite>) -> wasm_bridge::Result<bool> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<RecordsWrite>) -> wasm_bridge::Result<()> {
        todo!();
    }
}
