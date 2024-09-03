use anyhow::Result;
use wasm_bridge::component::Linker;

use crate::state::StoreState;

pub mod input_handler;

pub mod bindings {
    pub use super::input_handler::InputHandler;

    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-input",
        world: "host",
        with: {
            "wired:math": crate::api::wired::math::bindings,
            "wired:input/handler/input-handler": InputHandler,
        }
    });

    pub use self::wired::input::*;
}

impl bindings::handler::Host for StoreState {}

pub(crate) fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    bindings::wired::input::handler::add_to_linker(linker, |s| s)?;
    Ok(())
}
