use std::sync::mpsc::Receiver;

use anyhow::Result;
use wasm_component_layer::{Linker, Store};

use self::wired_ecs::WiredEcsCommand;

use super::{load::EngineBackend, StoreData};

pub mod wired_ecs;

pub struct HostApiReceivers {
    pub wired_ecs: Receiver<WiredEcsCommand>,
}

pub fn add_host_script_apis(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<HostApiReceivers> {
    let wired_ecs = wired_ecs::add_to_host(store, linker)?;
    Ok(HostApiReceivers { wired_ecs })
}
