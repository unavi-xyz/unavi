use wasm_bridge::component::Linker;

use super::state::ScriptState;

pub mod wired_ecs;

pub fn add_to_linker(linker: &mut Linker<ScriptState>) -> Result<(), wasm_bridge::Error> {
    wired_ecs::wired::ecs::types::add_to_linker(linker, |state| state)?;
    Ok(())
}
