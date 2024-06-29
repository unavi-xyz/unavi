use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::physics::types::Host;

use crate::state::StoreState;

mod collider;
mod rigid_body;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-physics",
    world: "host",
    with: {
        "wired:physics/types/collider": collider::Collider,
        "wired:physics/types/rigid-body": rigid_body::RigidBody,
    }
});

impl Host for StoreState {}

pub fn add_to_host(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::physics::types::add_to_linker(linker, |s| s)?;
    Ok(())
}
