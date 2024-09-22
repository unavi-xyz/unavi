use anyhow::Result;
use wasm_bridge::component::Linker;

use crate::data::StoreData;

mod api;
mod dwn;

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
