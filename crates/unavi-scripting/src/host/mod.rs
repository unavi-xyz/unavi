use anyhow::Result;
use wasm_component_layer::{Linker, Store};

use self::wired_gltf::{WiredGltfData, WiredGltfReceiver};

use super::{load::EngineBackend, StoreData};

pub mod wired_gltf;
mod wired_log;

pub struct HostScriptResults {
    pub wired_gltf_components: (WiredGltfReceiver, WiredGltfData),
}

pub fn add_host_script_apis(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<HostScriptResults> {
    let wired_gltf_components = wired_gltf::add_to_host(store, linker)?;
    wired_log::add_to_host(store, linker)?;

    Ok(HostScriptResults {
        wired_gltf_components,
    })
}
