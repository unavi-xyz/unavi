use anyhow::Result;
use bindings::dwn::{Dwn, Host, HostDwn, QueryBuilder};
use wasm_bridge::component::{Linker, Resource};

use crate::data::StoreData;

mod api;
mod query;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-dwn",
        world: "host",
    });

    pub use self::wired::dwn::*;
}

pub(crate) fn add_to_linker(linker: &mut Linker<StoreData>) -> Result<()> {
    bindings::wired::dwn::api::add_to_linker(linker, |s| s)?;
    bindings::wired::dwn::dwn::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl Host for StoreData {}

impl HostDwn for StoreData {
    fn query(&mut self, _self_: Resource<Dwn>) -> wasm_bridge::Result<Resource<QueryBuilder>> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<Dwn>) -> wasm_bridge::Result<()> {
        todo!();
    }
}
