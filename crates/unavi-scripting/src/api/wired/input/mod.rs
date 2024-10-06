use anyhow::Result;
use wasm_bridge::component::Linker;

use crate::data::ScriptData;

pub mod input_handler;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-input",
        world: "host",
        with: {
            "wired:math": crate::api::wired::math::bindings,
            "wired:input/handler/input-handler": super::input_handler::InputHandlerRes,
        }
    });

    pub use self::wired::input::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::wired::input::handler::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl bindings::handler::Host for ScriptData {}
