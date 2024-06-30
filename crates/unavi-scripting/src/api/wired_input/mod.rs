use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::input::handler::Host;

use crate::state::StoreState;

pub mod spatial_handler;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-input",
    world: "host",
    with: {
        "wired:scene/material/material": super::wired_scene::material::Material,
        "wired:scene/mesh/mesh": super::wired_scene::mesh::Mesh,
        "wired:scene/mesh/primitive": super::wired_scene::mesh::Primitive,
        "wired:scene/node/node": super::wired_scene::node::Node,
        "wired:input/handler/spatial-handler": spatial_handler::SpatialHandler,
    }
});

impl Host for StoreState {}

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::input::handler::add_to_linker(linker, |s| s)?;
    Ok(())
}
