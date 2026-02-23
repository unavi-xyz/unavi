use wasmtime::component::Linker;

use crate::load::state::StoreState;

pub mod scene;

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> anyhow::Result<()> {
    // scene::wired::scene::types::add_to_linker(linker, |s| &mut s.rt)?;

    Ok(())
}
