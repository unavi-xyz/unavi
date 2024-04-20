use anyhow::Result;
use wasm_component_layer::{Linker, Store};

use super::load::{EngineBackend, ScriptData};

pub mod wired_ecs;

pub fn add_host_script_apis(
    store: &mut Store<ScriptData, EngineBackend>,
    linker: &mut Linker,
) -> Result<()> {
    wired_ecs::add_to_host(store, linker)?;
    Ok(())
}
