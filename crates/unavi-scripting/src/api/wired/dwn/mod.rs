use anyhow::Result;
use bindings::dwn::{Host, HostDwn};
use dwn::{actor::Actor, DWN};
use wasm_bridge::component::{Linker, Resource};

use crate::data::ScriptData;

mod records_query;
mod records_write;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-dwn",
        world: "host",
        with: {
            "wired:dwn/dwn/dwn": super::DwnRes,
            "wired:dwn/records-query/records-query": super::records_query::RecordsQuery,
            "wired:dwn/records-query/records-query-builder": super::records_query::RecordsQueryBuilder,
            "wired:dwn/records-write/records-write": super::records_write::RecordsWrite,
            "wired:dwn/records-write/records-write-builder": super::records_write::RecordsWriteBuilder,
        }
    });

    pub use self::wired::dwn::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::wired::dwn::dwn::add_to_linker(linker, |s| s)?;
    bindings::wired::dwn::records_query::add_to_linker(linker, |s| s)?;
    bindings::wired::dwn::records_write::add_to_linker(linker, |s| s)?;
    Ok(())
}

pub struct WiredDwn {
    dwn: DWN,
}

impl WiredDwn {
    pub fn new(dwn: DWN) -> Self {
        Self { dwn }
    }
}

pub struct DwnRes {
    actor: Actor,
}

impl Host for ScriptData {
    fn local_dwn(&mut self) -> wasm_bridge::Result<Resource<DwnRes>> {
        let actor = Actor::new_did_key(self.api.wired_dwn.as_ref().unwrap().dwn.clone())?;
        let res = self.table.push(DwnRes { actor })?;
        Ok(res)
    }

    fn world_host_dwn(&mut self) -> wasm_bridge::Result<Resource<DwnRes>> {
        let actor = Actor::new_did_key(self.api.wired_dwn.as_ref().unwrap().dwn.clone())?;
        let res = self.table.push(DwnRes { actor })?;
        Ok(res)
    }
}

impl HostDwn for ScriptData {
    fn records_query(
        &mut self,
        _self_: Resource<DwnRes>,
    ) -> wasm_bridge::Result<Resource<records_query::RecordsQueryBuilder>> {
        todo!();
    }

    fn records_write(
        &mut self,
        _self_: Resource<DwnRes>,
    ) -> wasm_bridge::Result<Resource<records_write::RecordsWriteBuilder>> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<DwnRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
