use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::input::handler::Host;

use crate::state::StoreState;

pub mod spatial_handler;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-input",
    world: "host",
    with: {
        "wired:gltf/material/material": super::wired_gltf::material::Material,
        "wired:gltf/mesh/mesh": super::wired_gltf::mesh::Mesh,
        "wired:gltf/mesh/primitive": super::wired_gltf::mesh::Primitive,
        "wired:gltf/node/node": super::wired_gltf::node::Node,
        "wired:input/handler/spatial-handler": spatial_handler::SpatialHandler,
    }
});

impl Host for StoreState {}

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::input::handler::add_to_linker(linker, |s| s)?;
    Ok(())
}
