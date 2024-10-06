use dwn::actor::Actor;
use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::bindings::dwn::{Host, HostDwn, RecordsQueryBuilder, RecordsWriteBuilder};

pub struct DwnRes {
    pub actor: Actor,
}

impl HostDwn for ScriptData {
    fn records_query(
        &mut self,
        _self_: Resource<DwnRes>,
    ) -> wasm_bridge::Result<Resource<RecordsQueryBuilder>> {
        todo!();
    }

    fn records_write(
        &mut self,
        _self_: Resource<DwnRes>,
    ) -> wasm_bridge::Result<Resource<RecordsWriteBuilder>> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<DwnRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl Host for ScriptData {}
