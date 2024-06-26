use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::math::types::Host;

use crate::state::StoreState;

pub mod quat;
pub mod transform;
pub mod vec3;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-math",
    world: "host",
    with: {
        "wired:math/types/vec3": vec3::Vec3Res,
        "wired:math/types/quat": quat::QuatRes,
        "wired:math/types/transform": transform::Transform,
    }
});

impl Host for StoreState {}

pub fn add_to_host(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::math::types::add_to_linker(linker, |s| s)?;
    Ok(())
}
