use anyhow::Result;
use wasm_bridge::component::Linker;

use crate::data::ScriptData;

pub mod collider;
pub mod rigid_body;
pub(crate) mod systems;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-physics",
        world: "host",
        with: {
            "wired:math": crate::api::wired::math::bindings,
            "wired:physics/types/collider": super::collider::ColliderRes,
            "wired:physics/types/rigid-body": super::rigid_body::RigidBodyRes,
        }
    });

    pub use self::wired::physics::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::wired::physics::types::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl bindings::types::Host for ScriptData {}
