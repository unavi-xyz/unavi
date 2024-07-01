use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::input::handler::Host;

use crate::state::StoreState;

pub mod input_handler;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-input",
    world: "host",
    with: {
        "wired:input/handler/input-handler": input_handler::InputHandler,
    }
});

impl Host for StoreState {}

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::input::handler::add_to_linker(linker, |s| s)?;
    Ok(())
}
