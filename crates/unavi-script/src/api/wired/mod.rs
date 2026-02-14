use wasmtime::component::Linker;

use crate::load::state::StoreState;

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> anyhow::Result<()> {
    Ok(())
}
