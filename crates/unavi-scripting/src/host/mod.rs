use anyhow::Result;
use wasm_component_layer::{Linker, Store};

use super::{load::EngineBackend, StoreData};

pub mod wired_log;

pub fn add_host_script_apis(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<()> {
    wired_log::add_to_host(store, linker)?;
    Ok(())
}
