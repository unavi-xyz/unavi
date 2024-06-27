use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::input::handler::Host;

use crate::state::StoreState;

pub mod handler;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-input",
    world: "host",
    with: {
        "wired:gltf/material/material": super::wired_gltf::bindgen::Material,
        "wired:gltf/mesh/mesh": super::wired_gltf::bindgen::Mesh,
        "wired:gltf/mesh/primitive": super::wired_gltf::bindgen::Primitive,
        "wired:gltf/node/node": super::wired_gltf::bindgen::Node,
        "wired:input/handler/spatial-handler": handler::SpatialHandler,
    }
});

impl Host for StoreState {}

pub fn add_to_host(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::input::handler::add_to_linker(linker, |s| s)?;
    Ok(())
}
