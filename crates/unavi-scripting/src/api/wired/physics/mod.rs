use anyhow::Result;
use wasm_bridge::component::Linker;

use crate::state::StoreState;

mod collider;
mod rigid_body;
pub(crate) mod systems;

pub mod bindings {
    pub use super::{collider::Collider, rigid_body::RigidBody};

    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-physics",
        world: "host",
        with: {
            "wired:math": crate::api::wired::math::bindings,
            "wired:physics/types/collider": Collider,
            "wired:physics/types/rigid-body": RigidBody,
        }
    });

    pub use self::wired::physics::*;
}

impl bindings::types::Host for StoreState {}

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    bindings::wired::physics::types::add_to_linker(linker, |s| s)?;
    Ok(())
}
