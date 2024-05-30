use anyhow::Result;
use crossbeam::channel::Sender;
use wasm_component_layer::{Linker, Store};

use crate::{load::EngineBackend, StoreData};

use super::WiredGltfAction;

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    sender: Sender<WiredGltfAction>,
) -> Result<()> {
    let interface = linker.define_instance("wired:gltf/mesh".try_into()?)?;

    Ok(())
}
