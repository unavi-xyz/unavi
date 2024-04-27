use anyhow::Result;
use wasm_component_layer::{Linker, Store};

use self::wired_ecs::WiredEcsReceiver;

use super::{load::EngineBackend, StoreData};

pub mod wired_ecs;
pub mod wired_log;

pub fn add_host_script_apis(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<WiredEcsReceiver> {
    wired_log::add_to_host(store, linker)?;
    wired_ecs::add_to_host(store, linker)
}
