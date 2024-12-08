use anyhow::Result;
use dwn::Dwn;
use wasm_bridge::component::{Linker, Resource};

use crate::data::ScriptData;

use self::{bindings::api::Host, res::DwnRes};

mod records_query;
mod records_write;
mod res;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-dwn",
        world: "host",
        with: {
            "wired:dwn/dwn/dwn": super::res::DwnRes,
            "wired:dwn/records-query/records-query": super::records_query::RecordsQuery,
            "wired:dwn/records-query/records-query-builder": super::records_query::RecordsQueryBuilder,
            "wired:dwn/records-write/records-write": super::records_write::RecordsWrite,
            "wired:dwn/records-write/records-write-builder": super::records_write::RecordsWriteBuilder,
        }
    });

    pub use self::wired::dwn::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::wired::dwn::api::add_to_linker(linker, |s| s)?;
    bindings::wired::dwn::dwn::add_to_linker(linker, |s| s)?;
    bindings::wired::dwn::records_query::add_to_linker(linker, |s| s)?;
    bindings::wired::dwn::records_write::add_to_linker(linker, |s| s)?;
    Ok(())
}

pub struct WiredDwn {
    dwn: Dwn,
}

impl WiredDwn {
    pub fn new(dwn: Dwn) -> Self {
        Self { dwn }
    }
}

impl Host for ScriptData {
    fn user_dwn(&mut self) -> wasm_bridge::Result<Resource<DwnRes>> {
        todo!();
        // let res = self.table.push(DwnRes { dwn })?;
        // Ok(res)
    }

    fn world_host_dwn(&mut self) -> wasm_bridge::Result<Resource<DwnRes>> {
        todo!();
    }
}
